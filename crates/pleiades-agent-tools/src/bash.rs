use async_trait::async_trait;
use pleiades_agent_core::error::Error;
use pleiades_agent_core::tool::{PermissionLevel, Tool, ToolContext, ToolResult};

/// Execute shell commands with timeout and output capture.
pub struct BashTool;

impl BashTool {
    fn validate_command(cmd: &str) -> Result<(), Error> {
        let cmd_lower = cmd.to_lowercase();

        // Block dangerous commands
        let dangerous_patterns = [
            "rm -rf /",
            "rm -rf --no-preserve-root",
            ":(){ :|:& };:", // Fork bomb
            "> /dev/sda",
            "mkfs",
            "dd if=/dev/zero",
            "dd if=/dev/random",
            "chmod -R 000 /",
            "chown -R 0:0 /",
        ];

        for pattern in &dangerous_patterns {
            if cmd_lower.contains(pattern) {
                return Err(Error::tool(format!(
                    "Command blocked: potentially dangerous pattern '{}'",
                    pattern
                )));
            }
        }

        Ok(())
    }

    fn build_command(
        command: &str,
        workdir: &std::path::Path,
        ctx: &ToolContext,
    ) -> Result<tokio::process::Command, Error> {
        if ctx.sandbox_mode == "danger-full-access" {
            let mut process = tokio::process::Command::new("sh");
            process.arg("-c").arg(command).current_dir(workdir);
            process.kill_on_drop(true);
            return Ok(process);
        }

        #[cfg(target_os = "linux")]
        {
            let executable = ["/usr/bin/bwrap", "/bin/bwrap"]
                .into_iter()
                .find(|path| std::path::Path::new(path).is_file())
                .ok_or_else(|| {
                    Error::unsupported(
                        "Workspace shell commands require bubblewrap on Linux; install `bwrap` or explicitly switch to YOLO mode",
                    )
                })?;
            let workspace = ctx
                .working_directory
                .canonicalize()
                .map_err(|error| Error::io(format!("Could not resolve workspace root: {error}")))?;
            let mut process = tokio::process::Command::new(executable);
            process
                .args([
                    "--die-with-parent",
                    "--new-session",
                    "--ro-bind",
                    "/",
                    "/",
                    "--dev",
                    "/dev",
                    "--proc",
                    "/proc",
                    "--tmpfs",
                    "/tmp",
                    "--bind",
                ])
                .arg(&workspace)
                .arg(&workspace)
                .arg("--chdir")
                .arg(workdir)
                .args(["sh", "-c", command]);
            process.kill_on_drop(true);
            Ok(process)
        }

        #[cfg(target_os = "macos")]
        {
            let workspace = ctx
                .working_directory
                .canonicalize()
                .map_err(|error| Error::io(format!("Could not resolve workspace root: {error}")))?;
            let escaped = workspace.to_string_lossy().replace('"', "\\\"");
            let profile = format!(
                "(version 1) (allow default) (deny file-write*) (allow file-write* (subpath \"{escaped}\")) (allow file-write* (subpath \"/tmp\"))"
            );
            let mut process = tokio::process::Command::new("/usr/bin/sandbox-exec");
            process
                .args(["-p", &profile, "sh", "-c", command])
                .current_dir(workdir);
            process.kill_on_drop(true);
            return Ok(process);
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        Err(Error::unsupported(
            "Workspace shell isolation is not available on this platform; explicitly switch to YOLO mode to run shell commands",
        ))
    }
}

#[async_trait]
impl Tool for BashTool {
    fn name(&self) -> &str {
        "bash"
    }

    fn description(&self) -> &str {
        "Execute a shell command with timeout and output capture"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "Shell command to execute"
                },
                "timeout": {
                    "type": "integer",
                    "description": "Timeout in seconds",
                    "default": 30
                },
                "workdir": {
                    "type": "string",
                    "description": "Working directory (defaults to current)"
                }
            },
            "required": ["command"]
        })
    }

    fn is_readonly(&self) -> bool {
        false
    }

    fn is_concurrency_safe(&self) -> bool {
        false
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::Dangerous
    }

    async fn execute(
        &self,
        input: serde_json::Value,
        ctx: &ToolContext,
    ) -> Result<ToolResult, Error> {
        let command = input
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::invalid_input("Missing required 'command' parameter"))?;

        Self::validate_command(command)?;

        let timeout_secs = input.get("timeout").and_then(|v| v.as_u64()).unwrap_or(30);
        let workdir = input.get("workdir").and_then(|value| value.as_str());
        let workdir = if ctx.sandbox_mode == "danger-full-access" {
            workdir
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| ctx.working_directory.clone())
                .canonicalize()
                .map_err(|error| Error::io(format!("Could not resolve command workdir: {error}")))?
        } else {
            crate::workspace::resolve_path(workdir.unwrap_or("."), ctx, false)?
        };

        let mut process = Self::build_command(command, &workdir, ctx)?;

        let output = tokio::time::timeout(
            std::time::Duration::from_secs(timeout_secs),
            process.output(),
        )
        .await
        .map_err(|_| Error::timeout(format!("Command timed out after {} seconds", timeout_secs)))?
        .map_err(|e| Error::io(format!("Failed to execute command: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let content = if output.status.success() {
            stdout
        } else {
            format!(
                "Exit code: {}\n{}\n{}",
                output.status.code().unwrap_or(-1),
                stdout,
                stderr
            )
        };

        let metadata = serde_json::json!({
            "exit_code": output.status.code(),
            "success": output.status.success(),
        });

        Ok(ToolResult {
            success: output.status.success(),
            content,
            error: if output.status.success() {
                None
            } else {
                Some(stderr)
            },
            metadata: Some(metadata),
        })
    }
}
