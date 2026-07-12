use crate::types::Config;

/// Error from configuration validation.
#[derive(Debug)]
pub struct ValidationError {
    pub errors: Vec<String>,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.errors.join(", "))
    }
}

/// Validate a configuration, returning errors if any.
pub fn validate(config: &Config) -> Result<(), ValidationError> {
    let mut errors = Vec::new();

    // Validate permission mode
    match config.permissions.mode.as_str() {
        "allow" | "ask" | "deny" | "plan" => {}
        other => errors.push(format!(
            "Invalid permission mode '{}'. Must be one of: allow, ask, deny, plan",
            other
        )),
    }

    // Validate temperature range
    if let Some(temp) = config.core.temperature {
        if !(0.0..=2.0).contains(&temp) {
            errors.push(format!(
                "Temperature must be between 0.0 and 2.0, got {}",
                temp
            ));
        }
    }

    // Validate max_tokens
    if let Some(max) = config.core.max_tokens {
        if max == 0 {
            errors.push("max_tokens must be > 0".to_string());
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(ValidationError { errors })
    }
}
