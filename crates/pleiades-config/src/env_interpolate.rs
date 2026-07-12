/// Interpolate environment variables in string values.
///
/// Supports `${VAR_NAME}` and `$VAR_NAME` syntax.
/// Recursively resolves nested references.
pub fn interpolate(value: &str) -> String {
    let mut result = value.to_string();
    let mut attempts = 0;
    let max_attempts = 10;

    loop {
        if attempts >= max_attempts {
            break;
        }

        let before = result.clone();

        // Handle ${VAR_NAME} syntax
        while let Some(start) = result.find("${") {
            if let Some(end) = result[start..].find('}') {
                let var_name = &result[start + 2..start + end];
                let env_value = std::env::var(var_name).unwrap_or_default();
                result.replace_range(start..start + end + 1, &env_value);
            } else {
                break;
            }
        }

        // Handle $VAR_NAME syntax (only at word boundaries)
        let mut temp = String::new();
        let mut chars = result.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '$' {
                let mut var_name = String::new();
                while let Some(&next) = chars.peek() {
                    if next.is_alphanumeric() || next == '_' {
                        var_name.push(next);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if !var_name.is_empty() {
                    let env_value = std::env::var(&var_name).unwrap_or_default();
                    temp.push_str(&env_value);
                } else {
                    temp.push(ch);
                }
            } else {
                temp.push(ch);
            }
        }
        result = temp;

        if result == before {
            break;
        }
        attempts += 1;
    }

    result
}

/// Interpolate all string values in a configuration JSON value.
pub fn interpolate_config(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::String(s) => {
            *s = interpolate(s);
        }
        serde_json::Value::Object(map) => {
            for val in map.values_mut() {
                interpolate_config(val);
            }
        }
        serde_json::Value::Array(arr) => {
            for val in arr.iter_mut() {
                interpolate_config(val);
            }
        }
        _ => {}
    }
}

/// Resolve API keys from environment variables, config values, or keychain.
pub fn resolve_api_key(config_value: Option<String>, env_var: &str) -> Option<String> {
    // First check if config value exists and is not an env ref
    if let Some(value) = config_value {
        if !value.starts_with("${") && !value.starts_with('$') {
            return Some(value);
        }
        let interpolated = interpolate(&value);
        if !interpolated.is_empty() && interpolated != value {
            return Some(interpolated);
        }
    }

    // Fall back to environment variable
    std::env::var(env_var).ok()
}

/// Replace sensitive values in a config for safe display.
pub fn mask_secrets(value: &str) -> String {
    if value.len() > 8 {
        let prefix = &value[..4];
        let suffix = &value[value.len() - 4..];
        format!("{}...{}", prefix, suffix)
    } else {
        "****".to_string()
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
    fn test_interpolate_braces() {
        set_env("PLEIADES_TEST_VAR", "test_value");
        assert_eq!(interpolate("${PLEIADES_TEST_VAR}"), "test_value");
        remove_env("PLEIADES_TEST_VAR");
    }

    #[test]
    fn test_interpolate_no_braces() {
        set_env("PLEIADES_TEST_VAR2", "hello");
        assert_eq!(interpolate("$PLEIADES_TEST_VAR2"), "hello");
        remove_env("PLEIADES_TEST_VAR2");
    }

    #[test]
    fn test_interpolate_missing_var() {
        assert_eq!(interpolate("${NONEXISTENT_VAR}"), "");
    }

    #[test]
    fn test_interpolate_no_var() {
        assert_eq!(interpolate("plain text"), "plain text");
    }

    #[test]
    fn test_interpolate_multiple() {
        set_env("PLEIADES_A", "hello");
        set_env("PLEIADES_B", "world");
        assert_eq!(interpolate("${PLEIADES_A} ${PLEIADES_B}"), "hello world");
        remove_env("PLEIADES_A");
        remove_env("PLEIADES_B");
    }

    #[test]
    fn test_resolve_api_key_config_first() {
        set_env("PLEIADES_TEST_KEY", "from_env");
        let result = resolve_api_key(Some("from_config".to_string()), "PLEIADES_TEST_KEY");
        assert_eq!(result, Some("from_config".to_string()));
        remove_env("PLEIADES_TEST_KEY");
    }

    #[test]
    fn test_resolve_api_key_env_fallback() {
        remove_env("PLEIADES_TEST_KEY2");
        let result = resolve_api_key(None, "PLEIADES_TEST_KEY2");
        assert_eq!(result, None);
    }

    #[test]
    fn test_mask_secrets() {
        assert_eq!(mask_secrets("sk-1234567890abcdef"), "sk-1...cdef");
    }

    #[test]
    fn test_mask_short_secrets() {
        assert_eq!(mask_secrets("abc"), "****");
    }
}
