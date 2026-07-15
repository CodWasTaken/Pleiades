# Sessions

Sessions persist the live conversation so work can be resumed, searched, forked,
or exported later.

```text
/session list
/session search <query>
/session show <id>
/session rename <id> <name>
/session fork [id]
/session resume <id>
/session export <id> [markdown|json]
/session delete <id>
/session ephemeral <on|off>
```

The same management operations are available from the CLI:

```bash
pleiades session list
pleiades session search refresh
pleiades session show <id>
pleiades session rename <id> "New title"
pleiades session fork <id>
pleiades session resume <id>
pleiades session export <id> --format markdown
pleiades session delete <id>
```

Session IDs may be addressed by a unique prefix. If a prefix matches multiple
sessions, Pleiades refuses the operation and asks for a longer prefix.

## Search

Session search matches:

- session id;
- title;
- provider;
- model;
- tags;
- message text.

More structured filters such as branch, changed files, completion status, and
date ranges are planned for later 2.6 slices.

## Forking

`/session fork [id]` creates a new session with a new id and the same message
history. Forking never modifies the parent session. In the live workspace, the
new fork becomes the active conversation.

## Ephemeral mode

`/session ephemeral on` disables session persistence for the current live
process. Runtime save calls become no-ops until `/session ephemeral off` is
used or the process exits.

The headless CLI does not persist an ephemeral preference because the mode is
intended to be process-local. Use the live workspace command when you need it.
