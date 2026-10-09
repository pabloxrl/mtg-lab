"""Execute GH-212's bounded native/reference coverage and preserve earlier M1 receipts.

Run through the shared heavy lock. The joined receipt distinguishes native API
assertions from selected XMage observations; neither claims full-state parity.
"""
import argparse
import copy
import json
import os
from pathlib import Path
import re
import subprocess
import sys

import instant_reference
import xmage

ROOT = Path(__file__).resolve().parents[1]
ASSIGNMENT = ROOT / 'fixtures/reference/m2-foundation-assignment.json'
HOLDOUT = ROOT / 'fixtures/reference/m2-foundation-holdout.json'
ORACLE = ROOT / 'fixtures/reference/m2-foundation-holdout-expectations.json'
SUPPLEMENTAL = ['policy_library_only_twins_payment_errors_and_masked_rows',
                'policy_restore_invalidates_pre_restore_submissions_without_state_change',
                'thrill_private_pending_snapshots_cancel_stale_and_quantum']
PACKS = {
    'thrill': 'thrill', 'invoker': 'invoker', 'etb_trigger': 'etb-triggers',
    'deathtouch': 'deathtouch', 'instant': 'instant', 'shivan': 'shivan',
    'm2_trigger': 'm2-trigger-composition', 'haste': 'haste',
    'cast_trigger': 'cast-triggers', 'terminal': 'terminal',
    'surprise': 'surprise', 'combat': 'vanilla-combat',
}


def load(path):
    return json.loads(path.read_text())


def validate_catalog(rows):
    catalog = {c['id']: c for c in load(ROOT / 'doc/testing/capability-test-plan.json')['cases']}
    assigned = set(re.findall(r'\| `([^`]+)` \| #[0-9]+ \| #212 \|',
                             (ROOT / 'doc/programs/m2-test-crosswalk.md').read_text()))
    actual = [r['catalog']['id'] for r in rows]
    if len(actual) != len(set(actual)) or set(actual) != assigned:
        raise ValueError('missing, duplicate or extra assigned case')
    source = '\n'.join(p.read_text() for p in (ROOT / 'crates').rglob('*.rs'))
    for row in rows:
        if row['catalog'] != catalog[row['catalog']['id']]:
            raise ValueError('original catalog expectation or ownership changed')
        if not row['native_tests'] or not row['boundary']:
            raise ValueError('missing native mapping or observation boundary')
        for test in row['native_tests']:
            if f'fn {test}(' not in source:
                raise ValueError('missing native test: ' + test)
        pack = row['pack']
        if pack == 'native':
            if row['reference_cases'] or row['catalog']['capability_id'] != 'decisions/capacity':
                raise ValueError('unexpected reference omission')
            continue
        if pack not in PACKS or not row['reference_cases']:
            raise ValueError('missing reference mapping')
        if pack in ('terminal', 'combat'):
            fixture = 'terminal-v2' if pack == 'terminal' else 'vanilla-combat'
            available = {c['id'] for c in load(ROOT / f'fixtures/reference/{fixture}.json')['cases']}
        else:
            available = set(load(ROOT / f'fixtures/reference/{PACKS[pack]}-expectations.json'))
        if not set(row['reference_cases']) <= available:
            raise ValueError('unknown reference case: ' + row['catalog']['id'])


def expected_holdout():
    return {c['id']: {'checkpoints': load(ORACLE)[c['id']], 'consumed': c['script']}
            for c in load(HOLDOUT)['cases']}


def compare_holdout(actual):
    diff = instant_reference.difference(expected_holdout(), actual)
    if diff:
        raise ValueError('holdout first divergence: ' + json.dumps(diff, sort_keys=True))


def run(command, log):
    fd = os.environ.get('MTG_SYMPHONY_LOCK_FD')
    with log.open('w') as stream:
        subprocess.run(command, cwd=ROOT, stdin=subprocess.DEVNULL, stdout=stream,
                       stderr=subprocess.STDOUT, check=True,
                       pass_fds=(int(fd),) if fd else ())


