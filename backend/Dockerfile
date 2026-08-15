# --- Builder Stage ---
FROM rust:1.85-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    sqlite3 \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Setup cargo-chef to cache dependency builds
RUN cargo install cargo-chef --version ^0.1 --locked

# Prepare chef recipe
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Build dependency cache
RUN cargo chef cook --release --recipe-path recipe.json

# Copy remaining source code and build production binary
COPY . .
RUN cargo build --release --bin backend

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
COPY --from=builder /app/target/release/backend /usr/local/bin/backend

# Expose backend port
EXPOSE 8080

# Run daemon
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:${PORT:-8080}/api/v1/health || exit 1

ENTRYPOINT ["/usr/local/bin/backend"]
