//! Event-driven autonomous agent runtime.
//!
//! The runtime owns providers, tools, conversations, permissions, and task
//! lifecycle. Frontends communicate exclusively through bounded command and
//! event channels and never execute tools or write agent output directly.

use std::collections::{HashSet, VecDeque};
use std::time::{Duration, Instant};

use pleiades_agent_commands::{
    self as commands, AppEffect, CommandResult, Notification, NotificationLevel, OverlayKind,
    RenderableDocument,
};
use pleiades_agent_config::Config;
use pleiades_agent_core::conversation::{ContentBlock, Conversation, Message, MessageRole};
use pleiades_agent_core::provider::{AgentActivityKind, AgentActivityStatus, StreamEvent, Usage};
use pleiades_agent_core::tool::PermissionLevel;
use pleiades_agent_permissions::{DecisionKind, PermissionEngine, ToolInvocation, parse_shell};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::checkpoint::{CheckpointRecord, CheckpointStore};
use crate::context::{CompressionRecord, ContextAccountant, ContextPin, ContextReport, make_pin};
use crate::loop_detector::{DoomLoopDetector, LoopSignal};
use crate::verification::{VerificationReport, VerificationScope, VerificationService};
use crate::{Engine, SessionStore};

const COMMAND_CAPACITY: usize = 64;
const EVENT_CAPACITY: usize = 512;
const MAX_TOOL_OUTPUT: usize = 256 * 1024;
const MAX_DIFF_OUTPUT: usize = 512 * 1024;

/// When autonomous work must pause for user approval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ApprovalPolicy {
    Always,
    OnRisk,
    OnFailure,
    Never,
}

/// Filesystem and process boundary applied to autonomous work.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SandboxPolicy {
    ReadOnly,
    WorkspaceWrite,
    FullAccess,
}

impl SandboxPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReadOnly => "read-only",
            Self::WorkspaceWrite => "workspace-write",
            Self::FullAccess => "danger-full-access",
        }
    }
}

/// User-facing presets combining independent approval and sandbox policies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentMode {
    Plan,
    Agent,
    Auto,
    Yolo,
}

impl AgentMode {
    pub fn parse(value: &str) -> Self {
        match value {
            "plan" | "read-only" | "readonly" => Self::Plan,
            "auto" => Self::Auto,
            "yolo" | "unrestricted" | "danger-full-access" => Self::Yolo,
            _ => Self::Agent,
        }
    }

    pub fn sandbox(self) -> &'static str {
        self.sandbox_policy().as_str()
    }

    pub fn approval_policy(self) -> ApprovalPolicy {
        match self {
            Self::Plan | Self::Auto | Self::Yolo => ApprovalPolicy::Never,
            Self::Agent => ApprovalPolicy::OnRisk,
        }
    }

    pub fn sandbox_policy(self) -> SandboxPolicy {
        match self {
            Self::Plan => SandboxPolicy::ReadOnly,
            Self::Agent | Self::Auto => SandboxPolicy::WorkspaceWrite,
            Self::Yolo => SandboxPolicy::FullAccess,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Plan => "plan",
            Self::Agent => "agent",
            Self::Auto => "auto",
            Self::Yolo => "yolo",
        }
    }
}

/// A normalized activity item rendered by terminal and future graphical UIs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Activity {
    pub id: String,
    pub kind: AgentActivityKind,
    pub title: String,
    pub detail: Option<String>,
    pub status: AgentActivityStatus,
    pub started_at_ms: u128,
    pub duration_ms: Option<u64>,
}

impl Activity {
    fn new(
        id: impl Into<String>,
        kind: AgentActivityKind,
        title: impl Into<String>,
        status: AgentActivityStatus,
    ) -> Self {
        Self {
            id: id.into(),
            kind,
            title: title.into(),
            detail: None,
            status,
            started_at_ms: now_ms(),
            duration_ms: None,
        }
    }
}

/// Decision returned from an in-interface permission prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionDecision {
    AllowOnce,
    AllowSession,
    DenyOnce,
    DenySession,
}

/// Structured permission request presented by the frontend.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionRequest {
    pub id: String,
    pub tool: String,
    pub operation: String,
    pub target: String,
    pub reason: String,
    pub risk: String,
    pub input: serde_json::Value,
}

/// Commands accepted by the autonomous runtime.
#[derive(Debug, Clone)]
pub enum AgentCommand {
    Submit(String),
    Cancel,
    Permission {
        request_id: String,
        decision: PermissionDecision,
    },
    SetMode(AgentMode),
    SetProvider(String),
    SetModel(String),
    ClearConversation,
    LoadSession(String),
    SaveSession,
    /// Dispatch a slash command (with leading `/`) through the command
    /// registry.  The runtime resolves the spec, invokes its handler, applies
    /// any emitted [`AppEffect`]s, and forwards [`OverlayKind`] /
    /// [`Notification`] / [`RenderableDocument`] results to the frontend as
    /// [`AgentEvent`] variants.  Frontends use this for both typed slash
    /// commands and command-palette selections.
    DispatchSlash(String),
    Shutdown,
}

/// Events emitted by the autonomous runtime.
#[derive(Debug, Clone)]
pub enum AgentEvent {
    SessionReady {
        id: String,
        history: Vec<Message>,
    },
    UserMessage(String),
    TaskStarted {
        task: String,
        started_at_ms: u128,
    },
    TextDelta(String),
    ReasoningDelta(String),
    AssistantMessageCompleted(String),
    Activity(Activity),
    PermissionRequested(PermissionRequest),
    ToolOutput {
        activity_id: String,
        output: String,
        truncated: bool,
    },
    Usage(Usage),
    DiffUpdated(String),
    GitState {
        branch: Option<String>,
        dirty: bool,
    },
    QueueChanged(usize),
    ModeChanged(AgentMode),
    ProviderChanged(String),
    ModelChanged(String),
    ConversationCleared,
    SessionSaved(String),
    TaskCompleted {
        elapsed_ms: u64,
    },
    TaskFailed {
        message: String,
    },
    TaskCancelled,
    /// A slash/palette command requested the frontend open a native overlay.
    /// Forwarded verbatim from a [`CommandResult::OpenOverlay`].
    OpenOverlay(OverlayKind),
    /// A slash/palette command surfaced a transient notification.  Forwarded
    /// verbatim from a [`CommandResult::Notification`].
    Notify(Notification),
    /// A slash/palette command produced a structured document to render in
    /// the active panel.  Forwarded verbatim from a
    /// [`CommandResult::RenderDocument`].
    Document(RenderableDocument),
    /// Extension sources were reloaded and frontends should rebuild their
    /// command registry, picker data, and cached extension views.
    ExtensionsReloaded,
    /// Runtime has decided to shut down (e.g. the user typed `/quit`).  The
    /// frontend treats this as a request to leave the workspace and clean up
    /// its terminal state.  Sent before the runtime task exits; once it
    /// finishes, the event channel is dropped and `recv()` returns `None`.
    ShuttingDown,
    Error(String),
}

/// Frontend side of the runtime channels.
pub struct AgentHandle {
    pub commands: mpsc::Sender<AgentCommand>,
    pub events: mpsc::Receiver<AgentEvent>,
}

/// Owns the long-lived conversation and starts cancellable task executions.
pub struct AgentRuntime {
    config: Config,
    conversation: Conversation,
    provider_name: String,
    model_name: String,
    mode: AgentMode,
    session_store: SessionStore,
    engine_override: Option<Engine>,
    registry: commands::CommandRegistry,
}

impl AgentRuntime {
    pub fn new(
        config: Config,
        conversation: Conversation,
        provider_name: impl Into<String>,
        model_name: impl Into<String>,
        mode: AgentMode,
    ) -> Self {
        let session_store = SessionStore::from_config(&config);
        Self {
            config,
            conversation,
            provider_name: provider_name.into(),
            model_name: model_name.into(),
            mode,
            session_store,
            engine_override: None,
            registry: commands::defaults::default_registry(),
        }
    }

    /// Replace the runtime's command registry.  Used by tests and frontends
    /// that want to seed a smaller or extended registry.
    pub fn with_registry(mut self, registry: commands::CommandRegistry) -> Self {
        self.registry = registry;
        self
    }

    #[cfg(test)]
    fn with_engine(mut self, engine: Engine) -> Self {
        self.engine_override = Some(engine);
        self
    }

    /// Spawn the runtime actor and return bounded frontend channels.
    pub fn spawn(self) -> AgentHandle {
        let (command_tx, command_rx) = mpsc::channel(COMMAND_CAPACITY);
        let (event_tx, event_rx) = mpsc::channel(EVENT_CAPACITY);
        tokio::spawn(self.run(command_rx, event_tx));
        AgentHandle {
            commands: command_tx,
            events: event_rx,
        }
    }

