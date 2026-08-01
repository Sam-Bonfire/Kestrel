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
| Gmail sync (live) | 📋 Planned | Currently using mock provider plugin |
| Outlook sync (live) | 📋 Planned | Currently using mock provider plugin |
| SMTP/API email sending | 📋 Planned | Currently mocked |
| Attachment handling | 📋 Planned | CDN URLs mocked |
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
| Google Calendar sync | 📋 Planned | Currently using mock provider |
| Outlook Calendar sync | 📋 Planned | Currently using mock provider |
| Drag-to-create events | 📋 Planned | |
| Drag-to-resize events | 📋 Planned | |

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
| Deep linking | 🚧 In Progress | Registered schemes, native handler incomplete |

## Shared Platform

| Feature | Status | Notes |
|---------|--------|-------|
| API client library | ✅ Done | Typed HTTP wrapper for all endpoints |
| Auth store (Svelte) | ✅ Done | Reactive auth state with token management |
| Offline queue | ✅ Done | IndexedDB mutation replay on reconnect |
| Design tokens (Kestrel Slate) | ✅ Done | Dark theme with periwinkle accent |
| Settings modal | ✅ Done | Shortcut rebinding |
| Shared UI components | ✅ Done | Avatar, Button, Dropdown, Login, etc. |
| Native notifications | 🚧 In Progress | Tauri plugin registered, action handling partial |

## Backend Integrations

| Feature | Status | Notes |
|---------|--------|-------|
| Mock provider plugin | ✅ Done | Reference implementation for dev/test |
| Gmail WASM plugin | 📋 Planned | OAuth flow designed, plugin scaffolded |
| Outlook WASM plugin | 📋 Planned | OAuth flow designed, plugin scaffolded |
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
