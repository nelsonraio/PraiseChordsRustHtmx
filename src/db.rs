use dotenvy::{dotenv, from_filename};
use sqlx::PgPool;
use std::env;
use std::path::Path;

pub async fn connect() -> PgPool {
    // Try loading a .env in the current directory first; if not found,
    // try common alternatives including parent `.env` or `.env.local`.
    if dotenv().is_err() {
        let candidates = [".env.local", ".env", "../.env.local", "../.env"];
        for cand in candidates.iter() {
            if Path::new(cand).exists() {
                let _ = from_filename(cand);
                break;
            }
        }
    }

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgPool::connect(&database_url)
        .await
        .expect("failed to connect to DB")
}
