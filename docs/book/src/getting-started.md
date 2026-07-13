# Getting started

## Build from source

```console
git clone https://github.com/CodWasTaken/Pleiades.git
cd Pleiades
cargo build --release --bin pleiades
```

The binary is written to `target/release/pleiades`. Run `pleiades config init`, then configure at least one provider API key.

```console
pleiades config init
pleiades config set providers.openai.api_key '${OPENAI_API_KEY}'
pleiades config validate
pleiades repl
```

Use `pleiades --help` for the complete command tree. `pleiades --chat` starts a chat session and `pleiades repl --session ID` resumes a saved session.
