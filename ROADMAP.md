# Pleiades Roadmap

> See [docs/ROADMAP.md](docs/ROADMAP.md) for the complete development roadmap.

## Summary

| Phase | Description | Status |
|-------|-------------|--------|
| M0 | Planning | ✅ **Complete** |
| M1 | Bootstrap | ✅ **Complete** |
| M2 | Configuration System | ✅ **Complete** |
| M3 | Provider System | ✅ **Complete** |
| M4 | Model System | ✅ **Complete** |
| M5 | Chat Engine | ✅ **Complete** |
| M6 | Tool System | ✅ **Complete** |
| M7 | Interactive Chat (REPL) | ✅ **Complete** |
| M8 | Agent Loop | ✅ **Complete** |
| M9 | Memory & Persistence | ✅ **Complete** |
| M10 | Terminal UI (TUI) | ✅ **Complete** |
| M11 | Plugin System | ✅ **Complete** |
| M12 | Prompt Library | ✅ **Complete** |
| M13 | Workflow Engine | ✅ **Complete** |
| M14 | Git Integration | ✅ **Complete** |
| M15 | Testing & CI | ✅ **Complete** |
| M16 | Documentation | ✅ **Complete** |
| M17 | Optimization | ✅ **Complete** |
| M18 | Release | ✅ **Complete** |

**Status**: All 18 original milestones are complete. Pleiades v2.0 evolves the terminal milestone into a full-screen event-driven Ratatui workspace and hardens the agent runtime, cancellation, permissions, and workspace boundary.

**Milestone commits**:
- M1: 5fcf776 — Initial bootstrap (workspace, crate structure)
- M2: b28648a — Config system with env interpolation, profiles, secrets
- M3: 59e48b9 — Provider system (Anthropic, OpenAI, OpenAI-compatible)
- M4: 9bb6e7e — Model system with registry, discovery, aliasing
- M5: 26cc8ca — Chat engine with session persistence, context management
- M6: 2c8e9a1 — Tool system (9 built-in tools, execution timeout)
- M7: 3c2a014 — Interactive REPL with history, streaming, slash commands
- M8: db262d3 — Agent loop with multi-turn tool calling, permission prompts
- M9: 46e16eb — Memory system with FileStore, LLM summarization
- M10: 125cc2b — Terminal UI: markdown→ANSI rendering, syntax highlighting, LineEditor with tab completion, Spinner
- M11: b638572 — Plugin System: PluginManager, PluginRegistry, HookRunner, plugin.json manifest, CLI commands
- M12: `f2a6d8f` — Prompt library and CLI integration
- M13: `ed3fe30` — Workflow sequencing, parallelism, conditions, retries, and CLI
- M14: `0bc3294` — AI-assisted Git commit, review, summary, and diff commands
- M15: `ccd1f47` — CLI integration tests, snapshots, benchmarks, and CI tuning
- M16: `03306bc` — mdBook, user guide, configuration reference, and docs deployment
- M17: `bf5d46a` — Release optimization and performance baselines
- M18: `64df221` — v1.0 release automation and platform packaging
- Live workspace: `3d9495c` — asynchronous Ratatui shell and typed runtime channels
- Safe autonomy: `3724f90` — permission modals, cancellation tests, path confinement, searchable overlays

**GitHub**: https://github.com/CodWasTaken/Pleiades

## Release availability

Pleiades is published on crates.io under the `pleiades-agent` package family. GitHub releases include five platform binaries and checksums; installer, Homebrew, and AUR metadata are also available.
