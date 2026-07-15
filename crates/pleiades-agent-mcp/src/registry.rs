use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Transport-independent MCP server definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpServerDefinition {
    /// Stable local server ID.
    pub id: String,
    /// Whether the server is enabled.
    pub enabled: bool,
    /// Server transport.
    pub transport: McpTransportDefinition,
    /// Request timeout.
    pub timeout: Duration,
    /// Optional tool allowlist. Empty means all non-denied tools.
    pub tool_allowlist: Vec<String>,
    /// Tool denylist.
    pub tool_denylist: Vec<String>,
}

impl McpServerDefinition {
    /// Returns true when the server definition exposes the named tool.
    pub fn allows_tool(&self, tool: &str) -> bool {
        !self.tool_denylist.iter().any(|denied| denied == tool)
            && (self.tool_allowlist.is_empty()
                || self.tool_allowlist.iter().any(|allowed| allowed == tool))
    }
}

/// MCP transport definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "transport", rename_all = "kebab-case")]
pub enum McpTransportDefinition {
    /// Local stdio process.
    Stdio {
        /// Command to run.
        command: String,
        /// Command arguments.
        args: Vec<String>,
        /// Environment for the child process.
        env: HashMap<String, String>,
    },
    /// Remote HTTP endpoint.
    Http(RemoteMcpEndpoint),
    /// Streamable HTTP endpoint.
    StreamableHttp(RemoteMcpEndpoint),
}

impl McpTransportDefinition {
    /// Returns a redacted, user-facing transport label.
    pub fn redacted_label(&self) -> String {
        match self {
            Self::Stdio { command, args, .. } => {
                if args.is_empty() {
                    format!("stdio:{command}")
                } else {
                    format!("stdio:{} {}", command, args.join(" "))
                }
            }
            Self::Http(endpoint) => format!("http:{}", endpoint.redacted_url()),
            Self::StreamableHttp(endpoint) => {
                format!("streamable-http:{}", endpoint.redacted_url())
            }
        }
    }
}

/// Remote MCP endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RemoteMcpEndpoint {
    /// Endpoint URL.
    pub url: String,
    /// Optional auth source.
    pub auth: Option<McpAuthSource>,
}

impl RemoteMcpEndpoint {
    /// Returns a redacted URL suitable for logs and audit views.
    pub fn redacted_url(&self) -> String {
        redact_url_secret(&self.url)
    }
}

/// Authentication source. This type only stores names of secret sources, never
/// credential values.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum McpAuthSource {
    /// Bearer token read from an environment variable.
    Bearer { token_env: String },
    /// OAuth token read from environment variables.
    OAuth {
        client_id_env: Option<String>,
        token_env: String,
    },
}

/// Health state for an MCP server.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum McpServerHealth {
    /// Server has not been contacted.
    Unknown,
    /// Last health check succeeded.
    Healthy,
    /// Last health check failed.
    Unhealthy,
    /// Server is disabled by configuration.
    Disabled,
}

/// User-facing MCP server status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpServerStatus {
    /// Stable local server ID.
    pub id: String,
    /// Whether the server is enabled.
    pub enabled: bool,
    /// Redacted transport label.
    pub transport: String,
    /// Health state.
    pub health: McpServerHealth,
    /// Number of discovered tools, if known.
    pub tool_count: Option<usize>,
    /// Last error, if any.
    pub last_error: Option<String>,
    /// Last measured latency in milliseconds, if known.
    pub latency_ms: Option<u64>,
}

impl McpServerStatus {
    /// Create an initial status object from a server definition.
    pub fn from_definition(server: &McpServerDefinition) -> Self {
        Self {
            id: server.id.clone(),
            enabled: server.enabled,
            transport: server.transport.redacted_label(),
            health: if server.enabled {
                McpServerHealth::Unknown
            } else {
                McpServerHealth::Disabled
            },
            tool_count: None,
            last_error: None,
            latency_ms: None,
        }
    }
}

fn redact_url_secret(url: &str) -> String {
    let mut redacted = url.to_string();
    for marker in ["token=", "key=", "api_key=", "access_token="] {
        if let Some(index) = redacted.find(marker) {
            let value_start = index + marker.len();
            let value_end = redacted[value_start..]
                .find('&')
                .map(|offset| value_start + offset)
                .unwrap_or(redacted.len());
            redacted.replace_range(value_start..value_end, "REDACTED");
        }
    }
    redacted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_filters_use_allow_then_deny() {
        let server = McpServerDefinition {
            id: "test".to_string(),
            enabled: true,
            transport: McpTransportDefinition::Stdio {
                command: "server".to_string(),
                args: Vec::new(),
                env: HashMap::new(),
            },
            timeout: Duration::from_secs(30),
            tool_allowlist: vec!["read".to_string()],
            tool_denylist: vec!["write".to_string()],
        };

        assert!(server.allows_tool("read"));
        assert!(!server.allows_tool("write"));
        assert!(!server.allows_tool("search"));
    }

    #[test]
    fn status_redacts_url_query_secrets() {
        let server = McpServerDefinition {
            id: "remote".to_string(),
            enabled: true,
            transport: McpTransportDefinition::Http(RemoteMcpEndpoint {
                url: "https://example.test/mcp?token=secret&name=ok".to_string(),
                auth: Some(McpAuthSource::Bearer {
                    token_env: "MCP_TOKEN".to_string(),
                }),
            }),
            timeout: Duration::from_secs(30),
            tool_allowlist: Vec::new(),
            tool_denylist: Vec::new(),
        };

        let status = McpServerStatus::from_definition(&server);
        assert_eq!(
            status.transport,
            "http:https://example.test/mcp?token=REDACTED&name=ok"
        );
    }
}
