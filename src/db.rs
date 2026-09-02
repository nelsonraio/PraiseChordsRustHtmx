use dotenvy::{dotenv, from_read};
use sqlx::PgPool;
use std::env;
use std::io::Cursor;
use std::path::Path;

pub async fn connect() -> PgPool {
    // Try loading a .env in the current directory first; if not found,
    // try common alternatives including parent `.env` or `.env.local`.
    if dotenv().is_err() {
        let candidates = [".env.local", ".env", "../.env.local", "../.env"];
        for cand in candidates.iter() {
            if Path::new(cand).exists() {
                if let Ok(contents) = std::fs::read_to_string(cand) {
                    let _ = from_read(Cursor::new(normalize_comments(&contents).into_bytes()));
                }
                break;
            }
        }
    }

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgPool::connect(&database_url)
        .await
        .expect("failed to connect to DB")
}

/// O dotenvy só aceita comentários com `#`; uma linha `// chave=valor`
/// provoca erro de parsing e interrompe o carregamento do .env nesse ponto
/// (as variáveis seguintes ficam por definir). Converte `//` em `#`.
fn normalize_comments(contents: &str) -> String {
    contents
        .lines()
        .map(|line| {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                let indent_len = line.len() - trimmed.len();
                format!("{}#{}", &line[..indent_len], &trimmed[2..])
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
