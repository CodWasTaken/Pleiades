use async_trait::async_trait;
use pleiades_core::error::Error;
use pleiades_core::tool::{PermissionLevel, Tool, ToolContext, ToolResult};

/// Search file contents using regex patterns.
pub struct GrepTool;

#[async_trait]
impl Tool for GrepTool {
    fn name(&self) -> &str {
        "grep"
    }

    fn description(&self) -> &str {
        "Search file contents for lines matching a regex pattern"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Regex pattern to search for"
                },
                "path": {
                    "type": "string",
                    "description": "Directory to search in (defaults to current)"
                },
                "include": {
                    "type": "string",
                    "description": "File pattern to include (e.g., '*.rs')"
                },
                "max_results": {
                    "type": "integer",
                    "description": "Maximum number of results to return",
                    "default": 50
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

    async fn execute(
        &self,
        input: serde_json::Value,
        _ctx: &ToolContext,
    ) -> Result<ToolResult, Error> {
        let pattern = input
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::invalid_input("Missing required 'pattern' parameter"))?;

        let base_path = input
            .get("path")
            .and_then(|v| v.as_str())
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        let include = input.get("include").and_then(|v| v.as_str());
        let max_results = input
            .get("max_results")
            .and_then(|v| v.as_u64())
            .unwrap_or(50) as usize;

        let regex = regex::Regex::new(pattern).map_err(|e| {
            Error::invalid_input(format!("Invalid regex pattern '{}': {}", pattern, e))
        })?;

        let mut results = Vec::new();
        let mut visit_dir = Vec::new();
        visit_dir.push(base_path.clone());

        while let Some(dir) = visit_dir.pop() {
            if results.len() >= max_results {
                break;
            }

            let entries = match std::fs::read_dir(&dir) {
                Ok(entries) => entries,
                Err(_) => continue,
            };

            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_dir() {
                    visit_dir.push(path);
                } else if path.is_file() {
                    // Check include filter
                    if let Some(inc) = include {
                        let glob = glob::Pattern::new(inc)
                            .unwrap_or_else(|_| glob::Pattern::new("*").unwrap());
                        if !glob
                            .matches(path.file_name().unwrap_or_default().to_str().unwrap_or(""))
                        {
                            continue;
                        }
                    }

                    let content = match std::fs::read_to_string(&path) {
                        Ok(c) => c,
                        Err(_) => continue,
                    };

                    for (line_num, line) in content.lines().enumerate() {
                        if results.len() >= max_results {
                            break;
                        }
                        if regex.is_match(line) {
                            let relative = path.strip_prefix(&base_path).unwrap_or(&path);
                            results.push(format!(
                                "{}:{}:{}",
                                relative.display(),
                                line_num + 1,
                                line
                            ));
                        }
                    }
                }
            }
        }

        let content = if results.is_empty() {
            format!("No matches found for pattern '{}'", pattern)
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
