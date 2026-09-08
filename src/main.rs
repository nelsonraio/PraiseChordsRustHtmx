use axum::{
    extract::{Form, Path, Query, Request},
    http::{header, HeaderMap, StatusCode},
    middleware::{self, Next},
    response::Response,
    response::{Html, IntoResponse, Redirect},
    routing::delete,
    routing::get,
    routing::get_service,
    routing::post,
    Extension, Json, Router,
};
use axum_extra::extract::cookie::CookieJar;
use chrono::NaiveDateTime;
use chrono::{Duration, Utc};
use cookie::{Cookie, SameSite};
use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use sqlx::Row;
use std::collections::{HashMap, HashSet};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
mod db;
use askama::Template;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use lettre::{
    message::Mailbox, transport::smtp::authentication::Credentials, AsyncSmtpTransport,
    AsyncTransport, Message, Tokio1Executor,
};
use rand::{rngs::OsRng, RngCore};
use serde::Deserialize;
use tracing_subscriber;

#[derive(Serialize, sqlx::FromRow, Debug, Clone)]
struct Song {
    #[sqlx(rename = "ID")]
    id: i32,
    #[sqlx(rename = "Code")]
    code: Option<String>,
    #[sqlx(rename = "Name")]
    name: Option<String>,
    #[sqlx(rename = "OrgName")]
    org_name: Option<String>,
    #[sqlx(rename = "Composer")]
    composer: Option<String>,
    #[sqlx(rename = "ChordPro")]
    chord_pro: String,
    #[sqlx(rename = "Lyrics")]
    lyrics: Option<String>,
    #[sqlx(rename = "Themes")]
    themes: Option<String>,
    #[sqlx(rename = "Youtube")]
    youtube: Option<String>,
    #[sqlx(rename = "GenreType")]
    genre_type: Option<i32>,
    #[sqlx(rename = "Artistas")]
    artistas: Option<String>,
    #[sqlx(rename = "OrgKey")]
    org_key: Option<String>,
    #[sqlx(rename = "OrgTempo")]
    org_tempo: Option<i32>,
    #[sqlx(rename = "Copyright")]
    copyright: Option<String>,
    #[sqlx(rename = "Favorite")]
    favorite: Option<bool>,
    #[sqlx(rename = "IdInsertUser")]
    id_insert_user: Option<i32>,
    #[sqlx(rename = "IdUpdateUser")]
    id_update_user: Option<i32>,
    #[sqlx(rename = "ViewCount")]
    view_count: Option<i32>,
    #[sqlx(rename = "createdAt")]
    created_at: Option<NaiveDateTime>,
    #[sqlx(rename = "updatedAt")]
    updated_at: Option<NaiveDateTime>,
    #[sqlx(rename = "Status")]
    status: Option<String>,
}

#[derive(sqlx::FromRow)]
struct ChordSettingsRow {
    #[sqlx(rename = "fontSize")]
    font_size: f32,
    #[sqlx(rename = "columnCount")]
    column_count: i32,
    accidentals: i32,
    transpose: i32,
    #[sqlx(rename = "hideChords")]
    hide_chords: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChordSettingsInput {
    font_size: f32,
    column_count: i32,
    accidentals: i32,
    transpose: i32,
    hide_chords: bool,
}

#[derive(Serialize, Debug, Clone)]
struct SongListItem {
    id: i32,
    code: String,
    name: String,
    org_name: String,
    composer: String,
    artistas: String,
    org_key: String,
    youtube: Option<String>,
    favorite: bool,
    view_count: i64,
    inserted_by: String,
    updated_by: String,
    is_pending: bool,
    can_edit: bool,
    can_delete: bool,
    can_approve: bool,
}

#[derive(Serialize, Debug, Clone)]
struct SongDetailView {
    id: i32,
    name: String,
    code: String,
    chord_pro: String,
    lyrics: String,
    youtube: String,
    favorite: bool,
    is_pending: bool,
    can_edit: bool,
    can_delete: bool,
    can_approve: bool,
    font_size: f32,
    column_count: i32,
    accidentals: i32,
    transpose: i32,
    hide_chords: bool,
}

#[derive(Debug, Clone)]
struct SongEditView {
    id: i32,
    code: String,
    title: String,
    name: String,
    org_name: String,
    composer: String,
    chord_pro: String,
    lyrics: String,
    themes: String,
    youtube: String,
    artistas: String,
    org_key: String,
    org_tempo: String,
    copyright: String,
}

#[derive(Debug, Clone)]
struct GenreOption {
    id: i32,
    name: String,
    prefix: String,
    selected: bool,
}

#[derive(Debug, Clone)]
struct GenreTypeRow {
    id: i32,
    desc: String,
    #[allow(dead_code)]
    prefix: String,
}

#[derive(sqlx::FromRow, Debug, Clone)]
struct DbUser {
    id: i32,
    email: String,
    nome: Option<String>,
    perfil: Option<i32>,
    igreja: Option<String>,
    password_hash: String,
    ativo: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AuthClaims {
    id: i32,
    email: String,
    nome: Option<String>,
    perfil: Option<i32>,
    is_demo: bool,
    exp: usize,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    first_name: String,
    initials: String,
    profile_label: String,
    church: String,
    song_count: i64,
    favorite_count: i64,
    setlist_count: i64,
    is_programmer: bool,
    is_moderator: bool,
    can_submit_song: bool,
    #[allow(dead_code)]
    is_demo: bool,
    genres: Vec<GenreTypeRow>,
}

#[derive(Template)]
#[template(path = "statistics.html")]
struct StatisticsTemplate {
    days: i64,
}

#[derive(Serialize, Debug, Clone)]
struct DailyUserStat {
    date: String,
    unique_users: i64,
}

#[derive(Debug, sqlx::FromRow)]
struct RecentUserStat {
    nome: Option<String>,
    email: String,
    perfil: Option<i32>,
    last_active: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Debug, Clone)]
struct StatisticsData {
    total_unique_users: i64,
    total_logins: i64,
    daily_stats: Vec<DailyUserStat>,
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {}

#[derive(Template)]
#[template(path = "forgot_password.html")]
struct ForgotPasswordTemplate {}

#[derive(Template)]
#[template(path = "register.html")]
struct RegisterTemplate {}

#[derive(Template)]
#[template(path = "reset_password.html")]
struct ResetPasswordTemplate {
    token: String,
}

struct SmtpConfig {
    host: String,
    port: u16,
    secure: SmtpSecureMode,
    user: Option<String>,
    password: Option<String>,
    from: String,
    app_base_url: String,
    reset_ttl_minutes: i64,
}

/// Modo de segurança SMTP:
/// - `Tls`: TLS implícito desde o início da ligação (porta 465)
/// - `StartTls`: plaintext seguido de STARTTLS (porta 587)
/// - `None`: sem encriptação (apenas para Mailpit local)
#[derive(Debug, Clone, Copy, PartialEq)]
enum SmtpSecureMode {
    None,
    StartTls,
    Tls,
}

fn parse_smtp_secure(value: Option<String>) -> SmtpSecureMode {
    match value.map(|value| value.to_lowercase()).as_deref() {
        Some("true" | "1" | "yes" | "tls" | "ssl") => SmtpSecureMode::Tls,
        Some("starttls") => SmtpSecureMode::StartTls,
        _ => SmtpSecureMode::None,
    }
}

#[derive(Template)]
#[template(path = "songs_list.html")]
struct SongsListTemplate {
    heading: String,
    songs: Vec<SongListItem>,
}

#[derive(Template)]
#[template(path = "pending.html")]
struct PendingTemplate {
    heading: String,
    songs: Vec<SongListItem>,
}

#[derive(Template)]
#[template(path = "library.html")]
struct LibraryTemplate {
    title: String,
    subtitle: String,
    songs: Vec<SongListItem>,
}

#[derive(Template)]
#[template(path = "song_detail.html")]
struct SongDetailTemplate {
    song: SongDetailView,
    navigation: Option<SetlistNavigation>,
    is_public: bool,
}

#[derive(Debug, Clone)]
struct SetlistNavigation {
    current_position: usize,
    total_songs: usize,
    previous_url: Option<String>,
    next_url: Option<String>,
}

#[derive(Template)]
#[template(path = "setlist_player.html")]
struct SetlistPlayerTemplate {
    song: SongDetailView,
    navigation: Option<SetlistNavigation>,
    is_public: bool,
}

#[derive(Template)]
#[template(path = "public_setlist.html")]
struct PublicSetlistTemplate {
    setlist: SetlistItem,
    songs: Vec<SongInSetlist>,
    token: String,
}

#[derive(Template)]
#[template(path = "song_edit.html")]
struct SongEditTemplate {
    song: SongEditView,
    genres: Vec<GenreOption>,
}

#[derive(Serialize, sqlx::FromRow, Debug, Clone)]
struct Setlist {
    #[sqlx(rename = "ID")]
    id: i32,
    #[sqlx(rename = "Data")]
    data: Option<chrono::NaiveDate>,
    #[sqlx(rename = "Desc")]
    desc: Option<String>,
    #[sqlx(rename = "IDGrupo")]
    id_grupo: Option<i32>,
}

#[derive(Serialize, Debug, Clone)]
struct SetlistItem {
    id: i32,
    data: String,
    desc: String,
    song_count: i64,
}

#[derive(Serialize, Debug, Clone)]
struct SongInSetlist {
    id: i32,
    name: String,
    org_name: String,
    composer: String,
    artistas: String,
    youtube: Option<String>,
}

#[derive(Template)]
#[template(path = "setlists_list.html")]
struct SetlistsListTemplate {
    setlists: Vec<SetlistItem>,
    total_songs: i64,
}

#[derive(Template)]
#[template(path = "setlist_song_picker.html")]
struct SetlistSongPickerTemplate {
    song_id: i32,
    setlists: Vec<SetlistItem>,
}

#[derive(Template)]
#[template(path = "setlist_song_create.html")]
struct SetlistSongCreateTemplate {
    song_id: i32,
}

#[derive(Template)]
#[template(path = "setlist_create.html")]
struct SetlistCreateTemplate {}

#[derive(Template)]
#[template(path = "setlist_detail.html")]
struct SetlistDetailTemplate {
    setlist: SetlistItem,
    songs: Vec<SongInSetlist>,
}

fn auth_secret() -> String {
    std::env::var("JWT_SECRET").unwrap_or_else(|_| "your_super_secret_jwt_key_here".to_string())
}

fn smtp_config() -> SmtpConfig {
    let env_value = |key: &str| {
        std::env::var(key)
            .ok()
            .filter(|value| !value.trim().is_empty())
    };
    SmtpConfig {
        host: env_value("SMTP_HOST").unwrap_or_else(|| "127.0.0.1".to_string()),
        port: env_value("SMTP_PORT")
            .and_then(|value| value.parse().ok())
            .unwrap_or(1025),
        secure: parse_smtp_secure(env_value("SMTP_SECURE")),
        user: env_value("SMTP_USER"),
        password: env_value("SMTP_PASSWORD"),
        from: env_value("SMTP_FROM")
            .unwrap_or_else(|| "PraiseChords <noreply@localhost>".to_string()),
        app_base_url: env_value("APP_BASE_URL")
            .unwrap_or_else(|| "http://127.0.0.1:8080".to_string()),
        reset_ttl_minutes: env_value("PASSWORD_RESET_TTL_MINUTES")
            .and_then(|value| value.parse().ok())
            .filter(|value: &i64| *value > 0)
            .unwrap_or(60),
    }
}

fn new_reset_token() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn hash_reset_token(token: &str) -> String {
    format!("{:x}", Sha256::digest(token.as_bytes()))
}

async fn send_password_reset_email(
    config: &SmtpConfig,
    recipient: &str,
    reset_link: &str,
) -> Result<(), String> {
    let body = format!(
        "Recebemos um pedido para alterar a sua password.\n\nAbra este link para escolher uma nova password:\n{reset_link}\n\nO link expira em {} minutos. Se não fez este pedido, ignore este email.",
        config.reset_ttl_minutes
    );
    send_transactional_email(config, recipient, "Recuperação de password - PraiseChords", body).await
}

async fn send_registration_email(
    config: &SmtpConfig,
    recipient: &str,
    confirm_link: &str,
) -> Result<(), String> {
    let body = format!(
        "Obrigado por se registar no PraiseChords!\n\nAbra este link para confirmar a sua conta e escolher a sua password:\n{confirm_link}\n\nO link expira em {} minutos. Se não fez este pedido, ignore este email.",
        config.reset_ttl_minutes
    );
    send_transactional_email(config, recipient, "Confirme o seu registo - PraiseChords", body).await
}

async fn send_transactional_email(
    config: &SmtpConfig,
    recipient: &str,
    subject: &str,
    body: String,
) -> Result<(), String> {
    // Log de diagnóstico: mostra a ligação que vai ser tentada (sem revelar a password)
    tracing::info!(
        host = %config.host,
        port = config.port,
        secure = ?config.secure,
        authenticated = config.user.is_some(),
        "a tentar enviar email via SMTP"
    );
    let from: Mailbox = config
        .from
        .parse()
        .map_err(|_| "SMTP_FROM inválido".to_string())?;
    let to: Mailbox = recipient
        .parse()
        .map_err(|_| "email inválido".to_string())?;
    let message = Message::builder()
        .from(from)
        .to(to)
        .subject(subject)
        .body(body)
        .map_err(|error| error.to_string())?;

    let transport = match config.secure {
        SmtpSecureMode::Tls => {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host)
                .map_err(|error| error.to_string())?
        }
        SmtpSecureMode::StartTls => {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.host)
                .map_err(|error| error.to_string())?
        }
        SmtpSecureMode::None => AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&config.host),
    }
    .port(config.port);
    let transport = match (&config.user, &config.password) {
        (Some(user), Some(password)) => {
            transport.credentials(Credentials::new(user.clone(), password.clone()))
        }
        _ => transport,
    }
    .build();

    transport
        .send(message)
        .await
        .map_err(|error| {
            // Constrói a cadeia completa de causas do erro (ex.: connection -> io -> dns)
            let mut details = error.to_string();
            let mut source = std::error::Error::source(&error);
            while let Some(cause) = source {
                details.push_str(&format!(" | causado por: {cause}"));
                source = cause.source();
            }
            // Classificação rápida para debug de conectividade
            if error.is_tls() {
                details.push_str(" [TLS: SMTP_SECURE/porta incompatível com o servidor?]");
            }
            if error.is_timeout() {
                details.push_str(" [TIMEOUT: porta bloqueada por firewall/security list/ISP?]");
            }
            if error.is_response() {
                details.push_str(" [SERVIDOR SMTP REJEITOU: verifique o código na mensagem, ex. 535=credenciais, 550/571=remetente não aprovado]");
            }
            details
        })?;
    Ok(())
}

#[cfg(test)]
mod template_tests {
    use super::*;

    #[test]
    fn songs_list_shows_inserted_by() {
        let song = SongListItem {
            id: 1,
            code: "W0001".into(),
            name: "Teste".into(),
            org_name: String::new(),
            composer: String::new(),
            artistas: String::new(),
            org_key: String::new(),
            youtube: None,
            favorite: false,
            view_count: 0,
            inserted_by: "Utilizador Teste".into(),
            updated_by: String::new(),
            can_edit: true,
            is_pending: false,
            can_delete: false,
            can_approve: false,
        };
        let tpl = SongsListTemplate { heading: "Resultados da Pesquisa".to_string(), songs: vec![song] };
        let html = tpl.render().expect("render falhou");
        assert!(
            html.contains("Inserido por Utilizador Teste"),
            "o HTML não contém 'Inserido por':\n{html}"
        );
    }

    #[test]
    fn songs_list_hides_corrected_when_same_user() {
        let song = SongListItem {
            id: 1,
            code: "W0001".into(),
            name: "Teste".into(),
            org_name: String::new(),
            composer: String::new(),
            artistas: String::new(),
            org_key: String::new(),
            youtube: None,
            favorite: false,
            view_count: 0,
            inserted_by: "Ana".into(),
            updated_by: "Ana".into(),
            can_edit: true,
            is_pending: false,
            can_delete: false,
            can_approve: false,
        };
        let tpl = SongsListTemplate { heading: "Resultados da Pesquisa".to_string(), songs: vec![song] };
        let html = tpl.render().expect("render falhou");
        assert!(html.contains("Inserido por Ana"));
        assert!(!html.contains("Corrigido por"), "não deve mostrar 'Corrigido por' quando é o mesmo utilizador:\n{html}");
    }
}
fn normalize_newlines(value: String) -> String {
    value
        .replace("\\r\\n", "\n")
        .replace("\\n", "\n")
        .replace("\\r", "\n")
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

fn valid_profile(profile: i32) -> bool {
    (1..=4).contains(&profile)
}

/// Extrai o ID de 11 caracteres de um link do YouTube, ou devolve o valor
/// original se não for possível identificar um ID válido.
fn normalize_youtube(value: String) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    // Já é um ID puro
    if trimmed.len() == 11
        && trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return trimmed.to_string();
    }
    // Garante protocolo para o URL parser
    let candidate = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else {
        format!("https://{}", trimmed)
    };
    let Ok(parsed) = url::Url::parse(&candidate) else {
        return value;
    };
    let mut id = parsed
        .query_pairs()
        .find(|(k, _)| k == "v")
        .map(|(_, v)| v.into_owned());
    if id.is_none() && parsed.host_str() == Some("youtu.be") {
        id = parsed
            .path_segments()
            .and_then(|mut segs| segs.next())
            .map(String::from);
    }
    if id.is_none() {
        let path = parsed.path().to_string();
        for prefix in ["/embed/", "/shorts/", "/live/"] {
            if let Some(rest) = path.strip_prefix(prefix) {
                id = rest.split('/').next().map(String::from);
                break;
            }
        }
    }
    match id {
        Some(id)
            if id.len() == 11
                && id
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') =>
        {
            id
        }
        _ => value,
    }
}

