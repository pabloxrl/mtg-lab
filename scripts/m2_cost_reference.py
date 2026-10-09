"""Execute the bounded GH-209 cost compositions in native Rust and pinned XMage."""
import argparse
import copy
import json
import os
from pathlib import Path
import instant_reference
import xmage

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / 'fixtures/reference/m2-cost-composition.json'
EXPECTED = ROOT / 'fixtures/reference/m2-cost-composition-expectations.json'
BRIDGE = ROOT / 'references/xmage/M2CostCompositionTest.java'


def compare(actual):
    expected = json.loads(EXPECTED.read_text())
    diff = instant_reference.difference(expected, actual)
    if diff:
        raise ValueError('first divergence: ' + json.dumps(diff, sort_keys=True))


def validate_fixture(fixture):
    if (set(fixture) != {'version', 'cases'} or type(fixture['version']) is not int or
            fixture['version'] != 1 or not isinstance(fixture['cases'], list)):
        raise ValueError('unsupported composition schema')
    seen = set()
    for case in fixture['cases']:
        identity = case.get('id')
        if not isinstance(identity, str) or not identity or identity in seen:
            raise ValueError('invalid or duplicate composition id')
        seen.add(identity)
        kind = case.get('kind')
        if kind == 'land':
            if set(case) != {'id', 'kind', 'first'} or case['first'] not in {'mountain', 'forest'}:
                raise ValueError('unsupported land script')
        elif kind == 'haste-mana':
            if (set(case) != {'id', 'kind', 'card', 'payment'} or
                    case['card'] not in {'llanowar-elves', 'druid-of-the-cowl'} or
                    type(case['payment']) is not bool):
                raise ValueError('unsupported haste-mana script')
        elif kind == 'archer-thrill':
            required = {'id', 'kind', 'archers', 'library', 'discard', 'opponent_life'}
            if (not required <= set(case) or set(case) - required - {'authorship'} or
                    type(case['archers']) is not int or not 1 <= case['archers'] <= 2 or
                    case['discard'] not in {'mountain', 'goblin-surprise'} or
                    not isinstance(case['library'], list) or len(case['library']) != 3 or
                    any(c not in {'forest', 'mountain', 'bear-cub', 'giant-growth'} for c in case['library']) or
                    type(case['opponent_life']) is not int or case['opponent_life'] <= case['archers']):
                raise ValueError('unsupported archer-thrill script')
        else:
            raise ValueError('unsupported composition kind')
    if seen != set(json.loads(EXPECTED.read_text())):
        raise ValueError('composition inventory differs from independent oracle')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cache', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    cache, output = args.cache.resolve(), args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    (output / "receipt.json").unlink(missing_ok=True)
    validate_fixture(json.loads(FIXTURE.read_text()))
    xmage.verify_inputs(cache)
    xmage.scenario.require(xmage.dependencies(cache) == xmage.scenario.load(ROOT / 'references/xmage/dependencies.json'), 'dependency lock mismatch')
    output.mkdir(parents=True, exist_ok=True)
    (output / "receipt.json").unlink(missing_ok=True)
    target = cache / xmage.SOURCE / 'Mage.Tests/src/test/java/org/mage/test/mtglab/M2CostCompositionTest.java'
    target.write_bytes(BRIDGE.read_bytes())
    runs = []
    for repeat in range(2):
        folder = output / str(repeat)
        folder.mkdir(exist_ok=True)
        for old in folder.glob('*.json'):
            old.unlink()
        cmd = xmage.maven(cache) + ['-o', '-pl', 'Mage.Tests', '-am', 'test',
            '-Dtest=org.mage.test.mtglab.M2CostCompositionTest', '-Dsurefire.failIfNoSpecifiedTests=false',
            '-Dmtglab.fixture=' + str(FIXTURE), '-Dmtglab.output=' + str(folder),
            '-DargLine=-Djava.awt.headless=true']
        xmage.bounded(cmd, cache / xmage.SOURCE, xmage.environment(cache), folder / 'xmage.log', 300)
        actual = {p.stem: json.loads(p.read_text()) for p in folder.glob('*.json')}
        compare(actual)
        native = folder / 'native.json'
        env = dict(os.environ, MTG_M2_COST_OUTPUT=str(native))
        xmage.bounded(['cargo', 'test', '-p', 'mtg-core', '--locked', '--lib',
            'game::m2_cost_reference_tests::cost_compositions_match_independent_literal_checkpoints', '--', '--exact'],
            ROOT, env, folder / 'native.log', 180)
        compare(json.loads(native.read_text()))
        runs.append({'xmage_log_sha256': xmage.sha(folder / 'xmage.log'), 'native_log_sha256': xmage.sha(folder / 'native.log')})
    controls = []
    canonical = json.loads(EXPECTED.read_text())
    mutations = [('life', 'archer-thrill', 1, 'life'),
                 ('zone', 'archer-thrill', 2, 'hand'),
                 ('mana', 'cavalry-elf', 3, 'green'),
                 ('choice', 'holdout-two-archers-discard-surprise', 0, 'graveyard'),
                 ('checkpoint', 'cavalry-druid-bite', 1, 'checkpoint')]
    for label, case, index, field in mutations:
        wrong = copy.deepcopy(canonical)
        value = wrong[case][index][field]
        if label == 'life':
            value[1] += 1
        elif label in {'zone', 'choice'}:
            value[0] = 'forest' if label == 'choice' else 'mountain'
        elif label == 'mana':
            wrong[case][index][field] += 1
        else:
            wrong[case][index][field] = 'wrong-semantic-boundary'
        try:
            compare(wrong)
        except ValueError as error:
            controls.append({'field': label, 'detected': str(error)})
        else:
            raise ValueError('missed comparator mutation: ' + label)
    receipt = dict(status='agreed', cases=len(canonical), repetitions=2,
        upstream_commit=xmage.scenario.load(ROOT / 'references/xmage/pins.json')['upstream_commit'],
        fixture_sha256=xmage.sha(FIXTURE), expected_sha256=xmage.sha(EXPECTED),
        bridge_sha256=xmage.sha(BRIDGE), runner_sha256=xmage.sha(Path(__file__)),
        native_sources={str(p.relative_to(ROOT)): xmage.sha(p) for p in sorted((ROOT / 'crates/mtg-core/src').glob('*.rs'))},
        runs=runs, controls=controls, stdin='closed', display='unset', offline=True,
        limitations='Synthetic initial positions; existing actual cast, haste, mana, discard, trigger and resolution operations composed at explicit checkpoints. Normal-reset Driver/Run evidence is separate in prerequisite tests. No Forge/full-game or whole-pool agreement.')
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print('6 cost compositions agreed twice in native Rust and pinned XMage.')


if __name__ == '__main__':
    main()
