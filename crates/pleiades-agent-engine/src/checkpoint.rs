use std::path::{Path, PathBuf};
use std::process::Command;

use pleiades_agent_config::Config;
use pleiades_agent_core::conversation::Conversation;
use pleiades_agent_core::error::Error;
use serde::{Deserialize, Serialize};

use crate::AgentMode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointRecord {
    pub id: String,
    pub name: Option<String>,
    pub created_at_ms: u128,
    pub conversation: Conversation,
    pub conversation_position: usize,
    pub provider: String,
    pub model: String,
    pub mode: AgentMode,
    pub git_head: Option<String>,
    pub git_branch: Option<String>,
    pub changed_files: Vec<String>,
    pub unstaged_diff: Option<String>,
    pub staged_diff: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CheckpointSummary {
    pub id: String,
    pub name: Option<String>,
    pub created_at_ms: u128,
    pub message_count: usize,
    pub provider: String,
    pub model: String,
    pub mode: AgentMode,
    pub changed_files: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CheckpointStore {
    dir: PathBuf,
}

impl CheckpointStore {
    pub fn from_config(config: &Config) -> Self {
        let dir = config
            .session
            .history_dir
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                dirs::data_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("pleiades")
                    .join("sessions")
            })
            .join("checkpoints");
        Self { dir }
    }

    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn create(
        &self,
        conversation: &Conversation,
        provider: &str,
        model: &str,
        mode: AgentMode,
        name: Option<String>,
    ) -> Result<CheckpointRecord, Error> {
        std::fs::create_dir_all(&self.dir)
            .map_err(|error| Error::io(format!("failed to create checkpoints dir: {error}")))?;
        let git = GitSnapshot::capture();
        let mut changed_files = git.changed_files;
        changed_files.sort();
        changed_files.dedup();
        let record = CheckpointRecord {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            created_at_ms: now_ms(),
            conversation: conversation.clone(),
            conversation_position: conversation.messages.len(),
            provider: provider.to_string(),
            model: model.to_string(),
            mode,
            git_head: git.head,
            git_branch: git.branch,
            changed_files,
            unstaged_diff: git.unstaged_diff,
            staged_diff: git.staged_diff,
        };
        self.save(&record)?;
        Ok(record)
    }

    pub fn save(&self, record: &CheckpointRecord) -> Result<(), Error> {
        std::fs::create_dir_all(&self.dir)
            .map_err(|error| Error::io(format!("failed to create checkpoints dir: {error}")))?;
        let json = serde_json::to_string_pretty(record)
            .map_err(|error| Error::Serialization(error.to_string()))?;
        std::fs::write(self.path(&record.id), json)
            .map_err(|error| Error::io(format!("failed to save checkpoint: {error}")))
    }

    pub fn load(&self, id: &str) -> Result<CheckpointRecord, Error> {
        let path = self.path(id);
        if !path.exists() {
            return Err(Error::invalid_input(format!("checkpoint `{id}` not found")));
        }
        let json = std::fs::read_to_string(path)
            .map_err(|error| Error::io(format!("failed to read checkpoint: {error}")))?;
        serde_json::from_str(&json).map_err(|error| Error::Serialization(error.to_string()))
    }

    pub fn list(&self) -> Result<Vec<CheckpointSummary>, Error> {
        if !self.dir.exists() {
            return Ok(Vec::new());
        }
        let mut summaries = Vec::new();
        for entry in std::fs::read_dir(&self.dir)
            .map_err(|error| Error::io(format!("failed to read checkpoints dir: {error}")))?
            .flatten()
        {
            let path = entry.path();
            if path
                .extension()
                .is_some_and(|extension| extension == "json")
            {
                if let Ok(record) = self.load(
                    path.file_stem()
                        .and_then(|stem| stem.to_str())
                        .unwrap_or_default(),
                ) {
                    summaries.push(record.summary());
                }
            }
        }
        summaries.sort_by_key(|summary| std::cmp::Reverse(summary.created_at_ms));
        Ok(summaries)
    }

    pub fn delete(&self, id: &str) -> Result<(), Error> {
        let path = self.path(id);
        if !path.exists() {
            return Err(Error::invalid_input(format!("checkpoint `{id}` not found")));
        }
        std::fs::remove_file(path)
            .map_err(|error| Error::io(format!("failed to delete checkpoint: {error}")))
    }

    fn path(&self, id: &str) -> PathBuf {
        self.dir.join(format!("{id}.json"))
    }

    pub fn restore_workspace(&self, record: &CheckpointRecord) -> Result<Option<PathBuf>, Error> {
        let Some(expected_head) = record.git_head.as_deref() else {
            return Ok(None);
        };
        let current_head = git_output(["rev-parse", "HEAD"]).ok_or_else(|| {
            Error::invalid_input("checkpoint was created in Git, but this is not a Git repository")
        })?;
        if current_head != expected_head {
            return Err(Error::invalid_input(format!(
                "checkpoint was created at Git HEAD {expected_head}, but current HEAD is {current_head}"
            )));
        }
        let current_status = git_output(["status", "--porcelain=v1"]).unwrap_or_default();
        for path in current_status
            .lines()
            .filter(|line| line.starts_with("??"))
            .filter_map(status_path)
        {
            if !record.changed_files.iter().any(|changed| changed == &path) {
                return Err(Error::invalid_input(format!(
                    "untracked file `{path}` is not part of checkpoint `{}`; move it before restoring",
                    record.id
                )));
            }
        }

        let current_staged = non_empty(git_output([
            "diff",
            "--staged",
            "--binary",
            "--no-ext-diff",
        ]));
        let current_unstaged = non_empty(git_output(["diff", "--binary", "--no-ext-diff"]));
        let backup = self.backup_current_diff(&record.id, &current_staged, &current_unstaged)?;

        if let Some(diff) = current_staged.as_deref() {
            git_apply(diff, &["--cached", "-R"])?;
        }
        if let Some(diff) = current_unstaged.as_deref() {
            git_apply(diff, &["-R"])?;
        }
        if let Some(diff) = record.staged_diff.as_deref() {
            git_apply(diff, &["--cached"])?;
        }
        if let Some(diff) = record.unstaged_diff.as_deref() {
            git_apply(diff, &[])?;
        }
        Ok(backup)
    }

    fn backup_current_diff(
        &self,
        id: &str,
        staged: &Option<String>,
        unstaged: &Option<String>,
    ) -> Result<Option<PathBuf>, Error> {
        if staged.is_none() && unstaged.is_none() {
            return Ok(None);
        }
        let backup_dir = self.dir.join("restore-backups");
        std::fs::create_dir_all(&backup_dir).map_err(|error| {
            Error::io(format!("failed to create checkpoint backup dir: {error}"))
        })?;
        let path = backup_dir.join(format!("{id}-{}.patch", now_ms()));
        let mut content = String::new();
        if let Some(diff) = staged {
            content.push_str("# staged diff\n");
            content.push_str(diff);
            content.push('\n');
        }
        if let Some(diff) = unstaged {
            content.push_str("# unstaged diff\n");
            content.push_str(diff);
            content.push('\n');
        }
        std::fs::write(&path, content)
            .map_err(|error| Error::io(format!("failed to write checkpoint backup: {error}")))?;
        Ok(Some(path))
    }
}

