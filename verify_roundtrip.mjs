import { readFileSync } from 'node:fs';

globalThis.document = {
  createElement: () => ({ style: {}, setAttribute: () => {}, appendChild: () => {} }),
  body: { appendChild: () => {}, removeChild: () => {} },
};
globalThis.window = globalThis;
globalThis.process = { emitWarning: () => {} };

// chordsheetjs
const csCode = readFileSync('static/chordsheetjs.min.js', 'utf8');
const CS = new Function('window', 'globalThis', `${csCode}; return typeof ChordSheetJS !== 'undefined' ? ChordSheetJS : globalThis.ChordSheetJS;`).call(globalThis, globalThis, globalThis);
globalThis.ChordSheetJS = CS;

// fixed cifra2chordpro
const cifraCode = readFileSync('cifra2chordpro.FIXED.js', 'utf8');
new Function('window', cifraCode).call(globalThis, globalThis);

// chord-parser (renderer)
const parserCode = readFileSync('static/chord-parser.js', 'utf8');
new Function('window', parserCode).call(globalThis, globalThis);

// Realistic CifraClub input: chords line above lyrics, NO brackets
const cifraClub = [
  '   G          D           Em         C',
  'Gra-nas que nao se iam',
  '   G          D           C          G',
  'Nao ha nada como o amor',
  '',
  '   G          D           Am        D          G',
  'Sei que e certo',
].join('\n');

const converted = globalThis.cifraToChordPro(cifraClub);
console.log('=== CONVERTED ===');
console.log(JSON.stringify(converted));
console.log(converted);

const html = globalThis.parseChordPro2(converted, 0, 0);
console.log('\n=== RENDERED HTML ===');
console.log(html);