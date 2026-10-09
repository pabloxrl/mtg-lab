"""Rerun the bounded GH-209 reference pack and publish its exact catalog receipt.

Invoke through Symphony's shared heavy-work lock. Each child executes actual
pinned XMage and native rules (combat's native case runs in normal torture).
An error in any child prevents a joined agreement receipt.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
CATALOG = ROOT / 'doc/evidence/m2-cost-reference/catalog.json'
PACKS = ('creature_mana', 'haste', 'thrill', 'surprise', 'token', 'cast_trigger', 'combat', 'm2_cost')


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate_catalog(rows):
    plan = json.loads((ROOT / 'doc/testing/capability-test-plan.json').read_text())
    original = {case['id']: case for case in plan['cases']}
    assigned = set(re.findall(r'\| `([^`]+)` \| #[0-9]+ \| #209 \|',
                             (ROOT / 'doc/programs/m2-test-crosswalk.md').read_text()))
    actual = [row['catalog']['id'] for row in rows]
    if len(actual) != len(set(actual)) or set(actual) != assigned or len(actual) != 56:
        raise ValueError('missing, duplicate or extra assigned catalog case')
    sources = '\n'.join(p.read_text() for p in (ROOT / 'crates').rglob('*.rs'))
    for row in rows:
        if row['catalog'] != original[row['catalog']['id']]:
            raise ValueError('catalog expectation or original ownership changed')
        if row['pack'] not in PACKS or not row['reference_cases'] or not row['native_tests']:
            raise ValueError('missing executable mapping')
        for name in row['native_tests']:
            if f'fn {name}(' not in sources:
                raise ValueError('mapped native test absent: ' + name)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cache', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    receipt = output / 'receipt.json'
    receipt.unlink(missing_ok=True)
    rows = json.loads(CATALOG.read_text())
    validate_catalog(rows)
    runs = {}
    for pack in PACKS:
        child = output / pack
        result = child if pack == 'combat' else child / 'receipt.json'
        result.unlink(missing_ok=True)
        command = [sys.executable, str(ROOT / f'scripts/{pack}_reference.py'),
                   '--cache', str(args.cache.resolve()), '--output', str(child)]
        with (output / f'{pack}.log').open('w') as log:
            subprocess.run(command, cwd=ROOT, stdin=subprocess.DEVNULL,
                           stdout=log, stderr=subprocess.STDOUT, check=True, timeout=900)
        observed = json.loads(result.read_text())
        if observed.get('status') != 'agreed':
            raise ValueError('reference did not agree: ' + pack)
        if pack == 'combat':
            observations = observed['checkpoints']
            native_log = output / 'combat-native.log'
            with native_log.open('w') as log:
                subprocess.run(['cargo', 'test', '-p', 'mtg-core', '--locked', '--lib',
                                'game::combat::tests::combat_shared_xmage_reference_checkpoints',
                                '--', '--exact'], cwd=ROOT, stdin=subprocess.DEVNULL,
                               stdout=log, stderr=subprocess.STDOUT, check=True, timeout=180)
            if '1 passed; 0 failed' not in native_log.read_text():
                raise ValueError('native combat test did not execute')
        elif pack == 'token':
            observations = observed['checkpoints']
        else:
            observations = json.loads((child / '0/native.json').read_text())
        for row in rows:
            if row['pack'] == pack and not set(row['reference_cases']) <= set(observations):
                raise ValueError('mapped reference case was not executed: ' + row['catalog']['id'])
        runs[pack] = {'receipt': str(result.relative_to(output)), 'sha256': digest(result)}
        if pack == 'combat':
            runs[pack]['native_log_sha256'] = digest(native_log)
        print(f'{pack}: actual pinned reference passed', flush=True)
    receipt.write_text(json.dumps({
        'status': 'agreed', 'issue': 209, 'assigned_cases': 56,
        'catalog_sha256': digest(CATALOG), 'packs': runs,
        'holdout': 'holdout-two-archers-discard-surprise',
        'limitations': 'Selected synthetic reference positions, not full games or whole-pool qualification. '
                       'Native raw API rejection, snapshots, normal-reset replay and capture remain separate '
                       'mandatory torture checks. Catalog boundary notes identify the reference observations.'
    }, indent=2) + '\n')


if __name__ == '__main__':
    main()
