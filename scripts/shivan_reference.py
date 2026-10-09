"""Execute the shared shivan cases in native Rust and pinned XMage."""
import argparse
import copy
import json
import os
from pathlib import Path
import instant_reference
import xmage

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / 'fixtures/reference/shivan.json'
EXPECTED = ROOT / 'fixtures/reference/shivan-expectations.json'
BRIDGE = ROOT / 'references/xmage/ShivanTest.java'


def compare(actual):
    expected = json.loads(EXPECTED.read_text())
    diff = instant_reference.difference(expected, actual)
    if diff:
        raise ValueError('first divergence: ' + json.dumps(diff, sort_keys=True))


def validate_fixture(fixture):
    def require(ok, path):
        if not ok:
            raise ValueError('first divergence: fixture/' + path)
    require(set(fixture) == {'version', 'setup', 'basis', 'cases'}, 'fields')
    require(type(fixture['version']) is int and fixture['version'] == 1, 'version')
    require(isinstance(fixture['cases'], list), 'cases')
    seen = set()
    for i, case in enumerate(fixture['cases']):
        path = f'cases/{i}'
        require(isinstance(case, dict) and isinstance(case.get('id'), str), path)
        require(case['id'] not in seen, path + '/id')
        seen.add(case['id'])
        if not case['id'].startswith('exact_'):
            continue
        require(set(case) == {'id', 'mode', 'active', 'choices'}, path + '/fields')
        require(type(case['active']) is int and case['active'] in (0, 1), path + '/active')
        a, b = case['active'], 1 - case['active']
        mode = case['mode']
        require(mode in ('cub_cast', 'cub_reject', 'single_sentry', 'single_cub_illegal'), path + '/mode')
        require(case['id'] == f'exact_{mode}_{a}', path + '/id')
        wanted = []
        def add(kind, actor):
            wanted.append(dict(kind=kind, actor=actor))
        def pair(actor):
            add('pass', actor)
            add('pass', 1 - actor)
        if mode == 'cub_cast':
            add('cast_cub', a); add('pay_green', a); add('pay_red', a)
            pair(a); pair(a); pair(a); add('reject_attack', a)
        elif mode == 'cub_reject':
            add('pass', a); add('reject_cub', b)
        else:
            pair(a); pair(a); add('attack', a); pair(a)
            add('reject_block' if mode == 'single_cub_illegal' else 'block', b)
            if mode == 'single_sentry':
                add('pass', a); add('tap_forest', b); add('cast_growth', b); add('pay_green', b); pair(b)
            pair(a); add('damage', a)
        require(isinstance(case['choices'], list), path + '/choices')
        for j, c in enumerate(case['choices']):
            require(isinstance(c, dict) and set(c) == {'kind', 'actor'} and type(c['actor']) is int, path + f'/choices/{j}')
        diff = instant_reference.difference(wanted, case['choices'])
        require(diff is None, path + '/choices/' + str(diff))


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
    target = cache / xmage.SOURCE / 'Mage.Tests/src/test/java/org/mage/test/mtglab/ShivanTest.java'
    target.write_bytes(BRIDGE.read_bytes())
    (output / 'input.json').write_bytes(FIXTURE.read_bytes())
    (output / 'authored-expectations.json').write_bytes(EXPECTED.read_bytes())
    runs = []
    for repeat in range(2):
        folder = output / str(repeat)
        folder.mkdir(exist_ok=True)
        for old in folder.glob('*.json'):
            old.unlink()
        cmd = xmage.maven(cache) + ['-o', '-pl', 'Mage.Tests', '-am', 'test',
            '-Dtest=org.mage.test.mtglab.ShivanTest', '-Dsurefire.failIfNoSpecifiedTests=false',
            '-Dmtglab.fixture=' + str(FIXTURE), '-Dmtglab.output=' + str(folder),
            '-DargLine=-Djava.awt.headless=true']
        xmage.bounded(cmd, cache / xmage.SOURCE, xmage.environment(cache), folder / 'xmage.log', 300)
        actual = {p.stem: json.loads(p.read_text()) for p in folder.glob('*.json')}
        compare(actual)
        native = folder / 'native.json'
        env = dict(os.environ, MTG_SHIVAN_OUTPUT=str(native))
        xmage.bounded(['cargo', 'test', '-p', 'mtg-core', '--locked', '--lib',
            'game::shivan_tests::shivan_reference_literal_checkpoints', '--', '--exact'],
            ROOT, env, folder / 'native.log', 180)
        compare(json.loads(native.read_text()))
        runs.append({'xmage_log_sha256': xmage.sha(folder / 'xmage.log'), 'native_log_sha256': xmage.sha(folder / 'native.log'), 'observations': {p.name: xmage.sha(p) for p in sorted(folder.glob('*.json'))}})
    controls = []
    canonical = json.loads(EXPECTED.read_text())
    for field in ['dragon', 'others', 'life', 'mana']:
        wrong = copy.deepcopy(canonical)
        wrong['double'][field] = 99
        try:
            compare(wrong)
        except ValueError as error:
            controls.append({'field': field, 'detected': str(error)})
        else:
            raise ValueError('missed comparator mutation: ' + field)
    for name, result in canonical.items():
        if not name.startswith('exact_'):
            continue
        for target in result['combat_damage']:
            wrong = copy.deepcopy(canonical)
            wrong[name]['combat_damage'][target] += 1
            try:
                compare(wrong)
            except ValueError as error:
                controls.append({'case': name, 'field': 'combat_damage/' + target, 'detected': str(error)})
            else:
                raise ValueError('missed damage event mutation')
        for point, state in result['points'].items():
            for obj, values in state['objects'].items():
                for index in range(len(values)):
                    wrong = copy.deepcopy(canonical)
                    wrong[name]['points'][point]['objects'][obj][index] = 'mutated'
                    try:
                        compare(wrong)
                    except ValueError as error:
                        controls.append({'case': name, 'checkpoint': point, 'object': obj, 'field_index': index, 'detected': str(error)})
                    else:
                        raise ValueError('missed exact checkpoint mutation')
    # Execute altered inputs in each actual adapter. Build failures/timeouts do
    # not count: the retained log must identify the expected choice divergence.
    fixture = json.loads(FIXTURE.read_text())
    for mutation, case_id in [('omitted', 'exact_cub_cast_0'), ('extra', 'exact_cub_reject_0'), ('wrong_actor', 'exact_single_sentry_0')]:
        case = copy.deepcopy(next(c for c in fixture['cases'] if c['id'] == case_id))
        if mutation == 'omitted':
            del case['choices'][2]
            marker = '/choices/2'
        elif mutation == 'extra':
            case['choices'].append(dict(kind='pass', actor=1))
            marker = 'extra choices'
        else:
            case['choices'][7]['actor'] = 0
            marker = '/choices/7'
        altered = dict(fixture, cases=[case])
        folder = output / mutation
        folder.mkdir(exist_ok=True)
        input_path = folder / 'input.json'
        input_path.write_text(json.dumps(altered, indent=2) + '\n')
        cmd = xmage.maven(cache) + ['-o', '-pl', 'Mage.Tests', '-am', 'test',
            '-Dtest=org.mage.test.mtglab.ShivanTest', '-Dsurefire.failIfNoSpecifiedTests=false',
            '-Dmtglab.fixture=' + str(input_path), '-Dmtglab.output=' + str(folder),
            '-DargLine=-Djava.awt.headless=true']
        native_cmd = ['cargo', 'test', '-p', 'mtg-core', '--locked', '--lib',
            'game::shivan_tests::shivan_reference_literal_checkpoints', '--', '--exact']
        for engine, command, cwd, env in [
                ('xmage', cmd, cache / xmage.SOURCE, xmage.environment(cache)),
                ('native', native_cmd, ROOT, dict(os.environ, MTG_SHIVAN_FIXTURE=str(input_path)))]:
            log = folder / (engine + '.log')
            try:
                xmage.bounded(command, cwd, env, log, 300)
            except ValueError as error:
                required = 'choices/extra' if engine == 'native' and mutation == 'extra' else marker
                if 'command exited' not in str(error) or 'first divergence' not in log.read_text() or required not in log.read_text():
                    raise ValueError(f'{mutation}/{engine}: unrelated execution failure; {log}') from error
            else:
                raise ValueError(f'{mutation}/{engine}: invalid choice survived')
            controls.append(dict(mutation=mutation, engine=engine, input_sha256=xmage.sha(input_path), log_sha256=xmage.sha(log), first_divergence=required))
    receipt = dict(status='agreed', cases=len(canonical), repetitions=2,
        upstream_commit=xmage.scenario.load(ROOT / 'references/xmage/pins.json')['upstream_commit'],
        fixture_sha256=xmage.sha(FIXTURE), expected_sha256=xmage.sha(EXPECTED),
        pins_sha256=xmage.sha(ROOT / 'references/xmage/pins.json'), dependencies_sha256=xmage.sha(ROOT / 'references/xmage/dependencies.json'),
        rules_sha256=xmage.sha(ROOT / 'data/rules/cr-2026-09-25.json'), cards_sha256=xmage.sha(ROOT / 'data/cards/foundations_micro_v1.json'),
        bridge_sha256=xmage.sha(BRIDGE), runner_sha256=xmage.sha(Path(__file__)),
        native_sources={str(p.relative_to(ROOT)): xmage.sha(p) for p in sorted((ROOT / 'crates/mtg-core/src').glob('*.rs'))},
        runs=runs, controls=controls, stdin='closed', display='unset', offline=True,
        limitations='Synthetic control-age/departure positions, real pinned cards and rules. Exact cases observe scoped objects, mana, life, stack, actor, rejection and consumed choices; not full hidden state, RNG or full-game/Forge agreement. Native rejection additionally compares complete snapshots. Combat damage is observed from native applied marks before the queued death and XMage damage events. Legacy foreign target checks XMage target cardinality. GH-210 retains all 50-case/composition/holdout acceptance.')
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(f'{len(canonical)} Shivan/Cub cases agreed twice in native Rust and pinned XMage.')


if __name__ == '__main__':
    main()
