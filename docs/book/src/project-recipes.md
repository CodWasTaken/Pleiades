# Project recipes

Project recipes give Pleiades a shared way to detect, list, and run common
project commands from both the live workspace and the headless CLI.

```text
/project detect
/project commands
/project run <recipe>
/project verify
```

Headless equivalents are available through:

```bash
pleiades project detect
pleiades project commands
pleiades project run <recipe>
pleiades project verify
```

## Detection

Pleiades currently detects:

- Rust workspaces from `Cargo.toml`, suggesting `format`, `lint`, `test`, and
  `verify`.
- Node projects from `package.json`, suggesting `dev` and `test`.

Detected recipes are conservative defaults. Project configuration can override
them.

## Configuration

Define project-local recipes in `.pleiades/project.toml`:

```toml
[project.commands]
dev = "cargo run"
test = "cargo test --workspace"
lint = "cargo clippy --workspace --all-targets --all-features -- -D warnings"
format = "cargo fmt --all -- --check"
verify = "cargo fmt --all -- --check && cargo clippy --workspace --all-targets --all-features -- -D warnings && cargo test --workspace"
```

Configured recipes with the same name as detected recipes take precedence.

## Verification

`/project verify` runs the configured `verify` recipe when present. If no
explicit `verify` recipe exists, Pleiades composes one from available `format`,
`lint`, and `test` recipes.

Live workspace project commands run through the same verification runtime as
`/run`, so their output is captured as structured activity instead of being
printed directly by the TUI.
