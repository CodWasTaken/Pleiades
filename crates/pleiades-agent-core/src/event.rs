use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Events emitted by the Pleiades system.
///
/// The event system enables loose coupling between components.
/// Plugins can subscribe to events, the UI renders events,
/// and the engine emits events during processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    /// A message was added to the conversation.
    MessageAdded { conversation_id: String },

    /// A tool was called.
    ToolCalled {
        tool: String,
        input: Value,
        timestamp: u64,
    },

    /// A tool completed execution.
    ToolCompleted {
        tool: String,
        success: bool,
        duration_ms: u64,
    },

    /// A token was streamed from the provider.
    TokenStreamed {
        token: String,
        conversation_id: String,
    },

    /// A reasoning token was streamed.
    ReasoningToken { token: String },

    /// An error occurred.
    Error { error: String, source: String },

    /// Configuration was changed.
    ConfigChanged { key: String },

    /// A plugin was loaded.
    PluginLoaded { name: String, version: String },

    /// A plugin was unloaded.
    PluginUnloaded { name: String },

    /// Session started.
    SessionStarted { id: String },

    /// Session ended.
    SessionEnded { id: String },

    /// Provider rate limited.
    RateLimited {
        provider: String,
        retry_after: Option<u64>,
    },

    /// Generic event for extensions.
    Custom { name: String, data: Value },
}
