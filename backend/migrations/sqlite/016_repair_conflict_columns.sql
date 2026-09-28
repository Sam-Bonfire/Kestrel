-- Repair for 010: sqlx executes migration files whole, so the `-- Down`
-- section appended to 010 ran as part of the upgrade on every database,
-- dropping has_conflict from both tables and dropping historical_revisions
-- right after creating them. 010 itself is left byte-identical (its checksum
-- is already recorded in shipped databases), so this migration re-applies
-- the intended end state. Safe to run everywhere: every database that went
-- through 010 is missing these objects.
ALTER TABLE messages ADD COLUMN has_conflict BOOLEAN NOT NULL DEFAULT 0;
ALTER TABLE calendar_events ADD COLUMN has_conflict BOOLEAN NOT NULL DEFAULT 0;

CREATE TABLE IF NOT EXISTS historical_revisions (
    id TEXT PRIMARY KEY NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id TEXT NOT NULL,
    serialized_payload TEXT NOT NULL,
    revision_number INTEGER NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE INDEX IF NOT EXISTS idx_historical_revisions_resource ON historical_revisions(resource_type, resource_id);
