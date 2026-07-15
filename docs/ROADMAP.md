# Pleiades Roadmap

## Development Phases

### Milestone 0: Planning (✅ Complete)
- [x] Project vision and philosophy (VISION.md)
- [x] System architecture (ARCHITECTURE.md)
- [x] Requirements specification (REQUIREMENTS.md)
- [x] Development roadmap (ROADMAP.md)
- [x] Dependency analysis
- [x] Feature matrix vs. competitors
- [x] Risk analysis
- [x] Directory structure design
- [x] Technology stack finalization
- **Deliverable**: Planning documents

### Milestone 1: Bootstrap (✅ Complete)
- [x] Create Cargo workspace with crate structure (12 crates)
- [x] Set up linters (clippy, rustfmt)
- [x] Minimal executable with version info
- [x] .gitignore and project setup
- [ ] CI/CD (GitHub Actions) — **pending**
- [ ] Pre-commit hooks — **pending**
- [ ] Build scripts (Makefile) — **pending**
- **Deliverable**: Working binary with scaffolding

### Milestone 2: Configuration System (✅ Complete)
- [x] Config file loading (TOML, JSON, YAML)
- [x] Multi-level config merge (defaults < global < project)
- [x] Config validation with error reporting
- [x] Config CLI commands (get, set, edit, validate, show, path, init, reset)
- [x] Profile management (list, save, load, delete, active)
- [x] Environment variable interpolation (`${VAR}`, `$VAR`)
- [x] Secret management (env refs, provider→env-var mapping)
- **Deliverable**: Fully functional configuration system

### Milestone 3: Provider System (✅ Complete)
- [x] Provider trait definition
- [x] Chat completion interface
- [x] Streaming support (SSE parser, retry logic)
- [x] Tool calling support
- [x] Error handling and mapping
- [x] Anthropic provider (real HTTP, streaming, tool use, images)
- [x] OpenAI provider (chat, streaming, embeddings, models)
- [x] OpenAI-compatible provider (OpenRouter, Groq, DeepSeek)
- [x] Provider CLI commands (list, info, test, remove)
- **Deliverable**: Working multi-provider system with 3+ providers

### Milestone 4: Model System (✅ Complete)
- [x] Model registry with metadata
- [x] Model aliasing
- [x] Model capabilities tracking
- [x] Model discovery from providers
- [x] Model CLI commands (list, info, set-default, alias, unalias, discover)
- [x] Pricing information
- [x] Context window management
- **Deliverable**: Model system with registry and CLI

### Milestone 5: Chat Engine (✅ Complete)
- [x] Conversation management (types, roles, content blocks)
- [x] Message types and storage
- [x] Streaming response handling
- [x] Context window management
- [x] Automatic compression (truncation + message dropping)
- [x] Session persistence (save/load/resume as JSON)
- [x] Export functionality (markdown, JSON)
- [x] Session metadata and management
- [x] Session CLI commands (list, show, delete, export, path)
- **Deliverable**: Working chat engine with persistence

### Milestone 6: Tool System (✅ Complete)
- [x] Tool trait definition
- [x] Tool registry
- [x] Tool execution with timeout
- [x] Permission system (allow, ask, deny — config-driven)
- [x] Read tool (file reading with ranges)
- [x] Write tool (file creation)
- [x] Edit tool (targeted editing)
- [x] Glob tool (file pattern matching)
- [x] Grep tool (content search)
- [x] Bash tool (sandboxed execution)
- [x] Diff tool (git-based)
- [x] Search tool (DuckDuckGo web search)
- [x] Fetch tool (HTTP requests)
- [x] Tool CLI commands (list, info, call)
- **Deliverable**: Complete tool system with 9 built-in tools

### Milestone 7: Interactive REPL (✅ Complete)
- [x] REPL loop with rustyline line editing
- [x] History persistence (.pleiades_history)
- [x] Streaming token display
- [x] Session auto-save on each exchange
- [x] Slash commands (/help, /clear, /save, /load, /model, /provider, /info, /tokens, /export, /exit)
- [x] `--chat` flag and `repl` subcommand
- [x] `--session` flag for resuming sessions
- [x] ANSI-colored output
- **Deliverable**: Interactive REPL for chat