/// Check if a user is a demo account by their ID.
async fn is_user_demo(pool: &PgPool, user_id: i32) -> bool {
    sqlx::query_scalar::<_, bool>("SELECT COALESCE(is_demo, FALSE) FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .unwrap_or(false)
}

async fn create_auth_token(pool: &PgPool, user: &DbUser) -> Option<String> {
    let exp = Utc::now()
        .checked_add_signed(Duration::days(7))?
        .timestamp() as usize;
    let is_demo = is_user_demo(pool, user.id).await;
    let claims = AuthClaims {
        id: user.id,
        email: user.email.clone(),
        nome: user.nome.clone(),
        perfil: user.perfil.clone(),
        is_demo,
        exp,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(auth_secret().as_bytes()),
    )
    .ok()
}

fn is_authenticated(jar: &CookieJar) -> bool {
    authenticated_user_id(jar).is_some()
}

fn authenticated_user_id(jar: &CookieJar) -> Option<i32> {
    let token = match jar.get("token") {
        Some(cookie) => cookie.value().to_string(),
        None => return None,
    };

    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;

    decode::<AuthClaims>(
        &token,
        &DecodingKey::from_secret(auth_secret().as_bytes()),
        &validation,
    )
    .ok()
    .map(|data| data.claims.id)
}

async fn is_admin_user(pool: &PgPool, jar: &CookieJar) -> bool {
    let Some(user_id) = authenticated_user_id(jar) else {
        return false;
    };
    let perfil = sqlx::query_scalar::<_, i32>("SELECT perfil FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .unwrap_or(1);
    perfil >= 3
}

/// Check if user can approve song submissions: moderador (2) ou programador (4).
/// Colaborador (3) pode submeter músicas mas não se pode auto-aprovar nem aprovar as dos outros.
async fn is_moderator_user(pool: &PgPool, user_id: i32) -> bool {
    let perfil: Option<i32> = sqlx::query_scalar("SELECT perfil FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    matches!(perfil, Some(2) | Some(4))
}

/// Check if user can submit new song chords: colaborador (3), moderador (2) ou programador (4).
/// Utilizador comum (1) não pode criar músicas.
async fn can_submit_song_user(pool: &PgPool, user_id: i32) -> bool {
    let perfil: Option<i32> = sqlx::query_scalar("SELECT perfil FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    matches!(perfil, Some(2) | Some(3) | Some(4))
}

/// Check if the user can moderate songs (moderator or programmer).
async fn can_moderate_song(pool: &PgPool, user_id: Option<i32>) -> bool {
    let Some(user_id) = user_id else {
        return false;
    };
    is_moderator_user(pool, user_id).await
}

/// Check if the authenticated user is a demo account (reads `is_demo` from the JWT claims).
#[allow(dead_code)]
fn is_demo_user(jar: &CookieJar) -> bool {
    let token = match jar.get("token") {
        Some(cookie) => cookie.value().to_string(),
        None => return false,
    };
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    decode::<AuthClaims>(&token, &DecodingKey::from_secret(auth_secret().as_bytes()), &validation)
        .ok()
        .map(|data| data.claims.is_demo)
        .unwrap_or(false)
}

/// Delete all songs created by demo users. Returns the number of songs deleted.
async fn cleanup_demo_data(pool: &PgPool) -> Result<u64, sqlx::Error> {
    // Delete related records first (FK constraints), then the songs themselves.
    let _ = sqlx::query(
        r#"DELETE FROM "UserFavoriteSongs" WHERE "songId" IN (SELECT "ID" FROM songs WHERE "IdInsertUser" IN (SELECT id FROM users WHERE is_demo = TRUE))"#,
    )
    .execute(pool)
    .await;
    let _ = sqlx::query(
        r#"DELETE FROM "UserChordSettings" WHERE "songId" IN (SELECT "ID" FROM songs WHERE "IdInsertUser" IN (SELECT id FROM users WHERE is_demo = TRUE))"#,
    )
    .execute(pool)
    .await;
    let _ = sqlx::query(
        r#"DELETE FROM setlistsongs WHERE "IDSong" IN (SELECT "ID" FROM songs WHERE "IdInsertUser" IN (SELECT id FROM users WHERE is_demo = TRUE))"#,
    )
    .execute(pool)
    .await;
    let result = sqlx::query(
        r#"DELETE FROM songs WHERE "IdInsertUser" IN (SELECT id FROM users WHERE is_demo = TRUE)"#,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

/// When a demo user logs in, check if their oldest demo song is older than 24h.
/// If so, purge all demo data before they enter.
async fn auto_cleanup_on_demo_login(pool: &PgPool) {
    let oldest: Option<chrono::NaiveDateTime> = sqlx::query_scalar(
        r#"SELECT MIN("createdAt") FROM songs WHERE "IdInsertUser" IN (SELECT id FROM users WHERE is_demo = TRUE)"#,
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    let should_cleanup = match oldest {
        Some(ts) => {
            let elapsed = chrono::Utc::now().naive_utc() - ts;
            elapsed.num_hours() >= 24
        }
        None => false,
    };
    if should_cleanup {
        match cleanup_demo_data(pool).await {
            Ok(n) => tracing::info!("auto-cleanup demo data: {n} songs deleted"),
            Err(e) => tracing::error!("auto-cleanup demo data failed: {e}"),
        }
    }
}

async fn login_page(jar: CookieJar) -> impl IntoResponse {
    if is_authenticated(&jar) {
        return Redirect::to("/app").into_response();
    }

    let tpl = LoginTemplate {};
    let rendered = tpl
        .render()
        .unwrap_or_else(|e| format!("Erro ao renderizar template: {}", e));
    (
        [(header::HeaderName::from_static("cache-control"), "no-store")],
        Html(rendered),
    )
        .into_response()
}

async fn forgot_password_page(jar: CookieJar) -> impl IntoResponse {
    if is_authenticated(&jar) {
        return Redirect::to("/app").into_response();
    }

    let tpl = ForgotPasswordTemplate {};
    let rendered = tpl
        .render()
        .unwrap_or_else(|e| format!("Erro ao renderizar template: {}", e));
    (
        [(header::HeaderName::from_static("cache-control"), "no-store")],
        Html(rendered),
    )
        .into_response()
}

async fn register_page(jar: CookieJar) -> impl IntoResponse {
    if is_authenticated(&jar) {
        return Redirect::to("/app").into_response();
    }

    let tpl = RegisterTemplate {};
    let rendered = tpl
        .render()
        .unwrap_or_else(|e| format!("Erro ao renderizar template: {}", e));
    (
        [(header::HeaderName::from_static("cache-control"), "no-store")],
        Html(rendered),
    )
        .into_response()
}

#[derive(Deserialize)]
struct LoginFormData {
    email: String,
    password: String,
}

async fn login_submit(
    Extension(pool): Extension<PgPool>,
    jar: CookieJar,
    Form(form): Form<LoginFormData>,
) -> impl IntoResponse {
    let email = form.email.trim().to_lowercase();
    let password = form.password;

    if email.is_empty() || password.is_empty() {
        return (StatusCode::BAD_REQUEST, Html("<p class=\"text-center text-red-400 mb-4 text-sm sm:text-base\">Erro: Email e password são obrigatórios</p>".to_string())).into_response();
    }

    let user = sqlx::query_as::<_, DbUser>(
        "SELECT id, email, nome, perfil, igreja, password_hash, ativo FROM users WHERE LOWER(email) = $1 LIMIT 1",
    )
    .bind(&email)
    .fetch_optional(&pool)
    .await;

    let user = match user {
        Ok(Some(user)) => user,
        Ok(None) => {
            tracing::warn!(email = %email, "login rejected: user not found");
            return (StatusCode::UNAUTHORIZED, Html("<p class=\"text-center text-red-400 mb-4 text-sm sm:text-base\">Erro: Utilizador não encontrado</p>".to_string())).into_response();
        }
        Err(error) => {
            tracing::error!(email = %email, error = %error, "login failed: database query error");
            return (StatusCode::INTERNAL_SERVER_ERROR, Html("<p class=\"text-center text-red-400 mb-4 text-sm sm:text-base\">Erro: Erro ao fazer login</p>".to_string())).into_response();
        }
    };

    if !user.ativo {
        tracing::warn!(user_id = user.id, email = %email, "login rejected: inactive user");
        return (StatusCode::FORBIDDEN, Html("<p class=\"text-center text-red-400 mb-4 text-sm sm:text-base\">Erro: Utilizador inativo</p>".to_string())).into_response();
    }

    let password_match = bcrypt::verify(password, &user.password_hash).unwrap_or(false);
    if !password_match {
        tracing::warn!(user_id = user.id, email = %email, "login rejected: invalid password");
        return (StatusCode::UNAUTHORIZED, Html("<p class=\"text-center text-red-400 mb-4 text-sm sm:text-base\">Erro: Password incorreta</p>".to_string())).into_response();
    }

    let token = match create_auth_token(&pool, &user).await {
        Some(token) => token,
        None => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Html("<p class=\"text-center text-red-400 mb-4 text-sm sm:text-base\">Erro: Não foi possível criar sessão</p>".to_string())).into_response();
        }
    };

    let cookie = Cookie::build(("token", token))
        .path("/")
        .http_only(false) // Set to true in production for security
        .same_site(SameSite::Lax)
        .build();

    tracing::info!(user_id = user.id, email = %email, "login successful");

    // Record user activity for statistics
    let _ = sqlx::query(
        "INSERT INTO user_activity (\"userId\", \"action\", \"createdAt\") VALUES ($1, 'login', NOW())",
    )
    .bind(user.id)
    .execute(&pool)
    .await;

    // Auto-cleanup demo data on demo login if older than 24h
    if is_user_demo(&pool, user.id).await {
        auto_cleanup_on_demo_login(&pool).await;
    }

    (
        jar.add(cookie),
        [(header::HeaderName::from_static("hx-redirect"), "/app")],
        Html("<p class=\"text-center text-green-400 mb-4 text-sm sm:text-base\">Login bem-sucedido!</p>".to_string()),
    )
        .into_response()
}

#[derive(Deserialize)]
struct ForgotPasswordFormData {
    email: String,
}

#[derive(Deserialize)]
struct ResetPasswordFormData {
    token: String,
    password: String,
    password_confirmation: String,
}

#[derive(Deserialize)]
struct RegisterFormData {
    nome: String,
    email: String,
    igreja: Option<String>,
}

async fn register_submit(
    Extension(pool): Extension<PgPool>,
    Form(form): Form<RegisterFormData>,
) -> Html<String> {
    let email = form.email.trim().to_lowercase();
    let nome = form.nome.trim().to_string();
    if nome.is_empty() || email.is_empty() {
        return Html("<div class=\"p-4 bg-red-500/20 border border-red-500/50 rounded-lg\"><p class=\"text-red-400 text-sm\">Nome e email são obrigatórios</p></div>".to_string());
    }
    let igreja = form.igreja.and_then(|s| {
        let trimmed = s.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    });

    let existing = sqlx::query_as::<_, (i32, bool)>(
        "SELECT id, ativo FROM users WHERE LOWER(email) = $1 LIMIT 1",
    )
    .bind(&email)
    .fetch_optional(&pool)
    .await
    .ok()
    .flatten();

    let user_id = match existing {
        Some((_, true)) => {
            return Html("<div class=\"p-4 bg-yellow-500/20 border border-yellow-500/50 rounded-lg\"><p class=\"text-yellow-300 text-sm\">Este email já está registado. Se esqueceu a password, use a opção \"Esqueceu Password?\".</p></div>".to_string());
        }
        Some((id, false)) => {
            // Conta pendente de confirmação: atualiza os dados e reenvia o email
            let _ = sqlx::query("UPDATE users SET nome = $1, igreja = $2 WHERE id = $3")
                .bind(&nome)
                .bind(&igreja)
                .bind(id)
                .execute(&pool)
                .await;
            id
        }
        None => {
            let placeholder_hash = match bcrypt::hash(new_reset_token(), bcrypt::DEFAULT_COST) {
                Ok(hash) => hash,
                Err(_) => {
                    return Html(
                        "<p class=\"text-red-400\">Erro ao processar registo</p>".to_string(),
                    );
                }
            };
            let inserted = sqlx::query_scalar::<_, i32>(
                "INSERT INTO users (email, nome, password_hash, perfil, igreja, ativo) VALUES ($1, $2, $3, 1, $4, FALSE) RETURNING id",
            )
            .bind(&email)
            .bind(&nome)
            .bind(&placeholder_hash)
            .bind(&igreja)
            .fetch_one(&pool)
            .await;
            match inserted {
                Ok(id) => id,
                Err(error) => {
                    tracing::error!(%error, "failed to create pending registration");
                    return Html("<p class=\"text-red-400\">Erro ao criar registo. Tente novamente.</p>".to_string());
                }
            }
        }
    };

    let config = smtp_config();
    let token = new_reset_token();
    let token_hash = hash_reset_token(&token);
    let expires_at = Utc::now() + Duration::minutes(config.reset_ttl_minutes);
    let saved = sqlx::query("DELETE FROM password_reset_tokens WHERE user_id = $1 AND used_at IS NULL")
        .bind(user_id)
        .execute(&pool)
        .await
        .is_ok()
        && sqlx::query("INSERT INTO password_reset_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3)")
            .bind(user_id)
            .bind(&token_hash)
            .bind(expires_at)
            .execute(&pool)
            .await
            .is_ok();

    if saved {
        let confirm_link = format!(
            "{}/reset-password?token={}",
            config.app_base_url.trim_end_matches('/'),
            token
        );
        if let Err(error) = send_registration_email(&config, &email, &confirm_link).await {
            tracing::error!(%error, "failed to send registration confirmation email");
        }
    } else {
        tracing::error!(user_id, "failed to create registration confirmation token");
    }

    Html("<div class=\"p-4 bg-green-500/20 border border-green-500/50 rounded-lg\"><p class=\"text-green-400 text-sm\">Registo efetuado! Verifique o seu email para confirmar a conta e definir a password.</p></div>".to_string())
}

async fn forgot_password_submit(
    Extension(pool): Extension<PgPool>,
    Form(form): Form<ForgotPasswordFormData>,
) -> Html<String> {
    let email = form.email.trim().to_lowercase();
    if email.is_empty() {
        return Html("<div class=\"p-4 bg-red-500/20 border border-red-500/50 rounded-lg\"><p class=\"text-red-400 text-sm\">Erro: Email é obrigatório</p></div>".to_string());
    }

    let user = sqlx::query_as::<_, (i32, String)>(
        "SELECT id, email FROM users WHERE LOWER(email) = $1 AND ativo = TRUE LIMIT 1",
    )
    .bind(&email)
    .fetch_optional(&pool)
    .await;

    if let Ok(Some((user_id, recipient))) = user {
        let config = smtp_config();
        let token = new_reset_token();
        let token_hash = hash_reset_token(&token);
        let expires_at = Utc::now() + Duration::minutes(config.reset_ttl_minutes);
        let saved = sqlx::query("DELETE FROM password_reset_tokens WHERE user_id = $1 AND used_at IS NULL")
            .bind(user_id)
            .execute(&pool)
            .await
            .is_ok()
            && sqlx::query("INSERT INTO password_reset_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3)")
                .bind(user_id)
                .bind(&token_hash)
                .bind(expires_at)
                .execute(&pool)
                .await
                .is_ok();

        if saved {
            let reset_link = format!(
                "{}/reset-password?token={}",
                config.app_base_url.trim_end_matches('/'),
                token
            );
            if let Err(error) = send_password_reset_email(&config, &recipient, &reset_link).await {
                tracing::error!(%error, "failed to send password reset email");
            }
        } else {
            tracing::error!(user_id, "failed to create password reset token");
        }
    }

    Html("<div class=\"p-4 bg-green-500/20 border border-green-500/50 rounded-lg\"><p class=\"text-green-400 text-sm\">Se um utilizador com este email existir, será enviado um link de reset para o seu email.</p></div>".to_string())
}

async fn reset_password_page(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    let token = params.get("token").cloned().unwrap_or_default();
    if token.is_empty() {
        return Redirect::to("/forgot-password").into_response();
    }
    let rendered = ResetPasswordTemplate { token }
        .render()
        .unwrap_or_else(|error| format!("Erro ao renderizar template: {error}"));
    (
        [(header::HeaderName::from_static("cache-control"), "no-store")],
        Html(rendered),
    )
        .into_response()
}

async fn reset_password_submit(
    Extension(pool): Extension<PgPool>,
    Form(form): Form<ResetPasswordFormData>,
) -> impl IntoResponse {
    if form.password.len() < 6 || form.password != form.password_confirmation {
        return Html("<div class=\"p-4 bg-red-500/20 border border-red-500/50 rounded-lg\"><p class=\"text-red-400 text-sm\">As passwords devem coincidir e ter pelo menos 6 caracteres.</p></div>".to_string()).into_response();
    }

    let token_hash = hash_reset_token(&form.token);
    let mut transaction = match pool.begin().await {
        Ok(transaction) => transaction,
        Err(error) => {
            tracing::error!(%error, "failed to start password reset transaction");
            return Html("<p class=\"text-red-400\">Não foi possível alterar a password. Tente novamente.</p>".to_string()).into_response();
        }
    };
    let user_id = sqlx::query_scalar::<_, i32>("UPDATE password_reset_tokens SET used_at = NOW() WHERE token_hash = $1 AND used_at IS NULL AND expires_at > NOW() RETURNING user_id")
        .bind(&token_hash)
        .fetch_optional(&mut *transaction)
        .await;
    let user_id = match user_id {
        Ok(Some(user_id)) => user_id,
        Ok(None) => {
            return Html(
                "<p class=\"text-red-400\">Este link é inválido, já foi usado ou expirou.</p>"
                    .to_string(),
            )
            .into_response()
        }
        Err(error) => {
            tracing::error!(%error, "failed to consume password reset token");
            return Html("<p class=\"text-red-400\">Não foi possível alterar a password. Tente novamente.</p>".to_string()).into_response();
        }
    };
    let password_hash = match bcrypt::hash(&form.password, bcrypt::DEFAULT_COST) {
        Ok(hash) => hash,
        Err(error) => {
            tracing::error!(%error, "failed to hash reset password");
            return Html("<p class=\"text-red-400\">Não foi possível alterar a password. Tente novamente.</p>".to_string()).into_response();
        }
    };
    if let Err(error) = sqlx::query("UPDATE users SET password_hash = $1, ativo = TRUE WHERE id = $2")
        .bind(password_hash)
        .bind(user_id)
        .execute(&mut *transaction)
        .await
    {
        tracing::error!(%error, "failed to update reset password");
        return Html(
            "<p class=\"text-red-400\">Não foi possível alterar a password. Tente novamente.</p>"
                .to_string(),
        )
        .into_response();
    }
    if let Err(error) = transaction.commit().await {
        tracing::error!(%error, "failed to commit password reset");
        return Html(
            "<p class=\"text-red-400\">Não foi possível alterar a password. Tente novamente.</p>"
                .to_string(),
        )
        .into_response();
    }
    (
        [(header::HeaderName::from_static("hx-redirect"), "/")],
        Html("<p class=\"text-green-400\">Password alterada com sucesso. A redirecionar para o login…</p>".to_string()),
    ).into_response()
}

async fn logout(jar: CookieJar) -> impl IntoResponse {
    (
        jar.remove(Cookie::build("token").build()),
        Redirect::to("/"),
    )
        .into_response()
}

// User Management - Programmer only
async fn users_manage_page(
    jar: CookieJar,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    let Some(user_id) = authenticated_user_id(&jar) else {
        return Redirect::to("/").into_response();
    };

    let user = sqlx::query_as::<_, DbUser>(
        "SELECT id, email, nome, perfil, igreja, password_hash, ativo FROM users WHERE id = $1 LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(&pool)
    .await
    .ok()
    .flatten();

    let Some(user) = user else {
        return Redirect::to("/logout").into_response();
    };

    if user.perfil.unwrap_or(1) != 4 {
        return Redirect::to("/app").into_response();
    }

    let tpl = UsersManageTemplate {};
    let rendered = tpl
        .render()
        .unwrap_or_else(|e| format!("Erro ao renderizar template: {}", e));
    (
        [(header::HeaderName::from_static("cache-control"), "no-store")],
        Html(rendered),
    )
        .into_response()
}

#[derive(Template)]
#[template(path = "users_manage.html")]
struct UsersManageTemplate {}

#[derive(Deserialize)]
struct CreateUserForm {
    nome: String,
    email: String,
    password: String,
    igreja: Option<String>,
    perfil: i32,
    ativo: Option<bool>,
}

async fn create_user_htmx(
    Extension(pool): Extension<PgPool>,
    jar: CookieJar,
    Form(form): Form<CreateUserForm>,
) -> impl IntoResponse {
    let Some(user_id) = authenticated_user_id(&jar) else {
        return Html("<p class=\"text-red-400\">Sessão expirada</p>".to_string()).into_response();
    };

    let current_user =
        sqlx::query_as::<_, (Option<i32>,)>("SELECT perfil FROM users WHERE id = $1 LIMIT 1")
            .bind(user_id)
            .fetch_optional(&pool)
            .await
            .ok()
            .flatten();

    if current_user.map(|(p,)| p.unwrap_or(1)) != Some(4) {
        return Html("<p class=\"text-red-400\">Acesso negado</p>".to_string()).into_response();
    }

    let email = form.email.trim().to_lowercase();
    let nome = form.nome.trim();
    if nome.is_empty() || email.is_empty() || form.password.len() < 6 || !valid_profile(form.perfil)
    {
        return Html("<p class=\"text-red-400\">Preencha nome, email, password (mínimo 6 caracteres) e um perfil válido</p>".to_string()).into_response();
    }
    let password_hash = match bcrypt::hash(&form.password, bcrypt::DEFAULT_COST) {
        Ok(hash) => hash,
        Err(_) => {
            return Html("<p class=\"text-red-400\">Erro ao processar password</p>".to_string())
                .into_response();
        }
    };

    let igreja = form.igreja.and_then(|s| {
        if s.trim().is_empty() {
            None
        } else {
            Some(s.trim().to_string())
        }
    });
    let ativo = form.ativo.unwrap_or(true);

    let existing =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE LOWER(email) = $1")
            .bind(&email)
            .fetch_one(&pool)
            .await
            .unwrap_or(0);

    if existing > 0 {
        return Html("<p class=\"text-red-400\">Este email já está registado</p>".to_string())
            .into_response();
    }

    let result = sqlx::query(
        "INSERT INTO users (email, nome, password_hash, perfil, igreja, ativo) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(&email)
    .bind(nome)
    .bind(&password_hash)
    .bind(form.perfil)
    .bind(&igreja)
    .bind(ativo)
    .execute(&pool)
    .await;

    match result {
        Ok(_) => (
            [(
                header::HeaderName::from_static("hx-trigger"),
                "usersChanged",
            )],
            Html("<p class=\"text-green-400\">✓ Utilizador criado com sucesso!</p>".to_string()),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "failed to create user");
            Html("<p class=\"text-red-400\">Erro ao criar utilizador</p>".to_string())
                .into_response()
        }
    }
}

async fn list_users_htmx(Extension(pool): Extension<PgPool>, jar: CookieJar) -> impl IntoResponse {
    let Some(user_id) = authenticated_user_id(&jar) else {
        return Html("<p class=\"text-red-400\">Sessão expirada</p>".to_string()).into_response();
    };

    let current_user =
        sqlx::query_as::<_, (Option<i32>,)>("SELECT perfil FROM users WHERE id = $1 LIMIT 1")
            .bind(user_id)
            .fetch_optional(&pool)
            .await
            .ok()
            .flatten();

    if current_user.map(|(p,)| p.unwrap_or(1)) != Some(4) {
        return Html("<p class=\"text-red-400\">Acesso negado</p>".to_string()).into_response();
    }

    let users = sqlx::query_as::<_, (i32, String, String, Option<i32>, Option<String>, bool)>(
        "SELECT id, email, nome, perfil, igreja, ativo FROM users ORDER BY nome ASC",
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    if users.is_empty() {
        return Html(
            "<p class=\"text-gray-400 text-center py-8\">Nenhum utilizador encontrado</p>",
        )
        .into_response();
    }

    let rows = users
        .iter()
        .map(|(id, email, nome, perfil, igreja, ativo)| {
            let perfil_label = match perfil.unwrap_or(1) {
                2 => "Moderador",
                3 => "Colaborador",
                4 => "Programador",
                _ => "Utilizador",
            };
            let status_class = if *ativo { "text-green-400" } else { "text-red-400" };
            let status_text = if *ativo { "✓ Ativo" } else { "✗ Inativo" };
            let igreja_display = igreja.as_deref().unwrap_or("-");

            format!(
                r##"<tr class="border-b border-gray-700/50 hover:bg-gray-700/30">
                  <td class="py-3 px-4">
                    <div class="font-medium text-white">{}</div>
                    <div class="text-xs text-gray-400">{}</div>
                  </td>
                  <td class="py-3 px-4 text-gray-300">{}</td>
                  <td class="py-3 px-4">
                    <span class="px-2 py-1 rounded text-xs font-medium bg-blue-500/20 text-blue-300 border border-blue-400/30">{}</span>
                  </td>
                  <td class="py-3 px-4 {}">{}</td>
                  <td class="py-3 px-4 text-right">
                    <button hx-get="/htmx/users/{}/edit" hx-target="#edit-user-{}" hx-swap="innerHTML" class="text-blue-400 hover:text-blue-300 mr-2" title="Editar">✏️</button>
                    <button hx-post="/htmx/users/{}/toggle" hx-target="#edit-user-{}" hx-swap="innerHTML" class="text-yellow-400 hover:text-yellow-300 mr-2" title="Ativar/Desativar">🔄</button>
                    <button hx-get="/htmx/users/{}/password" hx-target="#edit-user-{}" hx-swap="innerHTML" class="text-purple-400 hover:text-purple-300 mr-2" title="Alterar Password">🔑</button>
                    <button hx-post="/htmx/users/{}/delete" hx-target="#user-form-message" hx-swap="innerHTML" hx-confirm="Tem a certeza que pretende apagar este utilizador? Esta ação não pode ser revertida." class="text-red-400 hover:text-red-300" title="Apagar">🗑️</button>
                  </td>
                </tr>
                <tr id="edit-user-{}"></tr>"##,
                escape_html(nome), escape_html(email), escape_html(igreja_display), perfil_label, status_class, status_text, id, id, id, id, id, id, id, id
            )
        })
        .collect::<Vec<_>>()
        .join("");

    Html(format!(
        r#"<div class="overflow-x-auto">
          <table class="w-full text-sm text-gray-300">
            <thead>
              <tr class="border-b border-gray-700 bg-gray-900/50">
                <th class="text-left py-3 px-4 font-semibold text-gray-200">Utilizador</th>
                <th class="text-left py-3 px-4 font-semibold text-gray-200">Igreja</th>
                <th class="text-left py-3 px-4 font-semibold text-gray-200">Perfil</th>
                <th class="text-left py-3 px-4 font-semibold text-gray-200">Status</th>
                <th class="text-right py-3 px-4 font-semibold text-gray-200">Ações</th>
              </tr>
            </thead>
            <tbody>
              {}
            </tbody>
          </table>
        </div>"#,
        rows
    ))
    .into_response()
}

#[derive(Deserialize)]
struct UpdateUserForm {
    nome: String,
    email: String,
    igreja: Option<String>,
    perfil: i32,
}

async fn edit_user_form_htmx(
    Extension(pool): Extension<PgPool>,
    Path(user_id): Path<i32>,
    jar: CookieJar,
) -> impl IntoResponse {
    let Some(current_user_id) = authenticated_user_id(&jar) else {
        return Html("<p class=\"text-red-400\">Sessão expirada</p>".to_string()).into_response();
    };

    let current_user =
        sqlx::query_as::<_, (Option<i32>,)>("SELECT perfil FROM users WHERE id = $1 LIMIT 1")
            .bind(current_user_id)
            .fetch_optional(&pool)
            .await
            .ok()
            .flatten();

    if current_user.map(|(p,)| p.unwrap_or(1)) != Some(4) {
        return Html("<p class=\"text-red-400\">Acesso negado</p>".to_string()).into_response();
    }

    let user = sqlx::query_as::<_, (String, String, Option<i32>, Option<String>, bool)>(
        "SELECT email, nome, perfil, igreja, ativo FROM users WHERE id = $1 LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(&pool)
    .await
    .ok()
    .flatten();

    match user {
        Some((email, nome, perfil, igreja, _ativo)) => {
            let perfil_value = perfil.unwrap_or(1);

            Html(format!(
                r##"<td colspan="5" class="py-4 px-4 bg-gray-700/30">
                  <form hx-post="/htmx/users/{}/update" hx-target="#edit-user-{}" hx-swap="innerHTML" class="space-y-3">
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                      <div>
                        <label class="block text-xs font-medium text-gray-400 mb-1">Nome</label>
                        <input type="text" name="nome" value="{}" class="w-full px-3 py-1.5 rounded bg-gray-600 text-white border border-gray-500 text-sm" />
                      </div>
                      <div>
                        <label class="block text-xs font-medium text-gray-400 mb-1">Email</label>
                        <input type="email" name="email" value="{}" class="w-full px-3 py-1.5 rounded bg-gray-600 text-white border border-gray-500 text-sm" />
                      </div>
                      <div>
                        <label class="block text-xs font-medium text-gray-400 mb-1">Igreja</label>
                        <input type="text" name="igreja" value="{}" class="w-full px-3 py-1.5 rounded bg-gray-600 text-white border border-gray-500 text-sm" />
                      </div>
                      <div>
                        <label class="block text-xs font-medium text-gray-400 mb-1">Perfil</label>
                        <select name="perfil" class="w-full px-3 py-1.5 rounded bg-gray-600 text-white border border-gray-500 text-sm">
                          <option value="1" {}>Utilizador</option>
                          <option value="2" {}>Moderador</option>
                          <option value="3" {}>Colaborador</option>
                          <option value="4" {}>Programador</option>
                        </select>
                      </div>
                    </div>
                    <div class="flex gap-2">
                      <button type="submit" class="bg-green-600 hover:bg-green-700 text-white text-sm py-1.5 px-4 rounded">Guardar</button>
                      <button type="button" hx-get="/htmx/users/list" hx-target="#users-list-container" hx-swap="innerHTML" class="bg-gray-600 hover:bg-gray-700 text-white text-sm py-1.5 px-4 rounded">Cancelar</button>
                    </div>
                  </form>
                </td>"##,
                user_id, user_id, escape_html(&nome), escape_html(&email), escape_html(&igreja.unwrap_or_default()),
                if perfil_value == 1 { "selected" } else { "" },
                if perfil_value == 2 { "selected" } else { "" },
                if perfil_value == 3 { "selected" } else { "" },
                if perfil_value == 4 { "selected" } else { "" },
            )).into_response()
        }
        None => Html("<p class=\"text-red-400\">Utilizador não encontrado</p>".to_string())
            .into_response(),
    }
}

async fn update_user_htmx(
    Extension(pool): Extension<PgPool>,
    Path(user_id): Path<i32>,
    jar: CookieJar,
    Form(form): Form<UpdateUserForm>,
) -> impl IntoResponse {
    let Some(current_user_id) = authenticated_user_id(&jar) else {
        return Html("<p class=\"text-red-400\">Sessão expirada</p>".to_string()).into_response();
    };

    let current_user =
        sqlx::query_as::<_, (Option<i32>,)>("SELECT perfil FROM users WHERE id = $1 LIMIT 1")
            .bind(current_user_id)
            .fetch_optional(&pool)
            .await
            .ok()
            .flatten();

    if current_user.map(|(p,)| p.unwrap_or(1)) != Some(4) {
        return Html("<p class=\"text-red-400\">Acesso negado</p>".to_string()).into_response();
    }

    let email = form.email.trim().to_lowercase();
    let nome = form.nome.trim();
    if nome.is_empty() || email.is_empty() || !valid_profile(form.perfil) {
        return Html(
            "<p class=\"text-red-400\">Preencha nome, email e um perfil válido</p>".to_string(),
        )
        .into_response();
    }
    if user_id == current_user_id && form.perfil != 4 {
        return Html("<p class=\"text-yellow-400\">⚠ Não pode remover o seu próprio perfil de Programador</p>".to_string()).into_response();
    }
    let igreja = form.igreja.and_then(|s| {
        if s.trim().is_empty() {
            None
        } else {
            Some(s.trim().to_string())
        }
    });

    let email_in_use = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM users WHERE LOWER(email) = $1 AND id <> $2",
    )
    .bind(&email)
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap_or(0);
    if email_in_use > 0 {
        return Html("<p class=\"text-red-400\">Este email já está registado</p>".to_string())
            .into_response();
    }

    let result = sqlx::query(
        "UPDATE users SET email = $1, nome = $2, igreja = $3, perfil = $4 WHERE id = $5",
    )
    .bind(&email)
    .bind(nome)
    .bind(&igreja)
    .bind(form.perfil)
    .bind(user_id)
    .execute(&pool)
    .await;

    match result {
        Ok(result) if result.rows_affected() == 1 => (
            [(
                header::HeaderName::from_static("hx-trigger"),
                "usersChanged",
            )],
            Html("<p class=\"text-green-400\">✓ Utilizador atualizado!</p>".to_string()),
        )
            .into_response(),
        Ok(_) => Html("<p class=\"text-red-400\">Utilizador não encontrado</p>".to_string())
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "failed to update user");
            Html("<p class=\"text-red-400\">Erro ao actualizar utilizador</p>".to_string())
                .into_response()
        }
    }
}

async fn toggle_user_active_htmx(
    Extension(pool): Extension<PgPool>,
    Path(user_id): Path<i32>,
    jar: CookieJar,
) -> impl IntoResponse {
    let Some(current_user_id) = authenticated_user_id(&jar) else {
        return Html("<p class=\"text-red-400\">Sessão expirada</p>".to_string()).into_response();
    };

    let current_user =
        sqlx::query_as::<_, (Option<i32>,)>("SELECT perfil FROM users WHERE id = $1 LIMIT 1")
            .bind(current_user_id)
            .fetch_optional(&pool)
            .await
            .ok()
            .flatten();

    if current_user.map(|(p,)| p.unwrap_or(1)) != Some(4) {
        return Html("<p class=\"text-red-400\">Acesso negado</p>".to_string()).into_response();
    }

    if user_id == current_user_id {
        return Html(
            "<p class=\"text-yellow-400\">⚠ Não pode desactivar a sua própria conta</p>"
                .to_string(),
        )
        .into_response();
    }

    let result = sqlx::query("UPDATE users SET ativo = NOT ativo WHERE id = $1")
        .bind(user_id)
        .execute(&pool)
        .await;

    match result {
        Ok(_) => {
            Html(r#"<script>htmx.ajax('GET', '/htmx/users/list', {target: '#users-list-container', swap: 'innerHTML'});</script><p class="text-green-400">✓ Status actualizado!</p>"#).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "failed to toggle user status");
            Html("<p class=\"text-red-400\">Erro ao actualizar status</p>".to_string()).into_response()
        }
    }
}

async fn user_password_form_htmx(
    Extension(pool): Extension<PgPool>,
    Path(user_id): Path<i32>,
    jar: CookieJar,
) -> impl IntoResponse {
    let Some(current_user_id) = authenticated_user_id(&jar) else {
        return Html("<p class=\"text-red-400\">Sessão expirada</p>".to_string()).into_response();
    };
    if !is_programmer_user(&pool, current_user_id).await {
        return Html("<p class=\"text-red-400\">Acesso negado</p>".to_string()).into_response();
    }

    let exists = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap_or(false);
    if !exists {
        return Html("<p class=\"text-red-400\">Utilizador não encontrado</p>".to_string())
            .into_response();
    }

    Html(format!(r##"<td colspan="5" class="py-4 px-4 bg-gray-700/30"><form hx-post="/htmx/users/{user_id}/password" hx-target="#edit-user-{user_id}" hx-swap="innerHTML" class="flex flex-wrap items-end gap-3"><div><label class="block text-xs font-medium text-gray-400 mb-1">Nova password</label><input type="password" name="password" required minlength="6" class="px-3 py-1.5 rounded bg-gray-600 text-white border border-gray-500 text-sm" placeholder="Mínimo 6 caracteres" /></div><button type="submit" class="bg-purple-600 hover:bg-purple-700 text-white text-sm py-1.5 px-4 rounded">Alterar password</button><button type="button" hx-get="/htmx/users/list" hx-target="#users-list-container" hx-swap="innerHTML" class="bg-gray-600 hover:bg-gray-700 text-white text-sm py-1.5 px-4 rounded">Cancelar</button></form></td>"##)).into_response()
}

async fn delete_user_htmx(
    Extension(pool): Extension<PgPool>,
    Path(user_id): Path<i32>,
    jar: CookieJar,
) -> impl IntoResponse {
    let Some(current_user_id) = authenticated_user_id(&jar) else {
        return Html("<p class=\"text-red-400\">Sessão expirada</p>".to_string()).into_response();
    };
    if !is_programmer_user(&pool, current_user_id).await {
        return Html("<p class=\"text-red-400\">Acesso negado</p>".to_string()).into_response();
    }
    if user_id == current_user_id {
        return Html(
            "<p class=\"text-yellow-400\">⚠ Não pode apagar a sua própria conta</p>".to_string(),
        )
        .into_response();
    }

    match sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&pool)
        .await
    {
        Ok(result) if result.rows_affected() == 1 => (
            [(
                header::HeaderName::from_static("hx-trigger"),
                "usersChanged",
            )],
            Html("<p class=\"text-green-400\">✓ Utilizador apagado com sucesso!</p>".to_string()),
        )
            .into_response(),
        Ok(_) => Html("<p class=\"text-red-400\">Utilizador não encontrado</p>".to_string())
            .into_response(),
        Err(error) => {
            tracing::error!(%error, user_id, "failed to delete user");
            Html("<p class=\"text-red-400\">Não foi possível apagar o utilizador porque tem dados associados</p>".to_string()).into_response()
        }
    }
}

/// HTMX handler: programmer-only endpoint to manually purge all demo data.
async fn cleanup_demo_htmx(
    Extension(pool): Extension<PgPool>,
    jar: CookieJar,
) -> impl IntoResponse {
    let Some(current_user_id) = authenticated_user_id(&jar) else {
        return Html("<p class=\"text-red-400\">Sessão expirada</p>".to_string()).into_response();
    };
    if !is_programmer_user(&pool, current_user_id).await {
        return Html("<p class=\"text-red-400\">Acesso negado</p>".to_string()).into_response();
    }
    match cleanup_demo_data(&pool).await {
        Ok(n) => (
            [(
                header::HeaderName::from_static("hx-trigger"),
                "demo-cleaned",
            )],
            Html(format!("<p class=\"text-green-400\">✓ Dados demo limpos: {n} músicas removidas.</p>")),
        )
            .into_response(),
        Err(error) => {
            tracing::error!(%error, "failed to cleanup demo data");
            Html("<p class=\"text-red-400\">Erro ao limpar dados demo.</p>".to_string()).into_response()
        }
    }
}

async fn is_programmer_user(pool: &PgPool, user_id: i32) -> bool {
    sqlx::query_scalar::<_, Option<i32>>("SELECT perfil FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .flatten()
        == Some(4)
}

/// Uma música pode ser editada pelo seu criador (`IdInsertUser`), por um moderador ou por um programador.
async fn can_edit_song(pool: &PgPool, user_id: i32, owner: Option<i32>) -> bool {
    is_moderator_user(pool, user_id).await || owner == Some(user_id)
}

/// Moderadores e programadores podem apagar qualquer música.
/// Colaboradores (perfil >= 3) só podem apagar as suas próprias.
async fn can_delete_song(pool: &PgPool, user_id: i32, owner: Option<i32>, jar: &CookieJar) -> bool {
    if is_moderator_user(pool, user_id).await {
        return true;
    }
    if is_admin_user(pool, jar).await {
        return owner == Some(user_id);
    }
    false
}

#[derive(Deserialize)]
struct PasswordUpdateForm {
    password: String,
}

async fn change_user_password_htmx(
    Extension(pool): Extension<PgPool>,
    Path(user_id): Path<i32>,
    jar: CookieJar,
    Form(form): Form<PasswordUpdateForm>,
) -> impl IntoResponse {
    let Some(current_user_id) = authenticated_user_id(&jar) else {
        return Html("<p class=\"text-red-400\">Sessão expirada</p>".to_string()).into_response();
    };

    let current_user =
        sqlx::query_as::<_, (Option<i32>,)>("SELECT perfil FROM users WHERE id = $1 LIMIT 1")
            .bind(current_user_id)
            .fetch_optional(&pool)
            .await
            .ok()
            .flatten();

    if current_user.map(|(p,)| p.unwrap_or(1)) != Some(4) {
        return Html("<p class=\"text-red-400\">Acesso negado</p>".to_string()).into_response();
    }

    if form.password.len() < 6 {
        return Html(
            "<p class=\"text-red-400\">Password deve ter pelo menos 6 caracteres</p>".to_string(),
        )
        .into_response();
    }

    let password_hash = match bcrypt::hash(&form.password, bcrypt::DEFAULT_COST) {
        Ok(hash) => hash,
        Err(_) => {
            return Html("<p class=\"text-red-400\">Erro ao processar password</p>".to_string())
                .into_response();
        }
    };

    let result = sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
        .bind(&password_hash)
        .bind(user_id)
        .execute(&pool)
        .await;

    match result {
        Ok(result) if result.rows_affected() == 1 => (
            [(
                header::HeaderName::from_static("hx-trigger"),
                "usersChanged",
            )],
            Html("<p class=\"text-green-400\">✓ Password alterada com sucesso!</p>".to_string()),
        )
            .into_response(),
        Ok(_) => Html("<p class=\"text-red-400\">Utilizador não encontrado</p>".to_string())
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "failed to update password");
            Html("<p class=\"text-red-400\">Erro ao alterar password</p>".to_string())
                .into_response()
        }
    }
}

async fn statistics_page(
    Extension(pool): Extension<PgPool>,
    Query(params): Query<HashMap<String, String>>,
    jar: CookieJar,
) -> impl IntoResponse {
    let Some(user_id) = authenticated_user_id(&jar) else {
        return Redirect::to("/").into_response();
    };

    let user = sqlx::query_as::<_, DbUser>(
        "SELECT id, email, nome, perfil, igreja, password_hash, ativo FROM users WHERE id = $1 LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(&pool)
    .await
    .ok()
    .flatten();

    let Some(user) = user else {
        return Redirect::to("/logout").into_response();
    };

    // Only programmers (perfil = 4) can access statistics
    if user.perfil.unwrap_or(1) != 4 {
        return Redirect::to("/app").into_response();
    }

    // Parse date range from query params (default: last 30 days)
    let days: i64 = params
        .get("days")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(30)
        .clamp(1, 365);

    let total_unique_users = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(DISTINCT \"userId\") FROM user_activity WHERE \"createdAt\" >= NOW() - ($1 || ' days')::interval",
    )
    .bind(days.to_string())
    .fetch_one(&pool)
    .await
    .unwrap_or(0);

    let total_logins = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM user_activity WHERE \"action\" = 'login' AND \"createdAt\" >= NOW() - ($1 || ' days')::interval",
    )
    .bind(days.to_string())
    .fetch_one(&pool)
    .await
    .unwrap_or(0);

    let daily_rows = sqlx::query_as::<_, (chrono::NaiveDate, i64)>(
        "SELECT DATE(\"createdAt\") as date, COUNT(DISTINCT \"userId\") as unique_users FROM user_activity WHERE \"createdAt\" >= NOW() - ($1 || ' days')::interval GROUP BY DATE(\"createdAt\") ORDER BY date DESC",
    )
    .bind(days.to_string())
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let daily_stats: Vec<DailyUserStat> = daily_rows
        .into_iter()
        .map(|(date, unique_users)| DailyUserStat {
            date: date.to_string(),
            unique_users,
        })
        .collect();

    // 50 utilizadores distintos com atividade mais recente
    let recent_rows = sqlx::query_as::<_, RecentUserStat>(
        "SELECT nome, email, perfil, last_active FROM (SELECT DISTINCT ON (ua.\"userId\") u.nome AS nome, u.email AS email, u.perfil AS perfil, ua.\"createdAt\" AS last_active FROM user_activity ua JOIN users u ON u.id = ua.\"userId\" ORDER BY ua.\"userId\", ua.\"createdAt\" DESC) recent ORDER BY last_active DESC LIMIT 50",
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let stats_data = StatisticsData {
        total_unique_users,
        total_logins,
        daily_stats,
    };

    let tpl = StatisticsTemplate { days };
    let rendered = tpl
        .render()
        .unwrap_or_else(|e| format!("Erro ao renderizar template: {}", e));

    // Build the HTML with the statistics data
    let stats_html = format!(
        r#"
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 mb-8">
          <div class="bg-gradient-to-br from-blue-500 to-blue-600 rounded-xl p-6 text-white shadow-lg">
            <div class="text-4xl font-bold mb-2">{total_unique_users}</div>
            <div class="text-sm text-blue-100">Utilizadores únicos (últimos {days} dias)</div>
          </div>
          <div class="bg-gradient-to-br from-purple-500 to-purple-600 rounded-xl p-6 text-white shadow-lg">
            <div class="text-4xl font-bold mb-2">{total_logins}</div>
            <div class="text-sm text-purple-100">Total de logins (últimos {days} dias)</div>
          </div>
        </div>
        <div class="bg-gray-800/50 rounded-xl p-6 border border-gray-700/50">
          <h3 class="text-xl font-bold text-white mb-4">Utilizadores únicos por dia</h3>
          <div class="overflow-x-auto">
            <table class="w-full text-sm text-gray-300">
              <thead>
                <tr class="border-b border-gray-700">
                  <th class="text-left py-2 px-3">Data</th>
                  <th class="text-right py-2 px-3">Utilizadores únicos</th>
                </tr>
              </thead>
              <tbody>
                {daily_rows_html}
              </tbody>
            </table>
          </div>
        </div>
        <div class="bg-gray-800/50 rounded-xl p-6 border border-gray-700/50">
          <h3 class="text-xl font-bold text-white mb-4">Últimos 50 utilizadores que usaram o programa</h3>
          <div class="overflow-x-auto">
            <table class="w-full text-sm text-gray-300">
              <thead>
                <tr class="border-b border-gray-700">
                  <th class="text-left py-2 px-3">Utilizador</th>
                  <th class="text-left py-2 px-3">Email</th>
                  <th class="text-left py-2 px-3">Perfil</th>
                  <th class="text-right py-2 px-3">Última atividade</th>
                </tr>
              </thead>
              <tbody>
                {recent_users_html}
              </tbody>
            </table>
          </div>
        </div>
        "#,
        total_unique_users = stats_data.total_unique_users,
        total_logins = stats_data.total_logins,
        days = days,
        daily_rows_html = stats_data
            .daily_stats
            .iter()
            .map(|stat| {
                format!(
                    r#"<tr class="border-b border-gray-700/50 hover:bg-gray-700/30">
                  <td class="py-2 px-3">{}</td>
                  <td class="text-right py-2 px-3 font-semibold text-white">{}</td>
                </tr>"#,
                    stat.date, stat.unique_users
                )
            })
            .collect::<Vec<_>>()
            .join(""),
        recent_users_html = recent_rows
            .iter()
            .map(|stat| {
                let perfil_label = match stat.perfil.unwrap_or(1) {
                    2 => "Moderador",
                    3 => "Colaborador",
                    4 => "Programador",
                    _ => "Utilizador",
                };
                let name = stat
                    .nome
                    .clone()
                    .filter(|n| !n.trim().is_empty())
                    .unwrap_or_else(|| "(sem nome)".to_string());
                let last_active = stat
                    .last_active
                    .with_timezone(&chrono::Local)
                    .format("%d/%m/%Y %H:%M")
                    .to_string();
                format!(
                    r#"<tr class="border-b border-gray-700/50 hover:bg-gray-700/30">
                  <td class="py-2 px-3">{}</td>
                  <td class="py-2 px-3 text-gray-400">{}</td>
                  <td class="py-2 px-3 text-xs uppercase tracking-wide text-blue-300">{}</td>
                  <td class="text-right py-2 px-3 text-xs text-gray-400">{}</td>
                </tr>"#,
                    name, stat.email, perfil_label, last_active
                )
            })
            .collect::<Vec<_>>()
            .join("")
    );

    // Replace the placeholder in the template
    let final_html = rendered.replace("<!-- STATS_CONTENT -->", &stats_html);
    Html(final_html).into_response()
}

async fn index(Extension(pool): Extension<PgPool>, jar: CookieJar) -> impl IntoResponse {
    let Some(user_id) = authenticated_user_id(&jar) else {
        return Redirect::to("/").into_response();
    };

    let user = sqlx::query_as::<_, DbUser>(
        "SELECT id, email, nome, perfil, igreja, password_hash, ativo FROM users WHERE id = $1 LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(&pool)
    .await
    .ok()
    .flatten();

    let Some(user) = user else {
        return Redirect::to("/logout").into_response();
    };

    let full_name = user.nome.unwrap_or_else(|| "Utilizador".to_string());
    let first_name = full_name
        .split_whitespace()
        .next()
        .unwrap_or("Utilizador")
        .to_string();
    let initials = full_name
        .split_whitespace()
        .filter_map(|name| name.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase();
    let profile_label = match user.perfil.unwrap_or(1) {
        2 => "Moderador",
        3 => "Colaborador",
        4 => "Programador",
        _ => "Utilizador",
    }
    .to_string();
    let is_programmer = user.perfil.unwrap_or(1) == 4;
    let is_moderator = is_moderator_user(&pool, user.id).await;
    let can_submit_song = can_submit_song_user(&pool, user.id).await;
    let is_demo = is_user_demo(&pool, user.id).await;

    let song_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM songs")
        .fetch_one(&pool)
        .await
        .unwrap_or(0);
    let favorite_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM \"UserFavoriteSongs\" WHERE \"userId\" = $1",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap_or(0);
    let setlist_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM usersetlists WHERE \"IDUser\" = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap_or(0);

    let genres = sqlx::query_as::<_, (i32, Option<String>, Option<String>)>(
        "SELECT \"ID\", \"Desc\", \"CodeSufix\" FROM genretypes ORDER BY \"ID\"",
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|(id, desc, prefix)| GenreTypeRow {
        id,
        desc: desc.unwrap_or_else(|| format!("Tipo {id}")),
        prefix: prefix.unwrap_or_default(),
    })
    .collect();

    let tpl = IndexTemplate {
        first_name,
        initials: if initials.is_empty() {
            "U".to_string()
        } else {
            initials
        },
        profile_label,
        church: user.igreja.unwrap_or_default(),
        song_count,
        favorite_count,
        setlist_count,
        is_programmer,
        is_moderator,
        can_submit_song,
        is_demo,
        genres,
    };
    let rendered = tpl
        .render()
        .unwrap_or_else(|e| format!("Erro ao renderizar template: {}", e));
    (
        [(header::HeaderName::from_static("cache-control"), "no-store")],
        Html(rendered),
    )
        .into_response()
}

async fn list_songs_htmx(Extension(pool): Extension<PgPool>, jar: CookieJar) -> impl IntoResponse {
    if !is_authenticated(&jar) {
        return Html(
            "<div class=\"p-4 text-red-400\">Sessão expirada. Faça login novamente.</div>"
                .to_string(),
        )
        .into_response();
    }

    let user_id = authenticated_user_id(&jar);
    let is_admin = is_admin_user(&pool, &jar).await;
    let is_moderator = match user_id {
        Some(uid) => is_moderator_user(&pool, uid).await,
        None => false,
    };
    let owner_id = user_id.unwrap_or(0);
    let fav_ids = favorite_song_ids(&pool, user_id).await;
    let sql = format!(
        "SELECT \"ID\", \"Code\", \"Name\", \"OrgName\", \"Composer\", \"ChordPro\", \"Lyrics\", \"Themes\", \"Youtube\", \"GenreType\", \"Artistas\", \"OrgKey\", \"OrgTempo\", \"Copyright\", \"Favorite\", \"IdInsertUser\", \"IdUpdateUser\", \"ViewCount\", \"createdAt\", \"updatedAt\", \"Status\" FROM songs WHERE (\"Status\" = 'approved' OR \"IdInsertUser\" = {owner_id} OR {is_moderator}) ORDER BY \"createdAt\" DESC NULLS LAST, \"ID\" DESC LIMIT 50"
    );
    let rows = sqlx::query_as::<_, Song>(&sql)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

    let list: Vec<SongListItem> = rows
        .into_iter()
        .map(|s| SongListItem {
            id: s.id,
            code: s.code.unwrap_or_default(),
            name: s.name.unwrap_or_else(|| "(sem título)".to_string()),
            org_name: s.org_name.unwrap_or_default(),
            composer: s.composer.unwrap_or_default(),
            artistas: s.artistas.unwrap_or_default(),
            org_key: s.org_key.unwrap_or_default(),
            youtube: s.youtube.filter(|url| !url.trim().is_empty()),
            favorite: fav_ids.contains(&s.id),
            view_count: s.view_count.unwrap_or(0) as i64,
            inserted_by: String::new(),
            updated_by: String::new(),
            is_pending: s.status.as_deref() == Some("pending"),
            can_edit: is_moderator || user_id == s.id_insert_user,
            can_delete: is_moderator || (is_admin && user_id == s.id_insert_user),
            can_approve: is_moderator,
        })
        .collect();

    let tpl = SongsListTemplate { heading: "Resultados da Pesquisa".to_string(), songs: list };
    let rendered = tpl
        .render()
        .unwrap_or_else(|e| format!("Erro ao renderizar template: {}", e));
    Html(rendered).into_response()
}

async fn list_songs_json(Extension(pool): Extension<PgPool>, jar: CookieJar) -> impl IntoResponse {
    if !is_authenticated(&jar) {
        return (StatusCode::UNAUTHORIZED, Json(Vec::<SongListItem>::new())).into_response();
    }

    let user_id = authenticated_user_id(&jar);
    let is_admin = is_admin_user(&pool, &jar).await;
    let is_moderator = match user_id {
        Some(uid) => is_moderator_user(&pool, uid).await,
        None => false,
    };
    let owner_id = user_id.unwrap_or(0);
    let fav_ids = favorite_song_ids(&pool, user_id).await;
    let sql = format!(
        "SELECT \"ID\", \"Code\", \"Name\", \"OrgName\", \"Composer\", \"ChordPro\", \"Lyrics\", \"Themes\", \"Youtube\", \"GenreType\", \"Artistas\", \"OrgKey\", \"OrgTempo\", \"Copyright\", \"Favorite\", \"IdInsertUser\", \"IdUpdateUser\", \"ViewCount\", \"createdAt\", \"updatedAt\", \"Status\" FROM songs WHERE (\"Status\" = 'approved' OR \"IdInsertUser\" = {owner_id} OR {is_moderator}) ORDER BY \"ID\" DESC LIMIT 50"
    );
    let rows = sqlx::query_as::<_, Song>(&sql)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let list: Vec<SongListItem> = rows
        .into_iter()
        .map(|s| SongListItem {
            id: s.id,
            code: s.code.unwrap_or_default(),
            name: s.name.unwrap_or_else(|| "(sem título)".to_string()),
            org_name: s.org_name.unwrap_or_default(),
            composer: s.composer.unwrap_or_default(),
            artistas: s.artistas.unwrap_or_default(),
            org_key: s.org_key.unwrap_or_default(),
            youtube: s.youtube.filter(|url| !url.trim().is_empty()),
            favorite: fav_ids.contains(&s.id),
            view_count: s.view_count.unwrap_or(0) as i64,
            inserted_by: String::new(),
            updated_by: String::new(),
            is_pending: s.status.as_deref() == Some("pending"),
            can_edit: is_moderator || user_id == s.id_insert_user,
            can_delete: is_moderator || (is_admin && user_id == s.id_insert_user),
            can_approve: is_moderator,
        })
        .collect();

    Json(list).into_response()
}

async fn library_page(
    Extension(pool): Extension<PgPool>,
    jar: CookieJar,
    title: &str,
    subtitle: &str,
    query: &str,
    user_id: Option<i32>,
) -> impl IntoResponse {
    if !is_authenticated(&jar) {
        return Redirect::to("/").into_response();
    }

    let current_user_id = authenticated_user_id(&jar);
    let is_admin = is_admin_user(&pool, &jar).await;
    let is_moderator = match current_user_id {
        Some(uid) => is_moderator_user(&pool, uid).await,
        None => false,
    };
    let owner_id = current_user_id.unwrap_or(0);
    let fav_ids = favorite_song_ids(&pool, current_user_id).await;
    let filtered_query = format!(
        "SELECT * FROM ({query}) \"song_visibility\" WHERE (\"Status\" = 'approved' OR \"IdInsertUser\" = {owner_id} OR {is_moderator})"
    );

    let rows = if let Some(user_id) = user_id {
        sqlx::query_as::<_, Song>(&filtered_query)
            .bind(user_id)
            .fetch_all(&pool)
            .await
            .unwrap_or_default()
    } else {
        sqlx::query_as::<_, Song>(&filtered_query)
            .fetch_all(&pool)
            .await
            .unwrap_or_default()
    };
    let songs = rows
        .into_iter()
        .map(|song| SongListItem {
            id: song.id,
            code: song.code.unwrap_or_default(),
            name: song.name.unwrap_or_else(|| "(sem título)".to_string()),
            org_name: song.org_name.unwrap_or_default(),
            composer: song.composer.unwrap_or_default(),
            artistas: song.artistas.unwrap_or_default(),
            org_key: song.org_key.unwrap_or_default(),
            youtube: song.youtube.filter(|url| !url.trim().is_empty()),
            favorite: fav_ids.contains(&song.id),
            view_count: song.view_count.unwrap_or(0) as i64,
            inserted_by: String::new(),
            updated_by: String::new(),
            is_pending: song.status.as_deref() == Some("pending"),
            can_edit: is_moderator || current_user_id == song.id_insert_user,
            can_delete: is_moderator || (is_admin && current_user_id == song.id_insert_user),
            can_approve: is_moderator,
        })
        .collect();
    let template = LibraryTemplate {
        title: title.to_string(),
        subtitle: subtitle.to_string(),
        songs,
    };
    (
        [(header::HeaderName::from_static("cache-control"), "no-store")],
        Html(
            template
                .render()
                .unwrap_or_else(|error| format!("Erro ao renderizar template: {error}")),
        ),
    )
    .into_response()
}

async fn favorites_page(Extension(pool): Extension<PgPool>, jar: CookieJar) -> impl IntoResponse {
    let Some(user_id) = authenticated_user_id(&jar) else {
        return Redirect::to("/").into_response();
    };
    library_page(
        Extension(pool), jar, "❤️ Minhas Favoritas", "Suas músicas marcadas como favoritas",
        "SELECT s.\"ID\", s.\"Code\", s.\"Name\", s.\"OrgName\", s.\"Composer\", s.\"ChordPro\", s.\"Lyrics\", s.\"Themes\", s.\"Youtube\", s.\"GenreType\", s.\"Artistas\", s.\"OrgKey\", s.\"OrgTempo\", s.\"Copyright\", s.\"Favorite\", s.\"IdInsertUser\", s.\"IdUpdateUser\", s.\"ViewCount\", s.\"createdAt\", s.\"updatedAt\", s.\"Status\" FROM songs s JOIN \"UserFavoriteSongs\" f ON f.\"songId\" = s.\"ID\" WHERE f.\"userId\" = $1 ORDER BY f.\"favoritedAt\" DESC LIMIT 100",
        Some(user_id),
    ).await.into_response()
}

async fn most_viewed_page(Extension(pool): Extension<PgPool>, jar: CookieJar) -> impl IntoResponse {
    library_page(Extension(pool), jar, "Mais Acessadas", "As cifras mais consultadas", "SELECT \"ID\", \"Code\", \"Name\", \"OrgName\", \"Composer\", \"ChordPro\", \"Lyrics\", \"Themes\", \"Youtube\", \"GenreType\", \"Artistas\", \"OrgKey\", \"OrgTempo\", \"Copyright\", \"Favorite\", \"IdInsertUser\", \"IdUpdateUser\", \"ViewCount\", \"createdAt\", \"updatedAt\", \"Status\" FROM songs WHERE \"ViewCount\" > 0 ORDER BY \"ViewCount\" DESC NULLS LAST, \"ID\" DESC LIMIT 100", None).await.into_response()
}

async fn recent_page(Extension(pool): Extension<PgPool>, jar: CookieJar) -> impl IntoResponse {
    library_page(Extension(pool), jar, "Músicas Recentes", "As últimas cifras adicionadas", "SELECT \"ID\", \"Code\", \"Name\", \"OrgName\", \"Composer\", \"ChordPro\", \"Lyrics\", \"Themes\", \"Youtube\", \"GenreType\", \"Artistas\", \"OrgKey\", \"OrgTempo\", \"Copyright\", \"Favorite\", \"IdInsertUser\", \"IdUpdateUser\", \"ViewCount\", \"createdAt\", \"updatedAt\", \"Status\" FROM songs ORDER BY \"createdAt\" DESC NULLS LAST, \"ID\" DESC LIMIT 100", None).await.into_response()
}

/// Página de músicas pendentes de aprovação — apenas moderadores (2) e programadores (4).
/// Usa o mesmo cartão da pesquisa (songs_list.html), com badge "Pendente" e botão de aprovar.
async fn pending_page(Extension(pool): Extension<PgPool>, jar: CookieJar) -> impl IntoResponse {
    let Some(user_id) = authenticated_user_id(&jar) else {
        return Redirect::to("/").into_response();
    };
    if !is_moderator_user(&pool, user_id).await {
        return Redirect::to("/app").into_response();
    }

    let is_admin = is_admin_user(&pool, &jar).await;
    let is_moderator = true;
    let fav_ids = favorite_song_ids(&pool, Some(user_id)).await;
    let sql = "SELECT \"ID\", \"Code\", \"Name\", \"OrgName\", \"Composer\", \"ChordPro\", \"Lyrics\", \"Themes\", \"Youtube\", \"GenreType\", \"Artistas\", \"OrgKey\", \"OrgTempo\", \"Copyright\", \"Favorite\", \"IdInsertUser\", \"IdUpdateUser\", \"ViewCount\", \"createdAt\", \"updatedAt\", \"Status\" FROM songs WHERE \"Status\" = 'pending' ORDER BY \"createdAt\" DESC NULLS LAST, \"ID\" DESC LIMIT 100";
    let rows = sqlx::query_as::<_, Song>(sql)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

    // Nomes de quem inseriu/corrigiu (uma única query para todos os resultados)
    let mut user_ids: Vec<i32> = rows
        .iter()
        .filter_map(|s| s.id_insert_user.or(s.id_update_user))
        .collect();
    user_ids.sort_unstable();
    user_ids.dedup();
    let mut user_names: std::collections::HashMap<i32, String> = std::collections::HashMap::new();
    if !user_ids.is_empty() {
        let name_rows: Vec<(i32, Option<String>)> =
            sqlx::query_as("SELECT id, nome FROM users WHERE id = ANY($1)")
                .bind(&user_ids)
                .fetch_all(&pool)
                .await
                .unwrap_or_default();
        for (id, nome) in name_rows {
            user_names.insert(id, nome.unwrap_or_default());
        }
    }

    let songs: Vec<SongListItem> = rows
        .into_iter()
        .map(|s| SongListItem {
            id: s.id,
            code: s.code.unwrap_or_default(),
            name: s.name.unwrap_or_else(|| "(sem título)".to_string()),
            org_name: s.org_name.unwrap_or_default(),
            composer: s.composer.unwrap_or_default(),
            artistas: s.artistas.unwrap_or_default(),
            org_key: s.org_key.unwrap_or_default(),
            youtube: s.youtube.filter(|url| !url.trim().is_empty()),
            favorite: fav_ids.contains(&s.id),
            view_count: s.view_count.unwrap_or(0) as i64,
            inserted_by: s.id_insert_user.and_then(|id| user_names.get(&id).cloned()).unwrap_or_default(),
            updated_by: s.id_update_user.and_then(|id| user_names.get(&id).cloned()).unwrap_or_default(),
            is_pending: s.status.as_deref() == Some("pending"),
            can_edit: is_moderator || Some(user_id) == s.id_insert_user,
            can_delete: is_moderator || (is_admin && Some(user_id) == s.id_insert_user),
            can_approve: is_moderator,
        })
        .collect();

    let tpl = PendingTemplate { heading: "Músicas Pendentes".to_string(), songs };
    (
        [(header::HeaderName::from_static("cache-control"), "no-store")],
        Html(
            tpl.render()
                .unwrap_or_else(|error| format!("Erro ao renderizar template: {error}")),
        ),
    )
        .into_response()
}

/// Normaliza acentos e cedilhas para permitir pesquisar sem os mesmos.
/// Ex: "coração" -> "coracao", "José" -> "Jose", "ação" -> "acao"
fn normalize_accents(input: &str) -> String
{
    input
        .chars()
        .map(|c| match c {
            'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => 'a',
            'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'Å' => 'A',
            'è' | 'é' | 'ê' | 'ë' => 'e',
            'È' | 'É' | 'Ê' | 'Ë' => 'E',
            'ì' | 'í' | 'î' | 'ï' => 'i',
            'Ì' | 'Í' | 'Î' | 'Ï' => 'I',
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' => 'o',
            'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' | 'Ø' => 'O',
            'ù' | 'ú' | 'û' | 'ü' => 'u',
            'Ù' | 'Ú' | 'Û' | 'Ü' => 'U',
            'ñ' => 'n',
            'Ñ' => 'N',
            'ç' => 'c',
            'Ç' => 'C',
            'ý' | 'ÿ' => 'y',
            'Ý' | 'Ÿ' => 'Y',
            _ => c,
        })
        .collect()
}

async fn search_songs(
    Extension(pool): Extension<PgPool>,
    Query(params): Query<HashMap<String, String>>,
    jar: CookieJar,
) -> impl IntoResponse {
    if !is_authenticated(&jar) {
        return (StatusCode::UNAUTHORIZED, Json(Vec::<SongListItem>::new())).into_response();
    }

    let q = params
        .get("q")
        .map(|s| format!("%{}%", s))
        .unwrap_or_else(|| "%".to_string());

    let user_id = authenticated_user_id(&jar);
    let is_admin = is_admin_user(&pool, &jar).await;
    let is_moderator = match user_id {
        Some(uid) => is_moderator_user(&pool, uid).await,
        None => false,
    };
    let owner_id = user_id.unwrap_or(0);
    let sql = format!(
        "SELECT \"ID\", \"Code\", \"Name\", \"Artistas\", \"IdInsertUser\" FROM songs WHERE (\"Name\" ILIKE $1 OR \"Artistas\" ILIKE $1 OR \"Code\" ILIKE $1) AND (\"Status\" = 'approved' OR \"IdInsertUser\" = {owner_id} OR {is_moderator}) ORDER BY \"ID\" DESC LIMIT 50"
    );
    let rows = sqlx::query_as::<_, (i32, Option<String>, Option<String>, Option<String>, Option<i32>)>(&sql)
    .bind(q)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let list: Vec<SongListItem> = rows
        .into_iter()
        .map(|(id, code, name, artistas, owner)| SongListItem {
            id,
            code: code.unwrap_or_default(),
            name: name.unwrap_or_else(|| "(sem título)".to_string()),
            org_name: String::new(),
            composer: String::new(),
            artistas: artistas.unwrap_or_default(),
            org_key: String::new(),
            youtube: None,
            favorite: false,
            view_count: 0,
            inserted_by: String::new(),
            updated_by: String::new(),
            is_pending: false,
            can_edit: is_moderator || user_id == owner,
            can_delete: is_moderator || (is_admin && user_id == owner),
            can_approve: is_moderator,
        })
        .collect();

    Json(list).into_response()
}

async fn search_songs_htmx(
    Extension(pool): Extension<PgPool>,
    Query(params): Query<HashMap<String, String>>,
    jar: CookieJar,
) -> impl IntoResponse {
    if !is_authenticated(&jar) {
        return Html(
            "<div class=\"p-4 text-red-400\">Sessão expirada. Faça login novamente.</div>"
                .to_string(),
        )
        .into_response();
    }

    let q_raw = params
        .get("q")
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    // Se o campo de pesquisa está vazio, não mostrar resultados
    if q_raw.is_empty() {
        let tpl = SongsListTemplate {
            heading: "Resultados da Pesquisa".to_string(),
            songs: Vec::new(),
        };
        let rendered = tpl
            .render()
            .unwrap_or_else(|e| format!("Erro ao renderizar template: {}", e));
        return Html(rendered).into_response();
    }

    let q = normalize_accents(&format!("%{}%", q_raw));
    let letter = params
        .get("letter")
        .filter(|value| value.trim().len() == 1)
        .map(|value| format!("{}%", value.trim()))
        .unwrap_or_else(|| "%".to_string());
    let filter = params.get("filter").map(String::as_str).unwrap_or("all");
    let condition = match filter {
        "org_name" => "unaccent(COALESCE(s.\"OrgName\", '')) ILIKE unaccent($1)",
        "artistas" => "unaccent(COALESCE(s.\"Artistas\", '')) ILIKE unaccent($1)",
        "composer" => "unaccent(COALESCE(s.\"Composer\", '')) ILIKE unaccent($1)",
        "lyrics" => "unaccent(COALESCE(s.\"Lyrics\", '')) ILIKE unaccent($1)",
        _ => "(unaccent(COALESCE(s.\"Name\", '')) ILIKE unaccent($1) OR unaccent(COALESCE(s.\"Artistas\", '')) ILIKE unaccent($1) OR unaccent(COALESCE(s.\"Code\", '')) ILIKE unaccent($1))",
    };
    let sort_value = params.get("sort").map(String::as_str).unwrap_or("all");
    let sort = match sort_value {
        "alpha" => "s.\"Name\" ASC NULLS LAST, s.\"ID\" DESC",
        "type" => "g.\"Desc\" ASC NULLS LAST, s.\"Name\" ASC NULLS LAST",
        "tempo" => "s.\"OrgTempo\" ASC NULLS LAST, s.\"Name\" ASC NULLS LAST",
        "views" => "s.\"ViewCount\" DESC NULLS LAST, s.\"ID\" DESC",
        "recent" => "s.\"createdAt\" DESC NULLS LAST, s.\"ID\" DESC",
        _ => "s.\"ID\" DESC", // "all" - sem ordenação específica, por ID
    };
    // Filtro por tipo de música (coluna "GenreType" é o ID da tabela genretypes)
    let genre_filter = params
        .get("type")
        .and_then(|value| value.trim().parse::<i32>().ok());
    // Filtro de favoritas do utilizador autenticado (combinável com tipo/sort/letra)
    let want_favorites = params
        .get("fav")
        .map(|v| v.trim() == "1")
        .unwrap_or(false);
    let fav_filter_active = want_favorites && is_authenticated(&jar);
    // Faz-se o JOIN à tabela genretypes quando há filtro por tipo OU quando a
    // ordenação é por tipo. LEFT JOIN para não excluir músicas cujo tipo ainda
    // não consta na tabela.
    let need_join = genre_filter.is_some() || sort_value == "type";
    let genre_join = if need_join {
        " LEFT JOIN genretypes g ON g.\"ID\" = s.\"GenreType\" "
    } else {
        ""
    };
    // JOIN às favoritas do utilizador (id já validado como i32 pelo parser do token)
    let fav_join = match fav_filter_active.then(|| authenticated_user_id(&jar)).flatten() {
        Some(uid) => format!(" JOIN \"UserFavoriteSongs\" uf ON uf.\"songId\" = s.\"ID\" AND uf.\"userId\" = {uid} "),
        None => String::new(),
    };
    let genre_cond = match genre_filter {
        Some(gt) => format!(" AND s.\"GenreType\" = {gt}"),
        None => String::new(),
    };
    let user_id = authenticated_user_id(&jar);
    let is_admin = is_admin_user(&pool, &jar).await;
    let is_moderator = match user_id {
        Some(uid) => is_moderator_user(&pool, uid).await,
        None => false,
    };
    let owner_id = user_id.unwrap_or(0);
    let fav_ids = favorite_song_ids(&pool, user_id).await;
    let sql = format!("SELECT s.\"ID\", s.\"Code\", s.\"Name\", s.\"OrgName\", s.\"Composer\", s.\"ChordPro\", s.\"Lyrics\", s.\"Themes\", s.\"Youtube\", s.\"GenreType\", s.\"Artistas\", s.\"OrgKey\", s.\"OrgTempo\", s.\"Copyright\", s.\"Favorite\", s.\"IdInsertUser\", s.\"IdUpdateUser\", s.\"ViewCount\", s.\"createdAt\", s.\"updatedAt\", s.\"Status\" FROM songs s{genre_join}{fav_join} WHERE {condition} AND COALESCE(s.\"Name\", '') ILIKE $2{genre_cond} AND (s.\"Status\" = 'approved' OR s.\"IdInsertUser\" = {owner_id} OR {is_moderator}) ORDER BY {sort} LIMIT 100");

    let rows = sqlx::query_as::<_, Song>(&sql)
        .bind(q)
        .bind(letter)
        .fetch_all(&pool)
        .await
        .unwrap_or_else(|e| {
            eprintln!("[search_songs_htmx] Erro na query: {e}");
            Vec::new()
        });

    // Nomes de quem inseriu/corrigiu (uma única query para todos os resultados)
    let mut user_ids: Vec<i32> = rows
        .iter()
        .filter_map(|s| s.id_insert_user.or(s.id_update_user))
        .collect();
    user_ids.sort_unstable();
    user_ids.dedup();
    let mut user_names: std::collections::HashMap<i32, String> = std::collections::HashMap::new();
    if !user_ids.is_empty() {
        let name_rows: Vec<(i32, Option<String>)> =
            sqlx::query_as("SELECT id, nome FROM users WHERE id = ANY($1)")
                .bind(&user_ids)
                .fetch_all(&pool)
                .await
                .unwrap_or_default();
        for (id, nome) in name_rows {
            user_names.insert(id, nome.unwrap_or_default());
        }
    }

    let list: Vec<SongListItem> = rows
        .into_iter()
        .map(|s| SongListItem {
            id: s.id,
            code: s.code.unwrap_or_default(),
            name: s.name.unwrap_or_else(|| "(sem título)".to_string()),
            org_name: s.org_name.unwrap_or_default(),
            composer: s.composer.unwrap_or_default(),
            artistas: s.artistas.unwrap_or_default(),
            org_key: s.org_key.unwrap_or_default(),
            youtube: s.youtube.filter(|url| !url.trim().is_empty()),
            favorite: fav_ids.contains(&s.id) || fav_filter_active,
            view_count: s.view_count.unwrap_or(0) as i64,
            inserted_by: s.id_insert_user.and_then(|id| user_names.get(&id).cloned()).unwrap_or_default(),
            updated_by: s.id_update_user.and_then(|id| user_names.get(&id).cloned()).unwrap_or_default(),
            is_pending: s.status.as_deref() == Some("pending"),
            can_edit: is_moderator || user_id == s.id_insert_user,
            can_delete: is_moderator || (is_admin && user_id == s.id_insert_user),
            can_approve: is_moderator,
        })
        .collect();

    let tpl = SongsListTemplate { heading: "Resultados da Pesquisa".to_string(), songs: list };
    let rendered = tpl
        .render()
        .unwrap_or_else(|e| format!("Erro ao renderizar template: {}", e));
    Html(rendered).into_response()
}

async fn list_setlists_htmx(
    Extension(pool): Extension<PgPool>,
    jar: CookieJar,
) -> impl IntoResponse {
    let Some(user_id) = authenticated_user_id(&jar) else {
        return Html(
            "<div class=\"p-4 text-red-400\">Sessão expirada. Faça login novamente.</div>"
                .to_string(),
        )
        .into_response();
    };

    let rows = sqlx::query_as::<_, (i32, Option<chrono::NaiveDate>, Option<String>, i64)>("SELECT s.\"ID\", s.\"Data\", s.\"Desc\", COUNT(ss.\"IDSong\") FROM setlists s JOIN usersetlists us ON us.\"IDSetlist\" = s.\"ID\" LEFT JOIN setlistsongs ss ON ss.\"IDSetlist\" = s.\"ID\" WHERE us.\"IDUser\" = $1 GROUP BY s.\"ID\", s.\"Data\", s.\"Desc\" ORDER BY s.\"Data\" DESC NULLS LAST, s.\"ID\" DESC LIMIT 100")
        .bind(user_id)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

    let list: Vec<SetlistItem> = rows
        .into_iter()
        .map(|(id, data, desc, song_count)| SetlistItem {
            id,
            data: data.map(|d| d.to_string()).unwrap_or_default(),
            desc: desc.unwrap_or_default(),
            song_count,
        })
        .collect();

    let total_songs = list.iter().map(|setlist| setlist.song_count).sum();
    let tpl = SetlistsListTemplate {
        setlists: list,
        total_songs,
    };
    let rendered = tpl
        .render()
        .unwrap_or_else(|e| format!("Erro ao renderizar template: {}", e));
    Html(rendered).into_response()
}

async fn delete_setlist_htmx(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<i32>,
    jar: CookieJar,
) -> impl IntoResponse {
    let Some(user_id) = authenticated_user_id(&jar) else {
        return Html(
            "<div class=\"p-4 text-red-400\">Sessão expirada. Faça login novamente.</div>"
                .to_string(),
        )
        .into_response();
    };

    let owns_setlist = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM usersetlists WHERE \"IDUser\" = $1 AND \"IDSetlist\" = $2)",
    )
    .bind(user_id)
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap_or(false);
    if !owns_setlist {
        return Html("<div class=\"p-4 text-red-400\">Setlist não encontrada.</div>".to_string())
            .into_response();
    }

    let _ = sqlx::query("DELETE FROM setlistsongs WHERE \"IDSetlist\" = $1")
        .bind(id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM usersetlists WHERE \"IDSetlist\" = $1")
        .bind(id)
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM setlists WHERE \"ID\" = $1")
        .bind(id)
        .execute(&pool)
        .await;

    // Re-render the updated setlists list
    let rows = sqlx::query_as::<_, (i32, Option<chrono::NaiveDate>, Option<String>, i64)>("SELECT s.\"ID\", s.\"Data\", s.\"Desc\", COUNT(ss.\"IDSong\") FROM setlists s JOIN usersetlists us ON us.\"IDSetlist\" = s.\"ID\" LEFT JOIN setlistsongs ss ON ss.\"IDSetlist\" = s.\"ID\" WHERE us.\"IDUser\" = $1 GROUP BY s.\"ID\", s.\"Data\", s.\"Desc\" ORDER BY s.\"Data\" DESC NULLS LAST, s.\"ID\" DESC LIMIT 100")
        .bind(user_id)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

    let list: Vec<SetlistItem> = rows
        .into_iter()
        .map(|(id, data, desc, song_count)| SetlistItem {
            id,
            data: data.map(|d| d.to_string()).unwrap_or_default(),
            desc: desc.unwrap_or_default(),
            song_count,
        })
        .collect();

    let total_songs = list.iter().map(|setlist| setlist.song_count).sum();
    let tpl = SetlistsListTemplate {
        setlists: list,
        total_songs,
    };
    let rendered = tpl
        .render()
        .unwrap_or_else(|e| format!("Erro ao renderizar template: {}", e));
    Html(rendered).into_response()
}

async fn setlist_song_picker_htmx(
    Extension(pool): Extension<PgPool>,
    Path(song_id): Path<i32>,
    jar: CookieJar,
) -> impl IntoResponse {
    let Some(user_id) = authenticated_user_id(&jar) else {
        return Html(
            "<div class=\"p-4 text-red-400\">Sessão expirada. Faça login novamente.</div>"
                .to_string(),
        )
        .into_response();
    };

    let rows = sqlx::query_as::<_, (i32, Option<chrono::NaiveDate>, Option<String>, i64)>("SELECT s.\"ID\", s.\"Data\", s.\"Desc\", COUNT(ss.\"IDSong\") FROM setlists s JOIN usersetlists us ON us.\"IDSetlist\" = s.\"ID\" LEFT JOIN setlistsongs ss ON ss.\"IDSetlist\" = s.\"ID\" WHERE us.\"IDUser\" = $1 GROUP BY s.\"ID\", s.\"Data\", s.\"Desc\" ORDER BY s.\"Data\" DESC NULLS LAST, s.\"ID\" DESC LIMIT 100")
        .bind(user_id)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

    let setlists = rows
        .into_iter()
        .map(|(id, data, desc, song_count)| SetlistItem {
            id,
            data: data.map(|value| value.to_string()).unwrap_or_default(),
            desc: desc.unwrap_or_default(),
            song_count,
        })
        .collect();
    Html(
        SetlistSongPickerTemplate { song_id, setlists }
            .render()
            .unwrap_or_default(),
    )
    .into_response()
}

async fn setlist_detail_htmx(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<i32>,
    jar: CookieJar,
) -> impl IntoResponse {
    let Some(user_id) = authenticated_user_id(&jar) else {
        return Html(
            "<div class=\"p-4 text-red-400\">Sessão expirada. Faça login novamente.</div>"
                .to_string(),
        )
        .into_response();
    };
    let owns_setlist = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM usersetlists WHERE \"IDUser\" = $1 AND \"IDSetlist\" = $2)",
    )
    .bind(user_id)
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap_or(false);
    if !owns_setlist {
        return Html("<div class=\"p-4 text-red-400\">Setlist não encontrada.</div>".to_string())
            .into_response();
    }

    let row = sqlx::query_as::<_, Setlist>(
        "SELECT \"ID\", \"Data\", \"Desc\", \"IDGrupo\" FROM setlists WHERE \"ID\" = $1",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .unwrap_or(None);

    if let Some(s) = row {
        let rows = sqlx::query_as::<_, (i32, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>)>(
            "SELECT songs.\"ID\", songs.\"Name\", songs.\"OrgName\", songs.\"Composer\", songs.\"Artistas\", songs.\"Youtube\" FROM setlistsongs JOIN songs ON songs.\"ID\" = setlistsongs.\"IDSong\" WHERE setlistsongs.\"IDSetlist\" = $1 ORDER BY setlistsongs.\"Order\""
        )
        .bind(id)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

        let songs_view: Vec<SongInSetlist> = rows
            .into_iter()
            .map(
                |(id, name, org_name, composer, artistas, youtube)| SongInSetlist {
                    id,
                    name: name.unwrap_or_default(),
                    org_name: org_name.unwrap_or_default(),
                    composer: composer.unwrap_or_default(),
                    artistas: artistas.unwrap_or_default(),
                    youtube: youtube.filter(|url| !url.trim().is_empty()),
                },
            )
            .collect();

        let setlist_item = SetlistItem {
            id: s.id,
            data: s.data.map(|d| d.to_string()).unwrap_or_default(),
            desc: s.desc.unwrap_or_default(),
            song_count: songs_view.len() as i64,
        };
        let tpl = SetlistDetailTemplate {
            setlist: setlist_item,
            songs: songs_view,
        };
        let rendered = tpl
            .render()
            .unwrap_or_else(|e| format!("Erro ao renderizar template: {}", e));
        Html(rendered).into_response()
    } else {
        Html("<div class=\"p-4 text-red-600\">Setlist não encontrado</div>".to_string())
            .into_response()
    }
}

async fn create_setlist_share_link_htmx(
    Extension(pool): Extension<PgPool>,
    Path(setlist_id): Path<i32>,
    jar: CookieJar,
) -> impl IntoResponse {
    let Some(user_id) = authenticated_user_id(&jar) else {
        return Html("<p class=\"text-red-400\">Sessão expirada.</p>".to_string()).into_response();
    };
    let owns_setlist = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM usersetlists WHERE \"IDUser\" = $1 AND \"IDSetlist\" = $2)",
    )
    .bind(user_id)
    .bind(setlist_id)
    .fetch_one(&pool)
    .await
    .unwrap_or(false);
    if !owns_setlist {
        return Html("<p class=\"text-red-400\">Setlist não encontrada.</p>".to_string()).into_response();
    }

    let token = new_reset_token();
    let token = match sqlx::query_scalar::<_, String>(
        "INSERT INTO setlist_share_links (setlist_id, token, created_by) VALUES ($1, $2, $3) ON CONFLICT (setlist_id) DO UPDATE SET token = EXCLUDED.token, created_by = EXCLUDED.created_by RETURNING token",
    )
    .bind(setlist_id)
    .bind(token)
    .bind(user_id)
    .fetch_one(&pool)
    .await
    {
        Ok(token) => token,
        Err(error) => {
            tracing::error!(%error, setlist_id, "failed to create setlist share link");
            return Html("<p class=\"text-red-400\">Não foi possível criar o link.</p>".to_string()).into_response();
        }
    };
    let url = format!("{}/share/{}/1", smtp_config().app_base_url.trim_end_matches('/'), token);
    Html(format!(r#"<div class="w-full rounded-lg bg-violet-500/15 p-3 text-sm text-violet-100"><p class="mb-2 font-semibold">Link público da setlist</p><div class="flex gap-2"><input id="setlist-share-url" readonly value="{}" class="min-w-0 flex-1 rounded bg-gray-900 px-3 py-2 text-xs text-white" /><button type="button" onclick="navigator.clipboard.writeText(document.getElementById('setlist-share-url').value)" class="rounded bg-violet-600 px-3 py-2 font-semibold hover:bg-violet-700">Copiar</button></div><p class="mt-2 text-xs text-violet-200">Qualquer pessoa com este link pode ver as músicas.</p></div>"#, escape_html(&url))).into_response()
}

async fn new_setlist_form(jar: CookieJar) -> impl IntoResponse {
    if !is_authenticated(&jar) {
        return Html(
            "<div class=\"p-4 text-red-400\">Sessão expirada. Faça login novamente.</div>"
                .to_string(),
        )
        .into_response();
    }
    Html(SetlistCreateTemplate {}.render().unwrap_or_default()).into_response()
}

async fn new_setlist_for_song_form(Path(song_id): Path<i32>, jar: CookieJar) -> impl IntoResponse {
    if !is_authenticated(&jar) {
        return Html(
            "<div class=\"p-4 text-red-400\">Sessão expirada. Faça login novamente.</div>"
                .to_string(),
        )
        .into_response();
    }
    Html(
        SetlistSongCreateTemplate { song_id }
            .render()
            .unwrap_or_default(),
    )
    .into_response()
}

#[derive(Deserialize)]
struct NewSetlistForm {
    desc: String,
    id_grupo: Option<i32>,
}

async fn create_setlist(
    Extension(pool): Extension<PgPool>,
    jar: CookieJar,
    Form(form): Form<NewSetlistForm>,
) -> impl IntoResponse {
    let Some(user_id) = authenticated_user_id(&jar) else {
        return (
            StatusCode::UNAUTHORIZED,
            Html("<p class=\"text-red-400\">Sessão expirada.</p>".to_string()),
        )
            .into_response();
    };
    let desc = form.desc.trim();
    if desc.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Html("<p class=\"text-red-400\">A descrição é obrigatória.</p>".to_string()),
        )
            .into_response();
    }
    let result = sqlx::query_scalar::<_, i32>("INSERT INTO setlists (\"Data\", \"Desc\", \"IDGrupo\") VALUES (CURRENT_DATE, $1, $2) RETURNING \"ID\"")
        .bind(desc)
        .bind(form.id_grupo.unwrap_or(0))
        .fetch_one(&pool)
        .await;
    match result {
        Ok(setlist_id) => {
            let _ =
                sqlx::query("INSERT INTO usersetlists (\"IDUser\", \"IDSetlist\") VALUES ($1, $2)")
                    .bind(user_id)
                    .bind(setlist_id)
                    .execute(&pool)
                    .await;
            (
                [(
                    header::HeaderName::from_static("hx-trigger"),
                    "setlist-created",
                )],
                Html("<p class=\"text-green-400\">Setlist criada.</p>".to_string()),
            )
                .into_response()
        }
        Err(error) => {
            tracing::error!(error = %error, "setlist creation failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("<p class=\"text-red-400\">Não foi possível criar a setlist.</p>".to_string()),
            )
                .into_response()
        }
    }
}

async fn create_setlist_for_song(
    Extension(pool): Extension<PgPool>,
    Path(song_id): Path<i32>,
    jar: CookieJar,
    Form(form): Form<NewSetlistForm>,
) -> impl IntoResponse {
    let Some(user_id) = authenticated_user_id(&jar) else {
        return (
            StatusCode::UNAUTHORIZED,
            Html("<p class=\"text-red-400\">Sessão expirada.</p>".to_string()),
        )
            .into_response();
    };
    let desc = form.desc.trim();
    if desc.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Html("<p class=\"text-red-400\">A descrição é obrigatória.</p>".to_string()),
        )
            .into_response();
    }
    let setlist_id = match sqlx::query_scalar::<_, i32>("INSERT INTO setlists (\"Data\", \"Desc\", \"IDGrupo\") VALUES (CURRENT_DATE, $1, $2) RETURNING \"ID\"")
        .bind(desc)
        .bind(form.id_grupo.unwrap_or(0))
        .fetch_one(&pool)
        .await {
            Ok(id) => id,
            Err(error) => {
                tracing::error!(error = %error, "setlist creation failed");
                return (StatusCode::INTERNAL_SERVER_ERROR, Html("<p class=\"text-red-400\">Não foi possível criar a setlist.</p>".to_string())).into_response();
            }
        };
    let mut transaction = match pool.begin().await {
        Ok(transaction) => transaction,
        Err(error) => {
            tracing::error!(error = %error, "failed to begin setlist transaction");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(
                    "<p class=\"text-red-400\">Não foi possível adicionar a música.</p>"
                        .to_string(),
                ),
            )
                .into_response();
        }
    };
    let result = async {
        sqlx::query("INSERT INTO usersetlists (\"IDUser\", \"IDSetlist\") VALUES ($1, $2)")
            .bind(user_id)
            .bind(setlist_id)
            .execute(&mut *transaction)
            .await?;
        sqlx::query(
            "INSERT INTO setlistsongs (\"IDSetlist\", \"IDSong\", \"Order\") VALUES ($1, $2, 1)",
        )
        .bind(setlist_id)
        .bind(song_id)
        .execute(&mut *transaction)
        .await?;
        transaction.commit().await
    }
    .await;
    match result {
        Ok(_) => (
            [(
                header::HeaderName::from_static("hx-trigger"),
                "song-added-to-setlist",
            )],
            Html("<p class=\"text-green-400\">Setlist criada e música adicionada.</p>".to_string()),
        )
            .into_response(),
        Err(error) => {
            tracing::error!(error = %error, "failed to attach new setlist to song");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(
                    "<p class=\"text-red-400\">Não foi possível adicionar a música.</p>"
                        .to_string(),
                ),
            )
                .into_response()
        }
    }
}

#[derive(Deserialize)]
struct AddSongForm {
    song_id: i32,
}

async fn add_song_to_setlist(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<i32>,
    jar: CookieJar,
    Form(form): Form<AddSongForm>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let Some(user_id) = authenticated_user_id(&jar) else {
        return Err((StatusCode::UNAUTHORIZED, "Sessão expirada".to_string()));
    };

    let owns_setlist = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM usersetlists WHERE \"IDUser\" = $1 AND \"IDSetlist\" = $2)",
    )
    .bind(user_id)
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap_or(false);
    if !owns_setlist {
        return Err((StatusCode::FORBIDDEN, "Setlist não encontrada".to_string()));
    }

    let res = sqlx::query("INSERT INTO setlistsongs (\"IDSetlist\", \"IDSong\", \"Order\") SELECT $1, $2, COALESCE(MAX(\"Order\"),0)+1 FROM setlistsongs WHERE \"IDSetlist\" = $1 ON CONFLICT (\"IDSetlist\", \"IDSong\") DO NOTHING")
        .bind(id)
        .bind(form.song_id)
        .execute(&pool)
        .await;

    match res {
        Ok(_) => Ok(Json(serde_json::json!({ "ok": true }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("db error: {}", e),
        )),
    }
}

async fn add_song_to_setlist_htmx(
    Extension(pool): Extension<PgPool>,
    Path((id, song_id)): Path<(i32, i32)>,
    jar: CookieJar,
) -> impl IntoResponse {
    let Some(user_id) = authenticated_user_id(&jar) else {
        return (
            StatusCode::UNAUTHORIZED,
            Html("<p class=\"text-red-400\">Sessão expirada.</p>".to_string()),
        )
            .into_response();
    };
    let owns_setlist = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM usersetlists WHERE \"IDUser\" = $1 AND \"IDSetlist\" = $2)",
    )
    .bind(user_id)
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap_or(false);
    if !owns_setlist {
        return (
            StatusCode::FORBIDDEN,
            Html("<p class=\"text-red-400\">Setlist não encontrada.</p>".to_string()),
        )
            .into_response();
    }
    let result = sqlx::query("INSERT INTO setlistsongs (\"IDSetlist\", \"IDSong\", \"Order\") SELECT $1, $2, COALESCE(MAX(\"Order\"), 0) + 1 FROM setlistsongs WHERE \"IDSetlist\" = $1 ON CONFLICT (\"IDSetlist\", \"IDSong\") DO NOTHING")
        .bind(id)
        .bind(song_id)
        .execute(&pool)
        .await;
    match result {
        Ok(result) if result.rows_affected() == 0 => (
            [(
                header::HeaderName::from_static("hx-trigger"),
                "song-added-to-setlist",
            )],
            Html("<p class=\"text-yellow-300\">A música já está nesta setlist.</p>".to_string()),
        )
            .into_response(),
        Ok(_) => (
            [(
                header::HeaderName::from_static("hx-trigger"),
                "song-added-to-setlist",
            )],
            Html("<p class=\"text-green-400\">Música adicionada à setlist.</p>".to_string()),
        )
            .into_response(),
        Err(error) => {
            tracing::error!(error = %error, "failed to add song to setlist");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(
                    "<p class=\"text-red-400\">Não foi possível adicionar a música.</p>"
                        .to_string(),
                ),
            )
                .into_response()
        }
    }
}

