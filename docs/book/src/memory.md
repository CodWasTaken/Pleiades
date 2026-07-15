# Memory

Memory stores reusable context outside the active transcript. Pleiades keeps
separate session, project, and user tiers and exposes them through the live
workspace and CLI.

```text
/memory show
/memory search <query>
/memory add <text>
/memory forget <id>
/memory refresh
/memory sources
/memory clear
```

CLI equivalents:

```bash
pleiades memory show
pleiades memory search nextest
pleiades memory add "Prefer cargo nextest for Rust tests"
pleiades memory forget <id>
pleiades memory sources
pleiades memory clear
```

Memory entries include:

- source;
- scope;
- creation time;
- confidence;
- last-used time when available;
- project association;
- whether the entry was generated or user-authored.

`memory forget` accepts a full id or a unique prefix. If a prefix is ambiguous,
Pleiades refuses the deletion.

## Persistence

Persistent memory is stored below the platform data directory:

```text
pleiades/memory/session
pleiades/memory/project
pleiades/memory/user
```

`/memory refresh` rebuilds the runtime engine and reloads persistent memory
from disk. `/memory clear` clears all tiers and should be treated as a
destructive action.
