/* syllable-align.js
   Heurística de silabação em português + realinhamento de acordes ChordPro
   para o início da sílaba onde caem (em vez de ficarem a meio de uma sílaba).
   Não é um silabador linguisticamente perfeito (não há dicionário/hifenização
   oficial embutido), mas cobre os casos comuns: ditongos, hiatos (acento em
   í/ú), dígrafos/encontros consonantais inseparáveis (br, pr, ch, lh, nh...)
   e consoantes duplicadas (rr, ss).
   Expõe: window.splitSyllables(word) -> string[]
          window.alignChordsToSyllables(chordProText) -> string
 */
(function () {
  'use strict';

  function isVowel(ch) {
    return /[aeiouáàâãéêíóôõúAEIOUÁÀÂÃÉÊÍÓÔÕÚ]/.test(ch);
  }
  // í/ú acentuados a seguir a outra vogal tipicamente marcam hiato (ex.: sa-ú-de, ba-ú)
  function isHiatusVowel(ch) {
    return /[íúÍÚ]/.test(ch);
  }
  var INSEPARABLE = [
    'bl', 'br', 'cl', 'cr', 'dl', 'dr', 'fl', 'fr', 'gl', 'gr',
    'pl', 'pr', 'tl', 'tr', 'vl', 'vr', 'ch', 'lh', 'nh',
  ];

  function splitSyllables(word) {
    var n = word.length;
    if (n === 0) return [word];
    var vowelIdx = [];
    for (var i = 0; i < n; i++) {
      if (isVowel(word[i])) vowelIdx.push(i);
    }
    if (vowelIdx.length <= 1) return [word];

    var boundaries = [0];
    for (var k = 0; k < vowelIdx.length - 1; k++) {
      var v1 = vowelIdx[k];
      var v2 = vowelIdx[k + 1];
      var gap = v2 - v1 - 1;
      if (gap === 0) {
        if (isHiatusVowel(word[v2])) boundaries.push(v2);
      } else if (gap === 1) {
        boundaries.push(v1 + 1);
      } else {
        var c1 = v1 + 1;
        var cLast = v2 - 1;
        var pair = word[cLast - 1] + word[cLast];
        if (gap === 2 && INSEPARABLE.indexOf(pair.toLowerCase()) !== -1) {
          boundaries.push(c1);
        } else if (gap === 2 && word[c1].toLowerCase() === word[c1 + 1].toLowerCase()) {
          boundaries.push(c1 + 1);
        } else {
          boundaries.push(cLast);
        }
      }
    }
    boundaries.push(n);

    var uniq = boundaries.filter(function (b, idx) {
      return boundaries.indexOf(b) === idx;
    }).sort(function (a, b) { return a - b; });

    var syllables = [];
    for (var j = 0; j < uniq.length - 1; j++) {
      syllables.push(word.slice(uniq[j], uniq[j + 1]));
    }
    return syllables.length ? syllables : [word];
  }

  // Linhas de diretivas ({t:...}, {soc:...}, etc.) não têm acordes inline a alinhar.
  function isChordLine(line) {
    return line.indexOf('[') !== -1 && line.indexOf(']') !== -1 && !/^\s*\{.*\}\s*$/.test(line);
  }

  function alignLine(line) {
    if (!isChordLine(line)) return line;

    var chordRe = /\[[^\]]*\]/g;
    var chords = [];
    var plain = '';
    var lastIndex = 0;
    var match;
    while ((match = chordRe.exec(line))) {
      plain += line.slice(lastIndex, match.index);
      chords.push({ tag: match[0], pos: plain.length });
      lastIndex = match.index + match[0].length;
    }
    plain += line.slice(lastIndex);
    if (!chords.length) return line;

    var wordRe = /[A-Za-zÀ-ÖØ-öø-ÿ]+/g;
    var words = [];
    var wordMatch;
    while ((wordMatch = wordRe.exec(plain))) {
      var word = wordMatch[0];
      var start = wordMatch.index;
      var syllables = splitSyllables(word);
      var offsets = [start];
      var acc = start;
      for (var s = 0; s < syllables.length; s++) {
        acc += syllables[s].length;
        offsets.push(acc);
      }
      words.push({ start: start, end: start + word.length, offsets: offsets });
    }

    function snapToSyllableStart(pos) {
      for (var w = 0; w < words.length; w++) {
        var word = words[w];
        if (pos > word.start && pos < word.end) {
          var best = word.start;
          for (var o = 0; o < word.offsets.length; o++) {
            if (word.offsets[o] <= pos) best = word.offsets[o];
            else break;
          }
          return best;
        }
      }
      return pos;
    }

    chords.forEach(function (c) { c.pos = snapToSyllableStart(c.pos); });
    chords.sort(function (a, b) { return a.pos - b.pos; });

    var result = '';
    var cursor = 0;
    chords.forEach(function (c) {
      result += plain.slice(cursor, c.pos) + c.tag;
      cursor = c.pos;
    });
    result += plain.slice(cursor);
    return result;
  }

  window.splitSyllables = splitSyllables;
  window.alignChordsToSyllables = function (text) {
    return String(text || '').split('\n').map(alignLine).join('\n');
  };
}());
