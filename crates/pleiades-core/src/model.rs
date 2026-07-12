use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Model capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
    pub max_context_length: usize,
    pub max_output_tokens: usize,
    pub supports_tools: bool,
    pub supports_vision: bool,
    pub supports_streaming: bool,
    pub supports_thinking: bool,
    pub supports_json_mode: bool,
}

/// Pricing information for a model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pricing {
    pub input_per_million: f64,
    pub output_per_million: f64,
    pub cache_read_per_million: Option<f64>,
    pub cache_write_per_million: Option<f64>,
}

/// Information about a specific model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub provider: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub capabilities: ModelCapabilities,
    pub pricing: Option<Pricing>,
}

/// A model alias mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelAlias {
    pub alias: String,
    pub model_id: String,
}

/// Central registry for models across all providers.
///
/// Maintains a collection of known models with metadata,
/// supports aliasing for convenience, and allows providers
/// to register their available models.
#[derive(Debug, Clone)]
pub struct ModelRegistry {
    models: HashMap<String, ModelInfo>,
    aliases: HashMap<String, String>,
}

impl ModelRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
            aliases: HashMap::new(),
        }
    }

    /// Register a model in the registry.
    pub fn register(&mut self, info: ModelInfo) {
        self.models.insert(info.id.clone(), info);
    }

    /// Register multiple models at once.
    pub fn register_all(&mut self, models: Vec<ModelInfo>) {
        for model in models {
            self.register(model);
        }
    }

    /// Add an alias for a model.
    pub fn add_alias(&mut self, alias: impl Into<String>, model_id: impl Into<String>) -> Result<(), String> {
        let alias = alias.into();
        let model_id = model_id.into();
        if !self.models.contains_key(&model_id) {
            return Err(format!("Model '{}' not found in registry", model_id));
        }
        self.aliases.insert(alias, model_id);
        Ok(())
    }

    /// Resolve a model identifier (handles aliasing).
    pub fn resolve(&self, name: &str) -> Option<&ModelInfo> {
        self.models
            .get(name)
            .or_else(|| self.aliases.get(name).and_then(|id| self.models.get(id)))
    }

    /// List all registered models.
    pub fn list(&self) -> Vec<&ModelInfo> {
        self.models.values().collect()
    }

    /// List models for a specific provider.
    pub fn list_by_provider(&self, provider: &str) -> Vec<&ModelInfo> {
        self.models
            .values()
            .filter(|m| m.provider == provider)
            .collect()
    }

    /// List all aliases.
    pub fn aliases(&self) -> &HashMap<String, String> {
        &self.aliases
    }

    /// Get the number of registered models.
    pub fn len(&self) -> usize {
        self.models.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.models.is_empty()
    }
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}
