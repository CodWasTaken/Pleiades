# ADR 0009: MCP client foundation

## Status

Accepted

## Context

Release 2.4 requires Model Context Protocol servers to become ordinary typed
tools in the Pleiades runtime. MCP integration must support local stdio
servers, remote HTTP-style transports, structured schemas, health reporting,
tool filtering, diagnostics, and permission integration without coupling the
terminal UI to protocol details.

The existing architecture already separates the live workspace, command
registry, application services, and runtime. MCP needs the same separation: the
protocol/client layer should not render UI, decide permissions, or mutate
runtime state.

## Decision

Add `pleiades-agent-mcp` as a dedicated crate for MCP protocol and transport
primitives.

The first implementation slice includes:

- JSON-RPC request and response types for `initialize`, `tools/list`, and
  `tools/call`;
- MCP tool descriptors and list results;
- a minimal line-delimited stdio client;
- transport-independent server definitions;
- redacted status objects suitable for overlays, logs, and audit views;
- explicit auth-source types that store environment variable names rather than
  credential values.

Configuration lives in `pleiades-agent-config` under `mcp.servers` so global,
project, and environment overlays can validate MCP definitions before runtime
use. The config layer validates empty commands, invalid remote URLs, zero
timeouts, empty auth environment names, and contradictory tool filters.

## Consequences

The MCP implementation remains independent from Ratatui and provider adapters.
Future work can add HTTP transports, reconnect/backoff, OAuth flows, server
logs, and runtime tool registration without changing the TUI contract.

This slice intentionally does not expose MCP tools to the agent loop yet. That
belongs in the next vertical slice, where `/mcp` workspace commands and the
runtime tool registry can share this foundation.

## Security implications

The foundation stores secret source names, not secret values. User-facing
status labels redact query parameters such as `token`, `key`, `api_key`, and
`access_token`. Permission policy is intentionally not implemented here; MCP
tool exposure must still pass through the main permission engine before any
tool becomes callable by the agent.
