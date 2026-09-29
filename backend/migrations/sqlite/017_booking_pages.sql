CREATE TABLE IF NOT EXISTS booking_pages (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    calendar_id TEXT NOT NULL REFERENCES calendars(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    duration_mins INTEGER NOT NULL CHECK (duration_mins BETWEEN 5 AND 720),
    buffer_mins INTEGER NOT NULL DEFAULT 0 CHECK (buffer_mins BETWEEN 0 AND 120),
    window_days INTEGER NOT NULL DEFAULT 14 CHECK (window_days BETWEEN 1 AND 60),
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch())
);
CREATE INDEX IF NOT EXISTS idx_booking_pages_user_id ON booking_pages(user_id);
CREATE INDEX IF NOT EXISTS idx_booking_pages_slug ON booking_pages(slug);