async fn remove_song_from_setlist_htmx(
    Extension(pool): Extension<PgPool>,
    Path((id, song_id)): Path<(i32, i32)>,
    jar: CookieJar,
) -> impl IntoResponse {
    let Some(user_id) = authenticated_user_id(&jar) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "ok": false, "error": "Sessão expirada" })),
        )
            .into_response();
    };
    let owns_setlist = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM usersetlists WHERE \"IDUser\" = $1 AND \"IDSetlist\" = $2)",
    )
    .bind(user_id)
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap_or(false);
    if !owns_setlist {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "ok": false, "error": "Setlist não encontrada" })),
        )
            .into_response();
    }

    let _ = sqlx::query("DELETE FROM setlistsongs WHERE \"IDSetlist\" = $1 AND \"IDSong\" = $2")
        .bind(id)
        .bind(song_id)
        .execute(&pool)
        .await;
    Json(serde_json::json!({ "ok": true })).into_response()
}

async fn move_setlist_song_htmx(
    Extension(pool): Extension<PgPool>,
    Path((id, song_id, direction)): Path<(i32, i32, String)>,
    jar: CookieJar,
) -> impl IntoResponse {
    let Some(user_id) = authenticated_user_id(&jar) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let owns_setlist = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM usersetlists WHERE \"IDUser\" = $1 AND \"IDSetlist\" = $2)",
    )
    .bind(user_id)
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap_or(false);
    if !owns_setlist {
        return StatusCode::FORBIDDEN.into_response();
    }
    let songs = sqlx::query_as::<_, (i32, Option<i32>)>("SELECT \"IDSong\", \"Order\" FROM setlistsongs WHERE \"IDSetlist\" = $1 ORDER BY \"Order\" ASC")
        .bind(id)
        .fetch_all(&pool)
        .await
        .unwrap_or_default();
    let Some(index) = songs
        .iter()
        .position(|(current_id, _)| *current_id == song_id)
    else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let adjacent_index = match direction.as_str() {
        "up" if index > 0 => index - 1,
        "down" if index + 1 < songs.len() => index + 1,
        _ => return StatusCode::BAD_REQUEST.into_response(),
    };
    let (adjacent_song_id, adjacent_order) = songs[adjacent_index];
    let current_order = songs[index].1.unwrap_or(index as i32);
    let adjacent_order = adjacent_order.unwrap_or(adjacent_index as i32);
    let result = sqlx::query("UPDATE setlistsongs SET \"Order\" = CASE WHEN \"IDSong\" = $2 THEN $3 WHEN \"IDSong\" = $4 THEN $5 END WHERE \"IDSetlist\" = $1 AND \"IDSong\" IN ($2, $4)")
        .bind(id)
        .bind(song_id)
        .bind(adjacent_order)
        .bind(adjacent_song_id)
        .bind(current_order)
        .execute(&pool)
        .await;
    if result.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    (
        [(
            header::HeaderName::from_static("hx-trigger"),
            "setlist-updated",
        )],
        Html(String::new()),
    )
        .into_response()
}

