CREATE TABLE IF NOT EXISTS contacts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    name VARCHAR(255),
    email VARCHAR(255) NOT NULL,
    avatar_url TEXT,
    last_contacted_at BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW()),
    created_at BIGINT NOT NULL DEFAULT EXTRACT(EPOCH FROM NOW()),
    UNIQUE(account_id, email)
);

CREATE INDEX idx_contacts_account_id ON contacts(account_id);
CREATE INDEX idx_contacts_email ON contacts(email);
CREATE INDEX idx_contacts_name ON contacts(name);
CREATE INDEX idx_contacts_last_contacted_at ON contacts(last_contacted_at);
