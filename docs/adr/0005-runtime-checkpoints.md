# ADR 0005: Runtime checkpoints

## Status

Accepted.

## Context

Pleiades needs safe rewind and restore behavior before it can perform larger
autonomous changes confidently. Users must be able to create a named recovery
point, preview it, and restore it without losing unrelated work.

Git repositories are the common case, but Pleiades also supports non-Git
workspaces. The checkpoint system therefore needs a portable metadata record
while using Git patch restoration when Git is available.

## Decision

Add a checkpoint store to the engine runtime. A checkpoint records:

- conversation state and message position;
- active provider, model, and agent mode;
- Git HEAD and branch when available;
- changed file list;
- staged and unstaged binary-safe diffs.

The runtime handles checkpoint commands through typed `AppEffect` values:

- `/checkpoint create [name]`
- `/checkpoint list`
- `/checkpoint show <id>`
- `/checkpoint restore <id> [--confirm]`
- `/checkpoint delete <id>`
- `/undo`
- `/redo`
- `/rewind`

Restore is intentionally conservative. It refuses to restore when the current
Git HEAD differs from the checkpoint or when unknown untracked files are
present. Before applying a checkpoint, it writes a patch backup of the current
staged and unstaged changes under the checkpoint store. The engine then emits
structured workspace and session events instead of writing to the terminal.

## Consequences

Checkpoints now provide a safe foundation for rewind workflows, verification
loops, and future diff-review restoration. The first implementation restores
tracked staged and unstaged changes through Git patches. It records non-Git
checkpoint metadata but does not yet restore arbitrary non-Git file snapshots
or untracked file contents; those remain future work for Release 2.3 hardening.
