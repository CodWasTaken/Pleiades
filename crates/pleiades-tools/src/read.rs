use async_trait::async_trait;
use pleiades_core::error::Error;
use pleiades_core::tool::{PermissionLevel, Tool, ToolContext, ToolResult};

/// Read file contents with optional line range filtering.
pub struct ReadTool;

#[async_trait]
impl Tool for ReadTool {
    fn name(&self) -> &str {
        "read"
    }

    fn description(&self) -> &str {
        "Read the contents of a file, optionally specifying a line range"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to read"
                },
                "offset": {
                    "type": "integer",
                    "description": "Starting line number (1-indexed)",
                    "minimum": 1
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of lines to read",
                    "minimum": 1
                }
            },
            "required": ["path"]
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

    async fn execute(
        &self,
        input: serde_json::Value,
        _ctx: &ToolContext,
    ) -> Result<ToolResult, Error> {
        let path = input
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::invalid_input("Missing required 'path' parameter"))?;

        let offset = input.get("offset").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let limit = input
            .get("limit")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);

        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::io(format!("Failed to read '{}': {}", path, e)))?;

        let result = if offset > 0 || limit.is_some() {
            let lines: Vec<&str> = content.lines().collect();
            let start = offset.saturating_sub(1);
            let end = limit.map(|l| start + l).unwrap_or(lines.len());
            let selected: Vec<&str> = lines
                .iter()
                .skip(start)
                .take(end - start)
                .copied()
                .collect();
            selected.join("\n")
        } else {
            content
        };

        let metadata = serde_json::json!({
            "path": path,
            "size_bytes": result.len(),
            "lines": result.lines().count(),
        });

        Ok(ToolResult {
            success: true,
            content: result,
            error: None,
            metadata: Some(metadata),
        })
    }
}
