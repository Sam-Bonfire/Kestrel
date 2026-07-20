# PROJECT_RULES.md

Version: 1.0

---

# Purpose

This document defines the engineering standards for this project.

Unlike ARCHITECTURE.md, which defines WHAT the project is built with, this document defines HOW code is written.

These rules apply to every file unless explicitly overridden by the user.

---

# Runtime Rules

This document is a living document.

Update it whenever

- Coding conventions change
- New patterns are adopted
- New standards are introduced
- User preferences change

Never silently violate these rules.

---

# Project Overview

Project Name: Kestrel

Repository: local (managed via Jujutsu)

Primary Goal: A unified mail and calendar client with premium UX.

Target Users: Private self-hosters.

Current Phase: Architecture & Design

---

# Engineering Philosophy

Prefer readability over cleverness.

Prefer explicit code over implicit behavior.

Prefer maintainability over temporary hacks.

Prefer composition over inheritance.

Prefer small modules over large files.

---

# Code Organization

Maximum file size: 400 lines

Maximum function size: 50 lines

Maximum class size: N/A (prefer Rust structs and Svelte modules)

Maximum nesting depth: 4 levels

Maximum parameter count: 5 parameters

Preferred module size: Small, single-responsibility files

Preferred folder organization: Grouped by component/feature under backend/frontend folders

Rules for splitting files: Split when a file exceeds 400 lines or addresses multiple distinct domain concerns

---

# Naming Conventions

Variables: `snake_case` in Rust, `camelCase` in TS/JS

Functions: `snake_case` in Rust, `camelCase` in TS/JS

Classes: `PascalCase` in TS, Rust structs are `PascalCase`

Interfaces: `PascalCase`

Enums: `PascalCase` for enum names, `PascalCase` or `UPPERCASE` for members

Types: `PascalCase`

Constants: `SCREAMING_SNAKE_CASE`

Files: `snake_case.rs` in Rust, `kebab-case.svelte` or `kebab-case.ts` in TS/JS

Folders: `kebab-case` or `snake_case` (consistent per module)

Database Tables: `snake_case` (plural, e.g. `messages`, `accounts`, `calendar_events`)

API Routes: `kebab-case` with `/api/v1` prefix

Environment Variables: `SCREAMING_SNAKE_CASE`

---

# Code Style

Formatting: Standard rustfmt for Rust, Prettier for frontend

Indentation: 4 spaces for Rust, 2 spaces for Svelte/TS/HTML/CSS

Quotes: Double quotes in Rust, single quotes preferred in TS/JS unless formatting HTML attributes

Semicolons: Required in TS/JS, standard semicolon rules in Rust

Trailing commas: Always in Rust multi-line structures, preferred in TS/JS multi-line arrays/objects

Import ordering: Core/Standard libraries, external dependencies, internal modules

Export ordering: Keep public structures grouped logically

Comments: Use comments only to explain "why" something is done, not "what" the code does (let code be self-documenting)

Documentation: Use docstrings for public-facing interface traits and types

Blank lines: Maximum 1 consecutive blank line

Line length: Max 120 characters

---

# Error Handling

Preferred error strategy: Explicit error propagation (`Result<T, E>`) in Rust; Try-Catch with custom error types in frontend

Typed errors: Create domain-specific error enums in Rust (e.g. `KestrelError`) implementing `thiserror`

Custom errors: Avoid catching-all or ignoring errors unless specifically logged

Logging: Log errors at warning/error level with structured context; trace details at debug/trace level

Recovery: Graceful degradation (e.g. show offline message if sync fails, do not crash UI)

Retries: Exponential backoff for network-related sync failures

User-facing errors: Return sanitized, helpful messages to the user; keep stack traces in logs

Developer-facing errors: Detailed errors logged to terminal/file

---

# Logging

When to log: App startup, sync start/stop, database migration, API requests, errors, external API call metrics

What to log: Request path, method, status code, error details, latency

What never to log: User passwords, OAuth access/refresh tokens, email body content, sensitive calendar event details

Log format: JSON log formatting in production, human-readable stdout in local development

Log levels: Error (system issues), Warn (sync failure/rate limits), Info (standard operations), Debug (sync loop queries), Trace (detailed token lifetimes)

Correlation IDs: Pass request-based correlation IDs to trace operations from client to server logs

---

# API Standards

REST: Follow standard REST semantics (GET for retrieve, POST for mutate, DELETE for remove)

GraphQL: N/A

RPC: N/A

Versioning: Version URL path (e.g., `/api/v1/...`)

Pagination: Cursor-based for message list; window/time-range filters for calendar events

Filtering: Query params for statuses (unread, archived, calendars)

Sorting: Standard descending date order for emails and ascending for upcoming events

Validation: Structured JSON body validation using serde/validator in Rust

Error responses: Consistent `{ error, message }` JSON payloads

Status codes: 200 OK, 201 Created, 400 Bad Request, 401 Unauthorized, 403 Forbidden, 404 Not Found, 500 Internal Server Error

---

# Database Standards

Migration strategy: SQLx migration files run automatically at server startup

Naming: Pluralized snake_case for tables, snake_case for columns

Transactions: Use SQLx Transactions for atomic write batches (e.g. saving an email and updating its FTS search index)

Repository usage: All database queries must go through repository structures implementing domain traits

Indexes: Index foreign keys and queried flags (e.g., `is_read`, `is_archived`, event start/end times)

Soft delete policy: Messages are marked `is_deleted` or moved to a trash folder structure; hard deleted only on purge

Audit fields: Include `created_at` and `updated_at` (unix epoch integer) on state tables

---

# Frontend Standards

Component size: Max 200 lines preferred for individual Svelte files; split logic into helper modules if too large

