use async_trait::async_trait;
use pleiades_core::error::Error;
use pleiades_core::tool::{PermissionLevel, Tool, ToolContext, ToolResult};
use serde_json::Value;

/// Tool for searching the web.
///
/// Uses the DuckDuckGo Instant Answer API for simple searches
/// (no API key required). For more advanced searches, users
/// can configure a custom search endpoint.
pub struct SearchTool {
    api_endpoint: String,
    http_client: reqwest::Client,
}

impl SearchTool {
    pub fn new() -> Self {
        Self {
            api_endpoint: "https://api.duckduckgo.com".to_string(),
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .user_agent("Pleiades/0.1 (AI assistant)")
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    pub fn with_endpoint(endpoint: impl Into<String>) -> Self {
        Self {
            api_endpoint: endpoint.into(),
            http_client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl Tool for SearchTool {
    fn name(&self) -> &str {
        "search"
    }

    fn description(&self) -> &str {
        "Search the web for information. Returns a list of relevant results with titles, URLs, and snippets."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query"
                },
                "max_results": {
                    "type": "integer",
                    "description": "Maximum number of results to return (default: 5)",
                    "default": 5
                }
            },
            "required": ["query"]
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
        let query = input
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::invalid_input("Missing required field 'query'"))?;

        let max_results = input
            .get("max_results")
            .and_then(|v| v.as_i64())
            .unwrap_or(5) as usize;

        // Try DuckDuckGo Instant Answer API first
        let url = format!(
            "{}/?q={}&format=json&no_html=1&skip_disambig=1",
            self.api_endpoint,
            urlencoding(query)
        );

        let response = self
            .http_client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| Error::Network(format!("Search request failed: {}", e)))?;

        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        if !status.is_success() {
            return Err(Error::ApiError {
                status: status.as_u16(),
                message: "Search API error".to_string(),
                provider: "search".to_string(),
            });
        }

        let result = self.parse_duckduckgo_response(&body, max_results)?;
        Ok(result)
    }
}

impl SearchTool {
    fn parse_duckduckgo_response(
        &self,
        body: &str,
        max_results: usize,
    ) -> Result<ToolResult, Error> {
        let parsed: serde_json::Value = serde_json::from_str(body)
            .map_err(|e| Error::Serialization(format!("Failed to parse search response: {}", e)))?;

        let mut results = Vec::new();
        let mut sources = Vec::new();

        // Abstract (if present, it's an instant answer)
        if let Some(abstract_text) = parsed.get("AbstractText").and_then(|v| v.as_str()) {
            if !abstract_text.is_empty() {
                if let Some(source) = parsed.get("AbstractSource").and_then(|v| v.as_str()) {
                    if !source.is_empty() {
                        if let Some(url) = parsed.get("AbstractURL").and_then(|v| v.as_str()) {
                            results.push(format!(
                                "**{}**\n{}\nSource: {}",
                                source, abstract_text, url
                            ));
                            sources.push(source.to_string());
                        }
                    }
                }
            }
        }

        // Answer (e.g., for calculations)
        if let Some(answer) = parsed.get("Answer").and_then(|v| v.as_str()) {
            if !answer.is_empty() && results.len() < max_results {
                let answer_type = parsed
                    .get("AnswerType")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                results.push(format!("**{}**\nAnswer: {}", answer_type, answer));
            }
        }

        // Related topics (from DuckDuckGo API)
        if let Some(related) = parsed.get("RelatedTopics").and_then(|v| v.as_array()) {
            for topic in related {
                if results.len() >= max_results {
                    break;
                }
                if let Some(text) = topic.get("Text").and_then(|v| v.as_str()) {
                    if let Some(url) = topic.get("FirstURL").and_then(|v| v.as_str()) {
                        results.push(format!("- {}\n  {}", text, url));
                    } else {
                        results.push(format!("- {}", text));
                    }
                }
                // Handle topics with sub-topics
                if let Some(topics) = topic.get("Topics").and_then(|v| v.as_array()) {
                    for sub in topics {
                        if results.len() >= max_results {
                            break;
                        }
                        if let Some(text) = sub.get("Text").and_then(|v| v.as_str()) {
                            if let Some(url) = sub.get("FirstURL").and_then(|v| v.as_str()) {
                                results.push(format!("- {}\n  {}", text, url));
                            }
                        }
                    }
                }
            }
        }

        if results.is_empty() {
            return Ok(ToolResult {
                success: true,
                content: "No search results found.".to_string(),
                error: None,
                metadata: Some(serde_json::json!({
                    "query": body,
                    "result_count": 0
                })),
            });
        }

        let summary = format!("Search results for '{}':\n\n{}", "", results.join("\n\n"));

        Ok(ToolResult {
            success: true,
            content: summary,
            error: None,
            metadata: Some(serde_json::json!({
                "result_count": results.len(),
                "sources": sources
            })),
        })
    }
}

impl Default for SearchTool {
    fn default() -> Self {
        Self::new()
    }
}

fn urlencoding(query: &str) -> String {
    query
        .chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => "+".to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}
