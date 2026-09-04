path = r'C:\laragon\www\PraiseChordsRustHtmx\src\main.rs'
with open(path, 'r', encoding='utf-8') as f:
    content = f.read()

# Adicionar funções is_moderator_user e can_moderate_song depois de is_admin_user
content = content.replace(
    '    perfil >= 3\n}\n\n/// Check if the authenticated user is a demo account',
    '''    perfil >= 3
}

/// Check if user is a moderator (perfil 2) or programmer (perfil 4).
async fn is_moderator_user(pool: &PgPool, user_id: i32) -> bool {
    let perfil: Option<i32> = sqlx::query_scalar("SELECT perfil FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();
    matches!(perfil, Some(2) | Some(4))
}

/// Check if the user can moderate songs (moderator or programmer).
async fn can_moderate_song(pool: &PgPool, user_id: Option<i32>) -> bool {
    let Some(user_id) = user_id else {
        return false;
    };
    is_moderator_user(pool, user_id).await
}

/// Check if the authenticated user is a demo account'''
)

with open(path, 'w', encoding='utf-8') as f:
    f.write(content)
print('Helper functions added')
