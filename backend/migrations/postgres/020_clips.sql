-- Clipped text spans with source jump-back, plus per-thread sticky notes.
-- Both local-only, never synced upstream.

CREATE TABLE IF NOT EXISTS clips (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    message_id TEXT NOT NULL,
    snippet TEXT NOT NULL,
    created_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM NOW()))::BIGINT
);
CREATE INDEX IF NOT EXISTS idx_clips_user_id ON clips(user_id);

CREATE TABLE IF NOT EXISTS thread_notes (
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    thread_id TEXT NOT NULL,
    note TEXT NOT NULL,
    updated_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM NOW()))::BIGINT,
    PRIMARY KEY (account_id, thread_id)
);
