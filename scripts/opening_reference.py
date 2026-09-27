"""Bounded test-only XMage opening runner; production engine is never imported."""
import argparse
import json
import platform
from pathlib import Path
import sys

import xmage

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / 'fixtures/reference/opening-counts.json'


def compare(fixture, results):
    expected_ids = [case['id'] for case in fixture['cases']]
    if set(results) != set(expected_ids):
        raise ValueError('missing or extra opening cases')
    for case in fixture['cases']:
        if not xmage.scenario.json_equal(results[case['id']], case['expected']):
            raise ValueError(f"opening checkpoint mismatch: {case['id']}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cache', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    cache = args.cache.resolve()
    try:
        args.output.unlink(missing_ok=True)
        xmage.scenario.require(not cache.is_relative_to(ROOT), 'cache must be external')
        xmage.verify_inputs(cache)
        xmage.scenario.require(xmage.dependencies(cache) == xmage.scenario.load(ROOT/'references/xmage/dependencies.json'), 'dependency lock mismatch')
        provenance = xmage.scenario.load(ROOT/'references/xmage/opening-provenance.json')
        for entry in provenance['consulted_sources']:
            xmage.scenario.require(xmage.sha(cache/xmage.SOURCE/entry['path']) == entry['sha256'],
                                   'opening source pin mismatch: '+entry['path'])
        bridge = ROOT/'references/xmage/OpeningCountsTest.java'
        target = cache/xmage.SOURCE/'Mage.Tests/src/test/java/org/mage/test/mtglab/OpeningCountsTest.java'
        target.write_bytes(bridge.read_bytes())
        output = cache/'opening-checkpoints'
        output.mkdir(exist_ok=True)
        for old in output.glob('*.json'):
            old.unlink()
        command = xmage.maven(cache)+['-o', '-pl', 'Mage.Tests', '-am', 'test',
            '-Dtest=org.mage.test.mtglab.OpeningCountsTest', '-Dsurefire.failIfNoSpecifiedTests=false',
            '-Dmtglab.fixture='+str(FIXTURE), '-Dmtglab.output='+str(output),
            '-DargLine=-Djava.awt.headless=true']
        xmage.bounded(command, cache/xmage.SOURCE, xmage.environment(cache), cache/'opening.log', 300)
        results = {p.stem: json.loads(p.read_text()) for p in output.glob('*.json')}
        compare(json.loads(FIXTURE.read_text()), results)
        report = dict(status='agreed', upstream_commit=xmage.scenario.load(ROOT/'references/xmage/pins.json')['upstream_commit'],
            platform=platform.system()+'/'+platform.machine(),
            runner_sha256=xmage.sha(Path(__file__)), shared_runner_sha256=xmage.sha(ROOT/'scripts/xmage.py'),
            pins_sha256=xmage.sha(ROOT/'references/xmage/pins.json'),
            dependencies_sha256=xmage.sha(ROOT/'references/xmage/dependencies.json'),
            provenance_sha256=xmage.sha(ROOT/'references/xmage/opening-provenance.json'),
            fixture_sha256=xmage.sha(FIXTURE), bridge_sha256=xmage.sha(bridge),
            log_sha256=xmage.sha(cache/'opening.log'), checkpoints=results,
            scope='Eight original synthetic opening prefixes; real London declarations, redraws and cumulative bottom counts, both starting seats.',
            limitations='Initial seven-card hands injected; identical basic cards per seat. No initial shuffle, card identity/order, PRNG, full-game or Forge comparison.',
            stdin='closed', display='unset', offline=True, timeout_seconds=300)
        args.output.write_text(json.dumps(report, indent=2)+'\n')
        print('Eight matched XMage opening count scenarios agreed.')
    except (ValueError, OSError, KeyError) as error:
        print(str(error), file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
