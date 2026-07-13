//! OpenAI subscription access through the official Codex CLI.
//!
//! This adapter deliberately delegates authentication and execution to Codex. It never reads,
//! copies, or refreshes the OAuth credentials stored by Codex.

use std::process::Stdio;

use async_trait::async_trait;
use pleiades_agent_core::conversation::{Message, MessageRole};
use pleiades_agent_core::error::Error;
use pleiades_agent_core::model::{ModelCapabilities, ModelInfo};
use pleiades_agent_core::provider::{
    ChatRequest, ChatResponse, Provider, ProviderCapabilities, StreamEvent, Usage,
};
use tokio::io::AsyncWriteExt;

const DEFAULT_MODEL: &str = "codex-default";

/// Provider that uses the user's authenticated Codex CLI session.
pub struct CodexCliProvider {
    command: String,
}

impl CodexCliProvider {
    /// Create a provider using `codex` from `PATH`.
    pub fn new() -> Self {
        Self::with_command(
            std::env::var("PLEIADES_CODEX_BIN").unwrap_or_else(|_| "codex".to_string()),
        )
    }

    /// Create a provider using an explicit Codex executable.
    pub fn with_command(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
        }
    }

    async fn execute(&self, request: &ChatRequest) -> Result<(String, Option<Usage>), Error> {
        let cwd = std::env::current_dir().map_err(Error::from)?;
        let mut command = tokio::process::Command::new(&self.command);
        command
            .arg("exec")
            .arg("--json")
            .arg("--ephemeral")
            .arg("--sandbox")
            .arg("read-only")
            .arg("--skip-git-repo-check")
            .arg("--ignore-rules")
            .arg("--ignore-user-config")
            .arg("--color")
            .arg("never")
            .arg("--cd")
            .arg(cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if !request.model.is_empty() && request.model != DEFAULT_MODEL {
            command.arg("--model").arg(&request.model);
        }
        command.arg("-");

        let mut child = command.spawn().map_err(|error| {
            Error::config(format!(
                "OpenAI subscription mode requires the official Codex CLI. Install it, then run `codex login`: {error}"
            ))
        })?;

        let prompt = build_prompt(request);
        child
            .stdin
            .take()
            .ok_or_else(|| Error::internal("Could not open Codex stdin"))?
            .write_all(prompt.as_bytes())
            .await
            .map_err(Error::from)?;

        let output = child.wait_with_output().await.map_err(Error::from)?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let message = if stderr.is_empty() {
                "Codex exited without an error message. Run `pleiades auth status` and `codex login` to repair authentication.".to_string()
            } else {
                stderr
            };
            return Err(Error::AuthError {
                provider: "openai-subscription".to_string(),
                message,
            });
        }

        parse_codex_output(&String::from_utf8_lossy(&output.stdout))
    }
}

impl Default for CodexCliProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for CodexCliProvider {
    fn name(&self) -> &str {
        "openai-subscription"
    }

    fn display_name(&self) -> &str {
        "OpenAI (ChatGPT subscription via Codex)"
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            streaming: false,
            tools: false,
            vision: false,
            embeddings: false,
            thinking: true,
            json_mode: false,
            function_calling: false,
        }
    }

    fn default_model(&self) -> &str {
        DEFAULT_MODEL
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, Error> {
        Ok(vec![ModelInfo {
            id: DEFAULT_MODEL.to_string(),
            provider: self.name().to_string(),
            display_name: Some("Codex account default".to_string()),
            description: Some(
                "The default model selected by the authenticated official Codex CLI".to_string(),
            ),
            capabilities: ModelCapabilities {
                max_context_length: 0,
                max_output_tokens: 0,
                supports_tools: false,
                supports_vision: false,
                supports_streaming: false,
                supports_thinking: true,
                supports_json_mode: false,
            },
            pricing: None,
        }])
    }

    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, Error> {
        let (text, usage) = self.execute(&request).await?;
        Ok(ChatResponse {
            message: Message::assistant(text),
            usage,
            finish_reason: Some("stop".to_string()),
        })
    }

    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<StreamEvent>, Error> {
        let (tx, rx) = tokio::sync::mpsc::channel(8);
        let result = self.execute(&request).await;
        tokio::spawn(async move {
            match result {
                Ok((text, usage)) => {
                    if !text.is_empty() {
                        let _ = tx.send(StreamEvent::Token(text)).await;
                    }
                    let _ = tx
                        .send(StreamEvent::Done {
                            finish_reason: "stop".to_string(),
                            usage,
                        })
                        .await;
                }
                Err(error) => {
                    let _ = tx
                        .send(StreamEvent::Error {
                            message: error.to_string(),
                            code: Some("codex_cli_error".to_string()),
                        })
                        .await;
                }
            }
        });
        Ok(rx)
    }
}

