-- Add email verification flag to users.
-- Existing users are grandfathered as verified so they are not locked out.
ALTER TABLE users ADD COLUMN email_verified BOOLEAN NOT NULL DEFAULT FALSE;
UPDATE users SET email_verified = TRUE;

-- One-time verification tokens (single-use, 24 h TTL, SHA-256-hashed).
CREATE TABLE email_verification_tokens (
    id         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id    UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash TEXT        NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at    TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX email_verification_tokens_user_id_idx   ON email_verification_tokens(user_id);
CREATE INDEX email_verification_tokens_token_hash_idx ON email_verification_tokens(token_hash);
