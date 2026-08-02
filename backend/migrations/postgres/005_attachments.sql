CREATE TABLE attachments (
    id UUID PRIMARY KEY,
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    filename TEXT NOT NULL,
    content_type TEXT NOT NULL,
    size BIGINT NOT NULL,
    external_id TEXT,
    created_at BIGINT NOT NULL
);

CREATE INDEX idx_attachments_message_id ON attachments(message_id);
