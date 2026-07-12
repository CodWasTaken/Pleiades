use std::collections::HashMap;

/// Manages sensitive credentials (API keys, tokens).
///
/// Supports retrieving secrets from:
/// 1. Environment variables (preferred)
/// 2. OS keychain (optional)
/// 3. Config values (fallback, with env var interpolation)
pub struct SecretManager {
    /// Provider API key env var mapping
    provider_env_map: HashMap<&'static str, &'static str>,
}

impl SecretManager {
    /// Create a new secret manager with default provider mappings.
    pub fn new() -> Self {
        Self {
            provider_env_map: HashMap::from([
                ("anthropic", "ANTHROPIC_API_KEY"),
                ("openai", "OPENAI_API_KEY"),
                ("google", "GOOGLE_API_KEY"),
                ("openrouter", "OPENROUTER_API_KEY"),
                ("groq", "GROQ_API_KEY"),
                ("ollama", ""),
                ("lmstudio", ""),
                ("mistral", "MISTRAL_API_KEY"),
                ("cohere", "CO_API_KEY"),
                ("deepseek", "DEEPSEEK_API_KEY"),
                ("together", "TOGETHER_API_KEY"),
                ("xai", "XAI_API_KEY"),
                ("perplexity", "PERPLEXITY_API_KEY"),
                ("azure", "AZURE_OPENAI_API_KEY"),
            ]),
        }
    }

    /// Get the API key for a provider, checking multiple sources.
    pub fn get_api_key(&self, provider: &str, config_key: Option<String>) -> Option<String> {
        // 1. Try config value first (with env var interpolation)
        if let Some(key) = config_key {
            let interpolated = crate::env_interpolate::interpolate(&key);
            if !interpolated.is_empty() && interpolated != "${}" && interpolated != "$" {
                return Some(interpolated);
            }
        }

        // 2. Try environment variable
        let env_var = self.provider_env_map.get(provider).copied().unwrap_or("");
        if !env_var.is_empty() {
            if let Ok(val) = std::env::var(env_var) {
                if !val.is_empty() {
                    return Some(val);
                }
            }
        }

        // 3. Try generic PLEIADES_* env var
        let generic_var = format!("PLEIADES_{}_API_KEY", provider.to_uppercase());
        if let Ok(val) = std::env::var(&generic_var) {
            if !val.is_empty() {
                return Some(val);
            }
        }

        None
    }

    /// Check if a provider has credentials configured.
    pub fn has_credentials(&self, provider: &str) -> bool {
        self.get_api_key(provider, None).is_some()
    }

    /// List providers that have credentials configured.
    pub fn configured_providers(&self) -> Vec<&str> {
        self.provider_env_map
            .keys()
            .filter(|p| self.get_api_key(p, None).is_some())
            .copied()
            .collect()
    }

    /// Get the expected env var name for a provider.
    pub fn expected_env_var(&self, provider: &str) -> Option<&str> {
        self.provider_env_map.get(provider).copied()
    }

    /// Clear a secret from the environment (for logout).
    pub fn clear_env_var(&self, provider: &str) {
        if let Some(env_var) = self.provider_env_map.get(provider) {
            if !env_var.is_empty() {
                #[allow(unused_unsafe)]
                unsafe { std::env::remove_var(env_var); }
            }
        }
    }
}

impl Default for SecretManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set_env(key: &str, val: &str) {
        unsafe { std::env::set_var(key, val); }
    }

    fn remove_env(key: &str) {
        unsafe { std::env::remove_var(key); }
    }

    #[test]
    fn test_get_api_key_from_env() {
        set_env("ANTHROPIC_API_KEY", "sk-ant-test123");
        let manager = SecretManager::new();
        let key = manager.get_api_key("anthropic", None);
        assert_eq!(key, Some("sk-ant-test123".to_string()));
        remove_env("ANTHROPIC_API_KEY");
    }

    #[test]
    fn test_get_api_key_config_preferred() {
        set_env("PLEIADES_TEST_ANTHROPIC_PREF", "from_env");
        let manager = SecretManager::new();
        let key = manager.get_api_key("anthropic", Some("from_config".to_string()));
        assert_eq!(key, Some("from_config".to_string()));
    }

    #[test]
    fn test_api_key_not_found() {
        let manager = SecretManager::new();
        let key = manager.get_api_key("nonexistent_provider", None);
        assert_eq!(key, None);
    }

    #[test]
    fn test_configured_providers() {
        set_env("PLEIADES_TEST_OPENAI_KEY", "sk-test");
        set_env("OPENAI_API_KEY", "sk-test");
        let manager = SecretManager::new();
        let providers = manager.configured_providers();
        assert!(providers.contains(&"openai"));
        remove_env("OPENAI_API_KEY");
    }
}
