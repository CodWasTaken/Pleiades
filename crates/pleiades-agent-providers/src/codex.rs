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
    AgentActivityKind, AgentActivityStatus, ChatRequest, ChatResponse, Provider,
    ProviderCapabilities, StreamEvent, Usage,
};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

const DEFAULT_MODEL: &str = "codex-default";

/// Provider that uses the user's authenticated Codex CLI session.
#[derive(Clone)]
pub struct CodexCliProvider {
    command: String,
    sandbox_mode: String,
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
            sandbox_mode: "workspace-write".to_string(),
        }
    }

    /// Set the Codex sandbox used for provider-managed agent actions.
    pub fn with_sandbox_mode(mut self, sandbox_mode: impl Into<String>) -> Self {
        let requested = sandbox_mode.into();
        self.sandbox_mode = normalize_sandbox_mode(&requested).to_string();
        self
    }

    fn build_command(&self) -> Result<tokio::process::Command, Error> {
        let cwd = std::env::current_dir().map_err(Error::from)?;
        let mut command = tokio::process::Command::new(&self.command);
        command
            .arg("exec")
            .arg("--json")
            .arg("--ephemeral")
            .arg("--sandbox")
            .arg(&self.sandbox_mode)
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
        Ok(command)
    }

    async fn execute(&self, request: &ChatRequest) -> Result<(String, Option<Usage>), Error> {
        let mut command = self.build_command()?;

        if !request.model.is_empty() && request.model != DEFAULT_MODEL {
            command.arg("--model").arg(&request.model);
        }
        command.arg("-");

        let mut child = command.spawn().map_err(|error| {
            Error::config(format!(
                "OpenAI subscription mode requires the official Codex CLI. Install it, then run `codex login`: {error}"
            ))
        })?;

        let prompt = build_agent_prompt(request);
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

    async fn stream_agent(
        &self,
        request: ChatRequest,
        tx: tokio::sync::mpsc::Sender<StreamEvent>,
    ) -> Result<(), Error> {
        let mut command = self.build_command()?;
        if !request.model.is_empty() && request.model != DEFAULT_MODEL {
            command.arg("--model").arg(&request.model);
        }
        command.arg("-");

        let mut child = command.spawn().map_err(|error| {
            Error::config(format!(
                "OpenAI subscription mode requires the official Codex CLI. Install it, then run `codex login`: {error}"
            ))
        })?;

        let prompt = build_agent_prompt(&request);
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| Error::internal("Could not open Codex stdin"))?;
        stdin
            .write_all(prompt.as_bytes())
            .await
            .map_err(Error::from)?;
        drop(stdin);

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| Error::internal("Could not open Codex stdout"))?;
        let mut stderr = child
            .stderr
            .take()
            .ok_or_else(|| Error::internal("Could not open Codex stderr"))?;
        let stderr_task = tokio::spawn(async move {
            let mut value = String::new();
            let _ = stderr.read_to_string(&mut value).await;
            value
        });

        let mut lines = BufReader::new(stdout).lines();
        let mut sent_done = false;
        loop {
            let line = tokio::select! {
                _ = tx.closed() => {
                    let _ = child.kill().await;
                    return Ok(());
                }
                line = lines.next_line() => line.map_err(Error::from)?,
            };
            let Some(line) = line else {
                break;
            };
            let Ok(event) = serde_json::from_str::<serde_json::Value>(&line) else {
                continue;
            };
            for stream_event in codex_stream_events(&event) {
                if matches!(stream_event, StreamEvent::Done { .. }) {
                    sent_done = true;
                }
                if tx.send(stream_event).await.is_err() {
                    let _ = child.kill().await;
                    return Ok(());
                }
            }
        }

        let status = child.wait().await.map_err(Error::from)?;
        let stderr = stderr_task.await.unwrap_or_default();
        if !status.success() {
            let message = if stderr.trim().is_empty() {
                "Codex agent exited without an error message".to_string()
            } else {
                stderr.trim().to_string()
            };
            return Err(Error::provider(message));
        }
        if !sent_done {
            tx.send(StreamEvent::Done {
                finish_reason: "stop".to_string(),
                usage: None,
            })
            .await
            .ok();
        }
        Ok(())
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
            streaming: true,
            tools: true,
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
                supports_tools: true,
                supports_vision: false,
                supports_streaming: true,
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
        let (tx, rx) = tokio::sync::mpsc::channel(64);
        let provider = self.clone();
        tokio::spawn(async move {
            let error_tx = tx.clone();
            if let Err(error) = provider.stream_agent(request, tx).await {
                let _ = error_tx
                    .send(StreamEvent::Error {
                        message: error.to_string(),
                        code: Some("codex_cli_error".to_string()),
                    })
                    .await;
            }
        });
        Ok(rx)
    }
}

fn normalize_sandbox_mode(value: &str) -> &'static str {
    match value {
        "read-only" | "plan" => "read-only",
        "danger-full-access" | "unrestricted" => "danger-full-access",
        _ => "workspace-write",
    }
}

