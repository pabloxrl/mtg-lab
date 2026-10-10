"""One-shot #268 candidate preservation check; not an implementation gate."""
import copy
import hashlib
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / 'scripts'))
from check_program import validate

HERE = Path(__file__).resolve().parent
baseline = json.loads((HERE / 'preservation-baseline.json').read_text())
program = json.loads((ROOT / 'doc/programs/rfc-0002.json').read_text())
ledger = json.loads((ROOT / 'doc/programs/rfc-0002-requirements.json').read_text())
source = (ROOT / program['rfc']).read_bytes()
validate(program, ledger, source)
old = baseline['program']
assert len(old['tasks']) == 129
assert len(program['tasks']) == 143
for key in old.keys() - {'tasks'}:
    assert old[key] == program[key], key
current = {t['issue']: t for t in program['tasks']}
for task in old['tasks']:
    expected = copy.deepcopy(task)
    if task['issue'] == 24:
        expected['depends_on'].append(277)
    if task['issue'] == 25:
        expected['depends_on'].append(281)
    assert current[task['issue']] == expected, task['issue']
original_ids = {t['issue'] for t in old['tasks']}
assert [t for t in program['tasks'] if t['issue'] in original_ids] == [
    current[t['issue']] for t in old['tasks']]
for before, after in zip(baseline['ledger']['blocks'], ledger['blocks']):
    assert set(before['owners']) <= set(after['owners']), before['id']
    assert {k:v for k,v in before.items() if k != 'owners'} == {
        k:v for k,v in after.items() if k != 'owners'}, before['id']
for key in baseline['ledger'].keys() - {'blocks'}:
    assert baseline['ledger'][key] == ledger[key]
for path, digest in baseline['immutable_files'].items():
    assert hashlib.sha256((ROOT / path).read_bytes()).hexdigest() == digest, path
for row in json.loads((HERE / 'archive-hashes.json').read_text()):
    data = (HERE / 'originals' / row['path']).read_bytes()
    assert len(data) == row['bytes']
    assert hashlib.sha256(data).hexdigest() == row['sha256'], row['path']
for issue in range(269,282):
    assert 268 in current[issue]['depends_on']
assert current[268]['requirements'] == []
assert current[268]['depends_on'] == [80]
# Meaningful negative controls through the existing production graph validator.
for mutation in ('cycle', 'gate', 'owner'):
    p, l = copy.deepcopy(program), copy.deepcopy(ledger)
    by = {t['issue']: t for t in p['tasks']}
    if mutation == 'cycle':
        by[269]['depends_on'].append(277)
    elif mutation == 'gate':
        by[24]['depends_on'].remove(277)
    else:
        l['blocks'][0]['owners'].append(268)
    try:
        validate(p,l,source)
    except ValueError:
        pass
    else:
        raise AssertionError('negative graph control accepted: '+mutation)
print('PASS: 129 originals, 143-task DAG/gates, additive ledger, immutable sources/catalog, 11 byte archives; cycle/gate/owner negatives rejected.')
