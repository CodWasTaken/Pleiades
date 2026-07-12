use async_trait::async_trait;
use pleiades_core::error::Error;
use pleiades_core::tool::{PermissionLevel, Tool, ToolContext, ToolResult};
use serde_json::Value;

/// Tool for fetching URLs and returning their content.
pub struct FetchTool {
    http_client: reqwest::Client,
}

impl FetchTool {
    pub fn new() -> Self {
        Self {
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .user_agent("Pleiades/0.1")
                .danger_accept_invalid_certs(false)
                .build()
                .expect("Failed to create HTTP client"),
        }
    }
}

#[async_trait]
impl Tool for FetchTool {
    fn name(&self) -> &str {
        "fetch"
    }

    fn description(&self) -> &str {
        "Fetch the contents of a URL. Returns the response body as text, with metadata about content type and size."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to fetch"
                },
                "method": {
                    "type": "string",
                    "description": "HTTP method (GET, POST, etc.)",
                    "enum": ["GET", "POST", "PUT", "DELETE", "HEAD"],
                    "default": "GET"
                },
                "headers": {
                    "type": "object",
                    "description": "Optional HTTP headers to include",
                    "additionalProperties": {"type": "string"}
                },
                "body": {
                    "type": "string",
                    "description": "Request body (for POST/PUT)"
                },
                "max_size": {
                    "type": "integer",
                    "description": "Maximum response size in bytes (default: 1048576 = 1MB)",
                    "default": 1048576
                }
            },
            "required": ["url"]
        })
    }

    fn is_readonly(&self) -> bool {
        true
    }

    fn is_concurrency_safe(&self) -> bool {
        true
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::ReadOnly
    }

    async fn execute(&self, input: Value, _ctx: &ToolContext) -> Result<ToolResult, Error> {
        let url = input.get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::invalid_input("Missing required field 'url'"))?;

        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(Error::invalid_input(format!("Invalid URL '{}'. Must start with http:// or https://", url)));
        }

        let method = input.get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("GET")
            .to_uppercase();

        let max_size = input.get("max_size")
            .and_then(|v| v.as_i64())
            .unwrap_or(1_048_576) as usize;

        let mut req = match method.as_str() {
            "GET" => self.http_client.get(url),
            "POST" => {
                let body = input.get("body").and_then(|v| v.as_str()).unwrap_or("");
                self.http_client.post(url).body(body.to_string())
            }
            "PUT" => {
                let body = input.get("body").and_then(|v| v.as_str()).unwrap_or("");
                self.http_client.put(url).body(body.to_string())
            }
            "DELETE" => self.http_client.delete(url),
            "HEAD" => self.http_client.head(url),
            _ => return Err(Error::invalid_input(format!("Unsupported method: {}", method))),
        };

        if let Some(headers) = input.get("headers").and_then(|v| v.as_object()) {
            for (key, value) in headers {
                if let Some(val) = value.as_str() {
                    if let (Ok(k), Ok(v)) = (
                        reqwest::header::HeaderName::from_bytes(key.as_bytes()),
                        reqwest::header::HeaderValue::from_str(val),
                    ) {
                        req = req.header(k, v);
                    }
                }
            }
        }

        let response = req.send().await
                .map_err(|e| Error::Network(format!("Request failed: {}", e)))?;

        let status = response.status();
        let headers = response.headers().clone();
        let content_type = headers
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown")
            .to_string();

        let body_bytes = response.bytes().await
                .map_err(|e| Error::Network(format!("Failed to read response body: {}", e)))?;

        let size = body_bytes.len();

        if size > max_size {
            return Ok(ToolResult {
                success: true,
                content: format!(
                    "Response too large: {} bytes (max: {}). Showing first {} bytes:\n\n{}",
                    size, max_size, max_size,
                    String::from_utf8_lossy(&body_bytes[..max_size])
                ),
                error: None,
                metadata: Some(serde_json::json!({
                    "url": url,
                    "status": status.as_u16(),
                    "content_type": content_type,
                    "size": size,
                    "truncated": true
                })),
            });
        }

        let body_text = String::from_utf8_lossy(&body_bytes).to_string();

        Ok(ToolResult {
            success: true,
            content: body_text,
            error: None,
            metadata: Some(serde_json::json!({
                "url": url,
                "status": status.as_u16(),
                "content_type": content_type,
                "size": size,
                "truncated": false
            })),
        })
    }
}

impl Default for FetchTool {
    fn default() -> Self {
        Self::new()
    }
}
