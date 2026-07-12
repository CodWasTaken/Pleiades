use pleiades_core::error::Error;

use crate::workflow::Workflow;

/// Workflow execution engine.
pub struct WorkflowExecutor;

impl WorkflowExecutor {
    pub fn new() -> Self {
        Self
    }

    /// Execute a workflow.
    pub async fn execute(&self, _workflow: &Workflow) -> Result<Vec<String>, Error> {
        // Workflow execution will be implemented in Milestone 13
        Err(Error::NotImplemented("Workflow execution not yet implemented".to_string()))
    }
}

impl Default for WorkflowExecutor {
    fn default() -> Self {
        Self::new()
    }
}
