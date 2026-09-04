path = r'C:\laragon\www\PraiseChordsRustHtmx\src\main.rs'
with open(path, 'r', encoding='utf-8') as f:
    content = f.read()

# 1. Adicionar Status ao struct Song
content = content.replace(
    '#[sqlx(rename = "updatedAt")]\n    updated_at: Option<NaiveDateTime>,\n}',
    '#[sqlx(rename = "updatedAt")]\n    updated_at: Option<NaiveDateTime>,\n    #[sqlx(rename = "Status")]\n    status: Option<String>,\n}'
)

# 2. Adicionar is_pending e can_approve ao SongListItem struct
content = content.replace(
    '    favorite: bool,\n    inserted_by: String,\n    updated_by: String,\n    can_edit: bool,\n    can_delete: bool,\n}',
    '    favorite: bool,\n    inserted_by: String,\n    updated_by: String,\n    is_pending: bool,\n    can_edit: bool,\n    can_delete: bool,\n    can_approve: bool,\n}'
)

# 3. Adicionar is_pending e can_approve ao SongDetailView struct
content = content.replace(
    '    favorite: bool,\n    can_edit: bool,\n    can_delete: bool,\n    font_size: f32,',
    '    favorite: bool,\n    is_pending: bool,\n    can_edit: bool,\n    can_delete: bool,\n    can_approve: bool,\n    font_size: f32,'
)

with open(path, 'w', encoding='utf-8') as f:
    f.write(content)
print('Structs atualizadas')
