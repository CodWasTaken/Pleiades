# Pleiades Architecture

Pleiades is a native Rust 2024 autonomous coding agent. Its architecture separates terminal rendering, agent orchestration, provider protocols, tools, and domain types so that no provider or frontend owns the whole product.

## System map

```text
┌──────────────────────── Full-screen terminal ─────────────────────────┐
│ Crossterm EventStream -> reducer -> AppState -> Ratatui widgets       │
│ composer · conversation · activity · overlays · status                │
└──────────────────────────────┬─────────────────────────────────────────┘
                               │ bounded AgentCommand / AgentEvent channels
┌──────────────────────────────▼─────────────────────────────────────────┐
│ AgentRuntime actor                                                     │
│ task queue · cancellation · permissions · sessions · normalized events│
└──────────────┬───────────────────────────────┬─────────────────────────┘
               │                               │
┌──────────────▼──────────────┐  ┌────────────▼─────────────────────────┐
│ Provider ports             │  │ Tool ports                           │
│ OpenAI · Anthropic         │  │ read · write · edit · bash           │
│ compatible endpoints      │  │ glob · grep · diff · search · fetch  │
│ official Codex delegation │  │ canonical workspace + process sandbox│
└──────────────┬──────────────┘  └────────────┬─────────────────────────┘
               └──────────────────┬────────────┘
                                  ▼
                   pleiades-agent-core domain types
```

## Workspace crates

| Crate | Responsibility |
|---|---|
| `pleiades-agent` | Clap command tree, setup/auth/doctor, one-shot commands, binary packaging |
| `pleiades-agent-core` | Provider and tool traits, conversations, models, normalized events and errors |
| `pleiades-agent-config` | Defaults, global/project files, profiles, environment interpolation, validation |
| `pleiades-agent-engine` | Provider/tool orchestration, autonomous runtime, permissions, cancellation, queue, memory, sessions, checkpoints, context accounting, verification evidence |
| `pleiades-agent-tui` | Ratatui app, reducer, native Markdown, textarea composer, overlays, terminal lifecycle, design tokens |
| `pleiades-agent-providers` | OpenAI, Anthropic, OpenAI-compatible, and official Codex CLI adapters |
| `pleiades-agent-tools` | Nine built-in tools, workspace path confinement, process isolation |
| `pleiades-agent-mcp` | MCP JSON-RPC protocol types, stdio client primitives, server definitions, redacted status reports |
| `pleiades-agent-plugins` | Manifest lifecycle and pre/post shell hooks |
| `pleiades-agent-memory` | Session, project, and user memory stores |
| `pleiades-agent-prompts` | Templates and professional coding-agent protocol |
| `pleiades-agent-workflow` | Reusable sequential, parallel, conditional, retryable workflows |
| `pleiades-agent-git` | Commit, review, summary, and diff generation |
| `pleiades-agent-sdk` | Stable re-exports for integrations |

Dependencies point inward toward core domain traits. The terminal crate depends on the engine command/event interface, not provider implementations or direct tools.

## Live application loop

`TuiApp::run` owns a panic-safe terminal guard and continuously selects independent event sources:

```rust,ignore
loop {
    tokio::select! {
        Some(event) = terminal_events.next() => reduce_terminal(event),
        Some(event) = agent_events.recv() => app.apply_agent(event),
        Some(files) = background_files.recv() => app.file_options = files,
        _ = render_tick.tick() => {}
    }

    terminal.draw(|frame| ui::render(frame, &mut app))?;
}
```

The real implementation handles input errors, shutdown effects, paste, mouse, and resize events. The important invariant is that model streams and tools never occupy the terminal task. The TUI contains no provider construction, tool execution, or direct stream printing.

### State and effects

Terminal actions update `AppState` and may return an `Effect::Command(AgentCommand)`. Agent events update transcript, activity, permissions, usage, Git state, diff, queue count, task timing, or selected provider/model. Every visible operation can therefore be recorded, collapsed, filtered, tested, and redrawn.

The application keeps five persistent regions and renders permission, palette, provider/model, file, session, help, configuration, tool detail/output, diff, and diagnostics overlays above them.

## Autonomous runtime

`AgentRuntime` is a long-lived actor owning the domain conversation and configured `Engine`. It accepts:

- task submissions and queued follow-ups;
- cancellation;
- four-way permission decisions;
- Plan, Agent, Auto, and YOLO mode changes;
- provider/model changes;
- session load/save/clear;
- shutdown.

Each active task owns a `CancellationToken` and a permission-response channel. Provider streaming and tool waits are wrapped in `tokio::select!` with that token. A mode change cancels work running under the prior security boundary before rebuilding the configured engine.

