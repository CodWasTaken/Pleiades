use std::path::{Path, PathBuf};

use crate::types::Config;
use crate::validate::{self, format_errors};

/// Loads configuration from multiple sources with proper layering.
pub struct ConfigLoader {
    global_dir: PathBuf,
    project_dir: PathBuf,
}

impl ConfigLoader {
    /// Create a new config loader.
    pub fn new() -> Self {
        let global_dir = dirs_config_dir().unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".config")
                .join("pleiades")
        });

        Self {
            global_dir,
            project_dir: PathBuf::from(".pleiades"),
        }
    }

    /// Create a loader with custom directories.
    pub fn with_dirs(global_dir: PathBuf, project_dir: PathBuf) -> Self {
        Self {
            global_dir,
            project_dir,
        }
    }

    /// Get the global config directory.
    pub fn global_dir(&self) -> &Path {
        &self.global_dir
    }

    /// Get the project config directory.
    pub fn project_dir(&self) -> &Path {
        &self.project_dir
    }

    /// Load configuration from all sources, merging them in order.
    /// Order: defaults → global → project → env vars
    pub fn load(&self) -> Result<Config, String> {
        let mut config = Config::default();

        if let Ok(global) = self.load_global() {
            config = Self::merge(config, global);
        }

        if let Ok(project) = self.load_project() {
            config = Self::merge(config, project);
        }

        self.apply_env(&mut config);

        if let Err(errors) = validate::validate(&config) {
            return Err(format!(
                "Configuration validation failed:\n{}",
                format_errors(&errors)
            ));
        }

        Ok(config)
    }

    /// Load configuration and perform env var interpolation.
    pub fn load_with_interpolation(&self) -> Result<Config, String> {
        let mut config = self.load()?;

        // Interpolate API keys in provider configs
        for provider in config.providers.values_mut() {
            if let Some(key) = &provider.api_key {
                let interpolated = crate::env_interpolate::interpolate(key);
                if interpolated != *key {
                    provider.api_key = Some(interpolated);
                }
            }
        }

        Ok(config)
    }

    /// Save configuration to the project config file.
    pub fn save_project(&self, config: &Config) -> Result<(), String> {
        self.save_to_dir(&self.project_dir, config)
    }

    /// Save configuration to the global config file.
    pub fn save_global(&self, config: &Config) -> Result<(), String> {
        self.save_to_dir(&self.global_dir, config)
    }

    /// Save configuration to a specific directory.
    fn save_to_dir(&self, dir: &Path, config: &Config) -> Result<(), String> {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;

        let path = dir.join("config.toml");
        let content = toml::to_string_pretty(config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        std::fs::write(&path, &content).map_err(|e| format!("Failed to write config: {}", e))?;

        Ok(())
    }

    /// Load global configuration (~/.config/pleiades/).
    fn load_global(&self) -> Result<Config, String> {
        self.load_from_dir(&self.global_dir)
    }

    /// Load project configuration (./.pleiades/).
    fn load_project(&self) -> Result<Config, String> {
        self.load_from_dir(&self.project_dir)
    }

    /// Load configuration from a directory, trying multiple formats.
    fn load_from_dir(&self, dir: &Path) -> Result<Config, String> {
        let formats = [
            (dir.join("config.toml"), "toml" as &str),
            (dir.join("config.json"), "json"),
            (dir.join("config.yaml"), "yaml"),
            (dir.join("config.yml"), "yaml"),
        ];

        for (path, format) in &formats {
            if path.exists() {
                let config = self.load_file(path, format)?;
                return Ok(config);
            }
        }

        Err("No config file found".to_string())
    }

    /// Load a single config file in the specified format.
    fn load_file(&self, path: &Path, format: &str) -> Result<Config, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

        match format {
            "toml" => toml::from_str(&content).map_err(|e| format!("Failed to parse TOML: {}", e)),
            "json" => {
                serde_json::from_str(&content).map_err(|e| format!("Failed to parse JSON: {}", e))
            }
            "yaml" => {
                serde_yaml::from_str(&content).map_err(|e| format!("Failed to parse YAML: {}", e))
            }
            _ => Err(format!("Unsupported config format: {}", format)),
        }
    }

    /// Merge two configurations deeply, with `override_config` taking precedence.
    pub fn merge(base: Config, override_config: Config) -> Config {
        let mut merged = base;

        if let Some(provider) = override_config.core.default_provider {
            merged.core.default_provider = Some(provider);
        }
        if let Some(model) = override_config.core.default_model {
            merged.core.default_model = Some(model);
        }
        if let Some(theme) = override_config.core.theme {
            merged.core.theme = Some(theme);
        }
        if override_config.core.verbose {
            merged.core.verbose = true;
        }
        if override_config.core.debug {
            merged.core.debug = true;
        }
        merged.core.max_tokens = override_config.core.max_tokens.or(merged.core.max_tokens);
        merged.core.temperature = override_config.core.temperature.or(merged.core.temperature);

        merged.providers.extend(override_config.providers);
        merged.models.aliases.extend(override_config.models.aliases);
        merged.models.default = override_config.models.default.or(merged.models.default);
        merged
            .models
            .favorites
            .extend(override_config.models.favorites);
        merged.models.favorites.sort();
        merged.models.favorites.dedup();
        merged.models.reasoning = override_config.models.reasoning.or(merged.models.reasoning);

        if override_config.session.context_size
            != crate::types::SessionConfig::default().context_size
        {
            merged.session.context_size = override_config.session.context_size;
        }
        if override_config.session.auto_save_interval_secs.is_some() {
            merged.session.auto_save_interval_secs =
                override_config.session.auto_save_interval_secs;
        }
        if override_config.session.history_dir.is_some() {
            merged.session.history_dir = override_config.session.history_dir;
        }
        if override_config.session.max_concurrent
            != crate::types::SessionConfig::default().max_concurrent
        {
            merged.session.max_concurrent = override_config.session.max_concurrent;
        }
        if override_config.session.compress_history {
            merged.session.compress_history = true;
        }
        if override_config.session.ephemeral {
            merged.session.ephemeral = true;
        }

        merged
            .plugins
            .enabled
            .extend(override_config.plugins.enabled);
        merged.plugins.paths.extend(override_config.plugins.paths);
        merged.mcp.servers.extend(override_config.mcp.servers);

        if !override_config.permissions.always_allow.is_empty() {
            merged.permissions.always_allow = override_config.permissions.always_allow;
        }
        if !override_config.permissions.always_deny.is_empty() {
            merged.permissions.always_deny = override_config.permissions.always_deny;
        }
        merged
            .permissions
            .rules
            .extend(override_config.permissions.rules);

        merged
    }

    /// Apply environment variable overrides.
    fn apply_env(&self, config: &mut Config) {
        macro_rules! env_set {
            ($var:expr, $target:expr) => {
                if let Ok(val) = std::env::var($var) {
                    $target = Some(val);
                }
            };
        }

        env_set!("PLEIADES_DEFAULT_PROVIDER", config.core.default_provider);
        env_set!("PLEIADES_DEFAULT_MODEL", config.core.default_model);
        env_set!("PLEIADES_THEME", config.core.theme);

        if let Ok(val) = std::env::var("PLEIADES_VERBOSE") {
            config.core.verbose = val == "1" || val.to_lowercase() == "true";
        }
        if let Ok(val) = std::env::var("PLEIADES_DEBUG") {
            config.core.debug = val == "1" || val.to_lowercase() == "true";
        }
        if let Ok(val) = std::env::var("PLEIADES_MAX_TOKENS") {
            if let Ok(n) = val.parse() {
                config.core.max_tokens = Some(n);
            }
        }
        if let Ok(val) = std::env::var("PLEIADES_TEMPERATURE") {
            if let Ok(f) = val.parse() {
                config.core.temperature = Some(f);
            }
        }
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the platform-appropriate config directory.
fn dirs_config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("pleiades"))
}