fn build_prompt(request: &ChatRequest) -> String {
    let mut prompt = String::from(
        "Act only as the language-model backend for Pleiades. Do not inspect files, run commands, browse, or call tools. Respond directly to the latest user message using the conversation below.\n\n",
    );
    if let Some(system) = request
        .system_prompt
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        prompt.push_str("SYSTEM:\n");
        prompt.push_str(system);
        prompt.push_str("\n\n");
    }
    for message in &request.messages {
        let role = match message.role {
            MessageRole::System => "SYSTEM",
            MessageRole::User => "USER",
            MessageRole::Assistant => "ASSISTANT",
            MessageRole::Tool => "TOOL",
        };
        prompt.push_str(role);
        prompt.push_str(":\n");
        prompt.push_str(&message.text_content());
        prompt.push_str("\n\n");
    }
    prompt.push_str("Return only the assistant response.");
    prompt
}

fn parse_codex_output(output: &str) -> Result<(String, Option<Usage>), Error> {
    let mut messages = Vec::new();
    let mut usage = None;

    for line in output.lines().filter(|line| !line.trim().is_empty()) {
        let Ok(event) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        match event.get("type").and_then(|value| value.as_str()) {
            Some("item.completed") => {
                let item = &event["item"];
                if item.get("type").and_then(|value| value.as_str()) == Some("agent_message") {
                    if let Some(text) = item.get("text").and_then(|value| value.as_str()) {
                        messages.push(text.to_string());
                    }
                }
            }
            Some("turn.completed") => {
                let value = &event["usage"];
                if value.is_object() {
                    usage = Some(Usage {
                        input_tokens: value
                            .get("input_tokens")
                            .and_then(|token| token.as_u64())
                            .unwrap_or(0),
                        output_tokens: value
                            .get("output_tokens")
                            .and_then(|token| token.as_u64())
                            .unwrap_or(0),
                        cache_read_tokens: value
                            .get("cached_input_tokens")
                            .and_then(|token| token.as_u64()),
                        cache_write_tokens: None,
                    });
                }
            }
            Some("turn.failed") | Some("error") => {
                let message = event
                    .pointer("/error/message")
                    .or_else(|| event.get("message"))
                    .and_then(|value| value.as_str())
                    .unwrap_or("Codex request failed");
                return Err(Error::provider(message));
            }
            _ => {}
        }
    }

    if messages.is_empty() {
        return Err(Error::provider(
            "Codex returned no assistant message. Run `pleiades auth status` to verify your login.",
        ));
    }
    Ok((messages.join("\n"), usage))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_agent_message_and_usage() {
        let output = r#"{"type":"thread.started","thread_id":"test"}
{"type":"item.completed","item":{"type":"agent_message","text":"Hello"}}
{"type":"turn.completed","usage":{"input_tokens":12,"output_tokens":3,"cached_input_tokens":2}}"#;
        let (text, usage) = parse_codex_output(output).expect("valid output");
        assert_eq!(text, "Hello");
        let usage = usage.expect("usage");
        assert_eq!(usage.input_tokens, 12);
        assert_eq!(usage.output_tokens, 3);
        assert_eq!(usage.cache_read_tokens, Some(2));
    }

    #[test]
    fn prompt_preserves_conversation_roles() {
        let request = ChatRequest {
            model: DEFAULT_MODEL.to_string(),
            messages: vec![Message::user("hello"), Message::assistant("hi")],
            system_prompt: Some("be useful".to_string()),
            temperature: None,
            top_p: None,
            max_tokens: None,
            stop: None,
            tools: None,
        };
        let prompt = build_prompt(&request);
        assert!(prompt.contains("SYSTEM:\nbe useful"));
        assert!(prompt.contains("USER:\nhello"));
        assert!(prompt.contains("ASSISTANT:\nhi"));
    }
}
