use crate::types::{Config, FieldError};

/// Validate the entire configuration.
/// Returns a list of field-level errors if validation fails.
pub fn validate(config: &Config) -> Result<(), Vec<FieldError>> {
    let mut errors = Vec::new();

    validate_core(&config.core, &mut errors);
    validate_providers(&config.providers, &mut errors);
    validate_session(&config.session, &mut errors);
    validate_display(&config.display, &mut errors);
    validate_agent(&config.agent, &mut errors);
    validate_plugins(&config.plugins, &mut errors);
    validate_permissions(&config.permissions, &mut errors);

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_core(core: &crate::types::CoreConfig, errors: &mut Vec<FieldError>) {
    if let Some(ref model) = core.default_model {
        if model.trim().is_empty() {
            errors.push(FieldError {
                field: "core.default_model".to_string(),
                message: "Default model cannot be empty".to_string(),
            });
        }
    }

    if let Some(ref provider) = core.default_provider {
        if provider.trim().is_empty() {
            errors.push(FieldError {
                field: "core.default_provider".to_string(),
                message: "Default provider cannot be empty".to_string(),
            });
        }
    }

    if let Some(tokens) = core.max_tokens {
        if tokens == 0 {
            errors.push(FieldError {
                field: "core.max_tokens".to_string(),
                message: "max_tokens must be greater than 0".to_string(),
            });
        }
        if tokens > 1_000_000 {
            errors.push(FieldError {
                field: "core.max_tokens".to_string(),
                message: "max_tokens exceeds maximum allowed (1,000,000)".to_string(),
            });
        }
    }

    if let Some(temp) = core.temperature {
        if !(0.0..=2.0).contains(&temp) {
            errors.push(FieldError {
                field: "core.temperature".to_string(),
                message: "temperature must be between 0.0 and 2.0".to_string(),
            });
        }
    }

    let valid_levels = ["error", "warn", "info", "debug", "trace"];
    if !valid_levels.contains(&core.log_level.as_str()) {
        errors.push(FieldError {
            field: "core.log_level".to_string(),
            message: format!(
                "Invalid log level '{}'. Must be one of: {}",
                core.log_level,
                valid_levels.join(", ")
            ),
        });
    }
}

fn validate_providers(
    providers: &std::collections::HashMap<String, crate::types::ProviderConfig>,
    errors: &mut Vec<FieldError>,
) {
    for (name, provider) in providers {
        if let Some(ref url) = provider.base_url {
            if !url.starts_with("http://") && !url.starts_with("https://") {
                errors.push(FieldError {
                    field: format!("providers.{}.base_url", name),
                    message: format!(
                        "Invalid base URL for provider '{}': must start with http:// or https://",
                        name
                    ),
                });
            }
        }
    }
}

fn validate_session(session: &crate::types::SessionConfig, errors: &mut Vec<FieldError>) {
    if session.context_size == 0 {
        errors.push(FieldError {
            field: "session.context_size".to_string(),
            message: "context_size must be greater than 0".to_string(),
        });
    }
    if session.context_size > 10_000 {
        errors.push(FieldError {
            field: "session.context_size".to_string(),
            message: "context_size too large (max 10,000)".to_string(),
        });
    }
    if session.max_concurrent == 0 {
        errors.push(FieldError {
            field: "session.max_concurrent".to_string(),
            message: "max_concurrent must be greater than 0".to_string(),
        });
    }
}

fn validate_display(display: &crate::types::DisplayConfig, errors: &mut Vec<FieldError>) {
    let valid_styles = ["plain", "rich", "minimal"];
    if !valid_styles.contains(&display.style.as_str()) {
        errors.push(FieldError {
            field: "display.style".to_string(),
            message: format!(
                "Invalid display style '{}'. Must be one of: {}",
                display.style,
                valid_styles.join(", ")
            ),
        });
    }
}

fn validate_agent(agent: &crate::types::AgentConfig, errors: &mut Vec<FieldError>) {
    if agent.max_tool_iterations == 0 {
        errors.push(FieldError {
            field: "agent.max_tool_iterations".to_string(),
            message: "max_tool_iterations must be greater than 0".to_string(),
        });
    }
    if agent.max_tool_iterations > 1000 {
        errors.push(FieldError {
            field: "agent.max_tool_iterations".to_string(),
            message: "max_tool_iterations too large (max 1000)".to_string(),
        });
    }
}

fn validate_plugins(_plugins: &crate::types::PluginConfig, _errors: &mut Vec<FieldError>) {
    // Plugin validation is minimal since plugins are loaded dynamically.
    // Plugin path existence is validated at runtime.
}

fn validate_permissions(
    permissions: &crate::types::PermissionConfig,
    errors: &mut Vec<FieldError>,
) {
    if permissions.grant_duration_minutes == 0 {
        errors.push(FieldError {
            field: "permissions.grant_duration_minutes".to_string(),
            message: "grant_duration_minutes must be greater than 0".to_string(),
        });
    }

    // Check for overlapping allow/deny rules
    for cmd in &permissions.always_allow {
        if permissions.always_deny.contains(cmd) {
            errors.push(FieldError {
                field: "permissions.always_allow".to_string(),
                message: format!("Command '{}' is in both always_allow and always_deny", cmd),
            });
        }
    }

    if let Err(error) = pleiades_agent_permissions::PermissionEngine::new(permissions.rules.clone())
    {
        errors.push(FieldError {
            field: "permissions.rules".to_string(),
            message: error.to_string(),
        });
    }
}

/// Validate a single config key-value pair.
pub fn validate_field(key: &str, value: &str) -> Result<(), String> {
    match key {
        "core.max_tokens" => {
            let n: u32 = value
                .parse()
                .map_err(|_| "max_tokens must be a positive integer".to_string())?;
            if n == 0 {
                return Err("max_tokens must be greater than 0".to_string());
            }
            if n > 1_000_000 {
                return Err("max_tokens exceeds maximum allowed (1,000,000)".to_string());
            }
        }
        "core.temperature" => {
            let f: f32 = value
                .parse()
                .map_err(|_| "temperature must be a number".to_string())?;
            if !(0.0..=2.0).contains(&f) {
                return Err("temperature must be between 0.0 and 2.0".to_string());
            }
        }
        "core.log_level" => {
            let valid = ["error", "warn", "info", "debug", "trace"];
            if !valid.contains(&value) {
                return Err(format!("log_level must be one of: {}", valid.join(", ")));
            }
        }
        "session.context_size" => {
            let n: usize = value
                .parse()
                .map_err(|_| "context_size must be a positive integer".to_string())?;
            if n == 0 {
                return Err("context_size must be greater than 0".to_string());
            }
            if n > 10_000 {
                return Err("context_size too large (max 10,000)".to_string());
            }
        }
        "display.style" => {
            let valid = ["plain", "rich", "minimal"];
            if !valid.contains(&value) {
                return Err(format!("style must be one of: {}", valid.join(", ")));
            }
        }
        "agent.max_tool_iterations" => {
            let n: u32 = value
                .parse()
                .map_err(|_| "max_tool_iterations must be a positive integer".to_string())?;
            if n == 0 {
                return Err("max_tool_iterations must be greater than 0".to_string());
            }
            if n > 1000 {
                return Err("max_tool_iterations too large (max 1000)".to_string());
            }
        }
        _ => {
            if key.starts_with("providers.")
                && key.ends_with(".base_url")
                && !value.starts_with("http://")
                && !value.starts_with("https://")
            {
                return Err("base_url must start with http:// or https://".to_string());
            }
        }
    }
    Ok(())
}

/// Format validation errors for user display.
pub fn format_errors(errors: &[FieldError]) -> String {
    errors
        .iter()
        .map(|e| format!("  - {}: {}", e.field, e.message))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    fn make_valid_config() -> Config {
        Config::default()
    }

    #[test]
    fn test_valid_config_passes() {
        let config = make_valid_config();
        assert!(validate(&config).is_ok());
    }

    #[test]
    fn test_invalid_temperature() {
        let mut config = make_valid_config();
        config.core.temperature = Some(3.0);
        let result = validate(&config);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "core.temperature"));
    }

    #[test]
    fn test_invalid_max_tokens() {
        let mut config = make_valid_config();
        config.core.max_tokens = Some(2_000_000);
        let result = validate(&config);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "core.max_tokens"));
    }

    #[test]
    fn test_invalid_log_level() {
        let mut config = make_valid_config();
        config.core.log_level = "invalid".to_string();
        let result = validate(&config);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "core.log_level"));
    }

    #[test]
    fn test_invalid_style() {
        let mut config = make_valid_config();
        config.display.style = "fancy".to_string();
        let result = validate(&config);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "display.style"));
    }

    #[test]
    fn test_zero_context_size() {
        let mut config = make_valid_config();
        config.session.context_size = 0;
        let result = validate(&config);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "session.context_size"));
    }

    #[test]
    fn test_overlapping_permissions() {
        let mut config = make_valid_config();
        config.permissions.always_allow.push("npm".to_string());
        config.permissions.always_deny.push("npm".to_string());
        let result = validate(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_permission_rule_pattern() {
        let mut config = make_valid_config();
        config
            .permissions
            .rules
            .push(pleiades_agent_permissions::PermissionRule {
                tool: "bash".to_string(),
                pattern: "[".to_string(),
                action: pleiades_agent_permissions::PermissionAction::Allow,
                cwd: None,
                network: None,
                mcp_server: None,
                mcp_tool: None,
            });
        let result = validate(&config);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .iter()
                .any(|error| error.field == "permissions.rules")
        );
    }

    #[test]
    fn test_validate_field() {
        assert!(validate_field("core.max_tokens", "1000").is_ok());
        assert!(validate_field("core.max_tokens", "0").is_err());
        assert!(validate_field("core.temperature", "0.5").is_ok());
        assert!(validate_field("core.temperature", "-1").is_err());
        assert!(validate_field("display.style", "rich").is_ok());
        assert!(validate_field("display.style", "invalid").is_err());
    }
}
