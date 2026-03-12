# ── Stage 1: Build ──────────────────────────────────────────────
FROM rust:1.91-bookworm AS builder

WORKDIR /build

# Copy manifests first for layer caching
COPY Cargo.toml Cargo.lock ./
COPY engine/core/Cargo.toml engine/core/Cargo.toml
COPY engine/sdk/Cargo.toml engine/sdk/Cargo.toml
COPY engine/schema/Cargo.toml engine/schema/Cargo.toml
COPY engine/exchanges/kalshi/Cargo.toml engine/exchanges/kalshi/Cargo.toml
COPY engine/exchanges/polymarket/Cargo.toml engine/exchanges/polymarket/Cargo.toml
COPY engine/exchanges/opinion/Cargo.toml engine/exchanges/opinion/Cargo.toml
COPY engine/exchanges/limitless/Cargo.toml engine/exchanges/limitless/Cargo.toml
COPY engine/exchanges/predictfun/Cargo.toml engine/exchanges/predictfun/Cargo.toml
COPY dashboard/Cargo.toml dashboard/Cargo.toml
COPY sdks/python/Cargo.toml sdks/python/Cargo.toml
COPY sdks/typescript/Cargo.toml sdks/typescript/Cargo.toml

# Create stub lib.rs / main.rs so cargo can resolve the workspace and cache deps
RUN mkdir -p engine/core/src && echo "" > engine/core/src/lib.rs && \
    mkdir -p engine/sdk/src && echo "" > engine/sdk/src/lib.rs && \
    mkdir -p engine/schema/src && echo "fn main() {}" > engine/schema/src/main.rs && \
    mkdir -p engine/exchanges/kalshi/src && echo "" > engine/exchanges/kalshi/src/lib.rs && \
    mkdir -p engine/exchanges/polymarket/src && echo "" > engine/exchanges/polymarket/src/lib.rs && \
    mkdir -p engine/exchanges/opinion/src && echo "" > engine/exchanges/opinion/src/lib.rs && \
    mkdir -p engine/exchanges/limitless/src && echo "" > engine/exchanges/limitless/src/lib.rs && \
    mkdir -p engine/exchanges/predictfun/src && echo "" > engine/exchanges/predictfun/src/lib.rs && \
    mkdir -p dashboard/src && echo "fn main() {}" > dashboard/src/main.rs && \
    mkdir -p sdks/python/src && echo "" > sdks/python/src/lib.rs && \
    mkdir -p sdks/typescript/src && echo "" > sdks/typescript/src/lib.rs

# Pre-build dependencies (this layer is cached unless Cargo.toml/lock changes)
RUN cargo build --release --package px-dashboard 2>/dev/null || true

# Copy real source code
COPY engine/ engine/
COPY dashboard/ dashboard/
COPY sdks/ sdks/

# Touch source files so cargo knows they changed (not the cached stubs)
RUN find engine/ dashboard/ sdks/ -name "*.rs" -exec touch {} +

# Build the dashboard binary
RUN cargo build --release --package px-dashboard

# ── Stage 2: Runtime ───────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary
COPY --from=builder /build/target/release/px-dashboard /app/px-dashboard

# Copy static assets for the web UI
COPY dashboard/static/ /app/static/

EXPOSE 3000

ENV RUST_LOG=info

ENTRYPOINT ["/app/px-dashboard"]