    async fn run(
        self,
        mut commands: mpsc::Receiver<AgentCommand>,
        events: mpsc::Sender<AgentEvent>,
    ) {
        let Self {
            mut config,
            conversation,
            mut provider_name,
            mut model_name,
            mut mode,
            session_store,
            engine_override,
            mut registry,
        } = self;
        config.core.default_provider = Some(provider_name.clone());
        config.core.default_model = Some(model_name.clone());

        let mut context = Some(RuntimeContext::new(
            config.clone(),
            conversation,
            mode,
            session_store,
            engine_override,
        ));
        let (outcome_tx, mut outcome_rx) = mpsc::channel::<TaskOutcome>(2);
        let mut active: Option<ActiveTask> = None;
        let mut queued = VecDeque::<String>::new();

        if let Some(ctx) = context.as_ref() {
            send_event(
                &events,
                AgentEvent::SessionReady {
                    id: ctx.conversation.id.clone(),
                    history: ctx.conversation.messages.clone(),
                },
            )
            .await;
        }
        emit_git_state(&events).await;

        loop {
            tokio::select! {
                Some(command) = commands.recv() => {
                    match command {
                        AgentCommand::Submit(task) if !task.trim().is_empty() => {
                            if active.is_some() {
                                queued.push_back(task);
                                send_event(&events, AgentEvent::QueueChanged(queued.len())).await;
                            } else if let Some(ctx) = context.take() {
                                active = Some(launch_task(
                                    ctx,
                                    task,
                                    provider_name.clone(),
                                    events.clone(),
                                    outcome_tx.clone(),
                                ));
                            }
                        }
                        AgentCommand::Cancel => {
                            if let Some(task) = &active {
                                task.cancellation.cancel();
                            }
                        }
                        AgentCommand::Permission { request_id, decision } => {
                            if let Some(task) = &active {
                                let _ = task.permissions.send(PermissionResponse {
                                    request_id,
                                    decision,
                                }).await;
                            }
                        }
                        AgentCommand::SetMode(next) => {
                            if let Some(task) = &active {
                                task.cancellation.cancel();
                            }
                            mode = next;
                            if let Some(ctx) = context.as_mut() {
                                ctx.set_mode(next);
                            }
                            send_event(&events, AgentEvent::ModeChanged(next)).await;
                        }
                        AgentCommand::SetProvider(provider) => {
                            provider_name = provider.clone();
                            config.core.default_provider = Some(provider.clone());
                            if let Some(ctx) = context.as_mut() {
                                ctx.set_config(config.clone(), mode);
                            }
                            send_event(&events, AgentEvent::ProviderChanged(provider)).await;
                        }
                        AgentCommand::SetModel(model) => {
                            model_name = model.clone();
                            config.core.default_model = Some(model.clone());
                            if let Some(ctx) = context.as_mut() {
                                ctx.set_config(config.clone(), mode);
                            }
                            send_event(&events, AgentEvent::ModelChanged(model)).await;
                        }
                        AgentCommand::ClearConversation if active.is_none() => {
                            if let Some(ctx) = context.as_mut() {
                                ctx.conversation.clear();
                                send_event(&events, AgentEvent::ConversationCleared).await;
                            }
                        }
                        AgentCommand::LoadSession(id) if active.is_none() => {
                            if let Some(ctx) = context.as_mut() {
                                match ctx.session_store.load(&id) {
                                    Ok(conversation) => {
                                        ctx.conversation = conversation;
                                        send_event(&events, AgentEvent::SessionReady {
                                            id: ctx.conversation.id.clone(),
                                            history: ctx.conversation.messages.clone(),
                                        }).await;
                                    }
                                    Err(error) => send_event(&events, AgentEvent::Error(error.to_string())).await,
                                }
                            }
                        }
                        AgentCommand::SaveSession => {
                            if let Some(ctx) = context.as_ref() {
                                match ctx.session_store.save(&ctx.conversation) {
                                    Ok(()) => send_event(
                                        &events,
                                        AgentEvent::SessionSaved(ctx.conversation.id.clone()),
                                    ).await,
                                    Err(error) => send_event(&events, AgentEvent::Error(error.to_string())).await,
                                }
                            }
                        }
                        AgentCommand::DispatchSlash(input) => {
                            let mut dispatch = SlashDispatchState {
                                config: &mut config,
                                mode: &mut mode,
                                provider_name: &mut provider_name,
                                model_name: &mut model_name,
                                context: &mut context,
                                active: &mut active,
                                queued: &mut queued,
                                outcome_tx: &outcome_tx,
                                events: &events,
                                registry: &mut registry,
                            };
                            let should_shutdown = dispatch_slash(&input, &mut dispatch).await;
                            if should_shutdown {
                                if let Some(task) = &active {
                                    task.cancellation.cancel();
                                }
                                break;
                            }
                        }
                        AgentCommand::Shutdown => {
                            if let Some(task) = &active {
                                task.cancellation.cancel();
                            }
                            break;
                        }
                        _ => {}
                    }
                }
                Some(mut outcome) = outcome_rx.recv(), if active.is_some() => {
                    active = None;
                    if outcome.context.mode != mode || outcome.context.config != config {
                        outcome.context.set_config(config.clone(), mode);
                    }
                    context = Some(outcome.context);
                    emit_git_state(&events).await;
                    send_event(&events, AgentEvent::QueueChanged(queued.len())).await;
                    if let Some(next) = queued.pop_front() {
                        send_event(&events, AgentEvent::QueueChanged(queued.len())).await;
                        let ctx = context.take().expect("runtime context must be available");
                        active = Some(launch_task(
                            ctx,
                            next,
                            provider_name.clone(),
                            events.clone(),
                            outcome_tx.clone(),
                        ));
                    }
                }
                else => break,
            }
        }

        if let Some(ctx) = context.as_ref() {
            let _ = ctx.session_store.save(&ctx.conversation);
        }
    }
}

struct RuntimeContext {
    engine: Engine,
    config: Config,
    conversation: Conversation,
    mode: AgentMode,
    session_store: SessionStore,
    checkpoint_store: CheckpointStore,
    context_pins: Vec<ContextPin>,
    compression_history: Vec<CompressionRecord>,
    allowed_session: HashSet<String>,
    denied_session: HashSet<String>,
}

impl RuntimeContext {
    fn new(
        config: Config,
        conversation: Conversation,
        mode: AgentMode,
        session_store: SessionStore,
        engine_override: Option<Engine>,
    ) -> Self {
        let engine =
            engine_override.unwrap_or_else(|| Engine::configured(config.clone(), mode.sandbox()));
        let checkpoint_store = CheckpointStore::from_config(&config);
        Self {
            engine,
            config,
            conversation,
            mode,
            session_store,
            checkpoint_store,
            context_pins: Vec::new(),
            compression_history: Vec::new(),
            allowed_session: HashSet::new(),
            denied_session: HashSet::new(),
        }
    }

    fn set_mode(&mut self, mode: AgentMode) {
        self.set_config(self.config.clone(), mode);
    }

    fn set_config(&mut self, config: Config, mode: AgentMode) {
        self.engine = Engine::configured(config.clone(), mode.sandbox());
        self.config = config;
        self.mode = mode;
    }

    fn context_report(&self) -> ContextReport {
        ContextAccountant::new(self.config.session.context_size * 4).report(
            &self.conversation,
            &self.context_pins,
            &self.compression_history,
        )
    }
}

struct ActiveTask {
    cancellation: CancellationToken,
    permissions: mpsc::Sender<PermissionResponse>,
}

struct PermissionResponse {
    request_id: String,
    decision: PermissionDecision,
}

struct TaskOutcome {
    context: RuntimeContext,
}

fn launch_task(
    context: RuntimeContext,
    task: String,
    provider_name: String,
    events: mpsc::Sender<AgentEvent>,
    outcomes: mpsc::Sender<TaskOutcome>,
) -> ActiveTask {
    let cancellation = CancellationToken::new();
    let task_cancellation = cancellation.clone();
    let (permission_tx, permission_rx) = mpsc::channel(8);
    tokio::spawn(async move {
        let context = execute_task(
            context,
            task,
            provider_name,
            events,
            permission_rx,
            task_cancellation,
        )
        .await;
        let _ = outcomes.send(TaskOutcome { context }).await;
    });
    ActiveTask {
        cancellation,
        permissions: permission_tx,
    }
}

async fn execute_task(
    mut context: RuntimeContext,
    task: String,
    provider_name: String,
    events: mpsc::Sender<AgentEvent>,
    mut permissions: mpsc::Receiver<PermissionResponse>,
    cancellation: CancellationToken,
) -> RuntimeContext {
    let started = Instant::now();
    context.conversation.add_message(Message::user(&task));
    send_event(&events, AgentEvent::UserMessage(task.clone())).await;
    send_event(
        &events,
        AgentEvent::TaskStarted {
            task,
            started_at_ms: now_ms(),
        },
    )
    .await;
    send_event(
        &events,
        AgentEvent::Activity(Activity::new(
            "understand-task",
            AgentActivityKind::Planning,
            "Understanding task and repository context",
            AgentActivityStatus::Completed,
        )),
    )
    .await;

    let max_iterations = context.config.agent.max_tool_iterations;
    let mut loop_detector = DoomLoopDetector::new(context.config.agent.max_repeats as usize);
    for iteration in 0..max_iterations {
        let planning_id = format!("planning-{iteration}");
        send_event(
            &events,
            AgentEvent::Activity(Activity::new(
                &planning_id,
                AgentActivityKind::Planning,
                if iteration == 0 {
                    "Formulating an execution plan"
                } else {
                    "Evaluating tool results and next steps"
                },
                AgentActivityStatus::Running,
            )),
        )
        .await;

        let mut stream = match context
            .engine
            .chat_stream(&mut context.conversation, &provider_name)
            .await
        {
            Ok(stream) => stream,
            Err(error) => {
                fail_activity(&events, &planning_id, error.to_string()).await;
                send_event(
                    &events,
                    AgentEvent::TaskFailed {
                        message: error.to_string(),
                    },
                )
                .await;
                return context;
            }
        };

        let mut response = String::new();
        let mut tool_calls = Vec::new();
        let mut planning_completed = false;
        let mut stream_failed = None;

        loop {
            let event = tokio::select! {
                _ = cancellation.cancelled() => {
                    drop(stream);
                    cancel_activity(&events, &planning_id).await;
                    send_event(&events, AgentEvent::TaskCancelled).await;
                    let _ = context.session_store.save(&context.conversation);
                    return context;
                }
                event = stream.recv() => event,
            };
            let Some(event) = event else { break };
            if !planning_completed {
                complete_activity(
                    &events,
                    &planning_id,
                    AgentActivityKind::Planning,
                    "Execution plan ready",
                    None,
                )
                .await;
                planning_completed = true;
            }
            match event {
                StreamEvent::Token(delta) => {
                    response.push_str(&delta);
                    send_event(&events, AgentEvent::TextDelta(delta)).await;
                }
                StreamEvent::ReasoningToken(delta) => {
                    send_event(&events, AgentEvent::ReasoningDelta(delta)).await;
                }
                StreamEvent::ToolCall { id, name, input } => {
                    tool_calls.push(ToolCall { id, name, input });
                }
                StreamEvent::AgentActivity {
                    id,
                    kind,
                    title,
                    detail,
                    status,
                } => {
                    send_event(
                        &events,
                        AgentEvent::Activity(Activity {
                            id,
                            kind,
                            title,
                            detail,
                            status,
                            started_at_ms: now_ms(),
                            duration_ms: None,
                        }),
                    )
                    .await;
                }
                StreamEvent::ToolResult { id, content } => {
                    let (output, truncated) = truncate_owned(content, MAX_TOOL_OUTPUT);
                    send_event(
                        &events,
                        AgentEvent::ToolOutput {
                            activity_id: id,
                            output,
                            truncated,
                        },
                    )
                    .await;
                }
                StreamEvent::Done { usage, .. } => {
                    if let Some(usage) = usage {
                        send_event(&events, AgentEvent::Usage(usage)).await;
                    }
                    break;
                }
                StreamEvent::Error { message, code } => {
                    stream_failed = Some(match code {
                        Some(code) => format!("{message} ({code})"),
                        None => message,
                    });
                    break;
                }
            }
        }

        if !planning_completed {
            complete_activity(
                &events,
                &planning_id,
                AgentActivityKind::Planning,
                "Execution plan ready",
                None,
            )
            .await;
        }
        if let Some(message) = stream_failed {
            send_event(&events, AgentEvent::TaskFailed { message }).await;
            return context;
        }

        if tool_calls.is_empty() {
            if !response.is_empty() {
                context
                    .conversation
                    .add_message(Message::assistant(response.clone()));
                send_event(&events, AgentEvent::AssistantMessageCompleted(response)).await;
            }
            let review_id = "review-final-diff";
            send_event(
                &events,
                AgentEvent::Activity(Activity::new(
                    review_id,
                    AgentActivityKind::Reviewing,
                    "Reviewing final workspace diff",
                    AgentActivityStatus::Running,
                )),
            )
            .await;
            let diff = workspace_diff().await;
            complete_activity(
                &events,
                review_id,
                AgentActivityKind::Reviewing,
                "Reviewed final workspace diff",
                None,
            )
            .await;
            send_event(&events, AgentEvent::DiffUpdated(diff)).await;
            let _ = context.session_store.save(&context.conversation);
            send_event(
                &events,
                AgentEvent::TaskCompleted {
                    elapsed_ms: started.elapsed().as_millis() as u64,
                },
            )
            .await;
            return context;
        }

        let mut blocks = Vec::new();
        if !response.is_empty() {
            blocks.push(ContentBlock::Text(response));
        }
        for call in &tool_calls {
            blocks.push(ContentBlock::ToolUse {
                id: call.id.clone(),
                name: call.name.clone(),
                input: call.input.clone(),
            });
        }
        context.conversation.add_message(Message {
            role: MessageRole::Assistant,
            content: blocks,
            reasoning: None,
            metadata: None,
        });

        for call in tool_calls {
            let kind = activity_kind_for_tool(&call.name);
            let target = permission_target(&call.input);
            send_event(
                &events,
                AgentEvent::Activity(Activity::new(
                    &call.id,
                    kind,
                    format!("{} {target}", call.name),
                    AgentActivityStatus::Queued,
                )),
            )
            .await;

            let allowed = match request_permission_if_needed(
                &mut context,
                &call,
                &events,
                &mut permissions,
                &cancellation,
            )
            .await
            {
                PermissionOutcome::Allowed => true,
                PermissionOutcome::Denied => false,
                PermissionOutcome::Cancelled => {
                    cancel_activity(&events, &call.id).await;
                    send_event(&events, AgentEvent::TaskCancelled).await;
                    return context;
                }
            };

            if !allowed {
                fail_activity(&events, &call.id, "Denied by permission policy").await;
                add_tool_result(
                    &mut context.conversation,
                    &call.id,
                    "Tool use denied by permission policy".to_string(),
                    true,
                );
                continue;
            }

            send_event(
                &events,
                AgentEvent::Activity(Activity::new(
                    &call.id,
                    kind,
                    format!("{} {target}", call.name),
                    AgentActivityStatus::Running,
                )),
            )
            .await;
            let tool_started = Instant::now();
            let result = tokio::select! {
                _ = cancellation.cancelled() => {
                    cancel_activity(&events, &call.id).await;
                    send_event(&events, AgentEvent::TaskCancelled).await;
                    return context;
                }
                result = context.engine.execute_tool(&call.name, call.input.clone()) => result,
            };

            match result {
                Ok(result) => {
                    let raw = if result.success {
                        result.content
                    } else {
                        result.error.unwrap_or(result.content)
                    };
                    let (output, truncated) = truncate_owned(raw.clone(), MAX_TOOL_OUTPUT);
                    send_event(
                        &events,
                        AgentEvent::ToolOutput {
                            activity_id: call.id.clone(),
                            output: output.clone(),
                            truncated,
                        },
                    )
                    .await;
                    if result.success {
                        let mut activity = Activity::new(
                            &call.id,
                            kind,
                            format!("{} {target}", call.name),
                            AgentActivityStatus::Completed,
                        );
                        activity.detail = Some(output);
                        activity.duration_ms = Some(tool_started.elapsed().as_millis() as u64);
                        send_event(&events, AgentEvent::Activity(activity)).await;
                    } else {
                        fail_activity(&events, &call.id, raw.clone()).await;
                    }
                    let failure_text = (!result.success).then(|| raw.clone());
                    add_tool_result(&mut context.conversation, &call.id, raw, !result.success);
                    if let Some(failure_text) = failure_text {
                        let Some(reason) = loop_detector.record(LoopSignal::ToolFailure {
                            tool: call.name.clone(),
                            target: target.clone(),
                            error: failure_text,
                        }) else {
                            continue;
                        };
                        send_event(&events, AgentEvent::TaskFailed { message: reason }).await;
                        let _ = context.session_store.save(&context.conversation);
                        return context;
                    }
                }
                Err(error) => {
                    let error = error.to_string();
                    fail_activity(&events, &call.id, error.clone()).await;
                    add_tool_result(
                        &mut context.conversation,
                        &call.id,
                        format!("Error: {error}"),
                        true,
                    );
                    if let Some(reason) = loop_detector.record(LoopSignal::ToolFailure {
                        tool: call.name.clone(),
                        target: target.clone(),
                        error,
                    }) {
                        send_event(&events, AgentEvent::TaskFailed { message: reason }).await;
                        let _ = context.session_store.save(&context.conversation);
                        return context;
                    }
                }
            }
        }
        let _ = context.session_store.save(&context.conversation);
    }

    send_event(
        &events,
        AgentEvent::TaskFailed {
            message: format!("Agent stopped after {max_iterations} tool iterations"),
        },
    )
    .await;
    context
}

