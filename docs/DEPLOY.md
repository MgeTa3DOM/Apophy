# Apophy — Deployment Guide

## Prerequisites

- Rust 1.81+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- UV (`curl -LsSf https://astral.sh/uv/install.sh | sh`) — for fine-tuning
- Bun (`curl -fsSL https://bun.sh/install | bash`) — for dashboard

## Quick Deploy

```bash
./scripts/deploy.sh --release
```

This builds Rust, sets up UV + Bun, and prepares data directories.

## Running

```bash
# API Server (Rust)
RUST_LOG=info cargo run --bin api-server

# Dashboard (Bun)
cd dashboard && bun dev

# Fine-tune (UV)
cd finetune && uv run python train.py --help

# Monitor
./scripts/monitor.sh
```

## API Reference

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Health check |
| GET | `/api/v1/router/info` | Model registry info |
| GET | `/api/v1/router/models` | List models |
| POST | `/api/v1/memory/fragments` | Insert fragment |
| GET | `/api/v1/memory/fragments/:id` | Get by ID |
| DELETE | `/api/v1/memory/fragments/:id` | Delete |
| GET | `/api/v1/memory/sessions` | List sessions |
| GET | `/api/v1/memory/sessions/:id/fragments` | Session fragments |
| GET | `/api/v1/memory/search?q=query` | Search |
| GET | `/api/v1/memory/count` | Count |

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `info` | Log level |
| `APOPHY_DB_PATH` | `memory.db` | SQLite path |
| `APOPHY_PORT` | `8080` | API server port |
| `DASHBOARD_PORT` | `3000` | Dashboard port |
| `APOPHY_API_URL` | `http://localhost:8080` | API URL for dashboard |
