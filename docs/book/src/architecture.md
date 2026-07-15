# Architecture

Pleiades is a Rust 2024 Cargo workspace organized around stable provider, tool, conversation, event, and error types. Tokio owns asynchronous execution; Ratatui and Crossterm own the live terminal frontend.

## Runtime boundary

The full-screen TUI never constructs providers, executes tools, mutates the domain conversation, or writes streamed output to stdout. It sends `AgentCommand` values over a bounded Tokio channel and reduces `AgentEvent` values into `AppState`.

```text
Crossterm events ─────┐
AgentEvent channel ───┼─> AppState reducer ─> Ratatui render
render ticks ─────────┤
background results ───┘

UI effects ─────────────> AgentCommand channel ─> AgentRuntime
```

Slash commands and command-palette actions resolve through
`pleiades-agent-commands`. Its handlers receive an immutable `CommandContext`
and return `CommandResult`; they do not access Ratatui, providers, tools, or
stdout. The runtime applies typed `AppEffect` values and emits typed overlay,
notification, document, and shutdown events back to the reducer. Searchable
help and nested autocomplete query the same registry, eliminating parallel
command lists and numeric palette dispatch.

The terminal loop uses `tokio::select!` across Crossterm's `EventStream`, the agent event receiver, background file discovery, and a 50 ms render interval. Rendering is immediate-mode: every frame is derived from state.

## Agent runtime

`AgentRuntime` owns the configured `Engine`, `Conversation`, `SessionStore`, checkpoint store, context pins, compression history, permission-session decisions, task queue, and cancellation token. A submitted task runs in a Tokio task while the runtime actor continues accepting cancellation, follow-ups, permission decisions, mode changes, provider/model changes, session operations, checkpoint operations, context inspection/compaction, and shutdown.

Provider streams normalize text, reasoning summaries, tool calls, provider-managed activity, tool output, usage, completion, and errors. The runtime adds typed lifecycle events for planning, permission waits, execution, validation, diff review, completion, failure, and cancellation.

## Crate responsibilities

- `pleiades-agent-core` — provider/tool traits and normalized domain types.
- `pleiades-agent-config` — layered configuration and environment interpolation.
- `pleiades-agent-commands` — command specifications, parsing, discovery,
  autocomplete, help generation, and typed command results.
- `pleiades-agent-services` — terminal-independent provider, model, plugin,
  permission, configuration, and extension operations shared by CLI and live commands.
- `pleiades-agent-engine` — chat preparation, event-driven autonomous runtime, permissions, cancellation, sessions, and memory.
- `pleiades-agent-permissions` — structured permission rules, shell command
  clause parsing, path escape checks, and deny-first decisions.
- `pleiades-agent-tui` — reducer state, Ratatui widgets, native Markdown spans, terminal lifecycle, editor, themes, and overlays.
- `pleiades-agent-providers` — Anthropic, OpenAI, OpenAI-compatible, and Codex CLI adapters.
- `pleiades-agent-tools` — confined filesystem/search/diff tools and sandboxed command execution.
- remaining crates — plugins, prompts, memory, workflows, Git automation, SDK, and CLI packaging.

## Safety layers

Plan mode denies mutating tools. Agent mode confines filesystem paths to the canonical workspace and guards sensitive operations through permission events. Auto mode keeps the workspace boundary but skips prompts unless a rule says to ask. YOLO removes that process boundary only after an explicit user choice. Shell writes use Bubblewrap on Linux or `sandbox-exec` on macOS when Agent-mode process isolation is required, refusing execution when that boundary cannot be provided.

Structured permission rules are evaluated before mode defaults. Deny rules win
over ask and allow, and every parsed shell clause in a compound command must be
covered before a command is automatically allowed. Agent and Auto canonicalize
working directories, redirection targets, explicit tool paths, and symlinks
against the workspace boundary.

The ChatGPT subscription adapter delegates execution to the official Codex CLI and maps modes to Codex sandboxes. API-provider tool calls execute in Pleiades and use Pleiades permission modals.

## Reliability

Bounded channels, output caps, a moving transcript window, and background repository file discovery keep redraw work predictable. A terminal guard restores raw mode, the alternate screen, cursor, mouse capture, and bracketed paste on normal and panic exits. Mock-provider tests cover permission waits, Plan denial, queued follow-ups, cancellation, and mode changes; Ratatui snapshots and resize tests cover the frontend.

See the repository's [full architecture document](https://github.com/CodWasTaken/Pleiades/blob/master/docs/ARCHITECTURE.md) for the detailed design.
