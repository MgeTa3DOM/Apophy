#!/usr/bin/env bash
# Apophy — Universal Deploy
# Pure Rust + UV + Bun. No Docker required.
# Usage: ./scripts/deploy.sh [--release]
set -euo pipefail

BUILD_MODE="debug"
[[ "${1:-}" = "--release" ]] && BUILD_MODE="release"

echo "=== Apophy — Deploy ==="
echo ""

# 1. Rust
echo "[1/4] Building Rust workspace..."
if [ "${BUILD_MODE}" = "release" ]; then
    cargo build --release --workspace
else
    cargo build --workspace
fi
cargo test --workspace --quiet
echo "  Rust: OK"

# 2. UV (Python fine-tune)
echo "[2/4] Setting up UV environment..."
if command -v uv &> /dev/null; then
    (cd finetune && uv sync --quiet 2>/dev/null) && echo "  UV: OK" || echo "  UV: skipped (install deps with 'cd finetune && uv sync')"
else
    echo "  UV: not installed (install: curl -LsSf https://astral.sh/uv/install.sh | sh)"
fi

# 3. Bun (Dashboard)
echo "[3/4] Setting up Bun dashboard..."
if command -v bun &> /dev/null; then
    (cd dashboard && bun install --silent 2>/dev/null) && echo "  Bun: OK" || echo "  Bun: skipped (install deps with 'cd dashboard && bun install')"
else
    echo "  Bun: not installed (install: curl -fsSL https://bun.sh/install | bash)"
fi

# 4. Data directory
echo "[4/4] Preparing data directory..."
mkdir -p data models
echo "  Data: OK"

echo ""
echo "=== Ready ==="
echo ""
echo "  API Server:  cargo run --bin api-server"
echo "  Dashboard:   cd dashboard && bun dev"
echo "  Fine-tune:   cd finetune && uv run python train.py --help"
echo "  Monitor:     ./scripts/monitor.sh"
