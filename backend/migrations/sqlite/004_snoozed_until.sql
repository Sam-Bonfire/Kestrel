-- Add snoozed_until column to messages

ALTER TABLE messages ADD COLUMN snoozed_until INTEGER;
