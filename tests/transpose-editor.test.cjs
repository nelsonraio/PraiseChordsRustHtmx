const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const root = path.join(__dirname, '..');
const parserSource = fs.readFileSync(path.join(root, 'static', 'chord-parser.js'), 'utf8');
const css = fs.readFileSync(path.join(root, 'static', 'tailwind.css'), 'utf8');

function parserSetup() {
  const context = { window: {}, clearTimeout, setTimeout };
  vm.createContext(context);
  vm.runInContext(parserSource, context);
  return context;
}

// Cria o componente Alpine real dos formulários, como key-detection.test.cjs.
function formComponent(template) {
  const html = fs.readFileSync(path.join(root, 'templates', template), 'utf8');
  const expression = html.match(/x-data="([\s\S]*?)"/)[1];
  const context = { window: { parseChordPro2: () => '<div>preview</div>' }, clearTimeout, setTimeout };
  vm.createContext(context);
  vm.runInContext(parserSource, context);
  const component = vm.runInContext('(' + expression + ')', context);
  return component;
}

test('transposeChordPro: transpõe todos os acordes meio tom', () => {
  const { window } = parserSetup();
  assert.equal(window.transposeChordPro('[C] [F] [G7] [C]', 1), '[C#] [F#] [G#7] [C#]');
  assert.equal(window.transposeChordPro('[Am]letra [Dm]letra [E7]', 1), '[A#m]letra [D#m]letra [F7]');
});

test('transposeChordPro: bemóis, baixo com barra e ciclos de 12', () => {
  const { window } = parserSetup();
  assert.equal(window.transposeChordPro('[Bb] [Eb/G] [F#m7b5]', 1), '[B] [E/G#] [Gm7b5]');
  // Uma oitava completa devolve os acordes originais.
  const text = '[C] [Am7] [D/F#] [Bb]';
  assert.equal(window.transposeChordPro(text, 12), text);
  assert.equal(window.transposeChordPro(text, -12), text);
  assert.equal(window.transposeChordPro(text, 13), window.transposeChordPro(text, 1));
});

