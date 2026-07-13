# Tools and permissions

The built-in tools are read, write, edit, bash, glob, grep, diff, search, and fetch. Inspect them with `pleiades tool list` and `pleiades tool info NAME`.

Tools declare read-only, workspace-write, or dangerous permission levels. `permissions.always_allow` and `always_deny` provide durable decisions; interactive agent sessions can grant or deny individual calls. Keep dangerous tools behind confirmation in shared projects.
