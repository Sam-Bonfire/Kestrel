-- Up
ALTER TABLE messages ADD COLUMN has_conflict BOOLEAN NOT NULL DEFAULT 0;
ALTER TABLE calendar_events ADD COLUMN has_conflict BOOLEAN NOT NULL DEFAULT 0;

DROP TABLE IF EXISTS historical_revisions;

CREATE TABLE historical_revisions (
    id TEXT PRIMARY KEY NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id TEXT NOT NULL,
    serialized_payload TEXT NOT NULL,
    revision_number INTEGER NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE INDEX idx_historical_revisions_resource ON historical_revisions(resource_type, resource_id);

-- Down
ALTER TABLE messages DROP COLUMN has_conflict;
ALTER TABLE calendar_events DROP COLUMN has_conflict;
DROP TABLE IF EXISTS historical_revisions;
