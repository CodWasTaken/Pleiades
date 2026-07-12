use serde::{Deserialize, Serialize};

/// A step in a workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub name: String,
    pub command: String,
    pub args: Option<Vec<String>>,
    pub condition: Option<String>,
    pub parallel: Option<bool>,
    pub timeout: Option<u64>,
    pub retry: Option<u32>,
}

/// A complete workflow definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub name: String,
    pub description: Option<String>,
    pub steps: Vec<WorkflowStep>,
    pub variables: Option<Vec<String>>,
}
