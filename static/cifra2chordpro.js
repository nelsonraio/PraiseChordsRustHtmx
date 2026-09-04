/* cifra2chordpro.js
   Converte uma cifra em texto (formato CifraClub: linha de acordes por cima da letra)
   para o formato ChordPro, usando a biblioteca ChordSheetJS
   (ChordSheetParser + ChordProFormatter). Se o texto já parecer ChordPro (acordes
   inline [G], [Bb], [D/F#]...), é devolvido tal como está.
   Expõe: window.cifraToChordPro(texto) -> string
 */
(function () {
  'use strict';

    // Deteta acordes inline no estilo ChordPro, ex.: [C], [Am7], [Bb], [D/F#], [G#m7]
  function looksLikeChordPro(text) {
    // NOTA: o '/' dentro do regex literal precisa de escape (\/), senão o motor
    // de JS interpreta-o como delimitador de regex e lança SyntaxError,
    // invalidando todo o ficheiro de conversão (chordsheetjs não é usado).
    return /\[[A-Ga-g][#b]?(?:m(?:aj|in|7)?|maj|min|dim|aug|sus|add|\d|\(|\/)*[^\]]*\]/.test(text);
  }

  function convert(text) {
    var value = String(text || '');
    if (!value.trim()) return '';
    value = value.replace(/\r\n/g, '\n').replace(/\r/g, '\n');

    // Já está em ChordPro? Devolve tal como está (só normaliza fins de linha).
    if (looksLikeChordPro(value)) return value;

    // Sem a biblioteca carregada, devolvemos o original para edição manual.
    var CS = window.ChordSheetJS || (typeof ChordSheetJS !== 'undefined' ? ChordSheetJS : null);
    if (!CS || !CS.ChordSheetParser || !CS.ChordProFormatter) return value;

    try {
      var song = new CS.ChordSheetParser().parse(value);
      var formatted = new CS.ChordProFormatter().format(song);
      // O ChordSheetParser cria por vezes um "acorde" vazio com os espaços de
      // alinhamento anteriores ao primeiro acorde real da linha (ex.: "[   ]texto"),
      // que aparece na pré-visualização como um colchete/chaveta vazia. Remove-se aqui.
      return formatted.replace(/\[[ \t]*\]/g, '');
    } catch (e) {
      return value;
    }
  }

  window.cifraToChordPro = convert;
}());