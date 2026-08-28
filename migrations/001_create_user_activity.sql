-- Tabela para registar atividade dos utilizadores (para estatísticas)
CREATE TABLE IF NOT EXISTS user_activity (
    id SERIAL PRIMARY KEY,
    "userId" INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    "action" VARCHAR(50) NOT NULL DEFAULT 'login',
    "createdAt" TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Índice para consultas rápidas por data e utilizador
CREATE INDEX IF NOT EXISTS idx_user_activity_created_at ON user_activity ("createdAt");
CREATE INDEX IF NOT EXISTS idx_user_activity_user_id ON user_activity ("userId");

CREATE TABLE IF NOT EXISTS password_reset_tokens (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(64) NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_lookup
    ON password_reset_tokens (token_hash, expires_at)
    WHERE used_at IS NULL;

CREATE TABLE IF NOT EXISTS setlist_share_links (
    id SERIAL PRIMARY KEY,
    setlist_id INTEGER NOT NULL UNIQUE REFERENCES setlists("ID") ON DELETE CASCADE,
    token VARCHAR(64) NOT NULL UNIQUE,
    created_by INTEGER REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
