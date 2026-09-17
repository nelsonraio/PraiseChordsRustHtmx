const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');

// Executa o script inline real do layout.html (segundo bloco <script>), com
// um DOM duplo mínimo, e devolve os listeners registados no body.
function setup(page = { pathname: '/', elements: {} }) {
  const html = fs.readFileSync(path.join(__dirname, '..', 'templates', 'layout.html'), 'utf8');
  const marker = html.indexOf("document.body.addEventListener('htmx:beforeSwap'");
  const scriptStart = html.lastIndexOf('<script>', marker);
  const scriptEnd = html.indexOf('</script>', marker);
  const source = html.slice(scriptStart + '<script>'.length, scriptEnd);

  const listeners = {};
  const calls = { htmxTrigger: [], htmxAjax: [], reload: 0 };

  function element(id, options = {}) {
    return {
      id,
      style: {},
      classList: { add() {}, remove() {} },
      children: [],
      events: {},
      appendChild(child) { this.children.push(child); },
      append(...kids) { this.children.push(...kids); },
      addEventListener(name, callback) { this.events[name] = callback; },
      showModal() { this.open = true; },
      close() { this.open = false; },
      remove() { this.removed = true; },
      getBoundingClientRect() { return { left: 0, right: 0, top: 0, bottom: 0 }; },
      querySelector: options.querySelector || (() => null),
      closest: () => null,
      dispatchEvent() {},
    };
  }

  const elements = new Map(Object.entries(Object.assign({ modal: element('modal') }, page.elements)));
  const documentStub = {
    body: Object.assign(element('body'), {
      addEventListener(name, callback) { listeners[name] = callback; },
    }),
    getElementById(id) { return elements.get(id) || null; },
    createElement: (tag) => element(tag),
  };

  const htmxStub = {
    trigger: (...args) => calls.htmxTrigger.push(args),
    ajax: (...args) => calls.htmxAjax.push(args),
    process: () => {},
  };

  const context = {
    window: {
      htmx: htmxStub,
      open: () => {},
      addEventListener: () => {},
      location: { pathname: page.pathname, reload: () => { calls.reload += 1; } },
    },
    htmx: htmxStub,
    document: documentStub,
    navigator: { userAgent: 'node-test' },
    URL,
    console: { log: () => {} },
  };
  vm.runInNewContext(source, context);
  return { listeners, calls };
}

test('layout.html: guardar na edição atualiza a grelha de resultados', () => {
  const html = fs.readFileSync(path.join(__dirname, '..', 'templates', 'layout.html'), 'utf8');
  assert.match(html, /addEventListener\('song-updated'[\s\S]*?refreshSongsList\(\)[\s\S]*?refreshLibraryPage\(\)/);
  // song-created partilha o mesmo comportamento de atualização.
  assert.match(html, /addEventListener\('song-created'[\s\S]*?refreshSongsList\(\)[\s\S]*?refreshLibraryPage\(\)/);
});

test('song-updated na pesquisa reenvia o formulário, preservando os filtros', () => {
  const searchForm = { id: 'search-controls' };
  const songsList = { id: 'songs-list', querySelector: (selector) => (selector === 'li' ? {} : null) };
  const { listeners, calls } = setup({ pathname: '/', elements: { 'search-controls': searchForm, 'songs-list': songsList } });

  listeners['song-updated']();
  assert.equal(calls.htmxTrigger.length, 1, 'deve reenviar a pesquisa atual');
  assert.equal(calls.htmxTrigger[0][0], searchForm);
  assert.equal(calls.htmxTrigger[0][1], 'submit');
  assert.equal(calls.htmxAjax.length, 0, 'não deve recarregar a lista por defeito');
  assert.equal(calls.reload, 0, 'não deve recarregar a página no dashboard');
});

test('song-updated sem formulário de pesquisa recarrega a lista por defeito', () => {
  const songsList = { id: 'songs-list', querySelector: (selector) => (selector === 'li' ? {} : null) };
  const { listeners, calls } = setup({ pathname: '/', elements: { 'songs-list': songsList } });

  listeners['song-updated']();
  assert.equal(calls.htmxAjax.length, 1);
  assert.equal(calls.htmxAjax[0][0], 'GET');
  assert.equal(calls.htmxAjax[0][1], '/htmx/songs');
  assert.deepEqual({ ...calls.htmxAjax[0][2] }, { target: '#songs-list', swap: 'innerHTML' });
  assert.equal(calls.htmxTrigger.length, 0);
  assert.equal(calls.reload, 0);
});

test('song-updated com lista vazia não faz pedidos', () => {
  const songsList = { id: 'songs-list', querySelector: () => null };
  const searchForm = { id: 'search-controls' };
  const { listeners, calls } = setup({ pathname: '/', elements: { 'songs-list': songsList, 'search-controls': searchForm } });

  listeners['song-updated']();
  assert.equal(calls.htmxTrigger.length, 0);
  assert.equal(calls.htmxAjax.length, 0);
  assert.equal(calls.reload, 0);
});

test('song-updated nas páginas de biblioteca recarrega a página', () => {
  for (const pathname of ['/most-viewed', '/favorites', '/recent', '/pending']) {
    const { listeners, calls } = setup({ pathname, elements: {} });
    listeners['song-updated']();
    assert.equal(calls.reload, 1, `deve recarregar ${pathname}`);
    assert.equal(calls.htmxAjax.length, 0);
    assert.equal(calls.htmxTrigger.length, 0);
  }
});

test('páginas sem grelha (ex.: setlists) não recarregam nem pedem listas', () => {
  const { listeners, calls } = setup({ pathname: '/setlists/5', elements: {} });
  listeners['song-updated']();
  assert.equal(calls.reload, 0);
  assert.equal(calls.htmxAjax.length, 0);
  assert.equal(calls.htmxTrigger.length, 0);
});

test('song-created partilha a atualização da grelha', () => {
  const searchForm = { id: 'search-controls' };
  const songsList = { id: 'songs-list', querySelector: (selector) => (selector === 'li' ? {} : null) };
  const { listeners, calls } = setup({ pathname: '/', elements: { 'search-controls': searchForm, 'songs-list': songsList } });

  listeners['song-created']();
  assert.equal(calls.htmxTrigger.length, 1);
  assert.equal(calls.reload, 0);
});
