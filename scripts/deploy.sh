#!/usr/bin/env bash
# Apophy Lab — 1-Click Proxmox Deploy
# Usage: ./scripts/deploy.sh [--container-id 101] [--gpu]
set -euo pipefail

CONTAINER_ID="${1:-101}"
GPU_PASSTHROUGH="${2:-}"
HOSTNAME="apophy-lab"

echo "=== Apophy Lab — Proxmox Deployment ==="

# 1. Build release
echo "[1/5] Building Rust workspace..."
cargo build --release --workspace

# 2. Create Proxmox LXC container
echo "[2/5] Creating Proxmox LXC container ${CONTAINER_ID}..."
pct create "${CONTAINER_ID}" local:vztmpl/ubuntu-24.04-standard_24.04-1_amd64.tar.zst \
  --hostname "${HOSTNAME}" \
  --cores 8 \
  --memory 32768 \
  --net0 name=eth0,bridge=vmbr0,ip=dhcp \
  --features nesting=1

# 3. GPU passthrough (optional)
if [ "${GPU_PASSTHROUGH}" = "--gpu" ]; then
    echo "[3/5] Configuring GPU passthrough..."
    qm set "${CONTAINER_ID}" -args '-device vfio-pci,host=01:00.0'
else
    echo "[3/5] Skipping GPU passthrough (use --gpu flag to enable)"
fi

# 4. Copy binaries
echo "[4/5] Deploying binaries..."
pct start "${CONTAINER_ID}"
pct exec "${CONTAINER_ID}" -- mkdir -p /opt/apophy/data

for binary in api-server; do
    if [ -f "target/release/${binary}" ]; then
        pct push "${CONTAINER_ID}" "target/release/${binary}" "/usr/local/bin/${binary}"
        pct exec "${CONTAINER_ID}" -- chmod +x "/usr/local/bin/${binary}"
    fi
done

# 5. Systemd service
echo "[5/5] Installing systemd service..."
pct exec "${CONTAINER_ID}" -- tee /etc/systemd/system/apophy.service > /dev/null <<'UNIT'
[Unit]
Description=Apophy AI Continuity Engine
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=/usr/local/bin/api-server
WorkingDirectory=/opt/apophy
Environment=RUST_LOG=info
Environment=APOPHY_DB_PATH=/opt/apophy/data/memory.db
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
UNIT

pct exec "${CONTAINER_ID}" -- systemctl daemon-reload
pct exec "${CONTAINER_ID}" -- systemctl enable --now apophy

echo ""
echo "=== Deployment complete ==="
echo "Container: ${CONTAINER_ID} (${HOSTNAME})"
echo "Status: pct exec ${CONTAINER_ID} -- systemctl status apophy"
echo "Logs:   pct exec ${CONTAINER_ID} -- journalctl -u apophy -f"