async fn remove_song_from_setlist(
    Extension(pool): Extension<PgPool>,
    Path((id, song_id)): Path<(i32, i32)>,
    jar: CookieJar,
) -> impl IntoResponse {
    let Some(user_id) = authenticated_user_id(&jar) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "ok": false, "error": "Sessão expirada" })),
        )
            .into_response();
    };
    let owns_setlist = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM usersetlists WHERE \"IDUser\" = $1 AND \"IDSetlist\" = $2)",
    )
    .bind(user_id)
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap_or(false);
    if !owns_setlist {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "ok": false, "error": "Setlist não encontrada" })),
        )
            .into_response();
    }

    let _ = sqlx::query("DELETE FROM setlistsongs WHERE \"IDSetlist\" = $1 AND \"IDSong\" = $2")
        .bind(id)
        .bind(song_id)
        .execute(&pool)
        .await;
    Json(serde_json::json!({ "ok": true })).into_response()
}

#[derive(serde::Serialize)]
struct CountResp {
    count: i64,
}

async fn increment_view(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<i32>,
    jar: CookieJar,
) -> impl IntoResponse {
    if !is_authenticated(&jar) {
        return (StatusCode::UNAUTHORIZED, Json(CountResp { count: 0 })).into_response();
    }

    let row = sqlx::query("UPDATE songs SET \"ViewCount\" = COALESCE(\"ViewCount\", 0) + 1 WHERE \"ID\" = $1 RETURNING \"ViewCount\"")
        .bind(id)
        .fetch_one(&pool)
        .await
        .ok();

    let count: i64 = row
        .and_then(|r| r.try_get::<Option<i32>, _>("ViewCount").ok())
        .flatten()
        .unwrap_or(0) as i64;
    Json(CountResp { count }).into_response()
}

