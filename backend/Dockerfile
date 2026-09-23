# --- Landing Stage (portfolio + download page served by the backend) ---
FROM node:22-bookworm-slim AS landing

RUN corepack enable

WORKDIR /app

# Landing page identity (override with --build-arg so forks ship the page
# with their own repo/image/contact; unset values fall back to placeholders).
ARG KESTREL_VERSION=''
ARG KESTREL_REPO=''
ARG KESTREL_DOCKER_IMAGE=''
ARG KESTREL_CONTACT_EMAIL=''
ARG KESTREL_IOS_TESTFLIGHT=''
ENV KESTREL_VERSION=$KESTREL_VERSION \
    KESTREL_REPO=$KESTREL_REPO \
    KESTREL_DOCKER_IMAGE=$KESTREL_DOCKER_IMAGE \
    KESTREL_CONTACT_EMAIL=$KESTREL_CONTACT_EMAIL \
    KESTREL_IOS_TESTFLIGHT=$KESTREL_IOS_TESTFLIGHT

# Install workspace deps for the landing page and its shared library
COPY package.json pnpm-workspace.yaml pnpm-lock.yaml ./
COPY frontend-shared/package.json frontend-shared/package.json
COPY frontend-landing/package.json frontend-landing/package.json
COPY frontend-mail/package.json frontend-mail/package.json
COPY frontend-calendar/package.json frontend-calendar/package.json
RUN pnpm install --filter frontend-landing...
COPY frontend-shared/ frontend-shared/
COPY frontend-landing/ frontend-landing/
COPY frontend-mail/ frontend-mail/
COPY frontend-calendar/ frontend-calendar/

# Generate SvelteKit tsconfigs (gitignored) so esbuild can resolve them
RUN pnpm --filter frontend-mail exec svelte-kit sync
RUN pnpm --filter frontend-calendar exec svelte-kit sync
RUN pnpm --filter frontend-landing build

# --- Builder Stage ---
# Floating 1.x: pinned 1.92 cannot compile specta 2.0.0-rc.25 (needs fmt::from_fn).
FROM rust:1-slim-bookworm AS builder

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
COPY --from=landing /app/frontend-landing/dist ./frontend-landing/dist
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
# Copy built landing page served by the backend (landing_fallback reads frontend-landing/dist)
COPY --from=builder /app/frontend-landing/dist /app/frontend-landing/dist

# Expose backend port
EXPOSE 8080

# Run daemon
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:${PORT:-8080}/api/v1/health || exit 1

ENTRYPOINT ["/usr/local/bin/kestrel-server"]
