# Pleiades implementation matrix

This document records implemented behavior rather than planned or competitor claims. `Implemented` means the repository contains a wired production path and tests or CI coverage; `Partial` identifies an explicit limitation.

| Area | Capability | Status | Notes |
|---|---|---|---|
| Runtime | Live full-screen Ratatui workspace | Implemented | Persistent async loop; no blocking read in the active TUI |
| Runtime | Concurrent input, streaming, tools, resize, ticks | Implemented | Bounded Tokio channels and `tokio::select!` |
| Runtime | Cancellation and queued follow-ups | Implemented | Deterministic mock tests |
| UI | Five persistent regions | Implemented | Header, conversation, activity, composer, status |
| UI | Native Markdown and code highlighting | Implemented | Ratatui spans plus Syntect regions |
| UI | Multiline selection, undo/redo, paste, history | Implemented | `tui-textarea`, bracketed paste, input history |
| UI | Palette/help/provider/model/file/session overlays | Implemented | Searchable keyboard navigation |
| UI | Permission, diff, output, details, diagnostics | Implemented | Ratatui modals; no active stdin prompts |
| UI | Theme/capability fallback | Implemented | Seven themes including high-contrast and ASCII |
| UI | Terminal background integration | Not implemented | Pleiades does not modify emulator settings |
| Providers | OpenAI Platform API | Implemented | Chat, streaming, tools, model/embedding APIs |
| Providers | Anthropic | Implemented | Chat, streaming, tool calls |
| Providers | OpenAI-compatible endpoints | Implemented | OpenRouter, Groq, DeepSeek, and configurable endpoints |
| Providers | ChatGPT subscription | Implemented | Delegated to the official Codex CLI; credentials are not copied |
| Providers | Local models | Implemented | Through an OpenAI-compatible local endpoint |
| Agent | Multi-turn tool loop | Implemented | Tool results return to the provider until completion/limit |
| Agent | Professional coding protocol | Implemented | Inspection, plan, focused changes, observed checks, diff review, report |
| Agent | Provider-independent activity | Implemented | Typed kinds and lifecycle statuses |
| Agent | Per-tool modal for API providers | Implemented | Four once/session decisions |
| Agent | Per-tool modal for delegated Codex calls | Partial | Codex owns internal calls; selected Codex sandbox is the boundary |
| Safety | Plan / Agent / Auto / YOLO | Implemented | Mode changes cancel old-boundary work |
| Safety | Structured permission rules | Implemented | Deny-first shell clause evaluation |
| Safety | Workspace path confinement | Implemented | Traversal and symlink escape tests |
| Safety | Sandboxed built-in shell | Partial | Bubblewrap on Linux, sandbox-exec on macOS; refused elsewhere in Agent mode |
| Tools | Read, write, edit, bash, glob, grep, diff | Implemented | Structured inputs/results and permission metadata |
| Tools | Web search and fetch | Implemented | DuckDuckGo/HTTP adapters |
| Sessions | Persistence, resume, export | Implemented | JSON domain state and Markdown/JSON export |
| Memory | Session/project/user tiers | Implemented | File-backed stores and summarization integration |
| Memory | Embedding semantic search | Not implemented | Embedding/vector-storage path remains future work |
| Plugins | Local manifests and shell hooks | Implemented | Install/enable lifecycle and pre/post hooks |
| Plugins | WASM sandbox | Not implemented | Hooks are trusted child processes |
| Workflows | Sequence, parallel, condition, retry, timeout | Implemented | CLI create/list/show/validate/run |
| Git | Commit, review, PR summary, diff explanation | Implemented | Provider-backed prompt flows |
| Quality | Linux/macOS/Windows CI | Implemented | Build/test matrix plus lint, docs, audit, coverage, typos |
| Quality | Widget snapshots and resize tests | Implemented | Ratatui `TestBackend` |
| Distribution | crates.io and release binaries | Implemented | Collision-free `pleiades-agent` package family |
| Distribution | Installer, Homebrew, AUR metadata | Implemented | Checked-in packaging assets |
| Telemetry | Anonymous usage collection | Not implemented | Pleiades sends no product telemetry |
