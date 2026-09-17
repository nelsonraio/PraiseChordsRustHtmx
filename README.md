# PraiseChords (Rust)

**PraiseChords** é uma aplicação web para gestão de um repertório de músicas de louvor com cifras, pensada para equipas de música de igreja. Permite consultar, pesquisar, criar e editar cifras no formato **ChordPro**, organizá-las em **setlists** (listas de músicas para cultos/eventos), reproduzir as setlists em modo "player" música a música e partilhá-las através de links públicos.

## Funcionalidades

- **Autenticação de utilizadores** — login com sessão via cookie JWT, logout, recuperação de palavra-passe por email (token com validade limitada) e gestão de utilizadores.
- **Biblioteca de músicas** — listagem, pesquisa, favoritos, contador de visualizações, criação e edição de músicas em formato ChordPro (com conversão automática de cifras em texto simples — estilo CifraClub — para ChordPro).
- **Cartões da grelha de resultados** — o tom e o tempo aparecem numa linha própria de metadados, por baixo dos artistas, em texto destacado (vermelho), em vez de ocuparem espaço na fila de botões de ação; quando não existem dados, a linha não é mostrada.
- **Visualizador de cifras** — renderização das cifras com destaque de acordes, alteração de tamanho de letra (incluindo pinch-to-zoom em ecrãs táteis), disposição em colunas, transposição de acordes, opção de bemóis/sustenidos e preferências guardadas por música.
- **Links do YouTube** — na criação e edição, aceita um ID ou link por linha: o primeiro é a versão original e os restantes são versões alternativas. Com um único link, o botão abre diretamente o YouTube numa nova aba; com vários, permite escolher a versão antes de abrir. Os links são guardados no campo `Youtube` existente, separados por quebras de linha, mantendo compatibilidade com músicas que têm apenas um link.
- **Leitor da música no editor** — nos formulários de criação e edição, um botão **▶ Tocar / ■ Parar** reproduz a música dentro da própria página (leitor incorporado), permitindo ouvir enquanto se corrige a cifra e se deteta o tempo. Havendo várias versões, uma lista permite escolher qual ouvir.
- **Deteção de tom** — nos formulários de criação e edição, um botão estima o tom a partir dos acordes do ChordPro e preenche o campo **Tom Original** (sempre editável antes de guardar).
- **Transposição no editor** — nos formulários de criação e edição, os botões **▲ +1** e **▼ −1** (ao lado de "Alinhar Acordes") transpõem todos os acordes do ChordPro meio tom acima/abaixo, preservando a grafia de bemóis/sustenidos, o baixo com barra (ex.: `D/F#`) e deixando a letra e as diretivas `{…}` intactas.
- **Tap BPM** — nos formulários de criação e edição, um botão mede o tempo da música pelos toques ao ritmo e preenche o campo **Tempo**. Após 4 segundos de pausa, a medição recomeça.
- **Atualização automática da grelha** — ao guardar alterações (criação ou edição), a grelha de resultados atualiza-se sem recarregar a página: no painel de pesquisa reenvia a pesquisa ativa (preservando filtros e ordenação); nas páginas de biblioteca renderizadas no servidor (Mais Acessadas, Favoritas, Recentes, Pendentes) recarrega a página.
- **Setlists** — criação, edição e reordenação de listas de músicas, modo de apresentação (player) para navegar entre músicas durante o culto e partilha de setlists via link público com token.
- **Estatísticas** — músicas mais vistas, favoritas e visualizações recentes.

## Tecnologias

### Backend
- **Rust** — linguagem principal do projeto.
- **Axum** (0.7) — framework web para as rotas HTML, endpoints HTMX e API JSON.
- **Tokio** — runtime assíncrono.
- **SQLx** (PostgreSQL) — acesso à base de dados PostgreSQL, com migrações em `migrations/`.
- **Askama** — motor de templates para as páginas HTML (`templates/`).
- **tower-http** — servir ficheiros estáticos (`static/`).
- **jsonwebtoken / bcrypt / sha2 / rand** — autenticação (JWT em cookies), hashing de palavras-passe e geração de tokens.
- **lettre** — envio de email (SMTP) para recuperação de palavra-passe.
- **serde / serde_json** — serialização JSON.
- **axum-extra / cookie** — gestão de cookies.
- **chrono**, **dotenvy**, **tracing** / **tracing-subscriber** — datas, configuração via `.env` e logging.

### Frontend
- **HTMX** — atualizações parciais da página sem recarregar (pesquisa, favoritos, setlists, etc.).
- **Alpine.js** — interatividade do visualizador de cifras e modais.
- **Tailwind CSS** — estilização (fonte em `static/tailwind-input.css`).
- **ChordSheetJS** — conversão de cifras em texto simples para ChordPro no cliente.
- **chord-parser.js** (próprio) — parsing, renderização e transposição de ChordPro no navegador.
- **Font Awesome** — iconografia.

## Como executar

1. Copiar `tailwind.css` gerado do projeto Next.js para `static/tailwind.css` (já incluído neste repositório).
2. Copiar `htmx.min.js` e `alpine.min.js` para `static/` ou usar CDN (já incluídos).
3. Criar `.env` baseado em `.env.example` com `DATABASE_URL` apontando para a base de dados PostgreSQL, além de `JWT_SECRET` e das definições SMTP.
4. Build e run:

```bash
# instalar Rust toolchain
cargo build
cargo run
```

O servidor inicia em `http://127.0.0.1:8080` e expõe as rotas de aplicação (`/login`, `/app`, `/statistics`, `/setlists/...`), endpoints HTMX (`/htmx/...`), a API JSON (`/api/songs`, `/api/setlists`, etc.) e os ficheiros estáticos em `/static/*`.
