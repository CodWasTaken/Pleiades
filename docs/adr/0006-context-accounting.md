# ADR 0006: Context accounting and compaction

## Status

Accepted.

## Context

Autonomous coding work needs visible context pressure. Before this decision,
Pleiades could truncate or compress conversations internally, but the live
workspace could not show how much context was being used, which sources were
represented, or what compaction had changed.

Provider-specific tokenizers are not yet integrated, so the first version must
be deterministic, provider-independent, and testable while leaving room for
more exact accounting later.

## Decision

Add an engine-owned context accountant that reports approximate token
contribution by source:

- conversation messages;
- tool output;
- memory context;
- compression summaries;
- pinned items;
- detected file, URL, search, and tool sources.

The command registry exposes the live workspace commands:

- `/context status`
- `/context inspect`
- `/context compact`
- `/context pin <file-or-message>`
- `/context unpin <id>`
- `/context sources`

Handlers emit typed effects. The runtime applies those effects against the
current conversation and sends structured documents back to the frontend. The
TUI does not inspect conversation internals, and the engine does not write
directly to terminal output.

Manual compaction removes older non-system messages, summarizes them through
the configured provider with the engine's existing summarization fallback, and
records before/after token estimates in runtime history.

## Consequences

Users can inspect context pressure without leaving the live workspace and can
compact history deliberately. The first implementation uses a deterministic
four-characters-per-token heuristic, so percentages are approximate and may
differ from provider-reported usage. Pins and compression history are runtime
state for this slice; durable pin persistence can be added when session
metadata is expanded.
