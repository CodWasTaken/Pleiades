use serde::{Deserialize, Serialize};

/// A step in a workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub name: String,
    pub command: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parallel: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry: Option<u32>,
}

impl WorkflowStep {
    pub fn is_parallel(&self) -> bool {
        self.parallel.unwrap_or(false)
    }
}

/// A complete workflow definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub steps: Vec<WorkflowStep>,
    /// Required variables. Entries may use `name=default` to declare a default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variables: Option<Vec<String>>,
}

impl Workflow {
    /// Validate the workflow definition without executing it.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if self.name.trim().is_empty() {
            errors.push("workflow name cannot be empty".to_string());
        }
        if self.steps.is_empty() {
            errors.push("workflow must contain at least one step".to_string());
        }

        let mut names = std::collections::HashSet::new();
        for (index, step) in self.steps.iter().enumerate() {
            if step.name.trim().is_empty() {
                errors.push(format!("step {} has an empty name", index + 1));
            } else if !names.insert(&step.name) {
                errors.push(format!("duplicate step name '{}'", step.name));
            }
            if step.command.trim().is_empty() {
                errors.push(format!("step '{}' has an empty command", step.name));
            }
            if step.timeout == Some(0) {
                errors.push(format!(
                    "step '{}' timeout must be greater than zero",
                    step.name
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
