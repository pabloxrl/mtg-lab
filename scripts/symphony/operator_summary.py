"""Read-only, bounded operator summaries. Never read transcripts or command output."""
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import urllib.request

REFRESH_SECONDS = 300
MAX_REPORT_BYTES = 16384
MAX_QUEUE_ISSUES = 200
PROJECT_GOAL = ('Verify the frozen red and green toy decks through complete games and reproducible '
                'replays of games played by Forge and XMage AI players, with independent rules checks.')
MILESTONES = {
    'M0': 'Establish the rules, card lists and independent checks that later engine work can rely on.',
    'M1': 'Get a small set of cards through complete games, with correct choices, replays and recorded results.',
    'M2': 'Make every card in the agreed first pool work correctly, then measure how fast games run.',
}
FIELDS = {'version', 'issue', 'updated_at', 'current', 'why', 'next', 'blocker'}


def timestamp(value):
    if not isinstance(value, str):
        raise ValueError('timestamp must be a string')
    parsed = datetime.fromisoformat(value.replace('Z', '+00:00'))
    if parsed.tzinfo is None:
        raise ValueError('timestamp must include its timezone')
    return parsed


def validate_report(report):
    if not isinstance(report, dict) or set(report) != FIELDS:
        raise ValueError('unexpected report fields')
    if type(report['version']) is not int or report['version'] != 1:
        raise ValueError('unsupported report version')
    if type(report['issue']) is not int or report['issue'] <= 0:
        raise ValueError('invalid issue')
    timestamp(report['updated_at'])
    if len(report['updated_at']) > 128:
        raise ValueError('timestamp too long')
    for key in ('current', 'why', 'next', 'blocker'):
        value = report[key]
        if key == 'blocker' and value is None:
            continue
        if not isinstance(value, str) or not value.strip() or len(value) > 700:
            raise ValueError('report sentences must be nonempty and at most 700 characters')
        if any(ord(char) < 32 for char in value):
            raise ValueError('report sentences must be plain single-line text')
    return report


def issue_number(row):
    value = row.get('issue_identifier', '')
    match = re.fullmatch(r'GH-([1-9][0-9]{0,8})', value) if isinstance(value, str) else None
    return int(match.group(1)) if match else None


def _summarize_one(state, report=None, milestone=None, now=None):
    now = now or datetime.now(timezone.utc)
    result = dict(version=1, checked_at=now.isoformat(), refresh_seconds=REFRESH_SECONDS,
                  issue=None, milestone=None, milestone_goal=None, project_goal=PROJECT_GOAL,
                  report_updated_at=None, freshness='unavailable', state='unavailable',
                  current='I cannot read the agent’s status right now.', why=None,
                  next='The dashboard will try again in five minutes.', blocker=None)
    if not isinstance(state, dict) or any(not isinstance(state.get(k), list)
                                         for k in ('running', 'retrying', 'blocked')):
        return result
    rows = state['running'] or state['retrying'] or state['blocked']
    if not rows:
        result.update(state='idle', current='No agent is running right now.',
                      next='Check the program workpad for the next task. An idle worker does not mean the milestone is finished.')
        return result
    row = rows[0]
    if not isinstance(row, dict):
        return result
    issue = issue_number(row)
    result.update(issue=issue, milestone=milestone if milestone in MILESTONES else None,
                  milestone_goal=MILESTONES.get(milestone))
    if not state['running']:
        retry = bool(state['retrying'])
        result.update(state='retrying' if retry else 'blocked',
                      current='The agent is waiting to retry its task.' if retry else 'The agent has stopped and needs attention.',
                      next='Open the task for the recorded reason and the latest plan.')
        return result  # A previous running note cannot override a stop/retry.
    result.update(state='running', freshness='missing',
                  current='The agent is working, but has not left a current progress note yet.',
                  next='A note should appear at the next five-minute refresh. The task workpad has the detailed plan.')
    try:
        validate_report(report)
        updated = timestamp(report['updated_at'])
        started = timestamp(row.get('started_at'))
        if issue is None or report['issue'] != issue or updated < started or updated > now:
            return result
    except (ValueError, TypeError, KeyError):
        return result
    result.update({key: report[key] for key in ('current', 'why', 'next', 'blocker')})
    result.update(report_updated_at=updated.isoformat(),
                  freshness='stale' if (now - updated).total_seconds() > REFRESH_SECONDS else 'fresh')
    return result


