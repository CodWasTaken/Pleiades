use async_trait::async_trait;
use pleiades_core::error::Error;
use pleiades_core::tool::{PermissionLevel, Tool, ToolContext, ToolResult};

/// Search for files matching glob patterns.
pub struct GlobTool;

#[async_trait]
impl Tool for GlobTool {
    fn name(&self) -> &str {
        "glob"
    }

    fn description(&self) -> &str {
        "Search for files and directories matching a glob pattern"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Glob pattern to search for (e.g., '**/*.rs')"
                },
                "path": {
                    "type": "string",
                    "description": "Directory to search in (defaults to current)"
                }
            },
            "required": ["pattern"]
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

    async fn execute(&self, input: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, Error> {
        let pattern = input.get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::invalid_input("Missing required 'pattern' parameter"))?;

        let base_path = input.get("path")
            .and_then(|v| v.as_str())
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        let full_pattern = if pattern.starts_with('/') || pattern.starts_with("./") || pattern.starts_with("~/") {
            pattern.to_string()
        } else {
            base_path.join(pattern).to_string_lossy().to_string()
        };

        let mut results = Vec::new();

        match glob::glob(&full_pattern) {
            Ok(entries) => {
                for entry in entries {
                    match entry {
                        Ok(path) => {
                            let is_dir = path.is_dir();
                            results.push(format!(
                                "{}{}",
                                path.display(),
                                if is_dir { "/" } else { "" }
                            ));
                        }
                        Err(e) => {
                            return Err(Error::io(format!("Glob error: {}", e)));
                        }
                    }
                }
            }
            Err(e) => {
                return Err(Error::invalid_input(format!("Invalid glob pattern '{}': {}", pattern, e)));
            }
        }

        results.sort();
        let content = if results.is_empty() {
            format!("No files found matching '{}'", pattern)
        } else {
            format!(
                "Found {} result{}:\n{}",
                results.len(),
                if results.len() == 1 { "" } else { "s" },
                results.join("\n")
            )
        };

        let metadata = serde_json::json!({
            "pattern": pattern,
            "count": results.len(),
        });

        Ok(ToolResult {
            success: true,
            content,
            error: None,
            metadata: Some(metadata),
        })
    }
}
