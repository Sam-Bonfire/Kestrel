# Kestrel Deployment Guide

This guide covers how to deploy the Kestrel backend and install the client applications on your devices.

## Table of Contents

- [1. Docker Compose (Recommended)](#1-docker-compose-recommended)
  - [docker-compose.yml](#docker-composeyml)
  - [Setup Steps](#setup-steps)
  - [Networking Options](#networking-options)
- [2. Database Configuration](#2-database-configuration)
  - [SQLite (Default)](#sqlite-default)
  - [PostgreSQL](#postgresql)
- [3. Installing Desktop Clients](#3-installing-desktop-clients)
- [4. Installing Mobile Clients](#4-installing-mobile-clients)
- [5. First-Time Setup](#5-first-time-setup)
- [6. Environment Variables Reference](#6-environment-variables-reference)
- [7. Updating](#7-updating)
- [8. Backup & Recovery](#8-backup--recovery)

---

## 1. Docker Compose (Recommended)

The primary and recommended deployment method is running the backend in Docker on a Linux server or NAS.

### docker-compose.yml

The project includes a `docker-compose.yml` file at the root. Key aspects include:

- **Backend image:** `kestrel-backend:latest` (or from GHCR: `ghcr.io/OWNER/kestrel-backend:vX.Y.Z`)
- **Volumes:** `kestrel-data` for the SQLite database, and `kestrel-plugins` for WASM plugins.
- **Ports:** Exposed on port `8080` (configurable via the `PORT` environment variable).
- **Healthcheck:** Verifies the service is running using `curl -f http://localhost:${PORT:-8080}/api/v1/health`.
- **Configuration:** Managed via a `.env` file.

### Setup Steps

1. **Pull the Docker image** from GHCR (or build it locally):
   ```bash
   docker pull ghcr.io/OWNER/kestrel-backend:latest
   ```

2. **Create a `.env` file** based on the provided template:
   ```bash
   cp .env.example .env
   ```

3. **Configure required variables** in your `.env` file (e.g., `SESSION_SECRET`, `KESTREL_BASE_URL`).

4. **Start the services** in the background:
   ```bash
   docker compose up -d
   ```

5. **Verify health** to ensure the backend is running correctly:
   ```bash
   curl http://localhost:8080/api/v1/health
   ```

> [!NOTE]
> Make sure you have Docker and Docker Compose installed on your host system before starting.

### Networking Options

Depending on your security and access needs, you can expose the Kestrel backend in different ways.

#### Option A: Cloudflare Tunnel (Public Access)
Best if you need to access Kestrel over the public internet and want simple OAuth integration.

- Install `cloudflared` on your server.
- Create a tunnel pointing to `http://localhost:8080`.
- Set `KESTREL_BASE_URL` in your `.env` to your public domain (e.g., `https://kestrel.yourdomain.com`).
- This method natively enables OAuth redirects for Gmail and Outlook.

#### Option B: Tailscale VPN (Private Access)
Best for maximum security where Kestrel is only accessible on your private mesh network.

- Install Tailscale on your server and all client devices.
- Set the `HOST` variable in `.env` to the server's Tailscale IP (e.g., `100.x.x.x`).
- Set `KESTREL_BASE_URL` to `http://100.x.x.x:8080`.
- All traffic stays securely on your private Tailnet.

---

## 2. Database Configuration

### SQLite (Default)

- **Zero configuration:** Just set `DATABASE_URL=sqlite:/app/data/kestrel.db`.
- Ideal for single-user or small deployments.
- Data is persistently stored in the `kestrel-data` Docker volume.

### PostgreSQL

- Set `DATABASE_URL=postgresql://user:pass@host:5432/kestrel`.
- Recommended for multi-user deployments or higher concurrency needs.
- You can easily add a `postgres` service block to your `docker-compose.yml` to run the database alongside the backend.

---

## 3. Installing Desktop Clients

To install the desktop application, download the latest release for your OS from the [GitHub Releases page](https://github.com/OWNER/kestrel/releases).

- **Windows:** Download and run the `.msi` installer.
- **Linux:** Install the `.deb` package (`sudo dpkg -i kestrel_*.deb`) or run the portable `.AppImage`.
- **macOS:** Download the `.dmg`, open it, and drag the Kestrel app into your Applications folder.

---

## 4. Installing Mobile Clients

- **Android:** Download the `.apk` from the GitHub Releases page and sideload it onto your device.
- **iOS:** Currently, only simulator builds are available, as code signing is not yet configured for physical devices.

---

## 5. First-Time Setup

After installing a client app (desktop or mobile), follow these steps to connect to your backend:

1. Open the Kestrel app.
2. Enter your backend URL when prompted (e.g., `https://kestrel.yourdomain.com` or `http://100.x.x.x:8080`).
3. Register a new user account.
4. Go to settings to connect your email and calendar providers via OAuth.

> [!TIP]
> Ensure your `KESTREL_BASE_URL` exactly matches the URL you enter here, otherwise OAuth redirects will fail.

---

## 6. Environment Variables Reference

Configure these variables in your `.env` file at the root of the backend deployment.

| Variable | Description | Default |
| --- | --- | --- |
| `PORT` | The port the backend listens on inside the container. | `8080` |
| `HOST` | The interface to bind to. Use `0.0.0.0` in Docker. | `0.0.0.0` |
| `KESTREL_BASE_URL` | The publicly accessible URL of the backend (used for OAuth). | *Required* |
| `DATABASE_URL` | Connection string for SQLite or PostgreSQL. | `sqlite:/app/data/kestrel.db` |
| `SESSION_SECRET` | Secret key for encrypting sessions. Must be a long, random string. | *Required* |
| `RUST_LOG` | Logging verbosity (e.g., `info`, `debug`). | `info` |

---

## 7. Updating

To update your deployment to the latest version:

**Backend:**
```bash
docker compose pull
docker compose up -d
```

**Desktop Clients:**
Download the new version from GitHub Releases and install it over the existing version.

> [!NOTE]
> Future updates will be handled automatically via Tauri's auto-update feature.

---

## 8. Backup & Recovery

> [!IMPORTANT]
> Regularly back up your database to prevent data loss.

- **SQLite:** Back up the `kestrel-data` Docker volume, or directly copy the `/app/data/kestrel.db` file from the container.
- **PostgreSQL:** Use standard tools like `pg_dump` to export your database.

```bash
# Example pg_dump command for Postgres
docker exec -t kestrel-db pg_dump -U user kestrel > backup.sql
```
