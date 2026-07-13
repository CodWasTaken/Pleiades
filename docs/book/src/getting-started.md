# Getting started

## Install a release

On Linux and macOS, the installer downloads a release archive and verifies its SHA-256 checksum:

```console
curl -fsSL https://raw.githubusercontent.com/CodWasTaken/Pleiades/master/install.sh | sh
export PATH="$HOME/.local/bin:$PATH"
```

The crates.io package named `pleiades` is an unrelated project. Do not use `cargo install pleiades`. A Rust source install of this repository is available with:

```console
cargo install --git https://github.com/CodWasTaken/Pleiades pleiades-agent
```

The published package family uses the `pleiades-agent` namespace because the unqualified `pleiades` and `pleiades-core` names belong to unrelated crates. The package `pleiades-agent` installs an executable named `pleiades`:

```console
cargo install pleiades-agent
```

## Build a checkout

```console
git clone https://github.com/CodWasTaken/Pleiades.git
cd Pleiades
cargo build --release --bin pleiades
```

The binary is written to `target/release/pleiades`. Run the guided setup and choose either ChatGPT subscription sign-in or usage-based OpenAI API access:

```console
pleiades setup
pleiades doctor
pleiades
```

For ChatGPT subscription access, install the official Codex CLI; Pleiades delegates `codex login` and never reads its token cache. For API access, setup stores `${OPENAI_API_KEY}` and asks you to export a newly created Platform API key. ChatGPT and API billing are separate.

Use `pleiades --help` for the complete command tree. Running `pleiades` or `pleiades chat` starts the live full-screen coding workspace once configured, `pleiades "your task"` runs an autonomous task once, and `pleiades chat --session ID` resumes a saved session.

The directory where Pleiades starts is the workspace root. Sessions default to `agent` mode. Use `/mode plan` for read-only analysis or start with `--permission-mode plan`. Press `F1` inside the workspace for searchable keyboard help.

On Linux, built-in API-provider shell commands in Agent mode require Bubblewrap (`bwrap`) for write isolation. On macOS they use `sandbox-exec`. If an Agent-mode sandbox is unavailable, Pleiades refuses the shell call instead of silently running it without isolation; `unrestricted` remains an explicit opt-in.