fn build_agent_prompt(request: &ChatRequest) -> String {
    let mut prompt = String::from(
        "You are Pleiades, an autonomous terminal coding agent. Work inside the current workspace and carry the user's latest task through to a verified result. Inspect relevant files, create or edit files, and run appropriate commands or tests when useful. Do not merely explain what you would do. Do not claim that filesystem or command tools are disabled. Make reasonable decisions without asking follow-up questions unless a missing choice would materially change the result. Respect the active sandbox and never attempt to escape it. Summarize completed work clearly at the end.\n\n",
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
    prompt.push_str("Continue autonomously from the latest USER message and finish the task.");
    prompt
}

fn codex_stream_events(event: &serde_json::Value) -> Vec<StreamEvent> {
    let event_type = event.get("type").and_then(|value| value.as_str());
    let item = &event["item"];
    let item_type = item.get("type").and_then(|value| value.as_str());
    let id = item
        .get("id")
        .and_then(|value| value.as_str())
        .unwrap_or("codex")
        .to_string();

    match (event_type, item_type) {
        (Some("item.completed"), Some("agent_message")) => item
            .get("text")
            .and_then(|value| value.as_str())
            .map(|text| vec![StreamEvent::Token(format!("{text}\n\n"))])
            .unwrap_or_default(),
        (Some("item.started"), Some("command_execution")) => vec![StreamEvent::AgentActivity {
            id,
            kind: activity_kind_for_command(
                item.get("command")
                    .and_then(|value| value.as_str())
                    .unwrap_or_default(),
            ),
            title: item
                .get("command")
                .and_then(|value| value.as_str())
                .unwrap_or("command")
                .to_string(),
            detail: None,
            status: AgentActivityStatus::Running,
        }],
        (Some("item.completed"), Some("command_execution")) => {
            vec![StreamEvent::AgentActivity {
                id,
                kind: activity_kind_for_command(
                    item.get("command")
                        .and_then(|value| value.as_str())
                        .unwrap_or_default(),
                ),
                title: item
                    .get("command")
                    .and_then(|value| value.as_str())
                    .unwrap_or("command")
                    .to_string(),
                detail: item
                    .get("aggregated_output")
                    .and_then(|value| value.as_str())
                    .filter(|value| !value.trim().is_empty())
                    .map(truncate_activity_detail),
                status: if item.get("exit_code").and_then(|value| value.as_i64()) == Some(0) {
                    AgentActivityStatus::Completed
                } else {
                    AgentActivityStatus::Failed
                },
            }]
        }
        (Some("item.completed"), Some("file_change")) => {
            let changes = item
                .get("changes")
                .and_then(|value| value.as_array())
                .map(|changes| {
                    changes
                        .iter()
                        .map(|change| {
                            let kind = change
                                .get("kind")
                                .and_then(|value| value.as_str())
                                .unwrap_or("update");
                            let path = change
                                .get("path")
                                .and_then(|value| value.as_str())
                                .unwrap_or("file");
                            format!("{kind} {path}")
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_else(|| "workspace files".to_string());
            vec![StreamEvent::AgentActivity {
                id,
                kind: AgentActivityKind::Editing,
                title: changes,
                detail: None,
                status: AgentActivityStatus::Completed,
            }]
        }
        (Some("item.completed"), Some(kind @ ("web_search" | "mcp_tool_call"))) => {
            vec![StreamEvent::AgentActivity {
                id,
                kind: if kind == "web_search" {
                    AgentActivityKind::Searching
                } else {
                    AgentActivityKind::Tool
                },
                title: item
                    .get("query")
                    .or_else(|| item.get("tool"))
                    .and_then(|value| value.as_str())
                    .unwrap_or(kind)
                    .to_string(),
                detail: None,
                status: AgentActivityStatus::Completed,
            }]
        }
        (Some("turn.completed"), _) => vec![StreamEvent::Done {
            finish_reason: "stop".to_string(),
            usage: parse_usage(&event["usage"]),
        }],
        (Some("turn.failed" | "error"), _) => vec![StreamEvent::Error {
            message: event
                .pointer("/error/message")
                .or_else(|| event.get("message"))
                .and_then(|value| value.as_str())
                .unwrap_or("Codex agent failed")
                .to_string(),
            code: Some("codex_agent_error".to_string()),
        }],
        _ => Vec::new(),
    }
}

fn activity_kind_for_command(command: &str) -> AgentActivityKind {
    let command = command.to_ascii_lowercase();
    if ["test", "nextest", "pytest", "vitest", "jest", "cargo test"]
        .iter()
        .any(|needle| command.contains(needle))
    {
        AgentActivityKind::Testing
    } else if command.contains("git diff") || command.contains("git status") {
        AgentActivityKind::Reviewing
    } else if command.contains("rg ") || command.contains("grep ") || command.contains("find ") {
        AgentActivityKind::Searching
    } else {
        AgentActivityKind::Executing
    }
}

fn parse_usage(value: &serde_json::Value) -> Option<Usage> {
    value.is_object().then(|| Usage {
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
    })
}

fn truncate_activity_detail(value: &str) -> String {
    const LIMIT: usize = 800;
    let value = value.trim();
    if value.chars().count() <= LIMIT {
        value.to_string()
    } else {
        format!("{}…", value.chars().take(LIMIT).collect::<String>())
    }
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
                    usage = parse_usage(value);
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
        let prompt = build_agent_prompt(&request);
        assert!(prompt.contains("SYSTEM:\nbe useful"));
        assert!(prompt.contains("USER:\nhello"));
        assert!(prompt.contains("ASSISTANT:\nhi"));
        assert!(prompt.contains("autonomous terminal coding agent"));
    }

    #[test]
    fn maps_codex_command_and_file_events() {
        let command = serde_json::json!({
            "type": "item.completed",
            "item": {"id":"1", "type":"command_execution", "command":"cargo test", "aggregated_output":"ok", "exit_code":0}
        });
        let file = serde_json::json!({
            "type": "item.completed",
            "item": {"id":"2", "type":"file_change", "changes":[{"path":"src/main.rs", "kind":"update"}]}
        });
        assert!(matches!(
            codex_stream_events(&command)[0],
            StreamEvent::AgentActivity {
                status: AgentActivityStatus::Completed,
                ..
            }
        ));
        assert!(matches!(
            codex_stream_events(&file)[0],
            StreamEvent::AgentActivity {
                kind: AgentActivityKind::Editing,
                ..
            }
        ));
    }
}