#[derive(serde::Serialize)]
struct FavoriteResp {
    favorite: bool,
}

async fn favorite_song_ids(pool: &PgPool, user_id: Option<i32>) -> HashSet<i32> {
    let Some(uid) = user_id else {
        return HashSet::new();
    };
    sqlx::query_scalar::<_, i32>("SELECT \"songId\" FROM \"UserFavoriteSongs\" WHERE \"userId\" = $1")
        .bind(uid)
        .fetch_all(pool)
        .await
        .unwrap_or_default()
        .into_iter()
        .collect()
}

async fn toggle_favorite(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<i32>,
    jar: CookieJar,
) -> impl IntoResponse {
    let Some(user_id) = authenticated_user_id(&jar) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(FavoriteResp { favorite: false }),
        )
            .into_response();
    };

    let exists = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM \"UserFavoriteSongs\" WHERE \"userId\" = $1 AND \"songId\" = $2)")
        .bind(user_id)
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap_or(false);
    let favorite = if exists {
        sqlx::query("DELETE FROM \"UserFavoriteSongs\" WHERE \"userId\" = $1 AND \"songId\" = $2")
            .bind(user_id)
            .bind(id)
            .execute(&pool)
            .await
            .map(|_| false)
            .unwrap_or(exists)
    } else {
        sqlx::query("INSERT INTO \"UserFavoriteSongs\" (\"userId\", \"songId\", \"favoritedAt\") VALUES ($1, $2, NOW()) ON CONFLICT (\"userId\", \"songId\") DO NOTHING")
            .bind(user_id)
            .bind(id)
            .execute(&pool)
            .await
            .map(|_| true)
            .unwrap_or(exists)
    };

    Json(FavoriteResp { favorite }).into_response()
}

