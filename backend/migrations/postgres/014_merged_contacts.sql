CREATE TABLE IF NOT EXISTS merged_contacts (
    account_id UUID NOT NULL,
    email VARCHAR(255) NOT NULL,
    merged_into VARCHAR(255) NOT NULL,
    created_at BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW()),
    PRIMARY KEY (account_id, email)
);
