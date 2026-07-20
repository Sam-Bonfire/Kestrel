# Architecture Specification

Version: 1.0

---

# Purpose

This document is the authoritative architecture contract for this project.

It defines the project's technical decisions, constraints, and architectural boundaries.

All implementation must conform to this document.

If implementation conflicts with this document, the implementation is incorrect.

---

# Generation Rules

If this file does not exist:

1. Analyze the user's requirements.
2. Infer only information that is explicitly supported.
3. Ask clarifying questions only for decisions that materially affect architecture.
4. Generate this document before writing production code.
5. Present it for review if major architectural decisions are inferred.
6. Treat the approved version as immutable until the user changes it.

Never invent technologies without justification.

---

# Project Snapshot

Project Name: Kestrel

Business Domain: Personal Productivity / Communication & Scheduling

Primary Goal: A private, lightweight, self-hosted productivity suite — **Kestrel Mail** and **Kestrel Calendar** — two separate app targets that share a common Axum backend, auth session, and WASM plugin runtime. Designed for Windows and Android with a premium, keyboard-driven UI and minimal resource footprint.

Users: Private self-hosters wanting low footprint and keyboard efficiency, supporting rudimentary multi-tenancy.

Deployment Target: Remote Linux NAS (Docker Compose backend with Cloudflare Tunnel or Tailscale for connectivity) & Native Desktop/Mobile Clients (Windows Tauri v2 / Android Tauri Mobile / iOS Tauri Mobile).

Network: Administrator configures `KESTREL_BASE_URL` at setup time. Cloudflare Tunnel is used for public deployments; Tailscale for private LAN-only deployments. Both are supported.

Current Phase: Architecture & Design

---

# Technology Stack

Language: Rust (Backend), TypeScript/JavaScript (Frontend)

Runtime: Tokio Async Runtime (Backend), Wasmtime (Backend WASM engine), WebView2/Chromium (Frontend via Tauri)

Package Manager: Cargo (Rust), pnpm (Node/Tauri)

Toolchain/Task Manager: `mise` (Tool versioning and task runner)

VCS: Jujutsu (jj) with Git-compatibility backend

Frontend: Svelte (UI Framework), TailwindCSS v4 (Styling)

Component Library: Skeleton UI (Svelte-native components with custom Notion/Morgen theme tokens)

Backend: Rust / Axum (Web Framework)

Database: SQLite & PostgreSQL dual-compatibility with logical multi-tenancy partitioning

ORM: sqlx (separate adapters for compile-time verified SQLite and PostgreSQL queries)

Authentication & Secrets: 
- Centralized Provider Token Vault (Access and refresh tokens stored on server)
- Rudimentary multi-tenancy user session cookie auth to the Kestrel daemon
- OAuth2 with Cloudflare Tunnels redirect mapping; Optional `secretspec` credentials loader (startup resolution)

State Management: Svelte Writable Stores

Testing: Rust unit/integration tests (`cargo test`), Vitest/Playwright for Svelte UI

Build Tool: Cargo, Vite

Deployment: Docker Compose (Backend Daemon), Tauri Bundler (Desktop & Mobile)

CI/CD: GitHub Actions (build/lint validation)

Monitoring: Prometheus/Grafana (optional metrics)

Logging: `tracing` / `tracing-subscriber` (Rust)

---

# Architectural Style

Primary Pattern: Clean Architecture / Hexagonal Architecture with Dynamic WASM Plugins

Explain why this pattern was selected:
To decouple remote integration engines (Gmail, Outlook, Yahoo, Notion) entirely from the core application. By compiling integrations to sandboxed WebAssembly (WASI) plugins loaded dynamically at runtime, community integrations can be added seamlessly without recompiling or altering the main Rust server codebase.

---

# Project Structure

