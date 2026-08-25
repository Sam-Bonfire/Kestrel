-- Up
ALTER TABLE messages ADD COLUMN has_conflict BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE calendar_events ADD COLUMN has_conflict BOOLEAN NOT NULL DEFAULT FALSE;

DROP TABLE IF EXISTS historical_revisions;

CREATE TABLE historical_revisions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    resource_type VARCHAR(100) NOT NULL,
    resource_id UUID NOT NULL,
    serialized_payload JSONB NOT NULL,
    revision_number INTEGER NOT NULL,
    created_at BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW())
);

CREATE INDEX idx_historical_revisions_resource ON historical_revisions(resource_type, resource_id);

-- Down
ALTER TABLE messages DROP COLUMN has_conflict;
ALTER TABLE calendar_events DROP COLUMN has_conflict;
DROP TABLE IF EXISTS historical_revisions;
