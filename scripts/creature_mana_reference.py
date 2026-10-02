"""Execute the shared creature-mana cases in native Rust and pinned XMage."""
import argparse
import copy
import json
import os
from pathlib import Path
import instant_reference
import xmage

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / 'fixtures/reference/creature-mana.json'
EXPECTED = ROOT / 'fixtures/reference/creature-mana-expectations.json'
BRIDGE = ROOT / 'references/xmage/CreatureManaTest.java'


def compare(actual):
    expected = json.loads(EXPECTED.read_text())
    diff = instant_reference.difference(expected, actual)
    if diff:
        raise ValueError('first divergence: ' + json.dumps(diff, sort_keys=True))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cache', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    cache, output = args.cache.resolve(), args.output.resolve()
    xmage.verify_inputs(cache)
    xmage.scenario.require(xmage.dependencies(cache) == xmage.scenario.load(ROOT / 'references/xmage/dependencies.json'), 'dependency lock mismatch')
    output.mkdir(parents=True, exist_ok=True)
    target = cache / xmage.SOURCE / 'Mage.Tests/src/test/java/org/mage/test/mtglab/CreatureManaTest.java'
    target.write_bytes(BRIDGE.read_bytes())
    runs = []
    for repeat in range(2):
        folder = output / str(repeat)
        folder.mkdir(exist_ok=True)
        for old in folder.glob('*.json'):
            old.unlink()
        cmd = xmage.maven(cache) + ['-o', '-pl', 'Mage.Tests', '-am', 'test',
            '-Dtest=org.mage.test.mtglab.CreatureManaTest', '-Dsurefire.failIfNoSpecifiedTests=false',
            '-Dmtglab.fixture=' + str(FIXTURE), '-Dmtglab.output=' + str(folder),
            '-DargLine=-Djava.awt.headless=true']
        xmage.bounded(cmd, cache / xmage.SOURCE, xmage.environment(cache), folder / 'xmage.log', 300)
        actual = {p.stem: json.loads(p.read_text()) for p in folder.glob('*.json')}
        compare(actual)
        native = folder / 'native.json'
        env = dict(os.environ, MTG_CREATURE_MANA_OUTPUT=str(native))
        xmage.bounded(['cargo', 'test', '-p', 'mtg-core', '--locked', '--lib',
            'game::creature_mana_reference_tests::creature_mana_reference_literal_checkpoints', '--', '--exact'],
            ROOT, env, folder / 'native.log', 180)
        compare(json.loads(native.read_text()))
        runs.append({'xmage_log_sha256': xmage.sha(folder / 'xmage.log'), 'native_log_sha256': xmage.sha(folder / 'native.log')})
    controls = []
    canonical = json.loads(EXPECTED.read_text())
    for field in ['green', 'stack', 'priority', 'tapped', 'damage', 'power', 'toughness', 'cub', 'cub_damage', 'life']:
        wrong = copy.deepcopy(canonical)
        name = 'druid-block' if field in ['damage', 'cub_damage'] else 'llanowar-elves-priority'
        value = wrong[name][field]
        wrong[name][field] = not value if isinstance(value, bool) else [19, 20] if isinstance(value, list) else value + 1
        try:
            compare(wrong)
        except ValueError as error:
            controls.append({'field': field, 'detected': str(error)})
        else:
            raise ValueError('missed comparator mutation: ' + field)
    receipt = dict(status='agreed', cases=len(canonical), repetitions=2,
        upstream_commit=xmage.scenario.load(ROOT / 'references/xmage/pins.json')['upstream_commit'],
        fixture_sha256=xmage.sha(FIXTURE), expected_sha256=xmage.sha(EXPECTED),
        bridge_sha256=xmage.sha(BRIDGE), runner_sha256=xmage.sha(Path(__file__)),
        native_sources={str(p.relative_to(ROOT)): xmage.sha(p) for p in sorted((ROOT / 'crates/mtg-core/src').glob('*.rs'))},
        runs=runs, controls=controls, stdin='closed', display='unset', offline=True,
        limitations='Synthetic setup; no Forge/full-game agreement. Native-only cancellation, stale handles, wrong-seat and snapshot nonmutation. Opponent-priority reference checks the priority boundary; it does not submit an out-of-turn player callback. No haste/Cavalry composition.')
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print('13 creature-mana cases agreed twice in native Rust and pinned XMage.')


if __name__ == '__main__':
    main()
