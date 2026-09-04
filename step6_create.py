path = r'C:\laragon\www\PraiseChordsRustHtmx\src\main.rs'
with open(path, 'r', encoding='utf-8') as f:
    content = f.read()

# Atualizar mensagem de erro
content = content.replace(
    '"Apenas administradores podem criar músicas."</p>',
    '"Apenas colaboradores podem criar músicas."</p>'
)

# Adicionar Status na INSERT query
content = content.replace(
    '"createdAt", "updatedAt") VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, NOW(), NOW())"',
    '"createdAt", "updatedAt", "Status") VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, NOW(), NOW(), $17)"'
)

# Adicionar bind para o status (pending/approved)
content = content.replace(
    '.bind(0)\n    .execute(&pool)\n    .await;\n    match result {\n        Ok(_) => (',
    '.bind(0)\n    .bind(if is_moderator_user(&pool, user_id).await { "approved" } else { "pending" })\n    .execute(&pool)\n    .await;\n    match result {\n        Ok(_) => ('
)

with open(path, 'w', encoding='utf-8') as f:
    f.write(content)
print('create_song_htmx updated')
