//! Built-in tool implementations for Pleiades.
//!
//! Provides the tools that the AI can use to interact with
//! the environment: read/write/edit files, run commands, search, etc.

pub mod read;
pub mod write;
pub mod edit;
pub mod bash;
pub mod glob_tool;
pub mod grep_tool;
pub mod diff;
pub mod search;
pub mod fetch;

use pleiades_core::tool::Tool;

/// Registry of built-in tools.
pub struct ToolRegistry {
    tools: Vec<Box<dyn Tool>>,
}

impl ToolRegistry {
    /// Create a new empty tool registry.
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }

    /// Register a single tool.
    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.push(tool);
    }

    /// Register all built-in default tools.
    pub fn register_defaults(&mut self) {
        self.register(Box::new(read::ReadTool));
        self.register(Box::new(write::WriteTool));
        self.register(Box::new(edit::EditTool));
        self.register(Box::new(bash::BashTool));
        self.register(Box::new(glob_tool::GlobTool));
        self.register(Box::new(grep_tool::GrepTool));
        self.register(Box::new(diff::DiffTool));
        self.register(Box::new(search::SearchTool::new()));
        self.register(Box::new(fetch::FetchTool::new()));
    }

    /// Get a tool by name.
    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.iter().find(|t| t.name() == name).map(|t| t.as_ref())
    }

    /// List all registered tools.
    pub fn list(&self) -> Vec<&dyn Tool> {
        self.tools.iter().map(|t| t.as_ref()).collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
