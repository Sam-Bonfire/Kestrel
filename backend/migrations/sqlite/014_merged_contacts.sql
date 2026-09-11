CREATE TABLE IF NOT EXISTS merged_contacts (
    account_id TEXT NOT NULL,
    email TEXT NOT NULL,
    merged_into TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    PRIMARY KEY (account_id, email)
);
