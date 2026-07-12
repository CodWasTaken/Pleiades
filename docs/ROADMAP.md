# Pleiades Roadmap

## Development Phases

### Milestone 0: Planning (Current)
- [x] Project vision and philosophy (VISION.md)
- [x] System architecture (ARCHITECTURE.md)
- [x] Requirements specification (REQUIREMENTS.md)
- [x] Development roadmap (ROADMAP.md)
- [ ] Dependency analysis
- [ ] Feature matrix vs. competitors
- [ ] Risk analysis
- [ ] Directory structure design
- [ ] Technology stack finalization
- **Deliverable**: Planning documents

### Milestone 1: Bootstrap
- [ ] Create Cargo workspace with crate structure
- [ ] Set up CI/CD (GitHub Actions)
- [ ] Configure linters (clippy, rustfmt)
- [ ] Configure formatters
- [ ] Set up test framework
- [ ] Configure pre-commit hooks
- [ ] Create build scripts (Makefile)
- [ ] Minimal executable: `pleiades` prints "Hello from Pleiades"
- [ ] Version info and help output
- **Deliverable**: Working binary with scaffolding

### Milestone 2: Configuration System
- [ ] Config file loading (TOML, JSON, YAML)
- [ ] Multi-level config merge (defaults < global < project < env < CLI)
- [ ] Config validation with error reporting
- [ ] Config CLI commands (get, set, edit, validate)
- [ ] Profile management
- [ ] Environment variable interpolation
- [ ] Secret management (env refs, keyring)
- [ ] Config watch for live reload
- **Deliverable**: Fully functional configuration system

### Milestone 3: Provider System
- [ ] Provider trait definition
- [ ] Chat completion interface
- [ ] Streaming support
- [ ] Tool calling support
- [ ] Error handling and mapping
- [ ] Anthropic provider
- [ ] OpenAI provider
- [ ] OpenAI-compatible provider base
- [ ] Provider CLI commands (list, add, remove, test)
- [ ] Provider documentation
- **Deliverable**: Working multi-provider system with 3+ providers

### Milestone 4: Model System
- [ ] Model registry with metadata
- [ ] Model aliasing
- [ ] Model capabilities tracking
- [ ] Model discovery from providers
- [ ] Model CLI commands (list, info, set-default, alias)
- [ ] Pricing information
- [ ] Context window management
- **Deliverable**: Model system with registry and CLI

### Milestone 5: Chat Engine
- [ ] Conversation management
- [ ] Message types and storage
- [ ] Streaming response handling
- [ ] Context window management
- [ ] Automatic compression
- [ ] Session persistence (save/load/resume)
- [ ] Conversation search
- [ ] Export functionality
- [ ] Session metadata and management
- **Deliverable**: Working chat engine with persistence

### Milestone 6: Tool System
- [ ] Tool trait definition
- [ ] Tool registry
- [ ] Tool execution with timeout
- [ ] Permission system (allow, ask, deny)
- [ ] Read tool (file reading with ranges)
- [ ] Write tool (file creation)
- [ ] Edit tool (targeted editing)
- [ ] Glob tool (file pattern matching)
- [ ] Grep tool (content search)
- [ ] Bash tool (sandboxed execution)
- [ ] Diff tool
- [ ] Search tool (web search)
- [ ] Fetch tool (HTTP)
- [ ] Tool CLI commands
- **Deliverable**: Complete tool system with 10+ tools

### Milestone 7: Agent Engine
- [ ] Task planning
- [ ] Multi-step execution
- [ ] Reflection and correction
- [ ] Retry logic with backoff
- [ ] Sub-agent spawning
- [ ] Progress reporting
- [ ] Interrupt handling
- [ ] Cancellation support
- **Deliverable**: Working agent engine with planning

### Milestone 8: Terminal UI
- [ ] Ratatui integration
- [ ] Main chat interface
- [ ] Streaming text renderer
- [ ] Markdown rendering
- [ ] Syntax highlighting
- [ ] Code block formatting
- [ ] Table rendering
- [ ] Status bar
- [ ] Progress indicators
- [ ] Keyboard shortcuts
- [ ] Responsive layout
- **Deliverable**: Beautiful terminal UI

### Milestone 9: Customization
- [ ] Theme system
- [ ] Built-in themes (light, dark, catppuccin, dracula, tokyo-night)
- [ ] Font configuration
- [ ] Status bar customization
- [ ] Animation toggles
- [ ] Terminal feature detection
- [ ] Wallpaper support (where available)
- **Deliverable**: Highly customizable terminal experience

