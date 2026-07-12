# Pleiades Requirements Specification

## 1. Functional Requirements

### 1.1 Core CLI
- `pleiades` — Launch interactive session (REPL mode)
- `pleiades <prompt>` — One-shot prompt execution
- `pleiades chat` — Start or resume an interactive chat session
- `pleiades run <file>` — Execute commands from a file
- `pleiades init` — Initialize Pleiades in current directory
- `pleiades version` — Display version information
- `pleiades help [command]` — Display help information

### 1.2 Configuration System
- `pleiades config` — View/edit configuration
- `pleiades config get <key>` — Get a config value
- `pleiades config set <key> <value>` — Set a config value
- `pleiades config edit` — Open config in editor
- `pleiades config validate` — Validate configuration
- `pleiades config profile list` — List configuration profiles
- `pleiades config profile use <name>` — Switch to a profile
- `pleiades config profile create <name>` — Create a new profile
- Support JSON, YAML, and TOML config formats
- Five levels of config precedence: defaults < global < project < env < CLI
- Config validation with clear error messages
- Environment variable interpolation in config values

### 1.3 Provider System
- `pleiades provider list` — List configured providers
- `pleiades provider add <name>` — Add a new provider
- `pleiades provider remove <name>` — Remove a provider
- `pleiades provider set-default <name>` — Set default provider
- `pleiades provider test <name>` — Test provider connectivity
- Support for: OpenAI, Anthropic, Google, OpenRouter, Groq, Ollama, LM Studio, Mistral, Cohere, Azure OpenAI, DeepSeek, Together AI, xAI/Grok, Perplexity, OpenAI-compatible endpoints
- Generic provider interface for custom endpoints

### 1.4 Model System
- `pleiades model list` — List available models
- `pleiades model info <name>` — Show model details
- `pleiades model set-default <name>` — Set default model
- `pleiades model alias <name> <alias>` — Create a model alias
- Model auto-discovery from provider APIs
- Model capabilities display (context window, pricing, features)
- Model aliases for common references

### 1.5 Chat Engine
- Multi-turn conversation
- Streaming responses with real-time rendering
- Conversation history management
- Context window management and compression
- Conversation persistence (save/load/resume)
- Session management with metadata
- Message editing and branching
- Search within conversation history
- Export conversations (markdown, JSON, HTML)

### 1.6 Tool System
- `pleiades tool list` — List available tools
- `pleiades tool info <name>` — Show tool details
- `pleiades tool permissions <name>` — Show/modify tool permissions
- Built-in tools: read, write, edit, glob, grep, bash, diff, search
- Tool permissions with three modes (allow, ask, deny)
- Concurrent tool execution where safe
- Tool result streaming
- Tool timeout and cancellation
- Tool composition and chaining

### 1.7 Agent Engine
- Task planning and decomposition
- Multi-step reasoning with reflection
- Automatic retry and recovery
- Sub-agent spawning for complex tasks
- Parallel execution where possible
- Interrupt and cancellation support
- Progress reporting
- Timeout management

### 1.8 Plugin System
- `pleiades plugin list` — List installed plugins
- `pleiades plugin install <name>` — Install a plugin
- `pleiades plugin uninstall <name>` — Remove a plugin
- `pleiades plugin enable <name>` — Enable a plugin
- `pleiades plugin disable <name>` — Disable a plugin
- `pleiades plugin info <name>` — Show plugin details
- `pleiades plugin search <query>` — Search plugin registry
- Plugin manifest with versioning
- WASM-based plugin execution
- Hook system for extension
- Permission declarations
- Dependency management

### 1.9 Memory System
- `pleiades memory` — View memory summary
- `pleiades memory search <query>` — Semantic search of memory
- `pleiades memory clear` — Clear memory
- Working memory (current conversation)
- Session memory (recent activity)
- Project memory (project-specific knowledge)
- User memory (preferences, patterns)
- Automatic memory pruning and summarization
- Embeddings-based semantic search

### 1.10 Workflow Engine
- `pleiades workflow list` — List workflows
- `pleiades workflow run <name>` — Run a workflow
- `pleiades workflow edit <name>` — Edit a workflow
- Workflow definitions in YAML/TOML
- Step sequencing and parallel execution
- Conditional steps
- Reusable workflow templates
- Custom command aliases

### 1.11 Git Integration
- `pleiades git commit` — Generate commit messages
- `pleiades git review` — Review staged changes
- `pleiades git diff <ref>` — Explain a diff
- `pleiades git pr` — Generate PR summaries
- `pleiades git conflict <file>` — Help resolve merge conflicts
- Automatic commit generation with conventional commit format
- PR description generation from branch changes

### 1.12 Diagnostics
- `pleiades doctor` — System health check
- `pleiades doctor --fix` — Auto-fix issues
- `pleiades update` — Check for and apply updates
- `pleiades login <provider>` — Authenticate with a provider
- `pleiades logout <provider>` — Clear authentication

## 2. Non-Functional Requirements

### 2.1 Performance
- Cold start: < 100ms to interactive prompt
- First token: < 50ms from request submission
- Streaming: < 20ms between tokens
- Memory: < 50MB RSS idle, < 200MB under load
- Plugin load: < 10ms per plugin
- Config parse: < 5ms
- No noticeable UI jank or lag

