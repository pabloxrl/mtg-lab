// Exercise the actual browser script with a small DOM and clock test double.
const assert = require('node:assert/strict');
const test = require('node:test');
const fs = require('node:fs');
const vm = require('node:vm');
const source = fs.readFileSync(__dirname + '/summary.js', 'utf8');

function setup() {
  const nodes = new Map(), timers = [], listeners = {};
  function element(tag) {
    return {tag, textContent: '', hidden: false, dataset: {},
      set id(id) { nodes.set(id, this); },
      setAttribute() {}, appendChild() {}, prepend() {}};
  }
  let value = {version: 1, current: 'I am testing spell responses.', why: 'Games need correct results.',
    next: 'Check both spells together.', blocker: null, milestone: 'M1', milestone_goal: 'Complete games.',
    project_goal: 'Trustworthy AI games.', issue: 70, freshness: 'fresh',
    checked_at: '2026-09-27T14:00:00Z', report_updated_at: '2026-09-27T13:59:00Z'};
  let fail = false, calls = 0;
  const context = {document: {createElement: element, body: element('body'), hidden: false,
      addEventListener(name, fn) { listeners[name] = fn; }},
    AbortController, setTimeout: () => 1, clearTimeout() {},
    setInterval(fn, ms) { timers.push({fn, ms}); },
    fetch: async (url, options) => {
      assert.equal(url, '/operator-summary.json');
      assert.equal(options.cache, 'no-store');
      calls++;
      if (fail) throw new Error('network');
      return {ok: true, json: async () => value};
    }};
  vm.runInNewContext(source, context);
  return {nodes, timers, listeners, context, set(v) {value = v;}, get: () => value,
    fail() {fail = true;}, calls: () => calls};
}
const flush = () => new Promise(resolve => setImmediate(resolve));

test('loads immediately, refreshes at five minutes and shows a human goal', async () => {
  const page = setup();
  await flush();
  assert.equal(page.calls(), 1);
  assert.equal(page.timers[0].ms, 300000);
  assert.equal(page.nodes.get('operator-current').textContent, 'I am testing spell responses.');
  assert.equal(page.nodes.get('operator-milestone').textContent, 'M1: Complete games.');
  assert.match(page.nodes.get('operator-goal').textContent, /AI games/);
  page.set({...page.get(), current: 'The response tests pass; I am checking the full suite.'});
  await page.timers[0].fn();
  assert.equal(page.calls(), 2);
  assert.match(page.nodes.get('operator-current').textContent, /full suite/);
});

test('hostile markup stays text; link destinations cannot come from report prose', async () => {
  const page = setup(); await flush();
  page.set({...page.get(), current: '<img src=x onerror=alert(1)>', why: '<script>attack()</script>',
    issue: 'javascript:alert(1)', issue_url: 'javascript:alert(2)'});
  await page.timers[0].fn();
  assert.equal(page.nodes.get('operator-current').textContent, '<img src=x onerror=alert(1)>');
  assert.equal(page.nodes.get('operator-issue').hidden, true);
  assert.equal(page.nodes.get('operator-why').textContent, '<script>attack()</script>');
  assert.equal(source.includes('innerHTML'), false);
});

test('old reports and connection failures never look freshly authored', async () => {
  const page = setup(); await flush();
  page.set({...page.get(), freshness: 'stale'});
  await page.timers[0].fn();
  assert.match(page.nodes.get('operator-notice').textContent, /more than five minutes old/);
  const before = page.nodes.get('operator-times').textContent;
  page.fail(); await page.timers[0].fn();
  assert.match(page.nodes.get('operator-notice').textContent, /Couldn’t refresh/);
  assert.equal(page.nodes.get('operator-times').textContent, before);
  assert.equal(page.nodes.get('operator-summary').dataset.freshness, 'unavailable');
});

test('changing task clears prior blocker; returning to the tab refreshes', async () => {
  const page = setup(); await flush();
  page.set({...page.get(), blocker: 'The reference service is unavailable.'});
  await page.timers[0].fn();
  assert.equal(page.nodes.get('operator-blocker').hidden, false);
  page.set({...page.get(), issue: 71, blocker: null});
  page.listeners.visibilitychange(); await flush();
  assert.equal(page.nodes.get('operator-blocker').hidden, true);
  assert.equal(page.nodes.get('operator-issue').href, 'https://github.com/pabloxrl/mtg-lab/issues/71');
});
