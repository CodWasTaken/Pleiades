//! Shared HTTP client utilities for provider implementations.

use std::time::Duration;

use pleiades_core::error::Error;

/// Send an HTTP request with retry logic for rate limits and transient errors.
pub async fn send_request(
    _client: &reqwest::Client,
    request: reqwest::RequestBuilder,
    provider_name: &str,
) -> Result<reqwest::Response, Error> {
    let max_retries = 3;
    let mut attempt = 0;

    loop {
        let req = request
            .try_clone()
            .ok_or_else(|| Error::internal("Failed to clone request for retry"))?;

        match req.send().await {
            Ok(response) => {
                let status = response.status();
                if status.is_success() {
                    return Ok(response);
                }

                let body = response.text().await.unwrap_or_default();

                if status.as_u16() == 429 {
                    if attempt < max_retries {
                        let retry_after = parse_retry_after(&body).unwrap_or(2_u64.pow(attempt));
                        tokio::time::sleep(Duration::from_secs(retry_after)).await;
                        attempt += 1;
                        continue;
                    }
                    return Err(Error::RateLimited {
                        provider: provider_name.to_string(),
                        retry_after: parse_retry_after(&body),
                    });
                }

                if status.as_u16() >= 500 && attempt < max_retries {
                    tokio::time::sleep(Duration::from_secs(2_u64.pow(attempt))).await;
                    attempt += 1;
                    continue;
                }

                return Err(map_api_error(status.as_u16(), &body, provider_name));
            }
            Err(e) => {
                if e.is_timeout() && attempt < max_retries {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    attempt += 1;
                    continue;
                }
                if e.is_connect() && attempt < max_retries {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    attempt += 1;
                    continue;
                }
                return Err(Error::Network(e.to_string()));
            }
        }
    }
}

/// Parse a retry-after value from the response body or headers.
fn parse_retry_after(body: &str) -> Option<u64> {
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(body) {
        if let Some(seconds) = val.get("error").and_then(|e| e.get("retry_after")).and_then(|v| v.as_u64()) {
            return Some(seconds);
        }
    }
    None
}

/// Map HTTP API errors to core error types.
pub fn map_api_error(status: u16, body: &str, provider_name: &str) -> Error {
    let message = extract_error_message(body).unwrap_or_else(|| body.to_string());

    match status {
        401 | 403 => Error::AuthError {
            provider: provider_name.to_string(),
            message,
        },
        404 => Error::Provider(format!("{} resource not found: {}", provider_name, message)),
        429 => Error::RateLimited {
            provider: provider_name.to_string(),
            retry_after: None,
        },
        400..=499 => Error::ApiError {
            status,
            message,
            provider: provider_name.to_string(),
        },
        500..=599 => Error::ApiError {
            status,
            message,
            provider: provider_name.to_string(),
        },
        _ => Error::ApiError {
            status,
            message,
            provider: provider_name.to_string(),
        },
    }
}

/// Extract error message from common API error response formats.
fn extract_error_message(body: &str) -> Option<String> {
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(body) {
        if let Some(msg) = val.get("error").and_then(|e| e.get("message")).and_then(|v| v.as_str()) {
            return Some(msg.to_string());
        }
        if let Some(msg) = val.get("error").and_then(|e| e.as_str()) {
            return Some(msg.to_string());
        }
        if let Some(msg) = val.get("message").and_then(|v| v.as_str()) {
            return Some(msg.to_string());
        }
    }
    None
}

/// Parse an SSE (Server-Sent Events) stream from a response body.
///
/// Returns a channel receiver that yields parsed SSE events as `(event_type, data)` pairs.
pub fn parse_sse_stream(
    response: reqwest::Response,
) -> tokio::sync::mpsc::Receiver<Result<SseEvent, Error>> {
    let (tx, rx) = tokio::sync::mpsc::channel(256);

    tokio::spawn(async move {
        let mut buffer = String::new();
        let mut current_event = String::new();
        let mut current_data = String::new();

        use futures::StreamExt;
        let mut stream = response.bytes_stream();

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    buffer.push_str(&String::from_utf8_lossy(&chunk));

                    while let Some(line_end) = buffer.find('\n') {
                        let line = buffer[..line_end].to_string();
                        buffer = buffer[line_end + 1..].to_string();

                        if line.is_empty() {
                            if !current_data.is_empty() {
                                let _ = tx.send(Ok(SseEvent {
                                    event: current_event.clone(),
                                    data: current_data.clone(),
                                })).await;
                            }
                            current_event.clear();
                            current_data.clear();
                        } else if let Some(value) = line.strip_prefix("event:") {
                            current_event = value.trim().to_string();
                        } else if let Some(value) = line.strip_prefix("data:") {
                            if !current_data.is_empty() {
                                current_data.push('\n');
                            }
                            current_data.push_str(value.trim());
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(Error::Network(format!("SSE stream error: {}", e)))).await;
                    return;
                }
            }
        }

        if !current_data.is_empty() {
            let _ = tx.send(Ok(SseEvent {
                event: current_event.clone(),
                data: current_data.clone(),
            })).await;
        }
    });

    rx
}

/// A parsed SSE event.
#[derive(Debug, Clone)]
pub struct SseEvent {
    pub event: String,
    pub data: String,
}

impl SseEvent {
    /// Parse the data field as JSON.
    pub fn parse_json<T: serde::de::DeserializeOwned>(&self) -> Result<T, Error> {
        serde_json::from_str(&self.data).map_err(|e| {
            Error::Serialization(format!("Failed to parse SSE data as JSON: {}", e))
        })
    }
}

/// Build a default HTTP client with sensible timeouts.
pub fn default_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .connect_timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to create HTTP client")
}
