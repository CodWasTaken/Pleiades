//! Invocation context handed to every [`crate::CommandHandler`].
//!
//! A [`CommandContext`] is an immutable snapshot of the workspace state at
//! the moment the command was invoked.  Handlers must never reach across
//! this snapshot — they return [`crate::CommandResult`]s that the frontend
//! or runtime applies.  This keeps the engine and the TUI cleanly
//! separated (rules 2/3/4).

use std::sync::Arc;

/// Immutable snapshot of workspace state at command invocation time.
#[derive(Debug, Clone)]
pub struct CommandContext {
    inner: Arc<CommandContextInner>,
}

#[derive(Debug)]
struct CommandContextInner {
    provider: String,
    model: String,
    mode: String,
    interactive: bool,
}

impl CommandContext {
    /// Compose a context from the most-recently-known workspace state.
    pub fn new(
        provider: impl Into<String>,
        model: impl Into<String>,
        mode: impl Into<String>,
    ) -> Self {
        Self {
            inner: Arc::new(CommandContextInner {
                provider: provider.into(),
                model: model.into(),
                mode: mode.into(),
                interactive: true,
            }),
        }
    }

    pub fn provider(&self) -> &str {
        &self.inner.provider
    }

    pub fn model(&self) -> &str {
        &self.inner.model
    }

    pub fn mode(&self) -> &str {
        &self.inner.mode
    }

    pub fn is_interactive(&self) -> bool {
        self.inner.interactive
    }
}

/// Builder for [`CommandContext`].  Used by the frontend when assembling the
/// context from live state; appears in tests for ergonomic construction.
#[derive(Debug, Clone, Default)]
pub struct CommandContextBuilder {
    provider: Option<String>,
    model: Option<String>,
    mode: Option<String>,
    interactive: bool,
}

impl CommandContextBuilder {
    pub fn provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
        self
    }
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }
    pub fn mode(mut self, mode: impl Into<String>) -> Self {
        self.mode = Some(mode.into());
        self
    }
    pub fn interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }
    pub fn build(self) -> CommandContext {
        CommandContext {
            inner: Arc::new(CommandContextInner {
                provider: self.provider.unwrap_or_default(),
                model: self.model.unwrap_or_default(),
                mode: self.mode.unwrap_or_else(|| "agent".to_string()),
                interactive: self.interactive,
            }),
        }
    }
}