struct ToolCall {
    id: String,
    name: String,
    input: serde_json::Value,
}

enum PermissionOutcome {
    Allowed,
    Denied,
    Cancelled,
}

async fn request_permission_if_needed(
    context: &mut RuntimeContext,
    call: &ToolCall,
    events: &mpsc::Sender<AgentEvent>,
    permissions: &mut mpsc::Receiver<PermissionResponse>,
    cancellation: &CancellationToken,
) -> PermissionOutcome {
    if context
        .config
        .permissions
        .always_deny
        .iter()
        .any(|name| name == &call.name)
        || context.denied_session.contains(&call.name)
    {
        return PermissionOutcome::Denied;
    }
    let level = match context.engine.tool_permission_level(&call.name) {
        Ok(level) => level,
        Err(_) => return PermissionOutcome::Denied,
    };
    let rule_decision = match PermissionEngine::new(context.config.permissions.rules.clone()) {
        Ok(engine) => engine.evaluate(&tool_invocation(context, call)),
        Err(_) => return PermissionOutcome::Denied,
    };
    if rule_decision.kind == DecisionKind::Deny {
        return PermissionOutcome::Denied;
    }
    if context.mode.sandbox_policy() == SandboxPolicy::ReadOnly
        && level != PermissionLevel::ReadOnly
    {
        return PermissionOutcome::Denied;
    }
    if rule_decision.kind == DecisionKind::Allow {
        return PermissionOutcome::Allowed;
    }
    let explicitly_allowed = context.allowed_session.contains(&call.name)
        || context
            .config
            .permissions
            .always_allow
            .iter()
            .any(|name| name == &call.name);
    let policy_allows = match context.mode.approval_policy() {
        ApprovalPolicy::Never | ApprovalPolicy::OnFailure => true,
        ApprovalPolicy::OnRisk => false,
        ApprovalPolicy::Always => false,
    };
    if rule_decision.kind != DecisionKind::Ask
        && (level == PermissionLevel::ReadOnly || explicitly_allowed || policy_allows)
    {
        return PermissionOutcome::Allowed;
    }

    let target = permission_target(&call.input);
    let description = context
        .engine
        .tool_description(&call.name)
        .unwrap_or("perform the requested operation");
    let request = PermissionRequest {
        id: call.id.clone(),
        tool: call.name.clone(),
        operation: description.to_string(),
        target,
        reason: if rule_decision.kind == DecisionKind::Ask {
            rule_decision.reason
        } else {
            "The model requested this operation to continue the current task.".to_string()
        },
        risk: match level {
            PermissionLevel::ReadOnly => "Reads project or network data".to_string(),
            PermissionLevel::WorkspaceWrite => {
                "May create or modify files inside the workspace".to_string()
            }
            PermissionLevel::Dangerous => {
                "May execute commands or perform operations with side effects".to_string()
            }
        },
        input: call.input.clone(),
    };
    let mut waiting = Activity::new(
        &call.id,
        activity_kind_for_tool(&call.name),
        format!("{} {}", call.name, request.target),
        AgentActivityStatus::WaitingForApproval,
    );
    waiting.detail = Some(request.risk.clone());
    send_event(events, AgentEvent::Activity(waiting)).await;
    send_event(events, AgentEvent::PermissionRequested(request)).await;

    loop {
        let response = tokio::select! {
            _ = cancellation.cancelled() => return PermissionOutcome::Cancelled,
            response = permissions.recv() => response,
        };
        let Some(response) = response else {
            return PermissionOutcome::Denied;
        };
        if response.request_id != call.id {
            continue;
        }
        return match response.decision {
            PermissionDecision::AllowOnce => PermissionOutcome::Allowed,
            PermissionDecision::AllowSession => {
                context.allowed_session.insert(call.name.clone());
                PermissionOutcome::Allowed
            }
            PermissionDecision::DenyOnce => PermissionOutcome::Denied,
            PermissionDecision::DenySession => {
                context.denied_session.insert(call.name.clone());
                PermissionOutcome::Denied
            }
        };
    }
}

fn tool_invocation(context: &RuntimeContext, call: &ToolCall) -> ToolInvocation {
    let process_cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let cwd = call
        .input
        .get("workdir")
        .and_then(|value| value.as_str())
        .map(|workdir| resolve_path(&process_cwd, workdir))
        .unwrap_or_else(|| process_cwd.clone());
    let workspace_root = if context.mode.sandbox_policy() == SandboxPolicy::FullAccess {
        std::path::PathBuf::new()
    } else {
        process_cwd
    };
    let command = call
        .input
        .get("command")
        .and_then(|value| value.as_str())
        .map(ToString::to_string);
    let mut invocation = ToolInvocation {
        tool: call.name.clone(),
        command: command.clone(),
        cwd,
        workspace_root,
        target_paths: collect_target_paths(&call.input),
        network_destinations: collect_network_destinations(&call.input),
        plugin_source: call
            .input
            .get("source")
            .and_then(|value| value.as_str())
            .map(ToString::to_string),
        mcp_server: call
            .input
            .get("mcp_server")
            .and_then(|value| value.as_str())
            .map(ToString::to_string),
        mcp_tool: call
            .input
            .get("mcp_tool")
            .and_then(|value| value.as_str())
            .map(ToString::to_string),
        env_vars: collect_env_vars(&call.input),
    };
    if let Some(command) = command {
        invocation
            .network_destinations
            .extend(network_destinations_from_command(&command));
        invocation.env_vars.extend(env_vars_from_command(&command));
    }
    invocation.network_destinations.sort();
    invocation.network_destinations.dedup();
    invocation.env_vars.sort();
    invocation.env_vars.dedup();
    invocation
}

fn collect_target_paths(input: &serde_json::Value) -> Vec<std::path::PathBuf> {
    let mut paths = Vec::new();
    for key in [
        "path",
        "file_path",
        "directory",
        "target",
        "dest",
        "destination",
    ] {
        if let Some(path) = input.get(key).and_then(|value| value.as_str()) {
            paths.push(std::path::PathBuf::from(path));
        }
    }
    if let Some(items) = input.get("paths").and_then(|value| value.as_array()) {
        paths.extend(
            items
                .iter()
                .filter_map(|value| value.as_str())
                .map(std::path::PathBuf::from),
        );
    }
    paths
}

fn collect_network_destinations(input: &serde_json::Value) -> Vec<String> {
    let mut destinations = Vec::new();
    for key in ["url", "endpoint", "base_url"] {
        if let Some(value) = input.get(key).and_then(|value| value.as_str()) {
            destinations.push(value.to_string());
        }
    }
    if let Some(items) = input.get("urls").and_then(|value| value.as_array()) {
        destinations.extend(
            items
                .iter()
                .filter_map(|value| value.as_str())
                .map(ToString::to_string),
        );
    }
    destinations
}

fn collect_env_vars(input: &serde_json::Value) -> Vec<String> {
    input
        .get("env")
        .and_then(|value| value.as_object())
        .map(|env| env.keys().cloned().collect())
        .unwrap_or_default()
}

