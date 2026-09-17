const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');

// Execute the actual shared player, with a minimal DOM double (no dependencies).
function setup() {
  const html = fs.readFileSync(path.join(__dirname, '../templates/layout.html'), 'utf8');
  const start = html.indexOf('window.praiseChordsYoutube = {');
  const end = html.indexOf("document.body.addEventListener('htmx:beforeSwap'", start);
  function element(tag) {
    return {
      tag, children: [], style: {}, events: {},
      appendChild(child) { this.children.push(child); },
      append(...children) { this.children.push(...children); },
      setAttribute() {},
      addEventListener(name, callback) { this.events[name] = callback; },
      showModal() { this.open = true; },
      close() { this.open = false; },
      remove() { this.removed = true; },
    };
  }
  const opened = [];
  const context = {
    window: { open: (...args) => opened.push(args) },
    document: { createElement: element, body: element('body') },
    URL,
  };
  vm.runInNewContext(html.slice(start, end), context);
  return { player: context.window.praiseChordsYoutube, opened };
}

test('empty values do nothing; a legacy single ID opens directly', () => {
  const { player, opened } = setup();
  player.play(' \r\n ');
  assert.equal(opened.length, 0);
  player.play(' dQw4w9WgXcQ ');
  assert.deepEqual(opened, [['https://www.youtube.com/watch?v=dQw4w9WgXcQ', '_blank', 'noopener,noreferrer']]);
  assert.equal(player.choiceDialog, undefined);
});

test('multiple links offer ordered choices and open only the selected version', () => {
  const { player, opened } = setup();
  player.play('dQw4w9WgXcQ\r\n\r\n https://youtu.be/abcdefghijk \n');
  const dialog = player.choiceDialog;
  assert.equal(dialog.open, true);
  assert.equal(opened.length, 0);
  assert.equal(dialog.children[1].children[0].textContent, 'Original (principal)');
  assert.equal(dialog.children[2].children[0].textContent, 'Versão alternativa 1');
  dialog.children[2].events.click();
  assert.equal(opened[0][0], 'https://www.youtube.com/watch?v=abcdefghijk');
  assert.equal(dialog.removed, true);
  assert.equal(player.choiceDialog, null);
});

test('cancel and Escape dismiss without opening; reopening replaces the chooser', () => {
  const { player, opened } = setup();
  player.play('dQw4w9WgXcQ\nabcdefghijk');
  const first = player.choiceDialog;
  player.play('dQw4w9WgXcQ\n12345678901');
  assert.equal(first.removed, true);
  player.choiceDialog.children.at(-1).events.click();
  assert.equal(player.choiceDialog, null);
  player.play('dQw4w9WgXcQ\nabcdefghijk');
  let prevented = false;
  player.choiceDialog.events.cancel({ preventDefault() { prevented = true; } });
  assert.equal(prevented, true);
  assert.equal(player.choiceDialog, null);
  assert.equal(opened.length, 0);
});

test('single URLs and playlists retain existing behavior', () => {
  const { player, opened } = setup();
  player.play('https://youtube.com/shorts/dQw4w9WgXcQ');
  player.play('https://www.youtube.com/playlist?list=PLtest');
  assert.equal(opened[0][0], 'https://www.youtube.com/watch?v=dQw4w9WgXcQ');
  assert.equal(opened[1][0], 'https://www.youtube.com/playlist?list=PLtest');
});
