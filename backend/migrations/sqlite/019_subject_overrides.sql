-- Personal thread subject overrides. Local-only, never synced upstream.

CREATE TABLE IF NOT EXISTS thread_subject_overrides (
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    thread_id TEXT NOT NULL,
    subject TEXT NOT NULL,
    updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
    PRIMARY KEY (account_id, thread_id)
);
