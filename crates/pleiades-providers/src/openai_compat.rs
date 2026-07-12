use async_trait::async_trait;
use pleiades_core::error::Error;
use pleiades_core::model::ModelInfo;
use pleiades_core::provider::{
    ChatRequest, ChatResponse, EmbeddingResponse, Provider, ProviderCapabilities, StreamEvent,
};

use crate::openai::OpenAIProvider;

/// Generic OpenAI-compatible API provider.
///
/// This provider delegates to the OpenAI implementation but with a
/// configurable name, display name, base URL, and default model.
/// Works with: OpenRouter, Groq, Together AI, DeepSeek, xAI, Perplexity, etc.
pub struct OpenAICompatibleProvider {
    name: String,
    display_name: String,
    inner: OpenAIProvider,
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
        let inner = OpenAIProvider::with_base_url(api_key, base_url);
        Self {
            name: name.into(),
            display_name: display_name.into(),
            inner,
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
            embeddings: false,
            thinking: false,
            json_mode: true,
            function_calling: true,
        }
    }

    fn default_model(&self) -> &str {
        &self.default_model
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, Error> {
        self.inner.list_models().await
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, Error> {
        self.inner.chat(request).await
    }

    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<StreamEvent>, Error> {
        self.inner.chat_stream(request).await
    }

    async fn embed(&self, input: Vec<String>, model: &str) -> Result<EmbeddingResponse, Error> {
        self.inner.embed(input, model).await
    }
}
