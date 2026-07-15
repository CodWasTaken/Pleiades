# Configuration reference

Configuration is merged in this order: built-in defaults, global configuration, project configuration, environment variables, then CLI flags. Global files live under `~/.config/pleiades`; project files live under `.pleiades`. TOML, JSON, and YAML are supported. `${NAME}` interpolates an environment variable.

| Key | Type | Default |
|---|---|---|
| `core.default_provider` | string? | unset |
| `core.default_model` | string? | unset |
| `core.theme` | string? | `seven-sisters` |
| `core.verbose`, `core.debug` | boolean | `false` |
| `core.max_tokens` | integer? | `4096` |
| `core.temperature` | float? | `0.7` |
| `core.auto_update` | boolean | `true` |
| `core.log_level` | string | `info` |
| `providers.<name>.api_key` | string? | unset |
| `providers.<name>.base_url` | string? | provider default |
| `providers.<name>.headers` | string map | empty |
| `providers.<name>.organization_id` | string? | unset |
| `providers.<name>.max_retries` | integer | `3` |
| `providers.<name>.timeout_secs` | integer | `120` |
| `models.aliases` | string map | empty |
| `models.default` | string? | unset |
| `plugins.enabled` | string list | empty |
| `plugins.paths` | string list | `~/.pleiades/plugins` |
| `plugins.settings` | nested string map | empty |
| `plugins.sandbox` | boolean | `false` (reserved for a future sandbox runtime) |
| `mcp.servers.<id>.transport` | `stdio`, `http`, or `streamable-http` | required per server |
| `mcp.servers.<id>.command` | string | required for `stdio` |
| `mcp.servers.<id>.args` | string list | empty |
| `mcp.servers.<id>.env` | string map | empty |
| `mcp.servers.<id>.url` | string | required for remote transports |
| `mcp.servers.<id>.auth` | auth source | unset |
| `mcp.servers.<id>.enabled` | boolean | `true` |
| `mcp.servers.<id>.timeout_secs` | integer | `30` |
| `mcp.servers.<id>.tool_allowlist` | string list | empty |
| `mcp.servers.<id>.tool_denylist` | string list | empty |
| `permissions.always_allow` | string list | empty |
| `permissions.always_deny` | string list | empty |
| `permissions.rules` | array of permission rules | empty |
| `permissions.ask_always` | boolean | `true` |
| `permissions.grant_duration_minutes` | integer | `60` |
| `session.context_size` | integer | `100` |
| `session.auto_save_interval_secs` | integer? | `60` |
| `session.history_dir` | string? | unset |
| `session.max_concurrent` | integer | `10` |
| `session.compress_history` | boolean | `false` |
| `session.ephemeral` | boolean | `false` |
| `display.style` | `plain`, `rich`, or `minimal` | `rich` |
| `display.syntax_highlighting` | boolean | `true` |
| `display.show_token_usage`, `display.show_timing` | boolean | `false` |
| `display.output_width` | integer | `0` (automatic) |
| `display.show_progress` | boolean | `true` |
| `agent.default_persona` | string? | unset |
| `agent.system_prompt_prefix` | string? | unset |
| `agent.default_tools` | string list | empty |
| `agent.max_tool_iterations` | integer | `25` |
| `agent.max_repeats` | integer | `3` |
| `agent.auto_edit` | boolean | `false` |

Never commit expanded secrets. Prefer `${OPENAI_API_KEY}` and equivalent environment references. Plugin hooks are ordinary child processes, not sandboxed; only install manifests you trust.

MCP servers are configured under `mcp.servers`. Stdio servers define a command,
arguments, and optional environment variables:

```toml
[mcp.servers.filesystem]
transport = "stdio"
command = "mcp-server-filesystem"
args = ["."]
timeout_secs = 30
tool_allowlist = ["read_file", "list_directory"]
```

Remote servers use `http` or `streamable-http` and may point authentication at
environment variables. Pleiades stores the variable names, not token values:

```toml
[mcp.servers.remote-docs]
transport = "streamable-http"
url = "https://example.test/mcp"
timeout_secs = 30

[mcp.servers.remote-docs.auth]
type = "bearer"
token_env = "REMOTE_DOCS_MCP_TOKEN"
```

The special `providers.openai-subscription` entry does not contain an API key. Authentication remains in the official Codex CLI credential store and can be checked with `pleiades auth status`. Run `pleiades setup` instead of editing this entry manually.

Interactive agent sandbox modes are selected with `--permission-mode
plan|agent|auto|yolo` or `/mode plan|agent|auto|yolo`. `agent` is the default.
`auto` runs without prompts inside the workspace. `yolo` removes workspace
isolation after explicit confirmation and should only be used in an already
isolated environment. The old `unrestricted` spelling remains accepted as a
compatibility alias for `yolo`.

Structured permission rules have `tool`, `pattern`, and `action` fields, plus
optional `cwd`, `network`, `mcp_server`, and `mcp_tool` matchers. `action` is
`allow`, `ask`, or `deny`. Rules can be edited directly or managed with
`pleiades permissions allow|ask|deny|show|test|reset`.

Built-in themes are `seven-sisters`, `andromeda`, `orion`, `event-horizon`, `solar-wind`, `high-contrast`, and `ascii`. Legacy `catppuccin-mocha`, `dracula`, and `tokyo-night` values remain accepted as compatibility aliases.
