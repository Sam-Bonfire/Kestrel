# Spec: CalDAV export via a `calendar-export` WIT interface (K-1598)

Status: design (read first, write later). Nothing here is implemented yet.

## What exists today

- `wit/kestrel.wit` (world `kestrel:provider/kestrel-plugin`) exports `mail-provider`,
  `calendar-provider`, `provider-branding`, `webhook-handler`. `calendar-provider`
  covers per-event reads/mutations only:
  `fetch-calendars`, `fetch-events`, `mutate-event`, `delete-event`, `query-freebusy`.
  There is no bulk export surface, so a provider's calendar cannot be pulled
  wholesale through the plugin ABI.
- The backend already owns a working ICS renderer: `backend/src/api/calendars.rs`
  has `escape_ics` and `format_ics_datetime`, and K-1597 mounted read-only
  per-calendar feeds at `GET /feeds/calendars/:filename` on the **public** router
  section, with revocable SHA-256-hashed tokens and ETag/304 caching. Those feeds
  render events from Kestrel's own `calendar_events` table only.
- WASM plugins are built with `cargo component` against this same WIT world
  (see `docs/PLUGINS.md`; gmail/outlook ship as `.wasm`).

## Gap

A user subscribed (in an external reader) to Kestrel's regenerated/graphed ICS
today sees only what Kestrel has already synced into `calendar_events`. If
provider-side events never synced (error, first run, skipped time ranges), the
feed silently under-serves. CalDAV-style export must therefore be able to pull
events from the provider in bulk, not only from our cache.

## Proposed slice 1: provider bulk pull via new WIT interface

Extend `wit/kestrel.wit` with one new interface, exported in addition to the
existing ones (additive; existing plugins keep compiling because the world gains
an export only when plugins opt in):

```wit
interface calendar-export {
    /// Large pull of a provider calendar. `since` is a provider-defined
    /// continuation token; empty string means "from the beginning".
    /// Returns events plus the next cursor ("finished" = done).
    export-events: func(
        auth-token: string,
        calendar-external-id: string,
        cursor: string,
        limit: u32,
    ) -> result<export-events-page, string>;

    /// Opaque provider cursor we persist between pulls.
    record export-events-page {
        events: list<event-input>,   // same shape calendar-provider.fetch-events already returns
        next-cursor: string,          // empty string = done
        complete: bool,
    }
}
```

Host responsibilities:

- `backend/src/plugins/traits.rs` gains a `CalendarExport` trait mirroring it;
  `manager.rs` exposes `as_calendar_export()` returning `Option` (None when a
  running plugin predates the interface).
- The sync worker (wherever `fetch_events` is already called) adds a bulk-drain
  loop: while `!page.complete`, call `export-events(cursor, 200)` and upsert
  through the existing event upsert path, storing `next-cursor` per
  `(account_id, calendar_id)` in a new `calendar_export_cursors` table
  (migrations `022_calendar_export_cursors.sql`, both dialects, mirror migration
  021's shape). A failed pull leaves the stored cursor untouched; retry resumes
  where it stopped.
- No rendering changes: K-1597 feeds keep reading `calendar_events` and gain
  completeness as a side effect of the drain.

Constants: pull limit 200 events/page host-side; a pull run stops after
20 consecutive pages without progress to bound runaway providers.

## Proposed slice 2 (later): true CalDAV server surface

Out of scope until slice 1 is shipped:

- CalDAV `REPORT`/`PROPFIND` verbs, `sync-token`, per-object etags. The K-1597
  feed endpoint already behaves correctly for static readers (ETag/304 +
  LAST-MODIFIED); full CalDAV would add `caldav` content-type routing and
  sync-token storage. Evaluate demand after slice 1 lands.

## Explicit non-goals for slice 1

- No plugin-side ICS rendering (host renders; one renderer, already tested).
- No write path from external readers back into providers (read-only export;
  RSVP/mutations stay in K-1628's surface).
- No calendar creates on provider side, no duplicates: uniqueness stays
  `(account_id, external_id)` in `calendar_events`.

## Verification plan for the implementing slice

- WIT: `wasm-tools`/`cargo component` build of gmail/outlook still succeeds
  (`mise` plugin build task, see `docs/PLUGINS.md`).
- Backend: `cargo test` adds (a) cursor drain test with a fake provider
  returning 3 pages incl. a mid-run error, (b) idempotency test: second drain
  with unchanged provider data upserts nothing new, (c) migration parity test
  postgres/sqlite.
- Feeds: an existing `GET /feeds/calendars/:filename` test asserts an event
  created only provider-side (via fake export) appears in the feed.
