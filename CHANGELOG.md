# Changelog

All notable changes to Pleiades will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [1.1.0] - 2026-07-13

### Changed

- Renamed the crates.io package family to the available `pleiades-agent` namespace.
- Renamed all workspace crate directories and Rust crate imports to match their public package names.
- Kept the installed executable name as `pleiades` for command-line compatibility.
- Added dependency-ordered crates.io publication with registry propagation retries.
- Updated installation, roadmap, architecture, and package documentation for the new namespace.

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
