# Pleiades Architecture

## System Architecture Overview

Pleiades follows a clean hexagonal architecture (ports and adapters) pattern with event-driven communication between subsystems.

```
┌─────────────────────────────────────────────────────┐
│                   CLI / TUI Layer                    │
│  ┌─────────┐  ┌─────────┐  ┌────────┐  ┌─────────┐ │
│  │  Clap   │  │ Ratatui │  │ Stdout │  │  JSON   │ │
│  │ Parser  │  │  TUI    │  │ Output │  │  Output │ │
│  └────┬────┘  └────┬────┘  └────┬───┘  └────┬────┘ │
└───────┼────────────┼─────────────┼────────────┼──────┘
        │            │             │            │
┌───────┴────────────┴─────────────┴────────────┴──────┐
│                   Application Layer                   │
│  ┌──────────┐  ┌──────────┐  ┌────────┐  ┌────────┐ │
│  │  Config  │  │  Engine  │  │ Memory │  │ Plugin │ │
│  │  System  │  │          │  │ System │  │ Manager│ │
│  └──────────┘  └────┬─────┘  └────────┘  └────────┘ │
│                     │                                │
│  ┌──────────┐  ┌────┴─────┐  ┌────────┐  ┌────────┐ │
│  │  Chat    │  │  Agent   │  │  Tool  │  │Workflow│ │
│  │  Engine  │  │  Engine  │  │ System │  │ Engine │ │
│  └──────────┘  └──────────┘  └────────┘  └────────┘ │
└───────┬───────────────────────────────────────────────┘
        │
┌───────┴───────────────────────────────────────────────┐
│                   Domain / Core Layer                  │
│  ┌──────────┐  ┌──────────┐  ┌────────┐  ┌──────────┐│
│  │ Provider │  │  Model   │  │Convers-│  │  Prompt  ││
│  │  Trait   │  │ Registry │  │ ation  │  │  System  ││
│  └──────────┘  └──────────┘  └────────┘  └──────────┘│
└───────┬───────────────────────────────────────────────┘
        │
┌───────┴───────────────────────────────────────────────┐
│                 Infrastructure Layer                   │
│  ┌──────────┐  ┌──────────┐  ┌────────┐  ┌──────────┐│
│  │  HTTP    │  │  File    │  │Process │  │  Crypto  ││
│  │  Client  │  │  System  │  │ Exec   │  │          ││
│  └──────────┘  └──────────┘  └────────┘  └──────────┘│
└───────────────────────────────────────────────────────┘
```

## Module Architecture

### Core Domain Layer

The innermost layer contains pure domain logic with zero external dependencies.

#### Provider Trait
```rust
#[async_trait]
pub trait Provider: Send + Sync {
    fn name(&self) -> &str;
    fn display_name(&self) -> &str;
    fn supports_streaming(&self) -> bool;
    fn supports_tools(&self) -> bool;
    fn supports_vision(&self) -> bool;
    fn default_model(&self) -> &str;
    fn available_models(&self) -> Vec<ModelInfo>;
    async fn chat(&self, request: ChatRequest, config: &Config) -> Result<ChatResponse>;
    async fn chat_stream(&self, request: ChatRequest, config: &Config) -> Result<ChatStream>;
    async fn embed(&self, input: Vec<String>, model: &str) -> Result<EmbeddingResponse>;
}
```

#### Model Registry
```rust
pub struct ModelRegistry {
    models: HashMap<String, ModelEntry>,
    aliases: HashMap<String, String>,
}

pub struct ModelEntry {
    pub id: String,
    pub provider: String,
    pub capabilities: ModelCapabilities,
    pub context_window: usize,
    pub max_output_tokens: usize,
    pub pricing: Pricing,
}
```

#### Conversation
```rust
pub struct Conversation {
    pub id: String,
    pub messages: Vec<Message>,
    pub metadata: ConversationMetadata,
    pub config: ConversationConfig,
}

pub enum Message {
    System { content: String },
    User { content: ContentBlock },
    Assistant { content: ContentBlock, reasoning: Option<String> },
    Tool { name: String, input: serde_json::Value, result: String },
}

pub enum ContentBlock {
    Text(String),
    Image { mime_type: String, data: Vec<u8> },
    ToolCall { id: String, name: String, input: serde_json::Value },
    ToolResult { id: String, content: String },
}
```

### Application Layer

The application layer orchestrates domain objects and coordinates workflows.

#### Engine
The central coordinator that ties together providers, tools, and conversation management.
- Processes user input
- Routes to appropriate handler (chat, command, tool)
- Manages conversation lifecycle
- Coordinates streaming and rendering

#### Config System
Multi-level configuration with layering:
1. Defaults (hardcoded safe defaults)
2. Global config (`~/.config/pleiades/config.toml`)
3. Project config (`./.pleiades/config.toml`)
4. Environment variables (`PLEIADES_*`)
5. CLI flags (highest priority)

