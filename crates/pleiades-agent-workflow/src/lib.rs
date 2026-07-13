//! Workflow engine for Pleiades.
//!
//! Define, share, and run multi-step workflows with
//! sequencing, parallelism, and conditional branching.

pub mod execute;
pub mod workflow;

pub use execute::{StepResult, StepStatus, WorkflowExecutor};
pub use workflow::{Workflow, WorkflowStep};
