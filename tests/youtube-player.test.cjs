const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const root = path.join(__dirname, '..');

// Run the real shared YouTube helper (from layout.html) inside a minimal DOM double.
const layout = fs.readFileSync(path.join(root, 'templates/layout.html'), 'utf8');
const playerSource = layout.slice(
  layout.indexOf('window.praiseChordsYoutube = {'),
  layout.indexOf("document.body.addEventListener('htmx:beforeSwap'")
);

function context() {
  const ctx = {
    window: {},
    document: { createElement: () => ({ style: {}, setAttribute() {}, appendChild() {} }), body: {} },
    URL,
  };
  vm.createContext(ctx);
  vm.runInContext(playerSource, ctx);
  return ctx;
}

test('shared player builds embed/watch URLs and skips unusable links', () => {
  const player = context().window.praiseChordsYoutube;
  assert.deepEqual(Array.from(player.videos('dQw4w9WgXcQ\n\n  \nhttps://youtu.be/abcdefghijk'), v => v.videoId), ['dQw4w9WgXcQ', 'abcdefghijk']);
  assert.equal(player.embedUrl('abcdefghijk'), 'https://www.youtube.com/embed/abcdefghijk?autoplay=1&rel=0');
  assert.equal(player.watchUrl('abcdefghijk'), 'https://www.youtube.com/watch?v=abcdefghijk');
  // Playlists have no single video id, so they cannot be embedded.
  assert.equal(player.videos('https://www.youtube.com/playlist?list=PLtest').length, 0);
});

for (const template of ['song_new.html', 'song_edit.html']) {
  function form(ctx, links) {
    const html = fs.readFileSync(path.join(root, 'templates', template), 'utf8');
    assert.match(html, /type="button" @click="toggleYoutubePlayback\(\)"/);
    assert.match(html, /x-ref="youtubeLinks" name="youtube"/);
    assert.match(html, /x-text="youtubePlaying \? '■ Parar' : '▶ Tocar'"/);
    assert.match(html, /<template x-if="youtubePlaying">/);
    assert.match(html, /:src="youtubeEmbedUrl"/);
    const component = vm.runInContext('(' + html.match(/x-data="([\s\S]*?)"/)[1] + ')', ctx);
    component.$refs = { youtubeLinks: { value: links } };
    return component;
  }

  test(`${template}: play reads the field, stop clears the player`, () => {
    const ctx = context();
    const component = form(ctx, 'dQw4w9WgXcQ');
    assert.equal(component.youtubePlaying, false);
    component.toggleYoutubePlayback();
    assert.equal(component.youtubePlaying, true);
    assert.equal(component.youtubeEmbedUrl, 'https://www.youtube.com/embed/dQw4w9WgXcQ?autoplay=1&rel=0');
    assert.equal(component.youtubeWatchUrl, 'https://www.youtube.com/watch?v=dQw4w9WgXcQ');
    assert.match(component.youtubeMessage, /A tocar: Original \(principal\)/);
    component.toggleYoutubePlayback();
    assert.equal(component.youtubePlaying, false);
    assert.equal(component.youtubeEmbedUrl, '');
    assert.match(component.youtubeMessage, /parada/);
  });

  test(`${template}: several links can be switched while playing`, () => {
    const ctx = context();
    const component = form(ctx, 'dQw4w9WgXcQ\nhttps://youtu.be/abcdefghijk');
    component.toggleYoutubePlayback();
    assert.equal(component.youtubeVersions.length, 2);
    component.youtubeVersionIndex = 1;
    component.changeYoutubeVersion();
    assert.equal(component.youtubeEmbedUrl, 'https://www.youtube.com/embed/abcdefghijk?autoplay=1&rel=0');
    assert.match(component.youtubeMessage, /Versão alternativa 1/);
  });

  test(`${template}: unusable or empty links keep playback off`, () => {
    const ctx = context();
    const component = form(ctx, '\n https://youtube.com/playlist?list=PLtest \n');
    component.toggleYoutubePlayback();
    assert.equal(component.youtubePlaying, false);
    assert.equal(component.youtubeEmbedUrl, '');
    assert.equal(component.youtubeVersions.length, 0);
    assert.match(component.youtubeMessage, /Sem link de vídeo/);
    // Changing version while stopped must not start playback.
    component.youtubeVersionIndex = 1;
    component.changeYoutubeVersion();
    assert.equal(component.youtubePlaying, false);
  });
}
