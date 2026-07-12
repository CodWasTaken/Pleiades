//! Workflow engine for Pleiades.
//!
//! Define, share, and run multi-step workflows with
//! sequencing, parallelism, and conditional branching.

pub mod workflow;
pub mod execute;

pub use workflow::Workflow;
pub use execute::WorkflowExecutor;