fn network_destinations_from_command(command: &str) -> Vec<String> {
    parse_shell(command)
        .ok()
        .into_iter()
        .flat_map(|program| program.clauses)
        .flat_map(|clause| clause.words)
        .filter(|word| word.starts_with("http://") || word.starts_with("https://"))
        .collect()
}

fn env_vars_from_command(command: &str) -> Vec<String> {
    parse_shell(command)
        .ok()
        .into_iter()
        .flat_map(|program| program.clauses)
        .flat_map(|clause| clause.words)
        .filter_map(|word| {
            if let Some((name, _)) = word.split_once('=') {
                if is_env_name(name) {
                    return Some(name.to_string());
                }
            }
            word.strip_prefix('$')
                .filter(|name| is_env_name(name))
                .map(ToString::to_string)
        })
        .collect()
}

fn is_env_name(value: &str) -> bool {
    let mut chars = value.chars();
    chars
        .next()
        .is_some_and(|character| character == '_' || character.is_ascii_alphabetic())
        && chars.all(|character| character == '_' || character.is_ascii_alphanumeric())
}

fn resolve_path(base: &std::path::Path, path: &str) -> std::path::PathBuf {
    let path = std::path::PathBuf::from(path);
    if path.is_absolute() {
        path
    } else {
        base.join(path)
    }
}

fn activity_kind_for_tool(name: &str) -> AgentActivityKind {
    match name {
        "read" | "fetch" => AgentActivityKind::Reading,
        "glob" | "grep" | "search" => AgentActivityKind::Searching,
        "write" => AgentActivityKind::Writing,
        "edit" => AgentActivityKind::Editing,
        "diff" => AgentActivityKind::Reviewing,
        "bash" => AgentActivityKind::Executing,
        _ => AgentActivityKind::Tool,
    }
}

fn permission_target(input: &serde_json::Value) -> String {
    ["path", "command", "url", "pattern", "query", "workdir"]
        .iter()
        .find_map(|key| input.get(key).and_then(|value| value.as_str()))
        .unwrap_or("workspace")
        .chars()
        .take(240)
        .collect()
}

fn add_tool_result(conversation: &mut Conversation, id: &str, content: String, is_error: bool) {
    conversation.add_message(Message {
        role: MessageRole::Tool,
        content: vec![ContentBlock::ToolResult {
            id: id.to_string(),
            content,
            is_error,
        }],
        reasoning: None,
        metadata: None,
    });
}

async fn complete_activity(
    events: &mpsc::Sender<AgentEvent>,
    id: &str,
    kind: AgentActivityKind,
    title: &str,
    detail: Option<String>,
) {
    let mut activity = Activity::new(id, kind, title, AgentActivityStatus::Completed);
    activity.detail = detail;
    send_event(events, AgentEvent::Activity(activity)).await;
}

async fn fail_activity(events: &mpsc::Sender<AgentEvent>, id: &str, detail: impl Into<String>) {
    let mut activity = Activity::new(
        id,
        AgentActivityKind::Tool,
        "Operation failed",
        AgentActivityStatus::Failed,
    );
    activity.detail = Some(detail.into());
    send_event(events, AgentEvent::Activity(activity)).await;
}

async fn cancel_activity(events: &mpsc::Sender<AgentEvent>, id: &str) {
    send_event(
        events,
        AgentEvent::Activity(Activity::new(
            id,
            AgentActivityKind::Tool,
            "Operation cancelled",
            AgentActivityStatus::Cancelled,
        )),
    )
    .await;
}

async fn workspace_diff() -> String {
    let output = tokio::process::Command::new("git")
        .args(["diff", "--no-ext-diff", "--"])
        .output()
        .await;
    match output {
        Ok(output) if output.status.success() => {
            truncate_owned(
                String::from_utf8_lossy(&output.stdout).to_string(),
                MAX_DIFF_OUTPUT,
            )
            .0
        }
        _ => String::new(),
    }
}

async fn emit_git_state(events: &mpsc::Sender<AgentEvent>) {
    let branch = tokio::process::Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .await
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|branch| !branch.is_empty());
    let dirty = tokio::process::Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .await
        .ok()
        .filter(|output| output.status.success())
        .map(|output| !output.stdout.is_empty())
        .unwrap_or(false);
    send_event(events, AgentEvent::GitState { branch, dirty }).await;
}

fn truncate_owned(value: String, limit: usize) -> (String, bool) {
    if value.len() <= limit {
        return (value, false);
    }
    let boundary = value
        .char_indices()
        .map(|(index, _)| index)
        .take_while(|index| *index <= limit)
        .last()
        .unwrap_or(0);
    (
        format!(
            "{}\n… output truncated at {} KiB …",
            &value[..boundary],
            limit / 1024
        ),
        true,
    )
}

async fn send_event(events: &mpsc::Sender<AgentEvent>, event: AgentEvent) {
    let _ = events.send(event).await;
}

fn now_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis()
}

/// Resolve and execute a slash command (with leading `/`) via the command
/// registry, applying the resulting [`CommandResult`] as runtime state
/// changes (for [`CommandResult::Effects`]) or forwarding overlay /
/// notification / document requests to the frontend as [`AgentEvent`]
/// variants.  Returns `true` when the runtime should shut down (e.g. the
/// handler emitted `Quit` / `Shutdown`).
struct SlashDispatchState<'a> {
    config: &'a mut Config,
    mode: &'a mut AgentMode,
    provider_name: &'a mut String,
    model_name: &'a mut String,
    context: &'a mut Option<RuntimeContext>,
    active: &'a mut Option<ActiveTask>,
    queued: &'a mut VecDeque<String>,
    outcome_tx: &'a mpsc::Sender<TaskOutcome>,
    events: &'a mpsc::Sender<AgentEvent>,
    registry: &'a mut commands::CommandRegistry,
}

async fn dispatch_slash(input: &str, state: &mut SlashDispatchState<'_>) -> bool {
    let ctx = commands::CommandContext::new(
        state.provider_name.clone(),
        state.model_name.clone(),
        state.mode.label().to_string(),
    );
    let result = match state.registry.dispatch(input, &ctx, true).await {
        Ok(res) => res,
        Err(err) => {
            send_event(state.events, AgentEvent::Error(err.to_string())).await;
            return false;
        }
    };
    match result {
        CommandResult::Effects(effects) => {
            for effect in effects {
                if apply_app_effect(effect, state).await {
                    return true;
                }
            }
        }
        CommandResult::OpenOverlay(kind) => {
            send_event(state.events, AgentEvent::OpenOverlay(kind)).await;
        }
        CommandResult::Notification(notification) => {
            send_event(state.events, AgentEvent::Notify(notification)).await;
        }
        CommandResult::RenderDocument(document) => {
            send_event(state.events, AgentEvent::Document(document)).await;
        }
        CommandResult::RuntimeRestart(req) => {
            send_event(
                state.events,
                AgentEvent::Error(format!("runtime restart requested: {}", req.reason)),
            )
            .await;
        }
        CommandResult::BackgroundTask(handle) => {
            send_event(
                state.events,
                AgentEvent::Notify(Notification {
                    level: NotificationLevel::Info,
                    message: format!("background task spawned: {}", handle.id),
                }),
            )
            .await;
        }
        CommandResult::Noop => {}
    }
    false
}

