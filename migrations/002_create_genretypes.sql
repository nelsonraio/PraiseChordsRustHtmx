-- --------------------------------------------------------
-- Tabela de tipos de música (Coros, Hinos Harpa, EBD, Outros Arranjos)
-- Convertida a partir do dump original (genretypes.sql) para PostgreSQL.
-- A coluna "ID" corresponde ao campo "GenreType" da tabela "songs".
-- --------------------------------------------------------
CREATE TABLE IF NOT EXISTS genretypes (
    "ID" INTEGER PRIMARY KEY,
    "Desc" VARCHAR(50),
    "CodeSufix" VARCHAR(10)
);

-- Semear dados caso a tabela esteja vazia / tenha acabado de ser criada
INSERT INTO genretypes ("ID", "Desc", "CodeSufix") VALUES
    (1, 'Coros', 'W'),
    (2, 'EBD', 'ED'),
    (3, 'Hinos Harpa', 'HM'),
    (4, 'Outros Arranjos', 'OH')
ON CONFLICT ("ID") DO NOTHING;
