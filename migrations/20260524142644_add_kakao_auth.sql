DROP INDEX idx_users_email;
ALTER TABLE users DROP COLUMN email;
ALTER TABLE users ADD COLUMN fcm_token           TEXT;
ALTER TABLE users ADD COLUMN is_deleted          BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE users ADD COLUMN current_refresh_jti UUID;

CREATE TABLE user_providers (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider    TEXT        NOT NULL,
    provider_id TEXT        NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (provider, provider_id)
);

CREATE INDEX idx_user_providers_user_id ON user_providers (user_id);
