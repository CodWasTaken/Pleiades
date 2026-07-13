# Architecture

Pleiades is a Cargo workspace organized around a dependency-light domain core. Provider, tool, persistence, plugin, prompt, workflow, Git, CLI, and TUI crates act as adapters around that core.

The engine coordinates conversations and emits events over Tokio channels. Providers implement one trait for complete and streaming responses. Tools similarly implement a domain trait with explicit permission metadata. This hexagonal boundary keeps provider APIs and terminal concerns out of domain types.

See the repository's [full architecture document](https://github.com/CodWasTaken/Pleiades/blob/master/docs/ARCHITECTURE.md) for crate relationships and design decisions.
