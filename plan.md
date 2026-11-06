1. **Repository Method**: Edit `backend/src/core/repository.rs` to add `get_latest_revision_for_resource` to the `HistoricalRevisionRepository` trait, returning `Result<Option<HistoricalRevision>, sqlx::Error>`. Then implement this method in `backend/src/db/sqlite/revision_repository.rs` and `backend/src/db/postgres/revision_repository.rs`.
2. **Conflict Metadata Struct**: Add a `ConflictMetadata` struct in `backend/src/core/models.rs`, containing `revision_id`, `revision_number`, `detected_at` (i.e. created_at), and `reason`.
3. **Detail Responses**: Modify `MessageDetail` in `backend/src/api/messages.rs` and `EventDetail` in `backend/src/api/calendars.rs` to include `pub conflict: Option<ConflictMetadata>`. Edit `get_message` and `get_event` logic to fetch the latest revision if `has_conflict` is true and construct the `ConflictMetadata` struct (with reason `"remote-changed-while-local-modified"`).
4. **Keep-local Resolution Endpoints**:
   - Add `pub async fn dismiss_message_conflict` endpoint in `backend/src/api/messages.rs` that sets `has_conflict` to false.
   - Add `pub async fn dismiss_event_conflict` endpoint in `backend/src/api/calendars.rs` that sets `has_conflict` to false.
   - Edit `backend/src/api/router.rs` to add `POST /api/v1/messages/:id/conflict/dismiss` and `POST /api/v1/events/:id/conflict/dismiss` pointing to these new handlers.
5. **Verify Backend Compilation**: Run `cargo check --manifest-path backend/Cargo.toml` to ensure the backend builds after changes.
6. **Frontend API Client**: Update `frontend-shared/src/api/client.ts` to export functions `dismissMessageConflict` and `dismissEventConflict`.
7. **Specta Generation**: Regenerate Specta types using `cargo run --bin export_types --manifest-path backend/Cargo.toml`.
8. **Frontend Banner Component**: Modify `frontend-mail/src/lib/components/ThreadPeekPanel.svelte` and `frontend-calendar/src/lib/components/EventPeekPanel.svelte` to show a banner if `conflict` is present on the selected item, with buttons for "Keep mine" (which calls dismiss API) and "Use server version" (which calls restore API).
9. **Verify Frontend Compilation**: Run `pnpm --filter @kestrel/shared build`, `pnpm --filter frontend-mail build`, and `pnpm --filter frontend-calendar build`.
10. **Backend Tests**:
    - Edit `backend/src/api/sync.rs` to add unit tests for `should_apply_remote_message`, `is_locally_modified`, and `message_content_differs`.
    - Edit `backend/tests/api_handler_tests.rs` to add integration tests verifying the conflict metadata in `GET /api/v1/messages/:id` responses and the behavior of the `POST /api/v1/messages/:id/conflict/dismiss` endpoint.
11. **Frontend Tests**: Create the file `frontend-mail/src/lib/components/ThreadPeekPanel.test.ts` and write Vitest tests to mock the API client, mount the component, and verify the 'Keep mine' and 'Use server version' banner actions.
12. **Run Tests**: Run `cargo test --manifest-path backend/Cargo.toml` and `pnpm --filter @kestrel/shared test` to ensure everything works.
13. **Pre commit**: Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
14. **Submit**: Use submit tool to push changes to PR branch.
