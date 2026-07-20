---

Version: 1.0

---

# Purpose

This document is the authoritative record of engineering decisions made during the project.

It preserves architectural consistency across long-running development sessions.

Conversation history is NOT the source of truth.

This document is.

Every significant decision must be recorded before implementation proceeds.

---

# Rules

Every decision must contain

- Unique ID
- Status
- Decision
- Reason
- Alternatives Considered
- Consequences
- Affected Components
- User Approval Status

Never silently overwrite a decision.

Never remove decisions.

If a decision changes, create a new revision.

---

# Decision Status

Allowed values: Proposed, Approved, Implemented, Deprecated, Rejected, Superseded

---

# Decision Categories

Architecture, Technology, Database, API, Frontend, Backend, Security, Infrastructure, Performance, Testing, Deployment, Product, Business Logic, Documentation, Other

---

# Active Decisions

| ID | Category | Decision | Status |
| ---- | ---------- | ---------- | -------- |
| DEC-001 | Product | Rename project from AetherMail to Kestrel | Approved |
| DEC-002 | Product | Expand project scope to include Calendar client alongside Email | Approved |
| DEC-003 | Infrastructure | Utilize Cloudflare Tunnels for OAuth callbacks redirect endpoints | Approved |
| DEC-004 | Architecture | Recurrence expansion: Direct API when online, Svelte client-side calculation when offline | Approved |
| DEC-005 | Database | Last-Write-Wins (LWW) with historical revision table for conflict resolution | Approved |
| DEC-006 | Security | Sandbox client-side iframe for HTML email rendering | Approved |
| DEC-007 | Architecture | Server-Sent Events (SSE) + Firebase Cloud Messaging (FCM) updates | Approved |
| DEC-008 | Architecture | Direct URL redirects for attachments, cached on Tauri clients for offline | Approved |
| DEC-009 | Technology | Use `mise` for tool version management and task runner execution | Approved |
| DEC-010 | Infrastructure | Deploy backend daemon using multi-stage Docker / Docker Compose | Approved |
| DEC-011 | Architecture | Server-side WASM plugin architecture for mail and calendar providers | Approved |
| DEC-012 | Frontend | Standardize on Skeleton UI with a custom Notion/Morgen styling theme | Approved |
| DEC-013 | Database | Repository Abstraction with SQLite and PostgreSQL dual SQLx backends | Approved |
| DEC-014 | Security | Optional `secretspec` startup secrets manager integration | Approved |
| DEC-015 | Architecture | Centralized Provider Token Vault with Unified Client Device Session | Approved |
| DEC-016 | Database | Rudimentary Logical Multi-Tenancy Partitioning with Composite Uniqueness | Approved |
| DEC-017 | Technology | Standardize on Jujutsu (jj) as Version Control System | Approved |
| DEC-018 | Frontend | Dynamic WASM Plugin Branding Exports for Provider Buttons/Banners | Approved |

---

# Pending Decisions

None.

---

# Superseded Decisions

None.

---

# Decision Log Details

## DEC-001: Rename project from AetherMail to Kestrel

- **Category:** Product
- **Status:** Approved
- **Date:** 2026-07-18
- **Author:** Antigravity AI
- **Decision:** Rename the self-hosted client project from "AetherMail" to "Kestrel".
- **Reason:** Requested by the user.
- **Alternatives Considered:** Keeping AetherMail.
- **Consequences:** Updated documentation and spec to Kestrel.
- **Affected Components:** All files.
- **User Approval:** Approved (2026-07-18)

---

## DEC-002: Expand project scope to include Calendar client alongside Email

- **Category:** Product
- **Status:** Approved
- **Date:** 2026-07-18
- **Author:** Antigravity AI
- **Decision:** Include calendar integration alongside email synchronization.
- **Reason:** Provide a comprehensive self-hosted productivity workspace.
- **Alternatives Considered:** Separated apps.
- **Consequences:** New database tables, expanded API, new UI views.
- **Affected Components:** DB, Backend, Frontend.
- **User Approval:** Approved (2026-07-18)

---

## DEC-003: Utilize Cloudflare Tunnels for OAuth callbacks redirect endpoints

