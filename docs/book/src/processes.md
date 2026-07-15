# Background processes

The live workspace can supervise background processes such as development
servers, file watchers, and long-running test commands.

Commands:

```text
/process list
/process start <command>
/process logs <id>
/process stop <id>
/process restart <id>
/process attach <id>
```

Headless equivalents exist for discoverability:

```bash
pleiades process list
pleiades process start <command>
pleiades process logs <id>
pleiades process stop <id>
pleiades process restart <id>
pleiades process attach <id>
```

The actual process manager is owned by the live workspace runtime, so headless
commands print guidance rather than starting an orphan process that would die
when the CLI invocation exits.

## Behavior

- Processes are started in the current workspace directory.
- stdout and stderr are captured into bounded per-process logs.
- `/process list` shows id, command, PID, working directory, status, and exit
  code when known.
- `/process stop <id>` terminates the child process.
- `/process restart <id>` stops the old process and starts a replacement with
  the same command and working directory.
- All runtime-owned processes are stopped when the live workspace shuts down.

## Limits

- Attach currently renders the captured log document; it is not a tmux-style
  interactive terminal.
- Process groups and graceful signal escalation are planned follow-up work.
- Process records live for the current workspace session only.
