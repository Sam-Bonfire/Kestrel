-- Reference-later pile flag (mirrors snooze storage, no provider sync).

ALTER TABLE messages ADD COLUMN is_set_aside BOOLEAN NOT NULL DEFAULT FALSE;
