# Apophy — AI Continuity Engine

Sovereign local AI infrastructure. Zero cloud. Zero fragmentation. Zero silent degradation.

## Architecture

```
toon-core → memory-engine → llm-router → prompt-engine → refine-loop → agent-runtime → api-server
```

7 Rust crates, single responsibility each. See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Quick Start

```bash
# Build
cargo build --workspace

# Test (48 tests across 7 crates)
cargo test --workspace

# Deploy (Proxmox 1-click)
./scripts/deploy.sh
```

## Stack

- **Rust 2021** / Tokio 1.40 / Axum 0.8
- **SQLite WAL** via rusqlite 0.32 (persistent memory)
- **TOON v3** + zstd-12 (compact serialization, -60% tokens)
- **BarqFlow** hybrid DAG workflow engine
- **Thermal guard** — GPU < 80°C or automatic CPU failover

## Models (Rolling Release Local GGUF)

| Model | Role | Resource |
|-------|------|----------|
| FunctionGemma:270m | Orchestration | CPU |
| GLM-4.7-Flash | Vision / Code | GPU 4GB |
| Gemma3:270m | Embedding / Memory | CPU |

## API

```bash
# Health
curl http://localhost:8080/health

# Router info (models + GPU temp)
curl http://localhost:8080/api/v1/router/info

# Memory operations
curl -X POST http://localhost:8080/api/v1/memory/fragments \
  -H 'Content-Type: application/json' \
  -d '{"id":"f1","session_id":"s1","content":"hello"}'

curl http://localhost:8080/api/v1/memory/search?q=hello
```

## Documentation

- [Architecture](docs/ARCHITECTURE.md) — System design and crate map
- [Security](docs/SECURITY.md) — Threat model and mitigations
- [Deployment](docs/DEPLOY.md) — Proxmox, Docker, and API reference

## Development

```bash
# Format
cargo fmt

# Lint
cargo clippy --workspace -- -D warnings

# Audit
cargo audit

# Monitor (GPU temp, memory, disk)
./scripts/monitor.sh
```

## License

MIT
