import { readFileSync } from 'node:fs';

globalThis.document = {
  createElement: () => ({ style: {}, setAttribute: () => {}, appendChild: () => {} }),
  body: { appendChild: () => {}, removeChild: () => {} },
};
globalThis.window = globalThis;
globalThis.process = { emitWarning: () => {} };

const csCode = readFileSync('static/chordsheetjs.min.js', 'utf8');
const CS = new Function('window', 'globalThis', `${csCode}; return typeof ChordSheetJS !== 'undefined' ? ChordSheetJS : globalThis.ChordSheetJS;`).call(globalThis, globalThis, globalThis);
globalThis.ChordSheetJS = CS;

const input = `teste
A B C D
teste teste`;

console.log('INPUT:', JSON.stringify(input));

try {
  const song = new CS.UltimateGuitarParser().parse(input);
  console.log('UG FORMATTED:', new CS.ChordProFormatter().format(song));
} catch (e) {
  console.error('UG PARSE ERROR:', e.message);
}
