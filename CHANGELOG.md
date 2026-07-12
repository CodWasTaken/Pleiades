# Changelog

All notable changes to Pleiades will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

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
