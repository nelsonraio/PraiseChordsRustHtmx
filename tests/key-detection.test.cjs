const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const root = path.join(__dirname, '..');
const source = fs.readFileSync(path.join(root, 'static/chord-parser.js'), 'utf8');

function setup() {
  const context = { window: {}, clearTimeout, setTimeout };
  vm.createContext(context);
  // Expose the private parser only in the test VM, not in production.
  vm.runInContext(source.replace('  function applyAccidentals(', '  window.testChordToNotes = chordToNotes;\n  function applyAccidentals('), context);
  return context;
}

test('chord notes: major/minor, extensions, suspensions, alterations and slash bass', () => {
  const { window } = setup();
  const cases = {
    Cmaj7: [0, 4, 7, 11], Cm7: [0, 3, 7, 10], CM7: [0, 4, 7, 11],
    Csus2: [0, 2, 7], Csus4: [0, 5, 7], Cdim7: [0, 3, 6, 9],
    Caug: [0, 4, 8], Cadd9: [0, 4, 7, 2], C9: [0, 4, 7, 10, 2],
    'Cm7(b5)': [0, 3, 10, 6], 'C/G': [0, 4, 7], 'C/F#': [0, 4, 7, 6],
    'Bb': [10, 2, 5], 'B♭': [10, 2, 5], C5: [0, 7],
    '|': [], 'Chorus': [], '': [], 'N.C.': [], 'Cunknown': [],
  };
  for (const [chord, notes] of Object.entries(cases)) {
    assert.deepEqual(Array.from(window.testChordToNotes(chord)), notes, chord);
  }
});

test('major and minor triads across all twelve roots', () => {
  const { window } = setup();
  for (const note of ['C', 'C#', 'D', 'Eb', 'E', 'F', 'F#', 'G', 'Ab', 'A', 'Bb', 'B']) {
    assert.equal(window.DetectKey(`[${note}]`), note);
    assert.equal(window.DetectKey(`[${note}m]`), note + 'm');
  }
});

test('progressions, metadata, escaped newlines and empty input', () => {
  const { window } = setup();
  assert.equal(window.DetectKey('[C] [F] [G7] [C]'), 'C');
  assert.equal(window.DetectKey('[Am] [Dm] [E7] [Am]'), 'Am');
  assert.equal(window.DetectKey('{t: [F#]}\n{c: [F#]}\n[C] [F] [G7] [C]'), 'C');
  assert.equal(window.DetectKey('{t: [F#]}\\n{inline: [C] [F] [G7] [C]}'), 'C');
  for (const text of ['', null, 'Só letra', '[|] [>] [Chorus]', '{c: [C]}']) {
    assert.equal(window.DetectKey(text), 'N/A');
  }
});

for (const template of ['song_new.html', 'song_edit.html']) {
  function form(context) {
    const html = fs.readFileSync(path.join(root, 'templates', template), 'utf8');
    assert.match(html, /type="button" @click="detectKey\(\)"/);
    assert.match(html, /x-ref="orgKey" name="org_key"/);
    const expression = html.match(/x-data="([\s\S]*?)"/)[1];
    const component = vm.runInContext('(' + expression + ')', context);
    component.$refs = { orgKey: { value: 'D' } };
    return component;
  }

  test(`${template}: fills only on click, preserves current key on failure`, () => {
    const context = setup();
    const component = form(context);
    component.chordPro = '[C] [F] [G7] [C]';
    assert.equal(component.$refs.orgKey.value, 'D');
    component.detectKey();
    assert.equal(component.$refs.orgKey.value, 'C');
    assert.match(component.keyMessage, /Tom provável: C/);
    component.chordPro = '';
    component.detectKey();
    assert.equal(component.$refs.orgKey.value, 'C');
    assert.match(component.keyMessage, /Não foi possível/);
    delete context.window.DetectKey;
    component.detectKey();
    assert.equal(component.$refs.orgKey.value, 'C');
    assert.match(component.keyMessage, /Recarregue/);
  });

  test(`${template}: flushes pending conversion, then uses manual ChordPro changes`, () => {
    const context = setup();
    context.window.cifraToChordPro = () => '[C] [F] [G7] [C]';
    const component = form(context);
    component.cifraOriginal = 'C  F  G7  C';
    component.convert();
    component.detectKey();
    assert.equal(component.convertTimer, null);
    assert.equal(component.$refs.orgKey.value, 'C');
    component.chordPro = '[Am] [Dm] [E7] [Am]';
    component.detectKey();
    assert.equal(component.$refs.orgKey.value, 'Am');
  });
}

test('existing chord rendering and transposition remain available', () => {
  const { window } = setup();
  assert.match(window.parseChordPro2('[C]Olá', 2, 0), /class='chord'>D</);
  assert.match(window.parseChordPro2('[Am]Olá', 0, 0), /class='chord'>Am</);
});
