# Apophy — Security Model

## Threat Model

### Attack Surface

| Vector | Mitigation |
|--------|-----------|
| Network exposure | Zero network calls without `--allow-network` flag |
| SQL injection | Parameterized queries only (`rusqlite::params!`) |
| Memory corruption | Rust ownership model — no `unsafe` blocks |
| Supply chain | Minimal dependencies, `cargo audit` in CI |
| GPU side-channel | Thermal guard auto-failover, no shared GPU state |
| Data persistence | SQLite WAL with file permissions, no plaintext secrets |

### Design Principles

1. **Zero cloud** — All inference runs locally on GGUF models
2. **Zero network** — No outbound connections without explicit flag
3. **Zero `unwrap()`** — All code uses `Result<T, AppError>` (enforced by grep audit)
4. **Minimal surface** — Each crate has a single responsibility
5. **Type safety** — Rust's ownership model prevents memory bugs

## Dependency Audit

Run manually:
```bash
cargo audit
```

Automated in CI via `rustsec/audit-check@v2`.

## Code Quality Rules

- No `unwrap()` in production code
- No `println!` in production (use `tracing::info/warn/error`)
- No Python in critical path
- All public functions documented
- Tests in every file (`#[cfg(test)]` module)
