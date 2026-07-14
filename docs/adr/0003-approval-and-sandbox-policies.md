# ADR 0003: Separate approval and sandbox policies

## Status

Accepted for Release 2.2.

## Context

The original `AgentMode` combined two independent decisions: whether an
operation needs confirmation and where it may execute. That made a
workspace-confined unattended mode impossible and treated full host access as
the only way to avoid prompts.

## Decision

The runtime exposes `ApprovalPolicy` (`always`, `on-risk`, `on-failure`,
`never`) and `SandboxPolicy` (`read-only`, `workspace-write`, `full-access`).
User-facing presets map to them as follows:

| Mode | Approval | Sandbox |
| --- | --- | --- |
| plan | never | read-only |
| agent | on-risk | workspace-write |
| auto | never | workspace-write |
| yolo | never | full-access |

Explicit deny rules are checked before either policy. Plan mode rejects
mutating tools without prompting. Agent mode prompts for mutating and dangerous
tools. Auto mode runs allowed tools without prompts while retaining workspace
isolation. YOLO is the only full-host preset. The legacy `unrestricted` input
is accepted as an alias for YOLO but is no longer presented in help.

## Consequences

Approval UX and sandbox enforcement can evolve and be tested separately.
Enabling YOLO still requires a dedicated confirmation flow and persistent
danger indicator; those safeguards are a subsequent Release 2.2 slice and
must land before YOLO is considered complete.
