//! Background process manager.
//!
//! The manager is designed to be owned by the long-lived runtime actor. It
//! keeps dev servers, watchers, and other user-started processes alive across
//! chat turns while exposing bounded logs and explicit stop/restart controls.

use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use pleiades_agent_core::Error;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

const MAX_LOG_LINES: usize = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProcessStatus {
    Running,
    Exited,
    Stopped,
    Failed,
}

impl ProcessStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Exited => "exited",
            Self::Stopped => "stopped",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessReport {
    pub id: String,
    pub command: String,
    pub cwd: PathBuf,
    pub status: ProcessStatus,
    pub pid: Option<u32>,
    pub exit_code: Option<i32>,
    pub started_at_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessLogs {
    pub id: String,
    pub lines: Vec<String>,
}

#[derive(Debug)]
struct ProcessRecord {
    report: ProcessReport,
    child: Option<Child>,
    logs: VecDeque<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ProcessManager {
    inner: Arc<Mutex<ProcessManagerInner>>,
}

#[derive(Debug, Default)]
struct ProcessManagerInner {
    next_id: u64,
    records: HashMap<String, ProcessRecord>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn start(
        &self,
        command: impl Into<String>,
        cwd: impl Into<PathBuf>,
    ) -> Result<ProcessReport, Error> {
        let command = command.into();
        if command.trim().is_empty() {
            return Err(Error::invalid_input("usage: /process start <command>"));
        }
        let cwd = cwd.into();
        let mut child = shell_command(&command)
            .current_dir(&cwd)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(Error::from)?;
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let pid = child.id();

        let mut inner = self.inner.lock().await;
        inner.next_id += 1;
        let id = format!("proc-{}", inner.next_id);
        let report = ProcessReport {
            id: id.clone(),
            command: command.clone(),
            cwd,
            status: ProcessStatus::Running,
            pid,
            exit_code: None,
            started_at_ms: now_ms(),
        };
        inner.records.insert(
            id.clone(),
            ProcessRecord {
                report: report.clone(),
                child: Some(child),
                logs: VecDeque::new(),
            },
        );
        drop(inner);

        if let Some(stdout) = stdout {
            self.spawn_log_reader(id.clone(), "stdout", stdout);
        }
        if let Some(stderr) = stderr {
            self.spawn_log_reader(id.clone(), "stderr", stderr);
        }
        Ok(report)
    }

    pub async fn list(&self) -> Vec<ProcessReport> {
        let mut inner = self.inner.lock().await;
        refresh_records(&mut inner);
        let mut reports = inner
            .records
            .values()
            .map(|record| record.report.clone())
            .collect::<Vec<_>>();
        reports.sort_by(|a, b| a.id.cmp(&b.id));
        reports
    }

    pub async fn logs(&self, id: &str) -> Result<ProcessLogs, Error> {
        let inner = self.inner.lock().await;
        let record = inner
            .records
            .get(id)
            .ok_or_else(|| Error::invalid_input(format!("process `{id}` not found")))?;
        Ok(ProcessLogs {
            id: id.to_string(),
            lines: record.logs.iter().cloned().collect(),
        })
    }

    pub async fn stop(&self, id: &str) -> Result<ProcessReport, Error> {
        let mut child = {
            let mut inner = self.inner.lock().await;
            let record = inner
                .records
                .get_mut(id)
                .ok_or_else(|| Error::invalid_input(format!("process `{id}` not found")))?;
            record.child.take()
        };
        if let Some(child) = child.as_mut() {
            let _ = child.start_kill();
            let _ = child.wait().await;
        }
        let mut inner = self.inner.lock().await;
        let record = inner
            .records
            .get_mut(id)
            .ok_or_else(|| Error::invalid_input(format!("process `{id}` not found")))?;
        record.report.status = ProcessStatus::Stopped;
        record.report.exit_code = None;
        Ok(record.report.clone())
    }

    pub async fn restart(&self, id: &str) -> Result<ProcessReport, Error> {
        let (command, cwd) = {
            let inner = self.inner.lock().await;
            let record = inner
                .records
                .get(id)
                .ok_or_else(|| Error::invalid_input(format!("process `{id}` not found")))?;
            (record.report.command.clone(), record.report.cwd.clone())
        };
        let _ = self.stop(id).await;
        self.start(command, cwd).await
    }

    pub async fn stop_all(&self) {
        let ids = self
            .list()
            .await
            .into_iter()
            .filter(|report| report.status == ProcessStatus::Running)
            .map(|report| report.id)
            .collect::<Vec<_>>();
        for id in ids {
            let _ = self.stop(&id).await;
        }
    }

    fn spawn_log_reader<R>(&self, id: String, stream: &'static str, reader: R)
    where
        R: tokio::io::AsyncRead + Send + Unpin + 'static,
    {
        let manager = self.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(reader).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                manager.append_log(&id, format!("[{stream}] {line}")).await;
            }
        });
    }

    async fn append_log(&self, id: &str, line: String) {
        let mut inner = self.inner.lock().await;
        if let Some(record) = inner.records.get_mut(id) {
            if record.logs.len() >= MAX_LOG_LINES {
                record.logs.pop_front();
            }
            record.logs.push_back(line);
        }
    }
}

fn refresh_records(inner: &mut ProcessManagerInner) {
    for record in inner.records.values_mut() {
        if record.report.status != ProcessStatus::Running {
            continue;
        }
        if let Some(child) = record.child.as_mut() {
            match child.try_wait() {
                Ok(Some(status)) => {
                    record.report.status = ProcessStatus::Exited;
                    record.report.exit_code = status.code();
                    record.child = None;
                }
                Ok(None) => {}
                Err(_) => {
                    record.report.status = ProcessStatus::Failed;
                    record.child = None;
                }
            }
        }
    }
}

fn shell_command(command: &str) -> Command {
    #[cfg(windows)]
    {
        let mut child = Command::new("cmd");
        child.args(["/C", command]);
        child
    }
    #[cfg(not(windows))]
    {
        let mut child = Command::new("sh");
        child.args(["-c", command]);
        child
    }
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[tokio::test]
    async fn starts_captures_logs_and_stops() {
        let manager = ProcessManager::new();
        #[cfg(windows)]
        let command = "echo hello && ping -n 6 127.0.0.1 >nul";
        #[cfg(not(windows))]
        let command = "printf 'hello\\n'; sleep 5";
        let report = manager
            .start(command, std::env::current_dir().unwrap())
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        let logs = manager.logs(&report.id).await.unwrap();
        assert!(logs.lines.iter().any(|line| line.contains("hello")));
        let stopped = manager.stop(&report.id).await.unwrap();
        assert_eq!(stopped.status, ProcessStatus::Stopped);
    }

    #[tokio::test]
    async fn refreshes_exited_processes() {
        let manager = ProcessManager::new();
        let report = manager
            .start("echo done", std::env::current_dir().unwrap())
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        let reports = manager.list().await;
        let report = reports.iter().find(|item| item.id == report.id).unwrap();
        assert_eq!(report.status, ProcessStatus::Exited);
    }
}
