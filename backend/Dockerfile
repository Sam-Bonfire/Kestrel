# --- Builder Stage ---
FROM rust:1.79-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    sqlite3 \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Setup cargo-chef to cache dependency builds
RUN cargo install cargo-chef --version ^0.1

# Prepare chef recipe
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Build dependency cache
RUN cargo chef cook --release --recipe-path recipe.json

# Copy remaining source code and build production binary
COPY . .
RUN cargo build --release --bin kestrel-server

# --- Runner Stage ---
FROM debian:bookworm-slim AS runner

# Install runtime dependencies (OpenSSL, SQLite)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

# Create application directory
WORKDIR /app

# Copy compiled binary from builder
COPY --from=builder /app/target/release/kestrel-server /usr/local/bin/kestrel-server

# Expose backend port
EXPOSE 8080

# Run daemon
ENTRYPOINT ["/usr/local/bin/kestrel-server"]
