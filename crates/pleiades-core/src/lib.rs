//! Core domain types and traits for Pleiades.
//!
//! This crate contains all domain-level types, traits, and abstractions
//! that form the foundation of the Pleiades system. It has zero internal
//! dependencies and is the only crate that other crates must depend on.

pub mod provider;
pub mod model;
pub mod conversation;
pub mod tool;
pub mod error;
pub mod event;

pub use provider::Provider;
pub use model::{ModelRegistry, ModelInfo, ModelCapabilities, Pricing, ModelAlias};
pub use conversation::Conversation;
pub use tool::Tool;
pub use error::Error;
pub use event::Event;
