# ADR 0004: Structured permission rules

## Status

Accepted.

## Context

The original persistent permission configuration used flat `always_allow` and
`always_deny` tool-name lists. That was enough for early tool gating, but it
could not distinguish `cargo test` from `git push`, could not reason about
compound shell commands, and could not explain why a decision was made.

Release 2.2 requires Auto mode to run workspace-safe work without prompts while
still preserving explicit deny rules. That needs a decision layer below the
mode preset.

## Decision

Introduce `pleiades-agent-permissions`, a terminal-independent crate that
evaluates `PermissionRule` values against structured `ToolInvocation` data.
Rules contain a tool matcher, command/path pattern, action, and optional
working-directory, network, MCP server, and MCP tool matchers.

For shell commands, the engine tokenizes quotes, operators, pipelines,
redirection, and command substitution into command clauses. Each clause is
evaluated independently. Deny wins over ask, ask wins over allow, and allow
requires every clause to be covered. Redirection targets, explicit target
paths, working directories, and symlinks are canonicalized against the
workspace in Agent and Auto modes.

CLI and live commands use the same `PermissionService`:

- `/permissions show`
- `/permissions allow <pattern>`
- `/permissions ask <pattern>`
- `/permissions deny <pattern>`
- `/permissions reset`
- `/permissions test <command>`

The external CLI exposes the same operations as `pleiades permissions ...`.

## Consequences

Auto mode can execute explicitly allowed workspace operations without approval
prompts while configured deny rules still block execution. Agent mode can skip
prompts for narrow allow rules but still prompts by default for risky tools.
Plan mode remains a hard read-only boundary, and YOLO removes workspace
confinement while retaining configured deny decisions.

The first user-facing command form creates bash rules. The underlying data
model already includes network, MCP, and plugin-related fields so later MCP and
plugin work can reuse the same policy engine instead of adding separate
permission stacks.
