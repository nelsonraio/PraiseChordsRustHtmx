import { readFileSync } from 'node:fs';

globalThis.document = {
  createElement: () => ({ style: {}, setAttribute: () => {}, appendChild: () => {} }),
  body: { appendChild: () => {}, removeChild: () => {} },
};
globalThis.window = globalThis;
globalThis.process = { emitWarning: () => {} };

// Load chordsheetjs
const csCode = readFileSync('static/chordsheetjs.min.js', 'utf8');
const CS = new Function('window', 'globalThis', `${csCode}; return typeof ChordSheetJS !== 'undefined' ? ChordSheetJS : globalThis.ChordSheetJS;`).call(globalThis, globalThis, globalThis);
globalThis.ChordSheetJS = CS;

// Load cifra2chordpro.js but FIX the broken regex (escape / as \/)
let cifraCode = readFileSync('static/cifra2chordpro.js', 'utf8');
cifraCode = cifraCode.replace(
  /\|\\\(\\)\/\\)\*/,
  '|\\(|\\\\/)*'
);
// More robust: replace the whole broken return line
cifraCode = cifraCode.replace(
  /return \/\\[[A-Ga-g][#b]?\(\?:m(?:\?aj\|in\|7\?)\|maj\|min\|dim\|aug\|sus\|add\|\\d\|\\\|/)\*\[\^\\]\*\\]\/\.test\(text\);/,
  "return /\\\\[[A-Ga-g][#b]?(?:m(?:aj|in|7)?|maj|min|dim|aug|sus|add|\\\\d|\\\\(\\\\|\\\\/)*[^\\\\]]*\\\\]/.test(text);"
);

// If the messy replace above failed, do a clean targeted one
if (!cifraCode.includes('\\(\\\\|\\')) {
  // fallback: just escape the bare / inside the literal
  cifraCode = readFileSync('static/cifra2chordpro.js', 'utf8')
    .replace(/\|\(\\)\//, '|\\(|\\/')  // not reliable
    .replace('|\\(|/', '|\\|\\/|');
}

console.log('--- cifra2chordpro.js snippet ---');
const lines = cifraCode.split('\n').filter(l => l.includes('looksLikeChordPro') || l.includes('return /'));
lines.forEach(l => console.log(l.trim()));

try {
  new Function('window', cifraCode).call(globalThis, globalThis);
  console.log('\ncifraToChordPro type:', typeof globalThis.cifraToChordPro);

  const cifraClub = '    G       D       Am      C\nGra...nas que nao se i-ram\n    G       D       C         G\nNao ha nada como o amor';
  const alreadyChordPro = '[G]Gra[G]nas [D]que [Am]se [C]i-ram\n[G]Nao [D]ha [C]nada [G]mor';

  console.log('\n=== CIFRACLUB INPUT -> ChordPro ===');
  console.log(globalThis.cifraToChordPro(cifraClub));

  console.log('\n=== ALREADY CHORDPRO INPUT (should pass through) ===');
  console.log(globalThis.cifraToChordPro(alreadyChordPro));
} catch (e) {
  console.error('FAILED:', e.message);
}