The runtime also owns checkpoint and context-management state. `/context`
commands flow through the shared command registry as typed effects; the runtime
accounts for approximate token usage by conversation, tool output, memory,
compression summaries, pins, and detected file/tool sources, then emits
structured documents for the frontend to render.

Verification commands are also typed effects. `/verify`, `/test`, and `/run`
launch background runtime tasks that detect project tooling, inspect Git diff
state, execute bounded commands, and return structured evidence. Plan mode
reports planned commands without executing them.

Autonomous tasks include a doom-loop detector keyed by repeated failure
signals. The first integrated signal is identical tool failure; when it reaches
`agent.max_repeats`, the runtime emits `TaskFailed` with the repeated-failure
reason instead of continuing until the global iteration limit.

### Typed activity

Provider and runtime activity use `AgentActivityKind` and `AgentActivityStatus`, not display strings. Kinds include inspecting, searching, reading, planning, editing, writing, executing, testing, reviewing, and tool. Statuses include queued, running, waiting for approval, completed, failed, and cancelled.

The Codex adapter maps official JSON events to these types. API providers return tool calls which the runtime wraps with the same lifecycle.

## Provider boundary

The `Provider` trait normalizes complete chat, streaming chat, model discovery, capabilities, usage, errors, and optional embeddings. A frontend never branches on provider JSON.

There are two autonomous execution paths:

1. **Pleiades tool runtime** — OpenAI API, Anthropic, and compatible providers emit tool calls. Pleiades applies permissions, executes tools, captures results, and continues the loop.
2. **Official Codex delegation** — `openai-subscription` calls the installed Codex CLI. Authentication remains in Codex's credential store. Pleiades receives normalized activity/text/usage events and maps its mode to Codex's sandbox.

Because Codex owns individual calls in path 2, Pleiades permission modals cannot interpose on each delegated command. The selected Codex sandbox is the enforcement boundary.

## Tool and permission boundary

Tools declare `ReadOnly`, `WorkspaceWrite`, or `Dangerous`. The runtime combines that level with:

- active mode;
- durable `always_allow` / `always_deny` configuration;
- allow/deny decisions stored for the current session;
- a structured modal request for unresolved sensitive calls.

The modal includes operation, target, reason, risk, and the four once/session decisions. Plan mode rejects mutation before a prompt. No active full-screen session reads approval from stdin.

### Filesystem confinement

Filesystem tools resolve relative paths against the canonical workspace. Existing targets are canonicalized to detect symlink escapes. Missing write targets resolve their nearest existing ancestor so a symlinked parent cannot escape. Absolute paths and parent traversal outside the workspace are rejected.

### Command isolation

Agent-mode API-provider commands execute with workspace-write process isolation:

- Linux: Bubblewrap, read-only root plus a writable workspace bind;
- macOS: `sandbox-exec`, denying writes outside the workspace and temporary directory;
- unsupported platform/isolation: refuse the call.

Auto mode keeps the workspace boundary but skips approval prompts unless a rule asks. YOLO uses the ordinary shell only after explicit selection. Child processes are killed when the future is dropped or the configured timeout expires.

## Rendering and terminal lifecycle

The `seven-sisters` design system contains typed colors, state styles, borders, diff colors, and Unicode/ASCII symbols. Native Markdown rendering produces Ratatui `Line` and `Span` values rather than ANSI strings, enabling wrapping and safe redraws.

Transcript messages, activities, tool output, and diffs are bounded at UTF-8 boundaries. Only a moving conversation window is converted into widgets each frame. Workspace file discovery runs off the terminal task.

`TerminalGuard` enables raw mode, alternate screen, mouse capture, and bracketed paste. `Drop` and the installed panic hook restore every capability and cursor visibility. Ctrl+C cancels active work; Ctrl+Q performs runtime shutdown before restoration.

## Persistence and evidence

Sessions persist domain messages, not rendered terminal text. The runtime saves after tool iterations, completion, cancellation, explicit save, and shutdown. Git status is refreshed around tasks; the current diff is captured for the review overlay when a task completes.

The professional system prompt requires repository inspection, a focused plan for substantial work, real validation, final diff review, and an evidence-based completion report. It explicitly prohibits claiming an unobserved check passed.

## Verification strategy

- deterministic mock-provider/tool tests for approvals, Plan denial, cancellation, mode changes, and queued follow-ups;
- path traversal and symlink-boundary tests;
- Ratatui shell snapshots and resize tests;
- huge UTF-8 stream bounds;
- CLI integration flows and help snapshots;
- Linux, macOS, and Windows build/test matrix;
- clippy, rustfmt, rustdoc, coverage, audit, typos, benchmarks, and mdBook deployment.
