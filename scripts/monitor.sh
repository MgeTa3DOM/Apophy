#!/usr/bin/env bash
# Apophy — System Monitor
set -euo pipefail

echo "=== Apophy Monitor ==="
echo ""

echo "[System]"
echo "OS:     $(uname -s) $(uname -m)"
echo "Memory: $(free -h 2>/dev/null | awk '/^Mem:/ {print $3 "/" $2}' || echo 'N/A')"
echo "CPUs:   $(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 'N/A')"

echo ""
echo "[Toolchain]"
echo "Rust:   $(rustc --version 2>/dev/null || echo 'not installed')"
echo "UV:     $(uv --version 2>/dev/null || echo 'not installed')"
echo "Bun:    $(bun --version 2>/dev/null || echo 'not installed')"

echo ""
echo "[Database]"
DB_PATH="${APOPHY_DB_PATH:-./data/memory.db}"
if [ -f "${DB_PATH}" ]; then
    echo "Path: ${DB_PATH}"
    echo "Size: $(du -h "${DB_PATH}" | cut -f1)"
else
    echo "No database at ${DB_PATH}"
fi

echo ""
echo "[API Server]"
PORT="${APOPHY_PORT:-8080}"
if curl -sf "http://localhost:${PORT}/health" > /dev/null 2>&1; then
    echo "Status: RUNNING (port ${PORT})"
    curl -s "http://localhost:${PORT}/health"
    echo ""
else
    echo "Status: NOT RUNNING"
fi

echo ""
echo "[Dashboard]"
DASH_PORT="${DASHBOARD_PORT:-3000}"
if curl -sf "http://localhost:${DASH_PORT}/" > /dev/null 2>&1; then
    echo "Status: RUNNING (port ${DASH_PORT})"
else
    echo "Status: NOT RUNNING"
fi
