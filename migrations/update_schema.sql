-- update_schema.sql
-- Script idempotente com todas as alterações de schema aplicadas pela aplicação
-- (as mesmas instruções que o src/main.rs corre automaticamente no arranque).
-- Pode ser corrido várias vezes sem risco (usa IF NOT EXISTS / ON CONFLICT DO NOTHING).
-- Uso: psql "$DATABASE_URL" -f migrations/update_schema.sql

BEGIN;

-- Extensão para normalização de acentos (necessária para pesquisa sem acentos)
CREATE EXTENSION IF NOT EXISTS unaccent;

-- Tabela de auditoria de logins/atividade dos utilizadores
CREATE TABLE IF NOT EXISTS user_activity (
    id SERIAL PRIMARY KEY,
    "userId" INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    "action" VARCHAR(50) NOT NULL DEFAULT 'login',
    "createdAt" TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_user_activity_created_at ON user_activity ("createdAt");
CREATE INDEX IF NOT EXISTS idx_user_activity_user_id ON user_activity ("userId");

-- Flag de conta demo
ALTER TABLE users ADD COLUMN IF NOT EXISTS is_demo BOOLEAN NOT NULL DEFAULT FALSE;

-- Workflow de aprovação de músicas: pending / approved
ALTER TABLE songs ADD COLUMN IF NOT EXISTS "Status" VARCHAR(20) NOT NULL DEFAULT 'approved';

-- Modo "apenas letra" (sem acordes) na visualização da cifra, gravável por utilizador/música/dispositivo
ALTER TABLE "UserChordSettings" ADD COLUMN IF NOT EXISTS "hideChords" BOOLEAN NOT NULL DEFAULT FALSE;

-- Tokens de reposição de password
CREATE TABLE IF NOT EXISTS password_reset_tokens (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(64) NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_lookup
    ON password_reset_tokens (token_hash, expires_at) WHERE used_at IS NULL;

-- Links públicos de partilha de setlists
CREATE TABLE IF NOT EXISTS setlist_share_links (
    id SERIAL PRIMARY KEY,
    setlist_id INTEGER NOT NULL UNIQUE REFERENCES setlists("ID") ON DELETE CASCADE,
    token VARCHAR(64) NOT NULL UNIQUE,
    created_by INTEGER REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Tipos de música (géneros) usados para gerar o código automático das cifras
CREATE TABLE IF NOT EXISTS genretypes (
    "ID" INTEGER PRIMARY KEY,
    "Desc" VARCHAR(50),
    "CodeSufix" VARCHAR(10)
);

INSERT INTO genretypes ("ID", "Desc", "CodeSufix") VALUES
    (1, 'Coros', 'W'),
    (2, 'EBD', 'ED'),
    (3, 'Hinos Harpa', 'HM'),
    (4, 'Outros Arranjos', 'OH')
ON CONFLICT ("ID") DO NOTHING;

-- As favoritas passaram a ser por utilizador (tabela "UserFavoriteSongs");
-- limpa os valores legados da coluna songs."Favorite" que mostravam a
-- estrela acesa sem a música constar nos favoritos do utilizador.
UPDATE songs SET "Favorite" = FALSE WHERE "Favorite" IS NOT FALSE;

COMMIT;

-- NOTA: a aplicação corre estas mesmas instruções automaticamente sempre que
-- arranca (src/main.rs, função main()), portanto este script só é necessário
-- se precisares de atualizar uma base de dados sem correr o binário (ex.:
-- preparar antecipadamente uma base de dados de produção).
