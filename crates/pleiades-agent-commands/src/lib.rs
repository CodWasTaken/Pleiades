//! Pleiades command registry and application command surface.
//!
//! This crate is the single source of truth for every user-invokable command
//! in Pleiades: slash commands in the live workspace, clap subcommands in the
//! CLI, entries in the command palette, help overlays, and autocomplete
//! suggestions.  Plugins, MCP servers, and custom user command files all
//! extend the same registry rather than bypassing it.
//!
//! Architectural rules enforced by this crate:
//!
//! * The registry contains pure command descriptors and handlers.  Handlers
//!   receive a [`CommandContext`] snapshot and return a typed
//!   [`CommandResult`] — they never touch the terminal or the runtime
//!   directly, preserving the event-driven separation between frontend and
//!   runtime.
//! * Search, help, and palette text are generated from the registry rather
//!   than maintained in parallel lists, eliminating index drift.
//! * Nested subcommands, aliases, usage validation, autocomplete
//!   suggestions, permission metadata, and headless-compatibility are
//!   first-class concerns.

pub mod context;
pub mod defaults;
pub mod handler;
pub mod parser;
pub mod registry;
pub mod result;
pub mod spec;

pub use context::{CommandContext, CommandContextBuilder};
pub use handler::{CommandHandler, HandlerResult};
pub use parser::{ParseError, tokenize};
pub use registry::{CommandRegistry, RegistrationError, Suggestion, SuggestionKind};
pub use result::{
    AppEffect, BackgroundTaskHandle, CommandResult, Notification, NotificationLevel, OverlayKind,
    RenderableDocument, RenderableSection, RuntimeRestartRequest,
};
pub use spec::{
    ArgumentSpec, CommandAvailability, CommandCategory, CommandSpec, CompletionSource,
    PermissionRequirement, Shortcut,
};