- **Category:** Infrastructure
- **Status:** Approved
- **Date:** 2026-07-18
- **Author:** Antigravity AI
- **Decision:** Register public URLs exposed via Cloudflare Tunnels on the NAS as authorized OAuth redirect endpoints with Google Cloud and Microsoft Entra ID.
- **Reason:** Enables standard public authorization code callbacks directly to the self-hosted NAS daemon.
- **Alternatives Considered:** Local protocol loops, gateway proxies, manual copy-paste.
- **Consequences:** Simplifies client auth flows. Requires running Cloudflared on the NAS.
- **Affected Components:** Backend auth routers, Infrastructure configuration.
- **User Approval:** Approved (2026-07-18)

---

## DEC-004: Recurrence expansion: Direct API when online, Svelte client-side calculation when offline

- **Category:** Architecture
- **Status:** Approved
- **Date:** 2026-07-18
- **Author:** Antigravity AI
- **Decision:** When connected to the internet, fetch expanded instances directly from upstream APIs. When disconnected, calculate calendar instances dynamically on the Svelte client using cached master events and RRULE structures.
- **Reason:** Balances accurate upstream recurrence/exceptions handling with absolute offline client readability.
- **Alternatives Considered:** Server-side pre-expansion cache, direct API only.
- **Consequences:** Client needs an offline fallback engine (e.g. `rrule.js` integration).
- **Affected Components:** Frontend Calendar Grid, Backend Event API.
- **User Approval:** Approved (2026-07-18)

---

## DEC-005: Last-Write-Wins (LWW) with historical revision table for conflict resolution

- **Category:** Database
- **Status:** Approved
- **Date:** 2026-07-18
- **Author:** Antigravity AI
- **Decision:** Resolve client write conflicts using Last-Write-Wins based on timestamps. When overwriting records, backup previous states into a local `historical_revisions` table.
- **Reason:** Simplifies client sync logic while preserving data integrity in case of collision overrides.
- **Alternatives Considered:** Interactive visual merge, vector clocks.
- **Consequences:** New SQLite table `historical_revisions` is created.
- **Affected Components:** Database schema, Sync engine.
- **User Approval:** Approved (2026-07-18)

---

## DEC-006: Sandbox client-side iframe for HTML email rendering

- **Category:** Security
- **Status:** Approved
- **Date:** 2026-07-18
- **Author:** Antigravity AI
- **Decision:** Render HTML email bodies inside a sandboxed HTML `<iframe>` with scripts disabled and standard browser controls.
- **Reason:** Protects client environment from malicious script execution and simplifies layout isolation.
- **Alternatives Considered:** Backend sanitization, Markdown translation.
- **Consequences:** Sandboxed iframe handles email rendering dynamically.
- **Affected Components:** Frontend Preview Component.
- **User Approval:** Approved (2026-07-18)

---

## DEC-007: Server-Sent Events (SSE) + Firebase Cloud Messaging (FCM) updates

- **Category:** Architecture
- **Status:** Approved
- **Date:** 2026-07-18
- **Author:** Antigravity AI
- **Decision:** Push real-time sync signals to active clients using unidirectional SSE connections. For background mobile updates, trigger background synchronizations using FCM.
- **Reason:** Highly efficient resource usage compared to WebSockets, while natively supporting background fetches on Android.
- **Alternatives Considered:** Bidirectional WebSockets, REST polling.
- **Consequences:** Server implements SSE streaming; Mobile app registers FCM background handler.
- **Affected Components:** Backend APIs, Frontend Connection Service.
- **User Approval:** Approved (2026-07-18)

---

## DEC-008: Direct URL redirects for attachments, cached on Tauri clients for offline

- **Category:** Architecture
- **Status:** Approved
- **Date:** 2026-07-18
- **Author:** Antigravity AI
- **Decision:** Serve attachments by redirecting clients to temporary direct links generated from upstream APIs. Tauri clients will save downloaded attachments to client-side disk storage for subsequent offline access.
- **Reason:** Zero NAS disk storage utilization for files while preserving client-side offline access.
- **Alternatives Considered:** NAS on-demand caching, NAS full pre-fetching.
- **Consequences:** Tauri app implements a local cache directory lookup for files.
- **Affected Components:** Frontend Attachment component, Backend Attachment API.
- **User Approval:** Approved (2026-07-18)

