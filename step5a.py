path = r'C:\laragon\www\PraiseChordsRustHtmx\src\main.rs'
with open(path, 'r', encoding='utf-8') as f:
    content = f.read()

# Test cases
content = content.replace(
    '            can_delete: false,\n        };\n        let tpl = SongsListTemplate { songs: vec![song] };',
    '            is_pending: false,\n            can_delete: false,\n            can_approve: false,\n        };\n        let tpl = SongsListTemplate { songs: vec![song] };'
)

# list_songs_htmx
content = content.replace(
    'inserted_by: String::new(),\n            updated_by: String::new(),\n            can_edit: is_programmer || user_id == s.id_insert_user,\n            can_delete: is_programmer || (is_admin && user_id == s.id_insert_user),\n        })\n        .collect();\n\n    let tpl = SongsListTemplate { songs: list };',
    'inserted_by: String::new(),\n            updated_by: String::new(),\n            is_pending: s.status.as_deref() == Some("pending"),\n            can_edit: is_programmer || user_id == s.id_insert_user,\n            can_delete: is_programmer || (is_admin && user_id == s.id_insert_user),\n            can_approve: is_admin || is_programmer,\n        })\n        .collect();\n\n    let tpl = SongsListTemplate { songs: list };'
)

with open(path, 'w', encoding='utf-8') as f:
    f.write(content)
print('Tests and list_songs_htmx updated')
