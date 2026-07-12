use async_trait::async_trait;
use pleiades_core::error::Error;
use pleiades_core::model::ModelInfo;
use pleiades_core::provider::{
    ChatRequest, ChatResponse, Provider, ProviderCapabilities, StreamEvent,
};

/// Generic OpenAI-compatible API provider.
///
/// This provider works with any API that follows the OpenAI chat completions format,
/// including: OpenRouter, Groq, Together AI, DeepSeek, xAI, Perplexity, etc.
#[allow(dead_code)]
pub struct OpenAICompatibleProvider {
    name: String,
    display_name: String,
    api_key: String,
    base_url: String,
    default_model: String,
}

impl OpenAICompatibleProvider {
    pub fn new(
        name: impl Into<String>,
        display_name: impl Into<String>,
        api_key: impl Into<String>,
        base_url: impl Into<String>,
        default_model: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            display_name: display_name.into(),
            api_key: api_key.into(),
            base_url: base_url.into(),
            default_model: default_model.into(),
        }
    }
}

#[async_trait]
impl Provider for OpenAICompatibleProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn display_name(&self) -> &str {
        &self.display_name
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
        &self.default_model
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, Error> {
        Err(Error::NotImplemented(format!("{} model listing not yet implemented", self.name)))
    }

    async fn chat(&self, _request: ChatRequest) -> Result<ChatResponse, Error> {
        Err(Error::NotImplemented(format!("{} chat not yet implemented", self.name)))
    }

    async fn chat_stream(
        &self,
        _request: ChatRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<StreamEvent>, Error> {
        Err(Error::NotImplemented(format!("{} streaming not yet implemented", self.name)))
    }
}
