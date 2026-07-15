# Custom commands

Custom commands let a project or user add reusable slash commands without
writing a plugin.

Pleiades loads command files from:

- project scope: `.pleiades/commands/*.toml`
- global scope: `~/.config/pleiades/commands/*.toml`

Each valid file is registered in the same command registry as built-in
commands, so it appears in help, the command palette, and slash autocomplete.
Invalid files are ignored for registration and reported by the service layer
without crashing the workspace.

## Example

`.pleiades/commands/release.toml`:

```toml
description = "Prepare a release"
aliases = ["rel"]
permission = "read"
prompt = "Prepare release {{version}}. Extra notes: {{extra_args|none}}"

[[arguments]]
name = "version"
description = "Version to release"
required = true
```

This creates:

```text
/release 2.1.0 final smoke test
/rel 2.1.0 final smoke test
```

The rendered prompt is submitted through the normal agent runtime. Provider
calls, tool use, verification, permission prompts, and sandbox rules remain the
same as if the user had typed the expanded prompt directly.

## Fields

```toml
name = "release"                # optional; defaults to file stem
path = ["project", "release"]   # optional; defaults to [name]
description = "Prepare release" # shown in help and palette
aliases = ["rel"]               # optional
permission = "read"             # none | read | write | dangerous
provider = "openai"             # metadata for future routing
model = "gpt-5"                 # metadata for future routing
skills = ["release"]            # metadata for future skill binding
workflow = "release"            # metadata for future workflow binding
background = false              # currently runs foreground with a notice
prompt = "Prepare {{version}}"
```

Argument variables use `{{name}}`. Built-in variables are:

- `{{args}}` — all positional arguments
- `{{extra_args}}` — positional arguments beyond declared arguments
- `{{provider}}`
- `{{model}}`
- `{{mode}}`
- `{{command}}`

Defaults use `{{name|fallback}}`.

## Safety

Custom commands are prompt templates. They do not bypass permissions and do not
execute shell commands directly. If a rendered prompt causes the agent to use a
tool, the active mode and permission engine decide whether that tool may run.
