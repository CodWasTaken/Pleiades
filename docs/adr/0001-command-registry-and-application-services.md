# ADR 0001 — Command registry and application services

- **Status:** Accepted
- **Date:** 2026-07-14
- **Supersedes:** Hardcoded slash dispatch and numeric command-palette mapping in
  `crates/pleiades-agent-tui/src/state.rs` (`/help`, `palette_commands`,
  `palette_matches`, `run_palette`) and the per-variant CLI handlers in
  `crates/pleiades-agent/src/main.rs` (`build_providers_from_config`,
  `build_test_provider`, `Repl::build_engine`, the reflection-style nested-key
  config get/set helpers).

## Context

Until this ADR, Pleiades described one command in three disconnected places:

1. `crates/pleiades-agent-tui/src/state.rs` — slash commands dispatched inside
   `AppState::execute_input` and the `palette_commands`/`palette_matches`/
   `run_palette` triple that drives the command palette overlay. `run_palette`
   is a numeric `match selected` block whose indices must be kept in sync by
   hand against the textual list.
2. `crates/pleiades-agent/src/main.rs` — clap `Cli` / `Commands` and their
   `handle_*` functions, plus a hand-rolled "reflection over `Config`" for
   nested-key `config` get and set operations.
3. `crates/pleiades-agent/src/repl.rs` — the legacy Rustyline REPL which
   repeats provider construction (`Repl::build_engine`) for a fourth path.

Provider construction is duplicated in `main.rs`, `repl.rs`, and
`providers/lib.rs::ProviderRegistry::from_config`,violating rule 6
("CLI subcommands and live workspace commands must use the same underlying
service layer") and rule 7 ("Do not duplicate business logic between Clap
commands and slash commands"). Plugin-provided commands
(`pleiades_agent_plugins::manifest::PluginCommandManifest`), forthcoming MCP
tools, and future "custom user commands" (`.pleiades/commands/*.toml`) have no
home; they would have to be patched into one of the above three lists.

## Decision

Introduce a new crate, **`pleiades-agent-commands`**, as the single source of
truth for every user-invocable command in Pleiades. The crate exports:

- `CommandSpec` — pure descriptor (path, aliases, description, usage, examples,
  category, availability, permission, arguments, shortcut, handler).
- `CommandHandler` — `async` trait that receives a `CommandContext` snapshot
  and positional `args` and returns a typed `CommandResult`. Handlers never
  touch the terminal, the runtime, or the filesystem.
- `CommandResult` — `Effects(Vec<AppEffect>)`, `OpenOverlay(OverlayKind)`,
  `Notification(...)`, `RenderDocument(...)`, `RuntimeRestart(...)`,
  `BackgroundTask(...)`, `Noop`.
- `AppEffect` — typed frontend/runtime side effects (`SetMode`, `SetProvider`,
  `SetModel`, `ClearConversation`, `LoadSession`, `SaveSession`, `Quit`,
  `Shutdown`, `ReloadExtensions`, `Status`, `Custom`).
- `CommandRegistry` — registration, lookup (canonical path + aliases),
  deepest-path resolution, nested-subcommand `children`, palette `filter`,
  slash `suggest`, and `help_document` generation.

Rules enforced:

- The registry contains only descriptors and handlers. Business logic lives
  in **application services** (next step — issue "unify CLI/TUI through
  application services"), each a pure function returning structured `Report`s
  the caller renders.
- Slash commands, the command palette, help overlays, CLI subcommands, and
  autocomplete suggestions all derive from the registry. No parallel
  hand-maintained lists. The numeric `match selected` in `state.rs:run_palette`
  is replaced by registry queries once item 2 of the release lands.
- Plugin-provided commands, MCP tool commands, and custom user commands extend
  the same registry rather than bypassing it — they implement
  `CommandHandler` and call `CommandRegistry::register` at load time.
- `CommandSpec.availability` (`Interactive`/`Headless`/`Both`) is checked
  before dispatch, so an overlay-only command cannot be invoked from
  `pleiades run --json` without explicit opt-in. `CommandSpec.permission`
  records the minimum permission a caller must hold (the permission engine
  decides enforcement, see 2.2).
- The engine and TUI preserve the existing event-driven separation (rules
  2/3/4): handlers emit typed `AppEffect`s; the runtime converts those into
  `AgentCommand`s in the engine crate; the TUI converts `OverlayKind` to its
  own Ratatui `Overlay` enum. The `pleiades-agent-commands` crate has no
  runtime or terminal dependencies.

## Consequences

- Adding a builtin becomes "write a `handler` closure, `CommandSpec::builder`,
  `register`", instead of editing three lists.
- Removing the index drift hazard between `palette_commands` and
  `run_palette` once item 2 lands.
- A new crate to dependency-order: `pleiades-agent-commands` depends only on
  `pleiades-agent-core`. The `engine` crate will eventually depend on it (we
  will add `From<AppEffect> for AgentCommand` there).
- Verification: parser unit tests, registration conflict tests, alias
  collision tests, subcommand pruning tests, palette filter tests, and help
  document content tests. 30 tests + 1 doctest land alongside this ADR.

## Status of release 2.1

Item 1 (this slice) ships the crate + the registry, types, parser, default
commands, and tests. Items 2–6 of release 2.1 (dynamic help & palette from
the registry, nested slash autocomplete, CLI/TUI service unification,
`/provider` and `/model` families, `/plugins` family) are tracked as GitHub
issues under milestone "2.1 — Unified Workspace Commands".
