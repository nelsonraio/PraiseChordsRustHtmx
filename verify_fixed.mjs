import { readFileSync } from 'node:fs';

globalThis.document = {
  createElement: () => ({ style: {}, setAttribute: () => {}, appendChild: () => {} }),
  body: { appendChild: () => {}, removeChild: () => {} },
};
globalThis.window = globalThis;
globalThis.process = { emitWarning: () => {} };

// Load chordsheetjs and expose as window.ChordSheetJS
const csCode = readFileSync('static/chordsheetjs.min.js', 'utf8');
const CS = new Function('window', 'globalThis', `${csCode}; return typeof ChordSheetJS !== 'undefined' ? ChordSheetJS : globalThis.ChordSheetJS;`).call(globalThis, globalThis, globalThis);
globalThis.ChordSheetJS = CS;

// Load the FIXED cifra2chordpro.js
const cifraCode = readFileSync('cifra2chordpro.FIXED.js', 'utf8');
new Function('window', cifraCode).call(globalThis, globalThis);

console.log('cifraToChordPro type:', typeof globalThis.cifraToChordPro);

const cifraClub = '    G       D       Am      C\nGra...nas que nao se i-ram\n    G       D       C         G\nNao ha nada como o amor';
const alreadyChordPro = '[G]Gra[G]nas [D]que [Am]se [C]i-ram\n[G]Nao [D]ha [C]nada [G]mor';

console.log('\n=== CIFRACLUB INPUT -> ChordPro ===');
const out1 = globalThis.cifraToChordPro(cifraClub);
console.log(out1);
console.log('\n=== ALREADY CHORDPRO INPUT (should pass through unchanged) ===');
const out2 = globalThis.cifraToChordPro(alreadyChordPro);
console.log(out2);
console.log('\n=== pass-through identical?', out2 === alreadyChordPro);

console.log('\n=== regex detection tests ===');
console.log('[G] inline ->', /\[[A-Ga-g][#b]?(?:m(?:aj|in|7)?|maj|min|dim|aug|sus|add|\d|\(|\/)*[^\]]*\]/.test('[G]ola'));
console.log('[D/F#] slash ->', /\/\/[A-Ga-g][#b]?(?:m(?:aj|in|7)?|maj|min|dim|aug|sus|add|\d|\(|\/)*[^\]]*\]/.test('[D/F#]ola'));
console.log('plain G (no bracket) ->', /\[[A-Ga-g][#b]?(?:m(?:aj|in|7)?|maj|min|dim|aug|sus|add|\d|\(|\/)*[^\]]*\]/.test('G   D   Am'));