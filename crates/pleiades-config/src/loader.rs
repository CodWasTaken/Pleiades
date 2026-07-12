use std::path::{Path, PathBuf};

use crate::types::Config;
use crate::validate;

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

    /// Load configuration from all sources, merging them in order.
    pub fn load(&self) -> Result<Config, String> {
        let mut config = Config::default();

        // Layer 1: Global config
        if let Ok(global) = self.load_global() {
            config = self.merge(config, global);
        }

        // Layer 2: Project config
        if let Ok(project) = self.load_project() {
            config = self.merge(config, project);
        }

        // Layer 3: Environment variables
        self.apply_env(&mut config);

        // Validate the merged config
        if let Err(errors) = validate::validate(&config) {
            return Err(format!("Configuration validation failed: {}", errors));
        }

        Ok(config)
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
                return self.load_file(path, format);
            }
        }

        Err("No config file found".to_string())
    }

    /// Load a single config file in the specified format.
    fn load_file(&self, path: &Path, format: &str) -> Result<Config, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

        match format {
            "toml" => toml::from_str(&content)
                .map_err(|e| format!("Failed to parse TOML: {}", e)),
            "json" => serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse JSON: {}", e)),
            "yaml" => serde_yaml::from_str(&content)
                .map_err(|e| format!("Failed to parse YAML: {}", e)),
            _ => Err(format!("Unsupported config format: {}", format)),
        }
    }

    /// Merge two configurations, with `override_config` taking precedence.
    fn merge(&self, base: Config, override_config: Config) -> Config {
        // Simple merge strategy: non-None/empty values in override replace base
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
        merged.core.verbose |= override_config.core.verbose;
        merged.core.debug |= override_config.core.debug;
        merged.providers.extend(override_config.providers);
        merged.models.aliases.extend(override_config.models.aliases);
        merged.plugins.enabled.extend(override_config.plugins.enabled);
        merged.plugins.paths.extend(override_config.plugins.paths);

        merged
    }

    /// Apply environment variable overrides.
    fn apply_env(&self, config: &mut Config) {
        if let Ok(val) = std::env::var("PLEIADES_DEFAULT_PROVIDER") {
            config.core.default_provider = Some(val);
        }
        if let Ok(val) = std::env::var("PLEIADES_DEFAULT_MODEL") {
            config.core.default_model = Some(val);
        }
        if let Ok(val) = std::env::var("PLEIADES_THEME") {
            config.core.theme = Some(val);
        }
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
    if let Some(config_dir) = dirs::config_dir() {
        return Some(config_dir.join("pleiades"));
    }
    None
}