#### Tool System
Interface-driven tool architecture:
```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> serde_json::Value;
    fn is_readonly(&self) -> bool;
    fn is_concurrency_safe(&self) -> bool;
    fn permission_level(&self) -> PermissionLevel;
    async fn execute(&self, input: serde_json::Value, ctx: &ToolContext) -> Result<ToolResult>;
}
```

#### Memory System
Multi-tier memory architecture:
1. **Working Memory**: Current conversation context
2. **Session Memory**: Recent conversations within a session
3. **Project Memory**: Persistent knowledge about the current project
4. **User Memory**: Long-term user preferences and patterns

#### Plugin Manager
WASM-based plugin system:
- Plugin manifest with versioning and dependencies
- Permission declarations
- Hook system for extension points
- Lifecycle management (install, update, remove, enable, disable)

### Infrastructure Layer

#### HTTP Client
Generic HTTP client with:
- Retry with exponential backoff
- Connection pooling
- Timeout management
- Proxy support
- Rate limiting

#### File System
Safe file operations with:
- Path traversal protection
- Binary file detection
- Large file handling
- Encoding detection

#### Process Execution
Sandboxed process execution:
- Configurable timeout
- Resource limits
- Working directory isolation
- Environment variable filtering

## Data Flow

### Chat Flow
```
User Input
  → CLI Parser (clap)
  → Config Resolution
  → Engine.process_input()
      → If command: route to command handler
      → If chat: 
          → Conversation.add_message()
          → Provider.chat_stream()
          → Stream response
              → Render tokens (TUI/Stdout)
              → Handle tool calls
                  → Permission check
                  → Tool.execute()
                  → Conversation.add_tool_result()
                  → Continue or return
```

### Tool Execution Flow
```
LLM decides to call tool
  → Tool call extracted from stream
  → Permission check (mode + rules)
      → If denied: return error message
      → If ask: prompt user for approval
  → Execute tool with timeout
  → Process result (truncation, summarization)
  → Return result to LLM
  → LLM continues generation
```

### Plugin Loading Flow
```
Plugin directory scan
  → Read manifest (pleiades.toml)
  → Validate version compatibility
  → Check permissions
  → Load WASM module
  → Register hooks and tools
  → Plugin is active
```

## Event System

```rust
pub enum Event {
    MessageAdded { message: Message },
    ToolCalled { tool: String, input: Value },
    ToolCompleted { tool: String, result: Result<ToolResult> },
    TokenStreamed { token: String },
    Error { error: Error },
    ConfigChanged { key: String, old: Option<Value>, new: Value },
    PluginLoaded { name: String, version: String },
    PluginUnloaded { name: String },
    SessionStarted { id: String },
    SessionEnded { id: String },
}
```

## Configuration File Format

### Global Config (`~/.config/pleiades/config.toml`)
```toml
[core]
default_provider = "anthropic"
default_model = "claude-sonnet-4"
theme = "catppuccin-mocha"
verbose = false

[providers.anthropic]
api_key = "${ANTHROPIC_API_KEY}"  # Environment variable reference
base_url = "https://api.anthropic.com"

[providers.openai]
api_key = "${OPENAI_API_KEY}"
base_url = "https://api.openai.com/v1"

[models]
default = "claude-sonnet-4"
aliases.opus = "claude-opus-4"
aliases.gpt4 = "gpt-4o"

[plugins]
enabled = ["git-workflow", "web-search"]
path = ["~/.config/pleiades/plugins", "./.pleiades/plugins"]

[permissions]
mode = "default"  # allow, ask, deny
always_allow = ["read", "glob", "grep"]
always_deny = ["rm -rf /"]
```

## Directory Structure

```
~/.config/pleiades/
├── config.toml           # Global configuration
├── profiles/             # Named configuration profiles
│   ├── default.toml
│   └── work.toml
├── plugins/              # Installed plugins
│   ├── git-workflow/
│   └── web-search/
├── themes/               # Custom themes
│   └── my-theme.toml
├── sessions/             # Session storage
│   └── *.jsonl
└── memory/               # Long-term memory
    ├── vectors.db
    └── summaries.db

.pleiades/                # Project-local config (in repo)
├── config.toml
└── plugins/
```

## Security Architecture

### Permission Levels
```rust
pub enum PermissionLevel {
    ReadOnly,       // Safe read operations
    WorkspaceWrite, // Write within project directory
    Dangerous,      // Shell execution, network, etc.
}
```

### Approval Modes
```rust
pub enum ApprovalMode {
    Auto,    // Auto-approve allowed operations
    Ask,     // Always ask before dangerous operations
    Deny,    // Deny all dangerous operations
    Plan,    // Plan mode - no execution
}
```

### Credential Storage
- API keys stored in config with env var references
- Optional OS keyring integration
- Encrypted at rest via system keychain
- Never logged or exposed in error messages

## Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| Cold start | < 100ms | Time from command to ready |
| First token | < 50ms | Time from request to first streamed token |
| Memory usage | < 50MB idle | RSS after startup |
| Plugin load | < 10ms per plugin | Time to validate and register |
| Config parse | < 5ms | Time to load and merge all config levels |
| Tool execution | < 100ms overhead | Time beyond the actual tool operation |
