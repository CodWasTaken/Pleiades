use async_trait::async_trait;
use pleiades_agent_core::error::Error;
use pleiades_agent_core::tool::{PermissionLevel, Tool, ToolContext, ToolResult};
use serde_json::Value;

/// Tool for showing diffs between files or comparing file contents.
pub struct DiffTool;

#[async_trait]
impl Tool for DiffTool {
    fn name(&self) -> &str {
        "diff"
    }

    fn description(&self) -> &str {
        "Show differences between two files or between a file and git HEAD. Returns a unified diff output."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path1": {
                    "type": "string",
                    "description": "First file path (for file-to-file diff) or the file to compare with git"
                },
                "path2": {
                    "type": "string",
                    "description": "Second file path (optional - if omitted, compares path1 with git HEAD)"
                },
                "context": {
                    "type": "integer",
                    "description": "Number of context lines (default: 3)",
                    "default": 3
                }
            },
            "required": ["path1"]
        })
    }

    fn is_readonly(&self) -> bool {
        true
    }

    fn is_concurrency_safe(&self) -> bool {
        true
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::ReadOnly
    }

    async fn execute(&self, input: Value, _ctx: &ToolContext) -> Result<ToolResult, Error> {
        let path1 = input
            .get("path1")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::invalid_input("Missing required field 'path1'"))?;

        let context = input.get("context").and_then(|v| v.as_i64()).unwrap_or(3);

        let context_arg = format!("-U{}", context);

        let output = if let Some(path2) = input.get("path2").and_then(|v| v.as_str()) {
            // File-to-file diff using git diff --no-index
            tokio::process::Command::new("git")
                .args(["diff", "--no-index", &context_arg, path1, path2])
                .output()
                .await
                .map_err(|e| Error::io(format!("Failed to run git diff: {}", e)))?
        } else {
            // Git diff (path1 vs git HEAD)
            tokio::process::Command::new("git")
                .args(["diff", &context_arg, path1])
                .output()
                .await
                .map_err(|e| Error::io(format!("Failed to run git diff: {}", e)))?
        };

        let content = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr);

        // If git diff failed or returned nothing useful, try a simple comparison
        if content.is_empty() && !stderr.is_empty() {
            return Ok(ToolResult {
                success: true,
                content: format!("Git diff failed: {}", stderr.trim()),
                error: None,
                metadata: Some(serde_json::json!({
                    "path1": path1,
                    "has_diff": false,
                    "lines": 0
                })),
            });
        }

        let lines_count = content.lines().count();
        let has_diff = lines_count > 0;

        Ok(ToolResult {
            success: true,
            content,
            error: None,
            metadata: Some(serde_json::json!({
                "path1": path1,
                "has_diff": has_diff,
                "lines": lines_count
            })),
        })
    }
}
