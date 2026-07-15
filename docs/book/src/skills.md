# Skills

Skills are reusable instructions that can be enabled for the live workspace or
headless runs. They are project-aware guidance, not executable plugins.

Pleiades loads skills from:

- project scope: `.pleiades/skills/*.toml`
- global scope: `~/.config/pleiades/skills/*.toml`

Each skill is a TOML file:

```toml
name = "review"
description = "Review changes before completion"
instructions = "Always inspect the final diff and report the validation commands that actually ran."
enabled = true
permissions = []
```

Enabled skills are appended to the engine system prompt under an `Enabled
Skills` section. Disabled skills remain available for inspection and editing but
do not affect agent behavior.

## CLI commands

```bash
pleiades skills list
pleiades skills show review
pleiades skills create review
pleiades skills edit review
pleiades skills enable review
pleiades skills disable review
pleiades skills reload
```

`skills create` writes a project-local skill in `.pleiades/skills/`.
`skills edit` opens `$EDITOR` when set; otherwise it prints the file path.

## Live workspace commands

The same service backs the live slash commands:

```text
/skills list
/skills show review
/skills create review
/skills edit review
/skills enable review
/skills disable review
/skills reload
```

`/skills edit` displays the skill file path inside the workspace. Edit the TOML
file with your editor and run `/skills reload` to refresh the live command
surface.

## Safety

Skills only contribute instructions. They do not execute commands, read files,
or grant permissions on their own. Future custom command and plugin features can
reference skills, but execution remains governed by the permission engine.
