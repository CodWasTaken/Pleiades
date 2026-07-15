//! Reducer-owned application state for the live terminal workspace.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use pleiades_agent_commands::{CommandRegistry, RenderableDocument};
use pleiades_agent_core::conversation::{Message, MessageRole};
use pleiades_agent_core::provider::{AgentActivityStatus, Usage};
use pleiades_agent_engine::{
    Activity, AgentCommand, AgentEvent, AgentMode, NotificationLevel, OverlayKind,
    PermissionDecision, PermissionRequest,
};
use ratatui::style::Style;
use tui_textarea::{Input, TextArea};

use crate::theme::Theme;

const MAX_MESSAGES: usize = 500;
const MAX_ACTIVITIES: usize = 500;
const MAX_MESSAGE_BYTES: usize = 512 * 1024;
const MAX_TRANSCRIPT_BYTES: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Conversation,
    Activity,
    Composer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Overlay {
    Help {
        query: String,
    },
    CommandPalette {
        selected: usize,
        query: String,
    },
    Permission(PermissionRequest),
    YoloWarning {
        confirmation: String,
    },
    Diff,
    ToolOutput {
        activity_id: String,
    },
    ToolDetails {
        activity_id: String,
    },
    Picker {
        kind: PickerKind,
        selected: usize,
        query: String,
    },
    Configuration,
    Diagnostics,
    McpManager,
    /// Structured document rendered by a slash/palette command (e.g. `/status`,
    /// `/info`).  We keep the title visible for this slice; richer overlay
    /// rendering lands with item 3 / the unified inspector panel (3.0).
    Document(RenderableDocument),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickerKind {
    Provider,
    Model,
    File,
    Session,
}

#[derive(Debug, Clone)]
pub struct TranscriptMessage {
    pub role: MessageRole,
    pub content: String,
    pub reasoning: Option<String>,
    pub streaming: bool,
}

impl From<Message> for TranscriptMessage {
    fn from(message: Message) -> Self {
        let content = truncate_display(message.text_content(), MAX_MESSAGE_BYTES);
        Self {
            role: message.role,
            content,
            reasoning: message.reasoning,
            streaming: false,
        }
    }
}

#[derive(Debug)]
pub enum Effect {
    Command(AgentCommand),
    Quit,
}

pub struct AppState {
    pub theme: Theme,
    pub workspace: PathBuf,
    pub workspace_name: String,
    pub session_id: String,
    pub provider: String,
    pub model: String,
    pub mode: AgentMode,
    pub messages: Vec<TranscriptMessage>,
    pub activities: Vec<Activity>,
    pub outputs: HashMap<String, String>,
    pub diff: String,
    pub branch: Option<String>,
    pub dirty: bool,
    pub usage: Option<Usage>,
    pub running: bool,
    pub task_started: Option<Instant>,
    pub task_elapsed: Duration,
    pub queued: usize,
    pub status: String,
    pub overlay: Option<Overlay>,
    pub focus: Focus,
    pub conversation_scroll: u16,
    pub activity_scroll: u16,
    pub follow_tail: bool,
    pub selected_activity: usize,
    pub composer: TextArea<'static>,
    pub provider_options: Vec<String>,
    pub model_options: Vec<String>,
    pub file_options: Vec<String>,
    pub session_options: Vec<String>,
    pub commands: CommandRegistry,
    /// Set when the runtime requested shutdown (e.g. user typed `/quit`).  The
    /// live workspace checks this flag after each event and stops the loop.
    pub quit_requested: bool,
    input_history: Vec<String>,
    history_cursor: Option<usize>,
    active_assistant: Option<usize>,
}

impl AppState {
    pub fn new(
        theme: Theme,
        workspace: PathBuf,
        provider: String,
        model: String,
        mode: AgentMode,
    ) -> Self {
        let workspace_name = workspace
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("/")
            .to_string();
        let mut state = Self {
            theme,
            workspace,
            workspace_name,
            session_id: String::new(),
            provider,
            model,
            mode,
            messages: Vec::new(),
            activities: Vec::new(),
            outputs: HashMap::new(),
            diff: String::new(),
            branch: None,
            dirty: false,
            usage: None,
            running: false,
            task_started: None,
            task_elapsed: Duration::ZERO,
            queued: 0,
            status: "Ready".to_string(),
            overlay: None,
            focus: Focus::Composer,
            conversation_scroll: 0,
            activity_scroll: 0,
            follow_tail: true,
            selected_activity: 0,
            composer: TextArea::default(),
            provider_options: Vec::new(),
            model_options: Vec::new(),
            file_options: Vec::new(),
            session_options: Vec::new(),
            commands: pleiades_agent_commands::defaults::default_registry(),
            quit_requested: false,
            input_history: Vec::new(),
            history_cursor: None,
            active_assistant: None,
        };
        state.reset_composer();
        state
    }

    /// Look up slash-command completion candidates from the registry.
    /// Returns the labels of every matching command, subcommand, or alias —
    /// nested subcommands surfaced for `/provider `, `/mcp `, etc.  Used by
    /// the composer Tab-completion path (item 2.1.3).
    pub fn complete_slash_candidates(&self, partial: &str) -> Vec<String> {
        self.commands
            .suggest(partial, true)
            .into_iter()
            .map(|suggestion| suggestion.label)
            .collect()
    }

    /// Build the palette listing for the current query, returning tuples of
    /// `(label, description)`.  Driven by [`CommandRegistry::filter`] so the
    /// palette cannot drift from the registry.
    pub fn palette_listing(&self, query: &str) -> Vec<(String, String)> {
        self.commands
            .filter(query, true)
            .into_iter()
            .map(|spec| {
                let label = spec.slash();
                let mut description = spec.description.to_string();
                if spec.shortcut != pleiades_agent_commands::Shortcut::None {
                    description.push_str(&format!("  [{}]", spec.shortcut.label()));
                }
                (label, description)
            })
            .collect()
    }

    pub fn set_picker_options(
        &mut self,
        providers: Vec<String>,
        models: Vec<String>,
        files: Vec<String>,
        sessions: Vec<String>,
    ) {
        self.provider_options = providers;
        self.model_options = models;
        self.file_options = files;
        self.session_options = sessions;
    }

    pub fn elapsed(&self) -> Duration {
        self.task_started
            .map_or(self.task_elapsed, |started| started.elapsed())
    }

    pub fn active_activity(&self) -> Option<&Activity> {
        self.activities.iter().rev().find(|item| {
            matches!(
                item.status,
                AgentActivityStatus::Running | AgentActivityStatus::WaitingForApproval
            )
        })
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Vec<Effect> {
        if key.code == KeyCode::Char('q') && key.modifiers.contains(KeyModifiers::CONTROL) {
            return vec![Effect::Command(AgentCommand::Shutdown), Effect::Quit];
        }

        if let Some(overlay) = self.overlay.clone() {
            return self.handle_overlay_key(overlay, key);
        }

        match (key.code, key.modifiers) {
            (KeyCode::Char('c'), modifiers) if modifiers.contains(KeyModifiers::CONTROL) => {
                if self.running {
                    self.status = "Cancelling current task…".to_string();
                    vec![Effect::Command(AgentCommand::Cancel)]
                } else {
                    vec![Effect::Quit]
                }
            }
            (KeyCode::Char('p'), modifiers) if modifiers.contains(KeyModifiers::CONTROL) => {
                self.overlay = Some(Overlay::CommandPalette {
                    selected: 0,
                    query: String::new(),
                });
                Vec::new()
            }
            (KeyCode::Char('r'), modifiers) if modifiers.contains(KeyModifiers::CONTROL) => {
                self.open_picker(PickerKind::Provider);
                Vec::new()
            }
            (KeyCode::Char('m'), modifiers) if modifiers.contains(KeyModifiers::CONTROL) => {
                self.open_picker(PickerKind::Model);
                Vec::new()
            }
            (KeyCode::Char('f'), modifiers) if modifiers.contains(KeyModifiers::CONTROL) => {
                self.open_picker(PickerKind::File);
                Vec::new()
            }
            (KeyCode::Char('l'), modifiers) if modifiers.contains(KeyModifiers::CONTROL) => {
                self.open_picker(PickerKind::Session);
                Vec::new()
            }
            (KeyCode::Char(','), modifiers) if modifiers.contains(KeyModifiers::CONTROL) => {
                self.overlay = Some(Overlay::Configuration);
                Vec::new()
            }
            (KeyCode::Char('t'), modifiers) if modifiers.contains(KeyModifiers::CONTROL) => {
                if let Some(activity) = self.activities.get(self.selected_activity) {
                    self.overlay = Some(Overlay::ToolDetails {
                        activity_id: activity.id.clone(),
                    });
                }
                Vec::new()
            }
            (KeyCode::Char('d'), modifiers) if modifiers.contains(KeyModifiers::CONTROL) => {
                self.overlay = Some(Overlay::Diff);
                Vec::new()
            }
            (KeyCode::Char('o'), modifiers) if modifiers.contains(KeyModifiers::CONTROL) => {
                if let Some(activity) = self.activities.get(self.selected_activity) {
                    self.overlay = Some(Overlay::ToolOutput {
                        activity_id: activity.id.clone(),
                    });
                }
                Vec::new()
            }
            (KeyCode::F(1), _) => {
                self.overlay = Some(Overlay::Help {
                    query: String::new(),
                });
                Vec::new()
            }
            (KeyCode::Tab, _) if self.focus == Focus::Composer => {
                let input = self.composer.lines().join("\n");
                if input.starts_with('/') {
                    let candidates = self.complete_slash_candidates(&input);
                    match candidates.len() {
                        0 => self.status = format!("No matching command for {input}"),
                        1 => {
                            let completion = format!("/{} ", candidates[0]);
                            if completion.trim_end().starts_with(input.as_str()) {
                                self.set_composer_text(completion);
                            } else {
                                self.status = format!("Match: /{}", candidates[0]);
                            }
                        }
                        _ => {
                            self.status = candidates
                                .iter()
                                .map(|label| format!("/{label}"))
                                .collect::<Vec<_>>()
                                .join("  ");
                        }
                    }
                } else {
                    self.focus = Focus::Conversation;
                }
                Vec::new()
            }
            (KeyCode::Tab, _) => {
                self.focus = match self.focus {
                    Focus::Conversation => Focus::Activity,
                    Focus::Activity | Focus::Composer => Focus::Composer,
                };
                Vec::new()
            }
            (KeyCode::Up, modifiers)
                if modifiers.contains(KeyModifiers::CONTROL) && !self.input_history.is_empty() =>
            {
                let next = self
                    .history_cursor
                    .map_or(self.input_history.len() - 1, |cursor| {
                        cursor.saturating_sub(1)
                    });
                self.history_cursor = Some(next);
                self.set_composer_text(self.input_history[next].clone());
                Vec::new()
            }
            (KeyCode::Down, modifiers)
                if modifiers.contains(KeyModifiers::CONTROL) && self.history_cursor.is_some() =>
            {
                let cursor = self.history_cursor.unwrap_or_default();
                if cursor + 1 < self.input_history.len() {
                    self.history_cursor = Some(cursor + 1);
                    self.set_composer_text(self.input_history[cursor + 1].clone());
                } else {
                    self.history_cursor = None;
                    self.reset_composer();
                }
                Vec::new()
            }
            (KeyCode::PageUp, _) => {
                self.scroll_conversation(5, false);
                Vec::new()
            }
            (KeyCode::PageDown, _) => {
                self.scroll_conversation(5, true);
                Vec::new()
            }
            (KeyCode::Up, _) if self.focus == Focus::Activity => {
                self.selected_activity = self.selected_activity.saturating_sub(1);
                Vec::new()
            }
            (KeyCode::Down, _) if self.focus == Focus::Activity => {
                self.selected_activity =
                    (self.selected_activity + 1).min(self.activities.len().saturating_sub(1));
                Vec::new()
            }
            (KeyCode::Enter, modifiers)
                if !modifiers.intersects(KeyModifiers::SHIFT | KeyModifiers::ALT) =>
            {
                self.submit_composer()
            }
            (KeyCode::Enter, _) | (KeyCode::Char('j'), KeyModifiers::CONTROL) => {
                self.composer.input(Input {
                    key: tui_textarea::Key::Enter,
                    ctrl: false,
                    alt: false,
                    shift: false,
                });
                Vec::new()
            }
            _ => {
                self.focus = Focus::Composer;
                self.composer.input(key);
                Vec::new()
            }
        }
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::ScrollUp => self.scroll_conversation(3, false),
            MouseEventKind::ScrollDown => self.scroll_conversation(3, true),
            _ => {}
        }
    }

    fn handle_overlay_key(&mut self, overlay: Overlay, key: KeyEvent) -> Vec<Effect> {
        if key.code == KeyCode::Esc {
            if !matches!(overlay, Overlay::Permission(_)) {
                self.overlay = None;
            }
            return Vec::new();
        }

        match overlay {
            Overlay::Permission(request) => {
                let decision = match key.code {
                    KeyCode::Char('a') => Some(PermissionDecision::AllowOnce),
                    KeyCode::Char('s') => Some(PermissionDecision::AllowSession),
                    KeyCode::Char('d') => Some(PermissionDecision::DenyOnce),
                    KeyCode::Char('x') => Some(PermissionDecision::DenySession),
                    _ => None,
                };
                if let Some(decision) = decision {
                    self.overlay = None;
                    return vec![Effect::Command(AgentCommand::Permission {
                        request_id: request.id,
                        decision,
                    })];
                }
            }
            Overlay::YoloWarning { mut confirmation } => {
                match key.code {
                    KeyCode::Enter if confirmation == "YOLO" => {
                        self.overlay = None;
                        return vec![Effect::Command(AgentCommand::SetMode(AgentMode::Yolo))];
                    }
                    KeyCode::Backspace => {
                        confirmation.pop();
                    }
                    KeyCode::Char(character) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                        confirmation.push(character);
                    }
                    _ => {}
                }
                self.overlay = Some(Overlay::YoloWarning { confirmation });
            }
            Overlay::Help { mut query } => {
                update_query(&mut query, key);
                self.overlay = Some(Overlay::Help { query });
            }
            Overlay::CommandPalette {
                mut selected,
                mut query,
            } => {
                let listing = self.palette_listing(&query);
                let count = listing.len();
                match key.code {
                    KeyCode::Up => selected = selected.saturating_sub(1),
                    KeyCode::Down => selected = (selected + 1).min(count.saturating_sub(1)),
                    KeyCode::Enter => {
                        self.overlay = None;
                        if let Some((command, _)) = listing.get(selected) {
                            // Dispatch the canonical slash form through the runtime
                            // so palette and slash commands share a single path.
                            return vec![Effect::Command(AgentCommand::DispatchSlash(
                                command.clone(),
                            ))];
                        }
                    }
                    _ => {
                        update_query(&mut query, key);
                        selected = 0;
                    }
                }
                self.overlay = Some(Overlay::CommandPalette { selected, query });
            }
            Overlay::Picker {
                kind,
                mut selected,
                mut query,
            } => {
                let options = self.filtered_picker_options(kind, &query);
                match key.code {
                    KeyCode::Up => selected = selected.saturating_sub(1),
                    KeyCode::Down => {
                        selected = (selected + 1).min(options.len().saturating_sub(1));
                    }
                    KeyCode::Enter => {
                        self.overlay = None;
                        if let Some(value) = options.get(selected) {
                            return self.choose_picker_value(kind, value);
                        }
                    }
                    _ => {
                        update_query(&mut query, key);
                        selected = 0;
                    }
                }
                self.overlay = Some(Overlay::Picker {
                    kind,
                    selected,
                    query,
                });
            }
            _ => {}
        }
        Vec::new()
    }

    fn submit_composer(&mut self) -> Vec<Effect> {
        let input = self.composer.lines().join("\n");
        let input = input.trim();
        if input.is_empty() {
            return Vec::new();
        }
        if self
            .input_history
            .last()
            .is_none_or(|previous| previous != input)
        {
            self.input_history.push(input.to_string());
            if self.input_history.len() > 500 {
                self.input_history.remove(0);
            }
        }
        self.history_cursor = None;
        let effects = self.execute_input(input);
        self.reset_composer();
        effects
    }

    fn execute_input(&mut self, input: &str) -> Vec<Effect> {
        if !input.starts_with('/') {
            return vec![Effect::Command(AgentCommand::Submit(input.to_string()))];
        }
        // Route every slash command through the registry (issue 2.1 items
        // 2-3).  The runtime resolves the spec, invokes its handler, applies
        // typed AppEffects, and forwards OpenOverlay / Notify / Document /
        // ShuttingDown back as AgentEvent variants consumed by `apply_agent`.
        vec![Effect::Command(AgentCommand::DispatchSlash(
            input.to_string(),
        ))]
    }

    fn open_picker(&mut self, kind: PickerKind) {
        self.overlay = Some(Overlay::Picker {
            kind,
            selected: 0,
            query: String::new(),
        });
    }

    pub fn filtered_picker_options(&self, kind: PickerKind, query: &str) -> Vec<String> {
        let options = match kind {
            PickerKind::Provider => &self.provider_options,
            PickerKind::Model => &self.model_options,
            PickerKind::File => &self.file_options,
            PickerKind::Session => &self.session_options,
        };
        let query = query.to_ascii_lowercase();
        options
            .iter()
            .filter(|value| value.to_ascii_lowercase().contains(&query))
            .cloned()
            .collect()
    }

    fn choose_picker_value(&mut self, kind: PickerKind, value: &str) -> Vec<Effect> {
        match kind {
            PickerKind::Provider => vec![Effect::Command(AgentCommand::SetProvider(
                value.to_string(),
            ))],
            PickerKind::Model => vec![Effect::Command(AgentCommand::SetModel(value.to_string()))],
            PickerKind::File => {
                self.composer.insert_str(format!("@{value} "));
                Vec::new()
            }
            PickerKind::Session => vec![Effect::Command(AgentCommand::LoadSession(
                value.to_string(),
            ))],
        }
    }

    pub fn apply_agent(&mut self, event: AgentEvent) {
        match event {
            AgentEvent::SessionReady { id, history } => {
                self.session_id = id;
                self.messages = history.into_iter().map(Into::into).collect();
                self.trim_messages();
            }
            AgentEvent::UserMessage(content) => {
                self.messages.push(TranscriptMessage {
                    role: MessageRole::User,
                    content: truncate_display(content, MAX_MESSAGE_BYTES),
                    reasoning: None,
                    streaming: false,
                });
                self.active_assistant = None;
                self.follow_tail = true;
                self.trim_messages();
            }
            AgentEvent::TaskStarted { .. } => {
                self.running = true;
                self.task_started = Some(Instant::now());
                self.task_elapsed = Duration::ZERO;
                self.status = "Agent is working".to_string();
            }
            AgentEvent::TextDelta(delta) => self.append_assistant(&delta, false),
            AgentEvent::ReasoningDelta(delta) => self.append_assistant(&delta, true),
            AgentEvent::AssistantMessageCompleted(content) => {
                if let Some(index) = self.active_assistant.take() {
                    if let Some(message) = self.messages.get_mut(index) {
                        message.content = content;
                        message.content = truncate_display(
                            std::mem::take(&mut message.content),
                            MAX_MESSAGE_BYTES,
                        );
                        message.streaming = false;
                    }
                } else if !content.is_empty() {
                    self.messages.push(TranscriptMessage {
                        role: MessageRole::Assistant,
                        content: truncate_display(content, MAX_MESSAGE_BYTES),
                        reasoning: None,
                        streaming: false,
                    });
                }
            }
            AgentEvent::Activity(activity) => self.upsert_activity(activity),
            AgentEvent::PermissionRequested(request) => {
                self.status = "Waiting for permission".to_string();
                self.overlay = Some(Overlay::Permission(request));
            }
            AgentEvent::ToolOutput {
                activity_id,
                mut output,
                truncated,
            } => {
                if truncated {
                    output.push_str("\n… output truncated by runtime");
                }
                self.outputs.insert(activity_id, output);
            }
            AgentEvent::Usage(usage) => self.usage = Some(usage),
            AgentEvent::DiffUpdated(diff) => self.diff = diff,
            AgentEvent::GitState { branch, dirty } => {
                self.branch = branch;
                self.dirty = dirty;
            }
            AgentEvent::QueueChanged(count) => self.queued = count,
            AgentEvent::ModeChanged(mode) => {
                self.mode = mode;
                self.status = format!("Mode: {}", mode.label());
            }
            AgentEvent::ProviderChanged(provider) => {
                self.provider = provider;
                self.status = "Provider changed".into();
            }
            AgentEvent::ModelChanged(model) => {
                self.model = model;
                self.status = "Model changed".into();
            }
            AgentEvent::ConversationCleared => {
                self.messages.clear();
                self.status = "Conversation cleared".into();
            }
            AgentEvent::SessionSaved(id) => self.status = format!("Session {id} saved"),
            AgentEvent::TaskCompleted { elapsed_ms } => {
                self.finish_task(Duration::from_millis(elapsed_ms));
                self.status = "Task completed".to_string();
            }
            AgentEvent::TaskFailed { message } => {
                self.finish_task(self.elapsed());
                self.status = format!("Task failed: {message}");
            }
            AgentEvent::TaskCancelled => {
                self.finish_task(self.elapsed());
                self.status = "Task cancelled".to_string();
            }
            AgentEvent::OpenOverlay(kind) => self.open_overlay_kind(kind),
            AgentEvent::Notify(notification) => {
                let label = match notification.level {
                    NotificationLevel::Info => "Info",
                    NotificationLevel::Success => "Success",
                    NotificationLevel::Warning => "Warning",
                    NotificationLevel::Error => "Error",
                };
                self.status = format!("{label}: {}", notification.message);
            }
            AgentEvent::Document(document) => {
                self.overlay = Some(Overlay::Document(document));
            }
            AgentEvent::ExtensionsReloaded => {
                self.commands = pleiades_agent_commands::defaults::default_registry();
                self.status = "Extensions reloaded".to_string();
            }
            AgentEvent::ShuttingDown => {
                self.quit_requested = true;
                self.status = "Shutting down".to_string();
            }
            AgentEvent::Error(message) => self.status = format!("Error: {message}"),
        }
    }

    fn open_overlay_kind(&mut self, kind: OverlayKind) {
        self.overlay = match kind {
            OverlayKind::Help => Some(Overlay::Help {
                query: String::new(),
            }),
            OverlayKind::CommandPalette => Some(Overlay::CommandPalette {
                selected: 0,
                query: String::new(),
            }),
            OverlayKind::Diff => Some(Overlay::Diff),
            OverlayKind::ToolOutput => {
                self.activities
                    .get(self.selected_activity)
                    .map(|activity| Overlay::ToolOutput {
                        activity_id: activity.id.clone(),
                    })
            }
            OverlayKind::Diagnostics => Some(Overlay::Diagnostics),
            OverlayKind::McpManager => Some(Overlay::McpManager),
            OverlayKind::Configuration => Some(Overlay::Configuration),
            OverlayKind::FilePicker => {
                self.open_picker(PickerKind::File);
                return;
            }
            OverlayKind::SessionPicker => {
                self.open_picker(PickerKind::Session);
                return;
            }
            OverlayKind::ProviderPicker | OverlayKind::ProviderWizard => {
                self.open_picker(PickerKind::Provider);
                return;
            }
            OverlayKind::ModelPicker => {
                self.open_picker(PickerKind::Model);
                return;
            }
            OverlayKind::YoloWarning => Some(Overlay::YoloWarning {
                confirmation: String::new(),
            }),
            unsupported => {
                self.status = format!("Overlay {unsupported:?} is not available in this release");
                None
            }
        };
    }

    fn append_assistant(&mut self, delta: &str, reasoning: bool) {
        let index = if let Some(index) = self.active_assistant {
            index
        } else {
            self.messages.push(TranscriptMessage {
                role: MessageRole::Assistant,
                content: String::new(),
                reasoning: None,
                streaming: true,
            });
            let index = self.messages.len() - 1;
            self.active_assistant = Some(index);
            index
        };
        if let Some(message) = self.messages.get_mut(index) {
            if reasoning {
                message
                    .reasoning
                    .get_or_insert_with(String::new)
                    .push_str(delta);
            } else {
                if message.content.len() < MAX_MESSAGE_BYTES {
                    let remaining = MAX_MESSAGE_BYTES - message.content.len();
                    message
                        .content
                        .push_str(&truncate_display(delta.to_string(), remaining));
                }
            }
        }
        self.follow_tail = true;
    }

    fn upsert_activity(&mut self, activity: Activity) {
        if let Some(existing) = self
            .activities
            .iter_mut()
            .find(|item| item.id == activity.id)
        {
            *existing = activity;
        } else {
            self.activities.push(activity);
            if self.activities.len() > MAX_ACTIVITIES {
                self.activities.remove(0);
            }
        }
        self.selected_activity = self.activities.len().saturating_sub(1);
    }

    fn finish_task(&mut self, elapsed: Duration) {
        self.running = false;
        self.task_elapsed = elapsed;
        self.task_started = None;
        self.active_assistant = None;
    }

    fn scroll_conversation(&mut self, amount: u16, down: bool) {
        if down {
            self.conversation_scroll = self.conversation_scroll.saturating_sub(amount);
            self.follow_tail = self.conversation_scroll == 0;
        } else {
            self.conversation_scroll = self.conversation_scroll.saturating_add(amount);
            self.follow_tail = false;
        }
    }

    fn trim_messages(&mut self) {
        if self.messages.len() > MAX_MESSAGES {
            let count = self.messages.len() - MAX_MESSAGES;
            self.messages.drain(0..count);
        }
        let mut total = self
            .messages
            .iter()
            .map(|message| message.content.len())
            .sum::<usize>();
        while total > MAX_TRANSCRIPT_BYTES && self.messages.len() > 1 {
            total = total.saturating_sub(self.messages.remove(0).content.len());
        }
    }

    fn reset_composer(&mut self) {
        self.set_composer_text(String::new());
    }

    fn set_composer_text(&mut self, value: String) {
        let mut composer = TextArea::new(value.lines().map(str::to_string).collect());
        composer.set_cursor_line_style(Style::default());
        composer.set_cursor_style(
            Style::default()
                .fg(self.theme.background)
                .bg(self.theme.starlight),
        );
        self.composer = composer;
    }
}

