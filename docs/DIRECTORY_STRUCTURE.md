# Pleiades Directory Structure

## Repository Structure

```
pleiades/
├── .github/                          # GitHub configuration
│   ├── ISSUE_TEMPLATE/
│   │   ├── bug_report.md
│   │   ├── feature_request.md
│   │   └── config.yml
│   ├── workflows/
│   │   ├── ci.yml                    # Main CI pipeline
│   │   ├── release.yml               # Release automation
│   │   ├── audit.yml                 # Security audit
│   │   ├── docs.yml                  # Documentation build
│   │   └── benchmark.yml             # Benchmark tracking
│   ├── dependabot.yml
│   ├── CODEOWNERS
│   └── PULL_REQUEST_TEMPLATE.md
│
├── docs/                             # Documentation
│   ├── ADR/                          # Architecture Decision Records
│   │   ├── 0001-use-rust.md
│   │   ├── 0002-hexagonal-arch.md
│   │   └── ...
│   ├── guides/
│   │   ├── getting-started.md
│   │   ├── configuration.md
│   │   ├── providers.md
│   │   ├── plugins.md
│   │   ├── themes.md
│   │   └── contributing.md
│   ├── reference/
│   │   ├── cli.md
│   │   ├── config.md
│   │   ├── api.md
│   │   └── sdk.md
│   ├── examples/
│   │   ├── custom-provider.md
│   │   ├── hello-world-plugin.md
│   │   └── workflow-templates/
│   ├── VISION.md
│   ├── ARCHITECTURE.md
│   ├── REQUIREMENTS.md
│   ├── ROADMAP.md
│   ├── FEATURE_MATRIX.md
│   ├── RISK_ANALYSIS.md
│   └── DIRECTORY_STRUCTURE.md
│
├── src/                              # Rust source code (workspace root)
│
├── crates/                           # Cargo workspace members
│   ├── pleiades-cli/                 # CLI binary crate
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs               # Entry point
│   │       ├── app.rs                # Application setup
│   │       ├── commands/             # CLI command handlers
│   │       │   ├── mod.rs
│   │       │   ├── chat.rs
│   │       │   ├── config.rs
│   │       │   ├── provider.rs
│   │       │   ├── model.rs
│   │       │   ├── tool.rs
│   │       │   ├── plugin.rs
│   │       │   ├── memory.rs
│   │       │   ├── workflow.rs
│   │       │   ├── doctor.rs
│   │       │   ├── init.rs
│   │       │   ├── login.rs
│   │       │   └── update.rs
│   │       └── output/               # Output formatting
│   │           ├── mod.rs
│   │           ├── text.rs
│   │           ├── json.rs
│   │           ├── ndjson.rs
│   │           └── silent.rs
│   │
│   ├── pleiades-core/                # Core domain logic
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── provider/             # Provider trait and types
│   │       │   ├── mod.rs
│   │       │   ├── trait.rs
│   │       │   ├── chat.rs
│   │       │   ├── streaming.rs
│   │       │   ├── embedding.rs
│   │       │   └── error.rs
│   │       ├── model/                # Model registry
│   │       │   ├── mod.rs
│   │       │   ├── registry.rs
│   │       │   ├── capabilities.rs
│   │       │   └── pricing.rs
│   │       ├── conversation/         # Conversation management
│   │       │   ├── mod.rs
│   │       │   ├── conversation.rs
│   │       │   ├── message.rs
│   │       │   ├── content.rs
│   │       │   └── compression.rs
│   │       ├── tool/                 # Tool trait and registry
│   │       │   ├── mod.rs
│   │       │   ├── trait.rs
│   │       │   ├── registry.rs
│   │       │   ├── permission.rs
│   │       │   └── result.rs
│   │       └── error.rs              # Core error types
│   │
│   ├── pleiades-config/              # Configuration system
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── loader.rs             # Config loading from sources
│   │       ├── merge.rs              # Config merging logic
│   │       ├── validate.rs           # Config validation
│   │       ├── profile.rs            # Profile management
│   │       ├── secret.rs             # Secret/key management
│   │       ├── watch.rs              # File watching
│   │       └── types.rs              # Config types
│   │
│   ├── pleiades-providers/           # Provider implementations
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── anthropic/            # Anthropic provider
│   │       │   ├── mod.rs
│   │       │   ├── client.rs
│   │       │   ├── chat.rs
│   │       │   └── types.rs
│   │       ├── openai/               # OpenAI provider
│   │       │   ├── mod.rs
│   │       │   ├── client.rs
│   │       │   ├── chat.rs
│   │       │   └── types.rs
│   │       ├── google/               # Google/Gemini provider
│   │       │   ├── mod.rs
│   │       │   ├── client.rs
│   │       │   ├── chat.rs
│   │       │   └── types.rs
│   │       ├── openrouter/           # OpenRouter provider
│   │       ├── groq/                 # Groq provider
│   │       ├── ollama/               # Ollama provider
│   │       ├── lmstudio/             # LM Studio provider
│   │       ├── mistral/              # Mistral provider
│   │       ├── cohere/               # Cohere provider
│   │       ├── deepseek/             # DeepSeek provider
│   │       ├── together/             # Together AI provider
│   │       ├── xai/                  # xAI/Grok provider
│   │       ├── perplexity/           # Perplexity provider
│   │       ├── azure/                # Azure OpenAI provider
│   │       └── openai_compat/        # Generic OpenAI-compatible
│   │
│   ├── pleiades-tools/               # Built-in tool implementations
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── read.rs               # File read tool
│   │       ├── write.rs              # File write tool
│   │       ├── edit.rs               # File edit tool
│   │       ├── glob.rs               # Glob pattern tool
│   │       ├── grep.rs               # Content search tool
│   │       ├── bash.rs               # Shell execution tool
│   │       ├── diff.rs               # Diff tool
│   │       ├── search.rs             # Web search tool
│   │       ├── fetch.rs              # HTTP fetch tool
│   │       ├── clipboard.rs          # Clipboard tool
│   │       ├── memory.rs             # Memory search tool
│   │       ├── agent.rs              # Sub-agent tool
│   │       └── think.rs              # Reasoning tool
│   │
│   ├── pleiades-engine/              # Chat and agent engine
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── engine.rs             # Main engine loop
│   │       ├── chat.rs               # Chat processing
│   │       ├── stream.rs             # Stream handling
│   │       ├── agent.rs              # Agent execution
│   │       ├── plan.rs               # Task planning
│   │       ├── retry.rs              # Retry logic
│   │       └── context.rs            # Context management
│   │
│   ├── pleiades-tui/                 # Terminal UI
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── app.rs                # TUI application
│   │       ├── render/               # Rendering
│   │       │   ├── mod.rs
│   │       │   ├── markdown.rs
│   │       │   ├── code.rs           # Code blocks with highlighting
│   │       │   ├── table.rs
│   │       │   ├── stream.rs         # Streaming text
│   │       │   └── image.rs          # Terminal images (sixels, kitty)
│   │       ├── widget/               # UI widgets
│   │       │   ├── mod.rs
│   │       │   ├── input.rs          # Input area
│   │       │   ├── output.rs         # Output area
│   │       │   ├── status.rs         # Status bar
│   │       │   ├── progress.rs       # Progress indicators
│   │       │   └── panel.rs          # Split panels
│   │       ├── theme/                # Theming
│   │       │   ├── mod.rs
│   │       │   ├── theme.rs
│   │       │   ├── builtin.rs        # Built-in themes
│   │       │   └── loader.rs         # Custom theme loading
│   │       ├── keybind.rs            # Keyboard shortcuts
│   │       └── terminal.rs           # Terminal detection
│   │
│   ├── pleiades-plugins/             # Plugin system
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── manifest.rs           # Plugin manifest parsing
│   │       ├── loader.rs             # Plugin loading
│   │       ├── wasm.rs               # WASM runtime
│   │       ├── hooks.rs              # Hook system
│   │       ├── events.rs             # Event subscription
│   │       ├── permissions.rs        # Plugin permissions
│   │       └── registry.rs           # Plugin registry
│   │
│   ├── pleiades-memory/              # Memory system
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── working.rs            # Working memory
│   │       ├── session.rs            # Session memory
│   │       ├── project.rs            # Project memory
│   │       ├── user.rs               # User memory
│   │       ├── store.rs              # Vector storage
│   │       ├── embed.rs              # Embedding generation
│   │       ├── search.rs             # Semantic search
│   │       └── prune.rs              # Memory pruning
│   │
│   ├── pleiades-workflow/            # Workflow engine
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── workflow.rs           # Workflow definition
│   │       ├── execute.rs            # Workflow execution
│   │       ├── step.rs               # Step types
│   │       └── alias.rs              # Command aliases
│   │
│   ├── pleiades-git/                 # Git integration
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── commit.rs             # Commit generation
│   │       ├── pr.rs                 # PR summaries
│   │       ├── review.rs             # Code review
│   │       └── diff.rs               # Diff explanation
│   │
│   └── pleiades-sdk/                 # Plugin SDK (for plugin authors)
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── prelude.rs            # SDK prelude
│           ├── plugin.rs             # Plugin trait
│           ├── tool.rs               # Tool definition
│           ├── hook.rs               # Hook registration
│           ├── event.rs              # Event types
│           ├── config.rs             # Plugin config
│           ├── storage.rs            # Plugin storage
│           ├── http.rs               # HTTP client
│           └── log.rs                # Logging
│
├── tests/                            # Integration tests
│   ├── common/                       # Test helpers
│   │   ├── mod.rs
│   │   ├── mock_provider.rs
│   │   └── fixture.rs
│   ├── config_tests.rs
│   ├── provider_tests.rs
│   ├── chat_tests.rs
│   ├── tool_tests.rs
│   ├── engine_tests.rs
│   ├── memory_tests.rs
│   └── workflow_tests.rs
│
├── benches/                          # Benchmarks
│   ├── config_bench.rs
│   ├── provider_bench.rs
│   ├── chat_bench.rs
│   ├── tool_bench.rs
│   └── memory_bench.rs
│
├── examples/                         # Usage examples
│   ├── basic_chat.rs
│   ├── custom_provider.rs
│   ├── tool_usage.rs
│   └── workflow_definition/
│
├── scripts/                          # Utility scripts
│   ├── setup.sh                      # Development setup
│   ├── release.sh                    # Release script
│   ├── benchmark.sh                  # Benchmark runner
│   └── coverage.sh                   # Coverage report
│
├── SourceCodeCC/                     # Reference implementations (study material)
│   ├── Claude Code/
│   └── Claw Code/
│
├── Cargo.toml                        # Workspace root
├── Cargo.lock
├── Makefile                          # Build automation
├── README.md
├── CONTRIBUTING.md
├── SECURITY.md
├── CODE_OF_CONDUCT.md
├── CHANGELOG.md
├── ROADMAP.md
├── LICENSE                           # MIT
├── .gitignore
├── .editorconfig
├── .env.example
├── .gitattributes
├── rust-toolchain.toml               # Rust toolchain configuration
├── rustfmt.toml                      # Formatter configuration
├── clippy.toml                       # Linter configuration
├── deny.toml                         # Dependency deny configuration
└── typos.toml                        # Typo checking configuration
```

