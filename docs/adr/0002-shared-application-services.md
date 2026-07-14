# ADR 0002 — Shared application services

- **Status:** Accepted
- **Date:** 2026-07-15
- **Depends on:** ADR 0001 command registry and application services

## Context

Provider construction and management logic was repeated across the Clap CLI,
agent engine, model discovery, and legacy REPL. Plugin commands constructed a
`PluginManager` directly, while the live provider picker read configuration
keys itself. These paths produced frontend-specific strings and made it easy
for behavior or secret handling to drift.

In particular, loading interpolated configuration before a mutation can turn
an environment reference such as `${OPENAI_API_KEY}` into its resolved value.
Saving that value would copy a secret into project configuration.

## Decision

Introduce `pleiades-agent-services`, a terminal-independent application service
crate. Services accept configuration roots and return typed reports or domain
errors. They do not print, render Ratatui widgets, or parse Clap arguments.

The first vertical slice provides:

- `ApplicationServices`, the frontend-facing service container;
- `ProviderService` for secret-safe list, info, removal, and configured adapter
  discovery;
- `ProviderFactory`, the canonical adapter constructor;
- `PluginService` for typed plugin discovery reports.

Provider mutations load unexpanded configuration before persistence. Reads
mask API key values. The CLI owns text formatting, while the live workspace
uses the same reports to populate native pickers.

## Consequences

- CLI and TUI provider discovery can no longer drift.
- Application logic becomes testable with temporary configuration roots and no
  terminal.
- Provider/model/plugin command handlers can call these services from the
  command registry in subsequent Release 2.1 slices.
- Existing write operations not yet migrated remain in the CLI temporarily;
  Release 2.1 is not complete until those operations and model discovery use
  the service layer.

## Verification

Unit tests cover sorted provider reports, secret masking, preservation of
environment references during removal, and builtin plugin discovery. CLI flow
tests ensure the existing external command surface remains compatible.
