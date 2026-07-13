//! Core domain types and traits for Pleiades.
//!
//! This crate contains all domain-level types, traits, and abstractions
//! that form the foundation of the Pleiades system. It has zero internal
//! dependencies and is the only crate that other crates must depend on.

pub mod conversation;
pub mod error;
pub mod event;
pub mod model;
pub mod provider;
pub mod tool;

pub use conversation::Conversation;
pub use error::Error;
pub use event::Event;
pub use model::{ModelAlias, ModelCapabilities, ModelInfo, ModelRegistry, Pricing};
pub use provider::Provider;
pub use tool::Tool;