fn truncate_display(value: String, limit: usize) -> String {
    if value.len() <= limit {
        return value;
    }
    let boundary = value
        .char_indices()
        .map(|(index, _)| index)
        .take_while(|index| *index <= limit.saturating_sub(32))
        .last()
        .unwrap_or(0);
    format!("{}\n… content truncated …", &value[..boundary])
}

fn update_query(query: &mut String, key: KeyEvent) {
    match key.code {
        KeyCode::Char(character) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            query.push(character);
        }
        KeyCode::Backspace => {
            query.pop();
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::{AppState, Effect, Overlay};
    use crate::theme::Theme;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use pleiades_agent_commands::{Notification, NotificationLevel, RenderableDocument};
    use pleiades_agent_engine::{AgentCommand, AgentEvent, AgentMode, OverlayKind};
    use std::path::PathBuf;

    fn state() -> AppState {
        AppState::new(
            Theme::default(),
            PathBuf::from("/tmp/project"),
            "mock".into(),
            "mock-1".into(),
            AgentMode::Agent,
        )
    }

    #[test]
    fn streamed_message_is_reconciled_with_completed_message() {
        let mut state = state();
        state.apply_agent(AgentEvent::TextDelta("he".into()));
        state.apply_agent(AgentEvent::TextDelta("llo".into()));
        state.apply_agent(AgentEvent::AssistantMessageCompleted("hello".into()));
        assert_eq!(state.messages.len(), 1);
        assert_eq!(state.messages[0].content, "hello");
        assert!(!state.messages[0].streaming);
    }

    #[test]
    fn cancellation_restores_idle_state() {
        let mut state = state();
        state.apply_agent(AgentEvent::TaskStarted {
            task: "test".into(),
            started_at_ms: 0,
        });
        state.apply_agent(AgentEvent::TaskCancelled);
        assert!(!state.running);
        assert_eq!(state.status, "Task cancelled");
    }

    #[test]
    fn bounds_huge_streamed_messages_without_splitting_utf8() {
        let mut state = state();
        state.apply_agent(AgentEvent::TextDelta("✦".repeat(300_000)));
        assert!(state.messages[0].content.len() <= super::MAX_MESSAGE_BYTES);
        assert!(
            state.messages[0]
                .content
                .is_char_boundary(state.messages[0].content.len())
        );
        assert!(state.messages[0].content.ends_with("… content truncated …"));
    }

    #[test]
    fn slash_commands_dispatch_through_the_runtime() {
        let mut state = state();
        let effects = state.execute_input("/provider use openai");
        assert!(matches!(
            effects.as_slice(),
            [Effect::Command(AgentCommand::DispatchSlash(command))]
                if command == "/provider use openai"
        ));
    }

    #[test]
    fn palette_and_nested_completion_come_from_the_registry() {
        let state = state();
        assert!(
            state
                .palette_listing("doctor")
                .iter()
                .any(|(command, _)| command == "/doctor")
        );
        let completions = state.complete_slash_candidates("/mode ");
        assert!(completions.iter().any(|item| item == "mode plan"));
        assert!(completions.iter().any(|item| item == "mode agent"));
    }

    #[test]
    fn typed_command_events_update_frontend_state() {
        let mut state = state();
        state.apply_agent(AgentEvent::OpenOverlay(OverlayKind::Diagnostics));
        assert!(matches!(state.overlay, Some(Overlay::Diagnostics)));

        state.apply_agent(AgentEvent::Notify(Notification {
            level: NotificationLevel::Success,
            message: "configuration saved".into(),
        }));
        assert_eq!(state.status, "Success: configuration saved");

        let document = RenderableDocument::new("Status").section("Mode", "agent");
        state.apply_agent(AgentEvent::Document(document.clone()));
        assert_eq!(state.overlay, Some(Overlay::Document(document)));

        state.apply_agent(AgentEvent::ShuttingDown);
        assert!(state.quit_requested);
    }

    #[test]
    fn extensions_reloaded_refreshes_custom_command_completion() {
        let original = std::env::current_dir().unwrap();
        let temp = tempfile::tempdir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();
        let mut state = state();
        assert!(
            !state
                .complete_slash_candidates("/release")
                .iter()
                .any(|item| item == "release")
        );

        std::fs::create_dir_all(".pleiades/commands").unwrap();
        std::fs::write(
            ".pleiades/commands/release.toml",
            r#"
description = "Prepare a release"
prompt = "Prepare release {{args}}"
"#,
        )
        .unwrap();
        state.apply_agent(AgentEvent::ExtensionsReloaded);
        std::env::set_current_dir(original).unwrap();

        assert!(
            state
                .complete_slash_candidates("/release")
                .iter()
                .any(|item| item == "release")
        );
        assert_eq!(state.status, "Extensions reloaded");
    }

    #[test]
    fn yolo_requires_exact_typed_confirmation() {
        let mut state = state();
        state.apply_agent(AgentEvent::OpenOverlay(OverlayKind::YoloWarning));
        for character in "yolo".chars() {
            let overlay = state.overlay.take().unwrap();
            let effects = state.handle_overlay_key(
                overlay,
                KeyEvent::new(KeyCode::Char(character), KeyModifiers::NONE),
            );
            assert!(effects.is_empty());
        }
        let overlay = state.overlay.take().unwrap();
        assert!(
            state
                .handle_overlay_key(overlay, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),)
                .is_empty()
        );

        state.apply_agent(AgentEvent::OpenOverlay(OverlayKind::YoloWarning));
        for character in "YOLO".chars() {
            let overlay = state.overlay.take().unwrap();
            state.handle_overlay_key(
                overlay,
                KeyEvent::new(KeyCode::Char(character), KeyModifiers::NONE),
            );
        }
        let overlay = state.overlay.take().unwrap();
        let effects =
            state.handle_overlay_key(overlay, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(matches!(
            effects.as_slice(),
            [Effect::Command(AgentCommand::SetMode(AgentMode::Yolo))]
        ));
    }
}
