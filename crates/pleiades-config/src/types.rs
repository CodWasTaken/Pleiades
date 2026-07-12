use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level configuration for Pleiades.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct Config {
    /// Core settings
    #[serde(default)]
    pub core: CoreConfig,

    /// Provider configurations
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,

    /// Model settings
    #[serde(default)]
    pub models: ModelConfig,

    /// Plugin settings
    #[serde(default)]
    pub plugins: PluginConfig,

    /// Permission settings
    #[serde(default)]
    pub permissions: PermissionConfig,

    /// Theme settings
    #[serde(default)]
    pub theme: ThemeConfig,

    /// Memory settings
    #[serde(default)]
    pub memory: MemoryConfig,
}


/// Core configuration settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct CoreConfig {
    pub default_provider: Option<String>,
    pub default_model: Option<String>,
    pub theme: Option<String>,
    pub verbose: bool,
    pub debug: bool,
    pub max_tokens: Option<u64>,
    pub temperature: Option<f32>,
}


/// Configuration for a single provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub default_model: Option<String>,
    pub max_retries: Option<u32>,
    pub timeout_secs: Option<u64>,
    pub organization: Option<String>,
    pub extra: Option<HashMap<String, String>>,
}

/// Model configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct ModelConfig {
    pub default: Option<String>,
    pub aliases: HashMap<String, String>,
}


/// Plugin configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct PluginConfig {
    pub enabled: Vec<String>,
    pub paths: Vec<String>,
}


/// Permission configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionConfig {
    pub mode: String,
    pub always_allow: Vec<String>,
    pub always_deny: Vec<String>,
}

impl Default for PermissionConfig {
    fn default() -> Self {
        Self {
            mode: "ask".to_string(),
            always_allow: Vec::new(),
            always_deny: Vec::new(),
        }
    }
}

/// Theme configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub name: Option<String>,
    pub font: Option<String>,
    pub font_size: Option<u32>,
    pub animations: bool,
    pub status_bar: bool,
    pub show_images: bool,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            name: None,
            font: None,
            font_size: None,
            animations: true,
            status_bar: true,
            show_images: false,
        }
    }
}

/// Memory configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    pub enabled: bool,
    pub max_tokens: usize,
    pub auto_prune: bool,
    pub prune_threshold: usize,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_tokens: 100_000,
            auto_prune: true,
            prune_threshold: 200_000,
        }
    }
}

/// A named configuration profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub config: Config,
}
