# Contributing to Pleiades

Thank you for considering contributing to Pleiades! We welcome contributions of all kinds.

## Development Philosophy

Pleiades follows a **milestone-based development** approach. Each milestone must be fully completed (all tests passing, documentation updated) before the next begins.

- **Never skip milestones**
- **Never rush into coding**
- **Always work incrementally**
- **Favor maintainability and correctness over speed**

## Getting Started

### Prerequisites

- Rust (stable, edition 2024)
- Cargo

### Setup

```bash
# Fork and clone the repository
git clone https://github.com/yourusername/pleiades.git
cd pleiades

# Build
cargo build

# Run tests
cargo test

# Lint
cargo clippy

# Format
cargo fmt
```

## Development Workflow

1. **Pick a milestone** from [ROADMAP.md](ROADMAP.md) that is ready to work on
2. **Research** — understand the problem space and existing solutions
3. **Design** — write architecture notes before implementing
4. **Implement** — write clean, tested code
5. **Test** — ensure all tests pass
6. **Document** — update relevant documentation
7. **Refactor** — clean up before committing
8. **Commit** — use conventional commits

## Code Style

- Follow Rust standard conventions
- Run `cargo fmt` before committing
- Ensure `cargo clippy` passes with no warnings
- Write documentation for all public APIs
- No dead code or TODOs in commits

## Commit Convention

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

Types:
- `feat` — New feature
- `fix` — Bug fix
- `refactor` — Code refactoring
- `docs` — Documentation changes
- `test` — Test additions/changes
- `perf` — Performance improvements
- `chore` — Build/maintenance tasks
- `style` — Code style changes (formatting, etc.)

Scopes correspond to crate names: `cli`, `core`, `config`, `providers`, `tools`, `engine`, `tui`, `plugins`, `memory`, `workflow`, `git`, `sdk`

Examples:
- `feat(cli): add pleiades config set command`
- `fix(providers): handle rate limiting in anthropic provider`
- `docs(readme): update quick start section`

## Testing

- All new code must have tests
- Run the full test suite before submitting
- Coverage should not decrease

```bash
# Run all tests
cargo test --workspace --all-features

# Run specific test
cargo test test_name

# Check coverage
cargo llvm-cov --workspace --all-features
```

## Pull Request Process

1. Ensure your branch is up to date with `main`
2. Run the full CI suite locally
3. Update documentation if needed
4. Add a changelog entry
5. Submit the PR with a clear description

## Code of Conduct

Please note that this project follows a [Code of Conduct](CODE_OF_CONDUCT.md). Be respectful and constructive in all interactions.

## Questions?

Open a [Discussion](https://github.com/yourusername/pleiades/discussions) or check existing [Issues](https://github.com/yourusername/pleiades/issues).