### Milestone 8: Agent Loop (✅ Complete)
- [x] Multi-turn tool calling loop
- [x] Anthropic streaming tool call fix (accumulate input_json_delta)
- [x] Typed four-way Ratatui permission decisions for runtime tool execution
- [x] Config-driven always_allow/always_deny support
- [x] Tool execution with output truncation
- [x] Tool result display inline
- [x] Max iteration limit with warning
- **Deliverable**: Working agent loop with tool orchestration

### Milestone 9: Memory & Persistence (✅ Complete)
- [x] FileStore — persistent disk-backed memory (JSON)
- [x] Session/Project/User memory tiers
- [x] MemoryManager wired into Engine
- [x] LLM-based conversation summarization
- [x] Summary injection as system messages
- [x] Auto-save summaries on compression
- Future extension: embedding generation, vector storage, and semantic search are not implemented
- **Deliverable**: Multi-tier memory system with persistence

### Milestone 10: Terminal UI (✅ Complete)
- [x] Full-screen Ratatui application with panic-safe alternate-screen lifecycle
- [x] Concurrent terminal, provider, agent, background, resize, and render events
- [x] Ratatui-native Markdown and Syntect syntax spans
- [x] Multiline textarea, history, paste, slash completion, queue, and cancellation
- [x] Persistent header, conversation, activity, composer, and status regions
- [x] Permission, palette, provider/model, file/session, diff, output, help, configuration, and diagnostics overlays
- [x] Seven Sisters typed design system and ASCII/high-contrast fallback
- **Deliverable**: Live autonomous coding workspace

### Milestone 11: Plugin System (✅ Complete)
- [x] `pleiades-agent-plugins` crate: manifest, hooks, plugin, registry, manager
- [x] Plugin trait + Builtin/Bundled/External kinds
- [x] `plugin.json` manifest parsing and validation
- [x] `HookRunner` for PreToolUse / PostToolUse / PostToolUseFailure
- [x] `PluginManager` with install/uninstall/enable/disable
- [x] `PluginRegistry` with aggregated hooks/tools and enabled state
- [x] CLI: `pleiades plugin {list,install,uninstall,enable,disable}`
- **Deliverable**: Complete plugin SDK with examples

### Milestone 12: Prompt Library (✅ Complete)
- [x] `pleiades-agent-prompts` crate: template, library, builtin, error
- [x] `PromptTemplate` engine with `{{var}}` and `{{var|default}}` substitution
- [x] 8 built-in prompts (assistant, summarizer, code-reviewer, commit-message, pr-summary, explain-diff, refactor, test-generator)
- [x] `PromptLibrary` with custom prompt persistence to disk
- [x] Wired into Engine: default assistant system prompt used when none configured
- [x] CLI: `pleiades prompt {list,show,render,save}`
- **Deliverable**: Prompt library with templates

### Milestone 13: Workflow Engine (✅ Complete)
- [x] Workflow definition format
- [x] Workflow execution
- [x] Step sequencing
- [x] Parallel steps
- [x] Conditional branching
- [x] Reusable workflow files
- **Deliverable**: Workflow engine with examples

### Milestone 14: Git Integration (✅ Complete)
- [x] Commit message generation
- [x] PR summary generation
- [x] Diff inspection
- [x] Code review automation
- **Deliverable**: Git integration tools

### Milestone 15: Testing & CI (✅ Complete)
- [x] Workspace unit test suite
- [x] Integration test suite
- [x] GitHub Actions CI on Linux, macOS, and Windows
- [x] Snapshot testing
- [x] Performance benchmarks
- **Deliverable**: Comprehensive test suite with CI

### Milestone 16: Documentation (✅ Complete)
- [x] mdBook documentation site
- [x] API documentation (rustdoc)
- [x] Architecture documentation
- [x] Configuration reference
- [x] User guide
- **Deliverable**: Professional documentation site

### Milestone 17: Optimization (✅ Complete)
- [x] Cold-start and memory baselines
- [x] Reduced hot-path cloning
- [x] Concurrent provider discovery
- [x] Built-in prompt caching
- [x] Fat LTO, symbol stripping, and release code-size optimization
- **Deliverable**: Performance-optimized release

