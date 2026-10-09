// Exercise the actual browser script with a small DOM and clock test double.
const assert = require('node:assert/strict');
const test = require('node:test');
const fs = require('node:fs');
const vm = require('node:vm');
const source = fs.readFileSync(__dirname + '/summary.js', 'utf8');

function setup() {
  const nodes = new Map(), timers = [], listeners = {};
  function element(tag) {
    return {tag, textContent: '', hidden: false, dataset: {}, children: [],
      set id(id) { nodes.set(id, this); },
      setAttribute() {}, removeAttribute(name) {delete this[name];},
      appendChild(child) {this.children.push(child);}, prepend(child) {this.children.unshift(child);},
      replaceChildren(...children) {this.children = children;}};
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

test('parallel cards isolate notes, blockers, freshness and original issue links', async () => {
  const page = setup(); await flush();
  const base = page.get();
  page.set({...base, tasks: [
    {...base, state: 'running', blocker: 'Waiting for a reference build.'},
    {...base, issue: 71, state: 'running', current: 'Testing cleanup.', freshness: 'stale'},
    {...base, issue: 72, state: 'queued', current: 'Waiting for a worker.', freshness: 'unavailable'}
  ], queue: {status: 'available', count: 1, truncated: false}});
  await page.timers[0].fn();
  assert.equal(page.nodes.get('operator-tasks').children.length, 3);
  assert.equal(page.nodes.get('operator-content').hidden, true);
  assert.equal(page.nodes.get('operator-task-0-current').textContent, base.current);
  assert.equal(page.nodes.get('operator-task-1-current').textContent, 'Testing cleanup.');
  assert.equal(page.nodes.get('operator-task-0-blocker').hidden, false);
  assert.equal(page.nodes.get('operator-task-1-blocker').hidden, true);
  assert.equal(page.nodes.get('operator-task-1').dataset.freshness, 'stale');
  assert.equal(page.nodes.get('operator-task-2-state').textContent, 'Queued');
  assert.equal(page.nodes.get('operator-task-1-issue').href, 'https://github.com/pabloxrl/mtg-lab/issues/71');
  page.set({...page.get(), tasks: [{...base, issue: 71, state: 'running', current: 'Finished cleanup.', blocker: null}]});
  await page.timers[0].fn();
  assert.equal(page.nodes.get('operator-tasks').children.length, 1);
  assert.equal(page.nodes.get('operator-task-0-blocker').hidden, true);
  assert.equal(page.nodes.get('operator-task-0-current').textContent, 'Finished cleanup.');
  assert.equal(page.nodes.get('operator-task-0-issue').href, 'https://github.com/pabloxrl/mtg-lab/issues/71');
});

test('queue failure is explicit and task prose never provides executable links', async () => {
  const page = setup(); await flush();
  const base = page.get();
  page.set({...base, tasks: [{...base, state: 'blocked', issue: 'javascript:attack()',
    current: '<script>private()</script>', issue_url: 'https://evil.example/', blocker: null}],
    queue: {status: 'unavailable', count: null}});
  await page.timers[0].fn();
  assert.match(page.nodes.get('operator-queue').textContent, /could not be checked/);
  assert.equal(page.nodes.get('operator-task-0-current').textContent, '<script>private()</script>');
  assert.equal(page.nodes.get('operator-task-0-issue').hidden, true);
  assert.equal(page.nodes.get('operator-task-0-issue').href, undefined);
  assert.equal(page.nodes.get('operator-task-0-state').textContent, 'Needs attention');
  const cards = page.nodes.get('operator-tasks').children;
  page.fail(); await page.timers[0].fn();
  assert.equal(page.nodes.get('operator-tasks').children, cards);
  assert.match(page.nodes.get('operator-notice').textContent, /last information received/);
});

test('queue truncation and unknown worker state stay visible', async () => {
  const page = setup(); await flush();
  page.set({...page.get(), state: 'unavailable', tasks: [],
    queue: {status: 'available', count: 200, truncated: true}});
  await page.timers[0].fn();
  assert.match(page.nodes.get('operator-queue').textContent, /partially shown/);
  assert.match(page.nodes.get('operator-notice').textContent, /Worker status is unavailable/);
});

test('idle ready queue coexists with GitHub controls and historical safe notes', async () => {
  const page = setup(); await flush();
  page.set({...page.get(), state: 'idle', tasks: [
    {issue: 23, state: 'controlled', github_controls: ['blocked', 'held'], github_controls_complete: true,
      freshness: 'stale', report_updated_at: '2026-09-27T12:00:00Z',
      previous_note: {current: '<script>attack()</script>', why: 'Games.', next: 'Checks.', blocker: null}}
  ], queue: {status: 'available', count: 0},
    tracker: {blocked: {status: 'available', count: 1}, held: {status: 'available', count: 1}}});
  await page.timers[0].fn();
  assert.match(page.nodes.get('operator-queue').textContent, /^0 open tasks/);
  assert.match(page.nodes.get('operator-tracker').textContent, /blocked tasks: 1/);
  assert.match(page.nodes.get('operator-task-0-controls').textContent, /blocked and held/);
  assert.match(page.nodes.get('operator-task-0-previous').textContent, /<script>attack\(\)<\/script>/);
  assert.match(page.nodes.get('operator-task-0-freshness').textContent, /more than five minutes old/);
  assert.equal(page.nodes.get('operator-task-0-issue').href, 'https://github.com/pabloxrl/mtg-lab/issues/23');
  page.set({...page.get(), tracker: {blocked: {status: 'unavailable', count: null}, held: {status: 'truncated', count: 200}},
    tasks: [{issue: 9999999999, state: 'running', github_controls: ['held'], freshness: 'missing'}]});
  await page.timers[0].fn();
  assert.match(page.nodes.get('operator-tracker').textContent, /count is unknown/);
  assert.match(page.nodes.get('operator-tracker').textContent, /at least 200/);
  assert.match(page.nodes.get('operator-task-0-freshness').textContent, /Open the task for the reason/);
  assert.equal(page.nodes.get('operator-task-0-state').textContent, 'Running');
  assert.equal(page.nodes.get('operator-task-0-issue').href, undefined);
});