## Crate Dependency Graph

```
pleiades-cli (binary)
  ├── pleiades-core     (traits, types, domain)
  ├── pleiades-config   (configuration)
  ├── pleiades-engine   (chat + agent engine)
  ├── pleiades-tui      (terminal UI)
  ├── pleiades-tools    (built-in tools)
  ├── pleiades-providers (all providers)
  ├── pleiades-plugins  (plugin system)
  ├── pleiades-memory   (memory system)
  ├── pleiades-workflow (workflow engine)
  └── pleiades-git      (git integration)

pleiades-core 
  └── (no internal deps, standalone)

pleiades-config
  └── pleiades-core (types only)

pleiades-providers
  ├── pleiades-core
  └── pleiades-config (for provider config)

pleiades-tools
  ├── pleiades-core
  └── pleiades-config

pleiades-engine
  ├── pleiades-core
  ├── pleiades-config
  ├── pleiades-providers
  └── pleiades-tools

pleiades-tui
  ├── pleiades-core
  └── pleiades-config

pleiades-plugins
  ├── pleiades-core
  └── pleiades-config

pleiades-memory
  ├── pleiades-core
  └── pleiades-config

pleiades-workflow
  ├── pleiades-core
  └── pleiades-config

pleiades-git
  ├── pleiades-core
  └── pleiades-config

pleiades-sdk
  └── pleiades-core (re-exported types)
```

This dependency structure ensures:
- `pleiades-core` has zero internal dependencies
- All crates depend on `pleiades-core` for domain types
- `pleiades-cli` is the only binary — everything else is a library
- The dependency graph is acyclic
- Feature modules can be compiled independently
