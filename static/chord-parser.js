/* Port of src/lib/chord-parser.ts used by the original React chord viewer. */
(function () {
  const sharpNotes = ['A', 'A#', 'B', 'C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#'];
  const flatToSharp = { Bb: 'A#', Db: 'C#', Eb: 'D#', Gb: 'F#', Ab: 'G#' };
  const sharpToFlat = { 'A#': 'Bb', 'C#': 'Db', 'D#': 'Eb', 'F#': 'Gb', 'G#': 'Ab' };

  function applyAccidentals(chord, accidentals) {
    if (!accidentals) return chord;
    return String(chord).replace(/([A-G])(?:[#b])?/g, function (match) {
      if (accidentals === 1) return flatToSharp[match] || match.replace('b', '#');
      return sharpToFlat[match] || match.replace('#', 'b');
    });
  }

  function transposeChord(chord, steps) {
    return String(chord).replace(/([A-G][b#]?)/g, function (note) {
      if (note.length > 1 && note[1] === 'b') note = note[0] === 'A' ? 'G#' : String.fromCharCode(note.charCodeAt(0) - 1) + '#';
      const index = sharpNotes.indexOf(note);
      return index < 0 ? 'XX' : sharpNotes[(index + steps + 1200) % sharpNotes.length];
    });
  }

  function chordLine(line, transpose, accidentals) {
    let html = "<div class='chordline'>";
    const parts = (line + ' ').split('[').map(function (part) {
      let chord = '', lyric = part;
      if (part.indexOf(']') !== -1) [chord, lyric] = part.split(']');
      // Preservar espaços entre acordes (indicam duração/posição): só usar
      // um espaço mínimo quando não há mesmo nenhum texto (ex.: acorde no fim da linha)
      if (lyric === '') lyric = ' ';
      if (transpose) chord = transposeChord(chord, transpose);
      if (accidentals) chord = applyAccidentals(chord, accidentals);
      return { chord: chord, lyric: lyric };
    });
    for (let i = 0; i < parts.length; i++) {
      const part = parts[i];
      const hasChord = part.chord !== '';
      if (hasChord) {
        // Se a sílaba anterior não tem acorde, agrupar apenas a ÚLTIMA palavra
        // com o acorde (para não quebrarem entre si). O resto do texto antes do
        // acorde fica fora do grupo e pode quebrar de linha normalmente — assim,
        // em modo colunas, o texto não salta todo para a linha de baixo.
        if (i > 0 && parts[i - 1].chord === '') {
          const prevLyric = parts[i - 1].lyric;
          const lastSpace = prevLyric.lastIndexOf(' ');
          const head = lastSpace === -1 ? '' : prevLyric.slice(0, lastSpace + 1);
          const tail = lastSpace === -1 ? prevLyric : prevLyric.slice(lastSpace + 1);
          if (head !== '') {
            html += "<span class='chord-block'><span class='chord'></span><span class='lyric'>" + head + '</span></span>';
          }
          if (tail !== '') {
            html += "<span class='keep-together'><span class='chord-block'><span class='chord'></span><span class='lyric'>" + tail + "</span></span><span class='chord-block'><span class='chord'>" + part.chord + "</span><span class='lyric'>" + part.lyric + '</span></span></span>';
          } else {
            html += "<span class='chord-block'><span class='chord'>" + part.chord + "</span><span class='lyric'>" + part.lyric + '</span></span>';
          }
        } else {
          html += "<span class='chord-block'><span class='chord'>" + part.chord + "</span><span class='lyric'>" + part.lyric + '</span></span>';
        }
      } else {
        // Sílaba sem acorde: só gerar se não for agrupada com a seguinte
        if (i < parts.length - 1 && parts[i + 1].chord !== '') continue;
        html += "<span class='chord-block'><span class='chord'></span><span class='lyric'>" + part.lyric + '</span></span>';
      }
    }
    return html + '</div>';
  }

  window.parseChordPro2 = function (chordPro, transpose, accidentals) {
    const simple = /^{(title|t|subtitle|st|comment|c|key|tempo):\s*(.*)}/;
    const normal = /^[A-Za-zÀ-ÖØ-öø-ÿ0-9\s,\.()\-:]*$/;
    const blockStart = /{(soc|start_of_chorus|sop|start_of_part|sov|start_of_verse|sob|start_of_bridge):([A-Za-zÀ-ÖØ-öø-ÿ0-9\s,\.()\-:]*)}/;
    const blockEnd = /^{(eoc|end_of_chorus|eop|end_of_part|eov|end_of_verse|eob|end_of_bridge)}\s*$/;
    const inline = /^{(inline):\s*(.*)}/i;
    const buffer = [];
    let inSection = false, lastWasSubtitle = false, inCommentBlock = false, inInlineBlock = false;
    const lines = String(chordPro || '').replace(/-->/g, '→').replace(/\r\n/g, '\n').replace(/\\n/g, '\n').replace(/\\r/g, '').split('\n');
    const closeComment = function () { if (inCommentBlock) { buffer.push('</div>'); inCommentBlock = false; } };
    const closeInline = function () { if (inInlineBlock) { buffer.push('</div>'); inInlineBlock = false; } };

    lines.forEach(function (line) {
      let result;
      if ((result = simple.exec(line))) {
        const command = result[1], text = result[2], isComment = command === 'comment' || command === 'c';
        if (!isComment) closeComment();
        closeInline();
        if (command === 'subtitle' || command === 'st') {
          if (inSection) buffer.push('</div>');
          buffer.push("<div class='section'>"); inSection = true; lastWasSubtitle = true;
        }
        if (isComment && !inCommentBlock) { buffer.push("<div class='comment-block'>"); inCommentBlock = true; }
        buffer.push("<div class='" + command + " resize'>" + text + '</div>');
      } else if ((result = inline.exec(line))) {
        closeComment();
        if (!inInlineBlock) { buffer.push("<div class='inline-block'>"); inInlineBlock = true; }
        const output = result[2].split(/\[([^\]]*)\]/).map(function (part, index) {
          return index % 2 ? applyAccidentals(transpose ? transposeChord(part, transpose) : part, accidentals) : part;
        }).join('');
        buffer.push("<div class='inline resize'>" + output + '</div>');
      } else if (normal.test(line)) {
        closeComment(); closeInline(); buffer.push("<div class='textonly resize'>" + line + '</div>');
      } else if (line.indexOf(']') >= 0) {
        closeComment(); closeInline(); buffer.push(chordLine(line, transpose || 0, accidentals || 0));
      } else if ((result = blockStart.exec(line))) {
        closeComment(); closeInline(); buffer.push("<div class='block block-" + result[1] + "'><span class='BlkText resize'>" + result[2].trim() + "</span><div class='" + result[1] + " resize'>");
      } else if (blockEnd.test(line)) {
        closeComment(); closeInline(); buffer.push('</div></div>');
      } else if (/^\s*$/.test(line)) {
        closeComment(); closeInline();
        if (lastWasSubtitle) { lastWasSubtitle = false; return; }
        if (inSection) { buffer.push('</div>'); inSection = false; }
        buffer.push("<div class='emptyline'></div>");
      } else lastWasSubtitle = false;
    });
    closeComment(); closeInline(); if (inSection) buffer.push('</div>');
    return buffer.join('');
  };
}());
