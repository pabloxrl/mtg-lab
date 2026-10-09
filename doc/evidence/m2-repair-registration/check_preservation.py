"""Read-only audit of the bounded #252 amendment against its inspected base."""
import hashlib
from functools import lru_cache
import json
from pathlib import Path
import subprocess

BASE = '522b7da8b86cd412c03c709bf88d8540c0385e16'
ROOT = Path(__file__).resolve().parents[3]


def original(path):
    return subprocess.check_output(['git', 'show', f'{BASE}:{path}'], cwd=ROOT)


def read(path):
    return json.loads((ROOT / path).read_text())


manifest = 'doc/programs/rfc-0002.json'
ledger = 'doc/programs/rfc-0002-requirements.json'
before, after = json.loads(original(manifest)), read(manifest)
assert {k: v for k, v in before.items() if k != 'tasks'} == {
    k: v for k, v in after.items() if k != 'tasks'}
tasks = {t['issue']: t for t in after['tasks']}
extra = {23: [254], 210: [255], 212: [256, 257], 217: [254]}
for task in before['tasks']:
    expected = dict(task, depends_on=task['depends_on'] + extra.get(task['issue'], []))
    assert tasks[task['issue']] == expected, task['issue']
new = set(tasks) - {t['issue'] for t in before['tasks']}
assert new == {252, 254, 255, 256, 257}
assert tasks[252]['kind'] == 'operations' and tasks[252]['requirements'] == []
assert tasks[252]['depends_on'] == [80]
for issue in new - {252}:
    assert {252, 80} <= set(tasks[issue]['depends_on'])
    assert not (set(tasks[issue]['depends_on']) & (new - {252}))


@lru_cache(None)
def ancestors(issue):
    return set(tasks[issue]['depends_on']).union(
        *(ancestors(dep) for dep in tasks[issue]['depends_on']))


assert {t['issue'] for t in after['tasks']
        if t['milestone'] == 'M2' and t['kind'] != 'gate'} <= ancestors(26)
assert all(22 in ancestors(issue) for issue in new)
old_ledger, new_ledger = json.loads(original(ledger)), read(ledger)
assert {k: v for k, v in old_ledger.items() if k != 'blocks'} == {
    k: v for k, v in new_ledger.items() if k != 'blocks'}
assert len(old_ledger['blocks']) == len(new_ledger['blocks']) == 44
for old, current in zip(old_ledger['blocks'], new_ledger['blocks']):
    assert {k: v for k, v in old.items() if k != 'owners'} == {
        k: v for k, v in current.items() if k != 'owners'}
    assert current['owners'] == old['owners'] + [
        n for n in sorted(new) if old['id'] in tasks[n]['requirements']]
catalog_path = 'doc/testing/capability-test-plan.json'
for path in [catalog_path, 'doc/rfcs/0001-project-charter.md',
             'doc/rfcs/0002-first-mvp.md', 'doc/rfcs/0003-data-driven-cards.md', 'README.md']:
    assert (ROOT / path).read_bytes() == original(path), path
crosswalk_path = 'doc/programs/m2-test-crosswalk.md'
rows = lambda text: [line for line in text.splitlines() if line.startswith('|')]
assert rows(original(crosswalk_path).decode()) == rows((ROOT / crosswalk_path).read_text())
crosswalk = read('doc/evidence/m2-repair-registration/crosswalk.json')
catalog = {case['id']: case for case in read(catalog_path)['cases']}
assert crosswalk['catalog_sha256'] == hashlib.sha256(original(catalog_path)).hexdigest()
for repair in crosswalk['repairs']:
    assert repair['depends_on'] == tasks[repair['issue']]['depends_on']
    assert repair['partial_requirements'] == tasks[repair['issue']]['requirements']
    for case in repair['unchanged_catalog_cases']:
        assert case == catalog[case['id']]
print(json.dumps(dict(result='PASS', inspected_main=BASE, original_tasks=len(before['tasks']),
                     new_tasks=sorted(new), verbatim_blocks=44, unchanged_cases=len(catalog),
                     unchanged_m2_rows=len(rows(original(crosswalk_path).decode())) - 2,
                     claim='planning preservation only; no engine or M2 completion'), indent=2))
