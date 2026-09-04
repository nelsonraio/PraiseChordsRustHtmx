path = r'C:\laragon\www\PraiseChordsRustHtmx\src\main.rs'
with open(path, 'r', encoding='utf-8') as f:
    content = f.read()

# list_songs_json
content = content.replace(
    'inserted_by: String::new(),\n            updated_by: String::new(),\n            can_edit: is_programmer || user_id == s.id_insert_user,\n            can_delete: is_programmer || (is_admin && user_id == s.id_insert_user),\n        })\n        .collect();\n\n    Json(list).into_response()',
    'inserted_by: String::new(),\n            updated_by: String::new(),\n            is_pending: s.status.as_deref() == Some("pending"),\n            can_edit: is_programmer || user_id == s.id_insert_user,\n            can_delete: is_programmer || (is_admin && user_id == s.id_insert_user),\n            can_approve: is_admin || is_programmer,\n        })\n        .collect();\n\n    Json(list).into_response()'
)

# library_page
content = content.replace(
    'inserted_by: String::new(),\n            updated_by: String::new(),\n            can_edit: is_programmer || current_user_id == song.id_insert_user,\n            can_delete: is_programmer || (is_admin && current_user_id == song.id_insert_user),\n        })',
    'inserted_by: String::new(),\n            updated_by: String::new(),\n            is_pending: song.status.as_deref() == Some("pending"),\n            can_edit: is_programmer || current_user_id == song.id_insert_user,\n            can_delete: is_programmer || (is_admin && current_user_id == song.id_insert_user),\n            can_approve: is_admin || is_programmer,\n        })'
)

# setlist_song_picker_htmx
content = content.replace(
    'inserted_by: String::new(),\n            updated_by: String::new(),\n            can_edit: is_programmer || user_id == owner,\n            can_delete: is_programmer || (is_admin && user_id == owner),\n        })',
    'inserted_by: String::new(),\n            updated_by: String::new(),\n            is_pending: false,\n            can_edit: is_programmer || user_id == owner,\n            can_delete: is_programmer || (is_admin && user_id == owner),\n            can_approve: is_admin || is_programmer,\n        })'
)

# search_songs_htmx
content = content.replace(
    'inserted_by: s.id_insert_user.and_then(|id| user_names.get(&id).cloned()).unwrap_or_default(),\n            updated_by: s.id_update_user.and_then(|id| user_names.get(&id).cloned()).unwrap_or_default(),\n            can_edit: is_programmer || user_id == s.id_insert_user,\n            can_delete: is_programmer || (is_admin && user_id == s.id_insert_user),\n        })',
    'inserted_by: s.id_insert_user.and_then(|id| user_names.get(&id).cloned()).unwrap_or_default(),\n            updated_by: s.id_update_user.and_then(|id| user_names.get(&id).cloned()).unwrap_or_default(),\n            is_pending: s.status.as_deref() == Some("pending"),\n            can_edit: is_programmer || user_id == s.id_insert_user,\n            can_delete: is_programmer || (is_admin && user_id == s.id_insert_user),\n            can_approve: is_admin || is_programmer,\n        })'
)

with open(path, 'w', encoding='utf-8') as f:
    f.write(content)
print('list_songs_json, library_page, setlist, search_songs_htmx updated')
