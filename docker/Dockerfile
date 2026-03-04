# Apophy Lab — Multi-stage Rust builder
# Usage: docker build -f docker/Dockerfile -t apophy-lab .

# ── Stage 1: Build ──────────────────────────────────────────────────────────
FROM rust:1.81-bookworm AS builder

WORKDIR /build

# Cache dependencies layer
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

# Build release
RUN cargo build --release --workspace

# ── Stage 2: Runtime ────────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r apophy && useradd -r -g apophy -d /opt/apophy apophy

WORKDIR /opt/apophy

# Copy binaries from builder
COPY --from=builder /build/target/release/api-server /usr/local/bin/ 2>/dev/null || true

# Create data directory
RUN mkdir -p /opt/apophy/data && chown -R apophy:apophy /opt/apophy

USER apophy

ENV RUST_LOG=info
ENV APOPHY_DB_PATH=/opt/apophy/data/memory.db

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

CMD ["api-server"]
