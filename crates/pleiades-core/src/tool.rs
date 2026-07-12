use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::Error;

/// Permission level required by a tool.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PermissionLevel {
    ReadOnly,
    WorkspaceWrite,
    Dangerous,
}

/// Permission mode for the current session.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PermissionMode {
    Allow,
    Ask,
    Deny,
}

/// A tool definition for inclusion in provider requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// Context provided to a tool during execution.
#[derive(Debug, Clone)]
pub struct ToolContext {
    pub cwd: std::path::PathBuf,
    pub working_directory: std::path::PathBuf,
    pub permission_mode: PermissionMode,
    pub config: std::sync::Arc<serde_json::Value>,
}

/// Result of a tool execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub content: String,
    pub error: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Generic tool interface for all built-in and plugin tools.
///
/// Tools are the primary way Pleiades interacts with the environment.
/// Each tool defines its input schema, permission requirements, and execution logic.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Unique name for this tool (used by the LLM to call it).
    fn name(&self) -> &str;

    /// Human-readable description of what this tool does.
    fn description(&self) -> &str;

    /// JSON Schema for the tool's input parameters.
    fn input_schema(&self) -> serde_json::Value;

    /// Whether this tool only reads data (no side effects).
    fn is_readonly(&self) -> bool;

    /// Whether this tool is safe to run concurrently.
    fn is_concurrency_safe(&self) -> bool;

    /// Permission level required to execute this tool.
    fn permission_level(&self) -> PermissionLevel;

    /// Execute the tool with the given input and context.
    async fn execute(&self, input: serde_json::Value, ctx: &ToolContext) -> Result<ToolResult, Error>;

    /// Convert this tool to a ToolDefinition for provider requests.
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name().to_string(),
            description: self.description().to_string(),
            input_schema: self.input_schema(),
        }
    }
}
