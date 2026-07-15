# Hot reload

The live workspace can reload extension definitions without restarting the
terminal session.

Reload currently refreshes:

- custom commands from `.pleiades/commands/*.toml` and global command files
- skill definitions from `.pleiades/skills/*.toml` and global skill files
- MCP configuration read by workspace commands
- plugin reports and trust metadata exposed through workspace commands
- prompt files discovered by command services

Run:

```text
/plugins reload
/mcp reload
/skills reload
```

Each command emits a typed reload event. The runtime rebuilds its command
registry, and the TUI rebuilds its autocomplete and command-palette registry.
This means a newly created custom command can appear in slash completion after a
reload without restarting Pleiades.

Misconfigured extension files are skipped by registration paths and surfaced by
their service reports where available. They must not crash the workspace.

## Limits

Hot reload does not replace a currently running provider request, tool
execution, plugin hook process, or MCP server process. Running work keeps the
state it started with; new commands use the reloaded registry.