test('transposeChordPro: letra, diretivas e comentários ficam intactos', () => {
  const { window } = parserSetup();
  const text = '{t: Canção [X]}\n{c: [F#] é comentário}\nA [C]letra [G]fica [C]igual';
  const up = window.transposeChordPro(text, 1);
  assert.match(up, /Canção \[X\]/);
  assert.match(up, /é comentário/);
  assert.match(up, /A \[C#\]letra \[G#\]fica \[C#\]igual/);
});

test('transposeChordPro: passos nulos ou vazios não alteram nada', () => {
  const { window } = parserSetup();
  for (const steps of [0, null, undefined]) {
    assert.equal(window.transposeChordPro('[C] olá', steps), '[C] olá');
  }
  assert.equal(window.transposeChordPro('', 1), '');
  assert.equal(window.transposeChordPro(null, 1), '');
});

for (const template of ['song_new.html', 'song_edit.html']) {
  const html = fs.readFileSync(path.join(root, 'templates', template), 'utf8');

  test(`${template}: botões de transposição ao lado de "Alinhar Acordes"`, () => {
    const shortcuts = html.match(/Atalhos:[\s\S]*?<\/div>/)[0];
    const alignIndex = shortcuts.indexOf('alignChords()');
    assert.ok(alignIndex > -1, 'botão de alinhar não encontrado na barra de atalhos');
    const upIndex = shortcuts.indexOf('transposeChords(1)');
    const downIndex = shortcuts.indexOf('transposeChords(-1)');
    assert.ok(upIndex > alignIndex, '+1 deve ficar ao lado do botão de alinhar');
    assert.ok(downIndex > upIndex, '−1 deve seguir o +1');
    assert.match(shortcuts, /title="Subir todos os acordes meio tom"/);
    assert.match(shortcuts, /title="Baixar todos os acordes meio tom"/);
  });

  test(`${template}: classes dos novos botões existem no tailwind.css`, () => {
    const buttons = html.match(/<button type="button" @click="transposeChords\(1\)"[\s\S]*?<\/button>/)[0];
    const classNames = buttons.match(/class="([^"]+)"/)[1].split(/\s+/);
    // No CSS compilado, os dois-pontos das variantes (ex.: hover:) surgem escapados (\:).
    const missing = classNames.filter(cls => {
      const escaped = cls.replace(/[.*+?^${}()|[\]\\]/g, '\\$&').replace(/:/g, '\\\\:');
      return !new RegExp('\\.' + escaped + '[\\s,{]').test(css);
    });
    assert.deepEqual(missing, [], 'classes ausentes no tailwind.css');
  });

  test(`${template}: transposeChords atualiza o ChordPro e a pré-visualização`, () => {
    const component = formComponent(template);
    component.chordPro = '[C] [F] [G7]';
    component.transposeChords(2);
    assert.equal(component.chordPro, '[D] [G] [A7]');
    // A pré-visualização usa o parser real e mostra os acordes transpostos.
    assert.match(component.preview, /class='chord'>D</);
    assert.match(component.preview, /class='chord'>G</);
    assert.match(component.preview, /class='chord'>A7</);
    component.chordPro = '[D] [G] [A7]';
    component.transposeChords(-1);
    assert.equal(component.chordPro, '[C#] [F#] [G#7]');
    component.transposeChords(12);
    assert.equal(component.chordPro, '[C#] [F#] [G#7]');
  });

  test(`${template}: sem o parser carregado, o ChordPro fica intacto`, () => {
    const html = fs.readFileSync(path.join(root, 'templates', template), 'utf8');
    const expression = html.match(/x-data="([\s\S]*?)"/)[1];
    const context = { window: {}, clearTimeout, setTimeout };
    vm.createContext(context);
    const component = vm.runInContext('(' + expression + ')', context);
    component.chordPro = '[C] olá';
    component.transposeChords(1);
    assert.equal(component.chordPro, '[C] olá');
  });
}

test('parseChordPro2: duas linhas inline seguidas ficam em blocos separados', () => {
  const { window } = parserSetup();
  const output = window.parseChordPro2('{inline: [C]linha um}\n{inline: [G]linha dois}', 0, 0, false);
  const blocks = output.match(/<div class='inline-block'>/g) || [];
  assert.equal(blocks.length, 2);
  assert.match(output, /linha um<\/div><\/div><div class='inline-block'><div class='inline resize'>Glinha dois/);
  // Uma linha inline isolada continua a gerar um único bloco fechado.
  const single = window.parseChordPro2('{inline: [C]linha um}', 0, 0, false);
  assert.equal((single.match(/<div class='inline-block'>/g) || []).length, 1);
  assert.match(single, /<\/div><\/div>$/);
});

test('layout.html: versão dos estilos e do parser para invalidar a cache', () => {
  const layout = fs.readFileSync(path.join(root, 'templates', 'layout.html'), 'utf8');
  assert.match(layout, /\/static\/chord-parser\.js\?v=transpose-3/);
  assert.match(layout, /\/static\/chordpro\.css\?v=7/);
});

test('chordpro.css: blocos inline seguidos ficam como linhas seguidas', () => {
  const css = fs.readFileSync(path.join(root, 'static', 'chordpro.css'), 'utf8');
  assert.match(css, /\.chord-container \.inline-block \{[^}]*display:\s*block/);
  // Sem o espaçamento de bloco (.9em) entre inline seguidos: comportam-se
  // como linhas normais, só com um pequeno respiro.
  assert.match(css, /\.chord-container \.inline-block \+ \.inline-block \{[^}]*margin-top:\s*-/);
});

test('applyAccidentalsChordPro: converte sustenidos em bemóis e vice-versa', () => {
  const { window } = parserSetup();
  assert.equal(window.applyAccidentalsChordPro('[C#] [F#] [G#7]', -1), '[Db] [Gb] [Ab7]');
  assert.equal(window.applyAccidentalsChordPro('[Db] [Gb] [Ab7]', 1), '[C#] [F#] [G#7]');
  // O baixo com barra também é convertido.
  assert.equal(window.applyAccidentalsChordPro('[Bb/Db] [F#/A#]', 1), '[A#/C#] [F#/A#]');
  assert.equal(window.applyAccidentalsChordPro('[A#/C#] [F#/G]', -1), '[Bb/Db] [Gb/G]');
  // Alturas extremas: E#→F e Cb→B (a substituição simbólica daria E♭/C♭... errado).
  assert.equal(window.applyAccidentalsChordPro('[E#] [Cb]', 1), '[F] [B]');
  // Letra e diretivas ficam intactas.
  const text = '{c: nota C#}\nAbc [C#]letra [Eb]';
  assert.equal(window.applyAccidentalsChordPro(text, -1), '{c: nota C#}\nAbc [Db]letra [Eb]');
});

test('applyAccidentalsChordPro: sem acidentes ou sem passos não altera nada', () => {
  const { window } = parserSetup();
  assert.equal(window.applyAccidentalsChordPro('[C] [G] olá', 1), '[C] [G] olá');
  assert.equal(window.applyAccidentalsChordPro('[C] [G] olá', -1), '[C] [G] olá');
  assert.equal(window.applyAccidentalsChordPro('[C] olá', 0), '[C] olá');
  assert.equal(window.applyAccidentalsChordPro('', 1), '');
});

for (const template of ['song_new.html', 'song_edit.html']) {
  const html = fs.readFileSync(path.join(root, 'templates', template), 'utf8');

  test(`${template}: botão de grafia junto aos de transposição`, () => {
    const shortcuts = html.match(/Atalhos:[\s\S]*?<\/div>/)[0];
    const downIndex = shortcuts.indexOf('transposeChords(-1)');
    const toggleIndex = shortcuts.indexOf('toggleAccidentals()');
    assert.ok(toggleIndex > downIndex, 'o botão # ⇄ b deve seguir os de transposição');
    assert.match(shortcuts, /title="Alternar a grafia dos acordes entre sustenidos \(#\) e bemóis \(b\)"/);
  });

  test(`${template}: toggleAccidentals alterna a grafia dos acordes`, () => {
    const component = formComponent(template);
    component.chordPro = '[C#] [F#] letra';
    component.toggleAccidentals();
    assert.equal(component.chordPro, '[Db] [Gb] letra');
    assert.match(component.preview, /class='chord'>Db</);
    component.chordPro = '[Db] [Gb] letra';
    component.toggleAccidentals();
    assert.equal(component.chordPro, '[C#] [F#] letra');
    // Sem acidentes, o texto fica intacto.
    component.chordPro = '[C] [G] letra';
    component.toggleAccidentals();
    assert.equal(component.chordPro, '[C] [G] letra');
  });

  test(`${template}: sem o parser carregado, o toggle não altera nada`, () => {
    const html = fs.readFileSync(path.join(root, 'templates', template), 'utf8');
    const expression = html.match(/x-data="([\s\S]*?)"/)[1];
    const context = { window: {}, clearTimeout, setTimeout };
    vm.createContext(context);
    const component = vm.runInContext('(' + expression + ')', context);
    component.chordPro = '[C#] olá';
    component.toggleAccidentals();
    assert.equal(component.chordPro, '[C#] olá');
  });
}
