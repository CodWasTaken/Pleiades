# Pleiades

> A next-generation, provider-agnostic terminal AI assistant. Extensible, fast, and beautifully designed for modern development workflows.

<p align="center">
  <img src="https://img.shields.io/badge/status-active-green" alt="Status: Active"/>
  <img src="https://img.shields.io/badge/license-MIT-blue" alt="License: MIT"/>
  <img src="https://img.shields.io/badge/language-Rust-orange" alt="Language: Rust"/>
  <img src="https://img.shields.io/badge/edition-2024-purple" alt="Edition: 2024"/>
</p>

---

## Overview

Pleiades is a **next-generation terminal AI assistant** that puts you in control. It has native OpenAI and Anthropic adapters plus a generic OpenAI-compatible adapter for services such as OpenRouter, Groq, DeepSeek, and self-hosted endpoints.

Named after the Seven Sisters star cluster, Pleiades represents a constellation of capabilities working in harmony.

## Key Features

- **Provider Agnostic** — Use OpenAI, Anthropic, or an OpenAI-compatible service
- **Plugin Architecture** — Install local plugin manifests with pre/post shell hooks
- **Multi-Engine** — Chat, agent, workflow — choose the right interaction model for each task
- **Beautiful Terminal** — Markdown rendering, syntax highlighting, status bar, progress indicators
- **Memory System** — Multi-tier memory from conversation context to long-term project knowledge
- **Permission System** — Granular control over what the AI can do. Read-only, workspace-write, or full access
- **Customizable** — Select terminal themes and configure models, permissions, prompts, and workflows
- **Production Quality** — Cross-platform CI, integration tests, release binaries, and checksummed installs

## Quick Start

```bash
# Install the latest Linux/macOS release
curl -fsSL https://raw.githubusercontent.com/CodWasTaken/Pleiades/master/install.sh | sh

# Ensure the default install directory is on PATH
export PATH="$HOME/.local/bin:$PATH"

# Create configuration
pleiades config init
pleiades config set core.default_provider openai
pleiades config set core.default_model gpt-4o
pleiades config set providers.openai.api_key '${OPENAI_API_KEY}'

# Start a chat
pleiades

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
pleiades repl               Start REPL session
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
| OpenAI | ✅ Implemented |
| Anthropic | ✅ Implemented |
| OpenRouter | ✅ Implemented |
| Groq | ✅ Implemented |
| DeepSeek | ✅ Implemented |
| Any OpenAI-compatible | ✅ Implemented |

## Project Status

**18 milestones complete** — Pleiades v1.1.0 is available from crates.io and as cross-platform GitHub release binaries, with a checksummed installer, Homebrew metadata, and AUR metadata.

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
- [x] **M10: Terminal UI** — Markdown→ANSI rendering, syntax highlighting, LineEditor with tab completion, Spinner
- [x] **M11: Plugin System** — PluginManager, PluginRegistry, HookRunner (PreToolUse/PostToolUse/PostToolUseFailure), CLI
- [x] **M12: Prompt Library** — PromptTemplate engine, 8 built-in prompts, PromptLibrary with persistence, CLI
- [x] **M13: Workflow Engine** — Step sequencing, parallel steps, conditional branching
- [x] **M14: Git Integration** — Commit messages, PR summaries, code review
- [x] **M15: Testing & CI** — Integration tests, snapshots, benchmarks, GitHub Actions CI
- [x] **M16: Documentation** — mdBook site, rustdoc CI, user guide
- [x] **M17: Optimization** — Cold start, memory, latency, LTO
- [x] **M18: Release** — crates.io packages, GitHub binaries, checksummed installer, Homebrew, and AUR metadata

## Architecture

Pleiades follows a clean **hexagonal architecture** with event-driven communication between subsystems.

```
┌─────────────┐  ┌─────────────┐  ┌──────────────┐
│   CLI/TUI   │  │   Engine    │  │   Plugins    │
├─────────────┤  ├─────────────┤  ├──────────────┤
│    clap     │  │  Provider   │  │ Shell Hooks  │
│   ratatui   │  │    Chat     │  │  Hook System │
│   crossterm │  │    Agent    │  │  Tool API    │
└─────────────┘  └─────────────┘  └──────────────┘
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
