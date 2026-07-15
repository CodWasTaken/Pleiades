//! Definition-of-done verification support.

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::process::Command;

const VERIFY_TIMEOUT_SECS: u64 = 120;
const OUTPUT_LIMIT: usize = 32 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationScope {
    Full,
    Test,
    Run,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationCommand {
    pub label: String,
    pub program: String,
    pub args: Vec<String>,
}

impl VerificationCommand {
    pub fn new(label: impl Into<String>, program: impl Into<String>, args: &[&str]) -> Self {
        Self {
            label: label.into(),
            program: program.into(),
            args: args.iter().map(|arg| (*arg).to_string()).collect(),
        }
    }

    pub fn shell(command: impl Into<String>) -> Self {
        let command = command.into();
        Self {
            label: command.clone(),
            program: shell_program().to_string(),
            args: shell_args(&command),
        }
    }

    pub fn display(&self) -> String {
        std::iter::once(self.program.as_str())
            .chain(self.args.iter().map(String::as_str))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationStepResult {
    pub label: String,
    pub command: String,
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationReport {
    pub project_kind: String,
    pub diff_summary: String,
    pub changed_files: Vec<String>,
    pub planned_commands: Vec<VerificationCommand>,
    pub results: Vec<VerificationStepResult>,
    pub skipped_reason: Option<String>,
}

impl VerificationReport {
    pub fn success(&self) -> bool {
        self.skipped_reason.is_none() && self.results.iter().all(|result| result.success)
    }
}

#[derive(Debug, Clone)]
pub struct VerificationService {
    workspace: PathBuf,
}

impl VerificationService {
    pub fn new(workspace: impl Into<PathBuf>) -> Self {
        Self {
            workspace: workspace.into(),
        }
    }

    pub async fn verify(&self, scope: VerificationScope) -> VerificationReport {
        let changed_files = git_changed_files(&self.workspace).await;
        let diff_summary = git_diff_summary(&self.workspace).await;
        let project_kind = detect_project_kind(&self.workspace);
        let planned_commands = self.plan(scope, &project_kind);
        let mut results = Vec::new();
        for command in &planned_commands {
            results.push(run_command(&self.workspace, command).await);
        }
        VerificationReport {
            project_kind,
            diff_summary,
            changed_files,
            planned_commands,
            results,
            skipped_reason: None,
        }
    }

    pub async fn plan_only(
        &self,
        scope: VerificationScope,
        reason: impl Into<String>,
    ) -> VerificationReport {
        let changed_files = git_changed_files(&self.workspace).await;
        let diff_summary = git_diff_summary(&self.workspace).await;
        let project_kind = detect_project_kind(&self.workspace);
        let planned_commands = self.plan(scope, &project_kind);
        VerificationReport {
            project_kind,
            diff_summary,
            changed_files,
            planned_commands,
            results: Vec::new(),
            skipped_reason: Some(reason.into()),
        }
    }

    pub async fn run_shell(&self, command: String) -> VerificationReport {
        let changed_files = git_changed_files(&self.workspace).await;
        let diff_summary = git_diff_summary(&self.workspace).await;
        let project_kind = detect_project_kind(&self.workspace);
        let planned_commands = vec![VerificationCommand::shell(command)];
        let mut results = Vec::new();
        for command in &planned_commands {
            results.push(run_command(&self.workspace, command).await);
        }
        VerificationReport {
            project_kind,
            diff_summary,
            changed_files,
            planned_commands,
            results,
            skipped_reason: None,
        }
    }

    fn plan(&self, scope: VerificationScope, project_kind: &str) -> Vec<VerificationCommand> {
        match (scope, project_kind) {
            (VerificationScope::Test, "rust") => vec![VerificationCommand::new(
                "Rust tests",
                "cargo",
                &["test", "--workspace"],
            )],
            (VerificationScope::Full, "rust") => vec![
                VerificationCommand::new(
                    "Rust format check",
                    "cargo",
                    &["fmt", "--all", "--", "--check"],
                ),
                VerificationCommand::new(
                    "Rust lint",
                    "cargo",
                    &[
                        "clippy",
                        "--workspace",
                        "--all-targets",
                        "--all-features",
                        "--",
                        "-D",
                        "warnings",
                    ],
                ),
                VerificationCommand::new("Rust tests", "cargo", &["test", "--workspace"]),
            ],
            (VerificationScope::Test, "node") => {
                vec![VerificationCommand::new("Node tests", "npm", &["test"])]
            }
            (VerificationScope::Full, "node") => vec![
                VerificationCommand::new("Node tests", "npm", &["test"]),
                VerificationCommand::new("Node lint", "npm", &["run", "lint", "--if-present"]),
            ],
            _ => Vec::new(),
        }
    }
}

fn detect_project_kind(workspace: &Path) -> String {
    if workspace.join("Cargo.toml").is_file() {
        "rust".to_string()
    } else if workspace.join("package.json").is_file() {
        "node".to_string()
    } else {
        "unknown".to_string()
    }
}

async fn run_command(workspace: &Path, command: &VerificationCommand) -> VerificationStepResult {
    let start = std::time::Instant::now();
    let output = tokio::time::timeout(
        Duration::from_secs(VERIFY_TIMEOUT_SECS),
        Command::new(&command.program)
            .args(&command.args)
            .current_dir(workspace)
            .output(),
    )
    .await;

    match output {
        Ok(Ok(output)) => VerificationStepResult {
            label: command.label.clone(),
            command: command.display(),
            success: output.status.success(),
            exit_code: output.status.code(),
            stdout: truncate(String::from_utf8_lossy(&output.stdout).to_string()),
            stderr: truncate(String::from_utf8_lossy(&output.stderr).to_string()),
            duration_ms: start.elapsed().as_millis() as u64,
        },
        Ok(Err(error)) => VerificationStepResult {
            label: command.label.clone(),
            command: command.display(),
            success: false,
            exit_code: None,
            stdout: String::new(),
            stderr: error.to_string(),
            duration_ms: start.elapsed().as_millis() as u64,
        },
        Err(_) => VerificationStepResult {
            label: command.label.clone(),
            command: command.display(),
            success: false,
            exit_code: None,
            stdout: String::new(),
            stderr: format!("timed out after {VERIFY_TIMEOUT_SECS}s"),
            duration_ms: start.elapsed().as_millis() as u64,
        },
    }
}

async fn git_changed_files(workspace: &Path) -> Vec<String> {
    command_stdout(workspace, "git", &["status", "--porcelain=v1"])
        .await
        .lines()
        .filter_map(|line| line.get(3..))
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect()
}

async fn git_diff_summary(workspace: &Path) -> String {
    let summary = command_stdout(workspace, "git", &["diff", "--stat", "--"])
        .await
        .trim()
        .to_string();
    if summary.is_empty() {
        "No unstaged tracked diff detected.".to_string()
    } else {
        truncate(summary)
    }
}

async fn command_stdout(workspace: &Path, program: &str, args: &[&str]) -> String {
    Command::new(program)
        .args(args)
        .current_dir(workspace)
        .output()
        .await
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).to_string())
        .unwrap_or_default()
}

fn truncate(mut value: String) -> String {
    if value.len() <= OUTPUT_LIMIT {
        return value;
    }
    let boundary = value
        .char_indices()
        .map(|(index, _)| index)
        .take_while(|index| *index <= OUTPUT_LIMIT)
        .last()
        .unwrap_or(0);
    value.truncate(boundary);
    value.push_str("\n… verification output truncated …");
    value
}

#[cfg(windows)]
fn shell_program() -> &'static str {
    "cmd"
}

#[cfg(not(windows))]
fn shell_program() -> &'static str {
    "sh"
}

#[cfg(windows)]
fn shell_args(command: &str) -> Vec<String> {
    vec!["/C".to_string(), command.to_string()]
}

#[cfg(not(windows))]
fn shell_args(command: &str) -> Vec<String> {
    vec!["-c".to_string(), command.to_string()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_rust_and_plans_full_verification() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("Cargo.toml"), "[workspace]\n").unwrap();
        let service = VerificationService::new(temp.path());
        let plan = service.plan(VerificationScope::Full, &detect_project_kind(temp.path()));
        assert_eq!(plan.len(), 3);
        assert!(
            plan.iter()
                .any(|command| command.display().contains("cargo test"))
        );
    }

    #[test]
    fn report_success_requires_executed_successful_results() {
        let report = VerificationReport {
            project_kind: "rust".to_string(),
            diff_summary: String::new(),
            changed_files: Vec::new(),
            planned_commands: Vec::new(),
            results: Vec::new(),
            skipped_reason: Some("plan".to_string()),
        };
        assert!(!report.success());
    }
}
