# Changelog

All notable changes to Pleiades will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- Added `pleiades-agent-mcp`, MCP configuration under `mcp.servers`, JSON-RPC
  protocol types, stdio client primitives, redacted server status reporting,
  auth-source metadata, tool filtering, and validation for invalid server
  definitions. See `docs/adr/0009-mcp-client-foundation.md`.
- Added shared `McpService`, `/mcp` workspace commands, `pleiades mcp`
  headless commands, and a native MCP manager overlay entry point. Configured
  server reports are redacted, enable/disable/remove persist through the shared
  service layer, and tool commands report configured exposure filters without
  claiming live schema discovery.
- Added plugin trust metadata parsing and enforcement. External plugins now
  install disabled, cannot be enabled until trusted, expose detailed trust
  reports through `/plugins info` and `pleiades plugin info`, and support
  `/plugins trust|untrust` plus `pleiades plugin trust|untrust`.
- Added checkpoint commands for Release 2.3: `/checkpoint create`, list, show,
  restore preview, confirmed restore, delete, plus `/undo`, `/redo`, and
  `/rewind` entry points. Checkpoints persist conversation state, provider,
  model, mode, Git head/branch, changed files, and staged/unstaged diffs.
- Added `/context status`, `/context inspect`, `/context compact`,
  `/context pin`, `/context unpin`, and `/context sources` live workspace
  commands backed by an engine-owned context accountant. Reports show
  approximate token usage by conversation, tool output, memory, compression
  summaries, pins, and detected sources.
- Added `/verify`, `/test`, `/run <command>`, and `/review` workspace
  commands backed by structured verification reports. Verification runs in a
  background task, captures changed files, diff stats, commands, exit status,
  stdout/stderr snippets, and refuses to report executed success in Plan mode.
- Added runtime doom-loop detection with `agent.max_repeats` and a tested stop
  path for repeated identical tool failures.
- Split runtime access control into typed `ApprovalPolicy` and
  `SandboxPolicy` values, and added `auto` and `yolo` mode presets. Auto runs
  without prompts inside the workspace; YOLO is the explicit full-host preset.
  See `docs/adr/0003-approval-and-sandbox-policies.md`.

- Added service-backed `/provider list`, `/provider use`, `/provider info`,
  `/provider add`, `/provider remove`, and `/provider reload` workspace
  commands with nested completion, structured documents, permission metadata,
  and an injectable service context for deterministic tests.
- Added a shared streamed provider connectivity test used by both
  `pleiades provider test` and `/provider test`, plus a provider-independent
  model service and `/model list`, `/model use`, `/model info`,
  `/model discover`, `/model alias`, `/model unalias`, `/model favorite`,
  `/model favorites`, and `/model reasoning` commands. Favorites and validated
  reasoning effort are persisted without expanding provider secrets.
- Added service-backed `/plugins list`, `/plugins info`, `/plugins install`,
  `/plugins uninstall`, `/plugins enable`, `/plugins disable`,
  `/plugins update`, `/plugins permissions`, and `/plugins reload` commands
  with structured executable-hook/tool permission reporting. External plugin
  updates validate staged content before replacing the installed copy and
  preserve the plugin's enable state.
- Added `pleiades-agent-permissions`, structured `permissions.rules`, runtime
  rule evaluation, `/permissions show|allow|ask|deny|reset|test`, and matching
  `pleiades permissions ...` CLI commands. Shell commands are parsed into
  clauses, deny rules take precedence, command substitution requires review
  unless denied, and workspace path escapes are blocked in Agent and Auto mode.
  See `docs/adr/0004-structured-permission-rules.md`.

### Fixed

- Built-in plugins now honor persisted enable/disable state during discovery.
- Added `pleiades-agent-services`, a terminal-independent application service
  layer with typed provider and plugin reports, canonical provider adapter
  construction, and temporary-root tests.
- Added `pleiades-agent-commands` crate: typed command registry, `CommandSpec`,
  `CommandResult`, `CommandHandler`, `AppEffect`, `OverlayKind`,
  `RenderableDocument`, `CommandContext`, `Suggestion`, parser, and the
  live-workspace default commands — the single source of truth for slash
  commands, the command palette, help, and autocomplete. See
  `docs/adr/0001-command-registry-and-application-services.md`.
- Added registry, runtime, and reducer coverage for command registration, alias collisions,
  nested subcommand lookup, deepest-path resolution, palette filtering, slash
  autocomplete, help document generation, typed command events, and live model-state updates.

### Changed

- CLI provider list/info, CLI plugin listing, and the live provider picker now
  consume shared application services instead of duplicating configuration
  traversal and presentation logic.