### Milestone 10: Plugin SDK
- [ ] Plugin trait definition
- [ ] WASM runtime integration
- [ ] Plugin manifest parsing
- [ ] Hook system
- [ ] Event subscription
- [ ] Plugin isolation and sandboxing
- [ ] Permission declaration
- [ ] Plugin lifecycle (install, update, remove)
- [ ] Plugin CLI commands
- [ ] Plugin documentation
- [ ] Example plugins
- **Deliverable**: Complete plugin SDK with examples

### Milestone 11: Memory System
- [ ] Working memory (conversation context)
- [ ] Session memory
- [ ] Project memory
- [ ] User memory
- [ ] Embedding generation
- [ ] Vector storage (local)
- [ ] Semantic search
- [ ] Automatic summarization
- [ ] Memory pruning
- [ ] Memory CLI commands
- **Deliverable**: Multi-tier memory system

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
- [ ] Command aliases
- **Deliverable**: Workflow engine with examples

### Milestone 14: Git Integration
- [ ] Commit message generation
- [ ] PR summary generation
- [ ] Diff explanation
- [ ] Code review automation
- [ ] Merge conflict resolution assistance
- **Deliverable**: Git integration tools

### Milestone 15: Testing
- [ ] Unit test coverage (90%+)
- [ ] Integration test suite
- [ ] Snapshot testing
- [ ] E2E test suite
- [ ] Performance benchmarks
- [ ] Stress testing
- [ ] Coverage reporting in CI
- **Deliverable**: Comprehensive test suite

### Milestone 16: Documentation
- [ ] MDBook documentation site
- [ ] API documentation (rustdoc)
- [ ] Architecture documentation
- [ ] Plugin development guide
- [ ] Provider implementation guide
- [ ] Configuration reference
- [ ] FAQ
- [ ] Troubleshooting guide
- **Deliverable**: Professional documentation site

### Milestone 17: Optimization
- [ ] Cold start time < 100ms
- [ ] Memory profiling and reduction
- [ ] Streaming latency optimization
- [ ] Concurrent request optimization
- [ ] Caching layer
- [ ] Profile-guided optimization
- [ ] LTO and code size optimization
- **Deliverable**: Performance-optimized release

### Milestone 18: Release
- [ ] Semantic versioning (v1.0.0)
- [ ] Release automation (GitHub Releases)
- [ ] Homebrew formula
- [ ] Cargo crate publish
- [ ] npm package (installer)
- [ ] Binary distribution (GitHub Releases)
- [ ] AUR package
- [ ] Deb package
- [ ] RPM package
- [ ] Scoop manifest
- [ ] Winget manifest
- [ ] Release blog post
- **Deliverable**: v1.0.0 release across all channels

## Post-1.0 Roadmap

### v1.1 — Voice & Audio
- Speech-to-text for voice input
- Text-to-speech for voice response
- Audio model support
- Voice activity detection

### v1.2 — Collaborative Features
- Session sharing
- Multi-user collaboration
- Comment/annotation on conversations
- Shared plugin repositories

### v1.3 — Advanced Agent Capabilities
- Long-running autonomous agents
- Scheduled tasks
- Multi-agent coordination
- Agent marketplace

### v1.4 — Enterprise Features
- SSO/SAML authentication
- Audit logging
- Compliance reporting
- Team management
- Centralized policy management

## Timeline Estimates

| Milestone | Estimated Effort | Complexity | Dependencies |
|-----------|-----------------|------------|--------------|
| M0: Planning | 1 day | Low | None |
| M1: Bootstrap | 2 days | Low | M0 |
| M2: Config | 3 days | Medium | M1 |
| M3: Providers | 5 days | High | M2 |
| M4: Models | 2 days | Low | M3 |
| M5: Chat Engine | 5 days | High | M3, M4 |
| M6: Tool System | 5 days | High | M5 |
| M7: Agent Engine | 5 days | High | M5, M6 |
| M8: Terminal UI | 5 days | High | M5 |
| M9: Customization | 3 days | Medium | M8 |
| M10: Plugin SDK | 5 days | High | M6, M8 |
| M11: Memory | 5 days | High | M5 |
| M12: Prompts | 2 days | Low | M5 |
| M13: Workflows | 3 days | Medium | M6 |
| M14: Git | 3 days | Medium | M6 |
| M15: Testing | 5 days | Medium | M1-M14 |
| M16: Docs | 3 days | Medium | M1-M14 |
| M17: Optimization | 3 days | Medium | M15 |
| M18: Release | 2 days | Low | M15, M16, M17 |

**Total estimated effort**: ~63 days of focused development

## Current Focus

**We are here → Milestone 0: Planning** (Completing planning documentation, dependency analysis, and architecture design before writing any code.)
