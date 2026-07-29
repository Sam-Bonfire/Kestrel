CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY NOT NULL,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE TABLE IF NOT EXISTS accounts (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,
    provider_account_id TEXT NOT NULL,
    display_name TEXT NOT NULL DEFAULT '',
    access_token TEXT,
    refresh_token TEXT,
    token_expires_at INTEGER,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
    UNIQUE(user_id, provider, provider_account_id)
);

CREATE INDEX IF NOT EXISTS idx_accounts_user_id ON accounts(user_id);

CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY NOT NULL,
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    external_id TEXT NOT NULL,
    thread_id TEXT NOT NULL DEFAULT '',
    subject TEXT,
    sender_name TEXT,
    sender_email TEXT NOT NULL,
    recipients TEXT NOT NULL DEFAULT '[]',
    date_sent INTEGER NOT NULL DEFAULT 0,
    date_received INTEGER NOT NULL DEFAULT 0,
    snippet TEXT,
    body_text TEXT,
    body_html TEXT,
    labels TEXT,
    is_read INTEGER NOT NULL DEFAULT 0,
    is_archived INTEGER NOT NULL DEFAULT 0,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    has_attachments INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
    UNIQUE(account_id, external_id)
);

CREATE INDEX IF NOT EXISTS idx_messages_account_id ON messages(account_id);
CREATE INDEX IF NOT EXISTS idx_messages_is_read ON messages(is_read);
CREATE INDEX IF NOT EXISTS idx_messages_is_archived ON messages(is_archived);
CREATE INDEX IF NOT EXISTS idx_messages_is_deleted ON messages(is_deleted);
CREATE INDEX IF NOT EXISTS idx_messages_date_received ON messages(date_received);

CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts USING fts5(
    subject,
    snippet,
    sender_name,
    sender_email,
    content=messages,
    content_rowid=rowid
);

CREATE TRIGGER IF NOT EXISTS messages_ai AFTER INSERT ON messages BEGIN
    INSERT INTO messages_fts(rowid, subject, snippet, sender_name, sender_email)
    VALUES (new.rowid, new.subject, new.snippet, new.sender_name, new.sender_email);
END;

CREATE TRIGGER IF NOT EXISTS messages_ad AFTER DELETE ON messages BEGIN
    INSERT INTO messages_fts(messages_fts, rowid, subject, snippet, sender_name, sender_email)
    VALUES ('delete', old.rowid, old.subject, old.snippet, old.sender_name, old.sender_email);
END;

CREATE TRIGGER IF NOT EXISTS messages_au AFTER UPDATE ON messages BEGIN
    INSERT INTO messages_fts(messages_fts, rowid, subject, snippet, sender_name, sender_email)
    VALUES ('delete', old.rowid, old.subject, old.snippet, old.sender_name, old.sender_email);
    INSERT INTO messages_fts(rowid, subject, snippet, sender_name, sender_email)
    VALUES (new.rowid, new.subject, new.snippet, new.sender_name, new.sender_email);
END;

CREATE TABLE IF NOT EXISTS calendars (
    id TEXT PRIMARY KEY NOT NULL,
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    external_id TEXT NOT NULL,
    name TEXT NOT NULL,
    color TEXT,
    is_primary INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
    UNIQUE(account_id, external_id)
);

CREATE INDEX IF NOT EXISTS idx_calendars_account_id ON calendars(account_id);

CREATE TABLE IF NOT EXISTS calendar_events (
    id TEXT PRIMARY KEY NOT NULL,
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    calendar_id TEXT NOT NULL REFERENCES calendars(id) ON DELETE CASCADE,
    external_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    location TEXT,
    start_time INTEGER NOT NULL DEFAULT 0,
    end_time INTEGER NOT NULL DEFAULT 0,
    is_all_day INTEGER NOT NULL DEFAULT 0,
    recurrence_rules TEXT,
    organizer_email TEXT,
    organizer_name TEXT,
    attendees TEXT,
    status TEXT,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
    UNIQUE(account_id, external_id)
);

CREATE INDEX IF NOT EXISTS idx_calendar_events_account_id ON calendar_events(account_id);
CREATE INDEX IF NOT EXISTS idx_calendar_events_calendar_id ON calendar_events(calendar_id);
CREATE INDEX IF NOT EXISTS idx_calendar_events_start_time ON calendar_events(start_time);
CREATE INDEX IF NOT EXISTS idx_calendar_events_end_time ON calendar_events(end_time);

CREATE VIRTUAL TABLE IF NOT EXISTS calendar_events_fts USING fts5(
    title,
    description,
    location,
    content=calendar_events,
    content_rowid=rowid
);

CREATE TRIGGER IF NOT EXISTS calendar_events_ai AFTER INSERT ON calendar_events BEGIN
    INSERT INTO calendar_events_fts(rowid, title, description, location)
    VALUES (new.rowid, new.title, new.description, new.location);
END;

CREATE TRIGGER IF NOT EXISTS calendar_events_ad AFTER DELETE ON calendar_events BEGIN
    INSERT INTO calendar_events_fts(calendar_events_fts, rowid, title, description, location)
    VALUES ('delete', old.rowid, old.title, old.description, old.location);
END;

CREATE TRIGGER IF NOT EXISTS calendar_events_au AFTER UPDATE ON calendar_events BEGIN
    INSERT INTO calendar_events_fts(calendar_events_fts, rowid, title, description, location)
    VALUES ('delete', old.rowid, old.title, old.description, old.location);
    INSERT INTO calendar_events_fts(rowid, title, description, location)
    VALUES (new.rowid, new.title, new.description, new.location);
END;

CREATE TABLE IF NOT EXISTS historical_revisions (
    id TEXT PRIMARY KEY NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    external_id TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    superseded_at INTEGER NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE INDEX IF NOT EXISTS idx_historical_revisions_entity ON historical_revisions(entity_type, entity_id);

CREATE TABLE IF NOT EXISTS offline_queue (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    action TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id TEXT NOT NULL,
    payload TEXT,
    queued_at INTEGER NOT NULL DEFAULT (unixepoch()),
    retry_count INTEGER NOT NULL DEFAULT 0
);
