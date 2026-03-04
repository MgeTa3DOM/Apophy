# Apophy — Security

## Design Principles

1. **Local-first** — all inference and data stays on your machine
2. **No `unwrap()`** — all production code uses `Result<T, Error>`
3. **Parameterized queries** — SQL injection prevented via `rusqlite::params!`
4. **Memory safe** — Rust ownership model, no `unsafe` blocks
5. **Minimal dependencies** — audited via `cargo audit` in CI
6. **No secrets in code** — environment variables for all configuration

## Dependency Audit

Automated in CI via `rustsec/audit-check`. Run manually:

```bash
cargo audit
```

## Code Quality Enforcement

- `cargo clippy -- -D warnings` in CI
- `cargo fmt --check` in CI
- Zero `unwrap()` policy (grep audit)
- Tests in every source file
