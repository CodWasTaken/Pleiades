use async_trait::async_trait;
use pleiades_agent_core::error::Error;
use pleiades_agent_core::tool::{PermissionLevel, Tool, ToolContext, ToolResult};

/// Apply targeted edits to a file using string replacement.
pub struct EditTool;

#[async_trait]
impl Tool for EditTool {
    fn name(&self) -> &str {
        "edit"
    }

    fn description(&self) -> &str {
        "Apply a targeted edit to a file by replacing text"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to edit"
                },
                "old_string": {
                    "type": "string",
                    "description": "Text to replace (must exist in the file)"
                },
                "new_string": {
                    "type": "string",
                    "description": "New text to replace with"
                }
            },
            "required": ["path", "old_string", "new_string"]
        })
    }

    fn is_readonly(&self) -> bool {
        false
    }

    fn is_concurrency_safe(&self) -> bool {
        false
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::WorkspaceWrite
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

        let old_string = input
            .get("old_string")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::invalid_input("Missing required 'old_string' parameter"))?;

        let new_string = input
            .get("new_string")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::invalid_input("Missing required 'new_string' parameter"))?;

        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::io(format!("Failed to read '{}': {}", path, e)))?;

        if !content.contains(old_string) {
            return Err(Error::tool(format!(
                "Could not find old_string in '{}'. The text to replace was not found.",
                path
            )));
        }

        let new_content = content.replace(old_string, new_string);
        std::fs::write(path, &new_content)
            .map_err(|e| Error::io(format!("Failed to write '{}': {}", path, e)))?;

        let metadata = serde_json::json!({
            "path": path,
            "replacements": content.len().abs_diff(new_content.len()) / old_string.len().max(1),
        });

        Ok(ToolResult {
            success: true,
            content: format!("Successfully edited {}", path),
            error: None,
            metadata: Some(metadata),
        })
    }
}
