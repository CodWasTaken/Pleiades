use async_trait::async_trait;
use pleiades_core::error::Error;
use pleiades_core::model::ModelInfo;
use pleiades_core::provider::{
    ChatRequest, ChatResponse, EmbeddingResponse, Provider, ProviderCapabilities, StreamEvent,
};

/// OpenAI provider implementation.
#[allow(dead_code)]
pub struct OpenAIProvider {
    api_key: String,
    base_url: String,
}

impl OpenAIProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: "https://api.openai.com/v1".to_string(),
        }
    }

    pub fn with_base_url(api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: base_url.into(),
        }
    }
}

#[async_trait]
impl Provider for OpenAIProvider {
    fn name(&self) -> &str {
        "openai"
    }

    fn display_name(&self) -> &str {
        "OpenAI"
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            streaming: true,
            tools: true,
            vision: true,
            embeddings: true,
            thinking: false,
            json_mode: true,
            function_calling: true,
        }
    }

    fn default_model(&self) -> &str {
        "gpt-4o"
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, Error> {
        Err(Error::NotImplemented("OpenAI model listing not yet implemented".to_string()))
    }

    async fn chat(&self, _request: ChatRequest) -> Result<ChatResponse, Error> {
        Err(Error::NotImplemented("OpenAI chat not yet implemented".to_string()))
    }

    async fn chat_stream(
        &self,
        _request: ChatRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<StreamEvent>, Error> {
        Err(Error::NotImplemented("OpenAI streaming not yet implemented".to_string()))
    }

    async fn embed(&self, _input: Vec<String>, _model: &str) -> Result<EmbeddingResponse, Error> {
        Err(Error::NotImplemented("OpenAI embeddings not yet implemented".to_string()))
    }
}
