use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::conversation::Message;
use crate::error::Error;
use crate::model::ModelInfo;

/// Stream event emitted during chat streaming.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamEvent {
    Token(String),
    ReasoningToken(String),
    ToolCall {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    ToolResult {
        id: String,
        content: String,
    },
    Done {
        finish_reason: String,
        usage: Option<Usage>,
    },
    Error {
        message: String,
        code: Option<String>,
    },
}

/// Usage statistics for a chat request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: Option<u64>,
    pub cache_write_tokens: Option<u64>,
}

/// Chat request sent to a provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub system_prompt: Option<String>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub max_tokens: Option<u64>,
    pub stop: Option<Vec<String>>,
    pub tools: Option<Vec<crate::tool::ToolDefinition>>,
}

/// Chat response from a provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub message: Message,
    pub usage: Option<Usage>,
    pub finish_reason: Option<String>,
}

/// Embedding response from a provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    pub embeddings: Vec<Vec<f32>>,
    pub model: String,
    pub usage: Option<Usage>,
}

/// Capabilities supported by a provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCapabilities {
    pub streaming: bool,
    pub tools: bool,
    pub vision: bool,
    pub embeddings: bool,
    pub thinking: bool,
    pub json_mode: bool,
    pub function_calling: bool,
}

/// Generic provider interface for AI model access.
///
/// All AI providers implement this trait, making them interchangeable
/// within the Pleiades system.
#[async_trait]
pub trait Provider: Send + Sync {
    /// Unique identifier for this provider (e.g., "anthropic", "openai").
    fn name(&self) -> &str;

    /// Human-readable display name.
    fn display_name(&self) -> &str;

    /// Provider capabilities.
    fn capabilities(&self) -> ProviderCapabilities;

    /// Default model identifier for this provider.
    fn default_model(&self) -> &str;

    /// List available models from this provider.
    async fn list_models(&self) -> Result<Vec<ModelInfo>, Error>;

    /// Send a chat request and get a complete response.
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, Error>;

    /// Send a chat request and receive a stream of events.
    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<StreamEvent>, Error>;

    /// Generate embeddings for input texts.
    async fn embed(&self, input: Vec<String>, model: &str) -> Result<EmbeddingResponse, Error> {
        let _ = (input, model);
        Err(Error::unsupported(
            "Embeddings not supported by this provider",
        ))
    }
}
