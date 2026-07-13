//! Reducer-owned application state for the live terminal workspace.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use pleiades_agent_core::conversation::{Message, MessageRole};
use pleiades_agent_core::provider::{AgentActivityStatus, Usage};
use pleiades_agent_engine::{
    Activity, AgentCommand, AgentEvent, AgentMode, PermissionDecision, PermissionRequest,
};
use ratatui::style::Style;
use tui_textarea::{Input, TextArea};

use crate::theme::Theme;

const MAX_MESSAGES: usize = 2_000;
const MAX_ACTIVITIES: usize = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Conversation,
    Activity,
    Composer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Overlay {
    Help,
    CommandPalette { selected: usize },
    Permission(PermissionRequest),
    Diff,
    ToolOutput { activity_id: String },
    Diagnostics,
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
        let content = message.text_content();
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
            active_assistant: None,
        };
        state.reset_composer();
        state
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
                self.overlay = Some(Overlay::CommandPalette { selected: 0 });
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
                self.overlay = Some(Overlay::Help);
                Vec::new()
            }
            (KeyCode::Tab, _) => {
                self.focus = match self.focus {
                    Focus::Composer => Focus::Conversation,
                    Focus::Conversation => Focus::Activity,
                    Focus::Activity => Focus::Composer,
                };
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
            Overlay::CommandPalette { mut selected } => {
                let count = palette_commands().len();
                match key.code {
                    KeyCode::Up => selected = selected.saturating_sub(1),
                    KeyCode::Down => selected = (selected + 1).min(count.saturating_sub(1)),
                    KeyCode::Enter => {
                        self.overlay = None;
                        return self.run_palette(selected);
                    }
                    _ => {}
                }
                self.overlay = Some(Overlay::CommandPalette { selected });
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
        let effects = self.execute_input(input);
        self.reset_composer();
        effects
    }

    fn execute_input(&mut self, input: &str) -> Vec<Effect> {
        if !input.starts_with('/') {
            return vec![Effect::Command(AgentCommand::Submit(input.to_string()))];
        }
        let mut parts = input.splitn(2, char::is_whitespace);
        let command = parts.next().unwrap_or_default();
        let value = parts.next().unwrap_or_default().trim();
        match command {
            "/help" => self.overlay = Some(Overlay::Help),
            "/diff" => self.overlay = Some(Overlay::Diff),
            "/output" => {
                if let Some(activity) = self.activities.get(self.selected_activity) {
                    self.overlay = Some(Overlay::ToolOutput {
                        activity_id: activity.id.clone(),
                    });
                }
            }
            "/doctor" => self.overlay = Some(Overlay::Diagnostics),
            "/clear" => return vec![Effect::Command(AgentCommand::ClearConversation)],
            "/save" => return vec![Effect::Command(AgentCommand::SaveSession)],
            "/mode" if !value.is_empty() => {
                return vec![Effect::Command(AgentCommand::SetMode(AgentMode::parse(
                    value,
                )))];
            }
            "/provider" if !value.is_empty() => {
                return vec![Effect::Command(AgentCommand::SetProvider(
                    value.to_string(),
                ))];
            }
            "/model" if !value.is_empty() => {
                return vec![Effect::Command(AgentCommand::SetModel(value.to_string()))];
            }
            "/exit" | "/quit" => {
                return vec![Effect::Command(AgentCommand::Shutdown), Effect::Quit];
            }
            _ => self.status = format!("Unknown or incomplete command: {input}"),
        }
        Vec::new()
    }

    fn run_palette(&mut self, selected: usize) -> Vec<Effect> {
        match selected {
            0 => {
                self.overlay = Some(Overlay::Help);
                Vec::new()
            }
            1 => {
                self.overlay = Some(Overlay::Diff);
                Vec::new()
            }
            2 => {
                self.overlay = Some(Overlay::Diagnostics);
                Vec::new()
            }
            3 => vec![Effect::Command(AgentCommand::SetMode(AgentMode::Plan))],
            4 => vec![Effect::Command(AgentCommand::SetMode(AgentMode::Agent))],
            5 => vec![Effect::Command(AgentCommand::SetMode(
                AgentMode::Unrestricted,
            ))],
            6 => vec![Effect::Command(AgentCommand::SaveSession)],
            _ => vec![Effect::Command(AgentCommand::Shutdown), Effect::Quit],
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
                    content,
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
                        message.streaming = false;
                    }
                } else if !content.is_empty() {
                    self.messages.push(TranscriptMessage {
                        role: MessageRole::Assistant,
                        content,
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
            AgentEvent::Error(message) => self.status = format!("Error: {message}"),
        }
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
                message.content.push_str(delta);
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
    }

    fn reset_composer(&mut self) {
        let mut composer = TextArea::default();
        composer.set_cursor_line_style(Style::default());
        composer.set_cursor_style(
            Style::default()
                .fg(self.theme.background)
                .bg(self.theme.starlight),
        );
        self.composer = composer;
    }
}

pub fn palette_commands() -> &'static [&'static str] {
    &[
        "Searchable help",
        "Review current diff",
        "Diagnostics",
        "Switch to Plan mode",
        "Switch to Agent mode",
        "Switch to Unrestricted mode",
        "Save session",
        "Quit Pleiades",
    ]
}

#[cfg(test)]
mod tests {
    use super::AppState;
    use crate::theme::Theme;
    use pleiades_agent_engine::{AgentEvent, AgentMode};
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
        state.apply_agent(AgentEvent::TextDelta("hel".into()));
        state.apply_agent(AgentEvent::TextDelta("lo".into()));
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
}
