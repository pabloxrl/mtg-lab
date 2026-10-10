"""Versioned test-only native/XMage reset, London and played-priority prefixes."""
import argparse
import copy
import json
import os
import shutil
from pathlib import Path
import subprocess
import sys
import tempfile
import instant_reference
import xmage

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / 'fixtures/reference/full-pool-reset.json'
EXPECTED = ROOT / 'fixtures/reference/full-pool-reset-expectations.json'
BRIDGE = ROOT / 'references/xmage/FullPoolResetTest.java'
PIN_PATHS = ('data/cards/foundations_micro_v1.json', 'data/rules/cr-2026-09-25.json', 'references/xmage/pins.json')


def require(ok, path):
    if not ok:
        raise ValueError('first divergence: ' + path)


def validate(doc):
    require(isinstance(doc, dict) and set(doc) == {'schema_version', 'family', 'pins', 'cases'}, '/envelope')
    require(type(doc['schema_version']) is int and doc['schema_version'] == 1, '/schema_version')
    require(doc['family'] == 'reset', '/family unsupported')
    require(doc['pins'] == {p: xmage.sha(ROOT / p) for p in PIN_PATHS}, '/pins')
    manifest = json.loads((ROOT / PIN_PATHS[0]).read_text())
    decks = {d['id']: d for d in manifest['decks']}
    require(isinstance(doc['cases'], list) and bool(doc['cases']), '/cases empty')
    ids = set()
    for case in doc['cases']:
        require(isinstance(case, dict) and set(case) == {'id', 'starter', 'decks', 'chance', 'choices', 'stop'}, '/case fields')
        require(isinstance(case['id'], str) and case['id'] and case['id'] not in ids, '/case id')
        ids.add(case['id'])
        require(type(case['starter']) is int and case['starter'] in (0, 1), '/starter')
        require(case['stop'] == 'first_declaration', '/stop unsupported callback')
        require(case['choices'] == [], '/choices unsupported mulligan/ongoing callback')
        require(isinstance(case['decks'], list) and len(case['decks']) == 2, '/decks')
        require(isinstance(case['chance'], list) and len(case['chance']) == 2, '/chance length')
        for seat, deck in enumerate(case['decks']):
            require(isinstance(deck, dict) and set(deck) == {'deck', 'occurrences'}, '/deck fields')
            require(isinstance(deck['deck'], str) and deck['deck'] in decks, '/deck unknown')
            canonical = [f'{seat}/{e["card_id"]}/{i}' for e in decks[deck['deck']]['cards'] for i in range(e['copies'])]
            require(deck['occurrences'] == canonical, f'/decks/{seat}/occurrences')
            event = case['chance'][seat]
            require(isinstance(event, dict) and set(event) == {'sequence', 'kind', 'actor', 'before', 'after'}, '/chance fields')
            require(type(event['sequence']) is int and event['sequence'] == seat, '/chance sequence')
            require(type(event['actor']) is int and event['actor'] == seat, '/chance actor')
            require(event['kind'] == 'initial_shuffle', '/chance kind')
            require(event['before'] == canonical, '/chance pre-event occurrences')
            require(isinstance(event['after'], list) and all(isinstance(o, str) for o in event['after']) and sorted(event['after']) == sorted(canonical), '/chance permutation')


def negative_inputs(doc):
    out = {}
    def add(name, mutate):
        bad = copy.deepcopy(doc)
        bad['cases'] = [bad['cases'][0]]
        mutate(bad, bad['cases'][0])
        out[name] = bad
    add('39_cards', lambda d, c: c['decks'][0]['occurrences'].pop())
    add('41_cards', lambda d, c: c['decks'][0]['occurrences'].append('0/mountain/16'))
    add('token', lambda d, c: c['decks'][0]['occurrences'].__setitem__(0, '0/goblin-token/0'))
    add('pin', lambda d, c: d['pins'].__setitem__(PIN_PATHS[0], 'unknown'))
    add('duplicate_occurrence', lambda d, c: c['decks'][0]['occurrences'].__setitem__(1, c['decks'][0]['occurrences'][0]))
    add('missing_occurrence', lambda d, c: c['chance'][0]['after'].pop())
    add('duplicate_permutation', lambda d, c: c['chance'][0]['after'].__setitem__(1, c['chance'][0]['after'][0]))
    add('actor', lambda d, c: c['chance'][0].__setitem__('actor', 1))
    add('kind', lambda d, c: c['chance'][0].__setitem__('kind', 'mulligan_shuffle'))
    add('sequence', lambda d, c: c['chance'][0].__setitem__('sequence', 1))
    add('before', lambda d, c: c['chance'][0]['before'].__setitem__(0, '0/mountain/99'))
    add('missing_event', lambda d, c: c['chance'].pop())
    add('extra_event', lambda d, c: c['chance'].append(copy.deepcopy(c['chance'][0])))
    add('reordered_event', lambda d, c: c['chance'].reverse())
    add('mulligan', lambda d, c: c['choices'].append({'kind': 'mulligan', 'actor': 0}))
    add('ongoing', lambda d, c: c.__setitem__('stop', 'upkeep'))
    add('schema', lambda d, c: d.__setitem__('schema_version', 2))
    add('boolean_sequence', lambda d, c: c['chance'][0].__setitem__('sequence', False))
    add('unknown_field', lambda d, c: c.__setitem__('fallback', True))
    return out


