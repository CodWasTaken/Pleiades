# Pleiades

> A professional, provider-agnostic autonomous coding agent. Native Rust, live Ratatui workspace, safe project automation.

<p align="center">
  <img src="https://img.shields.io/badge/status-active-green" alt="Status: Active"/>
  <img src="https://img.shields.io/badge/license-MIT-blue" alt="License: MIT"/>
  <img src="https://img.shields.io/badge/language-Rust-orange" alt="Language: Rust"/>
  <img src="https://img.shields.io/badge/edition-2024-purple" alt="Edition: 2024"/>
</p>

---

## Overview

Pleiades is a continuously running development environment that can inspect a project, plan a solution, edit code, execute guarded commands, diagnose failures, validate its work, review the resulting diff, and report the evidence. It supports OpenAI through usage-based API keys or ChatGPT subscription sign-in delegated to the official Codex CLI, alongside Anthropic and OpenAI-compatible services such as OpenRouter, Groq, DeepSeek, and self-hosted endpoints.

Named after the Seven Sisters star cluster, Pleiades represents a constellation of capabilities working in harmony.

## Key Features

- **Provider Agnostic** — Use OpenAI, Anthropic, or an OpenAI-compatible service
- **Two OpenAI Login Modes** — Choose Platform API billing or ChatGPT subscription access through the official Codex CLI
- **Plugin Architecture** — Install local plugin manifests with pre/post shell hooks
- **Live Ratatui Workspace** — Input, rendering, model streams, tools, permissions, and cancellation remain concurrent
- **Autonomous Coding Agent** — Inspect projects, create and edit files, run commands, and verify results
- **Transparent Activity** — Typed planning, searching, reading, editing, testing, review, completion, and failure events
- **Seven Sisters Design** — Native Markdown widgets, multiline composer, searchable overlays, diffs, tool output, and accessible fallbacks
- **Memory System** — Multi-tier memory from conversation context to long-term project knowledge
- **Safe Autonomy** — Plan, Agent, Auto, and YOLO modes; structured deny-first permission rules; workspace path confinement; modal permission decisions
- **Customizable** — Select terminal themes and configure models, permissions, prompts, and workflows
- **Production Quality** — Cross-platform CI, integration tests, release binaries, and checksummed installs

## Quick Start

```bash
# Install the latest Linux/macOS release
curl -fsSL https://raw.githubusercontent.com/CodWasTaken/Pleiades/master/install.sh | sh

# Ensure the default install directory is on PATH
export PATH="$HOME/.local/bin:$PATH"

# Choose ChatGPT subscription sign-in or an API key
pleiades setup

# Diagnose configuration at any time
pleiades doctor

# Start the live coding workspace
pleiades

# Or use the explicit agent command
pleiades chat

# One-shot prompt
pleiades "explain this codebase"

# Use a specific model
pleiades --model claude-sonnet-4

```

Do **not** run `cargo install pleiades`: that name belongs to an unrelated machine-learning crate on crates.io. To compile this project directly, use:

```bash
cargo install --git https://github.com/CodWasTaken/Pleiades pleiades-agent
```

The crates.io distribution uses the collision-free `pleiades-agent` package name while installing the same `pleiades` executable:

```bash
cargo install pleiades-agent
```

## CLI Commands

```
pleiades --chat             Start interactive session
pleiades <prompt>           One-shot prompt
pleiades repl               Start the legacy line-oriented compatibility REPL
pleiades chat               Start the live autonomous coding workspace
pleiades setup              Guided provider and authentication setup
pleiades auth               Sign in, check status, or sign out through Codex
pleiades doctor             Diagnose configuration and authentication
pleiades config             Configure settings (get, set, edit, validate, show, path, init, reset)
pleiades profile            Manage profiles (list, save, load, delete, active)
pleiades provider           Manage AI providers (list, info, test, remove)
pleiades model              Manage models (list, info, set-default, alias, unalias, discover)
pleiades session            Manage chat sessions (list, show, delete, export, path)
pleiades tool               Manage tools (list, info, call)
pleiades plugin             Manage plugins (list, install, uninstall, enable, disable)
pleiades prompt             Manage prompts (list, show, render, save)
pleiades workflow           Manage workflows
pleiades git                Git integration (commit, review, summary, diff)
pleiades --version          Show version
```

## Supported Providers

