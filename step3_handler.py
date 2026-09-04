path = r'C:\laragon\www\PraiseChordsRustHtmx\src\main.rs'
with open(path, 'r', encoding='utf-8') as f:
    content = f.read()

# Adicionar handler approve_song_htmx antes de device_hash
content = content.replace(
    '\nfn device_hash(headers: &HeaderMap) -> String {',
    '''
/// Approve a pending song. Only moderators (perfil 2) or programmers (perfil 4) can approve.
async fn approve_song_htmx(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<i32>,
    jar: CookieJar,
) -> impl IntoResponse {
    let Some(user_id) = authenticated_user_id(&jar) else {
        return (
            StatusCode::UNAUTHORIZED,
            Html("<p class=\\"text-red-400\\">Sessão expirada.</p>".to_string()),
        )
            .into_response();
    };

    // Only moderators or programmers can approve songs
    if !can_moderate_song(&pool, Some(user_id)).await {
        return (
            StatusCode::FORBIDDEN,
            Html("<p class=\\"text-red-400\\">Apenas moderadores podem aprovar músicas.</p>".to_string()),
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
            Html("<p class=\\"text-red-400\\">Música não encontrada.</p>".to_string()),
        )
            .into_response(),
        Ok(_) => (
            [(
                header::HeaderName::from_static("hx-trigger"),
                "song-approved",
            )],
            Html("<p class=\\"text-green-400\\">Música aprovada com sucesso!</p>".to_string()),
        )
            .into_response(),
        Err(error) => {
            tracing::error!(error = %error, "song approve failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("<p class=\\"text-red-400\\">Não foi possível aprovar a música.</p>".to_string()),
            )
                .into_response()
        }
    }
}
'''
+ '\nfn device_hash(headers: &HeaderMap) -> String {'
)

# Adicionar rota de aprovação
content = content.replace(
    '.route("/htmx/songs/:id/delete", delete(delete_song_htmx))',
    '.route("/htmx/songs/:id/delete", delete(delete_song_htmx))\n        .route("/htmx/songs/:id/approve", post(approve_song_htmx))'
)

with open(path, 'w', encoding='utf-8') as f:
    f.write(content)
print('Handler and route added')
