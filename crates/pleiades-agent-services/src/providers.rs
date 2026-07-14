use pleiades_agent_config::{Config, ConfigLoader, SecretManager};
use pleiades_agent_core::{Error, Provider};

/// Secret-safe provider information suitable for terminal or JSON rendering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderReport {
    pub name: String,
    pub authentication: String,
    pub api_key_display: String,
    pub base_url: String,
    pub expected_env_var: Option<String>,
    pub max_retries: u32,
    pub timeout_secs: u64,
}

/// Shared provider management operations.
pub struct ProviderService {
    loader: ConfigLoader,
}

impl ProviderService {
    pub(crate) fn new(loader: ConfigLoader) -> Self {
        Self { loader }
    }

    /// List configured providers without exposing resolved secret values.
    pub fn list(&self) -> Result<Vec<ProviderReport>, Error> {
        let config = self.loader.load().map_err(Error::config)?;
        let secret_manager = SecretManager::new();
        let mut reports = config
            .providers
            .iter()
            .map(|(name, provider)| ProviderReport {
                name: name.clone(),
                authentication: if name == "openai-subscription" {
                    "ChatGPT subscription via official Codex CLI".to_string()
                } else {
                    "API key".to_string()
                },
                api_key_display: provider
                    .api_key
                    .as_deref()
                    .map(pleiades_agent_config::env_interpolate::mask_secrets)
                    .unwrap_or_else(|| "not set".to_string()),
                base_url: provider
                    .base_url
                    .clone()
                    .unwrap_or_else(|| "(default)".to_string()),
                expected_env_var: secret_manager.expected_env_var(name).map(str::to_string),
                max_retries: provider.max_retries,
                timeout_secs: provider.timeout_secs,
            })
            .collect::<Vec<_>>();
        reports.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(reports)
    }

    pub fn info(&self, name: &str) -> Result<ProviderReport, Error> {
        self.list()?
            .into_iter()
            .find(|report| report.name == name)
            .ok_or_else(|| Error::ProviderNotFound {
                provider: name.to_string(),
            })
    }

    /// Remove a project provider using the unexpanded configuration, so an
    /// environment-backed API key can never be copied into the config file.
    pub fn remove(&self, name: &str) -> Result<(), Error> {
        let mut config = self.loader.load().map_err(Error::config)?;
        if config.providers.remove(name).is_none() {
            return Err(Error::ProviderNotFound {
                provider: name.to_string(),
            });
        }
        self.loader.save_project(&config).map_err(Error::config)
    }

    pub fn configured(&self) -> Result<Vec<Box<dyn Provider>>, Error> {
        let config = self
            .loader
            .load_with_interpolation()
            .map_err(Error::config)?;
        Ok(ProviderFactory::configured(&config))
    }
}

/// Canonical provider adapter construction used by services, CLI commands,
/// the agent engine, and model discovery.
pub struct ProviderFactory;

impl ProviderFactory {
    pub fn configured(config: &Config) -> Vec<Box<dyn Provider>> {
        config
            .providers
            .iter()
            .filter_map(|(name, provider)| {
                let key = provider.api_key.as_deref().unwrap_or("");
                let base_url = provider.base_url.as_deref().unwrap_or("");
                Self::build(name, key, base_url)
            })
            .collect()
    }

    pub fn build(name: &str, api_key: &str, base_url: &str) -> Option<Box<dyn Provider>> {
        if name == "openai-subscription" {
            return Some(Box::new(
                pleiades_agent_providers::codex::CodexCliProvider::new(),
            ));
        }
        if api_key.is_empty() {
            return None;
        }
        match name {
            "anthropic" if base_url.is_empty() => Some(Box::new(
                pleiades_agent_providers::anthropic::AnthropicProvider::new(api_key),
            )),
            "anthropic" => Some(Box::new(
                pleiades_agent_providers::anthropic::AnthropicProvider::with_base_url(
                    api_key, base_url,
                ),
            )),
            "openai" if base_url.is_empty() => Some(Box::new(
                pleiades_agent_providers::openai::OpenAIProvider::new(api_key),
            )),
            "openai" => Some(Box::new(
                pleiades_agent_providers::openai::OpenAIProvider::with_base_url(api_key, base_url),
            )),
            _ => Some(Box::new(
                pleiades_agent_providers::openai_compat::OpenAICompatibleProvider::new(
                    name,
                    name,
                    api_key.to_string(),
                    base_url.to_string(),
                    "gpt-4o".to_string(),
                ),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use pleiades_agent_config::{Config, ConfigLoader, ProviderConfig};

    use super::ProviderService;

    fn service(temp: &tempfile::TempDir, config: &Config) -> ProviderService {
        let loader =
            ConfigLoader::with_dirs(temp.path().join("global"), temp.path().join("project"));
        loader.save_project(config).unwrap();
        ProviderService::new(loader)
    }

    #[test]
    fn reports_are_sorted_and_secrets_are_masked() {
        let temp = tempfile::tempdir().unwrap();
        let mut config = Config::default();
        config.providers.insert(
            "openai".into(),
            ProviderConfig {
                api_key: Some("sk-secret-value".into()),
                ..ProviderConfig::default()
            },
        );
        config
            .providers
            .insert("anthropic".into(), ProviderConfig::default());
        let reports = service(&temp, &config).list().unwrap();
        assert_eq!(reports[0].name, "anthropic");
        assert_eq!(reports[1].name, "openai");
        assert!(!reports[1].api_key_display.contains("secret-value"));
    }

    #[test]
    fn removal_never_persists_an_expanded_environment_secret() {
        let temp = tempfile::tempdir().unwrap();
        let mut config = Config::default();
        config.providers.insert(
            "openai".into(),
            ProviderConfig {
                api_key: Some("${PLEIADES_TEST_SECRET}".into()),
                ..ProviderConfig::default()
            },
        );
        config
            .providers
            .insert("remove-me".into(), ProviderConfig::default());
        let service = service(&temp, &config);
        service.remove("remove-me").unwrap();
        let stored =
            std::fs::read_to_string(PathBuf::from(temp.path()).join("project/config.toml"))
                .unwrap();
        assert!(stored.contains("${PLEIADES_TEST_SECRET}"));
    }
}