def summarize(state, report=None, milestone=None, now=None, *, reports=None, milestones=None, queue=None, controls=None):
    """Keep the first-worker contract, plus an isolated summary for every task.

    Running takes precedence over retrying, blocked and the ready queue during
    tracker transitions. Queue membership alone never claims a task is running.
    """
    now = now or datetime.now(timezone.utc)
    reports, milestones = dict(reports or {}), dict(milestones or {})
    valid = isinstance(state, dict) and all(isinstance(state.get(k), list)
                                          for k in ('running', 'retrying', 'blocked'))
    if valid:
        primary = state['running'] or state['retrying'] or state['blocked']
        issue = issue_number(primary[0]) if primary and isinstance(primary[0], dict) else None
        if issue is not None:
            if report is not None:
                reports.setdefault(issue, report)
            if milestone is not None:
                milestones.setdefault(issue, milestone)
            report, milestone = reports.get(issue), milestones.get(issue)
    result = _summarize_one(state, report, milestone, now)
    tasks, seen = [], set()
    if valid:
        for kind in ('running', 'retrying', 'blocked'):
            for row in state[kind]:
                issue = issue_number(row) if isinstance(row, dict) else None
                if issue is not None and issue in seen:
                    continue
                if issue is not None:
                    seen.add(issue)
                single = dict(running=[], retrying=[], blocked=[])
                single[kind] = [row]
                tasks.append(_summarize_one(single, reports.get(issue), milestones.get(issue), now))
    available = (isinstance(queue, dict) and queue.get('status') == 'available'
                 and isinstance(queue.get('issues'), list))
    ready = []
    if available:
        ready = list(dict.fromkeys(number for number in queue['issues']
                     if type(number) is int and 0 < number <= 999999999))[:MAX_QUEUE_ISSUES]
        for issue in ready:
            if issue in seen:
                continue
            task = _summarize_one(None, now=now)
            task.update(issue=issue, state='queued' if valid else 'ready',
                        current='This task is ready and waiting for a worker.' if valid else
                                'This task is marked ready; worker status is unavailable.',
                        next='Symphony will dispatch it when capacity and its prerequisites allow.',
                        milestone=milestones.get(issue) if milestones.get(issue) in MILESTONES else None,
                        milestone_goal=MILESTONES.get(milestones.get(issue)))
            tasks.append(task)
    tracker = {}
    controlled = {}
    for kind in ('blocked', 'held'):
        source = (controls or {}).get(kind)
        ok = (isinstance(source, dict) and source.get('status') == 'available'
              and isinstance(source.get('issues'), list))
        numbers = list(dict.fromkeys(n for n in source['issues']
                       if type(n) is int and 0 < n <= 999999999))[:MAX_QUEUE_ISSUES] if ok else []
        truncated = bool(source.get('truncated')) if ok else False
        tracker[kind] = dict(status=('truncated' if truncated else 'available') if ok else 'unavailable',
                             count=len(numbers) if ok else None)
        for number in numbers:
            controlled.setdefault(number, []).append(kind)
    by_issue = {task['issue']: task for task in tasks}
    for number in sorted(controlled):
        if number not in by_issue:
            task = _summarize_one(None, now=now)
            task.update(issue=number, state='controlled',
                        milestone=milestones.get(number) if milestones.get(number) in MILESTONES else None,
                        milestone_goal=MILESTONES.get(milestones.get(number)))
            tasks.append(task)
            by_issue[number] = task
    for task in tasks:
        number = task['issue']
        task['controller_state'] = (task['state'] if task['state'] in ('running', 'retrying', 'blocked')
                                    else 'none' if valid else 'unavailable')
        task['github_controls'] = controlled.get(number, [])
        task['github_controls_complete'] = all(s['status'] == 'available' for s in tracker.values())
        if not task['github_controls'] or task['state'] in ('running', 'retrying', 'blocked'):
            continue
        task.update(state='controlled', freshness='missing',
                    current='GitHub marks this task blocked or held. No operator note explains the reason here.',
                    next='Open the task for its recorded reason and latest plan.')
        try:
            note = validate_report(reports.get(number))
            updated = timestamp(note['updated_at'])
            if note['issue'] != number or updated > now:
                continue
        except (ValueError, TypeError, KeyError):
            continue
        # Historical prose never replaces the current GitHub control state.
        task['previous_note'] = {k: note[k] for k in ('current', 'why', 'next', 'blocker')}
        task.update(current='GitHub marks this task blocked or held. The last operator note is shown below; it may predate this control.',
                    report_updated_at=updated.isoformat(),
                    freshness='stale' if (now - updated).total_seconds() > REFRESH_SECONDS else 'fresh')
    result['tracker'] = tracker
    result['tasks'] = tasks
    result['queue'] = dict(status='available' if available else 'unavailable',
                           count=len(ready) if available else None,
                           truncated=bool(queue.get('truncated')) if available else False)
    return result


def fetch_queue():
    """Compatibility entry point for the ready-label source."""
    return fetch_label('agent-ready')


