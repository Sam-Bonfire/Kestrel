# Kestrel

![CI](https://github.com/YOUR_USERNAME/kestrel/actions/workflows/ci.yml/badge.svg?branch=dev)
![Rust](https://img.shields.io/badge/rust-stable-orange.svg)
![Node](https://img.shields.io/badge/node->=20-green.svg)
![License](https://img.shields.io/badge/license-TBD-blue.svg)

Kestrel is a **private, lightweight, self-hosted email and calendar suite**. It provides a premium, keyboard-driven UI (inspired by Superhuman and Notion Mail) while running with a minimal resource footprint. 

The suite consists of:
- **Kestrel Mail** — a desktop and mobile email client
- **Kestrel Calendar** — a desktop and mobile calendar client
- **Kestrel Backend** — a Rust server that syncs with your email and calendar providers

## 🏗 Architecture

- **Backend:** High-performance Rust server using the Axum web framework and Tokio async runtime.
- **Frontend:** Built with Svelte and TailwindCSS v4 for a highly responsive, modern interface.
- **Desktop & Mobile:** Wrapped in Tauri v2 and Tauri Mobile v2 to provide native apps for Windows, macOS, Linux, Android, and iOS.
- **Database:** Uses SQLx to support both SQLite (for simple, single-file setups) and PostgreSQL.
- **Plugin System:** Server-side WebAssembly (WASM) plugins via Wasmtime for robust, sandboxed integrations with providers like Gmail and Outlook.
- **Deployment:** The backend runs in a Docker container on any Linux server or NAS. Clients connect securely via a Cloudflare Tunnel (public) or Tailscale VPN (private).

## 🛠 Tech Stack

| Layer | Technology |
|-------|------------|
| Backend Language | Rust |
| Web Framework | Axum + Tokio |
| Database | SQLite / PostgreSQL (via SQLx) |
| Plugin Engine | Wasmtime (WASI) |
| Frontend Framework | Svelte |
| Styling | TailwindCSS v4 |
| Desktop Runtime | Tauri v2 |
| Mobile Runtime | Tauri Mobile v2 |
| Package Manager | pnpm (Node), Cargo (Rust) |
| Toolchain | mise |
| VCS | Jujutsu (jj) with Git backend |
| CI/CD | GitHub Actions |
| Container Registry | GitHub Container Registry (ghcr.io) |

## 📂 Monorepo Structure

<details>
<summary>Click to expand folder structure</summary>

```text
kestrel/
├── backend/              # Rust Axum server
│   ├── src/
│   │   ├── api/          # REST API routes (auth, messages, calendars, sync)
│   │   ├── core/         # Domain models, repository traits, error types
│   │   ├── db/           # Database adapters (sqlite/, postgres/)
│   │   └── plugins/      # WASM plugin system (Gmail, Outlook, mock)
│   ├── tests/            # Integration tests
│   └── Dockerfile        # Multi-stage Docker build
├── frontend-mail/        # Kestrel Mail (SvelteKit + Tauri)
│   ├── src/
│   │   ├── lib/components/  # Mail UI components
│   │   └── routes/          # SvelteKit pages
│   └── src-tauri/           # Tauri native config
├── frontend-calendar/    # Kestrel Calendar (SvelteKit + Tauri)
│   ├── src/
│   │   ├── lib/components/  # Calendar UI components
│   │   └── routes/          # SvelteKit pages
│   └── src-tauri/           # Tauri native config
├── frontend-shared/      # Shared library (@kestrel/shared)
│   └── src/
│       ├── api/             # API client, auth, offline queue
│       ├── components/      # Shared UI (CommandPalette, Settings)
│       ├── stores/          # Svelte stores (theme, notifications)
│       └── utils/           # Keyboard shortcuts, date helpers
├── .github/workflows/    # CI + Release pipelines
├── docker-compose.yml    # Production deployment
└── mise.toml             # Toolchain versions & dev tasks
```
</details>

## 🚀 Current Status

This project is in **active early development (v0.1.0)**. 

> [!NOTE]
> The core UI for both the Mail and Calendar apps is fully functional with a polished dark-theme interface. The backend API layer is complete with dual database support. Currently, real provider integrations are using mock plugins — live sync is coming soon!

### What works today
- Full mail client UI (inbox, threading, compose, search, labels, keyboard shortcuts)
- Full calendar UI (week/day/month views, event creation/editing, sidebar navigation)
- Backend REST API with auth, message CRUD, calendar CRUD, search
- SQLite and PostgreSQL dual database support
- Docker deployment with health checks
- CI/CD pipeline producing Windows MSI, Linux .deb/.AppImage, macOS .dmg, Android APK, iOS builds, and Docker images

### Roadmap
- [x] Live Gmail/Outlook sync via real WASM provider plugins (mail + calendar)
- [x] Real SMTP/API email sending (via Gmail/Outlook APIs)
- [x] Calendar provider sync (Google Calendar, Outlook Calendar)
- [ ] Push notifications via FCM/APNs
- [ ] Tauri auto-update mechanism
- [ ] Android/iOS code signing for store distribution
- [ ] Rich text email composition
- [x] Attachment handling and download (via Tauri fs to OS download dir)
- [ ] Contact address book integration

## ⚡ Quick Start

### Prerequisites
- [mise](https://mise.jdx.dev/) (manages Rust, Node, and pnpm versions)
- [Docker](https://www.docker.com/) (for backend deployment)
- [jj](https://martinvonz.github.io/jj/) (optional, for version control)

### Development Setup

```bash
# Clone the repository
git clone https://github.com/YOUR_USERNAME/kestrel.git
cd kestrel

# Install toolchain and all project dependencies (run once)
mise run init

# Start everything (backend + mail + calendar)
mise run dev
```

### Docker Deployment

```bash
# Configure environment
cp .env.example .env
# Edit .env with your settings

# Start the backend
docker compose up -d
```

## ⚙️ Environment Variables

The backend can be configured using a `.env` file or native environment variables.

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | No | `sqlite:./data/kestrel.db` | SQLite path or PostgreSQL connection string |
| `PORT` | No | `8080` | Server port |
| `HOST` | No | `0.0.0.0` | Bind address |
| `RUST_LOG` | No | `info,kestrel=debug` | Log level |
| `SESSION_SECRET` | Yes (prod) | - | Secure key for session cookies |
| `GOOGLE_CLIENT_ID` | No | - | Google OAuth client ID |
| `GOOGLE_CLIENT_SECRET` | No | - | Google OAuth client secret |
| `MICROSOFT_CLIENT_ID` | No | - | Microsoft OAuth client ID |
| `MICROSOFT_CLIENT_SECRET` | No | - | Microsoft OAuth client secret |
| `KESTREL_BASE_URL` | Yes (prod) | - | Public URL for OAuth redirects |
| `SYNC_INTERVAL_MINUTES` | No | `5` | Background sync frequency |
| `PLUGINS_DIR` | No | `/app/plugins` | WASM plugins directory |

## 🚢 CI/CD

Kestrel employs a two-branch strategy for robust releases:
- **`dev` branch**: This is the integration gate. All PRs targeting `dev` must pass CI, which includes backend tests, frontend builds, Docker image building, and compilation for all target platforms (Windows, macOS, Linux, Android, iOS).
- **`main` branch**: This is the release branch. Merging `dev` into `main` automatically triggers a GitHub Release with tags and uploads all platform artifacts.

**Generated Artifacts:** Windows MSI/EXE, Linux .deb/.AppImage, macOS .dmg, Android APK, iOS .app, and Docker images on GHCR.

## 📖 Documentation

| Document | Description |
|----------|-------------|
| [ARCHITECTURE.md](ARCHITECTURE.md) | Authoritative architecture contract — tech stack, constraints, design decisions |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Development setup, workflow, code style, PR guidelines |
| [ROADMAP.md](ROADMAP.md) | Feature status tracker — what's done, in progress, and planned |
| [Deployment Guide](docs/DEPLOYMENT.md) | Docker setup, networking (Cloudflare/Tailscale), database config, client installation |
| [Decision Log](DECISION_LOG.md) | Historical record of all architectural decisions and rationale |
| [Project Rules](PROJECT_RULES.md) | Coding standards, naming conventions, file limits |
| [Technical Spec](kestrel_spec.md) | Full technical specification — schemas, API routes, plugin interface |

## 🤝 Contributing

We welcome contributions of all sizes! Whether it's a bug fix, new feature, or typo correction, we appreciate your help. Kestrel uses pnpm, Rust stable, and jj (compatible with Git) for its workflow. 

For detailed information on how to get started, please refer to our [CONTRIBUTING.md](CONTRIBUTING.md).

## 📄 License

License: TBD
