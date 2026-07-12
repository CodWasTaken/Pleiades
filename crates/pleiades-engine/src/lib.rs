//! Chat and agent engine for Pleiades.
//!
//! The engine manages conversations, coordinates with providers,
//! dispatches tools, and handles the core AI interaction loop.

pub mod engine;
pub mod chat;
pub mod agent;
pub mod session;

pub use engine::Engine;
pub use chat::ChatSession;
pub use session::{SessionStore, SessionInfo};
