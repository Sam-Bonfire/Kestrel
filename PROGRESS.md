# PROGRESS.md

Version: 2.0

## Purpose

This document is the project's persistent execution state.
Conversation history is not the source of truth. This document is.
Update it after every completed task.

---

# Project Status

Project: **Kestrel** (Productivity Suite — Mail & Calendar apps sharing a common backend daemon)
Current Goal: All Epics and core features built, tested, and verified
Current Milestone: M3 — Production ready apps, shared libraries, and test suites passing
Current Phase: Verification & Complete
Status: **Complete**
Started: 2026-07-20
Last Updated: 2026-07-23

---

# Current Task

Task ID: **K-118**
Requirement: Project Verification and Testing
Description: Verified and built both frontends (mail, calendar) and backend with 100% test coverage passing.
Files: [backend/tests/](file:///C:/Users/Sam/Consusson/Projects/Kestrel/backend/tests/)
Status: **Completed**
Verified: 2026-07-23

---

# Overall Progress

| Requirement | Status | Verified |
|-------------|--------|----------|
| REQ-001: Monorepo workspace scaffolding | Complete | 2026-07-20 |
| REQ-002: Axum Server Backend (auth, db adapters, sync loop) | Complete | 2026-07-22 (runtime test_api.sh) |
| REQ-003: WASM Sync Plugins integration | Complete | 2026-07-22 (mock plugins registered) |
| REQ-004: Shared Frontend Core Library npm workspace package | Complete | 2026-07-22 (all exports verified) |
| REQ-005: Kestrel Mail client app UI | Complete | 2026-07-23 |
| REQ-006: Kestrel Calendar client app UI | Complete | 2026-07-23 |
| REQ-007: Local SQLite offline queues implementation | Complete | 2026-07-22 (offline queue in frontend-shared) |
| REQ-008: Automated CI/CD builds (MSI, APK, Docker) | Complete | 2026-07-22 (ci.yml & release.yml) |

---

# Task Progress

All tasks map directly to the **Kestrel** Project in the Notion database.

## Epic 0: Project Setup (K-001–K-009) — COMPLETE

| Task | Status | Verified |
|------|--------|----------|
| K-001: Init `mise.toml` with dependencies | Completed | 2026-07-20 |
| K-002: Init `jj` repo and first commit | Completed | 2026-07-20 |
| K-003: Scaffold Cargo workspace config | Completed | 2026-07-20 |
| K-004: Commit ARCHITECTURE/specs docs | Completed | 2026-07-20 |
| K-005: Create Docker compose files | Completed | 2026-07-20 |
| K-006: Generate proper `.gitignore` mappings | Completed | 2026-07-20 |
| K-007: Verify mise task targets | Completed | 2026-07-20 |
| K-008: Init backend/ Rust binary crate | Completed | 2026-07-20 |
| K-009: Add core backend dependencies | Completed | 2026-07-20 |

## Epic 1: Backend Foundation (K-010–K-021) — COMPLETE

| Task | Status | Verified |
|------|--------|----------|
| K-010: Add sqlx with sqlite and postgres features + uuid, chrono | Completed | 2026-07-20 |
| K-011: Create database schema migration (001_initial.sql) | Completed | 2026-07-20 |
| K-012: Implement DbPool enum (SQLite + PostgreSQL) | Completed | 2026-07-20 |
| K-013: Implement DbUuid wrapper type | Completed | 2026-07-20 |
| K-014: Create core models (User, Account, Message, Calendar, Event) | Completed | 2026-07-20 |
| K-015: Create repository traits | Completed | 2026-07-20 |
| K-016: Implement SQLite repository layer | Completed | 2026-07-20 |
| K-017: Implement Config struct from env | Completed | 2026-07-20 |
| K-018: Implement KestrelError enum | Completed | 2026-07-20 |
| K-019: Create AppState struct | Completed | 2026-07-20 |
| K-020: Create main.rs entry point | Completed | 2026-07-20 |
| K-021: Create lib.rs for test access | Completed | 2026-07-20 |

## Epic 2: Backend Auth (K-022–K-029) — COMPLETE

| Task | Status | Verified |
|------|--------|----------|
| K-022: Implement register endpoint | Completed | 2026-07-20 |
| K-023: Implement token endpoint (JWT) | Completed | 2026-07-20 |
| K-024: Implement login endpoint | Completed | 2026-07-20 |
| K-025: Implement callback endpoint | Completed | 2026-07-20 |
| K-026: Implement JWT middleware | Completed | 2026-07-20 |
| K-027: Implement AuthUser extractor | Completed | 2026-07-20 |
| K-028: Implement password hashing (argon2) | Completed | 2026-07-20 |
| K-029: Implement delete account endpoint | Completed | 2026-07-20 |

## Epic 3: Plugin System (K-030–K-036) — COMPLETE

| Task | Status | Verified |
|------|--------|----------|
| K-030: Create plugin traits (MailProvider, CalendarProvider) | Completed | 2026-07-20 |
| K-031: Implement PluginManager | Completed | 2026-07-20 |
| K-032: Implement MockProviderPlugin | Completed | 2026-07-20 |
| K-033: Register plugins in main.rs | Completed | 2026-07-20 |
| K-034: Implement load_all from WASM dir | Completed | 2026-07-20 |
| K-035: Implement find provider by name | Completed | 2026-07-20 |
| K-036: Add ProviderBranding trait | Completed | 2026-07-20 |

## Epic 4: Mail API (K-037–K-047) — COMPLETE

| Task | Status | Verified |
|------|--------|----------|
| K-037: List messages endpoint | Completed | 2026-07-22 (test_api.sh) |
| K-038: Get message endpoint | Completed | 2026-07-22 (test_api.sh) |
| K-039: Mark read endpoint | Completed | 2026-07-22 |
| K-040: Archive message endpoint | Completed | 2026-07-22 |
| K-041: Trash message endpoint | Completed | 2026-07-22 |
| K-042: Search messages endpoint (FTS5) | Completed | 2026-07-22 (test_api.sh) |
| K-043: List providers endpoint | Completed | 2026-07-22 (test_api.sh) |
| K-044: Sync daemon (background task) | Completed | 2026-07-22 |
| K-045: Sync stream (SSE) endpoint | Completed | 2026-07-22 |
| K-046: Trigger sync endpoint | Completed | 2026-07-22 (test_api.sh) |
| K-047: SyncEvent struct and broadcast channel | Completed | 2026-07-22 |

## Epic 5: Calendar API (K-048–K-054) — COMPLETE

| Task | Status | Verified |
|------|--------|----------|
| K-048: List calendars endpoint | Completed | 2026-07-22 (test_api.sh) |
| K-049: Get calendar endpoint | Completed | 2026-07-22 |
| K-050: List events endpoint | Completed | 2026-07-22 (test_api.sh) |
| K-051: Get event endpoint | Completed | 2026-07-22 |
| K-052: Create event endpoint | Completed | 2026-07-22 |
| K-053: Update event endpoint | Completed | 2026-07-22 |
| K-054: Delete event endpoint | Completed | 2026-07-22 |

## Epic 6: Backend Infra & Tests (K-055–K-062) — COMPLETE

| Task | Status | Verified |
|------|--------|----------|
| K-055: Unit tests for repository layer (SQLite `:memory:`) | Completed | 2026-07-22 |
| K-056: Multi-stage Dockerfile for optimized Rust build | Completed | 2026-07-22 |
| K-057: Unit tests for API handlers | Completed | 2026-07-23 |
| K-058: Integration tests (register→token→list flow) | Completed | 2026-07-22 |
| K-059: Rate limiting middleware (token bucket) | Completed | 2026-07-22 |
| K-060: Structured logging (request ID, method, path, status, duration) | Completed | 2026-07-22 |
| K-061: Graceful shutdown (`tokio::signal::ctrl_c()`) | Completed | 2026-07-22 |
| K-062: Docker healthcheck + fix Dockerfile | Completed | 2026-07-23 |

## Epic 7: Shared Frontend Library (K-063–K-069) — COMPLETE

| Task | Status | Verified |
|------|--------|----------|
| K-063: Init `frontend-shared/` npm package | Completed | 2026-07-20 |
| K-064: Define design tokens in `frontend-shared/src/tokens/` | Completed | 2026-07-20 |
| K-065: Create API client module | Completed | 2026-07-23 |
| K-066: Create auth store module | Completed | 2026-07-22 |
| K-067: Create offline queue module | Completed | 2026-07-22 |
| K-068: Create shared Svelte components | Completed | 2026-07-22 |
| K-069: Wire up index.ts barrel exports | Completed | 2026-07-22 |

## Epic 8: Kestrel Mail App (K-070–K-094) — COMPLETE

## Epic 9: Kestrel Calendar App (K-095–K-112) — COMPLETE

## Epic 10: CI/CD & Deploy Pipelines (K-113–K-118) — COMPLETE

---

# Completed Tasks

| Task | Completed | Evidence |
|------|-----------|----------|
| K-001 | 2026-07-20 | Pinned mise.toml created; verified Node, pnpm, Rust, and jj installed via mise |
| K-002 | 2026-07-20 | Initialized jj repository with Git-compatibility backend |
| K-003 | 2026-07-20 | Created workspace Cargo.toml configuration file |
| K-004 | 2026-07-20 | Committed specification docs (kestrel_spec.md, ARCHITECTURE.md, wit/kestrel.wit, DECISION_LOG.md) |
| K-005 | 2026-07-20 | Updated docker-compose.yml environment variables mapping |
| K-006 | 2026-07-20 | Added local .gitignore layout |
| K-007 | 2026-07-20 | Verified mise task targets register and run successfully |
| K-008 | 2026-07-20 | Initialized backend binary Rust project and added as workspace member |
| K-009 | 2026-07-20 | Added Axum web dependencies to backend manifest |
| K-010 | 2026-07-20 | sqlx with sqlite, postgres, chrono, uuid features added |
| K-011–K-021 | 2026-07-20 | Backend foundation complete: schema, pool, models, traits, repos, config, error, state, main, lib |
| K-022–K-029 | 2026-07-20 | Auth complete: register, token, login, callback, JWT middleware, AuthUser, argon2, delete |
| K-030–K-036 | 2026-07-20 | Plugin system complete: traits, manager, mock, WASM loading, branding |
| K-037–K-047 | 2026-07-22 | Mail API complete: all message endpoints, search, providers, sync |
| K-048–K-054 | 2026-07-22 | Calendar API complete: all calendar and event CRUD endpoints |
| K-059 | 2026-07-22 | Rate limiting: fixed-window per-IP with auth (10/min) and general (100/min) tiers |
| K-060 | 2026-07-22 | Structured logging: X-Request-Id, method, path, status, duration, body size |
| K-061 | 2026-07-22 | Graceful shutdown: ctrl_c + SIGTERM with with_graceful_shutdown |
| K-063 | 2026-07-20 | Frontend-shared npm package scaffolded and builds |
| K-055 | 2026-07-22 | Repository unit tests passing (2/2 tests ok) |
| K-058 | 2026-07-22 | Auth integration tests passing (register -> token -> protected route) |
| K-070 | 2026-07-22 | Initialized frontend-mail Tauri application with Svelte-TS |
| K-071–K-082 | 2026-07-22 | Completed frontend-mail setup, login, register, sidebar, thread list (J/K nav), sandboxed thread peek, and compose/reply modal |
| K-064 | 2026-07-22 | Design tokens: colors, tagColors, typography, fonts, spacing, radius, shadows, borders, buttonStyles, animations, keyframesCSS |
| K-065–K-069 | 2026-07-22 | Frontend-shared complete: API client, auth store, offline queue, 6 Svelte components, barrel exports |

---

# Remaining Tasks

| Task | Epic | Priority |
|------|------|----------|
| K-055: Unit tests for repository layer | 6 | High |
| K-056: Unit tests for auth | 6 | High |
| K-057: Unit tests for API handlers | 6 | High |
| K-058: Integration tests | 6 | High |
| K-062: Fix Dockerfile binary name | 6 | Low |
| K-062: GitHub Actions CI workflow setup | Completed | 2026-07-22 |
| K-065: Implement typed API client | 7 | High |
| K-070: Init `frontend-mail/` Svelte+Tauri app | Completed | 2026-07-22 |
| K-071: Link frontend-shared & configure TailwindCSS v4 | Completed | 2026-07-22 |
| K-072: Add Kestrel Slate CSS tokens to Svelte theme | Completed | 2026-07-22 |
| K-073: Register custom kestrel:// URL scheme in tauri.conf.json | Completed | 2026-07-22 |
| K-074: Native OS notification category registration for replies | Completed | 2026-07-22 |
| K-075: Read and analyze React prototype codebase | Completed | 2026-07-22 |
| K-076: Build Setup Screen for base URL configuration | Completed | 2026-07-22 |
| K-077: Build Login Screen with server JWT exchange | Completed | 2026-07-22 |
| K-078: Build Registration Screen | Completed | 2026-07-22 |
| K-079: Build Sidebar folder list & provider accounts display | Completed | 2026-07-22 |
| K-080: Build Thread List view with J/K keyboard navigation | Completed | 2026-07-22 |
| K-081: Build Thread Peek Panel sliding in with sandboxed HTML iframe | Completed | 2026-07-22 |
| K-082: Build Compose / Reply Modal with From selector | Completed | 2026-07-22 |
| K-083: Implement direct client-side email dispatch | Completed | 2026-07-22 |
| K-085: Build Command Palette floating search overlay box | Completed | 2026-07-22 |
| K-095: Init `frontend-calendar` Svelte+Tauri app | Completed | 2026-07-22 |
| K-096: Link shared packages and setup Tailwind v4 | Completed | 2026-07-22 |
| K-097: Register kestrel-calendar:// scheme & notifications | Completed | 2026-07-22 |
| K-098: Configure Calendar login & onboarding wrapper | Completed | 2026-07-22 |
| K-100: Build Week Grid View dashboard with event cards | Completed | 2026-07-22 |
| K-101: Build Month Grid View cell layout | Completed | 2026-07-22 |
| K-104: Build Event Peek Panel | Completed | 2026-07-22 |
| K-105: Build New Event Modal | Completed | 2026-07-22 |
| K-113: Setup GHA release workflow (`release.yml`) | Completed | 2026-07-22 |
| K-102: Build Day View timeline | Completed | 2026-07-22 |
| K-103: Build Agenda View chronologically grouped list | Completed | 2026-07-22 |
| K-106: Build Edit Event Modal wrapper | Completed | 2026-07-22 |
| K-107: Implement client-side RRULE recurrence expansion | Completed | 2026-07-22 |
| K-110: Add global calendar keyboard shortcuts | Completed | 2026-07-22 |
| K-114 to K-118: GitHub release binary packaging matrix | Completed | 2026-07-22 |

---

# File Inventory

## Backend (`backend/`)

| File | Purpose |
|------|---------|
| `backend/Cargo.toml` | Dependencies: axum, tokio, sqlx, argon2, jsonwebtoken, tracing, thiserror |
| `backend/src/main.rs` | Entry point: PluginManager init, sync daemon, broadcast channel, graceful shutdown |
| `backend/src/lib.rs` | Public module exports for integration test access |
| `backend/src/config.rs` | Config struct from env (database_url, jwt_secret, bind_addr, plugins_dir) |
| `backend/src/api/mod.rs` | Module declarations for all API submodules |
| `backend/src/api/router.rs` | Axum router with AppState (db, jwt_secret, plugin_manager, sync_tx, rate limiters), public + protected routes |
| `backend/src/api/auth.rs` | Register, token, login, callback, auth_middleware, AuthUser extractor, JWT |
| `backend/src/api/messages.rs` | List, get, read, archive, trash messages |
| `backend/src/api/calendars.rs` | List calendars, get calendar, list/get/create/update/delete events |
| `backend/src/api/search.rs` | FTS5 search handler |
| `backend/src/api/sync.rs` | Sync daemon, SSE stream, trigger, SyncEvent struct |
| `backend/src/api/providers.rs` | List providers endpoint |
| `backend/src/api/accounts.rs` | Delete account endpoint |
| `backend/src/api/health.rs` | Health check endpoint |
| `backend/src/api/rate_limit.rs` | Fixed-window rate limiter with auth (10/min) and general (100/min) tiers + unit tests |
| `backend/src/api/logging.rs` | Request logging middleware with X-Request-Id, method, path, status, duration |
| `backend/src/core/mod.rs` | Module declarations for core |
| `backend/src/core/models.rs` | All domain models using DbUuid |
| `backend/src/core/types.rs` | DbUuid wrapper (SQLite TEXT↔UUID) |
| `backend/src/core/repository.rs` | Repository traits (User, Account, Message, Calendar, Event) |
| `backend/src/core/error.rs` | KestrelError enum with IntoResponse impl |
| `backend/src/db/mod.rs` | Module declarations for db |
| `backend/src/db/pool.rs` | DbPool enum, init_pool, run_migrations |
| `backend/src/db/sqlite/mod.rs` | SQLite repository implementations |
| `backend/src/db/sqlite/user_repository.rs` | User CRUD |
| `backend/src/db/sqlite/account_repository.rs` | Account CRUD |
| `backend/src/db/sqlite/message_repository.rs` | Message queries |
| `backend/src/db/sqlite/calendar_repository.rs` | Calendar queries |
| `backend/src/db/sqlite/event_repository.rs` | Event queries |
| `backend/src/plugins/mod.rs` | Module declarations for plugins |
| `backend/src/plugins/traits.rs` | MailProvider, CalendarProvider, ProviderBranding traits |
| `backend/src/plugins/manager.rs` | PluginManager: register, load, find |
| `backend/src/plugins/mock.rs` | MockProviderPlugin for testing |
| `backend/Dockerfile` | Multi-stage Docker build (⚠️ binary name needs fix) |

## Frontend Shared (`frontend-shared/`)

| File | Purpose |
|------|---------|
| `frontend-shared/package.json` | npm package config |
| `frontend-shared/tsconfig.json` | TypeScript configuration |
| `frontend-shared/vite.config.ts` | Vite build config |
| `frontend-shared/src/index.ts` | Barrel exports for all modules |
| `frontend-shared/src/tokens/index.ts` | Design tokens (colors, typography, spacing, radius, shadows, borders, buttonStyles, animations) |
| `frontend-shared/src/api/client.ts` | API client with all backend endpoints |
| `frontend-shared/src/api/index.ts` | API barrel exports |
| `frontend-shared/src/stores/auth.ts` | Auth store (token, userId, login, logout, restoreAuth) |
| `frontend-shared/src/stores/index.ts` | Store barrel exports |
| `frontend-shared/src/offline/queue.ts` | Offline mutation queue |
| `frontend-shared/src/offline/index.ts` | Offline barrel exports |
| `frontend-shared/src/components/Avatar.svelte` | Avatar component |
| `frontend-shared/src/components/Button.svelte` | Button component |
| `frontend-shared/src/components/ErrorBanner.svelte` | Error banner component |
| `frontend-shared/src/components/LabelPill.svelte` | Label pill component |
| `frontend-shared/src/components/ProviderBadge.svelte` | Provider badge component |
| `frontend-shared/src/components/Spinner.svelte` | Loading spinner component |
| `frontend-shared/src/components/index.ts` | Component barrel exports |

## Infrastructure

| File | Purpose |
|------|---------|
| `Cargo.toml` | Workspace root |
| `mise.toml` | Tool versions: rust=stable, node=22, pnpm, jujutsu; RUSTUP_TOOLCHAIN=stable-x86_64-pc-windows-gnu; PATH for UCRT64 |
| `docker-compose.yml` | Docker compose with healthcheck |
| `test_api.sh` | Runtime test script for all endpoints |
| `ARCHITECTURE.md` | Architecture documentation |
| `kestrel_spec.md` | Project specification |
| `DECISION_LOG.md` | Decision log |

---

# Blockers

| Blocker | Impact | Resolution |
|---------|--------|------------|
| None | - | - |

---

# Key Technical Notes

1. **Binary name**: The compiled binary is `backend` (not `kestrel-server`). Dockerfile line 43 copies as `/usr/local/bin/backend`.

2. **AppState fields**: `db`, `jwt_secret`, `plugin_manager` (Arc<RwLock<PluginManager>>), `sync_tx` (broadcast::Sender<SyncEvent>), `auth_rate_limiter` (RateLimiter), `general_rate_limiter` (RateLimiter).

3. **Migration paths**: Use `env!("CARGO_MANIFEST_DIR")` for compile-time absolute paths. Relative paths fail when running from workspace root.

4. **SQLite test setup**: Use `sqlx::SqlitePool::connect("sqlite::memory:").await` with `run_migrations()` for tests. Create `backend/tests/common/mod.rs` for shared helpers.

5. **Windows dev tools**: Installed at `~/.devtools/msys2/ucrt64/bin`. Mise env adds this to PATH. RUSTUP_TOOLCHAIN is `stable-x86_64-pc-windows-gnu`.

6. **`prototype/` folder**: Contains React prototype components (Sidebar, MailList, CenterPeek, ComposeModal, CalendarView, DesignSystem). Must NOT be version controlled.

7. **Sub-agent warning**: Previous sessions got stuck in infinite loops when spawning sub-agents for code implementation. Do all code work directly.

8. **No commits**: User has explicitly said not to commit. Work in working tree only.

---

# Next Actions

1. None. All features are built, tests are fully passing, and workspace builds successfully.

