"""Validate the separate current-pool M3 registry and plan one bounded handoff.

Read-only: no GitHub mutation or credential access. Workers verify live evidence
and re-fetch under the existing handoff lock before using a selection. A plan is
not authorization to apply labels without those checks (see the program guide).
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess

from check_program import positive_int, require

PATH = 'doc/programs/engine-validation-m3.json'
RFC = 'doc/rfcs/0003-data-driven-cards.md'
NAMESPACE = 'engine-validation-m3'


M3_PROFILE = dict(namespace=NAMESPACE, coordinator=241, milestone=7, stage='EV-M3',
                  prerequisites=[26, 241], source=RFC, prefix='EV3',
                  kinds=('implementation', 'gate'))


def validate(program, source, legacy, *, profile=None):
    profile = M3_PROFILE if profile is None else profile
    require(isinstance(program, dict), 'registry must be an object')
    require(set(program) == {'schema_version', 'program', 'parent_issue',
            'coordinator_issue', 'github_milestone', 'authorized_milestones',
            'prerequisites', 'rfc', 'rfc_sha256', 'requirements', 'tasks'},
            'unknown or missing registry fields')
    require(type(program['schema_version']) is int and program['schema_version'] == 1,
            'unsupported registry schema')
    require(program['program'] == profile['namespace'], 'wrong program namespace')
    require(type(program['parent_issue']) is int and program['parent_issue'] == 7,
            'wrong tracking parent')
    require(type(program['coordinator_issue']) is int and program['coordinator_issue'] == profile['coordinator'],
            'wrong registration authority')
    require(type(program['github_milestone']) is int and program['github_milestone'] == profile['milestone'],
            'wrong GitHub milestone')
    require(program['authorized_milestones'] == [profile['stage']], 'unauthorized milestone')
    require(program['prerequisites'] == profile['prerequisites']
            and all(positive_int(n) for n in program['prerequisites']),
            'missing verified M2/registration prerequisites')
    require(program['rfc'] == profile['source'] and isinstance(source, bytes)
            and program['rfc_sha256'] == hashlib.sha256(source).hexdigest(),
            'RFC content pin mismatch')
    require(legacy.get('authorized_milestones') == ['M0', 'M1', 'M2'],
            'legacy authorization changed')
    legacy_ids = {t['issue'] for t in legacy['tasks']}
    requirements = program['requirements']
    require(isinstance(requirements, list) and requirements, 'missing requirements')
    refs = {}
    for req in requirements:
        require(isinstance(req, dict) and set(req) == {'id', 'owner', 'deliverable', 'checks', 'exclusions'},
                'malformed requirement')
        require(isinstance(req['id'], str) and re.fullmatch(profile['prefix'] + r'-\d{3}', req['id']),
                'invalid requirement ID')
        require(req['id'] not in refs, 'duplicate requirement')
        require(positive_int(req['owner']), 'invalid owner')
        require(all(isinstance(req[k], str) and req[k].strip()
                    for k in ('deliverable', 'checks', 'exclusions')), 'empty contract')
        refs[req['id']] = req
    tasks = program['tasks']
    require(isinstance(tasks, list) and tasks, 'missing tasks')
    by_issue, keys = {}, set()
    for task in tasks:
        require(isinstance(task, dict) and set(task) == {
            'issue', 'key', 'milestone', 'kind', 'depends_on', 'requirements'}, 'malformed task')
        issue = task['issue']
        require(positive_int(issue) and issue not in {7, *profile['prerequisites']}, 'invalid task/parent issue')
        require(issue not in legacy_ids, 'legacy task cannot enter new namespace')
        require(issue not in by_issue, 'duplicate task')
        require(isinstance(task['key'], str) and re.fullmatch(r'[a-z][a-z_]*', task['key'])
                and task['key'] not in keys, 'invalid or duplicate task key')
        require(task['milestone'] == profile['stage'], 'unauthorized task milestone')
        require(task['kind'] in profile['kinds'], 'unknown task kind')
        deps, owned = task['depends_on'], task['requirements']
        require(isinstance(deps, list) and all(positive_int(n) for n in deps)
                and len(deps) == len(set(deps)), 'invalid/duplicate dependencies')
        require(issue not in deps, 'self dependency')
        require(isinstance(owned, list) and owned and all(isinstance(r, str) and r in refs for r in owned)
                and len(owned) == len(set(owned)), 'unknown/duplicate requirements')
        by_issue[issue] = task
        keys.add(task['key'])
    ancestors, visiting = {}, set()

    def visit(issue):
        require(issue in by_issue, 'unknown dependency')
        require(issue not in visiting, 'dependency cycle')
        if issue not in ancestors:
            visiting.add(issue)
            ancestors[issue] = set()
            for dep in by_issue[issue]['depends_on']:
                ancestors[issue].add(dep)
                ancestors[issue].update(visit(dep))
            visiting.remove(issue)
        return ancestors[issue]

    for issue in by_issue:
        visit(issue)
    gates = [t for t in tasks if t['kind'] == 'gate']
    require(len(gates) == 1, 'exactly one integrated gate required')
    gate = gates[0]
    implementation = set(by_issue) - {gate['issue']}
    require(implementation and implementation <= ancestors[gate['issue']],
            'gate omits implementation prerequisites')
    require(set(gate['requirements']) == set(refs), 'gate omits requirements')
    for ref, req in refs.items():
        require(req['owner'] in implementation and ref in by_issue[req['owner']]['requirements'],
                'missing requirement owner')
    for issue in implementation:
        require(set(by_issue[issue]['requirements']) == {r for r, v in refs.items() if v['owner'] == issue},
                'ownership disagreement')
    return by_issue


def labels(issue):
    return {label['name'] for label in issue['labels']}


def active(issue):
    return issue['state'] == 'open' and not labels(issue) & {
        'agent-held', 'agent-blocked', 'program-paused', 'program-cancelled'}


def receipt_valid(receipt):
    """Validate evidence shape; the worker must verify these facts from GitHub."""
    if not isinstance(receipt, dict):
        return False
    sha = receipt.get('merge_sha', '')
    return (isinstance(sha, str) and re.fullmatch('[0-9a-f]{40}', sha) is not None
            and receipt.get('ci_sha') == sha and receipt.get('ci_conclusion') == 'success'
            and receipt.get('review_verdict') == 'pass'
            and all(isinstance(receipt.get(k), str) and receipt[k].startswith(
                'https://github.com/pabloxrl/mtg-lab/') for k in
                ('acceptance', 'review', 'pull_request', 'main_ci', 'workpad')))


def select_successor(program, snapshot):
    """Plan at most one successor using a complete live snapshot under handoff lock.

