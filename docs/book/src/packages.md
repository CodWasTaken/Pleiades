# Cargo packages

The crates.io names `pleiades` and `pleiades-core` are owned by unrelated projects. Pleiades therefore publishes under one collision-free namespace while keeping its user-facing command unchanged.

| Package | Purpose |
|---|---|
| `pleiades-agent` | Installable CLI package; provides the `pleiades` executable |
| `pleiades-agent-core` | Domain traits and types |
| `pleiades-agent-config` | Configuration loading and validation |
| `pleiades-agent-engine` | Chat and agent orchestration |
| `pleiades-agent-providers` | Provider adapters |
| `pleiades-agent-tools` | Built-in tools |
| `pleiades-agent-mcp` | Model Context Protocol client primitives |
| `pleiades-agent-tui` | Terminal UI |
| `pleiades-agent-plugins` | Plugin manifests and shell hooks |
| `pleiades-agent-memory` | Persistent memory tiers |
| `pleiades-agent-prompts` | Prompt templates and library |
| `pleiades-agent-workflow` | Workflow definitions and executor |
| `pleiades-agent-git` | AI-assisted Git operations |
| `pleiades-agent-sdk` | Re-exported extension API |

Package names use hyphens; Rust import paths use underscores. For example:

```rust
use pleiades_agent_core::{Provider, Tool};
```

For end users, `cargo install pleiades-agent` installs a binary named `pleiades`. Workspace packages use compatible semantic versions and are published in dependency order by the release workflow; a release only republishes packages whose contents or dependency requirements changed.
