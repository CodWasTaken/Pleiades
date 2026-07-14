use pleiades_agent_config::ConfigLoader;
use pleiades_agent_core::{Error, ModelInfo, ModelRegistry};

use crate::ProviderFactory;

/// Result of querying one provider during model discovery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelProviderResult {
    pub provider: String,
    pub count: Option<usize>,
    pub error: Option<String>,
}

/// Provider-independent model discovery output.
#[derive(Debug, Clone)]
pub struct ModelDiscoveryReport {
    pub models: Vec<ModelInfo>,
    pub providers: Vec<ModelProviderResult>,
}

/// Shared model discovery and configuration operations.
pub struct ModelService {
    loader: ConfigLoader,
}

impl ModelService {
    pub(crate) fn new(loader: ConfigLoader) -> Self {
        Self { loader }
    }

    pub async fn discover(&self) -> Result<ModelDiscoveryReport, Error> {
        let config = self
            .loader
            .load_with_interpolation()
            .map_err(Error::config)?;
        let providers = ProviderFactory::configured(&config);
        let references = providers
            .iter()
            .map(|provider| provider.as_ref())
            .collect::<Vec<_>>();
        let mut registry = ModelRegistry::new();
        let results = registry.discover_from_providers(&references).await;
        let providers = results
            .into_iter()
            .map(|(provider, result)| match result {
                Ok(count) => ModelProviderResult {
                    provider: provider.to_string(),
                    count: Some(count),
                    error: None,
                },
                Err(error) => ModelProviderResult {
                    provider: provider.to_string(),
                    count: None,
                    error: Some(error),
                },
            })
            .collect();
        let models = registry.list().into_iter().cloned().collect();
        Ok(ModelDiscoveryReport { models, providers })
    }

    pub async fn list(
        &self,
        provider: Option<&str>,
        search: Option<&str>,
    ) -> Result<ModelDiscoveryReport, Error> {
        let mut report = self.discover().await?;
        if let Some(provider) = provider {
            report.models.retain(|model| model.provider == provider);
        }
        if let Some(query) = search {
            let query = query.to_lowercase();
            report.models.retain(|model| {
                model.id.to_lowercase().contains(&query)
                    || model.provider.to_lowercase().contains(&query)
                    || model
                        .display_name
                        .as_deref()
                        .is_some_and(|name| name.to_lowercase().contains(&query))
            });
        }
        Ok(report)
    }

    pub async fn info(&self, name: &str) -> Result<ModelInfo, Error> {
        let config = self.loader.load().map_err(Error::config)?;
        let resolved = config.models.aliases.get(name).map_or(name, String::as_str);
        self.discover()
            .await?
            .models
            .into_iter()
            .find(|model| model.id == resolved)
            .ok_or_else(|| Error::ModelNotFound {
                model: name.to_string(),
            })
    }

    pub fn set_default(&self, model: &str) -> Result<(), Error> {
        let mut config = self.loader.load().map_err(Error::config)?;
        config.core.default_model = Some(model.to_string());
        config.models.default = Some(model.to_string());
        self.loader.save_project(&config).map_err(Error::config)
    }

    pub fn alias(&self, alias: &str, model: &str) -> Result<(), Error> {
        if alias.trim().is_empty() || model.trim().is_empty() {
            return Err(Error::invalid_input(
                "model alias and identifier cannot be empty",
            ));
        }
        let mut config = self.loader.load().map_err(Error::config)?;
        config
            .models
            .aliases
            .insert(alias.to_string(), model.to_string());
        self.loader.save_project(&config).map_err(Error::config)
    }

    pub fn unalias(&self, alias: &str) -> Result<(), Error> {
        let mut config = self.loader.load().map_err(Error::config)?;
        if config.models.aliases.remove(alias).is_none() {
            return Err(Error::invalid_input(format!(
                "model alias `{alias}` not found"
            )));
        }
        self.loader.save_project(&config).map_err(Error::config)
    }
}

#[cfg(test)]
mod tests {
    use pleiades_agent_config::{Config, ProviderConfig};

    use super::ModelService;

    #[test]
    fn mutations_preserve_environment_secret_references() {
        let temp = tempfile::tempdir().unwrap();
        let loader = pleiades_agent_config::ConfigLoader::with_dirs(
            temp.path().join("global"),
            temp.path().join("project"),
        );
        let mut config = Config::default();
        config.providers.insert(
            "openai".to_string(),
            ProviderConfig {
                api_key: Some("${OPENAI_API_KEY}".to_string()),
                ..ProviderConfig::default()
            },
        );
        loader.save_project(&config).unwrap();
        let service = ModelService::new(loader);
        service.set_default("gpt-test").unwrap();
        service.alias("fast", "gpt-test").unwrap();
        let stored = std::fs::read_to_string(temp.path().join("project/config.toml")).unwrap();
        assert!(stored.contains("${OPENAI_API_KEY}"));
        assert!(stored.contains("gpt-test"));
    }
}
