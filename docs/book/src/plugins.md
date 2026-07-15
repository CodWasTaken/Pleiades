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
/plugins trust <id>
/plugins untrust <id>
/plugins reload
```

These commands call the same application service as the Clap commands.
Information and permission documents include the plugin source, executable
hooks, lifecycle commands, tools, custom commands, requested permissions,
accessible paths declared by the manifest, network declaration, environment
variables, checksum, and signature when provided.

External plugins install disabled and must be explicitly trusted before they
can be enabled:

```bash
pleiades plugin install ./my-plugin
pleiades plugin info my-plugin-external
pleiades plugin trust my-plugin-external
pleiades plugin enable my-plugin-external
```

In the live workspace, `/plugins enable <id>` renders the trust report instead
of enabling an untrusted external plugin. After review, run `/plugins trust
<id>` and then enable it. `/plugins untrust <id>` revokes trust and disables
the plugin.

External plugins installed from local directories remember their canonical
source path. `pleiades plugin update ID` and `/plugins update ID` validate a
fresh copy in a staging directory before replacing the installed version. A
malformed source leaves the active copy unchanged. Plugins installed before
source tracking was introduced must be reinstalled once before they can be
updated.

Plugin manifests may declare optional trust metadata:

```json
{
  "requestedPaths": ["./src", "~/.config/example"],
  "envVars": ["EXAMPLE_TOKEN"],
  "network": "api.example.com",
  "checksum": "sha256:...",
  "signature": "minisign:..."
}
```

The current runtime supports shell hooks for `PreToolUse`, `PostToolUse`, and
`PostToolUseFailure`. Hook input is provided as JSON on standard input with
contextual environment variables. Treat third-party hook commands as executable
code and inspect them before installation.
