"""Bounded test-only XMage terminal runner; production engine is never imported."""
import argparse
import copy
import os
import shutil
import subprocess
import json
from pathlib import Path
import sys
import re
from instant_reference import difference

import xmage

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / 'fixtures/reference/terminal.json'


V2_FIXTURE = ROOT / 'fixtures/reference/terminal-v2.json'


def load_json(text):
    def unique(pairs):
        result = {}
        for key, value in pairs:
            if key in result:
                raise ValueError('terminal input: duplicate field ' + key)
            result[key] = value
        return result
    try:
        return json.loads(text, object_pairs_hook=unique)
    except json.JSONDecodeError as error:
        raise ValueError('terminal input: malformed JSON') from error


def validate(fixture):
    """Version 1 retains its historical P0 draw/P1 concession semantics explicitly."""
    def require(ok, message):
        if not ok:
            raise ValueError('terminal input: ' + message)
    def fields(obj, required, optional=()):
        require(type(obj) is dict, 'object required')
        require(not (obj.keys() - set(required) - set(optional)), 'unknown fields')
        require(set(required) <= obj.keys(), 'missing fields')
    fields(fixture, ['version', 'basis', 'cases'])
    version = fixture['version']
    require(type(version) is int and version in (1, 2), 'unsupported version')
    require(type(fixture['basis']) is str and bool(fixture['basis']), 'basis required')
    require(type(fixture['cases']) is list and bool(fixture['cases']), 'cases required')
    seen = set()
    for c in fixture['cases']:
        required = ['id', 'life', 'action', 'expected']
        optional = ['library_card']
        if version == 2:
            required += ['life_order', 'injection_order', 'expected_pending', 'expected_drawn']
            optional += ['draw_seat', 'concede_seat']
        fields(c, required, optional)
        require(type(c['id']) is str and re.fullmatch(r'[a-z0-9][a-z0-9_-]*', c['id']), 'invalid id')
        require(c['id'] not in seen, 'duplicate id')
        seen.add(c['id'])
        require(type(c['life']) is list and len(c['life']) == 2 and
                all(type(n) is int and -(2**31) <= n < 2**31 for n in c['life']), 'invalid life')
        require(c['action'] in ('settle', 'draw', 'concede'), 'invalid action')
        if 'library_card' in c:
            require(c['library_card'] == 'Forest' and c['action'] == 'draw', 'conflicting library_card')
        if version == 2:
            require(type(c['life_order']) is list and
                    all(type(n) is int for n in c['life_order']) and
                    c['life_order'] in ([0, 1], [1, 0]), 'invalid life_order')
            require(c['injection_order'] in ('life_then_draw', 'draw_then_life'), 'invalid injection_order')
            require(c['injection_order'] == 'life_then_draw' or c['action'] == 'draw', 'conflicting injection_order')
            for action, field in [('draw', 'draw_seat'), ('concede', 'concede_seat')]:
                require((field in c) == (c['action'] == action), 'conflicting or missing ' + field)
                if field in c:
                    require(type(c[field]) is int and c[field] in (0, 1), 'invalid seat: ' + field)
            require(c['action'] != 'concede' or c['life'] == [20, 20], 'conflicting concession and life loss')
            require((type(c['expected_drawn']) is int and c['expected_drawn'] in (0, 1))
                    if c['action'] == 'draw' else c['expected_drawn'] is None, 'invalid expected_drawn')
        for key in (['expected'] if version == 1 else ['expected', 'expected_pending']):
            e = c[key]
            fields(e, ['life', 'lost', 'library', 'hand'] +
                   (['outcome', 'winner', 'draw'] if version == 2 else []))
            for field in ['life', 'lost', 'library', 'hand']:
                require(type(e[field]) is list and len(e[field]) == 2 and
                        all(type(n) is (bool if field == 'lost' else int) for n in e[field]), 'invalid ' + key + '.' + field)
            if version == 2:
                require(e['outcome'] in ('ongoing', 'win', 'draw'), 'invalid outcome')
                require(e['winner'] is None or (type(e['winner']) is int and e['winner'] in (0, 1)), 'invalid winner')
                require(type(e['draw']) is bool, 'invalid draw')
    return fixture


