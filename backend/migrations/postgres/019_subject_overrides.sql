-- Personal thread subject overrides. Local-only, never synced upstream.

CREATE TABLE IF NOT EXISTS thread_subject_overrides (
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    thread_id TEXT NOT NULL,
    subject TEXT NOT NULL,
    updated_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM NOW()))::BIGINT,
    PRIMARY KEY (account_id, thread_id)
);