async fn toggle_favorite_htmx(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<i32>,
    jar: CookieJar,
) -> impl IntoResponse {
    let Some(user_id) = authenticated_user_id(&jar) else {
        return Html("<span class=\"text-red-400\">Sessão expirada</span>".to_string())
            .into_response();
    };

    let exists = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM \"UserFavoriteSongs\" WHERE \"userId\" = $1 AND \"songId\" = $2)")
        .bind(user_id)
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap_or(false);
    let favorite = if exists {
        sqlx::query("DELETE FROM \"UserFavoriteSongs\" WHERE \"userId\" = $1 AND \"songId\" = $2")
            .bind(user_id)
            .bind(id)
            .execute(&pool)
            .await
            .map(|_| false)
            .unwrap_or(exists)
    } else {
        sqlx::query("INSERT INTO \"UserFavoriteSongs\" (\"userId\", \"songId\", \"favoritedAt\") VALUES ($1, $2, NOW()) ON CONFLICT (\"userId\", \"songId\") DO NOTHING")
            .bind(user_id)
            .bind(id)
            .execute(&pool)
            .await
            .map(|_| true)
            .unwrap_or(exists)
    };

    let btn = if favorite {
        format!("<button hx-post=\"/htmx/songs/{id}/favorite\" hx-swap=\"outerHTML\" class=\"px-2 text-yellow-400\">★</button>")
    } else {
        format!("<button hx-post=\"/htmx/songs/{id}/favorite\" hx-swap=\"outerHTML\" class=\"px-2 text-gray-300\">☆</button>")
    };

    Html(btn).into_response()
}

async fn song_detail_htmx(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<i32>,
    headers: HeaderMap,
    jar: CookieJar,
) -> impl IntoResponse {
    if !is_authenticated(&jar) {
        return Html(
            "<div class=\"p-4 text-red-400\">Sessão expirada. Faça login novamente.</div>"
                .to_string(),
        )
        .into_response();
    }

    let row = sqlx::query_as::<_, Song>(
        "SELECT \"ID\", \"Code\", \"Name\", \"OrgName\", \"Composer\", \"ChordPro\", \"Lyrics\", \"Themes\", \"Youtube\", \"GenreType\", \"Artistas\", \"OrgKey\", \"OrgTempo\", \"Copyright\", \"Favorite\", \"IdInsertUser\", \"IdUpdateUser\", \"ViewCount\", \"createdAt\", \"updatedAt\", \"Status\" FROM songs WHERE \"ID\" = $1",
    )
        .bind(id)
        .fetch_optional(&pool)
        .await
        .unwrap_or(None);

    if let Some(song) = row {
        let user_id = authenticated_user_id(&jar).expect("authenticated user must have an id");
        let fav_ids = favorite_song_ids(&pool, Some(user_id)).await;
        let is_pending = song.status.as_deref() == Some("pending");
        let can_approve = is_moderator_user(&pool, user_id).await;
        if is_pending && song.id_insert_user != Some(user_id) && !can_approve {
            return Html("<div class=\"p-4 text-red-600\">Música não encontrada</div>".to_string())
                .into_response();
        }
        let settings = sqlx::query_as::<_, ChordSettingsRow>(
            "SELECT \"fontSize\", \"columnCount\", \"accidentals\", \"transpose\", \"hideChords\" FROM \"UserChordSettings\" WHERE \"userId\" = $1 AND \"songId\" = $2 AND \"deviceHash\" = $3",
        )
        .bind(user_id)
        .bind(id)
        .bind(device_hash(&headers))
        .fetch_optional(&pool)
        .await
        .ok()
        .flatten();
        let settings = settings.unwrap_or(ChordSettingsRow {
            font_size: 16.0,
            column_count: 0,
            accidentals: 0,
            transpose: 0,
            hide_chords: false,
        });
        let view = SongDetailView {
            id: song.id,
            name: song.name.unwrap_or_else(|| "(sem título)".to_string()),
            code: song.code.unwrap_or_else(|| "-".to_string()),
            chord_pro: song.chord_pro,
            lyrics: song.lyrics.unwrap_or_default(),
            youtube: song.youtube.unwrap_or_default(),
            favorite: fav_ids.contains(&song.id),
            is_pending,
            can_edit: can_edit_song(&pool, user_id, song.id_insert_user).await,
            can_delete: can_delete_song(&pool, user_id, song.id_insert_user, &jar).await,
            can_approve,
            font_size: settings.font_size,
            column_count: settings.column_count,
            accidentals: settings.accidentals,
            transpose: settings.transpose,
            hide_chords: settings.hide_chords,
        };

        let tpl = SongDetailTemplate {
            song: view,
            navigation: None,
            is_public: false,
        };
        let rendered = tpl
            .render()
            .unwrap_or_else(|e| format!("Erro ao renderizar template: {}", e));
        (
            [(header::HeaderName::from_static("cache-control"), "no-store")],
            Html(rendered),
        )
            .into_response()
    } else {
        Html("<div class=\"p-4 text-red-600\">Música não encontrada</div>".to_string())
            .into_response()
    }
}

async fn setlist_player_page(
    Extension(pool): Extension<PgPool>,
    Path((setlist_id, position)): Path<(i32, usize)>,
    headers: HeaderMap,
    jar: CookieJar,
) -> impl IntoResponse {
    let Some(user_id) = authenticated_user_id(&jar) else {
        return Redirect::to("/").into_response();
    };
    let owns_setlist = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM usersetlists WHERE \"IDUser\" = $1 AND \"IDSetlist\" = $2)",
    )
    .bind(user_id)
    .bind(setlist_id)
    .fetch_one(&pool)
    .await
    .unwrap_or(false);
    let fav_ids = favorite_song_ids(&pool, Some(user_id)).await;
    if !owns_setlist {
        return (
            StatusCode::NOT_FOUND,
            Html("Setlist não encontrada".to_string()),
        )
            .into_response();
    }

    let song_ids = sqlx::query_scalar::<_, i32>(
        "SELECT \"IDSong\" FROM setlistsongs WHERE \"IDSetlist\" = $1 ORDER BY \"Order\", \"IDSong\"",
    )
    .bind(setlist_id)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();
    if position == 0 || position > song_ids.len() {
        return (
            StatusCode::NOT_FOUND,
            Html("Música não encontrada nesta setlist".to_string()),
        )
            .into_response();
    }

    let song_id = song_ids[position - 1];
    let song = sqlx::query_as::<_, Song>(
        "SELECT \"ID\", \"Code\", \"Name\", \"OrgName\", \"Composer\", \"ChordPro\", \"Lyrics\", \"Themes\", \"Youtube\", \"GenreType\", \"Artistas\", \"OrgKey\", \"OrgTempo\", \"Copyright\", \"Favorite\", \"IdInsertUser\", \"IdUpdateUser\", \"ViewCount\", \"createdAt\", \"updatedAt\", \"Status\" FROM songs WHERE \"ID\" = $1",
    )
    .bind(song_id)
    .fetch_optional(&pool)
    .await
    .unwrap_or(None);
    let Some(song) = song else {
        return (
            StatusCode::NOT_FOUND,
            Html("Música não encontrada".to_string()),
        )
            .into_response();
    };
    let is_pending = song.status.as_deref() == Some("pending");
    let can_approve = is_moderator_user(&pool, user_id).await;
    if is_pending && song.id_insert_user != Some(user_id) && !can_approve {
        return (
            StatusCode::NOT_FOUND,
            Html("Música não encontrada".to_string()),
        )
            .into_response();
    }
    let settings = sqlx::query_as::<_, ChordSettingsRow>(
        "SELECT \"fontSize\", \"columnCount\", \"accidentals\", \"transpose\", \"hideChords\" FROM \"UserChordSettings\" WHERE \"userId\" = $1 AND \"songId\" = $2 AND \"deviceHash\" = $3",
    )
    .bind(user_id)
    .bind(song_id)
    .bind(device_hash(&headers))
    .fetch_optional(&pool)
    .await
    .ok()
    .flatten()
    .unwrap_or(ChordSettingsRow { font_size: 16.0, column_count: 0, accidentals: 0, transpose: 0, hide_chords: false });
    let song = SongDetailView {
        id: song.id,
        name: song.name.unwrap_or_else(|| "(sem título)".to_string()),
        code: song.code.unwrap_or_else(|| "-".to_string()),
        chord_pro: song.chord_pro,
        lyrics: song.lyrics.unwrap_or_default(),
        youtube: song.youtube.unwrap_or_default(),
        favorite: fav_ids.contains(&song.id),
        is_pending,
        can_edit: can_edit_song(&pool, user_id, song.id_insert_user).await,
        can_delete: can_delete_song(&pool, user_id, song.id_insert_user, &jar).await,
        can_approve,
        font_size: settings.font_size,
        column_count: settings.column_count,
        accidentals: settings.accidentals,
        transpose: settings.transpose,
        hide_chords: settings.hide_chords,
    };
    let navigation = SetlistNavigation {
        current_position: position,
        total_songs: song_ids.len(),
        previous_url: position
            .checked_sub(1)
            .filter(|previous| *previous > 0)
            .map(|previous| format!("/setlists/{setlist_id}/play/{previous}")),
        next_url: (position < song_ids.len())
            .then_some(format!("/setlists/{setlist_id}/play/{}", position + 1)),
    };
    let rendered = SetlistPlayerTemplate {
        song,
        navigation: Some(navigation),
        is_public: false,
    }
    .render()
    .unwrap_or_else(|error| format!("Erro ao renderizar template: {error}"));
    (
        [(header::HeaderName::from_static("cache-control"), "no-store")],
        Html(rendered),
    )
        .into_response()
}

