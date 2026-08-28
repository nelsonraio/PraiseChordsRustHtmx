import { readFileSync } from 'node:fs';

globalThis.document = {
  createElement: () => ({ style: {}, setAttribute: () => {}, appendChild: () => {} }),
  body: { appendChild: () => {}, removeChild: () => {} },
};
globalThis.window = globalThis;
globalThis.process = { emitWarning: () => {} };

const csCode = readFileSync('static/chordsheetjs.min.js', 'utf8');
const CS = new Function('window', 'globalThis', `${csCode}; return typeof ChordSheetJS !== 'undefined' ? ChordSheetJS : globalThis.ChordSheetJS;`).call(globalThis, globalThis, globalThis);

// TRUE CifraClub format: chords on their own line above lyrics, NO brackets
const cifraClub = [
  '    G       D       Am      C',
  'Gra...nas que nao se i-ram',
  '    G       D       C         G',
  'Nao ha nada como o amor',
].join('\n');

console.log('=== CIFRA CLUB INPUT ===');
console.log(cifraClub);

try {
  const song = new CS.ChordSheetParser().parse(cifraClub);
  const out = new CS.ChordProFormatter().format(song);
  console.log('\n=== CONVERTED TO CHORDPRO ===');
  console.log(out);
} catch (e) {
  console.error('CONVERSION ERROR:', e.message);
}

// Prove the regex from the file is broken (compile it from string)
console.log('\n=== TESTING looksLikeChordPro regex (from file, as string) ===');
const fileRegexSource = '\[[A-Ga-g][#b]?(?:m(?:aj|in|7)?|maj|min|dim|aug|sus|add|\\d|\\(|/)*[^\]]*\]';
try {
  const re = new RegExp(fileRegexSource);
  console.log('regex compiled OK, test:', re.test('[G]ola [Am7] test'));
} catch (e) {
  console.error('REGEX BROKEN:', e.message);
}