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
| M13 | Workflow Engine | ❌ Pending |
| M14 | Git Integration | ❌ Pending |
| M15 | Testing & CI | ❌ Pending |
| M16 | Documentation | ❌ Pending |
| M17 | Optimization | ❌ Pending |
| M18 | Release | ❌ Pending |

**Current Focus**: Milestone 13 — Workflow Engine (step sequencing, parallel steps, conditional branching)

**Completed Commits** (12 milestones implemented):
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
- M12: (pending) — Prompt Library: pleiades-prompts crate with PromptTemplate engine ({{var}}, {{var|default}}), 8 built-in prompts, PromptLibrary with custom persistence, wired into Engine default system prompt, `pleiades prompt` CLI

**GitHub**: https://github.com/CodWasTaken/Pleiades

## What Remains (Next Priorities)

| Priority | Milestone | Effort | Description |
|----------|-----------|--------|-------------|
| **1** | **M13**: Workflow Engine | 3 days | Step sequencing, parallel steps, conditional branching |
| **2** | **M14**: Git Integration | 3 days | Commit generation, PR summaries, code review |
| **3** | **M15**: Testing & CI | 5 days | Unit tests, integration tests, GitHub Actions CI |
| **4** | **M16**: Documentation | 3 days | MDBook, rustdoc, guides |
| **5** | **M17**: Optimization | 3 days | Performance profiling, caching, latency |
| **6** | **M18**: Release | 2 days | Binary distribution, package managers |
