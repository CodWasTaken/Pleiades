//! Chat and agent engine for Pleiades.
//!
//! The engine manages conversations, coordinates with providers,
//! dispatches tools, and handles the core AI interaction loop.

pub mod agent;
pub mod budget;
pub mod chat;
pub mod checkpoint;
pub mod context;
pub mod engine;
pub mod loop_detector;
pub mod memory;
pub mod runtime;
pub mod session;
pub mod verification;

pub use budget::{BudgetLimits, BudgetReport, BudgetService, UsageTotals};
pub use chat::ChatSession;
pub use engine::Engine;
pub use memory::MemoryManager;
pub use runtime::{
    Activity, AgentCommand, AgentEvent, AgentHandle, AgentMode, AgentRuntime, ApprovalPolicy,
    PermissionDecision, PermissionRequest, SandboxPolicy,
};
pub use session::{SessionInfo, SessionStore};

/// Re-exports from the command registry crate.
///
/// The runtime owns a [`pleiades_agent_commands::CommandRegistry`] and routes
/// slash commands through [`AgentCommand::DispatchSlash`].  Frontends consume
/// the typed [`CommandResult`] variants emitted as [`AgentEvent`] payloads via
/// these aliases.
pub use pleiades_agent_commands::{
    CommandContext, CommandRegistry, CommandResult, Notification, NotificationLevel, OverlayKind,
    RenderableDocument,
};
