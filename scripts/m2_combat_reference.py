"""Execute the shared m2-combat cases in native Rust and pinned XMage."""
import argparse
import copy
import json
import os
from pathlib import Path
import instant_reference
import xmage

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / 'fixtures/reference/m2-combat.json'
EXPECTED = ROOT / 'fixtures/reference/m2-combat-expectations.json'
BRIDGE = ROOT / 'references/xmage/M2CombatTest.java'


def validate_fixture(document):
    def require(ok, path):
        if not ok:
            raise ValueError('unsupported composition fixture: ' + path)
    require(set(document) == {'version', 'setup', 'basis', 'cases'}, 'fields')
    require(type(document['version']) is int and document['version'] == 1, 'version')
    cases = document['cases']
    require(isinstance(cases, list), 'cases')
    seen = set()
    fields = {'id', 'target', 'boosts', 'growths', 'blocker', 'blockers', 'departure', 'holdout'}
    for i, case in enumerate(cases):
        path = f'cases/{i}'
        require(isinstance(case, dict) and set(case) == fields, path + '/fields')
        require(isinstance(case['id'], str) and case['id'] not in seen, path + '/id')
        seen.add(case['id'])
        require(case['target'] in ('bear-cub', 'thornweald-archer'), path + '/target')
        require(case['blocker'] in ('bear-cub', 'magnigoth-sentry'), path + '/blocker')
        for name in ('boosts', 'growths', 'blockers'):
            require(type(case[name]) is int and 0 <= case[name] <= 2, path + '/' + name)
        for name in ('departure', 'holdout'):
            require(type(case[name]) is bool, path + '/' + name)
        require(not case['departure'] or (case['blockers'], case['boosts'], case['growths']) == (1, 0, 1), path + '/departure')
    require(seen == set(json.loads(EXPECTED.read_text())), 'missing/extra scripts')
    require([c['id'] for c in cases if c['holdout']] == ['holdout-invoker-growth'], 'holdout')


def compare(actual):
    expected = json.loads(EXPECTED.read_text())
    diff = instant_reference.difference(expected, actual)
    if diff:
        raise ValueError('first divergence: ' + json.dumps(diff, sort_keys=True))


def mutations(canonical):
    split = copy.deepcopy(canonical)
    split['invoker-cubs']['after-damage']['life'][1] = 14
    retained = copy.deepcopy(canonical)
    retained['invoker-cubs']['after-cleanup']['trample'] = True
    return {'damage-split': split, 'retained-temporary-keyword': retained}


def validate_manifest(document):
    # Import lazily so the composition runner remains usable independently.
    from m2_combat_pack import validate_manifest as validate
    return validate(document)


def require_current_receipt(receipt, expected):
    if receipt.get('status') != 'agreed' or receipt.get('repetitions') != 2:
        raise ValueError('missing or unexecuted receipt')
    for field, value in expected.items():
        if receipt.get(field) != value:
            raise ValueError('stale receipt: ' + field)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cache', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    cache, output = args.cache.resolve(), args.output.resolve()
    validate_fixture(json.loads(FIXTURE.read_text()))
    xmage.verify_inputs(cache)
    xmage.scenario.require(xmage.dependencies(cache) == xmage.scenario.load(ROOT / 'references/xmage/dependencies.json'), 'dependency lock mismatch')
    output.mkdir(parents=True, exist_ok=True)
    (output / "receipt.json").unlink(missing_ok=True)
    target = cache / xmage.SOURCE / 'Mage.Tests/src/test/java/org/mage/test/mtglab/M2CombatTest.java'
    target.write_bytes(BRIDGE.read_bytes())
    runs = []
    for repeat in range(2):
        folder = output / str(repeat)
        folder.mkdir(exist_ok=True)
        for old in folder.glob('*.json'):
            old.unlink()
        cmd = xmage.maven(cache) + ['-o', '-pl', 'Mage.Tests', '-am', 'test',
            '-Dtest=org.mage.test.mtglab.M2CombatTest', '-Dsurefire.failIfNoSpecifiedTests=false',
            '-Dmtglab.fixture=' + str(FIXTURE), '-Dmtglab.output=' + str(folder),
            '-DargLine=-Djava.awt.headless=true']
        xmage.bounded(cmd, cache / xmage.SOURCE, xmage.environment(cache), folder / 'xmage.log', 300)
        actual = {p.stem: json.loads(p.read_text()) for p in folder.glob('*.json')}
        compare(actual)
        native = folder / 'native.json'
        env = dict(os.environ, MTG_M2_COMBAT_OUTPUT=str(native))
        xmage.bounded(['cargo', 'test', '-p', 'mtg-core', '--locked', '--lib',
            'game::m2_combat_tests::m2_combat_reference_literal_checkpoints', '--', '--exact'],
            ROOT, env, folder / 'native.log', 180)
        compare(json.loads(native.read_text()))
        runs.append({'xmage_log_sha256': xmage.sha(folder / 'xmage.log'), 'native_log_sha256': xmage.sha(folder / 'native.log'), 'observations': {p.name: xmage.sha(p) for p in sorted(folder.glob('*.json'))}})
    controls = []
    canonical = json.loads(EXPECTED.read_text())
    for name, wrong in mutations(canonical).items():
        try:
            compare(wrong)
        except ValueError as error:
            artifact = output / (name + '.json')
            artifact.write_text(json.dumps({'input': wrong, 'first_divergence': str(error)}, indent=2) + '\n')
            controls.append({'mutation': name, 'detected': str(error), 'artifact_sha256': xmage.sha(artifact)})
        else:
            raise ValueError('missed comparator mutation: ' + name)
    receipt = dict(status='agreed', cases=len(canonical), repetitions=2,
        upstream_commit=xmage.scenario.load(ROOT / 'references/xmage/pins.json')['upstream_commit'],
        fixture_sha256=xmage.sha(FIXTURE), expected_sha256=xmage.sha(EXPECTED),
        bridge_sha256=xmage.sha(BRIDGE), runner_sha256=xmage.sha(Path(__file__)),
        native_sources={str(p.relative_to(ROOT)): xmage.sha(p) for p in sorted((ROOT / 'crates/mtg-core/src').glob('*.rs'))},
        runs=runs, controls=controls, stdin='closed', display='unset', offline=True,
        limitations='Synthetic initial state and blocker departure; actual activation, Growth, combat and cleanup. Compare creature power/toughness/damage, trample, life, mana and stack depth at named checkpoints. Raw illegal damage submission/nonmutation is native; XMage exposes the constrained legal allocation domain. No Forge or full-game claim.')
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(f'{len(canonical)} composed combat cases agreed twice in native Rust and pinned XMage.')


if __name__ == '__main__':
    main()
