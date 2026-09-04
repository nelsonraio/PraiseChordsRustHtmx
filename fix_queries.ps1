$path = 'C:\laragon\www\PraiseChordsRustHtmx\src\main.rs'
$content = [System.IO.File]::ReadAllText($path)

$content = $content.Replace(
    'FROM songs ORDER BY "createdAt" DESC NULLS LAST, "ID" DESC LIMIT 50',
    'FROM songs WHERE ("Status" = ''approved'' OR "IdInsertUser" = $1 OR $2) ORDER BY "createdAt" DESC NULLS LAST, "ID" DESC LIMIT 50'
)

$content = $content.Replace(
    'FROM songs ORDER BY "ID" DESC LIMIT 50',
    'FROM songs WHERE ("Status" = ''approved'' OR "IdInsertUser" = $1 OR $2) ORDER BY "ID" DESC LIMIT 50'
)

$content = $content.Replace(
    'WHERE f."userId" = $1 ORDER BY',
    'WHERE f."userId" = $1 AND (s."Status" = ''approved'' OR s."IdInsertUser" = $2 OR $3) ORDER BY'
)

$content = $content.Replace(
    'FROM songs ORDER BY "ViewCount" DESC NULLS LAST, "ID" DESC LIMIT 100',
    'FROM songs WHERE ("Status" = ''approved'' OR "IdInsertUser" = $1 OR $2) ORDER BY "ViewCount" DESC NULLS LAST, "ID" DESC LIMIT 100'
)

$content = $content.Replace(
    'FROM songs ORDER BY "createdAt" DESC NULLS LAST, "ID" DESC LIMIT 100',
    'FROM songs WHERE ("Status" = ''approved'' OR "IdInsertUser" = $1 OR $2) ORDER BY "createdAt" DESC NULLS LAST, "ID" DESC LIMIT 100'
)

[System.IO.File]::WriteAllText($path, $content)
Write-Output "Queries atualizadas"
