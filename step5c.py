path = r'C:\laragon\www\PraiseChordsRustHtmx\src\main.rs'
with open(path, 'r', encoding='utf-8') as f:
    content = f.read()

# SongDetailView - song_detail_htmx
content = content.replace(
    'favorite: song.favorite.unwrap_or(false),\n            can_edit: can_edit_song(&pool, user_id, song.id_insert_user).await,\n            can_delete: can_delete_song(&pool, user_id, song.id_insert_user, &jar).await,\n            font_size: settings.font_size,',
    'favorite: song.favorite.unwrap_or(false),\n            is_pending: song.status.as_deref() == Some("pending"),\n            can_edit: can_edit_song(&pool, user_id, song.id_insert_user).await,\n            can_delete: can_delete_song(&pool, user_id, song.id_insert_user, &jar).await,\n            can_approve: is_admin_user(&pool, &jar).await || is_moderator_user(&pool, user_id).await,\n            font_size: settings.font_size,'
)

# SongDetailView - setlist player
content = content.replace(
    'favorite: song.favorite.unwrap_or(false),\n        can_edit: can_edit_song(&pool, user_id, song.id_insert_user).await,\n        can_delete: can_delete_song(&pool, user_id, song.id_insert_user, &jar).await,\n        font_size: settings.font_size,',
    'favorite: song.favorite.unwrap_or(false),\n        is_pending: song.status.as_deref() == Some("pending"),\n        can_edit: can_edit_song(&pool, user_id, song.id_insert_user).await,\n        can_delete: can_delete_song(&pool, user_id, song.id_insert_user, &jar).await,\n        can_approve: is_admin_user(&pool, &jar).await || is_moderator_user(&pool, user_id).await,\n        font_size: settings.font_size,'
)

# SongDetailView - share (com valores falsos)
content = content.replace(
    'favorite: false,\n        can_edit: false,\n        can_delete: false,\n        font_size: 16.0,',
    'favorite: false,\n        is_pending: false,\n        can_edit: false,\n        can_delete: false,\n        can_approve: false,\n        font_size: 16.0,'
)

with open(path, 'w', encoding='utf-8') as f:
    f.write(content)
print('SongDetailView instances updated')
