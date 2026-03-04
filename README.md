# Apophy — AI Continuity Engine

Persistent AI memory and intelligent workflow orchestration.
Every session leaves a trace. Every trace improves the next.

## Why Apophy

- **Memory that persists** — SQLite WAL ensures zero inter-session fragmentation
- **Token-efficient** — TOON v3 serialization saves 60% tokens vs JSON
- **Self-improving** — AZR self-play refines prompts automatically
- **Backend-agnostic** — pluggable `InferenceBackend` trait for any LLM runtime
- **Production-ready** — 48 tests, CI/CD, zero `unwrap()`, full docs

## Stack

| Layer | Technology | Role |
|-------|-----------|------|
| Core Engine | **Rust 2021** / Tokio 1.40 / Axum 0.8 | API, memory, workflows |
| Dashboard | **Bun** + Hono + TypeScript | Real-time monitoring UI |
| Fine-Tuning | **UV** + Unsloth + QLoRA | Self-improvement from memory |
| Database | **SQLite WAL** (rusqlite 0.32) | Persistent fragments |
| Serialization | **TOON v3** + zstd-12 | -60% tokens vs JSON |
| Workflow | **petgraph** DAG | BarqFlow task orchestration |

## Quick Start

```bash
git clone https://github.com/MgeTa3DOM/Apophy.git
cd Apophy

# Build & test (Rust)
cargo build --workspace
cargo test --workspace

# Full setup (Rust + UV + Bun)
./scripts/deploy.sh --release

# Run
cargo run --bin api-server        # API on :8080
cd dashboard && bun dev           # Dashboard on :3000
```

## Architecture

```
toon-core → memory-engine → llm-router → prompt-engine → refine-loop → agent-runtime → api-server
                                                                                            ↕
                                                                                      dashboard (Bun)
                                                                                            ↕
                                                                                      finetune (UV)
```

| Crate | What it does |
|-------|-------------|
| `toon-core` | Token-efficient serialization (zstd-12, -60% tokens) |
| `memory-engine` | Persistent memory with SQLite WAL + TOON auto-compress |
| `llm-router` | Pluggable model registry and task-based routing |
| `prompt-engine` | AZR self-play prompt refinement with 5 mutation types |
| `refine-loop` | Autorecursive convergence loop with configurable strategy |
| `agent-runtime` | BarqFlow DAG workflow engine with topological execution |
| `api-server` | RESTful API (10 endpoints) via Axum |

## API

```bash
curl http://localhost:8080/health
curl http://localhost:8080/api/v1/router/models
curl http://localhost:8080/api/v1/memory/search?q=hello

curl -X POST http://localhost:8080/api/v1/memory/fragments \
  -H 'Content-Type: application/json' \
  -d '{"id":"f1","session_id":"s1","content":"hello world"}'
```

## Fine-Tuning

Train your own model from conversation memory:

```bash
cd finetune
uv sync
uv run python train.py export --db ../data/memory.db
uv run python train.py train --dataset data/train.jsonl --output ../models/apophy-v1
```

## Documentation

- [Architecture](docs/ARCHITECTURE.md) — System design
- [Security](docs/SECURITY.md) — Security model
- [Deployment](docs/DEPLOY.md) — Setup guide + API reference
- [Contributing](CONTRIBUTING.md) — How to contribute

## License

[MIT](LICENSE)