/// Detect the config format from a file path.
pub fn format_for_path(path: &Path) -> Option<&'static str> {
    match path.extension()?.to_str()? {
        "toml" => Some("toml"),
        "json" => Some("json"),
        "yaml" | "yml" => Some("yaml"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loader_global_dir() {
        let loader = ConfigLoader::new();
        assert!(loader.global_dir().to_string_lossy().contains("pleiades"));
    }

    #[test]
    fn test_format_detection() {
        assert_eq!(format_for_path(Path::new("config.toml")), Some("toml"));
        assert_eq!(format_for_path(Path::new("config.json")), Some("json"));
        assert_eq!(format_for_path(Path::new("config.yaml")), Some("yaml"));
        assert_eq!(format_for_path(Path::new("config.yml")), Some("yaml"));
        assert_eq!(format_for_path(Path::new("config.txt")), None);
    }

    #[test]
    fn test_merge_simple() {
        let base = Config::default();
        let mut override_cfg = Config::default();
        override_cfg.core.default_provider = Some("openai".to_string());

        let merged = ConfigLoader::merge(base, override_cfg);
        assert_eq!(merged.core.default_provider, Some("openai".to_string()));
    }

    #[test]
    fn test_merge_preserves_defaults() {
        let base = Config::default();
        let override_cfg = Config::default();

        let merged = ConfigLoader::merge(base, override_cfg);
        assert_eq!(merged.core.default_provider, None);
    }

    #[test]
    fn partial_project_config_can_override_session_history_dir() {
        let temp = tempfile::tempdir().unwrap();
        let project = temp.path().join(".pleiades");
        std::fs::create_dir_all(&project).unwrap();
        std::fs::write(
            project.join("config.toml"),
            "[session]\nhistory_dir = \"/tmp/pleiades-test-sessions\"\n",
        )
        .unwrap();

        let loader = ConfigLoader::with_dirs(temp.path().join("global"), project);
        let config = loader.load().unwrap();

        assert_eq!(
            config.session.history_dir.as_deref(),
            Some("/tmp/pleiades-test-sessions")
        );
        assert_eq!(config.core.theme.as_deref(), Some("seven-sisters"));
    }

    #[test]
    fn test_merge_providers() {
        let mut base = Config::default();
        base.providers.insert(
            "anthropic".to_string(),
            crate::types::ProviderConfig {
                api_key: Some("key1".to_string()),
                ..Default::default()
            },
        );

        let mut override_cfg = Config::default();
        override_cfg.providers.insert(
            "openai".to_string(),
            crate::types::ProviderConfig {
                api_key: Some("key2".to_string()),
                ..Default::default()
            },
        );

        let merged = ConfigLoader::merge(base, override_cfg);
        assert_eq!(merged.providers.len(), 2);
        assert_eq!(
            merged.providers.get("anthropic").unwrap().api_key,
            Some("key1".to_string())
        );
        assert_eq!(
            merged.providers.get("openai").unwrap().api_key,
            Some("key2".to_string())
        );
    }
}
