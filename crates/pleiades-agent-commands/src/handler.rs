//! The command handler trait.
//!
//! Handlers are async functions that receive a [`CommandContext`] snapshot
//! and the already-tokenized positional arguments, and return a typed
//! [`CommandResult`].  They never touch the terminal, the runtime, or the
//! filesystem directly — they emit [`crate::AppEffect`]s instead.

use async_trait::async_trait;
use pleiades_agent_core::Error;

use crate::context::CommandContext;
use crate::result::CommandResult;

/// Result returned by [`CommandHandler::handle`].
pub type HandlerResult = Result<CommandResult, Error>;

/// Trait implemented by every command handler (builtin, plugin, MCP, or
/// user-defined).  Implementations must be `Send + Sync + 'static` so they
/// can live behind an `Arc` in the [`crate::CommandRegistry`].
#[async_trait]
pub trait CommandHandler: Send + Sync {
    /// Execute the command.  `args` is the positional arguments after the
    /// command path, already tokenized via [`crate::tokenize`].
    async fn handle(&self, ctx: &CommandContext, args: &[String]) -> HandlerResult;
}
