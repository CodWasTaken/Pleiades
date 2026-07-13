//! Plugin SDK for Pleiades plugin authors.
//!
//! This crate provides the types and traits that plugin authors
//! use to build Pleiades plugins. It re-exports key types from
//! `pleiades-core` and adds SDK-specific utilities.

pub use pleiades_core::error::Error;
pub use pleiades_core::event::Event;
/// Re-exported core types for plugin authors.
pub use pleiades_core::tool::{PermissionLevel, Tool, ToolContext, ToolDefinition, ToolResult};