impl CheckpointRecord {
    pub fn summary(&self) -> CheckpointSummary {
        CheckpointSummary {
            id: self.id.clone(),
            name: self.name.clone(),
            created_at_ms: self.created_at_ms,
            message_count: self.conversation.messages.len(),
            provider: self.provider.clone(),
            model: self.model.clone(),
            mode: self.mode,
            changed_files: self.changed_files.clone(),
        }
    }
}

#[derive(Default)]
struct GitSnapshot {
    head: Option<String>,
    branch: Option<String>,
    changed_files: Vec<String>,
    unstaged_diff: Option<String>,
    staged_diff: Option<String>,
}

impl GitSnapshot {
    fn capture() -> Self {
        if git_output(["rev-parse", "--show-toplevel"]).is_none() {
            return Self::default();
        }
        Self {
            head: git_output(["rev-parse", "HEAD"]),
            branch: git_output(["branch", "--show-current"]),
            changed_files: git_output(["status", "--porcelain=v1"])
                .map(|output| output.lines().filter_map(status_path).collect())
                .unwrap_or_default(),
            unstaged_diff: non_empty(git_output(["diff", "--binary", "--no-ext-diff"])),
            staged_diff: non_empty(git_output([
                "diff",
                "--staged",
                "--binary",
                "--no-ext-diff",
            ])),
        }
    }
}

fn git_output<const N: usize>(args: [&str; N]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn git_apply(diff: &str, args: &[&str]) -> Result<(), Error> {
    let mut command = Command::new("git");
    command.arg("apply");
    command.args(args);
    let mut child = command
        .stdin(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|error| Error::io(format!("failed to start git apply: {error}")))?;
    {
        use std::io::Write;
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| Error::io("failed to open git apply stdin"))?;
        stdin
            .write_all(diff.as_bytes())
            .map_err(|error| Error::io(format!("failed to write patch to git apply: {error}")))?;
    }
    let output = child
        .wait_with_output()
        .map_err(|error| Error::io(format!("failed to wait for git apply: {error}")))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(Error::tool(format!(
            "git apply failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )))
    }
}

fn status_path(line: &str) -> Option<String> {
    let value = line.get(3..)?.trim();
    if let Some((_, right)) = value.split_once(" -> ") {
        Some(right.to_string())
    } else if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.filter(|text| !text.trim().is_empty())
}

fn now_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use pleiades_agent_core::conversation::{Conversation, Message};

    use super::*;

    #[test]
    fn checkpoint_roundtrip_preserves_conversation() {
        let temp = tempfile::tempdir().unwrap();
        let store = CheckpointStore::new(temp.path().to_path_buf());
        let mut conversation = Conversation::new("test");
        conversation.add_message(Message::user("hello"));

        let record = store
            .create(
                &conversation,
                "mock-provider",
                "mock-model",
                AgentMode::Agent,
                Some("before edit".to_string()),
            )
            .unwrap();
        let loaded = store.load(&record.id).unwrap();

        assert_eq!(loaded.name.as_deref(), Some("before edit"));
        assert_eq!(loaded.conversation.messages.len(), 1);
        assert_eq!(store.list().unwrap().len(), 1);
        store.delete(&record.id).unwrap();
        assert!(store.list().unwrap().is_empty());
    }
}
