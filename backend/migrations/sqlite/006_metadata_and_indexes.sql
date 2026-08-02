-- Up
ALTER TABLE messages ADD COLUMN metadata TEXT;
ALTER TABLE calendar_events ADD COLUMN metadata TEXT;

-- Drop redundant single-column indexes (since they are now covered by composite indexes or were unused)
DROP INDEX IF EXISTS idx_messages_is_read;
DROP INDEX IF EXISTS idx_messages_is_archived;
DROP INDEX IF EXISTS idx_messages_is_deleted;
DROP INDEX IF EXISTS idx_messages_date_received;

-- Create highly optimized composite indexes
CREATE INDEX IF NOT EXISTS idx_messages_inbox ON messages(account_id, is_deleted, is_archived, date_received DESC);
CREATE INDEX IF NOT EXISTS idx_calendar_events_range ON calendar_events(account_id, start_time, end_time);
