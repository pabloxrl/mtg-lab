"""Bounded test-only XMage terminal runner; production engine is never imported."""
import argparse
import json
from pathlib import Path
import sys

import xmage

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / 'fixtures/reference/terminal.json'


def compare(fixture, results):
    expected_ids = [case['id'] for case in fixture['cases']]
    if set(results) != set(expected_ids):
        raise ValueError('missing or extra terminal cases')
    for case in fixture['cases']:
        if results[case['id']] != case['expected']:
            raise ValueError(f"terminal checkpoint mismatch: {case['id']}")


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
        bridge = ROOT/'references/xmage/TerminalTest.java'
        target = cache/xmage.SOURCE/'Mage.Tests/src/test/java/org/mage/test/mtglab/TerminalTest.java'
        target.write_bytes(bridge.read_bytes())
        output = cache/'terminal-checkpoints'
        output.mkdir(exist_ok=True)
        for old in output.glob('*.json'):
            old.unlink()
        command = xmage.maven(cache)+['-o', '-pl', 'Mage.Tests', '-am', 'test',
            '-Dtest=org.mage.test.mtglab.TerminalTest', '-Dsurefire.failIfNoSpecifiedTests=false',
            '-Dmtglab.fixture='+str(FIXTURE), '-Dmtglab.output='+str(output),
            '-DargLine=-Djava.awt.headless=true']
        xmage.bounded(command, cache/xmage.SOURCE, xmage.environment(cache), cache/'terminal.log', 300)
        results = {p.stem: json.loads(p.read_text()) for p in output.glob('*.json')}
        compare(json.loads(FIXTURE.read_text()), results)
        report = dict(status='agreed', upstream_commit=xmage.scenario.load(ROOT/'references/xmage/pins.json')['upstream_commit'],
            fixture_sha256=xmage.sha(FIXTURE), bridge_sha256=xmage.sha(bridge),
            log_sha256=xmage.sha(cache/'terminal.log'), checkpoints=results,
            scope='Seven original synthetic terminal boundaries: zero/negative/simultaneous life, empty versus last-card draw, nonacting concession; observed life, loss flags, hand/library counts.',
            limitations='No combat/stack action integration, winner flags, private views, full-game or Forge reference coverage.',
            stdin='closed', display='unset', offline=True, timeout_seconds=300)
        args.output.write_text(json.dumps(report, indent=2)+'\n')
        print('Seven matched XMage terminal scenarios agreed.')
    except (ValueError, OSError, KeyError) as error:
        print(str(error), file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