/// Apply one [`AppEffect`] to the runtime.  Returns `true` when the loop
/// should shut down afterwards (effects `Quit` and `Shutdown`).
async fn apply_app_effect(effect: AppEffect, state: &mut SlashDispatchState<'_>) -> bool {
    match effect {
        AppEffect::SetMode(name) => {
            if let Some(task) = state.active.as_ref() {
                task.cancellation.cancel();
            }
            let next = AgentMode::parse(&name);
            *state.mode = next;
            if let Some(ctx) = state.context.as_mut() {
                ctx.set_mode(next);
            }
            send_event(state.events, AgentEvent::ModeChanged(next)).await;
        }
        AppEffect::SetProvider(name) => {
            *state.provider_name = name.clone();
            state.config.core.default_provider = Some(name.clone());
            if let Some(ctx) = state.context.as_mut() {
                ctx.set_config(state.config.clone(), *state.mode);
            }
            send_event(state.events, AgentEvent::ProviderChanged(name)).await;
        }
        AppEffect::SetModel(name) => {
            *state.model_name = name.clone();
            state.config.core.default_model = Some(name.clone());
            if let Some(ctx) = state.context.as_mut() {
                ctx.set_config(state.config.clone(), *state.mode);
            }
            send_event(state.events, AgentEvent::ModelChanged(name)).await;
        }
        AppEffect::ClearConversation => {
            if state.active.is_none() {
                if let Some(ctx) = state.context.as_mut() {
                    ctx.conversation.clear();
                    send_event(state.events, AgentEvent::ConversationCleared).await;
                }
            }
        }
        AppEffect::LoadSession(id) => {
            if state.active.is_none() {
                if let Some(ctx) = state.context.as_mut() {
                    match ctx.session_store.load(&id) {
                        Ok(conv) => {
                            ctx.conversation = conv;
                            send_event(
                                state.events,
                                AgentEvent::SessionReady {
                                    id: ctx.conversation.id.clone(),
                                    history: ctx.conversation.messages.clone(),
                                },
                            )
                            .await;
                        }
                        Err(e) => send_event(state.events, AgentEvent::Error(e.to_string())).await,
                    }
                }
            }
        }
        AppEffect::SaveSession => {
            if let Some(ctx) = state.context.as_ref() {
                match ctx.session_store.save(&ctx.conversation) {
                    Ok(()) => {
                        send_event(
                            state.events,
                            AgentEvent::SessionSaved(ctx.conversation.id.clone()),
                        )
                        .await;
                    }
                    Err(e) => send_event(state.events, AgentEvent::Error(e.to_string())).await,
                }
            }
        }
        AppEffect::CreateCheckpoint(name) => {
            if state.active.is_none() {
                if let Some(ctx) = state.context.as_ref() {
                    match ctx.checkpoint_store.create(
                        &ctx.conversation,
                        state.provider_name,
                        state.model_name,
                        *state.mode,
                        name,
                    ) {
                        Ok(record) => {
                            send_event(
                                state.events,
                                AgentEvent::Notify(Notification {
                                    level: NotificationLevel::Success,
                                    message: format!(
                                        "Checkpoint created: {}",
                                        checkpoint_label(&record)
                                    ),
                                }),
                            )
                            .await;
                        }
                        Err(error) => {
                            send_event(state.events, AgentEvent::Error(error.to_string())).await
                        }
                    }
                }
            }
        }
        AppEffect::ListCheckpoints => {
            if let Some(ctx) = state.context.as_ref() {
                match ctx.checkpoint_store.list() {
                    Ok(checkpoints) => {
                        let mut document = RenderableDocument::new("Checkpoints");
                        if checkpoints.is_empty() {
                            document.push_section(
                                "No checkpoints",
                                "Create one with /checkpoint create [name].",
                            );
                        } else {
                            for checkpoint in checkpoints {
                                document.push_section(
                                    checkpoint.name.unwrap_or_else(|| checkpoint.id.clone()),
                                    format!(
                                        "ID: {}\nMessages: {}\nProvider/model: {}/{}\nMode: {}\nChanged files: {}",
                                        checkpoint.id,
                                        checkpoint.message_count,
                                        checkpoint.provider,
                                        checkpoint.model,
                                        checkpoint.mode.label(),
                                        format_changed_files(&checkpoint.changed_files)
                                    ),
                                );
                            }
                        }
                        send_event(state.events, AgentEvent::Document(document)).await;
                    }
                    Err(error) => {
                        send_event(state.events, AgentEvent::Error(error.to_string())).await
                    }
                }
            }
        }
        AppEffect::ShowCheckpoint(id) => {
            if let Some(ctx) = state.context.as_ref() {
                match ctx.checkpoint_store.load(&id) {
                    Ok(record) => {
                        send_event(
                            state.events,
                            AgentEvent::Document(checkpoint_document(&record, false)),
                        )
                        .await;
                    }
                    Err(error) => {
                        send_event(state.events, AgentEvent::Error(error.to_string())).await
                    }
                }
            }
        }
        AppEffect::RestoreCheckpoint { id, confirm } => {
            if state.active.is_none() {
                if let Some(ctx) = state.context.as_mut() {
                    match ctx.checkpoint_store.load(&id) {
                        Ok(record) if confirm => {
                            match ctx.checkpoint_store.restore_workspace(&record) {
                                Ok(backup) => {
                                    ctx.conversation = record.conversation.clone();
                                    let _ = ctx.session_store.save(&ctx.conversation);
                                    send_event(
                                        state.events,
                                        AgentEvent::SessionReady {
                                            id: ctx.conversation.id.clone(),
                                            history: ctx.conversation.messages.clone(),
                                        },
                                    )
                                    .await;
                                    send_event(state.events, AgentEvent::Notify(Notification {
                                        level: NotificationLevel::Success,
                                        message: backup
                                            .map(|path| format!("Checkpoint restored; previous diff backed up at {}", path.display()))
                                            .unwrap_or_else(|| "Checkpoint restored".to_string()),
                                    })).await;
                                    emit_git_state(state.events).await;
                                }
                                Err(error) => {
                                    send_event(state.events, AgentEvent::Error(error.to_string()))
                                        .await
                                }
                            }
                        }
                        Ok(record) => {
                            send_event(
                                state.events,
                                AgentEvent::Document(checkpoint_document(&record, true)),
                            )
                            .await;
                        }
                        Err(error) => {
                            send_event(state.events, AgentEvent::Error(error.to_string())).await
                        }
                    }
                }
            }
        }
        AppEffect::DeleteCheckpoint(id) => {
            if let Some(ctx) = state.context.as_ref() {
                match ctx.checkpoint_store.delete(&id) {
                    Ok(()) => {
                        send_event(
                            state.events,
                            AgentEvent::Notify(Notification {
                                level: NotificationLevel::Success,
                                message: format!("Checkpoint deleted: {id}"),
                            }),
                        )
                        .await;
                    }
                    Err(error) => {
                        send_event(state.events, AgentEvent::Error(error.to_string())).await
                    }
                }
            }
        }
        AppEffect::ContextStatus => {
            if let Some(ctx) = state.context.as_ref() {
                send_event(
                    state.events,
                    AgentEvent::Document(context_status_document(&ctx.context_report())),
                )
                .await;
            }
        }
        AppEffect::ContextInspect => {
            if let Some(ctx) = state.context.as_ref() {
                send_event(
                    state.events,
                    AgentEvent::Document(context_inspect_document(&ctx.context_report())),
                )
                .await;
            }
        }
        AppEffect::ContextCompact => {
            if state.active.is_none() {
                if let Some(ctx) = state.context.as_mut() {
                    match compact_context(ctx).await {
                        Ok(record) => {
                            let _ = ctx.session_store.save(&ctx.conversation);
                            send_event(
                                state.events,
                                AgentEvent::Document(context_compaction_document(
                                    &ctx.context_report(),
                                    &record,
                                )),
                            )
                            .await;
                        }
                        Err(error) => {
                            send_event(state.events, AgentEvent::Error(error.to_string())).await
                        }
                    }
                }
            }
        }
        AppEffect::ContextPin(target) => {
            if let Some(ctx) = state.context.as_mut() {
                let id = format!("pin-{}", ctx.context_pins.len() + 1);
                let pin = make_pin(id.clone(), target);
                ctx.context_pins.push(pin);
                send_event(
                    state.events,
                    AgentEvent::Notify(Notification {
                        level: NotificationLevel::Success,
                        message: format!("Context pin added: {id}"),
                    }),
                )
                .await;
                send_event(
                    state.events,
                    AgentEvent::Document(context_status_document(&ctx.context_report())),
                )
                .await;
            }
        }
        AppEffect::ContextUnpin(id) => {
            if let Some(ctx) = state.context.as_mut() {
                let before = ctx.context_pins.len();
                ctx.context_pins.retain(|pin| pin.id != id);
                let removed = before != ctx.context_pins.len();
                send_event(
                    state.events,
                    AgentEvent::Notify(Notification {
                        level: if removed {
                            NotificationLevel::Success
                        } else {
                            NotificationLevel::Warning
                        },
                        message: if removed {
                            format!("Context pin removed: {id}")
                        } else {
                            format!("Context pin not found: {id}")
                        },
                    }),
                )
                .await;
            }
        }
        AppEffect::ContextSources => {
            if let Some(ctx) = state.context.as_ref() {
                send_event(
                    state.events,
                    AgentEvent::Document(context_sources_document(&ctx.context_report())),
                )
                .await;
            }
        }
        AppEffect::Verify => {
            start_verification(
                VerificationScope::Full,
                None,
                *state.mode,
                state.events.clone(),
            )
            .await;
        }
        AppEffect::Test => {
            start_verification(
                VerificationScope::Test,
                None,
                *state.mode,
                state.events.clone(),
            )
            .await;
        }
        AppEffect::RunCommand(command) => {
            start_verification(
                VerificationScope::Run,
                Some(command),
                *state.mode,
                state.events.clone(),
            )
            .await;
        }
        AppEffect::ReviewDiff { staged } => {
            match pleiades_agent_git::working_diff_review(std::path::Path::new("."), staged).await {
                Ok(review) => {
                    send_event(state.events, AgentEvent::DiffUpdated(review.raw.clone())).await;
                    send_event(
                        state.events,
                        AgentEvent::Document(diff_review_document(&review)),
                    )
                    .await;
                }
                Err(error) => {
                    send_event(state.events, AgentEvent::Error(error.to_string())).await;
                }
            }
        }
        AppEffect::GitStatus => match git_status_document().await {
            Ok(document) => send_event(state.events, AgentEvent::Document(document)).await,
            Err(error) => send_event(state.events, AgentEvent::Error(error.to_string())).await,
        },
        AppEffect::SubmitPrompt(prompt) => {
            if prompt.trim().is_empty() {
                send_event(
                    state.events,
                    AgentEvent::Notify(Notification {
                        level: NotificationLevel::Warning,
                        message: "custom command rendered an empty prompt".to_string(),
                    }),
                )
                .await;
            } else if state.active.is_some() {
                state.queued.push_back(prompt);
                send_event(state.events, AgentEvent::QueueChanged(state.queued.len())).await;
            } else if let Some(ctx) = state.context.take() {
                *state.active = Some(launch_task(
                    ctx,
                    prompt,
                    state.provider_name.clone(),
                    state.events.clone(),
                    state.outcome_tx.clone(),
                ));
            }
        }
        AppEffect::CancelTask => {
            if let Some(task) = state.active.as_ref() {
                task.cancellation.cancel();
            }
        }
        AppEffect::Quit | AppEffect::Shutdown => {
            if let Some(task) = state.active.as_ref() {
                task.cancellation.cancel();
            }
            send_event(state.events, AgentEvent::ShuttingDown).await;
            return true;
        }
        AppEffect::ReloadExtensions => {
            *state.registry = commands::defaults::default_registry();
            send_event(state.events, AgentEvent::ExtensionsReloaded).await;
            send_event(
                state.events,
                AgentEvent::Notify(Notification {
                    level: NotificationLevel::Success,
                    message: "Extensions reloaded".to_string(),
                }),
            )
            .await;
        }
        AppEffect::Status(s) => {
            send_event(
                state.events,
                AgentEvent::Notify(Notification {
                    level: NotificationLevel::Info,
                    message: s,
                }),
            )
            .await;
        }
        AppEffect::Custom(name) => {
            send_event(
                state.events,
                AgentEvent::Error(format!(
                    "custom effect `{name}` not implemented in this slice"
                )),
            )
            .await;
        }
    }
    false
}

async fn git_status_document() -> Result<RenderableDocument, pleiades_agent_core::Error> {
    let output = tokio::process::Command::new("git")
        .args(["status", "--short", "--branch"])
        .output()
        .await?;
    if !output.status.success() {
        return Err(pleiades_agent_core::Error::tool(
            String::from_utf8_lossy(&output.stderr).trim(),
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(RenderableDocument::new("Git status").section(
        "Status",
        if stdout.trim().is_empty() {
            "(clean)".to_string()
        } else {
            stdout.into_owned()
        },
    ))
}

fn diff_review_document(review: &pleiades_agent_git::DiffReview) -> RenderableDocument {
    let mut document = RenderableDocument::new(if review.staged {
        "Staged diff review"
    } else {
        "Working-tree diff review"
    })
    .section("Summary", review.summary());
    for (file_index, file) in review.files.iter().enumerate() {
        let path = file
            .new_path
            .as_ref()
            .or(file.old_path.as_ref())
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "(unknown)".to_string());
        let mut body = String::new();
        body.push_str(&format!(
            "File index: {file_index}\nOld: {}\nNew: {}\nBinary: {}\nHunks: {}\n",
            file.old_path
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "/dev/null".to_string()),
            file.new_path
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "/dev/null".to_string()),
            file.binary,
            file.hunks.len()
        ));
        for (hunk_index, hunk) in file.hunks.iter().enumerate() {
            body.push_str(&format!(
                "\nHunk {hunk_index}: {}\n-old {},{} +new {},{}\n",
                hunk.header, hunk.old_start, hunk.old_len, hunk.new_start, hunk.new_len
            ));
            for line in hunk.lines.iter().take(80) {
                let prefix = match line.kind {
                    pleiades_agent_git::DiffLineKind::Context => " ",
                    pleiades_agent_git::DiffLineKind::Added => "+",
                    pleiades_agent_git::DiffLineKind::Removed => "-",
                    pleiades_agent_git::DiffLineKind::NoNewline => "\\",
                };
                body.push_str(prefix);
                body.push_str(line.content.trim_end_matches('\n'));
                body.push('\n');
            }
            if hunk.lines.len() > 80 {
                body.push_str("… hunk truncated in review document\n");
            }
        }
        document.push_section(path, body);
    }
    document
}

fn checkpoint_label(record: &CheckpointRecord) -> String {
    record
        .name
        .clone()
        .unwrap_or_else(|| record.id.chars().take(8).collect())
}

