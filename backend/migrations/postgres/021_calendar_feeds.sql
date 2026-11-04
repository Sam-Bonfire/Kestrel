CREATE TABLE IF NOT EXISTS calendar_feeds (
    calendar_id UUID PRIMARY KEY REFERENCES calendars(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL,
    created_at BIGINT NOT NULL
);
