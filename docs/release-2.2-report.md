# Release 2.2 milestone report

Date: 2026-07-15

Milestone: 2.2 — Safe Autonomous and YOLO Modes

Merged PRs:

- #68 — `feat(policy): split approval and sandbox modes`
- #69 — `feat(permissions): add structured rule engine`

Closed issues:

- #21 — split `ApprovalPolicy` and `SandboxPolicy`
- #22 — `/mode` commands and YOLO warning UX
- #23 — granular permission rules engine
- #24 — `/permissions` commands

## Implemented features

- Split approval behavior from sandbox boundaries with `ApprovalPolicy` and
  `SandboxPolicy`.
- Added `plan`, `agent`, `auto`, and `yolo` presets.
- Added in-TUI YOLO confirmation requiring exact `YOLO` input and persistent
  high-risk status styling.
- Added `pleiades-agent-permissions`, a terminal-independent structured rule
  engine.
- Added `permissions.rules` configuration with `allow`, `ask`, and `deny`
  actions.
- Added `/permissions show`, `/permissions allow`, `/permissions ask`,
  `/permissions deny`, `/permissions reset`, and `/permissions test`.
- Added matching `pleiades permissions ...` CLI commands through shared
  application services.

## Architecture changes

- Runtime permission decisions now evaluate structured rules before mode
  defaults.
- Deny rules take precedence over ask and allow decisions.
- Shell commands are parsed into clauses so compound commands, pipelines, and
  shell operators are evaluated per clause.
- Agent and Auto modes canonicalize working directories, explicit tool paths,
  redirection targets, and symlinks against the workspace boundary.
- The permission service is shared by slash commands and Clap commands, keeping
  TUI and CLI behavior aligned.

## Validation

Local validation:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `typos`
- `mdbook build docs/book`
- `cargo doc --workspace --no-deps`
- Manual isolated CLI smoke test:
  `pleiades permissions deny cargo test *` followed by
  `pleiades permissions test cargo test --workspace`, observed
  `Decision: deny`.

Hosted CI for #69:

- Lint: passed
- Coverage: passed
- Security Audit: passed
- Spell Check: passed
- Test (ubuntu-latest): passed
- Test (macos-latest): passed
- Test (windows-latest): passed

## Known limitations

- The first user-facing permission command form creates bash rules. The rule
  data model already includes network and MCP matchers for later extension
  work.
- YOLO removes workspace path confinement by design, but this implementation
  still honors explicit deny rules.
- `always_allow` and `always_deny` remain as legacy compatibility lists while
  new work should use `permissions.rules`.

## Security implications

Auto mode can run workspace-confined operations without prompts, but explicit
deny rules still block execution. Plan remains a hard read-only boundary.
Command substitution falls back to ask unless an explicit deny rule matches.
Path traversal, symlink escapes, outside working directories, and redirection
outside the workspace are denied in Agent and Auto modes.
