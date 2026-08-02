CREATE TABLE attachments (
    id TEXT PRIMARY KEY NOT NULL,
    message_id TEXT NOT NULL,
    filename TEXT NOT NULL,
    content_type TEXT NOT NULL,
    size INTEGER NOT NULL,
    external_id TEXT,
    created_at INTEGER NOT NULL,
    FOREIGN KEY(message_id) REFERENCES messages(id) ON DELETE CASCADE
);

CREATE INDEX idx_attachments_message_id ON attachments(message_id);
