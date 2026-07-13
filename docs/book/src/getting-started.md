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

The binary is written to `target/release/pleiades`. Run `pleiades config init`, then configure at least one provider API key.

```console
pleiades config init
pleiades config set core.default_provider openai
pleiades config set core.default_model gpt-4o
pleiades config set providers.openai.api_key '${OPENAI_API_KEY}'
pleiades config validate
pleiades repl
```

Set the referenced variable in your shell, for example `export OPENAI_API_KEY="..."`. Use `pleiades --help` for the complete command tree. `pleiades --chat` starts a chat session, `pleiades "your prompt"` runs once, and `pleiades repl --session ID` resumes a saved session.