---

## DEC-009: Use `mise` for tool version management and task runner execution

- **Category:** Technology
- **Status:** Approved
- **Date:** 2026-07-18
- **Author:** Antigravity AI
- **Decision:** Use `mise` (formerly `rtx`) as the project tool version manager (for Node and Rust compiler toolchains) and core task runner configurations (building, developing, and testing backend and frontend assets).
- **Reason:** Simplifies multi-language environment setup across Windows and NAS environments, standardizing project management tasks in a unified `mise.toml` declaration.
- **Alternatives Considered:** Makefiles, script folders, raw cargo/npm commands.
- **Consequences:** Added `mise.toml` configuration to root workspace directory.
- **Affected Components:** Toolchains, developer configuration.
- **User Approval:** Approved (2026-07-18)

---

## DEC-010: Deploy backend daemon using multi-stage Docker / Docker Compose

- **Category:** Infrastructure
- **Status:** Approved
- **Date:** 2026-07-18
- **Author:** Antigravity AI
- **Decision:** Package the Rust backend daemon using a multi-stage Dockerfile utilizing `cargo-chef` to cache dependencies, generating a minimized runtime container running on Debian slim. Route incoming traffic from the host's existing `cloudflared` service directly to the exposed backend container port.
- **Reason:** Standardizes deployment on the remote NAS server while reusing the existing server-level Cloudflare Tunnel environment.
- **Alternatives Considered:** Packaging cloudflared as a sidecar container, bare-metal deployment.
- **Consequences:** Created `backend/Dockerfile` and a single-service `docker-compose.yml` config.
- **Affected Components:** Build tooling, infrastructure deployment setups.
- **User Approval:** Approved (2026-07-18)

---

## DEC-011: Server-side WASM plugin architecture for mail and calendar providers

- **Category:** Architecture
- **Status:** Approved
- **Date:** 2026-07-18
- **Author:** Antigravity AI
- **Decision:** Embed a WebAssembly (WASM) runtime engine (e.g. `wasmtime` or `wasmer`) within the Rust backend daemon. All third-party mail providers (Gmail, Microsoft Graph, Yahoo, custom IMAP) and calendar integrations (Google Calendar, Outlook Calendar, Notion Databases) will be compiled as independent, sandboxed `.wasm` plugins. The core daemon loads these plugins dynamically from a `/plugins` folder at runtime via standardized WASM Interface Types (WIT).
- **Reason:** Ensures absolute extensibility without altering core source code, isolates third-party code in a secure sandbox, and enables community-driven development in multiple languages.
- **Alternatives Considered:** Dynamic Link Libraries (`.so`/`.dll` loading), Subprocess RPC orchestration.
- **Consequences:** Core codebase must define WIT specs (`kestrel.wit`) and expose host functions for DB access and networking triggers. Plugins must compile to `wasm32-wasi`.
- **Affected Components:** Backend sync loops, providers module, runtime build systems.
- **User Approval:** Approved (2026-07-18)

---

## DEC-012: Standardize on Skeleton UI with a custom Notion/Morgen styling theme

- **Category:** Frontend
- **Status:** Approved
- **Date:** 2026-07-18
- **Author:** Antigravity AI
- **Decision:** Standardize on Skeleton UI as Kestrel's primary component library. A custom styling theme (defined via Skeleton tokens for colors, typography, roundness, and spacing) will be established to match the design aesthetics of Notion Mail, Notion Calendar, and Morgen.
- **Reason:** Skeleton UI provides a Svelte-native, batteries-included, and opinionated foundation with complex pre-built components (drawers, dialogs, dropdowns, themes) while Tailwind design tokens make it easy to adapt to high-end minimalist design standards without style conflict.
- **Alternatives Considered:** shadcn-svelte (less batteries-included), Flowbite Svelte (too rigid).
- **Consequences:** UI theme will configure Tailwind theme integrations and custom tokens files.
- **Affected Components:** Frontend styling engines, Svelte layouts.
- **User Approval:** Approved (2026-07-18)

