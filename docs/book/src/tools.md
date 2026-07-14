# Tools and permissions

The built-in tools are read, write, edit, bash, glob, grep, diff, search, and fetch. Inspect them with `pleiades tool list` and `pleiades tool info NAME`.

Tools declare read-only, workspace-write, or dangerous permission levels.
Structured permission rules provide durable allow, ask, and deny decisions.
Interactive permission requests are keyboard-driven Ratatui modals showing the
operation, target, reason, and risk, with allow-once, allow-session, deny-once,
and deny-session decisions. The live workspace never falls back to a blocking
stdin question.

## Agent access modes

Approval behavior and sandbox boundaries are independent runtime policies,
presented through four access-mode presets:

- `plan` never prompts and rejects mutations in a read-only sandbox.
- `agent` prompts for risky operations and confines writes to the workspace.
- `auto` never prompts, but remains confined to the workspace and honors deny rules.
- `yolo` never prompts and permits full host access. Use it only in a trusted environment.

Switch modes with `/mode plan`, `/mode agent`, `/mode auto`, or `/mode yolo`,
or launch with `--permission-mode MODE`. The old `unrestricted` spelling is
accepted as a compatibility alias for `yolo`.

Built-in filesystem tools resolve relative paths against the selected workspace. They reject `..` traversal, absolute paths outside the workspace, and symlinks that resolve outside it. Agent-mode shell execution uses a platform sandbox to keep writes inside the workspace; if isolation is unavailable, the call is refused. Plan mode rejects every mutating tool before prompting.

With API-key providers, Pleiades runs built-in tools through the modal permission system above. With `openai-subscription`, Pleiades delegates the task to the official Codex CLI and maps these modes to Codex's read-only, workspace-write, and danger-full-access sandboxes. Codex owns individual delegated tool calls, so Pleiades's own permission modal applies only to tools executed by the Pleiades runtime. The workspace is the launch directory. ChatGPT credentials remain managed by Codex and are never copied into Pleiades.

## Structured rules

Use `/permissions show`, `/permissions allow PATTERN`, `/permissions ask
PATTERN`, `/permissions deny PATTERN`, `/permissions reset`, and
`/permissions test COMMAND` inside the live workspace. The same operations are
available from the CLI as `pleiades permissions ...`.

The current command form creates bash rules. Patterns are glob expressions
matched against each parsed shell command clause, not a raw command prefix.
Compound commands are evaluated clause by clause, so every clause must match an
allow rule before the whole command is allowed. Deny rules take precedence over
ask and allow rules.

```toml
[[permissions.rules]]
tool = "bash"
pattern = "cargo test *"
action = "allow"

[[permissions.rules]]
tool = "bash"
pattern = "git push *"
action = "ask"

[[permissions.rules]]
tool = "bash"
pattern = "rm -rf *"
action = "deny"
```

The engine parses shell operators, pipelines, redirection, quoted arguments,
and command substitution. Redirection targets and explicit tool paths are
canonicalized against the workspace in Agent and Auto modes; symlink escapes,
`..` traversal, and outside working directories are denied before a rule can
allow the operation. Command substitution falls back to ask unless an explicit
deny rule matches. YOLO removes workspace confinement but still evaluates
configured deny rules.
