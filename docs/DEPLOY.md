# Apophy — Deployment Guide

## Prerequisites

- Rust 1.81+ (stable)
- Proxmox VE 8.x (for production deployment)
- Docker 24+ (optional, for containerized deployment)

## Local Development

```bash
# Build
cargo build --workspace

# Test
cargo test --workspace

# Run API server
cargo run --bin api-server
```

## Docker Deployment

```bash
# Build image
docker build -f docker/Dockerfile -t apophy-lab .

# Run
docker compose -f docker/docker-compose.yml up -d

# Check health
curl http://localhost:8080/health
```

## Proxmox Deployment

```bash
# 1-click deploy (creates LXC container, copies binaries, installs systemd service)
./scripts/deploy.sh

# With GPU passthrough (RTX 5070)
./scripts/deploy.sh 101 --gpu

# Monitor
./scripts/monitor.sh
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Health check + version |
| GET | `/api/v1/router/info` | Model registry + GPU temp |
| GET | `/api/v1/router/models` | List all models |
| POST | `/api/v1/memory/fragments` | Insert memory fragment |
| GET | `/api/v1/memory/fragments/:id` | Get fragment by ID |
| DELETE | `/api/v1/memory/fragments/:id` | Delete fragment |
| GET | `/api/v1/memory/sessions` | List all sessions |
| GET | `/api/v1/memory/sessions/:id/fragments` | Get session fragments |
| GET | `/api/v1/memory/search?q=query` | Search fragments |
| GET | `/api/v1/memory/count` | Count total fragments |

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `info` | Log level (trace, debug, info, warn, error) |
| `APOPHY_DB_PATH` | `memory.db` | SQLite database path |
| `APOPHY_PORT` | `8080` | API server port |

## Monitoring

```bash
# Real-time logs
journalctl -u apophy -f

# System monitor (GPU temp, memory, disk)
./scripts/monitor.sh
```
