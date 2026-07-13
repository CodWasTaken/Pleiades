//! AI provider implementations for Pleiades.
//!
//! Each provider implements the `Provider` trait from `pleiades-agent-core`,
//! enabling seamless switching between AI backends.

pub mod anthropic;
pub mod client;
pub mod codex;
pub mod openai;
pub mod openai_compat;

use pleiades_agent_config::Config;
use pleiades_agent_core::provider::Provider;

/// Registry of available providers.
pub struct ProviderRegistry {
    providers: Vec<Box<dyn Provider>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    /// Register a provider.
    pub fn register(&mut self, provider: Box<dyn Provider>) {
        self.providers.push(provider);
    }

    /// Get a provider by name.
    pub fn get(&self, name: &str) -> Option<&dyn Provider> {
        self.providers
            .iter()
            .find(|p| p.name() == name)
            .map(|p| p.as_ref())
    }

    /// List all registered providers.
    pub fn list(&self) -> Vec<&dyn Provider> {
        self.providers.iter().map(|p| p.as_ref()).collect()
    }

    /// Build a registry from Pleiades configuration.
    ///
    /// Automatically creates providers for all configured API keys.
    pub fn from_config(config: &Config) -> Self {
        let mut registry = Self::new();

        if config.providers.contains_key("openai-subscription") {
            registry.register(Box::new(codex::CodexCliProvider::new()));
        }

        let register_builtin = |registry: &mut Self, name: &str, api_key: &str, base_url: &str| {
            if api_key.is_empty() {
                return;
            }
            let provider: Box<dyn Provider> = if base_url.is_empty() {
                match name {
                    "anthropic" => Box::new(anthropic::AnthropicProvider::new(api_key)),
                    "openai" => Box::new(openai::OpenAIProvider::new(api_key)),
                    _ => return,
                }
            } else {
                match name {
                    "anthropic" => Box::new(anthropic::AnthropicProvider::with_base_url(
                        api_key, base_url,
                    )),
                    "openai" => Box::new(openai::OpenAIProvider::with_base_url(api_key, base_url)),
                    _ => return,
                }
            };
            registry.register(provider);
        };

        let default_for = |name: &str| -> &str {
            match name {
                "openrouter" => "openrouter/auto",
                "groq" => "llama-3.3-70b-versatile",
                "deepseek" => "deepseek-chat",
                "together" => "mistralai/Mixtral-8x22B-Instruct-v0.1",
                "xai" => "grok-2",
                "perplexity" => "sonar-pro",
                "mistral" => "mistral-large-latest",
                "cohere" => "command-r-plus",
                "lmstudio" => "local-model",
                "ollama" => "llama3.2",
                "azure" => "gpt-4o",
                _ => "gpt-4o",
            }
        };

        let compat_providers = [
            "openrouter",
            "groq",
            "deepseek",
            "together",
            "xai",
            "perplexity",
            "mistral",
            "cohere",
            "lmstudio",
            "ollama",
            "azure",
        ];

        // Register native providers (Anthropic, OpenAI)
        for &name in &["anthropic", "openai"] {
            if let Some(pc) = config.providers.get(name) {
                register_builtin(
                    &mut registry,
                    name,
                    pc.api_key.as_deref().unwrap_or(""),
                    pc.base_url.as_deref().unwrap_or(""),
                );
            }
        }

        // Register OpenAI-compatible providers
        for &name in &compat_providers {
            if let Some(pc) = config.providers.get(name) {
                let api_key = pc.api_key.as_deref().unwrap_or("");
                if api_key.is_empty() {
                    continue;
                }
                let base_url = pc.base_url.as_deref().unwrap_or("");
                let display = match name {
                    "openrouter" => "OpenRouter",
                    "groq" => "Groq",
                    "deepseek" => "DeepSeek",
                    "together" => "Together AI",
                    "xai" => "xAI",
                    "perplexity" => "Perplexity",
                    "mistral" => "Mistral",
                    "cohere" => "Cohere",
                    "lmstudio" => "LM Studio",
                    "ollama" => "Ollama",
                    "azure" => "Azure OpenAI",
                    _ => name,
                };

                registry.register(Box::new(openai_compat::OpenAICompatibleProvider::new(
                    name,
                    display,
                    api_key.to_string(),
                    base_url.to_string(),
                    default_for(name).to_string(),
                )));
            }
        }

        registry
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
