/* Port of src/lib/chord-parser.ts used by the original React chord viewer. */
(function () {
  const sharpNotes = ['A', 'A#', 'B', 'C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#'];
  const sharpToFlat = { 'A#': 'Bb', 'C#': 'Db', 'D#': 'Eb', 'F#': 'Gb', 'G#': 'Ab' };

  // Perfis Krumhansl-Schmuckler, por classe de nota: C, C#, ... B.
  const majorProfile = [6.35, 2.23, 3.48, 2.33, 4.38, 4.09, 2.52, 5.19, 2.39, 3.66, 2.29, 2.88];
  const minorProfile = [6.33, 2.68, 3.52, 5.38, 2.60, 3.53, 2.54, 4.75, 3.98, 2.69, 3.34, 3.17];
  const keyNotes = ['C', 'C#', 'D', 'Eb', 'E', 'F', 'F#', 'G', 'Ab', 'A', 'Bb', 'B'];
  const naturalPitches = { C: 0, D: 2, E: 4, F: 5, G: 7, A: 9, B: 11 };

  function notePitch(note) {
    if (!/^[A-G][#b]?$/.test(note)) return null;
    return (naturalPitches[note[0]] + (note[1] === '#' ? 1 : note[1] === 'b' ? -1 : 0) + 12) % 12;
  }

  function chordToNotes(chord) {
    const value = String(chord || '').trim().replace(/♯/g, '#').replace(/♭/g, 'b');
    const match = /^([A-G][#b]?)([^/]*)(?:\/([A-G][#b]?))?$/.exec(value);
    if (!match) return [];
    const root = notePitch(match[1]);
    // Tokenização completa: não interpretar anotações como se fossem acordes.
    const quality = match[2].replace(/[()]/g, '');
    const tokens = quality.match(/maj|Maj|M|min|m|dim|aug|sus2|sus4|sus|add(?:2|4|9|11|13)|[#b](?:5|9|11|13)|13|11|9|7|6|5|4|2|°|ø|\+|-/g) || [];
    if (tokens.join('') !== quality) return [];
    const diminished = tokens.includes('dim') || tokens.includes('°') || tokens.includes('ø');
    const augmented = tokens.includes('aug') || tokens.includes('+');
    const minor = ['m', 'min', '-'].includes(tokens[0]) || diminished;
    let intervals = [0, minor ? 3 : 4, diminished ? 6 : augmented ? 8 : 7];
    if (tokens.includes('sus2') || quality === '2') intervals[1] = 2;
    if (tokens.includes('sus4') || tokens.includes('sus') || quality === '4') intervals[1] = 5;
    if (quality === '5') intervals = [0, 7];
    const extension = [13, 11, 9, 7].find(n => tokens.includes(String(n))) || 0;
    if (extension || tokens.includes('ø')) {
      const majorSeventh = tokens.some(t => ['maj', 'Maj', 'M'].includes(t));
      intervals.push(majorSeventh ? 11 : diminished && !tokens.includes('ø') && extension === 7 ? 9 : 10);
    }
    if (tokens.includes('6')) intervals.push(9);
    if (extension >= 9) intervals.push(2);
    if (extension >= 11) intervals.push(5);
    if (extension >= 13) intervals.push(9);
    const degrees = { 2: 2, 4: 5, 5: 7, 9: 2, 11: 5, 13: 9 };
    tokens.forEach(token => {
      if (token.startsWith('add')) intervals.push(degrees[token.slice(3)]);
      if (/^[#b]/.test(token)) {
        const degree = token.slice(1);
        const original = degree === '5' ? intervals[2] : degrees[degree];
        intervals = intervals.filter(interval => interval !== original);
        intervals.push((degrees[degree] + (token[0] === '#' ? 1 : -1) + 12) % 12);
      }
    });
    const notes = intervals.map(interval => (root + interval) % 12);
    if (match[3]) notes.push(notePitch(match[3]));
    return [...new Set(notes)];
  }

  function profileCorrelation(histogram, profile, shift) {
    const mean = histogram.reduce((sum, value) => sum + value, 0) / 12;
    const profileMean = profile.reduce((sum, value) => sum + value, 0) / 12;
    let product = 0, variance = 0, profileVariance = 0;
    for (let i = 0; i < 12; i++) {
      const a = histogram[i] - mean;
      const b = profile[(i - shift + 12) % 12] - profileMean;
      product += a * b;
      variance += a * a;
      profileVariance += b * b;
    }
    return variance ? product / Math.sqrt(variance * profileVariance) : -Infinity;
  }

  function detectKeyFromChords(chords) {
    const histogram = Array(12).fill(0);
    chords.forEach(chord => chordToNotes(chord).forEach(note => histogram[note]++));
    let best = -Infinity, key = 'N/A';
    keyNotes.forEach((note, shift) => {
      [majorProfile, minorProfile].forEach((profile, mode) => {
        const score = profileCorrelation(histogram, profile, shift);
        if (score > best) { best = score; key = note + (mode ? 'm' : ''); }
      });
    });
    return key;
  }

  // Estimativa, não certeza: não considera duração, melodia ou modulações.
  window.DetectKey = function (chordPro) {
    const chords = [];
    String(chordPro || '').replace(/\\r\\n|\\n|\\r/g, '\n').split(/\r?\n/).forEach(line => {
      // Ignorar metadados/comentários; {inline: ...} pode conter acordes tocados.
      if (/^\s*\{/.test(line) && !/^\s*\{inline\s*:/i.test(line)) return;
      for (const match of line.matchAll(/\[([^\]]*)\]/g)) chords.push(match[1]);
    });
    return detectKeyFromChords(chords);
  };

  // Nomes por altura (pitch 0..11), para conversão gráfica correta:
  // E#→F, Fb→E, Cb→B, B#→C (a substituição simbólica daria E♭/F#, errados).
  const pitchSharps = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];
  const pitchFlats = ['C', 'Db', 'D', 'Eb', 'E', 'F', 'Gb', 'G', 'Ab', 'A', 'Bb', 'B'];

  // Converte a grafia das notas de um acorde: 1 = sustenidos, -1 = bemóis.
  function applyAccidentals(chord, accidentals) {
    if (!accidentals) return chord;
    const names = accidentals === 1 ? pitchSharps : pitchFlats;
    return String(chord).replace(/([A-G])([#b])?/g, function (match, letter, accidental) {
      // Sem acidente não há nada a converter; com acidente, converter sempre
      // pela altura real: E#→F e B#→C (para #), Cb→B e Fb→E (para b).
      if (!accidental) return match;
      const pitch = (naturalPitches[letter] + (accidental === '#' ? 1 : -1) + 12) % 12;
      return names[pitch];
    });
  }

  function transposeChord(chord, steps) {
    return String(chord).replace(/([A-G][b#]?)/g, function (note) {
      let sharp;
      if (note.length > 1 && note[1] === 'b') sharp = note[0] === 'A' ? 'G#' : String.fromCharCode(note.charCodeAt(0) - 1) + '#';
      else sharp = note;
      const index = sharpNotes.indexOf(sharp);
      if (index < 0) return 'XX';
      // Preservar a grafia: bemóis continuam bemóis (Bb+12 → Bb, não A#).
      const transposed = sharpNotes[(index + steps + 1200) % sharpNotes.length];
      return note.length > 1 && note[1] === 'b' ? (sharpToFlat[transposed] || transposed) : transposed;
    });
  }

  // Transpõe de uma só vez todos os acordes entre parênteses retos [X] de um
  // documento ChordPro. A letra, as diretivas {…} e os comentários ficam intactos.
  window.transposeChordPro = function (chordPro, steps) {
    if (!steps) return String(chordPro || '');
    return String(chordPro || '').replace(/\[([^\][\r\n]*)\]/g, function (match, chord) {
      return '[' + transposeChord(chord, steps) + ']';
    });
  };

  // Converte de uma só vez a grafia de todos os acordes [X] de um documento
  // ChordPro para sustenidos (accidentals = 1) ou bemóis (accidentals = -1).
  window.applyAccidentalsChordPro = function (chordPro, accidentals) {
    if (!accidentals) return String(chordPro || '');
    return String(chordPro || '').replace(/\[([^\][\r\n]*)\]/g, function (match, chord) {
      return '[' + applyAccidentals(chord, accidentals) + ']';
    });
  };

  function chordLine(line, transpose, accidentals, hideChords) {
    if (hideChords) line = line.replace(/^[ \t]+/, '');
    let html = "<div class='chordline'>";
    const parts = (line + ' ').split('[').map(function (part) {
      let chord = '', lyric = part;
      if (part.indexOf(']') !== -1) [chord, lyric] = part.split(']');
      // Preservar espaços entre acordes (indicam duração/posição): só usar
      // um espaço mínimo quando não há mesmo nenhum texto (ex.: acorde no fim da linha).
      // Em modo "sem acordes" este espaço extra não tem função e apareceria como
      // espaço solto (ex.: início de linha que começa logo por um acorde).
      if (lyric === '') lyric = hideChords ? '' : ' ';
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

  window.parseChordPro2 = function (chordPro, transpose, accidentals, hideChords) {
    const simple = /^{(title|t|subtitle|st|comment|c|key|tempo):\s*(.*)}/;
    const normal = /^[A-Za-zÀ-ÖØ-öø-ÿ0-9\s,\.()\-:]*$/;
    const blockStart = /{(soc|start_of_chorus|sop|start_of_part|sov|start_of_verse|sob|start_of_bridge):([A-Za-zÀ-ÖØ-öø-ÿ0-9\s,\.()\-:]*)}/;
    const blockEnd = /^{(eoc|end_of_chorus|eop|end_of_part|eov|end_of_verse|eob|end_of_bridge)}\s*$/;
    const inline = /^{(inline):\s*(.*)}/i;
    const buffer = [];
    let inSection = false, lastWasSubtitle = false, inCommentBlock = false, inInlineBlock = false, inBlock = false, inImplicitBlock = false;
    const lines = String(chordPro || '').replace(/-->/g, '→').replace(/\r\n/g, '\n').replace(/\\n/g, '\n').replace(/\\r/g, '').split('\n');
    const closeComment = function () { if (inCommentBlock) { buffer.push('</div>'); inCommentBlock = false; } };
    const closeInline = function () { if (inInlineBlock) { buffer.push('</div>'); inInlineBlock = false; } };
    // Versos escritos sem {sop}/{eop} passam a ser agrupados implicitamente num
    // parágrafo, como se tivessem {sop:} e {eop} (equivalente a start_of_part).
    const openImplicitBlock = function () {
      if (!inImplicitBlock && !inBlock) {
        buffer.push("<div class='block block-sop'><div class='sop resize'>");
        inImplicitBlock = true;
      }
    };
    const closeImplicitBlock = function () {
      if (inImplicitBlock) { buffer.push('</div></div>'); inImplicitBlock = false; }
    };

    lines.forEach(function (line) {
      let result;
      if ((result = simple.exec(line))) {
        const command = result[1], text = result[2], isComment = command === 'comment' || command === 'c';
        closeImplicitBlock();
        if (!isComment) closeComment();
        closeInline();
        if (command === 'subtitle' || command === 'st') {
          if (inSection) buffer.push('</div>');
          buffer.push("<div class='section'>"); inSection = true; lastWasSubtitle = true;
        }
        if (isComment && !inCommentBlock) { buffer.push("<div class='comment-block'>"); inCommentBlock = true; }
        buffer.push("<div class='" + command + " resize'>" + text + '</div>');
      } else if ((result = inline.exec(line))) {
        closeComment(); closeImplicitBlock();
        // Cada linha {inline:} forma a sua própria linha visual independente.
        // Fechar o bloco anterior evita que duas linhas seguidas sejam coladas
        // visualmente dentro do mesmo contentor.
        closeInline();
        buffer.push("<div class='inline-block'>"); inInlineBlock = true;
        const output = result[2].split(/\[([^\]]*)\]/).map(function (part, index) {
          return index % 2 ? applyAccidentals(transpose ? transposeChord(part, transpose) : part, accidentals) : part;
        }).join('');
        buffer.push("<div class='inline resize'>" + output + '</div>');
        closeInline();
      } else if (normal.test(line)) {
        if (line.trim() === '') {
          closeImplicitBlock();
        } else {
          openImplicitBlock();
          buffer.push("<div class='textonly resize'>" + (hideChords ? line.replace(/^[ \t]+/, '') : line) + '</div>');
        }
      } else if (line.indexOf(']') >= 0) {
        closeComment(); closeInline(); openImplicitBlock();
        buffer.push(chordLine(line, transpose || 0, accidentals || 0, hideChords));
      } else if ((result = blockStart.exec(line))) {
        closeComment(); closeInline(); closeImplicitBlock(); inBlock = true;
        buffer.push("<div class='block block-" + result[1] + "'><span class='BlkText resize'>" + result[2].trim() + "</span><div class='" + result[1] + " resize'>");
      } else if (blockEnd.test(line)) {
        closeComment(); closeInline();
        if (inBlock) {
          inBlock = false;
          buffer.push('</div></div>');
        } else {
          closeImplicitBlock();
        }
      } else if (/^\s*$/.test(line)) {
        closeComment(); closeInline(); closeImplicitBlock();
        if (lastWasSubtitle) { lastWasSubtitle = false; return; }
        if (inSection) { buffer.push('</div>'); inSection = false; }
        buffer.push("<div class='emptyline'></div>");
      } else lastWasSubtitle = false;
    });
    closeComment(); closeInline(); closeImplicitBlock(); if (inSection) buffer.push('</div>');
    return buffer.join('');
  };
}());
