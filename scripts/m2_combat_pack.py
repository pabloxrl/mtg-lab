"""Rerun GH-210's exact assigned native/XMage cases and bounded compositions.

Run through Symphony's shared heavy-work lock. Child failures, missing case
observations and source-stale receipts prevent publication of the joined receipt.
"""
import argparse
import importlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import xmage
from m2_combat_reference import require_current_receipt

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / 'fixtures/reference/m2-combat-pack.json'
PACKS = ('shivan', 'haste', 'trample', 'deathtouch', 'flying_reach',
         'invoker', 'surprise', 'm2_combat')


def validate_manifest(document):
    assigned = set(re.findall(r'\| `([^`]+)` \| #[0-9]+ \| #210 \|',
                             (ROOT / 'doc/programs/m2-test-crosswalk.md').read_text()))
    rows = document.get('cases', [])
    ids = [r['catalog']['id'] for r in rows]
    if document.get('issue') != 210 or len(ids) != 50 or len(set(ids)) != 50 or set(ids) != assigned:
        raise ValueError('manifest must preserve all 50 assigned catalog cases exactly once')
    catalog = {c['id']: c for c in json.loads((ROOT / 'doc/testing/capability-test-plan.json').read_text())['cases']}
    for row in rows:
        if row['catalog'] != catalog[row['catalog']['id']]:
            raise ValueError('catalog expectation or owner changed: ' + row['catalog']['id'])
        if not row.get('executions') or not row.get('boundary'):
            raise ValueError('missing executable mapping/boundary')
        for execution in row['executions']:
            if execution['pack'] not in PACKS:
                raise ValueError('unknown reference pack')
            module = importlib.import_module(execution['pack'] + '_reference')
            available = json.loads(module.EXPECTED.read_text())
            if not execution['cases'] or not set(execution['cases']) <= set(available):
                raise ValueError('mapped reference case absent: ' + row['catalog']['id'])
    return rows


def validate_run(pack, folder):
    module = importlib.import_module(pack + '_reference')
    receipt = json.loads((folder / 'receipt.json').read_text())
    expected = dict(fixture_sha256=xmage.sha(module.FIXTURE),
                    expected_sha256=xmage.sha(module.EXPECTED),
                    runner_sha256=xmage.sha(Path(module.__file__)),
                    bridge_sha256=xmage.sha(module.BRIDGE),
                    upstream_commit=json.loads((ROOT / 'references/xmage/pins.json').read_text())['upstream_commit'],
                    native_sources={str(p.relative_to(ROOT)): xmage.sha(p)
                                    for p in sorted((ROOT / 'crates/mtg-core/src').glob('**/*.rs' if pack in ('trample', 'deathtouch') else '*.rs'))})
    require_current_receipt(receipt, expected)
    canonical = json.loads(module.EXPECTED.read_text())
    if receipt.get('cases') != len(canonical) or len(receipt.get('runs', [])) != 2:
        raise ValueError('incomplete case/repetition count: ' + pack)
    for repeat in range(2):
        run = folder / str(repeat)
        native = json.loads((run / 'native.json').read_text())
        module.compare(native)
        observed = {p.stem: json.loads(p.read_text()) for p in run.glob('*.json') if p.name != 'native.json'}
        module.compare(observed)
        for engine in ('native', 'xmage'):
            if receipt['runs'][repeat][engine + '_log_sha256'] != xmage.sha(run / (engine + '.log')):
                raise ValueError('altered execution log: ' + pack)
    if not receipt.get('controls'):
        raise ValueError('missing negative controls: ' + pack)
    return dict(receipt=str((folder / 'receipt.json').name),
                sha256=xmage.sha(folder / 'receipt.json'), cases=sorted(canonical))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cache', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    (output / 'receipt.json').unlink(missing_ok=True)
    rows = validate_manifest(json.loads(MANIFEST.read_text()))
    packs = {}
    for pack in PACKS:
        child = output / pack
        child.mkdir(exist_ok=True)
        (child / 'receipt.json').unlink(missing_ok=True)
        with (output / (pack + '.log')).open('w') as log:
            subprocess.run([sys.executable, str(ROOT / 'scripts' / (pack + '_reference.py')),
                            '--cache', str(args.cache.resolve()), '--output', str(child)],
                           cwd=ROOT, stdin=subprocess.DEVNULL, stdout=log, stderr=subprocess.STDOUT,
                           check=True, timeout=1200,
                           pass_fds=((int(os.environ['MTG_SYMPHONY_LOCK_FD']),)
                                     if 'MTG_SYMPHONY_LOCK_FD' in os.environ else ()))
        packs[pack] = validate_run(pack, child)
        packs[pack]['receipt'] = str((child / 'receipt.json').relative_to(output))
        print(pack + ': source-matched native/XMage observations agree twice', flush=True)
    counts = {}
    for row in rows:
        for execution in row['executions']:
            if not set(execution['cases']) <= set(packs[execution['pack']]['cases']):
                raise ValueError('unexecuted catalog mapping')
        capability = row['catalog']['capability_id']
        counts[capability] = counts.get(capability, 0) + 1
    receipt = dict(status='agreed', issue=210, assigned_cases=50, assignments=rows, packs=packs,
                   manifest_sha256=xmage.sha(MANIFEST), runner_sha256=xmage.sha(Path(__file__)),
                   by_capability={k: dict(planned=v, executed=v, bounded_observations_agreed=v,
                                          disputed=0, unavailable=0) for k, v in counts.items()},
                   holdout='holdout-invoker-growth',
                   limitations='Counts are assigned catalog mappings, not distinct scenarios. Synthetic reference positions; scoped checkpoints only. Native raw rejection/nonmutation and played Driver/Run replay/capture remain mandatory normal torture coverage. Each mapping states XMage observation limits. No Forge/full-game equality, whole-block or M2 gate verdict.')
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print('50 unchanged catalog cases have fresh bounded execution receipts.')


if __name__ == '__main__':
    main()