| Provider | Status |
|----------|--------|
| OpenAI API key | ✅ Implemented |
| OpenAI ChatGPT subscription | ✅ Via official Codex CLI |
| Anthropic | ✅ Implemented |
| OpenRouter | ✅ Implemented |
| Groq | ✅ Implemented |
| DeepSeek | ✅ Implemented |
| Any OpenAI-compatible | ✅ Implemented |

## Project Status

**18 milestones complete** — Pleiades v2.0.0 is a live, event-driven autonomous coding workspace built with Ratatui.

- [x] **M0: Planning** — Vision, architecture, requirements, roadmap
- [x] **M1: Bootstrap** — Cargo workspace (13 crates), CI, minimal executable
- [x] **M2: Configuration** — Multi-level config (TOML/JSON/YAML), profiles, env interpolation, secrets
- [x] **M3: Providers** — Provider system with Anthropic, OpenAI, OpenAI-compatible (OpenRouter, Groq, DeepSeek)
- [x] **M4: Models** — Model registry, discovery, aliasing, pricing, context windows
- [x] **M5: Chat Engine** — Conversation management, streaming, session persistence, export
- [x] **M6: Tool System** — 9 built-in tools (Read, Write, Edit, Bash, Glob, Grep, Diff, Search, Fetch)
- [x] **M7: Interactive REPL** — rustyline editing, history, streaming tokens, slash commands, session auto-save
- [x] **M8: Agent Loop** — Multi-turn tool calling, Anthropic streaming fix, permission prompts, iteration limits
- [x] **M9: Memory & Persistence** — FileStore, Session/Project/User tiers, LLM summarization, auto-compression
- [x] **M10: Terminal UI** — Full-screen Ratatui loop, native Markdown widgets, multiline editor, overlays, typed themes
- [x] **M11: Plugin System** — PluginManager, PluginRegistry, HookRunner (PreToolUse/PostToolUse/PostToolUseFailure), CLI
- [x] **M12: Prompt Library** — PromptTemplate engine, 8 built-in prompts, PromptLibrary with persistence, CLI
- [x] **M13: Workflow Engine** — Step sequencing, parallel steps, conditional branching
- [x] **M14: Git Integration** — Commit messages, PR summaries, code review
- [x] **M15: Testing & CI** — Integration tests, snapshots, benchmarks, GitHub Actions CI
- [x] **M16: Documentation** — mdBook site, rustdoc CI, user guide
- [x] **M17: Optimization** — Cold start, memory, latency, LTO
- [x] **M18: Release** — crates.io packages, GitHub binaries, checksummed installer, Homebrew, and AUR metadata

## Architecture

Pleiades follows a clean **hexagonal architecture** with typed, event-driven communication between the live UI and autonomous runtime.

```
Terminal events ─┐
Agent events ────┼─> AppState reducer ─> Ratatui render
Render ticks ────┘

UI actions ─────────> AgentCommand channel ─> Runtime ─> Providers + tools
```

See [ARCHITECTURE.md](docs/ARCHITECTURE.md) for the complete design.

## Configuration

Pleiades supports multiple configuration formats (TOML, JSON, YAML) with five levels of precedence:

1. Defaults (hardcoded)
2. Global config (`~/.config/pleiades/`)
3. Project config (`./.pleiades/`)
4. Environment variables (`PLEIADES_*`)
5. CLI flags

Profiles allow switching between configurations for different contexts.

The default `seven-sisters` theme is joined by `andromeda`, `orion`, `event-horizon`, `solar-wind`, `high-contrast`, and `ascii`. Press `F1` inside the application for searchable keyboard help.

## Development

```bash
# Clone and set up
git clone https://github.com/CodWasTaken/Pleiades.git
cd Pleiades
make setup

# Build
make build

# Test
make test

# Lint
make lint
```

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

This project follows a **milestone-based development** approach. Each milestone must be completed (all tests passing, documentation updated) before the next begins.

## License

MIT License — see [LICENSE](LICENSE) for details.

## Related Projects

Pleiades was inspired by studying:
- [Claude Code](https://docs.anthropic.com/en/docs/claude-code) — Anthropic's terminal AI
- [Claw Code](https://github.com/UltraWorkers/claw-code) — Rust reimplementation of Claude Code
- [OpenCode](https://opencode.ai) — Provider-agnostic terminal AI
- [Gemini CLI](https://github.com/google-gemini/gemini-cli) — Google's AI terminal tool

Pleiades is an **original implementation** that combines the best ideas from these projects while being provider-agnostic from the ground up.
