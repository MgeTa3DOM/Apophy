# Contributing to Apophy

## Getting Started

```bash
git clone https://github.com/MgeTa3DOM/Apophy.git
cd Apophy
cargo build --workspace
cargo test --workspace
```

## Development Rules

- All code must pass `cargo test --workspace`
- All code must pass `cargo clippy -- -D warnings`
- All code must be formatted with `cargo fmt`
- No `unwrap()` in production code
- Every file needs a `#[cfg(test)]` module
- Use `tracing` for logging, never `println!`

## Commit Convention

```
feat: add new feature
fix: fix a bug
refactor: restructure code
perf: improve performance
docs: update documentation
```

## Pull Request Process

1. Fork the repository
2. Create a feature branch
3. Write tests for your changes
4. Ensure all tests pass
5. Submit a PR with a clear description

## Architecture

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the system design.

Each crate has a single responsibility. Changes should respect crate boundaries.