- The Cargo workspace now includes `crates/pleiades-agent-commands`. Workspace
  formatting, Clippy with warnings denied, and tests remain green.
- Slash input, command-palette selections, help, and completion now derive from
  the shared registry. Command results cross the runtime boundary as typed
  overlay, notification, document, effect, and shutdown events.

## [2.0.0] - 2026-07-13

### Added

- Added a real full-screen Ratatui application with concurrent terminal input, provider streams, agent events, background work, resize handling, and 20 FPS redraws.
- Added five persistent regions, native Markdown and Syntect-to-Ratatui code highlighting, multiline `tui-textarea` composition, history, slash completion, queued follow-ups, and preserved scrolling.
- Added searchable command, provider, model, file, session, and help overlays plus configuration, diagnostics, tool detail/output, diff, and keyboard-driven permission views.
- Added typed provider-independent activity kinds/statuses and an engine-owned autonomous runtime with bounded command/event channels.
- Added deterministic mock-agent coverage for permission waits, Plan-mode denial, cancellation, mode-boundary changes, and queued follow-ups.
- Added canonical workspace confinement, symlink escape protection, Agent-mode Linux/macOS command isolation, huge-output bounds, resize tests, and Ratatui snapshots.
- Added the `seven-sisters`, `andromeda`, `orion`, `event-horizon`, `solar-wind`, `high-contrast`, and `ascii` design systems.

### Changed

- Replaced blocking Rustyline, direct ANSI streaming, direct tool execution in the TUI, and stdin permission prompts in the default session.
- Made `seven-sisters` the default theme and retained legacy theme names as aliases.
- Upgraded the default system protocol to require repository inspection, focused planning, observed validation, final diff review, and evidence-based completion reports.
- Changed public activity events and tool contexts to typed lifecycle and sandbox-boundary models, requiring a major version bump.

### Security

- Filesystem tools now reject traversal, absolute outside-workspace targets, and symlinked ancestors that escape the selected workspace.
- Plan mode rejects mutating tools before approval; mode changes cancel work executing under the previous boundary.
- Agent-mode shell execution refuses to run when the platform workspace-write isolation layer is unavailable.

## [1.2.0] - 2026-07-13

### Added

- Added autonomous ChatGPT subscription tasks through the official Codex CLI with workspace-scoped file editing and command execution.
- Added live provider-managed agent activity events for commands, file changes, searches, and tool calls.
- Added a real `pleiades chat` command plus `/mode`, `/workspace`, and `/status` interactive controls.

### Changed

- Replaced the legacy fixed-width default REPL with the richer markdown terminal application.
- Redesigned the interactive experience with a compact Pleiades constellation theme and clearer agent activity output.
- Default subscription sessions now use a `workspace-write` sandbox rooted at the launch directory; `/mode plan` switches to read-only operation.

## [1.1.1] - 2026-07-13

### Added

- Added guided `pleiades setup`, OpenAI subscription `auth`, and configuration `doctor` commands.
- Added ChatGPT subscription access by delegating authentication and execution to the official Codex CLI without reading its OAuth token cache.

### Fixed

- Preserve OpenAI HTTP 429 details and distinguish exhausted API quota from temporary request throttling.
- Explain that ChatGPT subscriptions and OpenAI Platform API billing are separate, with actionable recovery commands.
- Start the interactive REPL when `pleiades` is run without a subcommand after setup.

## [1.1.0] - 2026-07-13

### Changed

- Renamed the crates.io package family to the available `pleiades-agent` namespace.
- Renamed all workspace crate directories and Rust crate imports to match their public package names.
- Kept the installed executable name as `pleiades` for command-line compatibility.
- Added dependency-ordered crates.io publication with registry propagation retries.
- Updated installation, roadmap, architecture, and package documentation for the new namespace.
- Published all 13 v1.1.0 packages and five cross-platform release binaries.

## [1.0.1] - 2026-07-13

### Fixed

- Route one-shot prompts through the real streaming agent/tool loop.
- Exit the REPL cleanly instead of sending `/exit` to the configured provider.
- Make the release installer follow the latest GitHub release by default.
- Replace the incorrect crates.io install command with verified GitHub release and source-install instructions.
- Describe shell-hook plugins accurately and disable the reserved sandbox flag by default.
- Remove the crates.io release job that could not publish through occupied package names.

## [1.0.0] - 2026-07-13

### Added

- Workflow execution with sequencing, parallel batches, conditions, retries, timeouts, variables, and CLI management.
- AI-assisted Git commit messages, code review, PR summaries, and diff inspection.
- Black-box CLI integration tests, snapshots, Criterion benchmarks, and cached CI builds.
- mdBook user guide, complete configuration reference, and GitHub Pages deployment.
- Optimized release profile, concurrent model discovery, prompt caching, and performance baselines.
- Cross-platform GitHub Release artifacts, verified install script, Homebrew formula, and AUR package.