def execute_holdout(cache, output):
    output.mkdir(exist_ok=True)
    xmage.verify_inputs(cache)
    if xmage.dependencies(cache) != load(ROOT / 'references/xmage/dependencies.json'):
        raise ValueError('dependency lock mismatch')
    target = cache / xmage.SOURCE / 'Mage.Tests/src/test/java/org/mage/test/mtglab/InstantResponseTest.java'
    target.write_bytes(instant_reference.BRIDGE.read_bytes())
    for repeat in range(2):
        native = instant_reference.run_native(output / f'native-{repeat}.json',
                    output / f'native-{repeat}.log', HOLDOUT)
        compare_holdout(native)
        actual = instant_reference.run_xmage(cache, output / f'xmage-{repeat}',
                    output / f'xmage-{repeat}.log', HOLDOUT)
        compare_holdout(actual)
        (output / f'xmage-{repeat}.json').write_text(json.dumps(actual, indent=2)+'\n')
    wrong = copy.deepcopy(native)
    wrong['holdout-sentry-bite-grown-sentry']['checkpoints'][-1]['state']['damage_events'][0]['amount'] = 2
    try:
        compare_holdout(wrong)
    except ValueError as error:
        (output / 'corrupt-checkpoint.json').write_text(json.dumps({
            'status': 'detected', 'actual': wrong, 'error': str(error)}, indent=2)+'\n')
    else:
        raise ValueError('corrupt holdout was accepted')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cache', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    cache, output = args.cache.resolve(), args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    receipt = output / 'receipt.json'
    receipt.unlink(missing_ok=True)
    rows = load(ASSIGNMENT)
    validate_catalog(rows)
    # Retain exact historical bytes and provenance. They are not fresh runs.
    earlier = [ROOT / 'doc/evidence/m1-gate' / p for p in
               ['terminal.json', 'combat.json', 'opening.json', 'catalog.json', 'sources.json', 'instant/acceptance.json']]
    earlier += sorted((ROOT / 'doc/evidence/m1-gate').glob('*reference*.json'))
    retained = {str(p.relative_to(ROOT)): xmage.sha(p) for p in earlier}
    runs = {}
    for pack in PACKS:
        child = output / pack
        argument = child.with_suffix('.json') if pack in ('terminal', 'combat') else child
        result = (argument if pack in ('terminal', 'combat') else
                  child / ('acceptance.json' if pack == 'instant' else 'receipt.json'))
        run([sys.executable, str(ROOT / f'scripts/{pack}_reference.py'),
             '--cache', str(cache), '--output', str(argument)], output / f'{pack}.log')
        report = load(result)
        if report.get('status') != 'agreed':
            raise ValueError('reference did not agree: ' + pack)
        if pack == 'terminal':
            observed = load(child / 'v2-0/native.json')
        elif pack == 'instant':
            observed = load(child / 'native-1.json')
        elif pack == 'combat':
            observed = report['checkpoints']
        else:
            observed = load(child / '0/native.json')
        for row in rows:
            if row['pack'] == pack and not set(row['reference_cases']) <= set(observed):
                raise ValueError('mapped case did not execute: ' + row['catalog']['id'])
        runs[pack] = {'receipt': str(result.relative_to(output)), 'sha256': xmage.sha(result)}
        print(pack + ': pinned reference passed', flush=True)
    # Native complete core discovery includes every mapped test and the holdout;
    # the separate full workspace torture command remains mandatory.
    run(['cargo', 'test', '--locked', '-p', 'mtg-core', '--lib'], output / 'native-core.log')
    native_log = (output / 'native-core.log').read_text()
    for name in {t for row in rows for t in row['native_tests']} | set(SUPPLEMENTAL):
        if not re.search(r'test [^\n]*::' + re.escape(name) + r' \.\.\. ok', native_log):
            raise ValueError('native mapped test did not pass: ' + name)
    execute_holdout(cache, output / 'holdout')
    if retained != {p: xmage.sha(ROOT / p) for p in retained}:
        raise ValueError('historical M1 receipt changed')
    coverage = {}
    for row in rows:
        key = row['catalog']['capability_id']
        counts = coverage.setdefault(key, dict(planned=0, native_executed=0,
                                              bounded_reference_mappings=0, disputed=0, unsupported=0))
        counts['planned'] += 1
        counts['native_executed'] += 1
        counts['bounded_reference_mappings'] += row['pack'] != 'native'
    report = dict(status='executed', issue=212, assigned_cases=len(rows), assignments=rows,
                  packs=runs, by_capability=coverage, retained_m1_receipts=retained,
                  supplemental_native_tests=SUPPLEMENTAL,
                  holdout='holdout-sentry-bite-grown-sentry',
                  by_engine={'native': dict(planned=31, executed=31, agreed=31, disputed=0, unsupported=0),
                             'xmage': dict(planned=29, executed=29, bounded_observations_agreed=29,
                                           native_only_capacity_contracts=2, disputed=0, unsupported=0),
                             'forge': dict(planned=0, executed=0, agreed=0, disputed=0, unsupported=0)},
                  sources={str(p.relative_to(ROOT)): xmage.sha(p) for p in
                           [ASSIGNMENT, HOLDOUT, ORACLE, Path(__file__)]},
                  artifacts={str(p.relative_to(output)): xmage.sha(p)
                             for p in sorted(output.rglob('*')) if p.is_file()},
                  limitations='Assigned mappings are not distinct scenarios. Native capacity/private/policy '
                  'contracts have no XMage API parity claim; other reference boundaries are explicit per '
                  'mapping. No full-state/full-game/Forge or milestone completion claim. Full torture and '
                  'separate review remain required.')
    receipt.write_text(json.dumps(report, indent=2)+'\n')
    print('31 assigned cases executed with bounded reference receipts and a frozen holdout.')


if __name__ == '__main__':
    main()
