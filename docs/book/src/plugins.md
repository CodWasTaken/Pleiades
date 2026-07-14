# Plugins and hooks

Plugins are directories containing `plugin.json`. Install a local plugin with `pleiades plugin install PATH`, then use `plugin list`, `enable`, `disable`, and `uninstall`.

The same operations are available without leaving the live workspace:

```text
/plugins list
/plugins info <id>
/plugins install <path>
/plugins uninstall <id>
/plugins enable <id>
/plugins disable <id>
/plugins update <id>
/plugins permissions <id>
/plugins reload
```

These commands call the same application service as the Clap commands.
Information and permission documents include the plugin source, executable
hooks, and the permission boundary requested by each tool.

External plugins installed from local directories remember their canonical
source path. `pleiades plugin update ID` and `/plugins update ID` validate a
fresh copy in a staging directory before replacing the installed version. A
malformed source leaves the active copy unchanged. Plugins installed before
source tracking was introduced must be reinstalled once before they can be
updated.

The current runtime supports shell hooks for `PreToolUse`, `PostToolUse`, and `PostToolUseFailure`. Hook input is provided as JSON on standard input with contextual environment variables. Treat third-party hook commands as executable code and inspect them before installation.