fn checkpoint_document(record: &CheckpointRecord, restore_preview: bool) -> RenderableDocument {
    let mut document = RenderableDocument::new(if restore_preview {
        "Checkpoint restore preview"
    } else {
        "Checkpoint"
    });
    document.push_section("ID", record.id.clone());
    document.push_section("Name", record.name.as_deref().unwrap_or("(none)"));
    document.push_section("Messages", record.conversation.messages.len().to_string());
    document.push_section(
        "Provider/model",
        format!("{}/{}", record.provider, record.model),
    );
    document.push_section("Mode", record.mode.label());
    document.push_section(
        "Git branch",
        record.git_branch.as_deref().unwrap_or("(not git)"),
    );
    document.push_section(
        "Git HEAD",
        record.git_head.as_deref().unwrap_or("(not git)"),
    );
    document.push_section("Changed files", format_changed_files(&record.changed_files));
    if restore_preview {
        document.push_section(
            "Restore",
            format!(
                "Preview only. Run /checkpoint restore {} --confirm to restore this conversation and tracked Git diff.",
                record.id
            ),
        );
    }
    document
}

fn format_changed_files(files: &[String]) -> String {
    if files.is_empty() {
        "(none)".to_string()
    } else {
        files.join("\n")
    }
}

fn context_status_document(report: &ContextReport) -> RenderableDocument {
    RenderableDocument::new("Context status")
        .section(
            "Usage",
            format!(
                "{} / {} tokens ({}%)",
                report.total_tokens, report.provider_context_limit, report.percent_used
            ),
        )
        .section(
            "Contributions",
            format!(
                "Conversation: {}\nTool output: {}\nMemory: {}\nCompression summaries: {}\nPinned: {}",
                report.conversation_tokens,
                report.tool_output_tokens,
                report.memory_tokens,
                report.compression_tokens,
                report.pinned_tokens
            ),
        )
        .section(
            "Shape",
            format!(
                "Messages: {}\nTool results: {}\nCompression summaries: {}\nPins: {}\nSources: {}",
                report.message_count,
                report.tool_result_count,
                report.compression_summary_count,
                report.pinned.len(),
                report.sources.len()
            ),
        )
}

