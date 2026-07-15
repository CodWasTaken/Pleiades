//! Typed command results.
//!
//! [`CommandResult`] is the single type produced by every command handler.
//! The frontend / runtime converts these into `AgentCommand`s, Ratatui
//! overlays, status notifications, or document renderings.  Handlers never
//! produce raw strings, ensuring structured success and error reporting.

/// The result of any command invocation.
#[derive(Debug, Clone)]
pub enum CommandResult {
    /// Side effects to be applied by the runtime (`SetMode`, `SetProvider`,
    /// file writes, etc.).  Effects are applied in order.
    Effects(Vec<AppEffect>),
    /// Request that the frontend open a native overlay.  The variant names
    /// the overlay; the frontend owns the actual rendering state.
    OpenOverlay(OverlayKind),
    /// Surf a transient status or toast notification.
    Notification(Notification),
    /// Render a structured document (help, listing, etc.) in the active
    /// panel.
    RenderDocument(RenderableDocument),
    /// Request a runtime restart (e.g. after a critical config change).
    RuntimeRestart(RuntimeRestartRequest),
    /// Spawn a tracked background task.  The returned handle lets the
    /// caller poll status via the runtime; the work itself happens
    /// out-of-band.
    BackgroundTask(BackgroundTaskHandle),
    /// Nothing to apply.  The handler already produced its side effect via
    /// other channels or chose to do nothing.
    Noop,
}

impl CommandResult {
    pub fn effects<I: IntoIterator<Item = AppEffect>>(effects: I) -> Self {
        Self::Effects(effects.into_iter().collect())
    }
    pub fn noop() -> Self {
        Self::Noop
    }
    pub fn notification(level: NotificationLevel, message: impl Into<String>) -> Self {
        Self::Notification(Notification {
            level,
            message: message.into(),
        })
    }
    pub fn overlay(kind: OverlayKind) -> Self {
        Self::OpenOverlay(kind)
    }
}

/// Effects a command may emit to drive the runtime.
///
/// This enum is intentionally narrower than the runtime's own command set
/// — only the side effects that commands actually produce today appear
/// here.  As new commands land, expand this enum and teach the runtime to
/// convert from it (rule: handlers never touch the runtime directly).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppEffect {
    /// Switch to a named mode (`plan` / `agent` / `auto` / `yolo`).
    SetMode(String),
    /// Switch to a named provider.
    SetProvider(String),
    /// Switch to a named model.
    SetModel(String),
    /// Clear the conversation in the live workspace.
    ClearConversation,
    /// Load a named session.
    LoadSession(String),
    /// Save the current session.
    SaveSession,
    /// Create a checkpoint from the live runtime state.
    CreateCheckpoint(Option<String>),
    /// List saved checkpoints.
    ListCheckpoints,
    /// Show checkpoint details.
    ShowCheckpoint(String),
    /// Restore a checkpoint. `confirm` is false for preview-only calls.
    RestoreCheckpoint { id: String, confirm: bool },
    /// Delete a checkpoint.
    DeleteCheckpoint(String),
    /// Cancel any running task.
    CancelTask,
    /// Quit the live workspace (does not shut down the runtime cleanly;
    /// paired below).
    Quit,
    /// Shut the runtime down cleanly.
    Shutdown,
    /// Reload extension sources (plugins, MCP, skills, custom commands).
    ReloadExtensions,
    /// Set a transient status string shown in the workspace header.
    Status(String),
    /// Used by plugin / future commands to carry a typed escape name.
    /// Rendered in logs as the opaque identifier; runtime will ignore
    /// unknown names gracefully.
    Custom(String),
}

/// Overlay kinds the frontend knows how to render.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OverlayKind {
    Help,
    CommandPalette,
    Diff,
    ToolOutput,
    Diagnostics,
    Configuration,
    FilePicker,
    SessionPicker,
    ProviderPicker,
    ModelPicker,
    ModePicker,
    YoloWarning,
    ProviderWizard,
    PluginManager,
    McpManager,
    Permissions,
    Checkpoint,
    Context,
    Memory,
    Budget,
    GitReview,
    GitLog,
    LspInspector,
    ProcessManager,
    Browser,
    ProjectRecipes,
    SubagentInspector,
    AgentInspector,
    ThemePicker,
    BrowserConsole,
}

/// Notification level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}

/// A transient notification surfaced to the user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notification {
    pub level: NotificationLevel,
    pub message: String,
}

/// A structured document the frontend renders in the active panel.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RenderableDocument {
    pub title: String,
    pub sections: Vec<RenderableSection>,
}

impl RenderableDocument {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            sections: Vec::new(),
        }
    }
    pub fn section(mut self, heading: impl Into<String>, body: impl Into<String>) -> Self {
        self.sections.push(RenderableSection {
            heading: heading.into(),
            body: body.into(),
        });
        self
    }
    pub fn push_section(&mut self, heading: impl Into<String>, body: impl Into<String>) {
        self.sections.push(RenderableSection {
            heading: heading.into(),
            body: body.into(),
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderableSection {
    pub heading: String,
    pub body: String,
}

/// Request that the runtime restart itself (e.g. after a config schema bump).
#[derive(Debug, Clone)]
pub struct RuntimeRestartRequest {
    pub reason: String,
}

/// Handle to a background task spawned by a command.
#[derive(Debug, Clone)]
pub struct BackgroundTaskHandle {
    pub id: String,
}
