use futures::future::join_all;
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

    /// Remove a model from the registry.
    pub fn remove(&mut self, id: &str) -> Option<ModelInfo> {
        self.models.remove(id)
    }

    /// Get a model by ID (exact match, no alias resolution).
    pub fn get(&self, id: &str) -> Option<&ModelInfo> {
        self.models.get(id)
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

    /// Remove an alias.
    pub fn remove_alias(&mut self, alias: &str) -> Option<String> {
        self.aliases.remove(alias).map(|_| alias.to_string())
    }

    /// Resolve a model identifier (handles aliasing).
    pub fn resolve(&self, name: &str) -> Option<&ModelInfo> {
        self.models
            .get(name)
            .or_else(|| self.aliases.get(name).and_then(|id| self.models.get(id)))
    }

    /// List all registered models.
    pub fn list(&self) -> Vec<&ModelInfo> {
        let mut models: Vec<&ModelInfo> = self.models.values().collect();
        models.sort_by(|a, b| a.provider.cmp(&b.provider).then(a.id.cmp(&b.id)));
        models
    }

    /// List models for a specific provider.
    pub fn list_by_provider(&self, provider: &str) -> Vec<&ModelInfo> {
        let mut models: Vec<&ModelInfo> = self.models
            .values()
            .filter(|m| m.provider == provider)
            .collect();
        models.sort_by(|a, b| a.id.cmp(&b.id));
        models
    }

    /// List all aliases.
    pub fn aliases(&self) -> &HashMap<String, String> {
        &self.aliases
    }

    /// Find models by search query (matches id, display_name, provider).
    pub fn search(&self, query: &str) -> Vec<&ModelInfo> {
        let q = query.to_lowercase();
        self.models
            .values()
            .filter(|m| {
                m.id.to_lowercase().contains(&q)
                    || m.display_name.as_deref().unwrap_or("").to_lowercase().contains(&q)
                    || m.provider.to_lowercase().contains(&q)
                    || m.description.as_deref().unwrap_or("").to_lowercase().contains(&q)
            })
            .collect()
    }

    /// Get the number of registered models.
    pub fn len(&self) -> usize {
        self.models.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.models.is_empty()
    }

    /// Discover models from a list of providers.
    ///
    /// Queries each provider's `list_models()` and registers any returned models.
    /// Returns a list of (provider_name, result) pairs for reporting.
    pub async fn discover_from_providers<'a>(
        &mut self,
        providers: &[&'a dyn crate::provider::Provider],
    ) -> Vec<(&'a str, Result<usize, String>)> {
        let discovered = join_all(
            providers
                .iter()
                .map(|provider| async move { (provider.name(), provider.list_models().await) }),
        )
        .await;
        let mut results = Vec::with_capacity(discovered.len());

        for (provider_name, result) in discovered {
            match result {
                Ok(models) => {
                    let count = models.len();
                    self.register_all(models);
                    results.push((provider_name, Ok(count)));
                }
                Err(e) => {
                    results.push((provider_name, Err(e.to_string())));
                }
            }
        }

        results
    }

    /// Get a summary of models grouped by provider.
    pub fn summary_by_provider(&self) -> Vec<(&str, usize)> {
        let mut by_provider: HashMap<&str, usize> = HashMap::new();
        for model in self.models.values() {
            *by_provider.entry(model.provider.as_str()).or_insert(0) += 1;
        }
        let mut result: Vec<(&str, usize)> = by_provider.into_iter().collect();
        result.sort_by_key(|b| std::cmp::Reverse(b.1));
        result
    }
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Format a model's context length in a human-readable way.
pub fn format_context_length(len: usize) -> String {
    if len >= 1_000_000 {
        format!("{:.1}M", len as f64 / 1_000_000.0)
    } else if len >= 1_000 {
        format!("{}K", len / 1_000)
    } else {
        len.to_string()
    }
}

/// Format a price per million tokens.
pub fn format_price(price: f64) -> String {
    if price == 0.0 {
        "free".to_string()
    } else if price < 0.01 {
        format!("${:.4}", price)
    } else if price < 1.0 {
        format!("${:.3}", price)
    } else {
        format!("${:.2}", price)
    }
}