fn context_inspect_document(report: &ContextReport) -> RenderableDocument {
    let mut document = context_status_document(report);
    document.title = "Context inspect".to_string();
    document.push_section(
        "Pinned",
        if report.pinned.is_empty() {
            "(none)".to_string()
        } else {
            report
                .pinned
                .iter()
                .map(|pin| format!("{} · {} tokens · {}", pin.id, pin.tokens, pin.target))
                .collect::<Vec<_>>()
                .join("\n")
        },
    );
    document.push_section(
        "Compression history",
        if report.compression_history.is_empty() {
            "(none)".to_string()
        } else {
            report
                .compression_history
                .iter()
                .enumerate()
                .map(|(index, record)| {
                    format!(
                        "{}. {} → {} tokens · summary {} tokens · {}",
                        index + 1,
                        record.before_tokens,
                        record.after_tokens,
                        record.summary_tokens,
                        record.message
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        },
    );
    document.push_section(
        "Sources",
        if report.sources.is_empty() {
            "(none)".to_string()
        } else {
            report
                .sources
                .iter()
                .map(|source| {
                    format!(
                        "{} · {} tokens · {}",
                        source.kind, source.tokens, source.label
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        },
    );
    document
}

fn context_sources_document(report: &ContextReport) -> RenderableDocument {
    let mut document = RenderableDocument::new("Context sources");
    if report.sources.is_empty() {
        document.push_section("Sources", "No file, URL, search, or tool sources detected.");
    } else {
        for source in &report.sources {
            document.push_section(
                &source.kind,
                format!("{} tokens · {}", source.tokens, source.label),
            );
        }
    }
    document
}

fn context_compaction_document(
    report: &ContextReport,
    record: &CompressionRecord,
) -> RenderableDocument {
    let mut document = context_status_document(report);
    document.title = "Context compacted".to_string();
    document.push_section(
        "Compaction",
        format!(
            "{} → {} tokens\nSummary size: {} tokens\n{}",
            record.before_tokens, record.after_tokens, record.summary_tokens, record.message
        ),
    );
    document
}

async fn compact_context(ctx: &mut RuntimeContext) -> Result<CompressionRecord, String> {
    let before_tokens = ctx.context_report().total_tokens;
    let non_system = ctx
        .conversation
        .messages
        .iter()
        .filter(|message| message.role != MessageRole::System)
        .count();
    if non_system < 4 {
        return Err("not enough non-system messages to compact safely".to_string());
    }

    let target_remove = (non_system / 2).max(2);
    let mut removed = Vec::new();
    let mut kept = Vec::with_capacity(ctx.conversation.messages.len());
    for message in ctx.conversation.messages.drain(..) {
        if message.role != MessageRole::System && removed.len() < target_remove {
            removed.push(message);
        } else {
            kept.push(message);
        }
    }

    let summary = ctx.engine.summarize_messages(&removed).await;
    let summary_tokens = crate::context::estimate_tokens(&summary);
    kept.insert(
        0,
        Message::system(format!("[Conversation History Summary]\n{}", summary)),
    );
    ctx.conversation.messages = kept;
    let after_tokens = ctx.context_report().total_tokens;
    let record = CompressionRecord {
        before_tokens,
        after_tokens,
        summary_tokens,
        message: format!("Manually compacted {} messages", removed.len()),
    };
    ctx.compression_history.push(record.clone());
    Ok(record)
}

async fn start_verification(
    scope: VerificationScope,
    command: Option<String>,
    mode: AgentMode,
    events: mpsc::Sender<AgentEvent>,
) {
    let workspace = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let title = match scope {
        VerificationScope::Full => "Running verification",
        VerificationScope::Test => "Running tests",
        VerificationScope::Run => "Running command",
    };
    let activity_id = format!("verify-{}", now_ms());
    send_event(
        &events,
        AgentEvent::Activity(Activity::new(
            activity_id.clone(),
            AgentActivityKind::Testing,
            title,
            AgentActivityStatus::Running,
        )),
    )
    .await;

    tokio::spawn(async move {
        let service = VerificationService::new(workspace);
        let report = if mode.sandbox_policy() == SandboxPolicy::ReadOnly {
            let reason =
                "Plan mode is read-only; verification commands were planned but not executed.";
            service.plan_only(scope, reason).await
        } else if let Some(command) = command {
            service.run_shell(command).await
        } else {
            service.verify(scope).await
        };
        let status = if report.success() {
            AgentActivityStatus::Completed
        } else {
            AgentActivityStatus::Failed
        };
        send_event(
            &events,
            AgentEvent::Activity(Activity::new(
                activity_id,
                AgentActivityKind::Testing,
                "Verification finished",
                status,
            )),
        )
        .await;
        send_event(
            &events,
            AgentEvent::Document(verification_document(&report)),
        )
        .await;
    });
}

fn verification_document(report: &VerificationReport) -> RenderableDocument {
    let mut document = RenderableDocument::new(if report.success() {
        "Verification passed"
    } else if report.skipped_reason.is_some() {
        "Verification planned"
    } else {
        "Verification failed"
    });
    document.push_section("Project", report.project_kind.clone());
    document.push_section("Diff", report.diff_summary.clone());
    document.push_section(
        "Changed files",
        if report.changed_files.is_empty() {
            "(none)".to_string()
        } else {
            report.changed_files.join("\n")
        },
    );
    document.push_section(
        "Planned commands",
        if report.planned_commands.is_empty() {
            "No project verification commands were detected.".to_string()
        } else {
            report
                .planned_commands
                .iter()
                .map(|command| command.display())
                .collect::<Vec<_>>()
                .join("\n")
        },
    );
    if let Some(reason) = &report.skipped_reason {
        document.push_section("Skipped", reason.clone());
        return document;
    }
    if report.results.is_empty() {
        document.push_section(
            "Evidence",
            "No commands were executed; no success claim can be made.",
        );
        return document;
    }
    for result in &report.results {
        document.push_section(
            format!(
                "{} · {}",
                if result.success { "passed" } else { "failed" },
                result.label
            ),
            format!(
                "Command: {}\nExit: {}\nDuration: {}ms\nstdout:\n{}\nstderr:\n{}",
                result.command,
                result
                    .exit_code
                    .map(|code| code.to_string())
                    .unwrap_or_else(|| "unknown".to_string()),
                result.duration_ms,
                if result.stdout.trim().is_empty() {
                    "(empty)"
                } else {
                    result.stdout.trim()
                },
                if result.stderr.trim().is_empty() {
                    "(empty)"
                } else {
                    result.stderr.trim()
                }
            ),
        );
    }
    document.push_section(
        "Conclusion",
        if report.success() {
            "All executed verification commands completed successfully."
        } else {
            "One or more verification commands failed. Do not report completion as verified until the failures are addressed."
        },
    );
    document
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use async_trait::async_trait;
    use pleiades_agent_core::error::Error;
    use pleiades_agent_core::model::ModelInfo;
    use pleiades_agent_core::provider::{
        ChatRequest, ChatResponse, Provider, ProviderCapabilities,
    };
    use pleiades_agent_core::tool::{Tool, ToolContext, ToolResult};

    use crate::memory::MemoryManager;

    use super::*;

    #[test]
    fn parses_modes_and_maps_sandboxes() {
        assert_eq!(AgentMode::parse("plan"), AgentMode::Plan);
        assert_eq!(AgentMode::parse("workspace-write"), AgentMode::Agent);
        assert_eq!(AgentMode::parse("auto"), AgentMode::Auto);
        assert_eq!(AgentMode::parse("unrestricted"), AgentMode::Yolo);
        assert_eq!(AgentMode::Plan.sandbox(), "read-only");
        assert_eq!(AgentMode::Agent.sandbox(), "workspace-write");
        assert_eq!(AgentMode::Auto.sandbox(), "workspace-write");
        assert_eq!(AgentMode::Yolo.sandbox(), "danger-full-access");
        assert_eq!(AgentMode::Agent.approval_policy(), ApprovalPolicy::OnRisk);
        assert_eq!(AgentMode::Auto.approval_policy(), ApprovalPolicy::Never);
    }

    #[test]
    fn extracts_a_compact_permission_target() {
        let input = serde_json::json!({"command": "cargo test --workspace"});
        assert_eq!(permission_target(&input), "cargo test --workspace");
    }

    #[test]
    fn truncates_large_utf8_output_on_a_character_boundary() {
        let (output, truncated) = truncate_owned("✦".repeat(100), 17);
        assert!(truncated);
        assert!(output.starts_with("✦"));
        assert!(output.contains("truncated"));
    }

    #[derive(Clone, Copy)]
    enum MockBehavior {
        ToolThenFinish,
        RepeatedFailingTool,
        Slow,
    }

    struct MockProvider {
        calls: Arc<AtomicUsize>,
        behavior: MockBehavior,
    }

    #[async_trait]
    impl Provider for MockProvider {
        fn name(&self) -> &str {
            "mock"
        }

        fn display_name(&self) -> &str {
            "Mock"
        }

        fn capabilities(&self) -> ProviderCapabilities {
            ProviderCapabilities {
                streaming: true,
                tools: true,
                vision: false,
                embeddings: false,
                thinking: true,
                json_mode: false,
                function_calling: true,
            }
        }

        fn default_model(&self) -> &str {
            "mock-1"
        }

        async fn list_models(&self) -> Result<Vec<ModelInfo>, Error> {
            Ok(Vec::new())
        }

        async fn chat(&self, _request: ChatRequest) -> Result<ChatResponse, Error> {
            Err(Error::unsupported("mock only streams"))
        }

        async fn chat_stream(
            &self,
            _request: ChatRequest,
        ) -> Result<mpsc::Receiver<StreamEvent>, Error> {
            let call = self.calls.fetch_add(1, Ordering::SeqCst);
            let (sender, receiver) = mpsc::channel(8);
            match self.behavior {
                MockBehavior::ToolThenFinish if call == 0 => {
                    sender
                        .send(StreamEvent::ToolCall {
                            id: "write-1".to_string(),
                            name: "mock-write".to_string(),
                            input: serde_json::json!({"path": "src/lib.rs"}),
                        })
                        .await
                        .unwrap();
                    sender
                        .send(StreamEvent::Done {
                            finish_reason: "tool_use".to_string(),
                            usage: None,
                        })
                        .await
                        .unwrap();
                }
                MockBehavior::ToolThenFinish => {
                    sender
                        .send(StreamEvent::Token("Completed with evidence.".into()))
                        .await
                        .unwrap();
                    sender
                        .send(StreamEvent::Done {
                            finish_reason: "stop".to_string(),
                            usage: None,
                        })
                        .await
                        .unwrap();
                }
                MockBehavior::RepeatedFailingTool => {
                    sender
                        .send(StreamEvent::ToolCall {
                            id: format!("fail-{call}"),
                            name: "mock-fail".to_string(),
                            input: serde_json::json!({"path": "src/lib.rs"}),
                        })
                        .await
                        .unwrap();
                    sender
                        .send(StreamEvent::Done {
                            finish_reason: "tool_use".to_string(),
                            usage: None,
                        })
                        .await
                        .unwrap();
                }
                MockBehavior::Slow => {
                    tokio::spawn(async move {
                        tokio::time::sleep(Duration::from_secs(30)).await;
                        let _ = sender.send(StreamEvent::Token("too late".into())).await;
                    });
                }
            }
            Ok(receiver)
        }
    }

    struct MockWriteTool(Arc<AtomicUsize>);
    struct MockFailTool(Arc<AtomicUsize>);

    #[async_trait]
    impl Tool for MockWriteTool {
        fn name(&self) -> &str {
            "mock-write"
        }
        fn description(&self) -> &str {
            "Modify a mock workspace file"
        }
        fn input_schema(&self) -> serde_json::Value {
            serde_json::json!({"type": "object"})
        }
        fn is_readonly(&self) -> bool {
            false
        }
        fn is_concurrency_safe(&self) -> bool {
            false
        }
        fn permission_level(&self) -> PermissionLevel {
            PermissionLevel::WorkspaceWrite
        }

        async fn execute(
            &self,
            _input: serde_json::Value,
            _ctx: &ToolContext,
        ) -> Result<ToolResult, Error> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Ok(ToolResult {
                success: true,
                content: "mock write completed".to_string(),
                error: None,
                metadata: None,
            })
        }
    }

    #[async_trait]
    impl Tool for MockFailTool {
        fn name(&self) -> &str {
            "mock-fail"
        }
        fn description(&self) -> &str {
            "Fail with the same error"
        }
        fn input_schema(&self) -> serde_json::Value {
            serde_json::json!({"type": "object"})
        }
        fn is_readonly(&self) -> bool {
            false
        }
        fn is_concurrency_safe(&self) -> bool {
            false
        }
        fn permission_level(&self) -> PermissionLevel {
            PermissionLevel::WorkspaceWrite
        }

        async fn execute(
            &self,
            _input: serde_json::Value,
            _ctx: &ToolContext,
        ) -> Result<ToolResult, Error> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Ok(ToolResult {
                success: false,
                content: String::new(),
                error: Some("identical failure".to_string()),
                metadata: None,
            })
        }
    }

    fn runtime_with_mock(
        mode: AgentMode,
        behavior: MockBehavior,
        executions: Arc<AtomicUsize>,
    ) -> (tempfile::TempDir, AgentRuntime) {
        runtime_with_mock_config(mode, behavior, executions, |_| {})
    }

    fn runtime_with_mock_config(
        mode: AgentMode,
        behavior: MockBehavior,
        executions: Arc<AtomicUsize>,
        configure: impl FnOnce(&mut Config),
    ) -> (tempfile::TempDir, AgentRuntime) {
        let temp = tempfile::tempdir().unwrap();
        let mut config = Config::default();
        config.core.default_provider = Some("mock".into());
        config.core.default_model = Some("mock-1".into());
        config.session.history_dir = Some(temp.path().join("sessions").display().to_string());
        config.permissions.ask_always = true;
        configure(&mut config);
        let mut engine = Engine::with_memory(config.clone(), MemoryManager::new());
        engine.register_provider(Box::new(MockProvider {
            calls: Arc::new(AtomicUsize::new(0)),
            behavior,
        }));
        engine.register_tool(Box::new(MockWriteTool(executions.clone())));
        engine.register_tool(Box::new(MockFailTool(executions)));
        let runtime = AgentRuntime::new(
            config,
            Conversation::new("test-session"),
            "mock",
            "mock-1",
            mode,
        )
        .with_engine(engine);
        (temp, runtime)
    }

    async fn next_event(events: &mut mpsc::Receiver<AgentEvent>) -> AgentEvent {
        tokio::time::timeout(Duration::from_secs(2), events.recv())
            .await
            .expect("runtime event timeout")
            .expect("runtime event channel closed")
    }

    #[tokio::test]
    async fn slash_commands_flow_through_typed_runtime_events() {
        let executions = Arc::new(AtomicUsize::new(0));
        let (_temp, runtime) =
            runtime_with_mock(AgentMode::Agent, MockBehavior::ToolThenFinish, executions);
        let mut handle = runtime.spawn();
        handle
            .commands
            .send(AgentCommand::DispatchSlash("/model mock-2".into()))
            .await
            .unwrap();

        loop {
            if matches!(
                next_event(&mut handle.events).await,
                AgentEvent::ModelChanged(ref model) if model == "mock-2"
            ) {
                break;
            }
        }

        handle
            .commands
            .send(AgentCommand::DispatchSlash("/status".into()))
            .await
            .unwrap();
        let document = loop {
            if let AgentEvent::Document(document) = next_event(&mut handle.events).await {
                break document;
            }
        };
        assert!(
            document
                .sections
                .iter()
                .any(|section| section.heading == "Model" && section.body == "mock-2")
        );

        handle.commands.send(AgentCommand::Shutdown).await.unwrap();
    }

    #[tokio::test]
    async fn reload_extensions_emits_typed_reload_event() {
        let executions = Arc::new(AtomicUsize::new(0));
        let (_temp, runtime) =
            runtime_with_mock(AgentMode::Agent, MockBehavior::ToolThenFinish, executions);
        let mut handle = runtime.spawn();
        handle
            .commands
            .send(AgentCommand::DispatchSlash("/skills reload".into()))
            .await
            .unwrap();

        let mut saw_reload = false;
        let mut saw_notify = false;
        for _ in 0..5 {
            match next_event(&mut handle.events).await {
                AgentEvent::ExtensionsReloaded => saw_reload = true,
                AgentEvent::Notify(Notification {
                    level: NotificationLevel::Success,
                    message,
                }) if message == "Extensions reloaded" => saw_notify = true,
                _ => {}
            }
            if saw_reload && saw_notify {
                break;
            }
        }

        assert!(saw_reload);
        assert!(saw_notify);
        handle.commands.send(AgentCommand::Shutdown).await.unwrap();
    }

    #[tokio::test]
    async fn checkpoint_commands_create_list_and_preview_restore() {
        let executions = Arc::new(AtomicUsize::new(0));
        let (temp, runtime) =
            runtime_with_mock(AgentMode::Agent, MockBehavior::ToolThenFinish, executions);
        let mut handle = runtime.spawn();
        handle
            .commands
            .send(AgentCommand::DispatchSlash(
                "/checkpoint create before edit".into(),
            ))
            .await
            .unwrap();
        loop {
            if matches!(
                next_event(&mut handle.events).await,
                AgentEvent::Notify(Notification {
                    level: NotificationLevel::Success,
                    ..
                })
            ) {
                break;
            }
        }

        handle
            .commands
            .send(AgentCommand::DispatchSlash("/checkpoint list".into()))
            .await
            .unwrap();
        let document = loop {
            if let AgentEvent::Document(document) = next_event(&mut handle.events).await {
                break document;
            }
        };
        assert!(
            document
                .sections
                .iter()
                .any(|section| section.heading == "before edit")
        );

        let checkpoint_dir = temp.path().join("sessions/checkpoints");
        let checkpoint_id = std::fs::read_dir(checkpoint_dir)
            .unwrap()
            .flatten()
            .find_map(|entry| {
                entry
                    .path()
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .map(ToString::to_string)
            })
            .unwrap();
        handle
            .commands
            .send(AgentCommand::DispatchSlash(format!(
                "/checkpoint restore {checkpoint_id}"
            )))
            .await
            .unwrap();
        let preview = loop {
            if let AgentEvent::Document(document) = next_event(&mut handle.events).await {
                break document;
            }
        };
        assert_eq!(preview.title, "Checkpoint restore preview");

        handle.commands.send(AgentCommand::Shutdown).await.unwrap();
    }

    #[tokio::test]
    async fn context_commands_report_sources_and_pins() {
        let executions = Arc::new(AtomicUsize::new(0));
        let (_temp, mut runtime) =
            runtime_with_mock(AgentMode::Agent, MockBehavior::ToolThenFinish, executions);
        runtime
            .conversation
            .add_message(Message::user("inspect context"));
        runtime.conversation.add_message(Message {
            role: MessageRole::Assistant,
            content: vec![ContentBlock::ToolUse {
                id: "read-1".to_string(),
                name: "read".to_string(),
                input: serde_json::json!({"path": "src/lib.rs"}),
            }],
            reasoning: None,
            metadata: None,
        });
        let mut handle = runtime.spawn();

        handle
            .commands
            .send(AgentCommand::DispatchSlash(
                "/context pin src/main.rs".into(),
            ))
            .await
            .unwrap();
        let status = loop {
            if let AgentEvent::Document(document) = next_event(&mut handle.events).await {
                if document.title == "Context status" {
                    break document;
                }
            }
        };
        assert!(
            status
                .sections
                .iter()
                .any(|section| section.heading == "Shape" && section.body.contains("Pins: 1"))
        );

        handle
            .commands
            .send(AgentCommand::DispatchSlash("/context sources".into()))
            .await
            .unwrap();
        let sources = loop {
            if let AgentEvent::Document(document) = next_event(&mut handle.events).await {
                if document.title == "Context sources" {
                    break document;
                }
            }
        };
        assert!(
            sources
                .sections
                .iter()
                .any(|section| section.body.contains("src/lib.rs"))
        );

        handle.commands.send(AgentCommand::Shutdown).await.unwrap();
    }

    #[tokio::test]
    async fn context_compact_records_history() {
        let executions = Arc::new(AtomicUsize::new(0));
        let (_temp, mut runtime) =
            runtime_with_mock(AgentMode::Agent, MockBehavior::ToolThenFinish, executions);
        for index in 0..6 {
            runtime
                .conversation
                .add_message(Message::user(format!("message {index} with enough words")));
        }
        let mut handle = runtime.spawn();

        handle
            .commands
            .send(AgentCommand::DispatchSlash("/context compact".into()))
            .await
            .unwrap();
        let compacted = loop {
            if let AgentEvent::Document(document) = next_event(&mut handle.events).await {
                if document.title == "Context compacted" {
                    break document;
                }
            }
        };
        assert!(
            compacted
                .sections
                .iter()
                .any(|section| section.heading == "Compaction")
        );

        handle
            .commands
            .send(AgentCommand::DispatchSlash("/context inspect".into()))
            .await
            .unwrap();
        let inspect = loop {
            if let AgentEvent::Document(document) = next_event(&mut handle.events).await {
                if document.title == "Context inspect" {
                    break document;
                }
            }
        };
        assert!(inspect.sections.iter().any(|section| {
            section.heading == "Compression history" && section.body.contains("Manually compacted")
        }));

        handle.commands.send(AgentCommand::Shutdown).await.unwrap();
    }

    #[tokio::test]
    async fn run_command_records_verification_evidence() {
        let executions = Arc::new(AtomicUsize::new(0));
        let (_temp, runtime) =
            runtime_with_mock(AgentMode::Agent, MockBehavior::ToolThenFinish, executions);
        let mut handle = runtime.spawn();
        handle
            .commands
            .send(AgentCommand::DispatchSlash("/run rustc --version".into()))
            .await
            .unwrap();
        let document = loop {
            if let AgentEvent::Document(document) = next_event(&mut handle.events).await {
                if document.title == "Verification passed"
                    || document.title == "Verification failed"
                {
                    break document;
                }
            }
        };
        assert!(
            document
                .sections
                .iter()
                .any(|section| section.heading.contains("passed")
                    && section.body.contains("rustc --version"))
        );
        handle.commands.send(AgentCommand::Shutdown).await.unwrap();
    }

    #[tokio::test]
    async fn repeated_identical_tool_failure_halts_the_task() {
        let executions = Arc::new(AtomicUsize::new(0));
        let (_temp, runtime) = runtime_with_mock_config(
            AgentMode::Auto,
            MockBehavior::RepeatedFailingTool,
            executions.clone(),
            |config| {
                config.agent.max_repeats = 3;
                config.agent.max_tool_iterations = 10;
            },
        );
        let mut handle = runtime.spawn();
        handle
            .commands
            .send(AgentCommand::Submit("trigger repeated failure".into()))
            .await
            .unwrap();
        let message = loop {
            if let AgentEvent::TaskFailed { message } = next_event(&mut handle.events).await {
                break message;
            }
        };
        assert!(message.contains("repeated 3 times"));
        assert_eq!(executions.load(Ordering::SeqCst), 3);
        handle.commands.send(AgentCommand::Shutdown).await.unwrap();
    }

    #[tokio::test]
    async fn plan_mode_verification_reports_planned_commands_without_execution() {
        let executions = Arc::new(AtomicUsize::new(0));
        let (_temp, runtime) =
            runtime_with_mock(AgentMode::Plan, MockBehavior::ToolThenFinish, executions);
        let mut handle = runtime.spawn();
        handle
            .commands
            .send(AgentCommand::DispatchSlash("/verify".into()))
            .await
            .unwrap();
        let document = loop {
            if let AgentEvent::Document(document) = next_event(&mut handle.events).await {
                if document.title == "Verification planned" {
                    break document;
                }
            }
        };
        assert!(
            document
                .sections
                .iter()
                .any(|section| section.heading == "Skipped" && section.body.contains("Plan mode"))
        );
        handle.commands.send(AgentCommand::Shutdown).await.unwrap();
    }

    #[tokio::test]
    async fn waits_for_permission_before_executing_a_write() {
        let executions = Arc::new(AtomicUsize::new(0));
        let (_temp, runtime) = runtime_with_mock(
            AgentMode::Agent,
            MockBehavior::ToolThenFinish,
            executions.clone(),
        );
        let mut handle = runtime.spawn();
        handle
            .commands
            .send(AgentCommand::Submit("make a change".into()))
            .await
            .unwrap();

        let request = loop {
            if let AgentEvent::PermissionRequested(request) = next_event(&mut handle.events).await {
                break request;
            }
        };
        assert_eq!(executions.load(Ordering::SeqCst), 0);
        handle
            .commands
            .send(AgentCommand::Permission {
                request_id: request.id,
                decision: PermissionDecision::AllowOnce,
            })
            .await
            .unwrap();

        loop {
            if matches!(
                next_event(&mut handle.events).await,
                AgentEvent::TaskCompleted { .. }
            ) {
                break;
            }
        }
        assert_eq!(executions.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn plan_mode_denies_writes_without_opening_a_prompt() {
        let executions = Arc::new(AtomicUsize::new(0));
        let (_temp, runtime) = runtime_with_mock(
            AgentMode::Plan,
            MockBehavior::ToolThenFinish,
            executions.clone(),
        );
        let mut handle = runtime.spawn();
        handle
            .commands
            .send(AgentCommand::Submit("inspect only".into()))
            .await
            .unwrap();
        let mut prompted = false;
        loop {
            match next_event(&mut handle.events).await {
                AgentEvent::PermissionRequested(_) => prompted = true,
                AgentEvent::TaskCompleted { .. } => break,
                _ => {}
            }
        }
        assert!(!prompted);
        assert_eq!(executions.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn auto_mode_executes_workspace_writes_without_prompting() {
        let executions = Arc::new(AtomicUsize::new(0));
        let (_temp, runtime) = runtime_with_mock(
            AgentMode::Auto,
            MockBehavior::ToolThenFinish,
            executions.clone(),
        );
        let mut handle = runtime.spawn();
        handle
            .commands
            .send(AgentCommand::Submit("make a workspace change".into()))
            .await
            .unwrap();
        let mut prompted = false;
        loop {
            match next_event(&mut handle.events).await {
                AgentEvent::PermissionRequested(_) => prompted = true,
                AgentEvent::TaskCompleted { .. } => break,
                _ => {}
            }
        }
        assert!(!prompted);
        assert_eq!(executions.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn auto_mode_honors_explicit_structured_deny_rules() {
        let executions = Arc::new(AtomicUsize::new(0));
        let (_temp, runtime) = runtime_with_mock_config(
            AgentMode::Auto,
            MockBehavior::ToolThenFinish,
            executions.clone(),
            |config| {
                config
                    .permissions
                    .rules
                    .push(pleiades_agent_permissions::PermissionRule {
                        tool: "mock-write".to_string(),
                        pattern: "src/*".to_string(),
                        action: pleiades_agent_permissions::PermissionAction::Deny,
                        cwd: None,
                        network: None,
                        mcp_server: None,
                        mcp_tool: None,
                    });
            },
        );
        let mut handle = runtime.spawn();
        handle
            .commands
            .send(AgentCommand::Submit("make a workspace change".into()))
            .await
            .unwrap();
        let mut prompted = false;
        loop {
            match next_event(&mut handle.events).await {
                AgentEvent::PermissionRequested(_) => prompted = true,
                AgentEvent::TaskCompleted { .. } => break,
                _ => {}
            }
        }
        assert!(!prompted);
        assert_eq!(executions.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn agent_mode_honors_explicit_structured_allow_rules() {
        let executions = Arc::new(AtomicUsize::new(0));
        let (_temp, runtime) = runtime_with_mock_config(
            AgentMode::Agent,
            MockBehavior::ToolThenFinish,
            executions.clone(),
            |config| {
                config
                    .permissions
                    .rules
                    .push(pleiades_agent_permissions::PermissionRule {
                        tool: "mock-write".to_string(),
                        pattern: "src/*".to_string(),
                        action: pleiades_agent_permissions::PermissionAction::Allow,
                        cwd: None,
                        network: None,
                        mcp_server: None,
                        mcp_tool: None,
                    });
            },
        );
        let mut handle = runtime.spawn();
        handle
            .commands
            .send(AgentCommand::Submit("make a workspace change".into()))
            .await
            .unwrap();
        let mut prompted = false;
        loop {
            match next_event(&mut handle.events).await {
                AgentEvent::PermissionRequested(_) => prompted = true,
                AgentEvent::TaskCompleted { .. } => break,
                _ => {}
            }
        }
        assert!(!prompted);
        assert_eq!(executions.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn cancellation_interrupts_a_streaming_provider() {
        let executions = Arc::new(AtomicUsize::new(0));
        let (_temp, runtime) = runtime_with_mock(AgentMode::Agent, MockBehavior::Slow, executions);
        let mut handle = runtime.spawn();
        handle
            .commands
            .send(AgentCommand::Submit("wait forever".into()))
            .await
            .unwrap();
        loop {
            if matches!(
                next_event(&mut handle.events).await,
                AgentEvent::TaskStarted { .. }
            ) {
                break;
            }
        }
        handle.commands.send(AgentCommand::Cancel).await.unwrap();
        loop {
            if matches!(
                next_event(&mut handle.events).await,
                AgentEvent::TaskCancelled
            ) {
                break;
            }
        }
    }

    #[tokio::test]
    async fn changing_mode_cancels_work_running_under_the_old_boundary() {
        let executions = Arc::new(AtomicUsize::new(0));
        let (_temp, runtime) = runtime_with_mock(AgentMode::Agent, MockBehavior::Slow, executions);
        let mut handle = runtime.spawn();
        handle
            .commands
            .send(AgentCommand::Submit("active task".into()))
            .await
            .unwrap();
        loop {
            if matches!(
                next_event(&mut handle.events).await,
                AgentEvent::TaskStarted { .. }
            ) {
                break;
            }
        }
        handle
            .commands
            .send(AgentCommand::SetMode(AgentMode::Plan))
            .await
            .unwrap();
        let mut changed = false;
        let mut cancelled = false;
        while !(changed && cancelled) {
            match next_event(&mut handle.events).await {
                AgentEvent::ModeChanged(AgentMode::Plan) => changed = true,
                AgentEvent::TaskCancelled => cancelled = true,
                _ => {}
            }
        }
    }

    #[tokio::test]
    async fn follow_up_messages_are_queued_and_run_automatically() {
        let executions = Arc::new(AtomicUsize::new(0));
        let (_temp, runtime) =
            runtime_with_mock(AgentMode::Agent, MockBehavior::ToolThenFinish, executions);
        let mut handle = runtime.spawn();
        handle
            .commands
            .send(AgentCommand::Submit("first".into()))
            .await
            .unwrap();
        loop {
            if matches!(
                next_event(&mut handle.events).await,
                AgentEvent::TaskStarted { .. }
            ) {
                break;
            }
        }
        handle
            .commands
            .send(AgentCommand::Submit("follow up".into()))
            .await
            .unwrap();

        let mut completed = 0;
        let mut saw_queued = false;
        while completed < 2 {
            match next_event(&mut handle.events).await {
                AgentEvent::QueueChanged(1) => saw_queued = true,
                AgentEvent::PermissionRequested(request) => {
                    handle
                        .commands
                        .send(AgentCommand::Permission {
                            request_id: request.id,
                            decision: PermissionDecision::AllowOnce,
                        })
                        .await
                        .unwrap();
                }
                AgentEvent::TaskCompleted { .. } => completed += 1,
                _ => {}
            }
        }
        assert!(saw_queued);
    }
}
