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
        _ctx: &ToolContext,
    ) -> Result<ToolResult, Error> {
        let command = input
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::invalid_input("Missing required 'command' parameter"))?;

        Self::validate_command(command)?;

        let timeout_secs = input.get("timeout").and_then(|v| v.as_u64()).unwrap_or(30);

        let output = tokio::time::timeout(
            std::time::Duration::from_secs(timeout_secs),
            tokio::process::Command::new("sh")
                .arg("-c")
                .arg(command)
                .output(),
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