def expected_results(fixture):
    validate(fixture)
    if fixture['version'] == 1:
        return {c['id']: c['expected'] for c in fixture['cases']}
    result = {}
    for c in fixture['cases']:
        life = [{'action': 'life', 'seat': seat, 'value': c['life'][seat]} for seat in c['life_order']]
        draw = ([{'action': 'draw', 'seat': c['draw_seat'], 'drawn': c['expected_drawn']}]
                if c['action'] == 'draw' else [])
        consumed = life + draw if c['injection_order'] == 'life_then_draw' else draw + life
        if c['action'] == 'concede':
            consumed += [{'action': 'concede', 'seat': c['concede_seat']}]
        consumed += [{'action': 'settle'}]
        result[c['id']] = {'version': 2, 'pending': c['expected_pending'],
                           'settled': c['expected'], 'consumed': consumed}
    return result


def compare(fixture, results):
    wanted = expected_results(fixture)
    if set(results) != set(wanted):
        raise ValueError('missing or extra terminal cases')
    diff = difference(wanted, results)
    if diff:
        raise ValueError('terminal checkpoint mismatch; first divergence: ' + json.dumps(diff, sort_keys=True))


def run_native(fixture, output, log, fault='none'):
    output.unlink(missing_ok=True)
    env = dict(os.environ, MTG_TERMINAL_INPUT=str(fixture), MTG_TERMINAL_OUTPUT=str(output),
               MTG_TERMINAL_FAULT=fault)
    xmage.bounded(['cargo', 'test', '-p', 'mtg-core', '--locked', '--lib',
                  'game::terminal::tests::terminal_export_observations', '--', '--exact'],
                 ROOT, env, log, 180)
    return load_json(output.read_text())