---

## DEC-013: Repository Abstraction with SQLite and PostgreSQL dual SQLx backends

- **Category:** Database
- **Status:** Approved
- **Date:** 2026-07-18
- **Author:** Antigravity AI
- **Decision:** Design a clean Repository boundary in Kestrel's core using traits. We will write two separate compile-time verified database integration packages using SQLx: one optimized for a local file-based SQLite cache and one optimized for enterprise PostgreSQL deployments. The system detects the database type from the environment connection string and injects the corresponding repository implementation dynamically at runtime.
- **Reason:** Preserves SQLx's compile-time safety and high performance by executing native dialect-specific SQL optimizations (e.g. SQLite's FTS5 engine vs Postgres's native full-text search) instead of relying on a translation ORM.
- **Alternatives Considered:** Standard ORM abstraction (SeaORM / Diesel), single database constraint.
- **Consequences:** Added separate migration folders (`migrations/sqlite` and `migrations/postgres`) to backend, and defined parallel SQL implementation modules.
- **Affected Components:** Backend database adapters, SQLx configurations, repository code blocks.
- **User Approval:** Approved (2026-07-18)

---

## DEC-014: Optional `secretspec` startup secrets manager integration

- **Category:** Security
- **Status:** Approved
- **Date:** 2026-07-18
- **Author:** Antigravity AI
- **Decision:** Support retrieving sensitive keys (OAuth client IDs/secrets, session cookies, FCM keys) at backend startup using an optional integration wrapper resembling `secretspec`. If a configuration spec is present, credentials are dynamically resolved from the specified vault or secret provider at startup. If absent, the daemon falls back to standard Docker environment variables.
- **Reason:** Standardizes security integration configs for advanced deployments without forcing local developers/self-hosters to set up external secret architectures.
- **Alternatives Considered:** Required Vault setups, file-bound secrets only.
- **Consequences:** Bootstrapper executes the secret load routine synchronously before launching background threads. Exposes client keys securely to dynamic WASM plugins via a host call.
- **Affected Components:** Backend startup bootstrap logic, WIT definitions, config module.
- **User Approval:** Approved (2026-07-18)

---

## DEC-015: Centralized Provider Token Vault with Unified Client Device Session

- **Category:** Architecture
- **Status:** Approved
- **Date:** 2026-07-18
- **Author:** Antigravity AI
- **Decision:** Store and manage all external provider auth tokens (Google, Microsoft, etc.) in the centralized backend database on the NAS server, rather than locally inside individual desktop/mobile apps. Once a mail account is authorized by any client, it becomes instantly synced and accessible by all other authenticated client instances connecting to the same server.
- **Reason:** Automatically propagates mail/calendar access to all client devices, removes the need for repeating OAuth sign-ins on multiple platforms, and keeps refresh tokens secured in a single host vault.
- **Alternatives Considered:** Client-side token storage (requiring cross-device encryption exchanges or repeating oauth logs per client).
- **Consequences:** Client apps use a unified session token/cookie to authenticate with the Kestrel daemon. When a client adds an account, it updates the central database and triggers immediate updates globally on other screens.
- **Affected Components:** Backend account management API, SQLite/Postgres schemas, Client authentication logic.
- **User Approval:** Approved (2026-07-18)

---

## DEC-016: Rudimentary Logical Multi-Tenancy Partitioning with Composite Uniqueness

- **Category:** Database
- **Status:** Approved
- **Date:** 2026-07-18
- **Author:** Antigravity AI
- **Decision:** Integrate a rudimentary multi-tenancy layer at the database level by introducing a basic `users` table and associating a `user_id` foreign key with the centralized `accounts` table. To allow multiple independent users to pair the same email address without collisions while strictly securing separate sessions, the `accounts` table uses a composite unique constraint on `UNIQUE (user_id, email_address)` instead of a global unique constraint on email addresses.
- **Reason:** Prepares the system core schema natively for multi-user deployments while preventing data leakage. Separate tokens, configurations, and messages are generated per user-account linkage (UUID) and isolated at the database level.
- **Alternatives Considered:** Database-per-tenant isolation, strict single-user constraints, global unique email addresses (which would block different users from connecting the same email).
- **Consequences:** Database migrations include the `users` creation and composite unique index constraint on `accounts(user_id, email_address)`. API handlers filter account queries by the paired device's associated user context.
- **Affected Components:** SQLite/Postgres migration files, Repository select queries, API auth filters.
- **User Approval:** Approved (2026-07-18)

