# ADR 0007: Verification evidence

## Status

Accepted.

## Context

Pleiades must not report successful completion unless it has actually executed
and observed the relevant checks. The agent prompt already required honest
validation, but the live workspace lacked first-class commands for gathering
verification evidence.

## Decision

Add an engine-owned verification service. It detects the project type from the
workspace, inspects the Git diff, builds a small definition-of-done plan, runs
commands with bounded output capture, and returns a structured
`VerificationReport`.

The command registry exposes:

- `/verify` — run the detected definition-of-done plan;
- `/test` — run detected test commands only;
- `/run <command>` — run an explicit command and capture evidence;
- `/review` — open the diff overlay.

The runtime executes verification in a background task and emits typed activity
plus a structured document. Plan mode is read-only, so verification commands
are planned but not executed there. Reports include changed files, diff stats,
planned commands, observed exit status, stdout/stderr snippets, and a truthful
conclusion.

## Consequences

Users get explicit evidence for verification outcomes, and the assistant prompt
now requires skipped, blocked, or failed verification to be stated plainly.
The first implementation supports Rust and Node detection with deterministic
fallback behavior. More project recipes, targeted-test selection, and bounded
repair loops are future Release 2.3/2.5 work.
