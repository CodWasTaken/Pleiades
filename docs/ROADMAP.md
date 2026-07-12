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
- [x] Permission prompts (y/n) for tool execution
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
- [ ] Embedding generation — **pending**
- [ ] Vector storage — **pending**
- [ ] Semantic search — **pending**
- **Deliverable**: Multi-tier memory system with persistence

### Milestone 10: Terminal UI
- [ ] Ratatui integration
- [ ] Main chat interface (panels: chat, input, status)
- [ ] Streaming text renderer
- [ ] Markdown rendering
- [ ] Syntax highlighting
- [ ] Code block formatting
- [ ] Status bar with model/provider/token info
- [ ] Keyboard shortcuts (Ctrl+C, Tab, arrows)
- [ ] Responsive layout
- [ ] Scrollable history
- **Skeleton exists**: `pleiades-tui/` with TuiApp, Renderer, Theme
- **Deliverable**: Beautiful terminal UI

### Milestone 11: Plugin System
- [ ] Plugin trait definition
- [ ] WASM runtime integration
- [ ] Plugin manifest parsing
- [ ] Hook system
- [ ] Event subscription
- [ ] Plugin isolation and sandboxing
- [ ] Permission declaration
- [ ] Plugin lifecycle (install, update, remove)
- [ ] Plugin CLI commands
- [ ] Example plugins
- **Skeleton exists**: `pleiades-plugins/` with hooks, manifest, registry
- **Deliverable**: Complete plugin SDK with examples

### Milestone 12: Prompt Library
- [ ] Prompt template engine
- [ ] Built-in prompt templates
- [ ] Variable substitution
- [ ] Macros
- [ ] Snippets
- [ ] Custom prompt creation
- **Deliverable**: Prompt library with templates

### Milestone 13: Workflow Engine
- [ ] Workflow definition format
- [ ] Workflow execution
- [ ] Step sequencing
- [ ] Parallel steps
- [ ] Conditional branching
- [ ] Reusable workflows
- **Skeleton exists**: `pleiades-workflow/` with Workflow, Executor structs
- **Deliverable**: Workflow engine with examples

### Milestone 14: Git Integration
- [ ] Commit message generation
- [ ] PR summary generation
- [ ] Diff explanation
- [ ] Code review automation
- **Skeleton exists**: `pleiades-git/` with commit, review modules
- **Deliverable**: Git integration tools

### Milestone 15: Testing & CI
- [ ] Unit test coverage (80%+)
- [ ] Integration test suite
- [ ] GitHub Actions CI
- [ ] Snapshot testing
- [ ] Performance benchmarks
- **Deliverable**: Comprehensive test suite with CI

### Milestone 16: Documentation
- [ ] MDBook documentation site
- [ ] API documentation (rustdoc)
- [ ] Architecture documentation
- [ ] Configuration reference
- [ ] User guide
- **Deliverable**: Professional documentation site

### Milestone 17: Optimization
- [ ] Cold start time optimization
- [ ] Memory profiling and reduction
- [ ] Streaming latency optimization
- [ ] Caching layer
- [ ] LTO and code size optimization
- **Deliverable**: Performance-optimized release

### Milestone 18: Release
- [ ] Semantic versioning (v1.0.0)
- [ ] GitHub Releases with binaries
- [ ] Cargo crate publish
- [ ] Homebrew formula
- [ ] AUR package
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
| M10: Terminal UI | 5 days | High | M5, M7 |
| M11: Plugin System | 5 days | High | M6, M10 |
| M12: Prompt Library | 2 days | Low | M5 |
| M13: Workflow Engine | 3 days | Medium | M6 |
| M14: Git Integration | 3 days | Medium | M6 |
| M15: Testing & CI | 5 days | Medium | M1-M14 |
| M16: Documentation | 3 days | Medium | M1-M14 |
| M17: Optimization | 3 days | Medium | M15 |
| M18: Release | 2 days | Low | M15, M16, M17 |

**Completed**: 9 milestones ✅ (approx 28 days of effort)
**Remaining**: 9 milestones (approx 31 days of effort)

## Current Focus

**We are here → Milestone 10: Terminal UI** — Building a full Ratatui-based terminal interface with chat panels, markdown rendering, syntax highlighting, and keyboard navigation.
