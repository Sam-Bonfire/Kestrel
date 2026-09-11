CREATE TABLE IF NOT EXISTS event_polls (
    id TEXT PRIMARY KEY NOT NULL,
    event_id TEXT NOT NULL,
    question TEXT NOT NULL,
    options_json TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch())
);
CREATE INDEX IF NOT EXISTS idx_event_polls_event_id ON event_polls(event_id);
CREATE TABLE IF NOT EXISTS poll_votes (
    poll_id TEXT NOT NULL REFERENCES event_polls(id) ON DELETE CASCADE,
    voter_email TEXT NOT NULL,
    option_index INTEGER NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    PRIMARY KEY (poll_id, voter_email, option_index)
);
