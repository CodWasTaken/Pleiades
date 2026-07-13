//! Chat and agent engine for Pleiades.
//!
//! The engine manages conversations, coordinates with providers,
//! dispatches tools, and handles the core AI interaction loop.

pub mod agent;
pub mod chat;
pub mod engine;
pub mod memory;
pub mod session;

pub use chat::ChatSession;
pub use engine::Engine;
pub use memory::MemoryManager;
pub use session::{SessionInfo, SessionStore};