async fn public_setlist_index_page(
    Extension(pool): Extension<PgPool>,
    Path(token): Path<String>,
) -> impl IntoResponse {
    let setlist_id = sqlx::query_scalar::<_, i32>(
        "SELECT setlist_id FROM setlist_share_links WHERE token = $1",
    )
    .bind(&token)
    .fetch_optional(&pool)
    .await
    .ok()
    .flatten();

    let Some(setlist_id) = setlist_id else {
        return (StatusCode::NOT_FOUND, Html("Link de partilha inválido ou removido.".to_string())).into_response();
    };

    let row = sqlx::query_as::<_, Setlist>(
        "SELECT \"ID\", \"Data\", \"Desc\", \"IDGrupo\" FROM setlists WHERE \"ID\" = $1",
    )
    .bind(setlist_id)
    .fetch_optional(&pool)
    .await
    .unwrap_or(None);

    let Some(s) = row else {
        return (StatusCode::NOT_FOUND, Html("Setlist não encontrada.".to_string())).into_response();
    };

    let rows = sqlx::query_as::<_, (i32, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>)>(
        "SELECT songs.\"ID\", songs.\"Name\", songs.\"OrgName\", songs.\"Composer\", songs.\"Artistas\", songs.\"Youtube\" FROM setlistsongs JOIN songs ON songs.\"ID\" = setlistsongs.\"IDSong\" WHERE setlistsongs.\"IDSetlist\" = $1 ORDER BY setlistsongs.\"Order\""
    )
    .bind(setlist_id)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let songs_view: Vec<SongInSetlist> = rows
        .into_iter()
        .map(
            |(id, name, org_name, composer, artistas, youtube)| SongInSetlist {
                id,
                name: name.unwrap_or_default(),
                org_name: org_name.unwrap_or_default(),
                composer: composer.unwrap_or_default(),
                artistas: artistas.unwrap_or_default(),
                youtube: youtube.filter(|url| !url.trim().is_empty()),
            },
        )
        .collect();

    let setlist_item = SetlistItem {
        id: s.id,
        data: s.data.map(|d| d.to_string()).unwrap_or_default(),
        desc: s.desc.unwrap_or_default(),
        song_count: songs_view.len() as i64,
    };

    let rendered = PublicSetlistTemplate {
        setlist: setlist_item,
        songs: songs_view,
        token,
    }
    .render()
    .unwrap_or_else(|e| format!("Erro ao renderizar template: {}", e));

    Html(rendered).into_response()
}

async fn public_setlist_player_page(
    Extension(pool): Extension<PgPool>,
    Path((token, position)): Path<(String, usize)>,
) -> impl IntoResponse {
    let setlist_id = sqlx::query_scalar::<_, i32>(
        "SELECT setlist_id FROM setlist_share_links WHERE token = $1",
    )
    .bind(&token)
    .fetch_optional(&pool)
    .await
    .ok()
    .flatten();
    let Some(setlist_id) = setlist_id else {
        return (StatusCode::NOT_FOUND, Html("Link de partilha inválido ou removido.".to_string())).into_response();
    };
    let song_ids = sqlx::query_scalar::<_, i32>(
        "SELECT \"IDSong\" FROM setlistsongs WHERE \"IDSetlist\" = $1 ORDER BY \"Order\", \"IDSong\"",
    )
    .bind(setlist_id)
    .fetch_all(&pool)
    .await
    .unwrap_or_default();
    if position == 0 || position > song_ids.len() {
        return (StatusCode::NOT_FOUND, Html("Música não encontrada nesta setlist.".to_string())).into_response();
    }
    let song = sqlx::query_as::<_, Song>(
        "SELECT \"ID\", \"Code\", \"Name\", \"OrgName\", \"Composer\", \"ChordPro\", \"Lyrics\", \"Themes\", \"Youtube\", \"GenreType\", \"Artistas\", \"OrgKey\", \"OrgTempo\", \"Copyright\", \"Favorite\", \"IdInsertUser\", \"IdUpdateUser\", \"ViewCount\", \"createdAt\", \"updatedAt\", \"Status\" FROM songs WHERE \"ID\" = $1",
    )
    .bind(song_ids[position - 1])
    .fetch_optional(&pool)
    .await
    .unwrap_or(None);
    let Some(song) = song else {
        return (StatusCode::NOT_FOUND, Html("Música não encontrada.".to_string())).into_response();
    };
    let song = SongDetailView {
        id: song.id,
        name: song.name.unwrap_or_else(|| "(sem título)".to_string()),
        code: song.code.unwrap_or_else(|| "-".to_string()),
        chord_pro: song.chord_pro,
        lyrics: song.lyrics.unwrap_or_default(),
        youtube: song.youtube.unwrap_or_default(),
        favorite: false,
        is_pending: false,
        can_edit: false,
        can_delete: false,
        can_approve: false,
        font_size: 16.0,
        column_count: 0,
        accidentals: 0,
        transpose: 0,
        hide_chords: false,
    };
    let navigation = SetlistNavigation {
        current_position: position,
        total_songs: song_ids.len(),
        previous_url: position
            .checked_sub(1)
            .filter(|previous| *previous > 0)
            .map(|previous| format!("/share/{token}/{previous}")),
        next_url: (position < song_ids.len()).then_some(format!("/share/{token}/{}", position + 1)),
    };
    let rendered = SetlistPlayerTemplate { song, navigation: Some(navigation), is_public: true }
        .render()
        .unwrap_or_else(|error| format!("Erro ao renderizar template: {error}"));
    ([(header::HeaderName::from_static("cache-control"), "no-store")], Html(rendered)).into_response()
}

async fn song_edit_htmx(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<i32>,
    jar: CookieJar,
) -> impl IntoResponse {
    if !is_authenticated(&jar) {
        return Html(
            "<div class=\"p-4 text-red-400\">Sessão expirada. Faça login novamente.</div>"
                .to_string(),
        )
        .into_response();
    }
    let song = sqlx::query_as::<_, Song>("SELECT \"ID\", \"Code\", \"Name\", \"OrgName\", \"Composer\", \"ChordPro\", \"Lyrics\", \"Themes\", \"Youtube\", \"GenreType\", \"Artistas\", \"OrgKey\", \"OrgTempo\", \"Copyright\", \"Favorite\", \"IdInsertUser\", \"IdUpdateUser\", \"ViewCount\", \"createdAt\", \"updatedAt\", \"Status\" FROM songs WHERE \"ID\" = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .unwrap_or(None);
    let Some(song) = song else {
        return Html("<div class=\"p-4 text-red-400\">Música não encontrada.</div>".to_string())
            .into_response();
    };
    // Apenas o criador (IdInsertUser) ou o programador pode abrir a edição
    if let Some(user_id) = authenticated_user_id(&jar) {
        if !can_edit_song(&pool, user_id, song.id_insert_user).await {
            return (
                StatusCode::FORBIDDEN,
                Html("<div class=\"p-4 text-red-400\">Apenas o criador desta música ou o programador pode editá-la.</div>".to_string()),
            )
                .into_response();
        }
    } else {
        return Html("<div class=\"p-4 text-red-400\">Sessão expirada. Faça login novamente.</div>".to_string())
            .into_response();
    }
    let selected_genre = song.genre_type;
    let genres = sqlx::query_as::<_, (i32, Option<String>, Option<String>)>(
        "SELECT s.\"GenreType\" AS id, COALESCE(g.\"Desc\", 'Tipo ' || s.\"GenreType\") AS name, g.\"CodeSufix\" AS prefix \
         FROM (SELECT DISTINCT \"GenreType\" FROM songs WHERE \"GenreType\" IS NOT NULL) s \
         LEFT JOIN genretypes g ON g.\"ID\" = s.\"GenreType\" ORDER BY id",
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|(id, name, prefix)| GenreOption {
        id,
        name: name.unwrap_or_else(|| format!("Tipo {id}")),
        prefix: prefix.unwrap_or_default(),
        selected: Some(id) == selected_genre,
    })
    .collect();
    let view = SongEditView {
        id: song.id,
        code: song.code.unwrap_or_default(),
        title: song
            .name
            .clone()
            .or(song.org_name.clone())
            .unwrap_or_else(|| "Sem título".to_string()),
        name: song.name.unwrap_or_default(),
        org_name: song.org_name.unwrap_or_default(),
        composer: song.composer.unwrap_or_default(),
        chord_pro: normalize_newlines(song.chord_pro),
        lyrics: normalize_newlines(song.lyrics.unwrap_or_default()),
        themes: song.themes.unwrap_or_default(),
        youtube: song.youtube.unwrap_or_default(),
        artistas: song.artistas.unwrap_or_default(),
        org_key: song.org_key.unwrap_or_default(),
        org_tempo: song
            .org_tempo
            .map(|value| value.to_string())
            .unwrap_or_default(),
        copyright: song.copyright.unwrap_or_default(),
    };
    Html(
        SongEditTemplate { song: view, genres }
            .render()
            .unwrap_or_default(),
    )
    .into_response()
}

#[derive(Deserialize)]
struct SongEditForm {
    code: String,
    name: String,
    org_name: String,
    composer: String,
    chord_pro: String,
    lyrics: String,
    themes: String,
    youtube: String,
    genre_type: Option<String>,
    artistas: String,
    org_key: String,
    org_tempo: Option<String>,
    copyright: String,
}

async fn update_song_htmx(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<i32>,
    jar: CookieJar,
    Form(form): Form<SongEditForm>,
) -> impl IntoResponse {
    let Some(user_id) = authenticated_user_id(&jar) else {
        return (
            StatusCode::UNAUTHORIZED,
            Html("<p class=\"text-red-400\">Sessão expirada.</p>".to_string()),
        )
            .into_response();
    };
    // Apenas o criador (IdInsertUser) ou o programador pode gravar alterações
    let owner: Option<i32> = sqlx::query_scalar("SELECT \"IdInsertUser\" FROM songs WHERE \"ID\" = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .ok()
        .flatten();
    if !can_edit_song(&pool, user_id, owner).await {
        return (
            StatusCode::FORBIDDEN,
            Html(
                "<p class=\"text-red-400\">Apenas o criador desta música ou o programador pode editá-la.</p>"
                    .to_string(),
            ),
        )
            .into_response();
    }
    let genre_type = form
        .genre_type
        .as_deref()
        .and_then(|value| value.trim().parse::<i32>().ok());
    let org_tempo = form
        .org_tempo
        .as_deref()
        .and_then(|value| value.trim().parse::<i32>().ok());

    let result = sqlx::query("UPDATE songs SET \"Code\" = $1, \"Name\" = $2, \"OrgName\" = $3, \"Composer\" = $4, \"ChordPro\" = $5, \"Lyrics\" = $6, \"Themes\" = $7, \"Youtube\" = $8, \"GenreType\" = $9, \"Artistas\" = $10, \"OrgKey\" = $11, \"OrgTempo\" = $12, \"Copyright\" = $13, \"IdUpdateUser\" = $14, \"updatedAt\" = NOW() WHERE \"ID\" = $15")
        .bind(form.code).bind(form.name).bind(form.org_name).bind(form.composer).bind(normalize_newlines(form.chord_pro)).bind(normalize_newlines(form.lyrics)).bind(form.themes).bind(normalize_youtube(form.youtube)).bind(genre_type).bind(form.artistas).bind(form.org_key).bind(org_tempo).bind(form.copyright).bind(user_id).bind(id)
        .execute(&pool)
        .await;
    match result {
        Ok(result) if result.rows_affected() == 0 => (
            StatusCode::NOT_FOUND,
            Html("<p class=\"text-red-400\">Música não encontrada.</p>".to_string()),
        )
            .into_response(),
        Ok(_) => (
            [(
                header::HeaderName::from_static("hx-trigger"),
                "song-updated",
            )],
            Html("<p class=\"text-green-400\">Alterações guardadas.</p>".to_string()),
        )
            .into_response(),
        Err(error) => {
            tracing::error!(error = %error, "song update failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(
                    "<p class=\"text-red-400\">Não foi possível guardar as alterações.</p>"
                        .to_string(),
                ),
            )
                .into_response()
        }
    }
}

#[derive(Template)]
#[template(path = "song_new.html")]
struct SongNewTemplate {
    genres: Vec<GenreOption>,
}

#[derive(Deserialize)]
struct SongCreateForm {
    code: String,
    name: String,
    org_name: String,
    composer: String,
    chord_pro: String,
    lyrics: String,
    themes: String,
    youtube: String,
    genre_type: Option<String>,
    artistas: String,
    org_key: String,
    org_tempo: Option<String>,
    copyright: String,
}

fn empty_to_none(value: String) -> Option<String> {
    let trimmed = value.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

/// Compute the next sequential "Code" for a given prefix (e.g. "CO0001", "CO0002", ...).
async fn next_code_for_prefix(pool: &PgPool, prefix: &str) -> String {
    let pattern = format!("{}%", prefix);
    let result = sqlx::query_as::<_, (Option<String>,)>(
        "SELECT \"Code\" FROM songs WHERE \"Code\" LIKE $1 ORDER BY \"Code\" DESC LIMIT 1",
    )
    .bind(&pattern)
    .fetch_optional(pool)
    .await;
    let next_num = match result {
        Ok(Some((Some(code),))) => {
            // Extract the numeric suffix from the code
            let numeric_part: String = code.chars().skip(prefix.len()).collect();
            numeric_part
                .parse::<i32>()
                .ok()
                .map(|n| n + 1)
                .unwrap_or(1)
        }
        _ => 1,
    };
    format!("{}{:04}", prefix, next_num)
}

async fn next_code_htmx(
    Extension(pool): Extension<PgPool>,
    jar: CookieJar,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    if !is_authenticated(&jar) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Sessão expirada."})),
        )
            .into_response();
    }
    let prefix = params.get("prefix").map(|s| s.as_str()).unwrap_or("");
    if prefix.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Prefixo não informado."})),
        )
            .into_response();
    }
    let new_code = next_code_for_prefix(&pool, prefix).await;
    (StatusCode::OK, Json(serde_json::json!({"code": new_code}))).into_response()
}

async fn song_new_htmx(Extension(pool): Extension<PgPool>, jar: CookieJar) -> impl IntoResponse {
    let Some(user_id) = authenticated_user_id(&jar) else {
        return Html(
            "<div class=\"p-4 text-red-400\">Sessão expirada. Faça login novamente.</div>"
                .to_string(),
        )
        .into_response();
    };
    if !can_submit_song_user(&pool, user_id).await {
        return Html(
            "<div class=\"p-4 text-red-400\">Apenas colaboradores, moderadores ou programadores podem criar músicas.</div>"
                .to_string(),
        )
        .into_response();
    }
    let genres = sqlx::query_as::<_, (i32, Option<String>, Option<String>)>(
        "SELECT s.\"GenreType\" AS id, COALESCE(g.\"Desc\", 'Tipo ' || s.\"GenreType\") AS name, g.\"CodeSufix\" AS prefix \
         FROM (SELECT DISTINCT \"GenreType\" FROM songs WHERE \"GenreType\" IS NOT NULL) s \
         LEFT JOIN genretypes g ON g.\"ID\" = s.\"GenreType\" ORDER BY id",
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|(id, name, prefix)| GenreOption {
        id,
        name: name.unwrap_or_else(|| format!("Tipo {id}")),
        prefix: prefix.unwrap_or_default(),
        selected: false,
    })
    .collect();
    Html(SongNewTemplate { genres }.render().unwrap_or_default()).into_response()
}

async fn create_song_htmx(
    Extension(pool): Extension<PgPool>,
    jar: CookieJar,
    Form(form): Form<SongCreateForm>,
) -> impl IntoResponse {
    let Some(user_id) = authenticated_user_id(&jar) else {
        return (
            StatusCode::UNAUTHORIZED,
            Html("<p class=\"text-red-400\">Sessão expirada.</p>".to_string()),
        )
            .into_response();
    };
    if !can_submit_song_user(&pool, user_id).await {
        return (
            StatusCode::FORBIDDEN,
            Html("<p class=\"text-red-400\">Apenas colaboradores, moderadores ou programadores podem criar músicas.</p>".to_string()),
        )
            .into_response();
    }
    let genre_type = form
        .genre_type
        .as_deref()
        .and_then(|value| value.trim().parse::<i32>().ok());
    let org_tempo = form
        .org_tempo
        .as_deref()
        .and_then(|value| value.trim().parse::<i32>().ok());

    // "Code" is NOT NULL: generate one automatically if the client didn't send it
    // (e.g. the user submitted the form without selecting a género).
    let code = match empty_to_none(form.code) {
        Some(code) => code,
        None => {
            let prefix = match genre_type {
                Some(gt) => {
                    sqlx::query_scalar::<_, Option<String>>(
                        "SELECT \"CodeSufix\" FROM genretypes WHERE \"ID\" = $1",
                    )
                    .bind(gt)
                    .fetch_optional(&pool)
                    .await
                    .ok()
                    .flatten()
                    .flatten()
                    .unwrap_or_else(|| "SUB".to_string())
                }
                None => "SUB".to_string(),
            };
            next_code_for_prefix(&pool, &prefix).await
        }
    };

    let result = sqlx::query(
        "INSERT INTO songs (\"Code\", \"Name\", \"OrgName\", \"Composer\", \"ChordPro\", \"Lyrics\", \"Themes\", \"Youtube\", \"GenreType\", \"Artistas\", \"OrgKey\", \"OrgTempo\", \"Copyright\", \"Favorite\", \"IdInsertUser\", \"ViewCount\", \"Status\", \"createdAt\", \"updatedAt\") VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, NOW(), NOW())",
    )
    .bind(code)
    .bind(empty_to_none(form.name))
    .bind(empty_to_none(form.org_name))
    .bind(empty_to_none(form.composer))
    .bind(normalize_newlines(form.chord_pro))
    .bind(normalize_newlines(form.lyrics))
    .bind(empty_to_none(form.themes))
    .bind(empty_to_none(normalize_youtube(form.youtube)))
    .bind(genre_type)
    .bind(empty_to_none(form.artistas))
    .bind(empty_to_none(form.org_key))
    .bind(org_tempo)
    .bind(empty_to_none(form.copyright))
    .bind(false)
    .bind(user_id)
    .bind(0)
    .bind(if is_moderator_user(&pool, user_id).await { "approved" } else { "pending" })
    .execute(&pool)
    .await;
    match result {
        Ok(_) => (
            [(
                header::HeaderName::from_static("hx-trigger"),
                "song-created",
            )],
            Html("<p class=\"text-green-400\">Música criada com sucesso!</p>".to_string()),
        )
            .into_response(),
        Err(error) => {
            tracing::error!(error = %error, "song create failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("<p class=\"text-red-400\">Não foi possível criar a música.</p>".to_string()),
            )
                .into_response()
        }
    }
}

