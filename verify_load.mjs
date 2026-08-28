import { readFileSync } from 'node:fs';

globalThis.document = {
  createElement: () => ({ style: {}, setAttribute: () => {}, appendChild: () => {} }),
  body: { appendChild: () => {}, removeChild: () => {} },
};
globalThis.window = globalThis;
globalThis.process = { emitWarning: () => {} };

const csCode = readFileSync('static/chordsheetjs.min.js', 'utf8');
const CS = new Function('window', 'globalThis', `${csCode}; return typeof ChordSheetJS !== 'undefined' ? ChordSheetJS : globalThis.ChordSheetJS;`).call(globalThis, globalThis, globalThis);

// Now load the REAL cifra2chordpro.js exactly as the browser would (via new Function)
const cifraCode = readFileSync('static/cifra2chordpro.js', 'utf8');
try {
  new Function('window', cifraCode).call(globalThis, globalThis);
  console.log('cifra2chordpro.js loaded OK. window.cifraToChordPro =', typeof globalThis.cifraToChordPro);
} catch (e) {
  console.error('cifra2chordpro.js FAILED TO LOAD:', e.message);
}