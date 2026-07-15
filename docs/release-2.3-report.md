# Release 2.3 milestone report — Checkpoints, Context, Verification

Release 2.3 added recovery points, context visibility, evidence-backed
verification, and doom-loop protection to the live workspace.

## Implemented features

- `/checkpoint create|list|show|restore|delete`, plus `/undo`, `/redo`, and
  `/rewind` entry points.
- Conservative Git-backed checkpoint restore with HEAD checks, untracked-file
  guards, and patch backups.
- `/context status|inspect|compact|pin|unpin|sources` with engine-owned
  approximate token accounting.
- `/verify`, `/test`, `/run <command>`, and `/review` with structured
  verification evidence.
- Background verification execution so checks do not block the runtime actor.
- `agent.max_repeats` and doom-loop detection for repeated identical tool
  failures.
- Strengthened coding-agent prompt language requiring skipped, blocked, or
  failed verification to be reported plainly.

## Changed architecture

- Added `CheckpointStore` to the runtime.
- Added `ContextAccountant` and runtime-owned context pins/compression history.
- Added `VerificationService` and `VerificationReport`.
- Added `DoomLoopDetector` keyed by normalized repeated-failure signals.
- Added ADRs:
  - `docs/adr/0005-runtime-checkpoints.md`
  - `docs/adr/0006-context-accounting.md`
  - `docs/adr/0007-verification-evidence.md`
  - `docs/adr/0008-doom-loop-detection.md`

## Tests executed

For each code PR, local validation ran:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `typos`
- `mdbook build docs/book`

Hosted GitHub Actions passed on every 2.3 PR:

- Lint
- Coverage
- Security Audit
- Spell Check
- Test on Ubuntu, macOS, and Windows

## Observed results

- PR #71 implemented checkpoints and passed all local/hosted checks.
- PR #72 implemented context accounting and passed all local/hosted checks.
- PR #73 implemented verification evidence and passed all local/hosted checks.
- PR #74 implemented doom-loop detection and passed all local/hosted checks.

## Known limitations

- Checkpoints restore tracked staged/unstaged Git diffs; durable non-Git file
  snapshots and untracked-file restoration remain future work.
- Context token counts use a deterministic approximation, not provider-specific
  tokenizers.
- Context pins are runtime state in this release.
- Verification project detection is intentionally small: Rust and Node are
  supported first.
- Doom-loop integration currently stops repeated identical tool failures; the
  detector is ready for more signal types in later releases.

## Security implications

- Checkpoint restore refuses mismatched Git HEADs and unknown untracked files.
- Verification is skipped in Plan mode instead of executing commands under a
  read-only policy.
- Repeated tool failures now stop early with an explicit reason instead of
  consuming the full iteration budget.

## Follow-up issues

- Expand checkpoint restoration for non-Git workspaces.
- Persist context pins in session metadata.
- Add project recipe-driven verification plans.
- Integrate doom-loop signals for patch hashes, repeated reads, stream errors,
  and verification command failures.