### Changed

- All workspace crates are versioned at 1.0.0 and include publishable internal dependency versions.

## [0.1.0] - 2026-07-12

### Added (Milestone 0: Planning)
- **VISION.md** — Project vision, philosophy, and design principles
- **ARCHITECTURE.md** — Hexagonal architecture with event-driven design
- **REQUIREMENTS.md** — Full functional and non-functional requirements
- **ROADMAP.md** — 18-milestone development roadmap with timeline estimates
- **FEATURE_MATRIX.md** — Comparison with Claude Code, Claw Code, OpenCode, Gemini CLI
- **RISK_ANALYSIS.md** — 15-item risk assessment with mitigation strategies
- **DIRECTORY_STRUCTURE.md** — Complete module organization with dependency graph

### Added (Milestone 1: Bootstrap)
- **Cargo workspace** with 12 crate modules organized by feature
- **14 crate modules**: pleiades-cli, pleiades-core, pleiades-config, pleiades-engine,
  pleiades-tui, pleiades-providers, pleiades-tools, pleiades-plugins, pleiades-memory,
  pleiades-workflow, pleiades-git, pleiades-sdk
- **Core domain types**: Provider trait (async), ModelRegistry, Conversation,
  Tool trait, Error types, Event system
- **Configuration system**: Multi-level config loader (TOML/JSON/YAML),
  config types, validation, env var support
- **Provider stubs**: Anthropic, OpenAI, OpenAI-compatible providers
- **Built-in tools**: Read, Write, Edit, Bash, Glob, Grep — all fully implemented
- **Engine**: Chat engine with streaming support, tool execution, event emission
- **Plugin system**: Manifest parsing, hook registry, plugin lifecycle
- **Memory system**: 4-tier memory (working, session, project, user)
- **TUI foundation**: Theme system (Catppuccin, Dracula, Tokyo Night),
  renderer stubs
- **Workflow engine**: Workflow definitions and executor stub
- **Git integration**: Commit generation and review stubs
- **Plugin SDK**: Re-exported core types for plugin authors
- **CLI binary**: `pleiades` with chat, prompt, version, help modes
- **CI/CD**: GitHub Actions for lint, test, coverage, security audit,
  benchmarks, docs, release
- **Configuration**: rustfmt, clippy, deny, typos, editorconfig
- **Community**: ISSUE_TEMPLATE, PR_TEMPLATE, CODE_OF_CONDUCT, SECURITY,
  CONTRIBUTING, Makefile
- **Documentation**: Professional README with feature overview

## [0.2.0] - 2026-07-12

### Added (Milestone 10: Terminal UI)
- **pleiades-tui** crate with `TuiApp`, `TerminalRenderer`, `LineEditor`
- Markdown→ANSI rendering using pulldown-cmark + syntect syntax highlighting
- Streaming token renderer with `MarkdownStreamState` for incremental output
- `Spinner` with braille frames for progress indication
- `LineEditor` with slash-command tab completion (rustyline v14)
- Engine integration, permission prompts, session auto-save
- Tests: markdown rendering, code highlighting, tables, lists, links, streaming, spinner

### Added (Milestone 11: Plugin System)
- **pleiades-plugins** crate: manifest, hooks, plugin, registry, manager modules
- Plugin trait + Builtin/Bundled/External plugin kinds
- `plugin.json` manifest parsing and validation
- `HookRunner` for PreToolUse / PostToolUse / PostToolUseFailure hooks
- `PluginManager` with install/uninstall/enable/disable from local dirs
- `PluginRegistry` with aggregated hooks/tools and enabled state
- CLI: `pleiades plugin {list,install,uninstall,enable,disable}`
- Tests: hook execution (allow/deny/fail), plugin lifecycle (install/list/enable/disable/uninstall), hook aggregation

### Added (Milestone 12: Prompt Library)
- **pleiades-prompts** crate: template, library, builtin, error modules
- `PromptTemplate` engine with `{{var}}` and `{{var|default}}` substitution
- 8 built-in prompts: default-assistant, summarizer, code-reviewer, commit-message, pr-summary, explain-diff, refactor, test-generator
- `PromptLibrary` with custom prompt persistence to `~/.config/pleiades/prompts/`
- Wired into Engine: default assistant system prompt used when none configured
- CLI: `pleiades prompt {list,show,render,save}`
- Tests: template substitution, defaults, missing var errors, JSON rendering, library override, persistence
