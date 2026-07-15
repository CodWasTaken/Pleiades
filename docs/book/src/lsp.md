# Language services

Pleiades includes a shared language-service foundation used by both the CLI and
the live workspace.

Commands:

```text
/lsp status
/lsp servers
/lsp restart
/lsp diagnostics
/lsp symbols <query>
```

Headless equivalents:

```bash
pleiades lsp status
pleiades lsp servers
pleiades lsp restart
pleiades lsp diagnostics
pleiades lsp symbols <query>
```

## Current Rust support

For Rust workspaces, Pleiades detects `Cargo.toml`, reports `rust-analyzer`
availability, and runs diagnostics through:

```bash
cargo check --message-format=json
```

Compiler messages are normalized into LSP-compatible diagnostics using
`lsp-types`, so future persistent language-server clients can feed the same
runtime and UI documents.

Symbol search currently scans Rust source files for common definitions such as
functions, structs, enums, and traits. The scan skips `.git` and `target`.

## Limits

- Persistent JSON-RPC LSP server processes are not started yet.
- `/lsp restart` reports that no persistent server exists in this slice.
- Diagnostics may write normal Cargo build artifacts under `target/`.
- Non-Rust language servers are planned follow-up work.
