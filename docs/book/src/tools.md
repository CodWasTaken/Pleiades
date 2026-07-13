# Tools and permissions

The built-in tools are read, write, edit, bash, glob, grep, diff, search, and fetch. Inspect them with `pleiades tool list` and `pleiades tool info NAME`.

Tools declare read-only, workspace-write, or dangerous permission levels. `permissions.always_allow` and `always_deny` provide durable decisions; interactive agent sessions can grant or deny individual calls. Keep dangerous tools behind confirmation in shared projects.

## Agent access modes

Interactive sessions support three access modes:

- `plan` keeps the agent read-only.
- `agent` allows changes inside the workspace and is the default.
- `unrestricted` permits access outside the workspace and should be used only in a trusted environment.

Switch modes with `/mode plan`, `/mode agent`, or `/mode unrestricted`, or launch with `--permission-mode MODE`.

With API-key providers, Pleiades runs its built-in tools through the permission system above. With `openai-subscription`, Pleiades delegates the task to the official Codex CLI and maps these modes to Codex's read-only, workspace-write, and danger-full-access sandboxes. The workspace is the directory where Pleiades was launched. ChatGPT credentials remain managed by Codex and are never copied into Pleiades.
