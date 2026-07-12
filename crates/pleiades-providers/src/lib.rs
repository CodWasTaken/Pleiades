//! AI provider implementations for Pleiades.
//!
//! Each provider implements the `Provider` trait from `pleiades-core`,
//! enabling seamless switching between AI backends.

pub mod anthropic;
pub mod openai;
pub mod openai_compat;

use pleiades_core::provider::Provider;

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
        self.providers.iter().find(|p| p.name() == name).map(|p| p.as_ref())
    }

    /// List all registered providers.
    pub fn list(&self) -> Vec<&dyn Provider> {
        self.providers.iter().map(|p| p.as_ref()).collect()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
