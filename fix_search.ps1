$path = 'C:\laragon\www\PraiseChordsRustHtmx\src\main.rs'
$lines = [System.IO.File]::ReadAllLines($path)

# Corrigir a query SQL na linha 2324
$lines[2323] = '    let sql = format!(r#"SELECT s."ID", s."Code", s."Name", s."OrgName", s."Composer", s."ChordPro", s."Lyrics", s."Themes", s."Youtube", s."GenreType", s."Artistas", s."OrgKey", s."OrgTempo", s."Copyright", s."Favorite", s."IdInsertUser", s."IdUpdateUser", s."ViewCount", s."createdAt", s."updatedAt", s."Status" FROM songs s{genre_join}{fav_join} WHERE {condition} AND (s."Status" = ''approved'' OR s."IdInsertUser" = $3 OR $4) AND COALESCE(s."Name", '''') ILIKE $2{genre_cond} ORDER BY {sort} LIMIT 100"#);'

[System.IO.File]::WriteAllLines($path, $lines)
Write-Output "Query corrigida"