---

## DEC-017: Standardize on Jujutsu (jj) as Version Control System

- **Category:** Technology
- **Status:** Approved
- **Date:** 2026-07-18
- **Author:** Antigravity AI
- **Decision:** Standardize on Jujutsu (jj) as the project's primary Version Control System (VCS), interacting with remote GitHub origins via its native Git-compatible backend.
- **Reason:** Jujutsu provides a modern, fast, and branchless version control model. Its built-in support for implicit automatic commits, local stacked modifications, and representable conflict states natively supports single-developer velocity and structural branch graph integrity without requiring external PR staging frameworks.
- **Alternatives Considered:** Raw Git CLI, Git + Graphite.
- **Consequences:** Developers and AI agents will format repository sync scripts and commands using `jj` commands instead of raw `git` commands where appropriate.
- **Affected Components:** Developer tools, project guidelines.
- **User Approval:** Approved (2026-07-18)

---

## DEC-018: Dynamic WASM Plugin Branding Exports for Provider Buttons/Banners

- **Category:** Frontend
- **Status:** Approved
- **Date:** 2026-07-18
- **Author:** Antigravity AI
- **Decision:** Export a dynamic branding metadata interface from WASM plugins. This allows the backend to query visual parameters (name, button color, button text, logo SVG) from loaded guests at boot, serving them over an API endpoint so client frontends can render custom OAuth login buttons/banners dynamically.
- **Reason:** Eliminates the need for client-side hardcoding of provider identities, allowing new third-party plugins to register dynamic buttons on frontend login pages seamlessly without modifying the client app source code.
- **Alternatives Considered:** Client-side static logo configurations, hardcoded OAuth lists.
- **Consequences:** Plugins must export a `provider-branding` interface in `wit/kestrel.wit`. The backend exposes `/api/v1/providers` listing these configurations to client screens.
- **Affected Components:** `wit/kestrel.wit`, Svelte Login UI, backend provider API.
- **User Approval:** Approved (2026-07-18)

## DEC-019: Standardize Backend Auth on the `better-auth` Rust Port with API Key Support

- **Category:** Security / Backend
- **Status:** Approved
- **Date:** 2026-07-19
- **Author:** Antigravity AI
- **Decision:** Use the `better-auth` Rust port to manage user authentication, session cookies, and API key lifecycle management.
- **Reason:** Provides a declarative, type-safe authentication engine that integrates cleanly with Axum and SQLx. Built-in API key support allows users to generate scoped tokens for external automation scripts and plugins without exposing raw session credentials.
- **Alternatives Considered:** Modular stack (`axum-login` + `tower-sessions`), custom JWT implementation.
- **Consequences:** Backend auth routing delegates to `better-auth` handlers. Database schemas will incorporate `better-auth` required tables (sessions, api_keys).
- **Affected Components:** Backend auth module, Axum middleware, DB migrations.
- **User Approval:** Approved (2026-07-19)

---

## DEC-020: Tauri v2 Native Notifications with Interactive Custom Actions

- **Category:** Frontend / Tauri
- **Status:** Approved
- **Date:** 2026-07-19
- **Author:** Antigravity AI
- **Decision:** Implement desktop and mobile OS notifications using `@tauri-apps/plugin-notification`. Both app targets (`kestrel-mail` and `kestrel-calendar`) will register custom interactive action types (e.g., inline text replies, archive buttons, snooze actions) at application startup.
- **Reason:** Enables users to perform triage actions (replying to emails, marking reminders read) directly from their system notification prompts on Windows and Android without switching context or opening the application window.
- **Alternatives Considered:** Basic text-only notifications, custom in-app floating toasts only.
- **Consequences:** Must declare `notification:default` in Tauri capabilities. Frontend must implement event listeners to bridge OS notification action payloads to backend REST calls.
- **Affected Components:** `src-tauri/capabilities/default.json`, Svelte root layouts, API client services.
- **User Approval:** Approved (2026-07-19)