```text
kestrel/                              # Cargo workspace root
├── Cargo.toml                        # Workspace manifest
├── docker-compose.yml
├── mise.toml                         # Tool versioning & task runner
├── wit/
│   └── kestrel.wit                   # WIT spec for WASM plugins
├── backend/
│   ├── src/
│   │   ├── main.rs
│   │   ├── api/                      # Axum routes & controllers (shared by both apps)
│   │   ├── core/                     # Domain models, sync traits, business logic
│   │   ├── db/                       # Repository implementations
│   │   │   ├── mod.rs                # Runtime adapter dispatcher
│   │   │   ├── sqlite/               # SQLite-specific SQLx queries
│   │   │   └── postgres/             # Postgres-specific SQLx queries
│   │   └── plugins/                  # Wasmtime runtime manager & host bindings
│   └── migrations/
│       ├── sqlite/                   # SQLite migration scripts
│       └── postgres/                 # PostgreSQL migration scripts
├── frontend-shared/                  # Shared npm package used by both Tauri apps
│   ├── src/
│   │   ├── api/                      # Typed API client & endpoint wrappers
│   │   ├── stores/                   # Auth store, shared reactive state
│   │   ├── components/               # Avatar, Button, Spinner, ErrorBanner, etc.
│   │   └── tokens/                   # Kestrel Slate design tokens
│   └── package.json
├── frontend-mail/                    # Kestrel Mail — Tauri v2 app
│   ├── src/
│   │   ├── lib/
│   │   │   ├── components/           # MessageList, ThreadPeek, ComposeModal, CommandPalette
│   │   │   └── stores/               # Inbox store, thread store, offline queue
│   │   ├── routes/
│   │   └── app.html
│   └── src-tauri/                    # Tauri config (kestrel-mail target)
└── frontend-calendar/                # Kestrel Calendar — Tauri v2 app
    ├── src/
    │   ├── lib/
    │   │   ├── components/           # WeekGrid, EventChip, EventPeek, NewEventModal
    │   │   └── stores/               # Events store, calendars store, offline queue
    │   ├── routes/
    │   └── app.html
    └── src-tauri/                    # Tauri config (kestrel-calendar target)
```

> **Monorepo**: All targets live in a single repository managed as a Cargo workspace. Both Tauri frontends reference `frontend-shared` as a local npm dependency.

> **Offline queue**: Each Tauri app maintains a local `kestrel_queue.db` (SQLite) in the OS app-data directory to persist pending mutations (archive, trash, mark-read, event edits) for replay when connectivity returns.

---

# Dependency Rules

Allowed dependency directions:
- `api` depends on `core` and `db`.
- `plugins` depends on `core` (executing traits via WASM bindings).
- `db` depends on `core`.
- `core` has zero external implementation dependencies (contains only domain models, traits, and interface boundary definitions).

Forbidden dependency directions:
- `core` must never depend on `db` or specific provider plugins.
- `frontend` communicates with `backend` over Cloudflare Tunnels REST APIs (no direct DB access).

Circular dependency policy: Zero tolerance.

---

# Data Layer

Database: SQLite (default cache) and PostgreSQL (for external database deployments).

Persistence strategy: Persistent local caches logically partitioned by user. Conflict resolution uses Last-Write-Wins (LWW) with overridden states backed up to a `historical_revisions` table. Composite unique keys `UNIQUE (user_id, email_address)` on the accounts table allow different users to securely connect identical mailboxes without collision.

Migration strategy: SQLx migrations run on startup of the Axum daemon, dynamically checking the connection URL to select either the `migrations/sqlite` folder or `migrations/postgres` folder.

Repository strategy: Repository pattern implementing traits defined in `core`. The `db/mod.rs` instantiates either `sqlite` or `postgres` adapters at startup, filtering queries by the user's mapped accounts.

Caching strategy: Entire user inbox, folder mapping, and calendar event timelines are cached locally.

Transactions: SQLx transactions used for bulk mail/event ingest.

---

# API Design

REST:
- Unified endpoints for email and calendar actions.

Internal APIs:
- Secure communication between Svelte client and Axum daemon.

Versioning: `/api/v1` prefix.

Validation: Rust `serde` and `validator` crate.

Error format:
```json
{
  "error": "Error code string",
  "message": "Human readable message"
}
```

Authentication: Sessions cookie or bearer token generated after oauth handshakes, binding client device views to a specific User ID context.

Authorization: User account scoping checks (clients can only access accounts owned by their authenticated user).

---

# Frontend Architecture

## App Targets

Kestrel ships as two separate applications, both built with Svelte + Tauri:

| App | Tauri Target | Platforms |
|---|---|---|
| **Kestrel Mail** | `kestrel-mail` | Windows, Android, iOS |
| **Kestrel Calendar** | `kestrel-calendar` | Windows, Android, iOS |

Both apps share:
- The same Axum backend server and REST API (`KESTREL_BASE_URL`)
- The `frontend-shared` npm package (API client, auth store, design tokens, shared components)
- The same Kestrel Slate dark-mode design system

**Auth sessions are independent per app.** Each app authenticates separately with username + password and holds its own bearer token. There is no cross-app token sharing.

Neither app embeds components or routes from the other. Users launch each independently from their OS.

## First-run flow

