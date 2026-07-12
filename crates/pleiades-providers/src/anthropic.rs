use async_trait::async_trait;
use pleiades_core::error::Error;
use pleiades_core::model::ModelInfo;
use pleiades_core::provider::{
    ChatRequest, ChatResponse, Provider, ProviderCapabilities, StreamEvent,
};

/// Anthropic Claude provider implementation.
#[allow(dead_code)]
pub struct AnthropicProvider {
    api_key: String,
    base_url: String,
}

impl AnthropicProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: "https://api.anthropic.com/v1".to_string(),
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
impl Provider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    fn display_name(&self) -> &str {
        "Anthropic Claude"
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            streaming: true,
            tools: true,
            vision: true,
            embeddings: false,
            thinking: true,
            json_mode: false,
            function_calling: true,
        }
    }

    fn default_model(&self) -> &str {
        "claude-sonnet-4-20250514"
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, Error> {
        // Return known Claude models (Anthropic doesn't have a public model list endpoint)
        Ok(vec![
            ModelInfo {
                id: "claude-sonnet-4-20250514".to_string(),
                provider: "anthropic".to_string(),
                display_name: Some("Claude Sonnet 4".to_string()),
                description: Some("Best balance of speed and capability".to_string()),
                capabilities: pleiades_core::model::ModelCapabilities {
                    max_context_length: 200000,
                    max_output_tokens: 8192,
                    supports_tools: true,
                    supports_vision: true,
                    supports_streaming: true,
                    supports_thinking: true,
                    supports_json_mode: false,
                },
                pricing: Some(pleiades_core::model::Pricing {
                    input_per_million: 3.0,
                    output_per_million: 15.0,
                    cache_read_per_million: Some(0.30),
                    cache_write_per_million: Some(3.75),
                }),
            },
            ModelInfo {
                id: "claude-opus-4-20250514".to_string(),
                provider: "anthropic".to_string(),
                display_name: Some("Claude Opus 4".to_string()),
                description: Some("Most capable Claude model".to_string()),
                capabilities: pleiades_core::model::ModelCapabilities {
                    max_context_length: 200000,
                    max_output_tokens: 8192,
                    supports_tools: true,
                    supports_vision: true,
                    supports_streaming: true,
                    supports_thinking: true,
                    supports_json_mode: false,
                },
                pricing: Some(pleiades_core::model::Pricing {
                    input_per_million: 15.0,
                    output_per_million: 75.0,
                    cache_read_per_million: Some(1.50),
                    cache_write_per_million: Some(18.75),
                }),
            },
            ModelInfo {
                id: "claude-haiku-3-5-20241022".to_string(),
                provider: "anthropic".to_string(),
                display_name: Some("Claude Haiku 3.5".to_string()),
                description: Some("Fastest Claude model for simple tasks".to_string()),
                capabilities: pleiades_core::model::ModelCapabilities {
                    max_context_length: 200000,
                    max_output_tokens: 8192,
                    supports_tools: true,
                    supports_vision: true,
                    supports_streaming: true,
                    supports_thinking: true,
                    supports_json_mode: false,
                },
                pricing: Some(pleiades_core::model::Pricing {
                    input_per_million: 0.80,
                    output_per_million: 4.0,
                    cache_read_per_million: Some(0.08),
                    cache_write_per_million: Some(1.0),
                }),
            },
        ])
    }

    async fn chat(&self, _request: ChatRequest) -> Result<ChatResponse, Error> {
        Err(Error::NotImplemented("Anthropic provider chat not yet implemented".to_string()))
    }

    async fn chat_stream(
        &self,
        _request: ChatRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<StreamEvent>, Error> {
        Err(Error::NotImplemented("Anthropic provider streaming not yet implemented".to_string()))
    }
}
