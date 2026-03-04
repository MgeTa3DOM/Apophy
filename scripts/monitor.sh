#!/usr/bin/env bash
# Apophy Lab — System Monitor
# Displays GPU temp, memory usage, and service status
set -euo pipefail

echo "=== Apophy Lab Monitor ==="
echo ""

# GPU Temperature
echo "[GPU]"
if command -v nvidia-smi &> /dev/null; then
    nvidia-smi --query-gpu=name,temperature.gpu,utilization.gpu,memory.used,memory.total \
        --format=csv,noheader
elif [ -d /sys/class/drm/card0/device/hwmon ]; then
    for hwmon in /sys/class/drm/card0/device/hwmon/hwmon*; do
        if [ -f "${hwmon}/temp1_input" ]; then
            temp=$(cat "${hwmon}/temp1_input")
            echo "GPU temp: $((temp / 1000))°C"
        fi
    done
else
    echo "No GPU detected (CPU-only mode)"
fi

echo ""

# Memory / Disk
echo "[System]"
echo "Memory: $(free -h | awk '/^Mem:/ {print $3 "/" $2}')"
echo "Disk:   $(df -h /opt/apophy 2>/dev/null | awk 'NR==2 {print $3 "/" $2}' || echo 'N/A')"

echo ""

# SQLite DB size
echo "[Database]"
DB_PATH="${APOPHY_DB_PATH:-/opt/apophy/data/memory.db}"
if [ -f "${DB_PATH}" ]; then
    echo "Size: $(du -h "${DB_PATH}" | cut -f1)"
    echo "WAL:  $(du -h "${DB_PATH}-wal" 2>/dev/null | cut -f1 || echo 'N/A')"
else
    echo "No database found at ${DB_PATH}"
fi

echo ""

# Service status
echo "[Service]"
if systemctl is-active --quiet apophy 2>/dev/null; then
    echo "Status: RUNNING"
    systemctl show apophy --property=MainPID,MemoryCurrent --no-pager 2>/dev/null || true
else
    echo "Status: NOT RUNNING"
fi

echo ""

# Thermal guard check
echo "[Thermal Guard]"
LIMIT=80
if command -v nvidia-smi &> /dev/null; then
    GPU_TEMP=$(nvidia-smi --query-gpu=temperature.gpu --format=csv,noheader 2>/dev/null | head -1)
    if [ "${GPU_TEMP:-0}" -ge "${LIMIT}" ]; then
        echo "WARNING: GPU ${GPU_TEMP}°C >= ${LIMIT}°C — failover to CPU active"
    else
        echo "OK: GPU ${GPU_TEMP:-0}°C < ${LIMIT}°C"
    fi
else
    echo "OK: CPU-only mode (no thermal limit)"
fi
