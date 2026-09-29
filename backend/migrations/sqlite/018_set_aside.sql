-- Reference-later pile flag (mirrors snooze storage, no provider sync).

ALTER TABLE messages ADD COLUMN is_set_aside INTEGER NOT NULL DEFAULT 0;
