use std::time::Duration;

/// MCP client result type.
pub type Result<T> = std::result::Result<T, McpError>;

/// Errors produced by MCP protocol and transport operations.
#[derive(Debug, thiserror::Error)]
pub enum McpError {
    /// The server definition is invalid.
    #[error("invalid MCP server '{server}': {message}")]
    InvalidServer { server: String, message: String },

    /// A JSON-RPC response contained an error object.
    #[error("MCP server returned JSON-RPC error {code}: {message}")]
    JsonRpc {
        code: i64,
        message: String,
        data: Option<serde_json::Value>,
    },

    /// The server returned a response that could not be matched to a request.
    #[error("unexpected MCP response: {0}")]
    UnexpectedResponse(String),

    /// The transport failed.
    #[error("MCP transport error: {0}")]
    Transport(String),

    /// The server did not respond within the configured timeout.
    #[error("MCP request timed out after {0:?}")]
    Timeout(Duration),

    /// I/O error from a process transport.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// JSON serialization or parsing error.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
