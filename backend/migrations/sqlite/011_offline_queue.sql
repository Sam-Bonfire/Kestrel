CREATE TABLE IF NOT EXISTS offline_queue (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    action TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id TEXT NOT NULL,
    payload TEXT,
    queued_at INTEGER NOT NULL DEFAULT (unixepoch()),
    retry_count INTEGER NOT NULL DEFAULT 0
);