---

## DEC-021: Centralized Customizable Shortcut Registry with DOM Input Guarding

- **Category:** Frontend / UI
- **Status:** Approved
- **Date:** 2026-07-19
- **Author:** Antigravity AI
- **Decision:** Manage all keyboard navigation and UI shortcuts via a centralized Svelte reactive store coupled with the `tinykeys` library. The listener engine will enforce a strict DOM Input Guard and support a "recording mode" state for user customization.
- **Reason:** Hardcoding shortcuts prevents user customization and leads to input collisions. A centralized registry allows the Command Palette, shortcut cheatsheet modal, and window event listeners to read from a single source of truth, dynamically updating whenever keybindings are modified.
- **Alternatives Considered:** Static `keydown` window event listeners, scattered component-level shortcuts.
- **Consequences:** All keyboard shortcuts must be registered in the central shortcut store rather than bound locally in components. The engine must check `document.activeElement` before firing non-modifier sequences.
- **Affected Components:** `frontend-shared` stores, Command Palette UI, Layout wrappers.
- **User Approval:** Approved (2026-07-19)

---

## DEC-022: Relegate Standalone CalDAV Server Exposure to Long-Term Roadmap

- **Category:** Architecture / Roadmap
- **Status:** Approved
- **Date:** 2026-07-19
- **Author:** Antigravity AI
- **Decision:** Exposing Kestrel's backend as a public, third-party accessible CalDAV server (via crates like `mailrs-dav`) is explicitly out of scope for initial releases and marked as a long-term roadmap goal.
- **Reason:** Keeps the core Axum daemon focused entirely on serving the native Kestrel client targets over optimized REST/SSE endpoints. Eliminates immediate requirements for complex XML WebDAV parsing, complex routing redirects, and specialized edge-caching bypass rules.
- **Alternatives Considered:** Building a hybrid REST/CalDAV server from Phase 1.
- **Consequences:** Third-party calendar clients (Apple Calendar, Thunderbird) will not connect directly to the Kestrel daemon in initial phases; synchronization happens strictly between Kestrel clients and the Kestrel backend daemon.
- **Affected Components:** Backend routing, Phase release milestones.
- **User Approval:** Approved (2026-07-19)

---

# Change Log

| Date | Change | Reason | Changed By | Affected Decisions |
|------|--------|--------|------------|--------------------|
| 2026-07-18 | Initial log creation | Record renaming and scope expansion decisions | Antigravity AI | DEC-001, DEC-002 |
| 2026-07-18 | Record design decisions from user interview | Document core architecture specifics for Kestrel | Antigravity AI | DEC-003 to DEC-008 |
| 2026-07-18 | Document toolchain manager selection | Record decision to standardise on `mise` for management | Antigravity AI | DEC-009 |
| 2026-07-18 | Document containerization strategy | Record decision to deploy via multi-stage Docker Compose | Antigravity AI | DEC-010 |
| 2026-07-18 | Document WASM plugin architecture | Record decision to standardise on server-side WASM dynamic loading | Antigravity AI | DEC-011 |
| 2026-07-18 | Document UI component library selection | Standardize on Skeleton UI custom theme framework | Antigravity AI | DEC-012 |
| 2026-07-18 | Document database cross-compatibility design | Standardize on SQLx dual-backend repository traits | Antigravity AI | DEC-013 |
| 2026-07-18 | Document secrets manager integration | Introduce startup-time optional secret loader specs | Antigravity AI | DEC-014 |
| 2026-07-18 | Document centralized token synch | Establish server-authoritative unified device accounts | Antigravity AI | DEC-015 |
| 2026-07-18 | Document logical multi-tenancy partitioning | Introduce basic user partitions to schemas and accounts | Antigravity AI | DEC-016 |
| 2026-07-18 | Document VCS Selection | Standardize on Jujutsu (jj) version control | Antigravity AI | DEC-017 |
| 2026-07-18 | Add dynamic provider branding | Allow WASM plugins to define custom buttons/banners | Antigravity AI | DEC-018 |
