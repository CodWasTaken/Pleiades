# Directory structure

This document describes the current v1.1 workspace. Public Cargo packages use the collision-free `pleiades-agent` namespace. The command installed by the `pleiades-agent` package remains `pleiades`.

```text
Pleiades/
├── crates/
│   ├── pleiades-agent/            # CLI binary, REPL, command handlers, integration tests
│   ├── pleiades-agent-core/       # Provider, tool, conversation, model, event, and error types
│   ├── pleiades-agent-config/     # Layered config, profiles, interpolation, secrets, validation
│   ├── pleiades-agent-engine/     # Chat orchestration, agent entry point, memory and sessions
│   ├── pleiades-agent-providers/  # Anthropic, OpenAI, and OpenAI-compatible adapters
│   ├── pleiades-agent-tools/      # Read, write, edit, shell, glob, grep, diff, search, fetch
│   ├── pleiades-agent-tui/        # Terminal input, rendering, themes, and interactive app
│   ├── pleiades-agent-plugins/    # Manifests, shell hooks, registry, and lifecycle manager
│   ├── pleiades-agent-memory/     # Persistent store and session/project/user tiers
│   ├── pleiades-agent-prompts/    # Templates, built-ins, persistence, and benchmarks
│   ├── pleiades-agent-workflow/   # Workflow definitions, validation, and execution
│   ├── pleiades-agent-git/        # Commit, review, PR summary, and diff generation
│   └── pleiades-agent-sdk/        # Public re-exports for extension authors
├── docs/
│   ├── book/                       # mdBook source and configuration
│   ├── ARCHITECTURE.md
│   ├── DIRECTORY_STRUCTURE.md
│   ├── FEATURE_MATRIX.md
│   ├── REQUIREMENTS.md
│   ├── RISK_ANALYSIS.md
│   ├── ROADMAP.md
│   └── VISION.md
├── .github/workflows/              # CI, docs, benchmarks, and release publishing
├── Formula/pleiades.rb             # Homebrew formula
├── packaging/aur/PKGBUILD          # AUR package metadata
├── Cargo.toml                      # Workspace membership and release profile
├── Cargo.lock                      # Reproducible application dependency lock
├── install.sh                      # Checksummed GitHub release installer
└── README.md
```

## Dependency direction

```text
pleiades-agent (binary)
├── pleiades-agent-engine
│   ├── pleiades-agent-core
│   ├── pleiades-agent-config
│   ├── pleiades-agent-providers
│   ├── pleiades-agent-tools
│   ├── pleiades-agent-memory
│   └── pleiades-agent-prompts
├── pleiades-agent-tui
├── pleiades-agent-plugins
├── pleiades-agent-workflow
└── pleiades-agent-git
```

`pleiades-agent-core` has no internal workspace dependencies. Adapters depend inward on its domain traits and types; the CLI composes the adapters at the outer edge.

## Package and Rust crate names

Cargo package hyphens become underscores in Rust paths. For example, the package `pleiades-agent-core` is imported as `pleiades_agent_core`. Physical crate directories match the public package names.
