# Pleiades product requirements

This specification describes the current canonical product. The [implementation matrix](FEATURE_MATRIX.md) is the source of truth for shipped versus future capabilities.

## Product

Pleiades is a professional, provider-agnostic autonomous coding agent written in Rust. Running `pleiades` in a project must open a responsive full-screen workspace where a developer can submit a task, observe investigation and changes, approve sensitive actions, cancel or redirect work, inspect evidence, and resume the session later.

The default experience must not be a blocking REPL. The compatibility `pleiades repl` command may remain line-oriented, but the default and `pleiades chat` paths use Ratatui.

## Live terminal

The active application must:

- select terminal input, agent events, background results, and render ticks concurrently;
- keep input, scroll, resize, cancellation, overlays, and follow-up queuing responsive during long work;
- render a header, conversation, activity timeline, multiline composer, and status bar;
- render Markdown and syntax-highlighted code as Ratatui text primitives rather than streamed ANSI;
- provide keyboard-driven permission, command, provider/model, file/session, help, configuration, tool, output, diff, and diagnostics overlays;
- restore terminal raw mode, alternate screen, cursor, mouse, and paste state on exit, error, or panic;
- degrade through high-contrast and ASCII themes without modifying terminal-emulator configuration.

## Autonomous behavior

For substantial development work the agent must inspect relevant repository files and conventions, form a focused plan, make coherent changes, run relevant checks, diagnose practical failures, review the final diff, and produce an evidence-based report. It must never report a check as passed unless it ran and observed the result.

The runtime must emit typed lifecycle activity and keep raw output available without flooding the main conversation. Tasks must be cancellable. Follow-ups submitted during work must queue and run in order.

## Providers

Stable traits must normalize text streaming, reasoning summaries when exposed, tool calls, activity, output, usage, finish reasons, capabilities, context information, errors, and rate limits.

Required adapters are OpenAI Platform API, Anthropic, configurable OpenAI-compatible endpoints, and ChatGPT subscription delegation through the official Codex CLI. Local models are supported through compatible endpoints. Provider credentials must not be copied between adapters; Codex remains the owner of subscription credentials.

## Tools and safety

Built-in tools are read, write, edit, bash, glob, grep, diff, web search, and fetch. Tools declare read-only, workspace-write, or dangerous permission levels.

The runtime provides:

- Plan mode: inspection without mutating tools;
- Agent mode: canonical workspace path confinement and guarded commands;
- Auto mode: workspace-confined execution without approval prompts;
- YOLO mode: ordinary process access only after explicit selection;
- allow once, allow session, deny once, and deny session modal decisions;
- persistent allow/ask/deny configuration rules;
- traversal and symlink-escape protection;
- Agent-mode process write isolation where the platform implementation exists, and refusal otherwise;
- cancellation-aware timeouts and bounded UTF-8 output.

Pleiades permission modals apply to tools Pleiades executes. When execution is delegated to the official Codex CLI, the selected Codex sandbox is the boundary for its internal calls.

## Persistence and extensions

Conversations support JSON persistence, resume, Markdown/JSON export, context management, and memory-summary injection. File-backed session, project, and user memory tiers are required. Embedding/vector semantic search is future work and must not be documented as shipped.

Local plugin manifests and trusted pre/post shell hooks are supported. A WASM runtime, plugin marketplace, and automatic dependency resolution are future work and must not be represented as security boundaries.

Reusable workflows support sequencing, parallel batches, simple conditions, retries, timeouts, variables, validation, persistence, and CLI management. Git assistance supports commit messages, reviews, summaries, and diff explanation.

## CLI and configuration

The CLI provides setup/auth/doctor, config/profile, provider/model, session, tool, plugin, prompt, workflow, and Git command families. Configuration merges hardcoded defaults, global files, project files, environment values, and CLI overrides. TOML, JSON, YAML, `${ENV}` interpolation, validation, and masked secrets are required.

## Quality gates

Required verification includes:

- unit and black-box CLI integration tests;
- deterministic mock-provider/tool agent tests;
- permission, traversal, symlink, cancellation, queue, and mode-boundary tests;
- Ratatui snapshots, resize cases, huge-output bounds, and PTY terminal restoration exercises;
- Linux, macOS, and Windows build/test CI;
- rustfmt, clippy with warnings denied, rustdoc, coverage, security audit, spelling, benchmark compilation, and mdBook deployment.

No fixed latency, memory, uptime, or coverage number may be claimed without a measured, enforced gate.

## Technical foundation

- Rust 2024 edition
- Tokio asynchronous runtime and bounded channels
- Ratatui immediate-mode terminal UI
- Crossterm terminal events and capabilities
- `tui-textarea` full-screen composer
- Serde domain, event, configuration, and persistence models
- Clap command parsing
- Reqwest/Rustls provider networking
- Fat-LTO release profile and cross-platform release artifacts

Pleiades does not collect product telemetry. It does not silently change terminal backgrounds. It does not currently provide sub-agents, semantic vector memory, a WASM plugin sandbox, or a hosted plugin marketplace.
