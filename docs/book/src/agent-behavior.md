# Autonomous coding behavior

Pleiades is designed to finish software-engineering tasks, not merely answer questions. For substantial work its system protocol directs the model to:

1. understand the request;
2. inspect repository instructions, conventions, implementation, and tests;
3. form a focused execution plan;
4. make the smallest coherent changes;
5. run relevant formatting, linting, tests, and builds;
6. diagnose practical failures;
7. review the resulting workspace diff;
8. report the cause or goal, files changed, decisions, observed checks, and remaining risk.

The agent is explicitly prohibited from claiming a check passed unless it ran and observed that result. Use `/verify` to gather explicit definition-of-done evidence: Pleiades detects project tooling, inspects the diff, runs bounded checks, and renders command outcomes. If checks are skipped, blocked, or fail, the report must say so instead of implying the task is fully verified. Tool and provider activity is normalized into typed events, so the terminal can show what actually happened separately from the final prose response.

## Provider execution paths

API and OpenAI-compatible providers return normalized text, reasoning summaries, tool calls, usage, finish reasons, and errors. The Pleiades runtime executes their built-in tools, applies permission policy, captures output, records tool results in the conversation, and continues the agent loop.

The `openai-subscription` adapter delegates autonomous execution and ChatGPT authentication to the official Codex CLI. Credentials remain in Codex's store. Pleiades maps provider JSON events into the same activity model and selects Codex's `read-only`, `workspace-write`, or `danger-full-access` sandbox from the active mode. Codex owns individual command execution in this delegated path, so Pleiades cannot display its own per-tool approval modal for those internal calls; the selected sandbox remains the enforced boundary.

## Cancellation and follow-ups

Every task has a cancellation token. `Ctrl+C` interrupts a provider stream, tool wait, or permission wait and records a cancellation event. Switching access mode also cancels work running under the old boundary before applying the new one.

Messages sent while a task runs enter a FIFO queue. The runtime starts each queued follow-up automatically using the updated conversation and emits queue-count changes for the status bar.

The runtime also stops repeated identical failures before they consume the
entire tool-iteration budget. `agent.max_repeats` defaults to `3`; when the
same tool failure repeats that many times, Pleiades emits a clear failure such
as `Stopping: identical failure ... repeated 3 times` and saves the session.

## Evidence and persistence

Sessions save after tool iterations, completion, cancellation, explicit `/save`, and shutdown. At completion, the runtime captures the current Git diff and branch state for in-interface review. Model and tool output is bounded on UTF-8 boundaries, while the conversation viewport renders a moving window so unusually large sessions do not monopolize redraw work.
