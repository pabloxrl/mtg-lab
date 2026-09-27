"""Read-only, bounded operator summaries. Never read transcripts or command output."""
from datetime import datetime, timezone
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import re
import stat
import urllib.request

REFRESH_SECONDS = 300
MAX_REPORT_BYTES = 16384
PROJECT_GOAL = ('Build a trustworthy Magic simulator where AI players can play reproducible games, '
                'learn from them, and leave a record we can inspect when something goes wrong.')
MILESTONES = {
    'M0': 'Establish the rules, card lists and independent checks that later engine work can rely on.',
    'M1': 'Get a small set of cards through complete games, with correct choices, replays and recorded results.',
    'M2': 'Make every card in the agreed first pool work correctly, then measure how fast games run.',
    'M3': 'Run many games together and connect them to Python training tools without losing or leaking game data.',
    'M4': 'Make the command-line tools and two-player interface work reliably without manual intervention.',
    'M5': 'Stress the finished system, compare it with independent engines, and prove it is ready to use.',
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


def summarize(state, report=None, milestone=None, now=None):
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
        rows = state.get('running') or state.get('retrying') or state.get('blocked') or []
        issue = issue_number(rows[0]) if isinstance(rows, list) and rows and isinstance(rows[0], dict) else None
    except (OSError, ValueError, TypeError):
        return summarize(None)
    report, milestone = None, None
    if issue:
        workspace = workspaces / f'GH-{issue}'
        try:
            report = bounded_json(workspace / '.agent-artifacts/operator-summary.json', MAX_REPORT_BYTES)
        except (OSError, ValueError):
            pass
        for base in (workspace, control):
            try:
                program = bounded_json(base / 'doc/programs/rfc-0002.json', 262144)
                milestone = next(t['milestone'] for t in program['tasks'] if t['issue'] == issue)
                if milestone not in MILESTONES:
                    raise ValueError('unknown milestone')
                break
            except (OSError, ValueError, KeyError, TypeError, StopIteration):
                milestone = None
    return summarize(state, report, milestone)


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