def run_xmage(cache, fixture, folder, log, fault='none', invalid=False):
    folder.mkdir(exist_ok=True)
    for old in folder.glob('*.json'):
        old.unlink()
    command = xmage.maven(cache) + ['-o', '-pl', 'Mage.Tests', '-am', 'test',
        '-Dtest=org.mage.test.mtglab.TerminalTest', '-Dsurefire.failIfNoSpecifiedTests=false',
        '-Dmtglab.fixture=' + str(fixture), '-Dmtglab.output=' + str(folder),
        '-Dmtglab.fault=' + fault, '-Dmtglab.invalid=' + str(invalid).lower(),
        '-DargLine=-Djava.awt.headless=true']
    xmage.bounded(command, cache / xmage.SOURCE, xmage.environment(cache), log, 300)
    return {p.stem: load_json(p.read_text()) for p in sorted(folder.glob('*.json'))}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cache', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True, help='JSON receipt; raw artifacts use a sibling directory')
    args = parser.parse_args()
    cache, output = args.cache.resolve(), args.output.resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    output.unlink(missing_ok=True)
    folder = output.with_suffix('')
    folder.mkdir(exist_ok=True)
    xmage.scenario.require(not cache.is_relative_to(ROOT), 'cache must be external')
    xmage.verify_inputs(cache)
    xmage.scenario.require(xmage.dependencies(cache) == xmage.scenario.load(ROOT/'references/xmage/dependencies.json'), 'dependency lock mismatch')
    bridge = ROOT/'references/xmage/TerminalTest.java'
    target = cache/xmage.SOURCE/'Mage.Tests/src/test/java/org/mage/test/mtglab/TerminalTest.java'
    target.write_bytes(bridge.read_bytes())
    runs = []
    for fixture in [FIXTURE, V2_FIXTURE]:
        document = validate(load_json(fixture.read_text()))
        shutil.copyfile(fixture, folder / fixture.name)
        for repeat in range(2):
            run = folder / f"v{document['version']}-{repeat}"
            run.mkdir(exist_ok=True)
            actual = run_xmage(cache, fixture, run/'xmage', run/'xmage.log')
            native = run_native(fixture, run/'native.json', run/'native.log')
            compare(document, actual)
            compare(document, native)
            runs.append({'version': document['version'], 'repeat': repeat,
                         'cases': len(actual), 'status': 'agreed'})
    document = load_json(V2_FIXTURE.read_text())
    canonical = expected_results(document)
    controls = []
    # Every scalar observation and consumed-choice leaf is independently corrupted.
    def paths(obj, path=()):
        if isinstance(obj, dict):
            for key, value in obj.items():
                yield from paths(value, path + (key,))
        elif isinstance(obj, list):
            for key, value in enumerate(obj):
                yield from paths(value, path + (key,))
        else:
            yield path
    for path in paths(canonical):
        wrong = copy.deepcopy(canonical)
        at = wrong
        for key in path[:-1]:
            at = at[key]
        value = at[path[-1]]
        at[path[-1]] = (not value if type(value) is bool else
                       value + 1 if type(value) is int else
                       0 if value is None else 'CORRUPTED')
        try:
            compare(document, wrong)
        except ValueError as error:
            controls.append({'path': list(path), 'detected': str(error)})
        else:
            raise ValueError('missed comparator control: ' + str(path))
    (folder/'comparator-controls.json').write_text(json.dumps(controls, indent=2)+'\n')
    # Execute two deliberately faulty adapters on one minimized mixed input.
    minimal = copy.deepcopy(document)
    minimal['cases'] = [next(c for c in document['cases'] if c['id'] == 'mixed-life-p0-draw-p1-life_then_draw')]
    path = folder/'minimal-mixed.json'
    path.write_text(json.dumps(minimal, indent=2)+'\n')
    faults = []
    for fault in ['wrong_draw_seat', 'premature_settlement']:
        run = folder/fault
        run.mkdir(exist_ok=True)
        for engine, actual in [('native', run_native(path, run/'native.json', run/'native.log', fault)),
                               ('xmage', run_xmage(cache, path, run/'xmage', run/'xmage.log', fault))]:
            try:
                compare(minimal, actual)
            except ValueError as error:
                faults.append({'fault': fault, 'engine': engine, 'detected': str(error)})
            else:
                raise ValueError('missed actual-engine fault: ' + fault + '/' + engine)
    invalid = ROOT/'fixtures/reference/terminal-invalid.json'
    rejected = run_xmage(cache, invalid, folder/'invalid', folder/'invalid.log', invalid=True)
    required = {c['id']: {'rejected': True} for c in json.loads(invalid.read_text())}
    if difference(required, rejected):
        raise ValueError('missing strict XMage parser rejection')
    shutil.copyfile(invalid, folder/invalid.name)
    # Hash inputs, toolchains, both native source and all retained raw observations.
    sources = [FIXTURE, V2_FIXTURE, invalid, bridge, Path(__file__),
               ROOT/'scripts/instant_reference.py', ROOT/'scripts/xmage.py', ROOT/'Cargo.lock',
               ROOT/'Cargo.toml', ROOT/'crates/mtg-core/Cargo.toml',
               ROOT/'references/xmage/terminal-provenance.json', ROOT/'tests/test_terminal_reference.py',
               ROOT/'references/xmage/pins.json', ROOT/'references/xmage/dependencies.json',
               ROOT/'data/rules/cr-2026-09-25.json', ROOT/'data/cards/foundations_micro_v1.json']
    sources += sorted((ROOT/'crates/mtg-core/src').glob('*.rs'))
    receipt = dict(version=2, status='agreed', repetitions=2, runs=runs, faults=faults,
        strict_rejections=len(rejected), comparator_controls=len(controls),
        upstream_commit=xmage.scenario.load(ROOT/'references/xmage/pins.json')['upstream_commit'],
        sources={str(p.relative_to(ROOT)):xmage.sha(p) for p in sources},
        rustc_version=subprocess.check_output(['rustc', '-Vv'], text=True).strip(),
        cargo_version=subprocess.check_output(['cargo', '-V'], text=True).strip(),
        toolchains={name:xmage.sha(Path(path)) for name,path in
                    [('java', str(Path(os.environ['JAVA_HOME'])/'bin/java')),
                     ('maven', shutil.which('mvn')),
                     ('rustc', subprocess.check_output(['rustup', 'which', 'rustc'], text=True).strip())]},
        artifacts={str(p.relative_to(folder)):xmage.sha(p) for p in sorted(folder.rglob('*')) if p.is_file()},
        stdin='closed', display='unset', offline=True,
        limitations='Synthetic pre-SBA injection with at most one attempted draw, both seats and injection orders. No production rules change, reachability, Forge, full-game or GH-212/M2 aggregate acceptance.')
    output.write_text(json.dumps(receipt, indent=2)+'\n')
    print(f'Seven legacy and {len(document["cases"])} version-2 terminal cases agreed twice in native Rust and pinned XMage.')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