1. App launches → detects no server URL configured → shows **Setup screen**
2. User enters `KESTREL_BASE_URL` (e.g. `https://kestrel.example.com`)
3. User logs in with username + password → receives bearer token stored in OS keychain
4. User connects mail/calendar accounts via OAuth (system browser → `kestrel://` deep-link return)

## Component structure

**Kestrel Mail**: Sidebar (folder nav + account list), ThreadList, ThreadPeekPanel, ComposeModal, CommandPalette, SettingsRoutes.

**Kestrel Calendar**: Sidebar (calendar toggles + mini-month), WeekGrid / MonthGrid / DayGrid / AgendaList, EventChip, EventPeekPanel, NewEventModal, SettingsRoutes.

## Offline support

Both apps are **fully functional offline**. Mutations (archive, trash, mark-read, event edits) are written to a local SQLite queue (`kestrel_queue.db` in OS app-data dir) and replayed against the backend when connectivity returns. Read operations serve from the local server cache.

## State management

Centralized Svelte writable stores, scoped per-app. Auth store and API client come from `frontend-shared`.

## Routing

Static client-side routing within each app. Mobile uses single-pane stack navigation (list → peek → back) with a bottom tab bar replacing the desktop sidebar.

## Data fetching

Typed HTTP client from `frontend-shared/api` fetching from `KESTREL_BASE_URL`. All responses are cached in Svelte stores; components never call the API directly.

## UI framework

Svelte + Skeleton UI.

## Design system

**Kestrel Slate** — dark-mode only. Deep slate blue-gray canvas (`#0B0D12`), warm lavender-white text, periwinkle accent (`#5B6AD0`), vibrant event color palette for calendar chips.

## Accessibility

Full ARIA support via Skeleton primitives. Complete keyboard navigation via `tinykeys` shortcut registry with Input Guard (shortcuts disabled when focus is inside an input/textarea).

---

# Backend Architecture

Services: Sync loop daemon running as background Tokio tasks.

Repositories: Dual SQLite/Postgres implementations implementing core repository traits.

Controllers: Axum handler functions.

Workers: Background async sync loops (runs every 5 minutes or triggered on webhook/client requests).

Queues: In-memory async channels (`tokio::sync::mpsc`) for scheduling sync jobs.

Scheduling: Tokio interval loops.

Plugins Runtime: Wasmtime engine loading `.wasm` plugins compiled to WASI specifications.

External integrations: Handled entirely dynamically inside WASM provider plugins.

---

# Security

Secrets: Resolved at backend startup from an optional `secretspec` config folder or fallback system environment parameters. Resolved credentials are held securely in host memory and injected into guest WASM runtimes via explicit host call traits.

Authentication: Username + password login per app, returning a bearer token stored in the OS keychain. OAuth2 auth code flow with PKCE for Google/Microsoft provider connections — initiated via system browser, with callback to `KESTREL_BASE_URL/api/auth/callback`. The Tauri app receives the confirmation via a `kestrel://` deep-link URL scheme registered at install time. Each Tauri app holds its own independent bearer token; sessions are not shared between Mail and Calendar apps.

Token management: Provider OAuth tokens (Google/Microsoft) are managed server-side. The sync daemon silently refreshes tokens before expiry. The user never sees token expiry flows for provider accounts.

Authorization: Cloudflare Access / Tunnels authentication wrapper checks. Strict data isolation boundaries prevent users from accessing database rows linked to accounts they do not own.

Encryption: TLS-encrypted tunnels.

Rate limiting: Basic IP-based rate limiting using Axum middleware.

Input validation: String sanitization and structural checking via Rust validator.

Audit logging: Simple structured logger output to stdout/file.

Compliance requirements: None.

---

# Performance

Caching: Local cache ensures sub-10ms response times for all list and search operations.

Pagination: Cursor-based pagination for messages.

Batching: Sync loop batches upstream requests.

Concurrency: Async tokio runtime, WAL mode SQLite or concurrent Postgres pool.

Lazy loading: Defer email rendering and attachment downloads.

Streaming: Event streaming (Server-Sent Events) for real-time sync notifications to the client.

Performance targets: UI initial load under 200ms; local search under 50ms.

---

# Scalability

Expected users: Multiple self-hosted users (e.g. family configuration).

Expected traffic: Extremely low (personal client).

Deployment model: Docker container on private NAS.

Horizontal scaling: Easy integration with external high-concurrency PostgreSQL databases.

Background processing: Handled efficiently by multi-threaded Tokio background pool.

Storage growth: Cache size expected to be < 10GB for a decade of typical email/calendar volume (excluding attachments).

---

# Coding Standards

