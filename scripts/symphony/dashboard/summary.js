/* Kept outside Phoenix's managed tree so live patches cannot erase the cards. */
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
  const goal = add('p', 'operator-goal', '');
  const next = add('p', 'operator-next', '', content);
  const blocker = add('p', 'operator-blocker', '', content);
  const queue = add('p', 'operator-queue', '');
  const tasks = add('div', 'operator-tasks', '');
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
  const messages = {
    fresh: 'Latest agent note',
    stale: 'This note is more than five minutes old. The agent has not posted a newer one; its current activity may have changed.',
    missing: 'Waiting for the agent’s first note for this session.',
    unavailable: 'No current agent note is available.'
  };
  function link(node, number) {
    node.hidden = !Number.isSafeInteger(number) || number < 1;
    if (!node.hidden) {
      node.href = `https://github.com/pabloxrl/mtg-lab/issues/${number}`;
      node.textContent = `Open task #${number}`;
    } else {
      node.removeAttribute('href');
      node.textContent = '';
    }
  }
  function taskCard(task, index) {
    const node = document.createElement('article');
    node.id = `operator-task-${index}`;
    node.className = 'operator-task';
    const prefix = `operator-task-${index}-`;
    const states = {running: 'Running', retrying: 'Waiting to retry', blocked: 'Needs attention',
      queued: 'Queued', ready: 'Ready · worker status unavailable', unavailable: 'Status unavailable'};
    node.dataset.state = states[task.state] ? task.state : 'unavailable';
    node.dataset.freshness = task.freshness;
    add('h3', prefix + 'title', Number.isSafeInteger(task.issue) && task.issue > 0 ?
      `Task #${task.issue}` : 'Task status unavailable', node);
    add('p', prefix + 'state', states[task.state] || states.unavailable, node).className = 'operator-task-state';
    const fields = {
      current: task.current, why: task.why,
      milestone: task.milestone_goal ? `${task.milestone}: ${task.milestone_goal}` : null,
      next: task.next ? `Next: ${task.next}` : null,
      blocker: task.blocker ? `What’s in the way: ${task.blocker}` : null,
      freshness: task.state === 'running' ? (messages[task.freshness] || messages.unavailable) : null,
      time: task.report_updated_at ? `Agent wrote this ${date(task.report_updated_at)}.` : null
    };
    for (const [name, value] of Object.entries(fields)) {
      const field = add('p', prefix + name, '', node);
      field.className = 'operator-task-' + name;
      text(field, value);
    }
    link(add('a', prefix + 'issue', '', node), task.issue);
    return node;
  }
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
      text(notice, messages[data.freshness] || messages.unavailable);
      card.dataset.freshness = data.freshness;
      times.textContent = (data.report_updated_at ? `Agent wrote this ${date(data.report_updated_at)}. ` : '') +
        `Checked ${date(data.checked_at)}. Refreshes every five minutes.`;
      link(issue, data.issue);
      const multiple = Array.isArray(data.tasks);
      content.hidden = multiple;
      tasks.hidden = !multiple;
      queue.hidden = !multiple;
      if (multiple) {
        // Each response replaces the task cards, including cleared blockers and
        // removed tasks. Never reuse one worker's note for another worker.
        tasks.replaceChildren(...data.tasks.map(taskCard));
        issue.hidden = true;
        notice.textContent = data.state === 'unavailable' ? 'Worker status is unavailable.' :
          (data.tasks.length ? `${data.tasks.length} tasks visible.` : 'No active tasks are reported.');
        const available = data.queue && data.queue.status === 'available';
        queue.textContent = available ? (data.queue.truncated ?
          'Ready queue is partially shown; more tasks may be waiting. See the project workpad.' :
          `${data.queue.count} open tasks are marked ready on GitHub; tasks already assigned appear with their worker status.`) :
          'The ready queue could not be checked. Additional tasks may be waiting.';
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
