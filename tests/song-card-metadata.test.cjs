const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const root = path.join(__dirname, '..');
const css = fs.readFileSync(path.join(root, 'static', 'tailwind.css'), 'utf8');

// O Tailwind é um CSS pré-compilado, por isso uma classe que não exista no
// ficheiro gerado simplesmente não produz efeito nenhum.
function missingClasses(markup) {
  const classNames = [...markup.matchAll(/class="([^"]+)"/g)]
    .flatMap(match => match[1].split(/\s+/))
    .filter(cls => cls && !cls.startsWith('fa'));
  return classNames.filter(cls => {
    const pattern = new RegExp('\\.' + cls.replace(/[.*+?^${}()|[\]\\]/g, '\\$&') + '\\s*\\{');
    return !pattern.test(css);
  });
}

// A grelha de resultados mostra o tom e o tempo numa linha própria de
// metadados, em vez de os misturar com a fila de botões de ação.
for (const template of ['songs_list.html', 'library.html']) {
  const html = fs.readFileSync(path.join(root, 'templates', template), 'utf8');

  test(`${template}: tom e tempo ficam fora da fila de botões`, () => {
    const buttonsStart = html.indexOf('flex items-center gap-1 bg-gray-800/60');
    assert.ok(buttonsStart > -1, 'fila de botões não encontrada');
    // A fila termina no selo de código (quando existe) ou na linha do título.
    const endMarkers = ['bg-blue-600 text-white', 'flex flex-wrap items-center gap-3']
      .map(marker => html.indexOf(marker, buttonsStart))
      .filter(index => index > buttonsStart);
    assert.ok(endMarkers.length > 0, 'fim da fila de botões não encontrado');
    const buttons = html.slice(buttonsStart, Math.min(...endMarkers));
    assert.doesNotMatch(buttons, /Tom:/);
    assert.doesNotMatch(buttons, /BPM/);
    // A "pill" amarela que ocupava espaço na fila de botões foi removida.
    assert.doesNotMatch(buttons, /bg-yellow-400 text-black/);
  });

  test(`${template}: linha de metadados apresenta tom e tempo com destaque legível`, () => {
    const meta = html.match(/<p class="(mt-1 flex flex-wrap items-center gap-6 text-xs font-semibold text-red-400)">([\s\S]*?)<\/p>/);
    assert.ok(meta, 'linha de metadados do cartão não encontrada');
    assert.match(meta[2], /\{% if song\.org_key != "" %\}/);
    assert.match(meta[2], /Tom: \{\{ song\.org_key \}\}/);
    assert.match(meta[2], /\{% if let Some\(tempo\) = song\.org_tempo %\}/);
    assert.match(meta[2], /\{\{ tempo \}\} BPM/);
    // Sem dados, a linha não é renderizada (evita espaço vazio no cartão).
    assert.match(html, /\{% if song\.org_key != "" \|\| song\.org_tempo\.is_some\(\) %\}/);
  });

  test(`${template}: classes da linha de metadados existem no tailwind.css`, () => {
    const meta = html.match(/<p class="mt-1 flex flex-wrap items-center gap-6 text-xs font-semibold text-red-400">[\s\S]*?<\/p>/);
    assert.ok(meta, 'linha de metadados do cartão não encontrada');
    // O espaçamento entre o tom e o tempo usa gap-6: gap-x-* não existe no CSS gerado.
    assert.deepEqual(missingClasses(meta[0]), [], 'classes ausentes no tailwind.css');
  });

  test(`${template}: metadados aparecem depois da fila de botões`, () => {
    const buttonsStart = html.indexOf('flex items-center gap-1 bg-gray-800/60');
    const metaStart = html.indexOf('text-red-400');
    assert.ok(metaStart > buttonsStart, 'os metadados devem ficar fora da fila de botões');
  });
}
