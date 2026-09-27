/* Kept outside Phoenix's managed tree so live patches cannot erase the card. */
(() => {
  'use strict';
  const card = document.createElement('section');
  card.id = 'operator-summary';
  card.setAttribute('aria-labelledby', 'operator-summary-title');
  function add(tag, id, text, parent = card) {
    const node = document.createElement(tag);
    node.id = id;
    node.textContent = text;
    parent.appendChild(node);
    return node;
  }
  add('p', 'operator-eyebrow', 'FROM THE AGENT');
  add('h2', 'operator-summary-title', 'What’s happening');
  const notice = add('p', 'operator-notice', 'Reading the latest progress note…');
  notice.setAttribute('role', 'status');
  const content = add('div', 'operator-content', '');
  const current = add('p', 'operator-current', '', content);
  const why = add('p', 'operator-why', '', content);
  const milestone = add('p', 'operator-milestone', '', content);
  const goal = add('p', 'operator-goal', '', content);
  const next = add('p', 'operator-next', '', content);
  const blocker = add('p', 'operator-blocker', '', content);
  const footer = add('div', 'operator-footer', '');
  const times = add('span', 'operator-times', 'Refreshes every five minutes.', footer);
  const issue = add('a', 'operator-issue', '', footer);
  const program = add('a', 'operator-program', 'Project workpad', footer);
  program.href = 'https://github.com/pabloxrl/mtg-lab/issues/7';
  document.body.prepend(card);
  let loading = false;
  function text(node, value) {
    node.textContent = value || '';
    node.hidden = !value;
  }
  function date(value) { return new Date(value).toLocaleString(); }
  async function refresh() {
    if (loading) return;
    loading = true;
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), 8000);
    try {
      const response = await fetch('/operator-summary.json', {cache: 'no-store', signal: controller.signal});
      if (!response.ok) throw new Error('unavailable');
      const data = await response.json();
      if (data.version !== 1 || typeof data.current !== 'string') throw new Error('invalid summary');
      text(current, data.current);
      text(why, data.why);
      text(milestone, data.milestone_goal ? `${data.milestone}: ${data.milestone_goal}` : null);
      text(goal, `The bigger goal: ${data.project_goal}`);
      text(next, data.next ? `Next: ${data.next}` : null);
      text(blocker, data.blocker ? `What’s in the way: ${data.blocker}` : null);
      const messages = {
        fresh: 'Latest agent note',
        stale: 'This note is more than five minutes old. The agent has not posted a newer one; its current activity may have changed.',
        missing: 'Waiting for the agent’s first note for this session.',
        unavailable: 'No current agent note is available.'
      };
      text(notice, messages[data.freshness] || messages.unavailable);
      card.dataset.freshness = data.freshness;
      times.textContent = (data.report_updated_at ? `Agent wrote this ${date(data.report_updated_at)}. ` : '') +
        `Checked ${date(data.checked_at)}. Refreshes every five minutes.`;
      issue.hidden = !Number.isSafeInteger(data.issue) || data.issue < 1;
      if (!issue.hidden) {
        issue.href = `https://github.com/pabloxrl/mtg-lab/issues/${data.issue}`;
        issue.textContent = `Open task #${data.issue}`;
      }
    } catch (_) {
      notice.textContent = 'Couldn’t refresh this update. Anything shown below is the last information received; I’ll try again in five minutes.';
      card.dataset.freshness = 'unavailable';
    } finally {
      clearTimeout(timer);
      loading = false;
    }
  }
  refresh();
  setInterval(refresh, 300000);
  document.addEventListener('visibilitychange', () => { if (!document.hidden) refresh(); });
})();
