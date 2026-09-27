"""Bounded test-only XMage combat runner; production engine is never imported."""
import argparse
import json
from pathlib import Path
import sys

import xmage

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / 'fixtures/reference/vanilla-combat.json'


def compare(fixture, results):
    expected_ids = [case['id'] for case in fixture['cases']]
    if set(results) != set(expected_ids):
        raise ValueError('missing or extra combat cases')
    for case in fixture['cases']:
        if results[case['id']] != case['expected']:
            raise ValueError(f"combat checkpoint mismatch: {case['id']}")


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
        bridge = ROOT/'references/xmage/VanillaCombatTest.java'
        target = cache/xmage.SOURCE/'Mage.Tests/src/test/java/org/mage/test/mtglab/VanillaCombatTest.java'
        target.write_bytes(bridge.read_bytes())
        output = cache/'combat-checkpoints'
        output.mkdir(exist_ok=True)
        for old in output.glob('*.json'):
            old.unlink()
        command = xmage.maven(cache)+['-o', '-pl', 'Mage.Tests', '-am', 'test',
            '-Dtest=org.mage.test.mtglab.VanillaCombatTest', '-Dsurefire.failIfNoSpecifiedTests=false',
            '-Dmtglab.fixture='+str(FIXTURE), '-Dmtglab.output='+str(output),
            '-DargLine=-Djava.awt.headless=true']
        xmage.bounded(command, cache/xmage.SOURCE, xmage.environment(cache), cache/'combat.log', 300)
        results = {p.stem: json.loads(p.read_text()) for p in output.glob('*.json')}
        compare(json.loads(FIXTURE.read_text()), results)
        report = dict(status='agreed', upstream_commit=xmage.scenario.load(ROOT/'references/xmage/pins.json')['upstream_commit'],
            fixture_sha256=xmage.sha(FIXTURE), bridge_sha256=xmage.sha(bridge),
            log_sha256=xmage.sha(cache/'combat.log'), checkpoints=results,
            scope='Five original synthetic vanilla combats: declarations, tapping, blocked pairs, life, marked damage, power/toughness, simultaneous deaths; modern 1+1 and 0+2 allocations.',
            limitations='No illegal-action enumeration, hidden views, full games, keywords or Forge coverage.',
            stdin='closed', display='unset', offline=True, timeout_seconds=300)
        args.output.write_text(json.dumps(report, indent=2)+'\n')
        print('Five matched XMage combat scenarios agreed.')
    except (ValueError, OSError, KeyError) as error:
        print(str(error), file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