/// Delete a song. Only admin (perfil >= 3) can delete their own songs, or programmer (perfil 4) can delete any.
async fn delete_song_htmx(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<i32>,
    jar: CookieJar,
) -> impl IntoResponse {
    let Some(user_id) = authenticated_user_id(&jar) else {
        return (
            StatusCode::UNAUTHORIZED,
            Html("<p class=\"text-red-400\">Sessão expirada.</p>".to_string()),
        )
            .into_response();
    };

    // Check if user is moderator, programmer or admin
    let is_moderator = is_moderator_user(&pool, user_id).await;
    let is_admin = is_admin_user(&pool, &jar).await;

    // Normal users cannot delete songs
    if !is_moderator && !is_admin {
        return (
            StatusCode::FORBIDDEN,
            Html(
                "<p class=\"text-red-400\">Apenas colaboradores podem apagar músicas.</p>"
                    .to_string(),
            ),
        )
            .into_response();
    }

    // Moderator/programmer can delete any song, colaborador can only delete their own
    if !is_moderator {
        let owner: Option<i32> =
            sqlx::query_scalar("SELECT \"IdInsertUser\" FROM songs WHERE \"ID\" = $1")
                .bind(id)
                .fetch_optional(&pool)
                .await
                .ok()
                .flatten();

        if owner != Some(user_id) {
            return (
                StatusCode::FORBIDDEN,
                Html(
                    "<p class=\"text-red-400\">Só pode apagar as suas próprias músicas.</p>"
                        .to_string(),
                ),
            )
                .into_response();
        }
    }

    // Delete related records first (FK constraints), then the song
    let _ = sqlx::query(
        r#"DELETE FROM "UserFavoriteSongs" WHERE "songId" = $1"#,
    )
    .bind(id)
    .execute(&pool)
    .await;

    let _ = sqlx::query(
        r#"DELETE FROM "UserChordSettings" WHERE "songId" = $1"#,
    )
    .bind(id)
    .execute(&pool)
    .await;

    let _ = sqlx::query(
        r#"DELETE FROM setlistsongs WHERE "IDSong" = $1"#,
    )
    .bind(id)
    .execute(&pool)
    .await;

    match sqlx::query(r#"DELETE FROM songs WHERE "ID" = $1"#)
        .bind(id)
        .execute(&pool)
        .await
    {
        Ok(_) => (
            [(
                header::HeaderName::from_static("hx-trigger"),
                "song-deleted",
            )],
            Html("<p class=\"text-green-400\">Música apagada com sucesso!</p>".to_string()),
        )
            .into_response(),
        Err(error) => {
            tracing::error!(error = %error, "song delete failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("<p class=\"text-red-400\">Não foi possível apagar a música.</p>".to_string()),
            )
                .into_response()
        }
    }
}

/// Approve a pending song. Only moderators (perfil 2) or programmers (perfil 4) can approve.
async fn approve_song_htmx(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<i32>,
    jar: CookieJar,
) -> impl IntoResponse {
    let Some(user_id) = authenticated_user_id(&jar) else {
        return (
            StatusCode::UNAUTHORIZED,
            Html("<p class=\"text-red-400\">Sessão expirada.</p>".to_string()),
        )
            .into_response();
    };

    // Only moderators or programmers can approve songs
    if !can_moderate_song(&pool, Some(user_id)).await {
        return (
            StatusCode::FORBIDDEN,
            Html("<p class=\"text-red-400\">Apenas moderadores podem aprovar músicas.</p>".to_string()),
        )
            .into_response();
    }

    match sqlx::query(r#"UPDATE songs SET "Status" = 'approved' WHERE "ID" = $1"#)
        .bind(id)
        .execute(&pool)
        .await
    {
        Ok(result) if result.rows_affected() == 0 => (
            StatusCode::NOT_FOUND,
            Html("<p class=\"text-red-400\">Música não encontrada.</p>".to_string()),
        )
            .into_response(),
        Ok(_) => (
            [(
                header::HeaderName::from_static("hx-trigger"),
                "song-approved",
            )],
            Html("<p class=\"text-green-400\">Música aprovada com sucesso!</p>".to_string()),
        )
            .into_response(),
        Err(error) => {
            tracing::error!(error = %error, "song approve failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("<p class=\"text-red-400\">Não foi possível aprovar a música.</p>".to_string()),
            )
                .into_response()
        }
    }
}

fn device_hash(headers: &HeaderMap) -> String {
    let ua = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let mut hasher = Sha256::new();
    hasher.update(ua);
    format!("{:x}", hasher.finalize())
}

/// Create a Stripe Checkout Session for donations
async fn create_stripe_checkout(
    Extension(_pool): Extension<PgPool>,
    Query(params): Query<HashMap<String, String>>,
    _jar: CookieJar,
) -> impl IntoResponse {
    let amount = params
        .get("amount")
        .and_then(|a| a.parse::<i64>().ok())
        .unwrap_or(500); // Default 5€ in cents

    // Minimum amount: 1€ (100 cents)
    if amount < 100 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Valor mínimo: 1€"})),
        )
            .into_response();
    }

    let stripe_secret = std::env::var("STRIPE_SECRET_KEY").unwrap_or_default();
    if stripe_secret.is_empty() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Stripe não configurado"})),
        )
            .into_response();
    }

    let app_url = std::env::var("APP_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());

    // Create Stripe Checkout Session via API
    let client = reqwest::Client::new();
    let form = [
        ("payment_method_types[]", "card"),
        ("line_items[0][price_data][currency]", "eur"),
        (
            "line_items[0][price_data][product_data][name]",
            "Donativo PraiseChords",
        ),
        (
            "line_items[0][price_data][product_data][description]",
            "Apoio ao projeto PraiseChords",
        ),
        (
            "line_items[0][price_data][unit_amount]",
            &amount.to_string(),
        ),
        ("line_items[0][quantity]", "1"),
        ("mode", "payment"),
        (
            "success_url",
            &format!("{}/app?donation=success", app_url),
        ),
        (
            "cancel_url",
            &format!("{}/app?donation=cancel", app_url),
        ),
        ("locale", "pt"),
    ];

    let response = client
        .post("https://api.stripe.com/v1/checkout/sessions")
        .header("Authorization", format!("Bearer {}", stripe_secret))
        .form(&form)
        .send()
        .await;

    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                let body = resp.text().await.unwrap_or_default();
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                    if let Some(url) = json.get("url").and_then(|u| u.as_str()) {
                        return (
                            StatusCode::OK,
                            Json(serde_json::json!({"url": url})),
                        )
                            .into_response();
                    }
                }
                tracing::error!("Stripe API error: {}", body);
            } else {
                let error_text = resp.text().await.unwrap_or_default();
                tracing::error!("Stripe API error: {}", error_text);
            }
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Erro ao criar sessão de pagamento"})),
            )
                .into_response()
        }
        Err(error) => {
            tracing::error!(error = %error, "Stripe request failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Erro de comunicação com Stripe"})),
            )
                .into_response()
        }
    }
}

async fn get_chord_settings(
    Extension(pool): Extension<PgPool>,
    Path(song_id): Path<i32>,
    headers: HeaderMap,
    jar: CookieJar,
) -> impl IntoResponse {
    let Some(user_id) = authenticated_user_id(&jar) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Não autenticado"})),
        )
            .into_response();
    };
    let settings = sqlx::query_as::<_, ChordSettingsRow>(
        "SELECT \"fontSize\", \"columnCount\", \"accidentals\", \"transpose\", \"hideChords\" FROM \"UserChordSettings\" WHERE \"userId\" = $1 AND \"songId\" = $2 AND \"deviceHash\" = $3",
    )
    .bind(user_id)
    .bind(song_id)
    .bind(device_hash(&headers))
    .fetch_optional(&pool)
    .await;
    let body = match settings {
        Ok(Some(value)) => Json(serde_json::json!({"data": {"fontSize": value.font_size, "columnCount": value.column_count, "accidentals": value.accidentals, "transpose": value.transpose, "hideChords": value.hide_chords}})).into_response(),
        Ok(None) => Json(serde_json::json!({"data": {"fontSize": 16, "columnCount": 0, "accidentals": 0, "transpose": 0, "hideChords": false}})).into_response(),
        Err(error) => { tracing::error!(error = %error, "failed to load chord settings"); (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Não foi possível obter as preferências"}))).into_response() }
    };
    (
        [(header::HeaderName::from_static("cache-control"), "no-store")],
        body,
    )
        .into_response()
}

async fn save_chord_settings(
    Extension(pool): Extension<PgPool>,
    Path(song_id): Path<i32>,
    headers: HeaderMap,
    jar: CookieJar,
    Json(settings): Json<ChordSettingsInput>,
) -> impl IntoResponse {
    let Some(user_id) = authenticated_user_id(&jar) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Não autenticado"})),
        )
            .into_response();
    };
    if !(8.0..=32.0).contains(&settings.font_size)
        || !(0..=4).contains(&settings.column_count)
        || ![-1, 0, 1].contains(&settings.accidentals)
        || !(-11..=11).contains(&settings.transpose)
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Preferências inválidas"})),
        )
            .into_response();
    }
    let result = sqlx::query("INSERT INTO \"UserChordSettings\" (\"userId\", \"songId\", \"deviceHash\", \"fontSize\", \"columnCount\", \"accidentals\", \"transpose\", \"hideChords\", \"updatedAt\") VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW()) ON CONFLICT (\"userId\", \"songId\", \"deviceHash\") DO UPDATE SET \"fontSize\" = EXCLUDED.\"fontSize\", \"columnCount\" = EXCLUDED.\"columnCount\", \"accidentals\" = EXCLUDED.\"accidentals\", \"transpose\" = EXCLUDED.\"transpose\", \"hideChords\" = EXCLUDED.\"hideChords\", \"updatedAt\" = NOW()")
        .bind(user_id).bind(song_id).bind(device_hash(&headers)).bind(settings.font_size).bind(settings.column_count).bind(settings.accidentals).bind(settings.transpose).bind(settings.hide_chords)
        .execute(&pool)
        .await;
    match result {
        Ok(_) => Json(serde_json::json!({"ok": true})).into_response(),
        Err(error) => {
            tracing::error!(error = %error, "failed to save chord settings");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Não foi possível guardar as preferências"})),
            )
                .into_response()
        }
    }
}

/// Middleware: impede o browser de colocar páginas HTML em cache,
/// garantindo que alterações nos templates aparecem sempre após reload.
async fn no_cache_html(req: Request, next: Next) -> Response {
    let mut res = next.run(req).await;
    let is_html = res
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.starts_with("text/html"))
        .unwrap_or(false);
    if is_html {
        res.headers_mut().insert(
            header::CACHE_CONTROL,
            header::HeaderValue::from_static("no-store"),
        );
    }
    res
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,sqlx=warn")),
        )
        .init();

    // centralised DB connect (loads .env from parent if needed)
    let pool = db::connect().await;

    // Create unaccent extension for accent-insensitive search
    let _ = sqlx::query(
        r#"
        CREATE EXTENSION IF NOT EXISTS unaccent
        "#,
    )
    .execute(&pool)
    .await;

    // Auto-create user_activity table if it doesn't exist
    let _ = sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS user_activity (
            id SERIAL PRIMARY KEY,
            "userId" INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            "action" VARCHAR(50) NOT NULL DEFAULT 'login',
            "createdAt" TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(&pool)
    .await;

    // Add is_demo column to users table if it doesn't exist
    let _ = sqlx::query(
        r#"
        ALTER TABLE users ADD COLUMN IF NOT EXISTS is_demo BOOLEAN NOT NULL DEFAULT FALSE
        "#,
    )
    .execute(&pool)
    .await;

    // Add Status column to songs table if it doesn't exist (approval workflow)
    let _ = sqlx::query(
        r#"
        ALTER TABLE songs ADD COLUMN IF NOT EXISTS "Status" VARCHAR(20) NOT NULL DEFAULT 'approved'
        "#,
    )
    .execute(&pool)
    .await;

    // Add hideChords column to UserChordSettings if it doesn't exist (lyrics-only mode toggle)
    let _ = sqlx::query(
        r#"
        ALTER TABLE "UserChordSettings" ADD COLUMN IF NOT EXISTS "hideChords" BOOLEAN NOT NULL DEFAULT FALSE
        "#,
    )
    .execute(&pool)
    .await;

    // Ensure demo user exists (id=2 is reserved for demo)
    let demo_exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE is_demo = TRUE)")
        .fetch_one(&pool)
        .await
        .unwrap_or(false);
    if !demo_exists {
        let _ = sqlx::query(
            r#"
            INSERT INTO users (email, nome, password_hash, perfil, ativo, is_demo)
            VALUES ('demo@praisechords.local', 'Demo', crypt('demo123', gen_salt('bf')), 3, TRUE, TRUE)
            ON CONFLICT (email) DO UPDATE SET is_demo = TRUE, perfil = 3, nome = 'Demo', igreja = NULL
            "#,
        )
        .execute(&pool)
        .await;
    } else {
        // Corrigir utilizador demo existente: nome "Demo", manter perfil admin, sem igreja
        let _ = sqlx::query(
            r#"
            UPDATE users SET nome = 'Demo', igreja = NULL, perfil = 3 WHERE is_demo = TRUE
            "#,
        )
        .execute(&pool)
        .await;
    }

    let _ = sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_user_activity_created_at ON user_activity ("createdAt")
        "#,
    )
    .execute(&pool)
    .await;

    let _ = sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_user_activity_user_id ON user_activity ("userId")
        "#,
    )
    .execute(&pool)
    .await;

    let _ = sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS password_reset_tokens (
            id SERIAL PRIMARY KEY,
            user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            token_hash VARCHAR(64) NOT NULL UNIQUE,
            expires_at TIMESTAMPTZ NOT NULL,
            used_at TIMESTAMPTZ,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(&pool)
    .await;

    let _ = sqlx::query(
        r#"CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_lookup ON password_reset_tokens (token_hash, expires_at) WHERE used_at IS NULL"#,
    )
    .execute(&pool)
    .await;

    // Auto-create setlist_share_links table if it doesn't exist (public share links)
    let _ = sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS setlist_share_links (
            id SERIAL PRIMARY KEY,
            setlist_id INTEGER NOT NULL UNIQUE REFERENCES setlists("ID") ON DELETE CASCADE,
            token VARCHAR(64) NOT NULL UNIQUE,
            created_by INTEGER REFERENCES users(id) ON DELETE SET NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(&pool)
    .await;

    // Auto-create genretypes table (tipos de música) and seed it if needed
    let _ = sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS genretypes (
            "ID" INTEGER PRIMARY KEY,
            "Desc" VARCHAR(50),
            "CodeSufix" VARCHAR(10)
        )
        "#,
    )
    .execute(&pool)
    .await;

    let _ = sqlx::query(
        r#"
        INSERT INTO genretypes ("ID", "Desc", "CodeSufix") VALUES
            (1, 'Coros', 'W'),
            (2, 'EBD', 'ED'),
            (3, 'Hinos Harpa', 'HM'),
            (4, 'Outros Arranjos', 'OH')
        ON CONFLICT ("ID") DO NOTHING
        "#,
    )
    .execute(&pool)
    .await;

    // Migrate Coros codes to Wxxxx format (ordered alphabetically by name)
    let _ = sqlx::query(
        r#"
        WITH numbered AS (
            SELECT "ID", ROW_NUMBER() OVER (ORDER BY COALESCE("Name", '') ASC, "ID" ASC) AS rn
            FROM songs
            WHERE "GenreType" = 1
        )
        UPDATE songs
        SET "Code" = 'W' || LPAD(numbered.rn::text, 4, '0')
        FROM numbered
        WHERE songs."ID" = numbered."ID"
        "#,
    )
    .execute(&pool)
    .await;

    // Migrate EBD codes to EDxxxx format (ordered alphabetically by name)
    let _ = sqlx::query(
        r#"
        WITH numbered AS (
            SELECT "ID", ROW_NUMBER() OVER (ORDER BY COALESCE("Name", '') ASC, "ID" ASC) AS rn
            FROM songs
            WHERE "GenreType" = 2
        )
        UPDATE songs
        SET "Code" = 'ED' || LPAD(numbered.rn::text, 4, '0')
        FROM numbered
        WHERE songs."ID" = numbered."ID"
        "#,
    )
    .execute(&pool)
    .await;

    // Migrate Hinos Harpa codes to HMxxxx format (ordered alphabetically by name)
    let _ = sqlx::query(
        r#"
        WITH numbered AS (
            SELECT "ID", ROW_NUMBER() OVER (ORDER BY COALESCE("Name", '') ASC, "ID" ASC) AS rn
            FROM songs
            WHERE "GenreType" = 3
        )
        UPDATE songs
        SET "Code" = 'HM' || LPAD(numbered.rn::text, 4, '0')
        FROM numbered
        WHERE songs."ID" = numbered."ID"
        "#,
    )
    .execute(&pool)
    .await;

    // Migrate Outros Arranjos codes to OHxxxx format (ordered alphabetically by name)
    let _ = sqlx::query(
        r#"
        WITH numbered AS (
            SELECT "ID", ROW_NUMBER() OVER (ORDER BY COALESCE("Name", '') ASC, "ID" ASC) AS rn
            FROM songs
            WHERE "GenreType" = 4
        )
        UPDATE songs
        SET "Code" = 'OH' || LPAD(numbered.rn::text, 4, '0')
        FROM numbered
        WHERE songs."ID" = numbered."ID"
        "#,
    )
    .execute(&pool)
    .await;

    // A coluna legada songs."Favorite" deixou de ser usada: as favoritas passaram
    // a ser por utilizador (tabela "UserFavoriteSongs"). Os valores antigos
    // faziam a estrela aparecer acesa sem a música constar nos favoritos
    // (ex.: W0554). Limpa-os para o estado mostrar apenas as favoritas reais.
    let _ = sqlx::query(r#"UPDATE songs SET "Favorite" = FALSE WHERE "Favorite" IS NOT FALSE"#)
        .execute(&pool)
        .await;

    let static_dir: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/static");
    let static_files = get_service(ServeDir::new(static_dir));

    let app = Router::new()
        .route("/", get(login_page))
        .route("/login", get(login_page))
        .route("/register", get(register_page))
        .route("/forgot-password", get(forgot_password_page))
        .route("/reset-password", get(reset_password_page))
        .route("/app", get(index))
        .route("/users/manage", get(users_manage_page))
        .route("/statistics", get(statistics_page))
        .route("/setlists/:id/play/:position", get(setlist_player_page))
        .route("/share/:token", get(public_setlist_index_page))
        .route("/share/:token/:position", get(public_setlist_player_page))
        .route("/favorites", get(favorites_page))
        .route("/most-viewed", get(most_viewed_page))
        .route("/recent", get(recent_page))
        .route("/pending", get(pending_page))
        .route("/logout", post(logout))
        .route("/api/login", post(login_submit))
        .route("/api/users/register", post(register_submit))
        .route("/api/users/forgot-password", post(forgot_password_submit))
        .route("/api/users/reset-password", post(reset_password_submit))
        // HTMX endpoints
        .route("/htmx/users/create", post(create_user_htmx))
        .route("/htmx/users/list", get(list_users_htmx))
        .route("/htmx/users/:id/edit", get(edit_user_form_htmx))
        .route("/htmx/users/:id/update", post(update_user_htmx))
        .route(
            "/htmx/users/:id/password",
            get(user_password_form_htmx).post(change_user_password_htmx),
        )
        .route("/htmx/users/:id/toggle", post(toggle_user_active_htmx))
        .route("/htmx/users/:id/delete", post(delete_user_htmx))
        .route("/htmx/admin/cleanup-demo", post(cleanup_demo_htmx))
        .route("/htmx/songs", get(list_songs_htmx))
        .route("/htmx/songs/new", get(song_new_htmx).post(create_song_htmx))
        .route("/htmx/songs/next-code", get(next_code_htmx))
        .route("/htmx/songs/:id", get(song_detail_htmx))
        .route(
            "/htmx/songs/:id/edit",
            get(song_edit_htmx).post(update_song_htmx),
        )
        .route("/htmx/songs/:id/setlists", get(setlist_song_picker_htmx))
        .route(
            "/htmx/songs/:id/setlists/new",
            get(new_setlist_for_song_form),
        )
        .route("/htmx/songs/:id/delete", delete(delete_song_htmx))
        .route("/htmx/songs/:id/approve", post(approve_song_htmx))
        .route("/htmx/setlists", get(list_setlists_htmx))
        .route("/htmx/setlists/new", get(new_setlist_form))
        .route("/api/setlists", post(create_setlist))
        .route("/htmx/songs/:id/setlists", post(create_setlist_for_song))
        .route("/htmx/setlists/:id", get(setlist_detail_htmx))
        .route("/htmx/setlists/:id/delete", delete(delete_setlist_htmx))
        .route("/htmx/setlists/:id/share", post(create_setlist_share_link_htmx))
        .route(
            "/htmx/setlists/:id/songs/:song_id/:direction",
            post(move_setlist_song_htmx),
        )
        // simple JSON API for compatibility
        .route("/api/songs", get(list_songs_json))
        .route("/api/songs/search", get(search_songs))
        .route("/htmx/songs/search", get(search_songs_htmx))
        .route("/htmx/songs/:id/favorite", post(toggle_favorite_htmx))
        .route(
            "/htmx/setlists/:id/songs/:song_id",
            post(add_song_to_setlist_htmx),
        )
        .route(
            "/htmx/setlists/:id/songs/:song_id/remove",
            delete(remove_song_from_setlist_htmx),
        )
        .route("/api/songs/:id/view", post(increment_view))
        .route("/api/songs/:id/favorite", post(toggle_favorite))
        .route(
            "/api/chords/:id/settings",
            get(get_chord_settings).put(save_chord_settings),
        )
        .route("/api/setlists/:id/songs", post(add_song_to_setlist))
        .route(
            "/api/setlists/:id/songs/:song_id",
            delete(remove_song_from_setlist),
        )
        // Stripe donation endpoint
        .route("/htmx/donate/stripe", post(create_stripe_checkout))
        .nest_service("/static", static_files)
        .layer(middleware::from_fn(no_cache_html))
        .layer(Extension(pool));

    // Escuta em 0.0.0.0 para permitir acesso na rede local (ex.: via telemóvel na mesma Wi-Fi).
    // A porta pode ser configurada via env var PORT (default 8080).
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
