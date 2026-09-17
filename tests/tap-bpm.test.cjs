const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');

for (const template of ['song_new.html', 'song_edit.html']) {
  function setup() {
    const html = fs.readFileSync(path.join(__dirname, '..', 'templates', template), 'utf8');
    assert.match(html, /type="button" @click="tapBpm\(\)"/);
    assert.match(html, /x-ref="orgTempo" name="org_tempo" type="number"/);
    assert.match(html, /role="status"[^>]*x-text="tapMessage"/);
    if (template === 'song_edit.html') {
      assert.match(html, /name="org_tempo" type="number" value="\{\{ song.org_tempo \}\}"/);
    }
    let now = 0;
    const context = vm.createContext({ performance: { now: () => now } });
    const component = vm.runInContext('(' + html.match(/x-data="([\s\S]*?)"/)[1] + ')', context);
    component.$refs = { orgTempo: { value: '90' } };
    return { component, tap(time) { now = time; component.tapBpm(); } };
  }

  test(`${template}: first tap preserves tempo; subsequent taps average intervals`, () => {
    const { component, tap } = setup();
    assert.equal(component.$refs.orgTempo.value, '90');
    tap(0);
    assert.equal(component.$refs.orgTempo.value, '90');
    assert.match(component.tapMessage, /Continue/);
    tap(500);
    assert.equal(component.$refs.orgTempo.value, 120);
    tap(1100);
    assert.equal(component.$refs.orgTempo.value, 109);
    assert.match(component.tapMessage, /109 BPM/);
    component.$refs.orgTempo.value = '115';
    assert.equal(component.$refs.orgTempo.value, '115');
  });

  test(`${template}: pause resets measurement without clearing tempo`, () => {
    const { component, tap } = setup();
    tap(0); tap(500); tap(5001);
    assert.equal(component.tapTimes.length, 1);
    assert.equal(component.$refs.orgTempo.value, 120);
    tap(6001);
    assert.equal(component.$refs.orgTempo.value, 60);
  });

  test(`${template}: duplicate clicks are ignored and history is bounded`, () => {
    const { component, tap } = setup();
    tap(0); tap(0); tap(100);
    assert.equal(component.tapTimes.length, 1);
    assert.equal(component.$refs.orgTempo.value, '90');
    for (let i = 1; i <= 20; i++) tap(i * 500);
    assert.equal(component.tapTimes.length, 8);
    assert.equal(component.$refs.orgTempo.value, 120);
    for (let i = 1; i <= 8; i++) tap(10000 + i * 1000);
    assert.equal(component.tapTimes.length, 8);
    assert.equal(component.$refs.orgTempo.value, 60);
  });

  test(`${template}: slow rhythms and separate form instances`, () => {
    const first = setup();
    const second = setup();
    first.tap(0); first.tap(3000);
    assert.equal(first.component.$refs.orgTempo.value, 20);
    assert.equal(second.component.tapTimes.length, 0);
    assert.equal(second.component.$refs.orgTempo.value, '90');
  });
}
