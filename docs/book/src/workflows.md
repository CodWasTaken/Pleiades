# Workflows

Workflow definitions may be TOML, YAML, or JSON and are discovered in `.pleiades/workflows` and the global Pleiades configuration directory.

Each step has a name, command, optional argument list, condition, `parallel` flag, timeout in seconds, and retry count. Adjacent parallel steps run as a batch. Conditions support truthy variables, `!name`, `name == value`, and `name != value`. Variables come from `--var name=value`, declared defaults, or the environment.

```console
pleiades workflow create checks
pleiades workflow validate checks
pleiades workflow run checks --var profile=release
```
