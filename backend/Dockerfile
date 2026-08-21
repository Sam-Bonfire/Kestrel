# --- Builder Stage ---
FROM rust:1.92-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    sqlite3 \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Setup cargo-chef and WASM tools
RUN rustup target add wasm32-wasip1 && \
    cargo install cargo-chef --version ^0.1 --locked && \
    cargo install cargo-component --locked

# Prepare chef recipe
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Build dependency cache (scoped to backend only)
RUN cargo chef cook --release --recipe-path recipe.json --bin kestrel-server

# Copy remaining source code and build production binary + WASM plugins
COPY . .
RUN cargo build --release --bin kestrel-server
RUN cargo component build --release --manifest-path plugins/gmail/Cargo.toml
RUN cargo component build --release --manifest-path plugins/outlook/Cargo.toml

# --- Runner Stage ---
FROM debian:bookworm-slim AS runner

# Install runtime dependencies (OpenSSL, SQLite)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    libssl3 \
    libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

# Create application directory
WORKDIR /app

# Copy compiled binary from builder
COPY --from=builder /app/target/release/kestrel-server /usr/local/bin/kestrel-server
# Copy WASM plugins from builder
COPY --from=builder /app/target/wasm32-wasip1/release/gmail.wasm /app/plugins/gmail.wasm
COPY --from=builder /app/target/wasm32-wasip1/release/outlook.wasm /app/plugins/outlook.wasm

# Expose backend port
EXPOSE 8080

# Run daemon
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:${PORT:-8080}/api/v1/health || exit 1

ENTRYPOINT ["/usr/local/bin/kestrel-server"]
