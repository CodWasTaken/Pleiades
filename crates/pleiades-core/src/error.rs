use thiserror::Error;

/// Unified error type for Pleiades.
///
/// All errors in the system use this type, ensuring consistent
/// error handling and reporting throughout the application.
#[derive(Error, Debug)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Provider error: {0}")]
    Provider(String),

    #[error("Provider '{provider}' not found")]
    ProviderNotFound { provider: String },

    #[error("Model '{model}' not found")]
    ModelNotFound { model: String },

    #[error("API error ({status}): {message}")]
    ApiError {
        status: u16,
        message: String,
        provider: String,
    },

    #[error("Network error: {0}")]
    Network(String),

    #[error("Authentication failed for provider '{provider}': {message}")]
    AuthError { provider: String, message: String },

    #[error("Rate limited by provider '{provider}'. Retry after {retry_after:?}")]
    RateLimited {
        provider: String,
        retry_after: Option<u64>,
    },

    #[error("Tool execution error: {0}")]
    ToolError(String),

    #[error("Tool '{name}' not found")]
    ToolNotFound { name: String },

    #[error("Tool '{name}' permission denied")]
    ToolPermissionDenied { name: String, level: String },

    #[error("Plugin error: {0}")]
    Plugin(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("IO error: {0}")]
    Io(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Unsupported operation: {0}")]
    Unsupported(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl Error {
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    pub fn provider(msg: impl Into<String>) -> Self {
        Self::Provider(msg.into())
    }

    pub fn tool(msg: impl Into<String>) -> Self {
        Self::ToolError(msg.into())
    }

    pub fn plugin(msg: impl Into<String>) -> Self {
        Self::Plugin(msg.into())
    }

    pub fn unsupported(msg: impl Into<String>) -> Self {
        Self::Unsupported(msg.into())
    }

    pub fn io(msg: impl Into<String>) -> Self {
        Self::Io(msg.into())
    }

    pub fn invalid_input(msg: impl Into<String>) -> Self {
        Self::InvalidInput(msg.into())
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    pub fn timeout(msg: impl Into<String>) -> Self {
        Self::Timeout(msg.into())
    }

    /// Whether this error is retryable.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::RateLimited { .. }
                | Self::Network(_)
                | Self::Timeout(_)
                | Self::ApiError {
                    status: 429 | 500 | 502 | 503 | 504,
                    ..
                }
        )
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for Error {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::Internal(err.to_string())
    }
}
