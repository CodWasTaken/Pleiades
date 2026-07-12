use std::ffi::OsStr;
use std::io::Write;
use std::process::Command;

use crate::manifest::PluginHooks;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookEvent {
    PreToolUse,
    PostToolUse,
    PostToolUseFailure,
}

impl HookEvent {
    fn as_str(self) -> &'static str {
        match self {
            Self::PreToolUse => "PreToolUse",
            Self::PostToolUse => "PostToolUse",
            Self::PostToolUseFailure => "PostToolUseFailure",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookRunResult {
    denied: bool,
    failed: bool,
    messages: Vec<String>,
}

impl HookRunResult {
    pub fn allow(messages: Vec<String>) -> Self {
        Self {
            denied: false,
            failed: false,
            messages,
        }
    }

    pub fn is_denied(&self) -> bool {
        self.denied
    }

    pub fn is_failed(&self) -> bool {
        self.failed
    }

    pub fn messages(&self) -> &[String] {
        &self.messages
    }

    pub fn into_messages(self) -> Vec<String> {
        self.messages
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookRunner {
    hooks: PluginHooks,
}

impl HookRunner {
    pub fn new(hooks: PluginHooks) -> Self {
        Self { hooks }
    }

    pub fn pre_tool_use(&self, tool_name: &str, tool_input: &str) -> HookRunResult {
        Self::run_commands(
            HookEvent::PreToolUse,
            &self.hooks.pre_tool_use,
            tool_name,
            tool_input,
            None,
            false,
        )
    }

    pub fn post_tool_use(
        &self,
        tool_name: &str,
        tool_input: &str,
        tool_output: &str,
        is_error: bool,
    ) -> HookRunResult {
        Self::run_commands(
            HookEvent::PostToolUse,
            &self.hooks.post_tool_use,
            tool_name,
            tool_input,
            Some(tool_output),
            is_error,
        )
    }

    pub fn post_tool_use_failure(
        &self,
        tool_name: &str,
        tool_input: &str,
        tool_error: &str,
    ) -> HookRunResult {
        Self::run_commands(
            HookEvent::PostToolUseFailure,
            &self.hooks.post_tool_use_failure,
            tool_name,
            tool_input,
            Some(tool_error),
            true,
        )
    }

    fn run_commands(
        event: HookEvent,
        commands: &[String],
        tool_name: &str,
        tool_input: &str,
        tool_output: Option<&str>,
        is_error: bool,
    ) -> HookRunResult {
        if commands.is_empty() {
            return HookRunResult::allow(Vec::new());
        }

        let payload = hook_payload(event, tool_name, tool_input, tool_output, is_error).to_string();
        let mut messages = Vec::new();

        for command in commands {
            match run_single_command(
                command, event, tool_name, tool_input, tool_output, is_error, &payload,
            ) {
                HookCommandOutcome::Allow { message } => {
                    if let Some(msg) = message {
                        messages.push(msg);
                    }
                }
                HookCommandOutcome::Deny { message } => {
                    messages.push(message.unwrap_or_else(|| {
                        format!("{} hook denied tool `{tool_name}`", event.as_str())
                    }));
                    return HookRunResult {
                        denied: true,
                        failed: false,
                        messages,
                    };
                }
                HookCommandOutcome::Failed { message } => {
                    messages.push(message);
                    return HookRunResult {
                        denied: false,
                        failed: true,
                        messages,
                    };
                }
            }
        }

        HookRunResult::allow(messages)
    }
}

enum HookCommandOutcome {
    Allow { message: Option<String> },
    Deny { message: Option<String> },
    Failed { message: String },
}

fn run_single_command(
    command: &str,
    event: HookEvent,
    tool_name: &str,
    tool_input: &str,
    tool_output: Option<&str>,
    is_error: bool,
    payload: &str,
) -> HookCommandOutcome {
    let mut child = build_shell_command(command);
    child.stdin(std::process::Stdio::piped());
    child.stdout(std::process::Stdio::piped());
    child.stderr(std::process::Stdio::piped());

    child.env("HOOK_EVENT", event.as_str());
    child.env("HOOK_TOOL_NAME", tool_name);
    child.env("HOOK_TOOL_INPUT", tool_input);
    child.env("HOOK_TOOL_IS_ERROR", if is_error { "1" } else { "0" });
    if let Some(output) = tool_output {
        child.env("HOOK_TOOL_OUTPUT", output);
    }

    let mut spawned = match child.spawn() {
        Ok(c) => c,
        Err(e) => {
            return HookCommandOutcome::Failed {
                message: format!(
                    "{} hook `{command}` failed to start for `{tool_name}`: {e}",
                    event.as_str()
                ),
            };
        }
    };

    if let Some(mut stdin) = spawned.stdin.take() {
        let _ = stdin.write_all(payload.as_bytes());
    }

    match spawned.wait_with_output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let message = (!stdout.is_empty()).then_some(stdout);

            match output.status.code() {
                Some(0) => HookCommandOutcome::Allow { message },
                Some(2) => HookCommandOutcome::Deny { message },
                Some(code) => {
                    let detail = message
                        .as_deref()
                        .filter(|s| !s.is_empty())
                        .or_else(|| (!stderr.is_empty()).then_some(stderr.as_str()))
                        .unwrap_or("");
                    HookCommandOutcome::Failed {
                        message: if detail.is_empty() {
                            format!("Hook `{command}` exited with status {code}")
                        } else {
                            format!("Hook `{command}` exited with status {code}: {detail}")
                        },
                    }
                }
                None => HookCommandOutcome::Failed {
                    message: format!(
                        "{} hook `{command}` was terminated by signal",
                        event.as_str()
                    ),
                },
            }
        }
        Err(e) => HookCommandOutcome::Failed {
            message: format!(
                "{} hook `{command}` failed for `{tool_name}`: {e}",
                event.as_str()
            ),
        },
    }
}

fn hook_payload(
    event: HookEvent,
    tool_name: &str,
    tool_input: &str,
    tool_output: Option<&str>,
    is_error: bool,
) -> serde_json::Value {
    match event {
        HookEvent::PostToolUseFailure => serde_json::json!({
            "hook_event_name": event.as_str(),
            "tool_name": tool_name,
            "tool_input": parse_tool_input(tool_input),
            "tool_input_json": tool_input,
            "tool_error": tool_output,
            "tool_result_is_error": true,
        }),
        _ => serde_json::json!({
            "hook_event_name": event.as_str(),
            "tool_name": tool_name,
            "tool_input": parse_tool_input(tool_input),
            "tool_input_json": tool_input,
            "tool_output": tool_output,
            "tool_result_is_error": is_error,
        }),
    }
}

fn parse_tool_input(tool_input: &str) -> serde_json::Value {
    serde_json::from_str(tool_input).unwrap_or_else(|_| serde_json::json!({"raw": tool_input}))
}

fn build_shell_command(command: &str) -> Command {
    let (shell, flag) = if cfg!(windows) {
        ("cmd", "/C")
    } else {
        ("sh", "-c")
    };
    let mut cmd = Command::new(shell);
    cmd.arg(flag);
    cmd.arg(command);
    cmd
}

pub trait HookCommandExt {
    fn env<K, V>(&mut self, key: K, value: V) -> &mut Self
    where
        K: AsRef<OsStr>,
        V: AsRef<OsStr>;
}

impl HookCommandExt for Command {
    fn env<K, V>(&mut self, key: K, value: V) -> &mut Self
    where
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.env(key, value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pre_tool_use_allows_when_hooks_succeed() {
        let runner = HookRunner::new(PluginHooks {
            pre_tool_use: vec!["printf 'all good'".to_string()],
            post_tool_use: Vec::new(),
            post_tool_use_failure: Vec::new(),
        });

        let result = runner.pre_tool_use("Read", r#"{"path":"test"}"#);

        assert!(!result.is_denied());
        assert!(!result.is_failed());
        assert_eq!(result.messages(), &["all good"]);
    }

    #[test]
    fn pre_tool_use_denies_when_hook_exits_two() {
        let runner = HookRunner::new(PluginHooks {
            pre_tool_use: vec!["printf 'blocked'; exit 2".to_string()],
            post_tool_use: Vec::new(),
            post_tool_use_failure: Vec::new(),
        });

        let result = runner.pre_tool_use("Bash", r#"{"command":"pwd"}"#);

        assert!(result.is_denied());
        assert_eq!(result.messages(), &["blocked"]);
    }

    #[test]
    fn pre_tool_use_fails_on_nonzero_exit() {
        let runner = HookRunner::new(PluginHooks {
            pre_tool_use: vec!["exit 1".to_string()],
            post_tool_use: Vec::new(),
            post_tool_use_failure: Vec::new(),
        });

        let result = runner.pre_tool_use("Bash", r#"{"command":"pwd"}"#);

        assert!(result.is_failed());
    }

    #[test]
    fn pre_tool_use_first_failure_stops_later_hooks() {
        let runner = HookRunner::new(PluginHooks {
            pre_tool_use: vec![
                "exit 2".to_string(),
                "printf 'should not run'".to_string(),
            ],
            post_tool_use: Vec::new(),
            post_tool_use_failure: Vec::new(),
        });

        let result = runner.pre_tool_use("Read", "{}");

        assert!(result.is_denied());
        assert!(!result.messages().iter().any(|m| m.contains("should not run")));
    }

    #[test]
    fn post_tool_use_runs_after_successful_tool() {
        let runner = HookRunner::new(PluginHooks {
            pre_tool_use: Vec::new(),
            post_tool_use: vec!["printf 'tool completed'".to_string()],
            post_tool_use_failure: Vec::new(),
        });

        let result = runner.post_tool_use("Write", r#"{"path":"f"}"#, "wrote 1 file", false);

        assert!(!result.is_denied());
        assert!(result.messages().iter().any(|m| m.contains("tool completed")));
    }

    #[test]
    fn post_tool_use_failure_runs_on_tool_error() {
        let runner = HookRunner::new(PluginHooks {
            pre_tool_use: Vec::new(),
            post_tool_use: Vec::new(),
            post_tool_use_failure: vec!["printf 'tool errored'".to_string()],
        });

        let result =
            runner.post_tool_use_failure("Bash", r#"{"command":"invalid"}"#, "command not found");

        assert!(!result.is_denied());
        assert!(result.messages().iter().any(|m| m.contains("tool errored")));
    }

    #[test]
    fn empty_hooks_returns_allow_with_no_messages() {
        let runner = HookRunner::new(PluginHooks::default());

        let result = runner.pre_tool_use("Read", "{}");

        assert!(!result.is_denied());
        assert!(!result.is_failed());
        assert!(result.messages().is_empty());
    }
}
