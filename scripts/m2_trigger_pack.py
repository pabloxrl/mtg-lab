"""Rerun GH-211's prerequisite packs and bounded compositions; emit exact assignment receipt."""
import argparse
import copy
import json
import os
from pathlib import Path
import subprocess
import sys
import cleanup_reference
import instant_reference
import m2_trigger_reference
import xmage

ROOT = Path(__file__).resolve().parents[1]
ASSIGNMENT = ROOT / 'fixtures/reference/m2-trigger-assignment.json'
PACKS = {
    'cast': ('cast_trigger_reference.py', 'cast-triggers'),
    'etb': ('etb_trigger_reference.py', 'etb-triggers'),
    'cleanup': ('cleanup_reference.py', 'cleanup'),
    'composition': ('m2_trigger_reference.py', 'm2-trigger-composition'),
}


def assignments():
    rows = json.loads(ASSIGNMENT.read_text())['cases']
    assigned = {line.split('`')[1] for line in
                (ROOT / 'doc/programs/m2-test-crosswalk.md').read_text().splitlines()
                if '| #211 |' in line}
    if set(rows) != assigned:
        raise ValueError('assignment differs from unchanged current-main crosswalk')
    for key, row in rows.items():
        _, fixture = PACKS[row['pack']]
        available = json.loads((ROOT / f'fixtures/reference/{fixture}-expectations.json').read_text())
        if not row['cases'] or not set(row['cases']) <= set(available) or not row['boundary']:
            raise ValueError('missing execution mapping: ' + key)
    return rows


def minimized_controls(output):
    """Single case/checkpoint counterexamples, independently specified expectations."""
    expected = json.loads(m2_trigger_reference.EXPECTED.read_text())['apnap_p0'][0]
    wrong = copy.deepcopy(expected)
    wrong['stack'][0], wrong['stack'][1] = wrong['stack'][1], wrong['stack'][0]
    cleanup = json.loads(cleanup_reference.EXPECTED.read_text())['rules-continuous-hand-size-cleanup-interaction']
    controls = [
        ('source-order', expected, wrong, 'APNAP source order at placement; equal life/count is insufficient.'),
        ('skipped-cleanup', cleanup[-1:], [], 'CR 514.3a: required post-repeat cleanup/upkeep checkpoint omitted.'),
    ]
    result = []
    for name, expected, actual, basis in controls:
        diff = instant_reference.difference(expected, actual)
        if not diff:
            raise ValueError('undetected minimized control: ' + name)
        path = output / (name + '-minimized.json')
        path.write_text(json.dumps(dict(expected=expected, actual=actual,
                                      first_divergence=diff, basis=basis), indent=2) + '\n')
        result.append(dict(name=name, artifact=path.name, sha256=xmage.sha(path)))
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cache', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    (output / 'receipt.json').unlink(missing_ok=True)
    rows = assignments()
    receipts = {}
    for pack, (script, _) in PACKS.items():
        subprocess.run([sys.executable, str(ROOT / 'scripts' / script), '--cache',
                        str(args.cache.resolve()), '--output', str(output / pack)],
                       cwd=ROOT, stdin=subprocess.DEVNULL, check=True,
                       pass_fds=((int(os.environ['MTG_SYMPHONY_LOCK_FD']),)
                                 if 'MTG_SYMPHONY_LOCK_FD' in os.environ else ()))
        path = output / pack / 'receipt.json'
        receipt = json.loads(path.read_text())
        if receipt['status'] != 'agreed' or receipt['repetitions'] != 2:
            raise ValueError('incomplete pack: ' + pack)
        receipts[pack] = dict(path=str(path.relative_to(output)), sha256=xmage.sha(path),
                              cases=receipt['cases'])
    controls = minimized_controls(output)
    catalog = {c['id']: c for c in json.loads(
        (ROOT / 'doc/testing/capability-test-plan.json').read_text())['cases']}
    capabilities = {}
    for identifier in rows:
        capability = catalog[identifier]['capability_id']
        capabilities[capability] = capabilities.get(capability, 0) + 1
    coverage = {key: dict(planned=count, executed=count, bounded_observations_agreed=count,
                          disputed=0, unavailable=0) for key, count in capabilities.items()}
    report = dict(status='executed', assigned_catalog_cases=len(rows), assignments=rows,
                  packs=receipts, controls=controls, by_capability=coverage,
                  engines=['native', 'xmage'],
                  count_basis='Assigned case mappings, not distinct scenarios or full API parity; bounded observations only. See every assignment boundary.',
                  assignment_sha256=xmage.sha(ASSIGNMENT),
                  catalog_sha256=xmage.sha(ROOT / 'doc/testing/capability-test-plan.json'),
                  runner_sha256=xmage.sha(Path(__file__)),
                  limitations='Assignment counts are not distinct scenario counts. Raw negative command/snapshot guarantees are native where each boundary says so; no full-state, normal-reset or Forge reference claim. Original owners and M2 gate retain acceptance.')
    (output / 'receipt.json').write_text(json.dumps(report, indent=2) + '\n')
    print(f'{len(rows)} unchanged assigned catalog cases have fresh bounded execution receipts.')


if __name__ == '__main__':
    main()