Naming conventions: `snake_case` in Rust, `camelCase` in TypeScript/Svelte.

File naming: `kebab-case` for UI components and Svelte files.

Folder naming: `kebab-case` for directory structures.

Module boundaries: Strictly enforced by Rust visibility rules (`pub(crate)`, `pub`).

Maximum file size: 400 lines preferred.

Maximum function size: 50 lines preferred.

Documentation expectations: Public API interfaces must be documented using docstrings.

---

# Testing Strategy

Unit tests: Cargo unit tests for core utilities and sync logic.

Integration tests: SQLx tests running against temporary memory DBs.

End-to-end tests: Playwright for frontend interaction verification.

Manual verification: Verified by running local dev servers and inspecting UI states.

Performance tests: Local benchmark suite for SQLite search.

Security tests: Static vulnerability scanning (cargo-audit).

Coverage goals: 70%+ of core business logic.

---

# Forbidden Technologies

List technologies that may not be introduced without explicit approval.

- Technology: IMAP/POP3 directly (Unless OAuth is configured, raw IMAP password login is prohibited due to security concerns).
- Technology: Heavy ORM like Diesel (sqlx is selected for lightweight compile-time SQL validation without schema compilation overhead).

---

# Architectural Decisions

- Decision: Axum + SQLite cache.
  - Reason: Lightweight resource consumption and high concurrency.
  - Alternatives considered: Postgres, Node.js backend.
  - Tradeoffs: SQLite is file-bound, making scaling to multiple nodes hard, but perfect for self-hosted NAS.
  - Date: 2026-07-18

- Decision: Tauri v2 + Svelte.
  - Reason: Super-lightweight runtime compared to Electron, compilation to native JS with Svelte is extremely fast.
  - Alternatives considered: Electron, React.
  - Tradeoffs: Requires setting up Rust-mobile NDK for Android wrappers, but results in tiny build outputs and low RAM footprint.
  - Date: 2026-07-18

- Decision: Sandboxed iframe client-side for HTML emails.
  - Reason: Simplifies layout isolation and secures code environment.
  - Alternatives considered: Backend sanitization, Markdown converter.
  - Date: 2026-07-18

- Decision: LWW Sync with Conflict History Table.
  - Reason: Robust enough for low-frequency changes, with recovery hooks via historical backup.
  - Alternatives considered: Interactive merges, Vector clocks.
  - Date: 2026-07-18

- Decision: Direct URL redirects for attachments, cached on client-side disk.
  - Reason: Minimizes NAS storage footprint.
  - Alternatives considered: Full NAS caching.
  - Date: 2026-07-18

- Decision: Use `mise` for environment versions and runner commands.
  - Reason: Simplifies script-free operations across various dev platforms.
  - Alternatives considered: Local custom scripts, Makefiles.
  - Date: 2026-07-18

- Decision: Deploy backend daemon using multi-stage Docker / Docker Compose.
  - Reason: Standardizes deployment on the remote NAS server while reusing the existing server-level Cloudflare Tunnel environment.
  - Alternatives considered: Packaging cloudflared as a sidecar container, bare-metal deployment.
  - Date: 2026-07-18

- Decision: Server-side WASM plugin architecture for mail and calendar providers.
  - Reason: Decouples all third-party integrations, secures plugins inside Wasmtime sandbox, and allows extensions to be written in various languages without modifying core code.
  - Alternatives considered: Process RPC, Dynamic Libs loading.
  - Date: 2026-07-18

- Decision: Standardize on Skeleton UI with a custom Notion/Morgen styling theme.
  - Reason: Svelte-native, batteries-included UI component framework styled via design tokens, resolving the tension between high productivity (pre-built elements) and custom high-end visual aesthetics.
  - Alternatives considered: shadcn-svelte, Flowbite Svelte.
  - Date: 2026-07-18

- Decision: Repository Abstraction with SQLite and PostgreSQL dual SQLx backends.
  - Reason: Preserves compile-time query verification and maximum runtime performance by using dialect-native SQL engines instead of dynamic ORMs.
  - Alternatives considered: Database-agnostic ORMs (SeaORM / Diesel), single database constraint.
  - Date: 2026-07-18

- Decision: Optional `secretspec` startup secrets manager integration.
  - Reason: Securely resolves sensitive tokens at container launch from specified vaults or encrypted files, falling back to standard environments without complicating local development setups.
  - Alternatives considered: Required Vault constraints, plaintext environment keys only.
  - Date: 2026-07-18

