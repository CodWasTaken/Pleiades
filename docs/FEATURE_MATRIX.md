# Pleiades Feature Matrix

## Comparison with Reference Implementations

This matrix compares Pleiades (planned) with Claude Code, Claw Code, OpenCode, and Gemini CLI across key capabilities.

| Feature | Pleiades (planned) | Claude Code | Claw Code | OpenCode | Gemini CLI |
|---------|-------------------|-------------|-----------|----------|------------|
| **Core** | | | | | |
| Language | Rust | TypeScript/Bun | Rust | TypeScript | TypeScript |
| Open Source | ✅ MIT | ❌ Proprietary | ✅ MIT | ✅ Apache 2.0 | ❌ Proprietary |
| Provider Agnostic | ✅ Primary | ❌ Anthropic-only | ⚠️ Anthropic-first | ✅ Multiple | ❌ Google-only |
| Plugin System | ✅ WASM-based | ⚠️ Built-in only | ✅ Hook-based | ✅ SDK | ❌ |
| **Providers** | | | | | |
| Anthropic | ✅ | ✅ Native | ✅ Native | ✅ | ❌ |
| OpenAI | ✅ | ❌ | ⚠️ Compat | ✅ | ❌ |
| Google/Gemini | ✅ | ❌ | ❌ | ⚠️ | ✅ Native |
| OpenRouter | ✅ | ❌ | ❌ | ❌ | ❌ |
| Groq | ✅ | ❌ | ❌ | ❌ | ❌ |
| Ollama | ✅ | ❌ | ❌ | ✅ | ❌ |
| LM Studio | ✅ | ❌ | ❌ | ❌ | ❌ |
| Mistral | ✅ | ❌ | ❌ | ❌ | ❌ |
| Cohere | ✅ | ❌ | ❌ | ❌ | ❌ |
| DeepSeek | ✅ | ❌ | ❌ | ❌ | ❌ |
| Together AI | ✅ | ❌ | ❌ | ❌ | ❌ |
| xAI/Grok | ✅ | ❌ | ✅ | ❌ | ❌ |
| Perplexity | ✅ | ❌ | ❌ | ❌ | ❌ |
| Azure OpenAI | ✅ | ❌ | ❌ | ❌ | ❌ |
| Custom Endpoint | ✅ Generic | ❌ | ❌ | ✅ | ❌ |
| **Models** | | | | | |
| Model Registry | ✅ | ✅ | ✅ | ✅ | ✅ |
| Model Aliases | ✅ | ✅ | ✅ | ✅ | ✅ |
| Auto-Discovery | ✅ | ❌ | ❌ | ❌ | ❌ |
| Pricing Info | ✅ | ❌ | ❌ | ❌ | ❌ |
| Context Window Mgmt | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Chat** | | | | | |
| Streaming | ✅ | ✅ | ✅ | ✅ | ✅ |
| Multi-turn | ✅ | ✅ | ✅ | ✅ | ✅ |
| Session Persistence | ✅ | ✅ | ✅ | ✅ | ✅ |
| Search History | ✅ | ❌ | ❌ | ✅ | ❌ |
| Export | ✅ | ✅ | ❌ | ✅ | ❌ |
| Message Edit | ✅ | ⚠️ Limited | ❌ | ❌ | ❌ |
| **Tools** | | | | | |
| File Read | ✅ | ✅ | ✅ | ✅ | ✅ |
| File Write | ✅ | ✅ | ✅ | ✅ | ✅ |
| File Edit | ✅ | ✅ | ✅ | ✅ | ✅ |
| Glob | ✅ | ✅ | ✅ | ✅ | ✅ |
| Grep | ✅ | ✅ | ✅ | ✅ | ✅ |
| Bash/Shell | ✅ | ✅ | ✅ | ✅ | ✅ |
| Diff | ✅ | ✅ | ❌ | ✅ | ❌ |
| Web Search | ✅ | ✅ | ✅ | ✅ | ✅ |
| Web Fetch | ✅ | ✅ | ✅ | ✅ | ✅ |
| Clipboard | ✅ | ❌ | ❌ | ❌ | ❌ |
| Memory | ✅ | ✅ | ❌ | ✅ | ❌ |
| Sub-Agent | ✅ | ✅ | ✅ | ❌ | ❌ |
| **Permissions** | | | | | |
| 3-Tier Mode | ✅ | ✅ | ✅ | ✅ | ❌ |
| Granular Rules | ✅ | ✅ | ✅ | ✅ | ❌ |
| Plan Mode | ✅ | ✅ | ❌ | ❌ | ❌ |
| Sandboxed Bash | ✅ | ❌ | ✅ | ❌ | ❌ |
| **UI** | | | | | |
| Markdown Rendering | ✅ | ✅ | ✅ | ✅ | ❌ |
| Syntax Highlighting | ✅ | ✅ | ✅ | ✅ | ❌ |
| Code Blocks | ✅ | ✅ | ✅ | ✅ | ✅ |
| Tables | ✅ | ✅ | ❌ | ❌ | ❌ |
| Status Bar | ✅ | ✅ | ❌ | ❌ | ❌ |
| Progress Indicators | ✅ | ✅ | ✅ | ❌ | ❌ |
| Images in Terminal | ✅ | ❌ | ❌ | ❌ | ❌ |
| **Customization** | | | | | |
| Themes | ✅ | ✅ | ❌ | ❌ | ❌ |
| Font Config | ✅ | ❌ | ❌ | ❌ | ❌ |
| Keybindings | ✅ | ❌ | ❌ | ❌ | ❌ |
| Terminal Wallpaper | ✅ | ❌ | ❌ | ❌ | ❌ |
| **Plugin System** | | | | | |
| External Plugins | ✅ | ❌ | ⚠️ Limited | ✅ | ❌ |
| WASM Runtime | ✅ | ❌ | ❌ | ❌ | ❌ |
| Hook System | ✅ | ✅ | ✅ | ✅ | ❌ |
| Marketplace | ✅ | ❌ | ❌ | ❌ | ❌ |
| SDK/Tooling | ✅ | ❌ | ❌ | ✅ | ❌ |
| **Memory** | | | | | |
| Conversation Memory | ✅ | ✅ | ✅ | ✅ | ✅ |
| Project Memory | ✅ | ❌ | ❌ | ❌ | ❌ |
| User Memory | ✅ | ❌ | ❌ | ❌ | ❌ |
| Semantic Search | ✅ | ❌ | ❌ | ✅ | ❌ |
| Auto-Pruning | ✅ | ✅ | ❌ | ❌ | ❌ |
| **Advanced** | | | | | |
| Agent Planning | ✅ | ✅ | ✅ | ❌ | ❌ |
| Multi-Agent | ✅ | ✅ | ✅ | ❌ | ❌ |
| Workflow Engine | ✅ | ❌ | ❌ | ❌ | ❌ |
| Git Integration | ✅ | ✅ | ✅ | ✅ | ❌ |
| Prompt Templates | ✅ | ❌ | ❌ | ✅ | ❌ |
| **Operations** | | | | | |
| Config Profiles | ✅ | ❌ | ❌ | ❌ | ❌ |
| Doctor/Diagnostics | ✅ | ✅ | ❌ | ✅ | ❌ |
| Update Mechanism | ✅ | ✅ | ❌ | ✅ | ❌ |
| Telemetry (opt-in) | ✅ | ⚠️ Mixed | ❌ | ✅ | ❌ |
| **Release Channels** | | | | | |
| Homebrew | ✅ | ✅ | ❌ | ❌ | ❌ |
| Cargo | ✅ | ❌ | ❌ | ❌ | ❌ |
| npm | ✅ | ✅ | ❌ | ❌ | ❌ |
| AUR | ✅ | ❌ | ✅ | ❌ | ❌ |
| Deb/RPM | ✅ | ❌ | ❌ | ❌ | ❌ |
| Scoop/Winget | ✅ | ❌ | ❌ | ❌ | ❌ |
| Binary Downloads | ✅ | ✅ | ✅ | ✅ | ✅ |

## Legend
- ✅ = Supported / First-class
- ⚠️ = Partial / Limited / Bolted-on
- ❌ = Not supported
- ? = Unknown

## Key Differentiators for Pleiades

1. **True Provider Agnosticism**: Not Anthropic-first with others working via compat layer. Every provider is a first-class citizen.
2. **WASM Plugin System**: External plugins via WebAssembly — full isolation, multiple languages, runtime safety.
3. **Memory System**: Multi-tier (conversation, session, project, user) with semantic search.
4. **Workflow Engine**: Define, share, and run multi-step workflows.
5. **Terminal Customization**: Themes, fonts, keybindings, even wallpapers where supported.
6. **Multi-Format Config**: TOML, JSON, and YAML with profiles and live reload.
7. **Comprehensive Release**: Every major package manager across all platforms.
8. **Open Source from Day One**: MIT licensed with full transparency.
