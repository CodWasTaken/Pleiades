//! Secret-redacting JSONL audit log.

use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use pleiades_agent_config::Config;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// One persisted audit entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditEvent {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub kind: AuditKind,
    pub action: String,
    pub target: Option<String>,
    pub details: Value,
}

/// Coarse audit category.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AuditKind {
    Command,
    Permission,
    Tool,
    Shell,
    File,
    Plugin,
    Mcp,
    Checkpoint,
    Provider,
    Model,
    Mode,
    Task,
    Session,
    Config,
}

/// Append-only JSONL audit writer.
#[derive(Debug, Clone)]
pub struct AuditLogger {
    dir: PathBuf,
}

impl AuditLogger {
    pub fn default_dir() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("pleiades")
            .join("audit")
    }

    pub fn from_config(config: &Config) -> Self {
        let dir = config
            .session
            .history_dir
            .as_ref()
            .and_then(|path| Path::new(path).parent().map(|parent| parent.join("audit")))
            .unwrap_or_else(Self::default_dir);
        Self { dir }
    }

    pub fn from_dir(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn log(
        &self,
        kind: AuditKind,
        action: impl Into<String>,
        target: Option<String>,
        details: Value,
    ) -> std::io::Result<AuditEvent> {
        std::fs::create_dir_all(&self.dir)?;
        let event = AuditEvent {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            kind,
            action: action.into(),
            target: target.map(|value| redact_text(&value)),
            details: redact_value(details),
        };
        let line = serde_json::to_string(&event)
            .map_err(|error| std::io::Error::other(error.to_string()))?;
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.dir.join("audit.jsonl"))?;
        writeln!(file, "{line}")?;
        Ok(event)
    }
}

pub fn redact_value(value: Value) -> Value {
    match value {
        Value::String(value) => Value::String(redact_text(&value)),
        Value::Array(items) => Value::Array(items.into_iter().map(redact_value).collect()),
        Value::Object(map) => Value::Object(
            map.into_iter()
                .map(|(key, value)| {
                    let redacted = if is_sensitive_key(&key) {
                        Value::String("[REDACTED]".to_string())
                    } else {
                        redact_value(value)
                    };
                    (key, redacted)
                })
                .collect(),
        ),
        other => other,
    }
}

pub fn redact_text(value: &str) -> String {
    let mut output = Vec::new();
    for token in value.split_whitespace() {
        let trimmed = token.trim_matches(|character: char| {
            matches!(
                character,
                '"' | '\'' | ',' | ';' | ')' | '(' | '[' | ']' | '{' | '}'
            )
        });
        if looks_secret(trimmed) {
            output.push(token.replace(trimmed, "[REDACTED]"));
        } else if let Some((name, secret)) = trimmed.split_once('=')
            && is_sensitive_key(name)
            && !secret.is_empty()
        {
            output.push(token.replace(secret, "[REDACTED]"));
        } else {
            output.push(token.to_string());
        }
    }
    output.join(" ")
}

fn is_sensitive_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    key.contains("api_key")
        || key.contains("apikey")
        || key.contains("access_token")
        || key.contains("refresh_token")
        || key.contains("token")
        || key.contains("secret")
        || key.contains("password")
        || key.contains("credential")
        || key == "authorization"
        || key == "cookie"
}

fn looks_secret(value: &str) -> bool {
    value.starts_with("sk-")
        || value.starts_with("sk_")
        || value.starts_with("sk-proj-")
        || value.starts_with("ghp_")
        || value.starts_with("github_pat_")
        || value.starts_with("xoxb-")
        || value.starts_with("Bearer ")
        || (value.len() >= 32
            && value.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.')
            }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_sensitive_keys_and_token_like_strings() {
        let value = serde_json::json!({
            "api_key": "sk-proj-secret",
            "command": "curl -H Authorization=sk-proj-secret https://example.test",
            "nested": {"token": "abc123"}
        });
        let redacted = redact_value(value).to_string();
        assert!(!redacted.contains("sk-proj-secret"));
        assert!(!redacted.contains("abc123"));
        assert!(redacted.contains("[REDACTED]"));
    }

    #[test]
    fn writes_jsonl_without_secret_payloads() {
        let temp = tempfile::tempdir().unwrap();
        let logger = AuditLogger::from_dir(temp.path());
        logger
            .log(
                AuditKind::Command,
                "dispatch",
                Some("echo sk-proj-secret".to_string()),
                serde_json::json!({"api_key": "sk-proj-secret"}),
            )
            .unwrap();
        let content = std::fs::read_to_string(temp.path().join("audit.jsonl")).unwrap();
        assert!(!content.contains("sk-proj-secret"));
        assert!(content.contains("dispatch"));
    }
}
