CREATE TABLE IF NOT EXISTS event_polls (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    event_id UUID NOT NULL,
    question TEXT NOT NULL,
    options_json TEXT NOT NULL,
    created_at BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())
);
CREATE INDEX IF NOT EXISTS idx_event_polls_event_id ON event_polls(event_id);
CREATE TABLE IF NOT EXISTS poll_votes (
    poll_id UUID NOT NULL REFERENCES event_polls(id) ON DELETE CASCADE,
    voter_email VARCHAR(255) NOT NULL,
    option_index INTEGER NOT NULL,
    created_at BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW()),
    PRIMARY KEY (poll_id, voter_email, option_index)
);
