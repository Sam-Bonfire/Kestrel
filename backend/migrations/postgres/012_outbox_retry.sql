ALTER TABLE offline_queue ADD COLUMN next_attempt_at BIGINT NOT NULL DEFAULT 0;
