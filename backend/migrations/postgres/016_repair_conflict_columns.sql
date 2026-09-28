-- Repair for 010: sqlx executes migration files whole, so the `-- Down`
-- section appended to 010 ran as part of the upgrade on every database,
-- dropping has_conflict from both tables and dropping historical_revisions
-- right after creating them. 010 itself is left byte-identical (its checksum
-- is already recorded in shipped databases), so this migration re-applies
-- the intended end state.
ALTER TABLE messages ADD COLUMN IF NOT EXISTS has_conflict BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE calendar_events ADD COLUMN IF NOT EXISTS has_conflict BOOLEAN NOT NULL DEFAULT FALSE;

CREATE TABLE IF NOT EXISTS historical_revisions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    resource_type VARCHAR(100) NOT NULL,
    resource_id UUID NOT NULL,
    serialized_payload JSONB NOT NULL,
    revision_number INTEGER NOT NULL,
    created_at BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())
);

CREATE INDEX IF NOT EXISTS idx_historical_revisions_resource ON historical_revisions(resource_type, resource_id);
