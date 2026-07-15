# Diff review

Pleiades includes a structured diff review path for Git workspaces.

Commands:

```text
/diff
/review
/git status
/git diff
/git diff --staged
```

`/git diff` parses the unified diff into files and hunks and renders a review
document with:

- staged or unstaged scope;
- file paths;
- hunk indexes;
- old and new ranges;
- added, removed, and context lines;
- raw diff updates for the diff overlay.

The parser lives in `pleiades-agent-git` so CLI, runtime, and future overlays
can share the same hunk model.

## Hunk restoration

The Git crate exposes `revert_hunk_in_worktree` for simple unstaged hunks. It
replaces the changed hunk range in the worktree with the old hunk lines and is
covered by a regression test that restores file content exactly.

Current limits:

- staged hunk restoration is rejected in this slice;
- binary diffs are not reverted;
- split-view rendering and hunk staging are planned follow-up work.