def compare(actual):
    diff = instant_reference.difference(json.loads(EXPECTED.read_text()), actual)
    if diff:
        raise ValueError('first divergence: ' + json.dumps(diff, sort_keys=True))


def native(folder, family='reset'):
    """Run the real Rust test client, including its runtime negative inputs."""
    if family == 'reset':
        family_module = sys.modules[__name__]
    elif family == 'mulligan':
        import mulligan_reference as family_module
    elif family == 'priority':
        import priority_reference as family_module
    elif family == 'spells':
        import spells_reference as family_module
    else:
        raise ValueError('unsupported family')
    folder = folder.resolve()
    folder.mkdir(parents=True, exist_ok=True)
    output = folder / 'native.json'
    output.unlink(missing_ok=True)
    negatives = folder / 'negative-inputs.json'
    negatives.write_text(json.dumps(family_module.negative_inputs(json.loads(family_module.FIXTURE.read_text()))))
    env = dict(os.environ, MTG_FULL_POOL_OUTPUT=str(output), MTG_FULL_POOL_NEGATIVES=str(negatives), MTG_FULL_POOL_FAMILY=family)
    command = ['cargo', 'test', '-p', 'mtg-core', '--locked', '--lib', 'game::full_pool_reference_tests::full_pool_' + family + '_literal_checkpoints', '--', '--exact']
    # Torture already holds the shared lock; do not deadlock by reacquiring it.
    fd = env.get('MTG_SYMPHONY_LOCK_FD')
    pass_fds = ()
    if fd is not None:
        try:
            os.fstat(int(fd))
            pass_fds = (int(fd),)
        except OSError:
            fd = None
    if fd is None:
        # The toolchain-only CI image declares a controller root but does not
        # contain that checkout. This checked-in helper uses the same shared
        # lock directory and policy in every environment.
        command = [sys.executable, str(ROOT / 'scripts/symphony/resource_lock.py'), 'heavy', '--'] + command
    with (folder / 'native.log').open('w') as log:
        try:
            subprocess.run(command, cwd=ROOT, env=env, stdin=subprocess.DEVNULL, stdout=log, stderr=subprocess.STDOUT, timeout=3600, check=True, pass_fds=pass_fds)
        except subprocess.CalledProcessError as error:
            log.flush()
            raise ValueError('native client failed: ' + (folder / 'native.log').read_text()[-7000:]) from error
    require(output.exists(), '/native missing observations')
    result = json.loads(output.read_text())
    family_module.compare(result['checkpoints'])
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--family', choices=['reset', 'mulligan', 'priority', 'spells'], required=True)
    parser.add_argument('--cache', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    if args.family == 'spells':
        import spells_reference
        return spells_reference.run(args)
    if args.family == 'priority':
        import priority_reference
        return priority_reference.run(args)
    if args.family == 'mulligan':
        import mulligan_reference
        return mulligan_reference.run(args)
    output = args.output.resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    output.unlink(missing_ok=True)
    folder = Path(tempfile.mkdtemp(prefix=output.stem + '.privileged-', dir=output.parent))
    os.chmod(folder, 0o700)
    doc = json.loads(FIXTURE.read_text())
    (folder / 'input.json').write_bytes(FIXTURE.read_bytes())
    (folder / 'independent-oracle.json').write_bytes(EXPECTED.read_bytes())
    validate(doc)
    sources = [FIXTURE, EXPECTED, BRIDGE, Path(__file__).resolve(),
               ROOT / 'Cargo.lock', ROOT / 'Cargo.toml', ROOT / 'crates/mtg-core/Cargo.toml',
               ROOT / 'scripts/xmage.py', ROOT / 'scripts/instant_reference.py', ROOT / 'scripts/scenario.py',
               ROOT / 'fixtures/reference/full-pool-reset-negative-inputs.json',
               ROOT / 'references/xmage/pins.json', ROOT / 'references/xmage/dependencies.json',
               ROOT / 'references/xmage/full-pool-provenance.json',
               *sorted((ROOT / 'crates/mtg-core/src').glob('*.rs'))]
    source_hashes = {str(p.relative_to(ROOT)): xmage.sha(p) for p in sources}
    native_result = native(folder)
    require(native_result['repeat_checkpoints'] == native_result['checkpoints'] and native_result['stale_handles_rejected'] == 1280, '/native repeat/stale handles')
    cache = args.cache.resolve()
    xmage.verify_inputs(cache)
    require(xmage.dependencies(cache) == json.loads((ROOT / 'references/xmage/dependencies.json').read_text()), '/dependencies')
    target = cache / xmage.SOURCE / 'Mage.Tests/src/test/java/org/mage/test/mtglab/FullPoolResetTest.java'
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(BRIDGE.read_bytes())
    result_path = folder / 'xmage.json'
    result_path.unlink(missing_ok=True)
    command = xmage.maven(cache) + ['-o', '-pl', 'Mage.Tests', '-am', 'test', '-Dtest=org.mage.test.mtglab.FullPoolResetTest', '-Dsurefire.failIfNoSpecifiedTests=false', '-Dmtglab.fixture=' + str(FIXTURE), '-Dmtglab.root=' + str(ROOT), '-Dmtglab.negatives=' + str(folder / 'negative-inputs.json'), '-Dmtglab.output=' + str(result_path), '-DargLine=-Djava.awt.headless=true']
    xmage.bounded(command, cache / xmage.SOURCE, xmage.environment(cache), folder / 'xmage.log', 600)
    reference = json.loads(result_path.read_text())
    compare(reference['checkpoints'])
    require(reference['checkpoints'] == reference['repeat_checkpoints'], '/xmage repeat')
    require(set(reference['rejections']) == set(native_result['rejections']), '/rejections')
    require(set(reference['callback_controls']) == {'priority', 'mulligan_again'} and all(v.startswith('first divergence:') for v in reference['callback_controls'].values()), '/unsupported callback controls')
    controls = []
    for field in next(iter(native_result['checkpoints'].values())):
        wrong = copy.deepcopy(native_result['checkpoints'])
        del wrong['red-green-0'][field]
        try:
            compare(wrong)
        except ValueError as error:
            controls.append(str(error))
        else:
            raise ValueError('missed field mutation: ' + field)
    wrong = copy.deepcopy(native_result['checkpoints'])
    row = wrong['red-green-0']['library'][0]
    row[-1], row[-2] = row[-2], row[-1]
    try:
        compare(wrong)
    except ValueError as error:
        controls.append(str(error))
    else:
        raise ValueError('missed same-name occurrence mutation')
    (folder / 'comparator-controls.json').write_text(json.dumps(controls, indent=2) + '\n')
    (folder / 'input.json').write_bytes(FIXTURE.read_bytes())
    (folder / 'independent-oracle.json').write_bytes(EXPECTED.read_bytes())
    toolchain = {}
    for name, command in [('java', ['java', '-version']), ('maven', ['mvn', '-version']), ('rustc', ['rustc', '--version']), ('python', [sys.executable, '--version'])]:
        proc = subprocess.run(command, capture_output=True, text=True, timeout=30, check=True, stdin=subprocess.DEVNULL)
        executable = Path(shutil.which(command[0])).resolve()
        toolchain[name] = {'version': proc.stdout + proc.stderr, 'executable_sha256': xmage.sha(executable)}
    (folder / 'toolchain.json').write_text(json.dumps(toolchain, indent=2) + '\n')
    for path in folder.iterdir():
        if path.is_file():
            os.chmod(path, 0o600)
    require(source_hashes == {str(p.relative_to(ROOT)): xmage.sha(p) for p in sources}, '/source changed during reference execution')
    receipt = dict(schema_version=1, family='reset', status='agreed', cases=16, resets_per_engine=32, completion='prefix',
        limits='First declaration only; no submitted mulligan, ongoing choices, full game, Forge or complete legal-set claim.',
        pins=doc['pins'], upstream_commit=json.loads((ROOT / 'references/xmage/pins.json').read_text())['upstream_commit'],
        sources=source_hashes,
        privileged_directory=folder.name,
        privileged_artifacts={p.name: xmage.sha(p) for p in folder.iterdir() if p.is_file()},
        comparator_controls_detected=len(controls), negative_controls=list(reference['rejections']), callback_controls=list(reference['callback_controls']),
        stdin='closed', display='unset', offline=True)
    output.write_text(json.dumps(receipt, indent=2) + '\n')
    print('16 normal-reset prefixes agreed; repeat resets and strict negative controls passed.')


if __name__ == '__main__':
    main()