- Decision: Centralized Provider Token Vault with Unified Client Device Session.
  - Reason: Automatically shares credentials across all authenticated desktop and mobile client interfaces seamlessly without requiring repeating OAuth setups or insecure local syncs.
  - Alternatives considered: Client-side local key caches.
  - Date: 2026-07-18

- Decision: Rudimentary Logical Multi-Tenancy Partitioning with Composite Uniqueness.
  - Reason: Supports multi-user deployments natively by adding basic user tables and composite unique keys `UNIQUE(user_id, email_address)` on accounts. This prevents database constraint collisions and isolates synchronized accounts securely by user-session context.
  - Alternatives considered: Global unique emails (blocks identical inbox setups), physical multi-database routing.
  - Date: 2026-07-18

- Decision: Standardize on Jujutsu (jj) as Version Control System.
  - Reason: Modern, fast, branchless mutation-centric commit architecture that maps natively to git-compatible remotes. Improves developer efficiency via automated local commit logs and stack rebases without pull-request overhead.
  - Alternatives considered: Raw Git CLI, Git + Graphite.
  - Date: 2026-07-18

- Decision: Mail and Calendar as separate Tauri app targets sharing one backend.
  - Reason: Cleaner UX separation — users expect a dedicated mail client and a dedicated calendar client. Each app is independently installable, has a focused component tree, and can be updated independently.
  - Alternatives considered: Single unified app with tab navigation, iframe micro-frontend.
  - Tradeoffs: Two Tauri manifests and two CI build jobs. Each app authenticates independently (separate bearer token).
  - Date: 2026-07-18

- Decision: Threaded email conversations grouped by thread_id.
  - Reason: Matches user mental model from Gmail/Outlook. Reduces list noise for multi-reply conversations.
  - Alternatives considered: Flat message list.
  - Date: 2026-07-20

- Decision: Tiered email body caching — snippets always, full body on-demand.
  - Reason: Prevents unbounded cache growth for high-volume inboxes. Full body only pre-cached for starred/flagged messages.
  - Alternatives considered: Always cache full body, cache nothing.
  - Date: 2026-07-20

- Decision: Provider-native label mirroring (no normalization).
  - Reason: Avoids lossy translation between Gmail labels and Outlook categories. Displayed as pills in the UI directly from the provider string array.
  - Alternatives considered: Normalized internal label system.
  - Date: 2026-07-20

- Decision: Offline-first with local SQLite mutation queue.
  - Reason: Users expect full read access and ability to queue actions (archive, delete, reply) when disconnected from their home server.
  - Alternatives considered: Read-only offline, in-memory queue (lost on app close).
  - Date: 2026-07-20

- Decision: Outbound mail sent directly from Tauri client via provider API.
  - Reason: Keeps backend stateless for outbound. Client already holds the OAuth token; routing through the backend adds latency and complexity.
  - Alternatives considered: Backend SMTP relay.
  - Date: 2026-07-20

- Decision: Single KESTREL_BASE_URL env var as the canonical server address.
  - Reason: One value drives both OAuth redirect URI construction and API base URL for clients. Eliminates misconfiguration surface.
  - Alternatives considered: Separate OAUTH_REDIRECT_URL and API_BASE_URL.
  - Date: 2026-07-20

- Decision: Baked-in plugins for v1, no dynamic plugin loading.
  - Reason: Simplifies v1 build and security model. Gmail and Outlook plugins compiled into the binary. Dynamic loading deferred to post-v1.
  - Alternatives considered: Drop-in .wasm file directory, plugin registry UI.
  - Date: 2026-07-20

- Decision: iOS included as a v1 build target (UX polish deferred).
  - Reason: Tauri Mobile supports iOS; building it in CI from day one avoids later integration pain. UX refinement is a post-v1 milestone.
  - Alternatives considered: iOS as future milestone only.
  - Date: 2026-07-20

- Decision: Platform-native push notifications (WNS / APNs / FCM).
  - Reason: Avoids dependency on a third-party push service. Each platform's native system is used directly via Tauri plugin-notification.
  - Alternatives considered: FCM for all platforms, no push.
  - Date: 2026-07-20

- Decision: Soft sign-out (app-local token cleared; server data retained).
  - Reason: Users can sign back in and immediately have access to synced data without re-running the full OAuth flow.
  - Alternatives considered: Hard sign-out wiping all server data.
  - Date: 2026-07-20

---

# Validation Checklist

Before implementation verify

✓ Technology stack complete
✓ Folder structure defined
✓ Architectural style documented
✓ Security documented
✓ Testing strategy documented
✓ Forbidden technologies documented
✓ Deployment documented
✓ Major decisions recorded