def fetch_label(label):
    """Read only bounded numeric identities for an allowlisted task control."""
    if label not in ('agent-ready', 'agent-blocked', 'agent-held'):
        raise ValueError('unsupported task control')
    try:
        process = subprocess.run(
            ['gh', 'issue', 'list', '--repo', 'pabloxrl/mtg-lab', '--state', 'open',
             '--label', label, '--limit', str(MAX_QUEUE_ISSUES + 1), '--json', 'number'],
            stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
            timeout=3, check=False)
        if process.returncode or len(process.stdout) > MAX_REPORT_BYTES:
            raise ValueError('queue unavailable')
        rows = json.loads(process.stdout)
        if not isinstance(rows, list) or len(rows) > MAX_QUEUE_ISSUES + 1:
            raise ValueError('invalid queue')
        if any(not isinstance(row, dict) or set(row) != {'number'} or
               type(row['number']) is not int or not 0 < row['number'] <= 999999999 for row in rows):
            raise ValueError('invalid queue issue')
        return dict(status='available', issues=sorted({row['number'] for row in rows})[:MAX_QUEUE_ISSUES],
                    truncated=len(rows) > MAX_QUEUE_ISSUES)
    except (OSError, ValueError, TypeError, subprocess.TimeoutExpired):
        return dict(status='unavailable', issues=[], truncated=False)


def bounded_json(path, limit):
    """Do not follow workspace symlinks into credentials or unrelated files."""
    if path.resolve() != path.absolute():
        raise ValueError('symlinked source is not a progress report')
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    with os.fdopen(fd, 'rb') as stream:
        if not stat.S_ISREG(os.fstat(stream.fileno()).st_mode):
            raise ValueError('source must be a regular file')
        raw = stream.read(limit + 1)
    if len(raw) > limit:
        raise ValueError('source too large')
    return json.loads(raw)


def collect(workspaces=None, control=None):
    workspaces = workspaces or Path.home() / '.local/share/mtg-lab-symphony/workspaces'
    control = control or Path('/opt/mtg-lab')
    try:
        with urllib.request.urlopen('http://127.0.0.1:4318/api/v1/state', timeout=4) as response:
            raw = response.read(262145)
        if len(raw) > 262144:
            raise ValueError('state too large')
        state = json.loads(raw)
        if not isinstance(state, dict):
            raise ValueError('invalid state')
    except (OSError, ValueError, TypeError):
        state = None
    # Parallel bounded reads preserve the existing endpoint/browser time budget.
    with ThreadPoolExecutor(max_workers=3) as pool:
        ready_fetch = pool.submit(fetch_queue)
        blocked_fetch = pool.submit(fetch_label, 'agent-blocked')
        held_fetch = pool.submit(fetch_label, 'agent-held')
        queue = ready_fetch.result()
        controls = dict(blocked=blocked_fetch.result(), held=held_fetch.result())
    controlled = set(n for source in controls.values() for n in source['issues'])
    reports, milestones, issues, running = {}, {}, set(queue['issues']) | controlled, set()
    if isinstance(state, dict):
        for kind in ('running', 'retrying', 'blocked'):
            rows = state.get(kind)
            if isinstance(rows, list):
                for row in rows:
                    issue = issue_number(row) if isinstance(row, dict) else None
                    if issue:
                        issues.add(issue)
                        if kind == 'running':
                            running.add(issue)
    for issue in sorted(issues):
        workspace = workspaces / f'GH-{issue}'
        if issue in running or issue in controlled:
            try:
                reports[issue] = bounded_json(workspace / '.agent-artifacts/operator-summary.json', MAX_REPORT_BYTES)
            except (OSError, ValueError):
                pass
        for base in (workspace, control):
            try:
                program = bounded_json(base / 'doc/programs/rfc-0002.json', 262144)
                milestone = next(t['milestone'] for t in program['tasks'] if t['issue'] == issue)
                if milestone not in MILESTONES:
                    raise ValueError('unknown milestone')
                milestones[issue] = milestone
                break
            except (OSError, ValueError, KeyError, TypeError, StopIteration):
                pass
    return summarize(state, reports=reports, milestones=milestones, queue=queue, controls=controls)


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path != '/operator-summary.json':
            self.send_error(404)
            return
        payload = json.dumps(collect(), ensure_ascii=False).encode()
        self.send_response(200)
        self.send_header('Content-Type', 'application/json; charset=utf-8')
        self.send_header('Cache-Control', 'no-store')
        self.send_header('Content-Length', str(len(payload)))
        self.end_headers()
        self.wfile.write(payload)

    def log_message(self, *args):
        pass  # Never log report text or request headers.


if __name__ == '__main__':
    ThreadingHTTPServer(('127.0.0.1', 4319), Handler).serve_forever()
