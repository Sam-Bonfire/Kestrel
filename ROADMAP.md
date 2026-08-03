# Roadmap

This document tracks the current feature status and planned work for Kestrel.

## Legend

| Status | Meaning |
|--------|---------|
| ✅ Done | Implemented and verified |
| 🚧 In Progress | Actively being worked on |
| 📋 Planned | Designed but not yet started |
| 💡 Exploring | Under consideration |

---

## Core Infrastructure

| Feature | Status | Notes |
|---------|--------|-------|
| Monorepo workspace (pnpm + Cargo) | ✅ Done | `frontend-shared`, `frontend-mail`, `frontend-calendar`, `backend` |
| Axum REST API server | ✅ Done | Auth, messages, calendars, search, sync, health |
| SQLite database adapter | ✅ Done | Full repository implementation with FTS5 search |
| PostgreSQL database adapter | ✅ Done | Full repository implementation with tsvector search |
| Multi-tenant user isolation | ✅ Done | Composite unique keys per user+account |
| WASM plugin architecture | ✅ Done | Wasmtime runtime with WIT interface |
| Docker deployment | ✅ Done | Multi-stage build with cargo-chef caching |
| CI/CD pipeline | ✅ Done | GitHub Actions: backend, frontend, desktop, mobile, Docker |
| Release pipeline | ✅ Done | Auto-tag, GitHub Release, GHCR push |

## Mail Client

| Feature | Status | Notes |
|---------|--------|-------|
| Inbox UI with threading | ✅ Done | Keyboard-driven (J/K navigation) |
| Email body rendering | ✅ Done | Sandboxed iframe |
| Compose modal | ✅ Done | Account/from selection |
| Labels and folders | ✅ Done | Inbox, Sent, Drafts, Archive, Trash |
| Search | ✅ Done | Full-text search via API |
| Command palette (Cmd+K) | ✅ Done | Quick actions and navigation |
| Keyboard shortcuts | ✅ Done | Vim-style navigation |
| Peek panel (split view) | ✅ Done | Notion-style sliding panel |
| Gmail sync (live) | ✅ Done | Real Gmail API via WASM plugin (mail + calendar fetch, send, mutate) |
| Outlook sync (live) | ✅ Done | Real Microsoft Graph via WASM plugin (mail + calendar fetch, send, mutate) |
| SMTP/API email sending | 📋 Planned | Currently mocked |
| Attachment handling | ✅ Done | Downloads via Tauri fs to OS download dir; browser fallback |
| Snooze | 📋 Planned | Currently mocked as archive |
| Rich text signatures | 📋 Planned | Basic textarea only |

## Calendar Client

| Feature | Status | Notes |
|---------|--------|-------|
| Week view | ✅ Done | Overlapping event layout algorithm |
| Day view | ✅ Done | |
| Month view | ✅ Done | |
| Event creation/editing | ✅ Done | Modal and peek panel |
| Sidebar with mini calendar | ✅ Done | Calendar account toggles |
| Keyboard shortcuts (1-7, D/W/M) | ✅ Done | View switching |
| Settings panel | ✅ Done | Start hour, weekends toggle |
| Recurring events (client) | ✅ Done | Via rrule.js for offline calculation |
| Google Calendar sync | ✅ Done | Real Google Calendar API via WASM plugin |
| Outlook Calendar sync | ✅ Done | Real Microsoft Graph Calendar API via WASM plugin |
| Drag-to-create events | ✅ Done | Week/day/N-day timeline: drag on empty slot to create |
| Drag-to-resize events | ✅ Done | Week/day/N-day timeline: drag selected event's bottom edge |

## Desktop & Mobile

| Feature | Status | Notes |
|---------|--------|-------|
| Windows MSI installer | ✅ Done | CI produces artifact |
| Linux .deb package | ✅ Done | CI produces artifact |
| Linux AppImage | ✅ Done | CI produces artifact |
| macOS universal .dmg | ✅ Done | CI produces artifact |
| Android APK (unsigned) | ✅ Done | CI produces artifact (aarch64 + x86_64) |
| iOS simulator build | ✅ Done | CI produces artifact |
| Android code signing | 📋 Planned | Keystore not yet configured |
| iOS code signing | 📋 Planned | Apple Developer cert not yet configured |
| Tauri auto-update | 📋 Planned | Signing key and update endpoint needed |
| Deep linking | ✅ Done | `kestrel://` / `kestrel-calendar://` registered + onOpenUrl listeners (OAuth callback + create-event actions) |

## Shared Platform

| Feature | Status | Notes |
|---------|--------|-------|
| API client library | ✅ Done | Typed HTTP wrapper for all endpoints |
| Auth store (Svelte) | ✅ Done | Reactive auth state with token management |
| Offline queue | ✅ Done | IndexedDB mutation replay on reconnect |
| Design tokens (Kestrel Slate) | ✅ Done | Dark theme with periwinkle accent |
| Settings modal | ✅ Done | Shortcut rebinding |
| Shared UI components | ✅ Done | Avatar, Button, Dropdown, Login, etc. |
| Native notifications | ✅ Done | Tauri plugin registered; Mail reply/mark_read/archive actions call API; Calendar snooze/dismiss + reminders |

## Backend Integrations

| Feature | Status | Notes |
|---------|--------|-------|
| Mock provider plugin | ✅ Done | Reference implementation for dev/test |
| Gmail WASM plugin | ✅ Done | OAuth flow, mail sync/send, calendar fetch + mutate implemented |
| Outlook WASM plugin | ✅ Done | OAuth flow, mail sync/send, calendar fetch + mutate implemented |
| SSE real-time sync stream | ✅ Done | `/api/sync/stream` endpoint |
| Background sync daemon | ✅ Done | Configurable interval |
| Push notifications (FCM/APNs) | 📋 Planned | |
| Contact address book | 💡 Exploring | |
| CalDAV server exposure | 💡 Exploring | Long-term roadmap (DEC-022) |
| Custom IMAP providers | 💡 Exploring | Via WASM plugin |

---

## Version History

| Version | Date | Highlights |
|---------|------|------------|
| v0.1.0 | _Unreleased_ | Initial release — full UI for Mail and Calendar, backend API, dual DB support, CI/CD pipeline, Docker deployment |
