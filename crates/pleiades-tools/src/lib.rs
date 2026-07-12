//! Built-in tools for Pleiades.
//!
//! Each tool implements the `Tool` trait and provides
//! a specific capability to the AI assistant.

pub mod read;
pub mod write;
pub mod edit;
pub mod bash;
pub mod glob_tool;
pub mod grep_tool;

use pleiades_core::tool::Tool;

/// Registry of all built-in tools.
pub struct ToolRegistry {
    tools: Vec<Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: Vec::new(),
        }
    }

    /// Register a tool.
    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.push(tool);
    }

    /// Register all default built-in tools.
    pub fn register_defaults(&mut self) {
        self.register(Box::new(read::ReadTool));
        self.register(Box::new(write::WriteTool));
        self.register(Box::new(edit::EditTool));
        self.register(Box::new(bash::BashTool));
        self.register(Box::new(glob_tool::GlobTool));
        self.register(Box::new(grep_tool::GrepTool));
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