### Milestone 18: Release (✅ Complete)
- [x] Semantic versioning and v1.0.x releases
- [x] GitHub Releases with five platform binaries
- [x] Publish the `pleiades-agent` package family
- [x] Homebrew formula
- [x] AUR package metadata
- **Deliverable**: v1.0.0 release across multiple channels

## Timeline Estimates

| Milestone | Estimated Effort | Complexity | Dependencies |
|-----------|-----------------|------------|--------------|
| M0: Planning | ✅ Done | Low | None |
| M1: Bootstrap | ✅ Done | Low | M0 |
| M2: Config | ✅ Done | Medium | M1 |
| M3: Providers | ✅ Done | High | M2 |
| M4: Models | ✅ Done | Low | M3 |
| M5: Chat Engine | ✅ Done | High | M3, M4 |
| M6: Tool System | ✅ Done | High | M5 |
| M7: Interactive REPL | ✅ Done | Medium | M5, M6 |
| M8: Agent Loop | ✅ Done | High | M6, M7 |
| M9: Memory & Persistence | ✅ Done | High | M5, M8 |
| M10: Terminal UI | ✅ Done | High | M5, M7 |
| M11: Plugin System | ✅ Done | High | M6, M10 |
| M12: Prompt Library | ✅ Done | Low | M5 |
| M13: Workflow Engine | ✅ Done | Medium | M6 |
| M14: Git Integration | ✅ Done | Medium | M6 |
| M15: Testing & CI | ✅ Done | Medium | M1-M14 |
| M16: Documentation | ✅ Done | Medium | M1-M14 |
| M17: Optimization | ✅ Done | Medium | M15 |
| M18: Release | ✅ Done | Low | M15, M16, M17 |

**Completed**: 18 milestones ✅
**Remaining**: None

## Current Focus

All original milestones are complete. The current v2 architecture upgrades the default session to a live Ratatui coding workspace with typed agent events, safe cancellation, workspace confinement, modal permissions, and evidence-based autonomous behavior. Future work is listed explicitly as future work rather than presented as implemented.

---

## Staged release road map (2.1 → 3.0)

The "Master Implementation Directive" defines a staged evolution from the
v2.0 functional live workspace toward a professional coding environment.
Releases must land in order, each behind the next. Each feature family is
tracked as a GitHub issue under the matching milestone.

> Milestones and issues live at <https://github.com/CodWasTaken/Pleiades/milestones>.

| Release | Title | Status |
|---|---|---|
| 2.1 | Unified Workspace Commands | implemented |
| 2.2 | Safe Autonomous and YOLO Modes | implemented |
| 2.3 | Checkpoints, Context, Verification | implemented |
| 2.4 | MCP, Plugins, Skills, Custom Commands | pending |
| 2.5 | Professional Coding Workspace | pending |
| 2.6 | Sessions, Memory, Observability | pending |
| 2.7 | Subagents and Parallel Work | pending |
| 2.8 | Headless API and IDE Integration | pending |
| 2.9 | Security Hardening | pending |
| 3.0 | Interface and Product Polish | pending |

### Release 2.1 — Unified Workspace Commands

Implemented. Items (GitHub issues):

- [x] 1. Shared command registry (`pleiades-agent-commands` crate, ADR 0001)
- [x] 2. Dynamic help and command palette from the registry
- [x] 3. Nested slash-command autocomplete
- [x] 4. CLI/TUI service unification (application services layer)
- [x] 5. `/provider` and `/model` workspace managers
- [x] 6. `/plugins` workspace manager

### Release 2.2 — Safe Autonomous and YOLO Modes

Implemented. Items (GitHub issues):

- [x] Split approval and sandbox policies (`ApprovalPolicy`, `SandboxPolicy`)
- [x] Add `plan`, `agent`, `auto`, and `yolo` presets
- [x] Add YOLO confirmation and persistent danger status
- [x] Add structured `permissions.rules` and deny-first evaluation
- [x] Add `/permissions` and `pleiades permissions` management commands
- [x] Publish cross-platform CI report and milestone summary

A slice is considered done when `cargo fmt`, `cargo clippy -D warnings`, and
`cargo test --workspace` all pass on Linux, macOS, and Windows, and the
relevant tests and docs are committed alongside the implementation.