### 2.2 Reliability
- Zero crashes from recoverable errors
- Graceful degradation on provider failure
- Automatic reconnection on network interruption
- Session recovery on crash
- 99.9% uptime of core functionality
- Comprehensive error recovery

### 2.3 Security
- Secrets never logged or exposed in output
- API keys stored with environment variable references
- Optional OS keyring integration
- No telemetry by default (opt-in only)
- Path traversal protection
- Command injection prevention
- Tool permission enforcement
- Sandboxed process execution

### 2.4 Compatibility
- Linux (primary target, all distros)
- macOS (Intel and Apple Silicon)
- Windows (WSL2, native Windows Terminal)
- All major terminal emulators
- tmux and screen compatibility
- SSH session compatibility
- Pipe/redirect compatibility

### 2.5 Usability
- Tab completion for all commands
- Clear, colorful, readable output
- Progress indicators for long operations
- Context-sensitive help
- Keyboard shortcuts for common operations
- Configurable output verbosity
- Dark and light theme support

### 2.6 Extensibility
- Plugin SDK with comprehensive documentation
- Well-defined extension points (hooks, events, tools)
- Plugin marketplace architecture
- Theme system
- Custom tool creation
- Provider implementation guide

### 2.7 Maintainability
- 90%+ test coverage
- Comprehensive documentation
- Clear code organization
- Consistent coding style
- Public API documentation
- Architecture decision records

## 3. Technical Requirements

### 3.1 Language and Runtime
- Written in Rust (edition 2024)
- Async runtime: tokio
- CLI framework: clap (derive API)
- TUI framework: ratatui
- Terminal: crossterm
- Serialization: serde + serde_json
- Config parsing: toml, json, yaml

### 3.2 Build System
- Cargo with workspace layout
- Multi-crate architecture
- Feature flags for optional capabilities
- Release profile optimized for size and speed

### 3.3 Testing
- Unit tests alongside code
- Integration tests in tests/ directory
- Property-based testing (proptest)
- Benchmark tests (criterion)
- Snapshot testing (insta)
- E2E tests with mocked providers

### 3.4 CI/CD
- GitHub Actions for CI
- Pre-commit hooks
- Clippy linting
- Rustfmt formatting
- Security audit (cargo-audit)
- Dependency review
- Coverage reporting (tarpaulin)

### 3.5 Documentation
- Rustdoc for API documentation
- MDBook for user documentation
- Architecture Decision Records (ADRs)
- Example-driven documentation
- Video tutorials (eventual)

## 4. Provider Requirements

Each provider implementation must support:
- Chat completions (streaming and non-streaming)
- Tool/function calling
- Configurable parameters (temperature, top_p, max_tokens, etc.)
- Error handling with provider-specific error mapping
- Rate limiting
- Authentication (API key, OAuth, custom)

### Required Providers (v1.0)
1. **OpenAI** — gpt-4o, gpt-4o-mini, o3, o4-mini
2. **Anthropic** — claude-sonnet-4, claude-opus-4, claude-haiku-4
3. **Google** — gemini-2.5-pro, gemini-2.5-flash
4. **OpenRouter** — Unified access to 200+ models
5. **Groq** — Fast inference for open models
6. **Ollama** — Local model hosting
7. **LM Studio** — Local model hosting
8. **Mistral** — mistral-large, mistral-small
9. **Cohere** — command-r, command-r+
10. **DeepSeek** — deepseek-chat, deepseek-reasoner
11. **Together AI** — Hosted open models
12. **xAI/Grok** — grok-3, grok-3-mini
13. **Perplexity** — sonar-pro, sonar-deep-research
14. **Azure OpenAI** — Enterprise OpenAI access
15. **OpenAI-compatible** — Any custom endpoint

## 5. Tool Requirements

### Core Tools (built-in, v1.0)
1. **Read** — Read files with line ranges
2. **Write** — Write/create files
3. **Edit** — Targeted file editing
4. **Glob** — File pattern matching
5. **Grep** — Content search
6. **Bash** — Command execution with sandbox
7. **Diff** — Show file differences
8. **Search** — Web search
9. **Fetch** — HTTP requests
10. **Clipboard** — System clipboard access
11. **Memory** — Semantic memory search
12. **Agent** — Sub-agent spawning
13. **Think** — Structured reasoning

### Optional Tools (plugins)
- GitHub integration
- Git operations
- Image generation
- PDF processing
- Database queries
- Kubernetes management
- Docker operations
- Cloud provider SDKs
- Language-specific tools

## 6. Plugin SDK Requirements

### Version 1.0
- Plugin manifest format (TOML)
- Hook registration API
- Tool registration API
- Event subscription API
- Configuration API
- Storage API (key-value)
- Logging API
- HTTP client API
- Permission declaration

### Plugin Manifest (`pleiades.toml`)
```toml
[plugin]
name = "my-plugin"
version = "1.0.0"
description = "Does amazing things"
author = "Developer Name"
license = "MIT"
min_pleiades_version = "0.1.0"

[permissions]
required = ["filesystem:read", "network"]
optional = ["filesystem:write"]

[hooks]
on_tool_call = "pre"
on_message = "post"
on_startup = true

[tools]
my_tool = { description = "Does a thing" }
```