No cached boolean completion flags: each dependency needs completion state and
structured verified receipts. The current successful task alone may still be open.
All repository ready/running claims count toward capacity, including other programs.
"""
    require(type(snapshot.get('current_success')) is bool, 'invalid current outcome')
    current = snapshot['current']
    issues = {i['number']: i for i in snapshot['issues']}
    require(len(issues) == len(snapshot['issues']), 'duplicate issue snapshot')
    coordinator = program['coordinator_issue']
    expected = {program['parent_issue'], current, *program['prerequisites']} | {t['issue'] for t in program['tasks']}
    require(expected <= set(issues), 'incomplete issue snapshot')
    require(current == coordinator or current in {t['issue'] for t in program['tasks']},
            'current task outside registry')
    require(active(issues[7]), 'parent paused/closed')
    require(active(issues[current]) and 'agent-ready' in labels(issues[current]),
            'current authorization withdrawn')
    receipts = snapshot['receipts']

    def complete(n):
        issue = issues[n]
        state_ok = issue['state'] == 'closed' and issue.get('state_reason') == 'completed'
        if n == current and snapshot['current_success'] is True:
            state_ok = True
        return state_ok and receipt_valid(receipts.get(str(n)))

    require(all(complete(n) for n in program['prerequisites'] if n != current),
            'unverified program prerequisite')
    if current == coordinator:
        require(complete(current), 'unverified registration')
    reservation = snapshot.get('reserved_successor')
    if reservation is not None:
        require(reservation in {t['issue'] for t in program['tasks']} and reservation != current,
                'invalid successor reservation')
        require(active(issues[reservation]) or complete(reservation),
                'reserved successor requires reconciliation')
        if complete(reservation):
            return {'successor': reservation, 'already_owned': True}
    claims = snapshot['open_claims']  # all pages, not only M3
    require(isinstance(claims, list) and len({i['number'] for i in claims}) == len(claims),
            'invalid claims snapshot')
    count = sum(i['number'] != current and i['state'] == 'open'
                and bool(labels(i) & {'agent-ready', 'agent-running'}) for i in claims)
    for task in program['tasks']:
        n = task['issue']
        issue = issues[n]
        if n == current or not active(issue):
            continue
        require(issue.get('milestone', {}).get('number') == program['github_milestone'],
                'task milestone mismatch')
        dependencies = task['depends_on']
        if task['kind'] == 'gate':
            dependencies = [t['issue'] for t in program['tasks'] if t['kind'] != 'gate']
        if not all(complete(d) for d in dependencies):
            continue
        if reservation is not None and n != reservation:
            continue
        events = snapshot['events'].get(str(n))
        require(isinstance(events, list), 'missing paginated label events')
        previously_ready = any(e.get('event') == 'labeled'
                               and e.get('label', {}).get('name') == 'agent-ready' for e in events)
        ready = 'agent-ready' in labels(issue)
        withdrawn = previously_ready and not ready
        grant = 'agent-resume-authorized' in labels(issue)
        if n == reservation:
            require(not withdrawn or grant, 'reserved successor withdrawal requires reconciliation')
            if ready:
                return {'successor': n, 'already_owned': True}
            require('agent-running' not in labels(issue) or (withdrawn and grant),
                    'reserved successor ownership requires reconciliation')
        elif labels(issue) & {'agent-ready', 'agent-running'}:
            continue
        if withdrawn and not grant:
            continue
        if count >= 3:
            return {'successor': None, 'reason': 'capacity'}
        return {'successor': n, 'already_owned': False}
    require(reservation is None, 'reserved successor dependencies require reconciliation')
    return {'successor': None, 'reason': 'dependencies, controls, ownership or completion'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--ref', help='read registry, RFC and legacy metadata from this Git ref')
    parser.add_argument('--snapshot', type=Path, help='verified live handoff inputs; read-only plan')
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]

    def read(path):
        if args.ref:
            return subprocess.check_output(['git', 'show', f'{args.ref}:{path}'], cwd=root)
        return (root / path).read_bytes()

    try:
        program = json.loads(read(PATH))
        validate(program, read(RFC), json.loads(read('doc/programs/rfc-0002.json')))
        if args.snapshot:
            print(json.dumps(select_successor(program, json.loads(args.snapshot.read_text()))))
        else:
            print('Engine validation M3 registry, ownership and integrated gate passed.')
    except (ValueError, KeyError, TypeError, OSError, subprocess.CalledProcessError) as error:
        parser.exit(1, f'M3 registry validation failed: {error}\n')


if __name__ == '__main__':
    main()