Hooks: Keep reactivity clean using Svelte's reactive declarations (`$:`) and custom stores

Styling: Strictly TailwindCSS v4 classes; use CSS custom properties for theme colors

Accessibility: Ensure interactive elements have correct ARIA tags, labels, and full tab-index navigation support

Responsiveness: Mobile-first responsive grids/flex layouts that scale smoothly from Android mobile screen sizes up to ultrawide Windows desktop viewports

State management: Centralized Svelte writable/readable stores for cached emails and calendar events

Forms: Simple HTML forms validating values client-side before sending JSON payloads

Loading states: Skeleton load indicators for async API fetches

Empty states: Clear informative empty state views (e.g., "Inbox Zero", "No events scheduled")

Error states: Visual error banners with a "Retry" button for API failures

---

# Backend Standards

Services: Background worker threads orchestrating OAuth syncing

Repositories: Structs encapsulating SQLx calls

Controllers: Axum route handlers mapping requests to domain services

DTOs: Rust structs representing API inputs/outputs, distinct from database row models

Validation: Structural checks inside controller entry points

Dependency injection: Axum state passing for repositories and HTTP client resources

Configuration: Environment variables loaded via a centralized config module

Background jobs: Tokio interval timers running synchronization tasks in background loops

---

# Testing Standards

Minimum coverage: 70%+ for core domain sync modules and authentication validation

Unit tests: Mock-less unit tests for parser utilities, calendar calculations, and date parsing

Integration tests: SQLx tests running against temporary memory DBs

E2E tests: Playwright/Tauri testing user UI flows (login flow mock, search typing, message archiving)

Mock policy: External API integrations (Google/Microsoft APIs) must use mock servers or mock traits in tests

Fixtures: Standard JSON/SQL fixture structures for test data setup

Factories: Helper builders for generating test models

Test naming: Descriptive names (e.g., `test_sync_ignores_duplicate_external_ids`)

Test organization: Inline unit tests (`mod tests` at bottom of files) and separate integration test files in `/tests/` directory

---

# Documentation Standards

README updates: Keep setup instructions, project layout, and stack information current

Architecture updates: Update ARCHITECTURE.md for any framework, DB schema, or pattern changes

API documentation: Document endpoints in code or maintain a clean Markdown OpenAPI specification

Code comments: Restrict comments to explanation of complex domain algorithms

Decision log updates: Document any design trade-offs, scope updates, or technical decisions in DECISION_LOG.md

Release notes: Document version increments and changelogs

---

# Security Standards

Input validation: Strict validation of all inputs; sanitize HTML in email previews using a secure sanitizer (e.g., `ammonia` in Rust or equivalent)

Output encoding: HTML escape all dynamic texts in frontend views

Authentication: OAuth2 code flow using state params to prevent CSRF

Authorization: Tailscale network bindings (only bind web daemon to Tailscale IP interface)

Secrets: Secure storage of client secrets, database encryption keys using OS keystores or environment variables

Rate limiting: Limit login and sync initiation request rates

Sensitive logging: Redact or filter tokens and body payload fields from logs

Dependency updates: Cargo audit and npm audit checks run periodically

---

# Performance Standards

Performance budgets: Axum server memory usage under 200MB (including WASM plugins runtimes); Tauri client under 40MB RAM

Caching: Cache active inbox messages (last 30 days) and active month events locally in memory/SQLite

Pagination: Always paginate messages in chunks of 50 or less

Batching: Sync loop queries should batch IDs in requests to external mail APIs

Lazy loading: Defer rendering of email HTML content and attachments until explicitly selected

Memory limits: Cap in-memory list caches and garbage collect old messages from RAM stores

Network requests: Minify client assets, compress HTTP responses, use HTTP/2 where applicable

Database queries: Always use prepared statements (sqlx does this natively), analyze query plans for slow lookups

---

# VCS Standards (Jujutsu / jj)

Version Control Tool: Standardize on Jujutsu (`jj`).

Commit Message conventions: Use the conventional commit formatting syntax when describing changes in Jujutsu revisions. The message must strictly conform to:
```
type(scope): Title summarizing the change in imperative mood

Detailed explanation of the commit's context, reasoning, and technical decisions.
```
Example:
```
feat(db): Add sqlite dual-migration framework

Configure the startup migrations directory router to automatically resolve
connection engine dialects and verify queries at compile time.
```

Working Copy usage: Avoid creating manual branch refs. Utilize Jujutsu's implicit automatic commits and revision stacks (`jj new`, `jj squash`, `jj rebase`) to keep local changes clean and traceable.

Push/Sync process: Export modifications to Git bookmarks and push them to the upstream Git origin using `jj git push` or native bookmarked references.

---

# AI-Specific Rules

Never introduce technologies not listed in ARCHITECTURE.md.

Never generate placeholder implementations unless requested.

Never mark a task complete without satisfying the Definition of Done.

Never ignore failed commands.

Never ignore failed tests.

Never skip numbered requirements.

Always update

- IMPLEMENTATION_PLAN.md
- PROGRESS.md
- DECISION_LOG.md

when they are affected.

---

# Project-Specific Preferences

Prefer compile-time verified queries (sqlx).

Avoid heavy external client state management libraries (Svelte stores are fully sufficient).

Keep frontend components modular and styled exclusively with utility classes.

Use Feature folders to group related client routes, stores, and components.

---

# Rule Changes

Every modification should record:
- Date
- Reason
- Approved By
- Affected Rules

---

# Validation Checklist

Before implementation verify:

✓ Naming conventions followed
✓ File organization followed
✓ Error handling followed
✓ Testing standards followed
✓ Security standards followed
✓ Performance standards followed
✓ Documentation updated
✓ AI-specific rules satisfied

If any check fails, implementation is incomplete.
