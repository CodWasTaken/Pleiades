use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level application configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Config {
    /// Core application settings.
    pub core: CoreConfig,
    /// Provider-specific configurations.
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,
    /// Model configuration.
    #[serde(default)]
    pub models: ModelsConfig,
    /// Plugin configuration.
    #[serde(default)]
    pub plugins: PluginConfig,
    /// Permission configuration.
    #[serde(default)]
    pub permissions: PermissionConfig,
    /// Session configuration.
    #[serde(default)]
    pub session: SessionConfig,
    /// UI/Display configuration.
    #[serde(default)]
    pub display: DisplayConfig,
    /// Agent configuration.
    #[serde(default)]
    pub agent: AgentConfig,
}

/// Core application settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CoreConfig {
    /// Default provider to use.
    pub default_provider: Option<String>,
    /// Default model to use (e.g., "claude-sonnet-4-20250514").
    pub default_model: Option<String>,
    /// UI theme name.
    pub theme: Option<String>,
    /// Enable verbose output.
    #[serde(default)]
    pub verbose: bool,
    /// Enable debug mode.
    #[serde(default)]
    pub debug: bool,
    /// Maximum tokens for responses.
    pub max_tokens: Option<u32>,
    /// Temperature for generation.
    pub temperature: Option<f32>,
    /// Path to custom config file.
    pub config_file: Option<String>,
    /// Whether to auto-update.
    #[serde(default = "default_true")]
    pub auto_update: bool,
    /// Logging level (error, warn, info, debug, trace).
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_true() -> bool {
    true
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            default_provider: None,
            default_model: None,
            theme: None,
            verbose: false,
            debug: false,
            max_tokens: Some(4096),
            temperature: Some(0.7),
            config_file: None,
            auto_update: true,
            log_level: "info".to_string(),
        }
    }
}

/// Provider-specific settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProviderConfig {
    /// API key for the provider.
    pub api_key: Option<String>,
    /// Base URL for the provider API.
    pub base_url: Option<String>,
    /// Custom headers to include in requests.
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// Organization ID (for OpenAI/Anthropic).
    pub organization_id: Option<String>,
    /// Maximum retries.
    #[serde(default = "default_retries")]
    pub max_retries: u32,
    /// Request timeout in seconds.
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

fn default_retries() -> u32 {
    3
}

fn default_timeout() -> u64 {
    120
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            base_url: None,
            headers: HashMap::new(),
            organization_id: None,
            max_retries: 3,
            timeout_secs: 120,
        }
    }
}

/// Model configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ModelsConfig {
    /// Model aliases (short name → full model ID).
    #[serde(default)]
    pub aliases: HashMap<String, String>,
    /// Default model override.
    pub default: Option<String>,
}

/// Plugin configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginConfig {
    /// Globally enabled plugins.
    #[serde(default)]
    pub enabled: Vec<String>,
    /// Plugin search paths.
    #[serde(default)]
    pub paths: Vec<String>,
    /// Per-plugin settings.
    #[serde(default)]
    pub settings: HashMap<String, HashMap<String, String>>,
    /// Sandbox mode for plugins.
    #[serde(default)]
    pub sandbox: bool,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            enabled: Vec::new(),
            paths: vec!["~/.pleiades/plugins".to_string()],
            settings: HashMap::new(),
            // Shell-hook plugins execute as normal child processes. This must remain false
            // until a real sandbox runtime is implemented.
            sandbox: false,
        }
    }
}

/// Permission configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PermissionConfig {
    /// Commands always allowed without confirmation.
    #[serde(default)]
    pub always_allow: Vec<String>,
    /// Commands always denied.
    #[serde(default)]
    pub always_deny: Vec<String>,
    /// Allow mode confirmation.
    #[serde(default)]
    pub ask_always: bool,
    /// Maximum permission grant duration in minutes.
    #[serde(default = "default_permission_duration")]
    pub grant_duration_minutes: u32,
}

fn default_permission_duration() -> u32 {
    60
}

impl Default for PermissionConfig {
    fn default() -> Self {
        Self {
            always_allow: Vec::new(),
            always_deny: Vec::new(),
            ask_always: true,
            grant_duration_minutes: 60,
        }
    }
}

/// Session configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionConfig {
    /// Number of messages to keep in context.
    #[serde(default = "default_context_size")]
    pub context_size: usize,
    /// Auto-save session interval in seconds.
    pub auto_save_interval_secs: Option<u64>,
    /// Session history directory.
    pub history_dir: Option<String>,
    /// Maximum number of concurrent sessions.
    #[serde(default = "default_max_sessions")]
    pub max_concurrent: usize,
    /// Whether to compress older sessions.
    #[serde(default)]
    pub compress_history: bool,
}

fn default_context_size() -> usize {
    100
}

fn default_max_sessions() -> usize {
    10
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            context_size: 100,
            auto_save_interval_secs: Some(60),
            history_dir: None,
            max_concurrent: 10,
            compress_history: false,
        }
    }
}

/// Display configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DisplayConfig {
    /// UI style (plain, rich, minimal).
    #[serde(default = "default_style")]
    pub style: String,
    /// Enable syntax highlighting.
    #[serde(default = "default_true2")]
    pub syntax_highlighting: bool,
    /// Show token usage in output.
    #[serde(default)]
    pub show_token_usage: bool,
    /// Show timing information.
    #[serde(default)]
    pub show_timing: bool,
    /// Output width (0 = auto).
    #[serde(default)]
    pub output_width: u32,
    /// Whether to show progress indicators.
    #[serde(default = "default_true2")]
    pub show_progress: bool,
}

fn default_style() -> String {
    "rich".to_string()
}

fn default_true2() -> bool {
    true
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            style: "rich".to_string(),
            syntax_highlighting: true,
            show_token_usage: false,
            show_timing: false,
            output_width: 0,
            show_progress: true,
        }
    }
}

/// Agent configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentConfig {
    /// Default agent persona.
    pub default_persona: Option<String>,
    /// Custom system prompt additions.
    pub system_prompt_prefix: Option<String>,
    /// List of tool categories to enable by default.
    #[serde(default)]
    pub default_tools: Vec<String>,
    /// Maximum tool call iterations.
    #[serde(default = "default_tool_iters")]
    pub max_tool_iterations: u32,
    /// Allow agent to edit files without confirmation.
    #[serde(default)]
    pub auto_edit: bool,
}

fn default_tool_iters() -> u32 {
    25
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            default_persona: None,
            system_prompt_prefix: None,
            default_tools: Vec::new(),
            max_tool_iterations: 25,
            auto_edit: false,
        }
    }
}

/// Configuration field path for error reporting.
#[derive(Debug, Clone)]
pub struct FieldError {
    /// Dot-separated path to the field (e.g., "core.default_provider").
    pub field: String,
    /// Human-readable error message.
    pub message: String,
}

/// Result of a config modification operation.
#[derive(Debug, Clone)]
pub struct ConfigChange {
    /// Path of the changed field.
    pub path: String,
    /// Previous value (as string).
    pub old_value: Option<String>,
    /// New value (as string).
    pub new_value: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.core.default_provider.is_none());
        assert_eq!(config.core.max_tokens, Some(4096));
        assert_eq!(config.core.log_level, "info");
        assert_eq!(config.display.style, "rich");
        assert_eq!(config.session.context_size, 100);
        assert!(!config.plugins.sandbox);
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_provider_config_defaults() {
        let provider = ProviderConfig::default();
        assert_eq!(provider.max_retries, 3);
        assert_eq!(provider.timeout_secs, 120);
    }
}
