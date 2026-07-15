# MCP servers

Pleiades has a shared MCP management layer used by the CLI and live workspace.
The current Release 2.4 slice manages configured servers and tool exposure
filters. Runtime connections, live logs, reconnects, OAuth flows, and schema
discovery build on this foundation in the next MCP slices.

## Live workspace commands

```text
/mcp
/mcp list
/mcp info <id>
/mcp add
/mcp remove <id>
/mcp enable <id>
/mcp disable <id>
/mcp auth <id>
/mcp logout <id>
/mcp tools <id>
/mcp tool-info <server> <tool>
/mcp reload
/mcp restart <id>
/mcp logs <id>
/mcp debug <id>
```

`/mcp add`, `/mcp auth`, `/mcp logout`, `/mcp restart`, `/mcp logs`, and
`/mcp debug` open the native MCP manager overlay. The overlay is intentionally
read-only until runtime connections and log streams are wired.

## Headless CLI commands

```bash
pleiades mcp list
pleiades mcp info docs
pleiades mcp disable docs
pleiades mcp enable docs
pleiades mcp remove docs
pleiades mcp tools docs
pleiades mcp tool-info docs search
pleiades mcp reload
```

Headless commands read the latest configuration on each invocation. Secrets are
not resolved for display; query parameters such as `token`, `key`, `api_key`,
and `access_token` are redacted.

## Configuration

MCP servers live under `mcp.servers`:

```toml
[mcp.servers.docs]
transport = "streamable-http"
url = "https://example.test/mcp"
timeout_secs = 30
tool_allowlist = ["search"]

[mcp.servers.docs.auth]
type = "bearer"
token_env = "DOCS_MCP_TOKEN"
```

Pleiades stores environment variable names for credentials, not credential
values. Tool allowlists and denylists are configuration filters only until live
schema discovery is connected.
