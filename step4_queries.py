path = r'C:\laragon\www\PraiseChordsRustHtmx\src\main.rs'
with open(path, 'r', encoding='utf-8') as f:
    content = f.read()

# Adicionar Status a todas as queries SELECT de songs
replacements = [
    ('"updatedAt" FROM songs ORDER BY "createdAt"', '"updatedAt", "Status" FROM songs ORDER BY "createdAt"'),
    ('"updatedAt" FROM songs ORDER BY "ID"', '"updatedAt", "Status" FROM songs ORDER BY "ID"'),
    ('s."updatedAt" FROM songs WHERE', 's."updatedAt", s."Status" FROM songs WHERE'),
    ('s."updatedAt" FROM songs s JOIN', 's."updatedAt", s."Status" FROM songs s JOIN'),
    ('"ViewCount", "createdAt", "updatedAt" FROM songs WHERE', '"ViewCount", "createdAt", "updatedAt", "Status" FROM songs WHERE'),
    ('"ViewCount", "createdAt", "updatedAt" FROM songs s JOIN', '"ViewCount", "createdAt", "updatedAt", "Status" FROM songs s JOIN'),
    ('"ViewCount", "createdAt", "updatedAt", "Status" FROM songs WHERE', '"ViewCount", "createdAt", "updatedAt", "Status" FROM songs WHERE'),  # No-op, evita duplicar
    ('"createdAt", "updatedAt" FROM songs WHERE', '"createdAt", "updatedAt", "Status" FROM songs WHERE'),
    ('"createdAt", "updatedAt" FROM songs s JOIN', '"createdAt", "updatedAt", "Status" FROM songs s JOIN'),
    # song_detail_htmx query
    ('"createdAt", "updatedAt" FROM songs WHERE "ID" = $1', '"createdAt", "updatedAt", "Status" FROM songs WHERE "ID" = $1'),
    # setlist queries
    ('"ViewCount", "createdAt", "updatedAt", "Status" FROM songs WHERE "ID" IN', '"ViewCount", "createdAt", "updatedAt", "Status" FROM songs WHERE "ID" IN'),
]

for old, new in replacements:
    content = content.replace(old, new)

with open(path, 'w', encoding='utf-8') as f:
    f.write(content)
print('Queries updated')
