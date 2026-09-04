import re

path = r'C:\laragon\www\PraiseChordsRustHtmx\src\main.rs'
with open(path, 'r', encoding='utf-8') as f:
    content = f.read()

# Pattern para SongListItem instances que têm can_delete seguido de })
# Adicionar is_pending e can_approve após inserted_by
content = content.replace(
    'inserted_by: String::new(),\n            updated_by: String::new(),\n            can_edit:',
    'inserted_by: String::new(),\n            updated_by: String::new(),\n            is_pending: s.status.as_deref() == Some("pending"),\n            can_edit:'
)

# Adicionar can_approve após can_delete nas instâncias sem can_approve
# Pattern para SongListItem instances
content = re.sub(
    r'(can_delete: (?:is_programmer \|\| \(is_admin && user_id == s\.id_insert_user\)|is_programmer \|\| \(is_admin && current_user_id == song\.id_insert_user\)|is_programmer \|\| user_id == owner))',
    r'\1,\n            can_approve: is_admin || is_programmer',
    content
)

# Para os setlists
content = content.replace(
    'inserted_by: String::new(),\n            updated_by: String::new(),\n            can_edit: is_programmer || user_id == owner',
    'inserted_by: String::new(),\n            updated_by: String::new(),\n            is_pending: s.status.as_deref() == Some("pending"),\n            can_edit: is_programmer || user_id == owner'
)

with open(path, 'w', encoding='utf-8') as f:
    f.write(content)

print('Done')
