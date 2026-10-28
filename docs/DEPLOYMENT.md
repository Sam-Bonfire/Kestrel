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

- **Windows:** Download and run the NSIS `.exe` installer. (The `.msi` is built but not published: only one Windows installer ships, and NSIS is the format the in-app updater can install in place.)
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
| `BIND_ADDR` | Direct socket override (e.g. `0.0.0.0:8080`). | `<HOST>:<PORT>` |
| `KESTREL_BASE_URL` | The publicly accessible URL of the backend (used for OAuth). | *Required* |
| `DATABASE_URL` | Connection string for SQLite or PostgreSQL. | `sqlite:/app/data/kestrel.db` |
| `SESSION_SECRET` / `JWT_SECRET` | Secret key for signing JWTs and session auth. Must be a long, random string (32+ chars). | *Required* |
| `RUST_LOG` | Logging verbosity (e.g., `info`, `debug`). | `info` |
| `FORMS_ENDPOINT` | URL that site forms post to. Empty disables forms. See [SELFHOST.md](./SELFHOST.md). | *Disabled* |
| `FORMS_TOKEN` | Bearer token your forms receiver checks. Empty sends no `Authorization` header. | *Unset* |

---

## 7. Updating

To update your deployment to the latest version:

**Backend:**
```bash
docker compose pull
docker compose up -d
```

**Desktop Clients:**
Download the new version from GitHub Releases and install it over the existing version. The apps also check for updates at startup against `<app>-latest.json` on the latest release; those manifests and their signatures are produced by the release workflow (`scripts/build-updater-manifest.mjs`) from the signed installers.

### Updater signing keys

- The private key lives only in the `TAURI_SIGNING_PRIVATE_KEY` repository secret; the workflow exports it for every desktop build so `tauri build` can sign the bundles. Never commit it, print it, or paste it into logs.
- The matching public key is the `plugins.updater.pubkey` value in each app's `src-tauri/tauri.conf.json`. Rotating the key means shipping a release signed with the old key that carries the new pubkey, then swapping the secret.
- If the secret is missing or wrong, the release job fails while generating `*-latest.json` (an unsigned manifest makes every client reject updates), instead of publishing a release nobody can update from.
- Bump the `version` in `frontend-mail/src-tauri/tauri.conf.json` and `frontend-calendar/src-tauri/tauri.conf.json` for each release: the manifest advertises the bundle version, so an unbumped build never offers (or keeps re-offering) the update.

> [!NOTE]
> Future updates will be handled automatically via Tauri's auto-update feature.

---

## 8. Backup & Recovery

> [!IMPORTANT]
> A backup you have never restored is a guess. Rehearse the restore on a clean
> host once, then put the backup on a timer.

### What to back up

| Item | Why | Where |
| --- | --- | --- |
| The database (SQLite file or Postgres dump) | Accounts, messages, events, revisions | `kestrel-data` volume / your Postgres host |
| Secrets (`SESSION_SECRET`, OAuth client secrets) | Restores cannot log in or sync without them | Your password manager |
| Optional: `data/crashes/` | Local crash reports, if you enabled them | `kestrel-data` volume |

WASM plugins and the frontend bundles are rebuilt from source; do not back them up.

### SQLite

`sqlite3` ships in the backend image, so back up online without stopping the
server:

```bash
docker exec -t kestrel-backend \
  sh -c 'sqlite3 /app/data/kestrel.db ".backup /app/data/kestrel-backup.db"'
docker cp kestrel-backend:/app/data/kestrel-backup.db ./kestrel-backup.db
```

`.backup` uses SQLite's online backup API: it is consistent while the server is
writing. Copying the raw file (or the volume) is only safe with the container
stopped, because a live database can have a `-wal` file holding recent writes.

Verify the copy before you trust it:

```bash
sqlite3 kestrel-backup.db "PRAGMA integrity_check;"      # expect: ok
sqlite3 kestrel-backup.db "SELECT count(*) FROM messages;"
```

No local `sqlite3`? Check the file through the image instead:
`docker compose run --rm -v "$PWD/kestrel-backup.db:/tmp/backup.db" --entrypoint sqlite3 backend /tmp/backup.db "PRAGMA integrity_check;"`

Restore on a clean host (or over a broken one):

```bash
docker compose stop backend
docker compose run --rm --entrypoint sh backend \
  -c 'rm -f /app/data/kestrel.db /app/data/kestrel.db-wal /app/data/kestrel.db-shm'
docker compose cp ./kestrel-backup.db backend:/app/data/kestrel.db
docker compose start backend
curl -f http://localhost:8080/api/v1/health
```

Migrations run at startup, so a backup from an older release is upgraded
automatically. Log in and check the message count before you delete the backup.

### PostgreSQL

```bash
# Custom-format dump: compressed, restorable table-by-table
pg_dump -Fc -h db.example.com -U kestrel kestrel > "kestrel-$(date +%F).dump"

# Inside Docker instead of a local client:
docker exec -t kestrel-db pg_dump -U kestrel -Fc kestrel > "kestrel-$(date +%F).dump"
```

Verify and restore:

```bash
pg_restore --list kestrel-2026-10-28.dump | head        # sanity: objects are listed
createdb -h newhost -U kestrel kestrel                  # clean target
pg_restore --clean --if-exists -h newhost -U kestrel -d kestrel kestrel-2026-10-28.dump
```

`--clean --if-exists` makes the restore repeatable over a half-populated
database. It drops and recreates objects, so never point it at a database you
still need.

### Before deleting an account

Account purge is destructive and is not covered by per-message revisions or the
contacts export. Take a database snapshot first (sections above), keep it for as
long as your retention policy says, then purge.

### Scheduling

Keep at least seven daily copies and four weekly ones, off the host:

```bash
# cron: every day at 03:15
15 3 * * * cd /srv/kestrel && docker exec -t kestrel-backend \
  sh -c 'sqlite3 /app/data/kestrel.db ".backup /app/data/kestrel-backup.db"' \
  && docker cp kestrel-backend:/app/data/kestrel-backup.db \
     /srv/backups/kestrel-$(date +\%F).db
```

Test the timer by running the command by hand once and restoring the file.

### Cloudflare Tunnel config

A ready-to-edit tunnel config lives in
[`deploy/cloudflared/config.yml`](../deploy/cloudflared/config.yml). It maps one
public hostname to `http://localhost:8080` and returns 404 for everything else.
Set `KESTREL_BASE_URL` to that hostname so OAuth redirects and API links match.
