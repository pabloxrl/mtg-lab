"""Version 3 strict played-priority extension of the normal-reset runner."""
import copy
import json
from pathlib import Path
import full_pool_reference as base
import mulligan_reference as opening

ROOT = base.ROOT
FIXTURE = ROOT / 'fixtures/reference/full-pool-priority.json'
EXPECTED = ROOT / 'fixtures/reference/full-pool-priority-native.json'
REFERENCE_EXPECTED = ROOT / 'fixtures/reference/full-pool-priority-xmage.json'
NEGATIVES = ROOT / 'fixtures/reference/full-pool-priority-negative-inputs.json'
BRIDGE = ROOT / 'references/xmage/FullPoolPriorityTest.java'
require = base.require


def validate(doc):
    require(type(doc.get('schema_version')) is int and doc['schema_version'] == 3, '/schema_version')
    require(doc.get('family') == 'priority', '/family')
    projected = copy.deepcopy(doc)
    projected.update(schema_version=2, family='mulligan')
    for c in projected['cases']:
        require('play' in c, '/missing play')
        del c['play']
        c['stop'] = 'first_upkeep'
    opening.validate(projected)
    for c in doc['cases']:
        require(c['stop'] in ('first_cast_committed', 'second_creature_resolved'), '/unsupported stop')
        require(isinstance(c['play'], list) and bool(c['play']), '/missing play')
        for n, e in enumerate(c['play']):
            require(set(e) == {'sequence', 'turn', 'step', 'actor', 'kind', 'source', 'incarnation', 'color'}, '/play fields')
            require(type(e['sequence']) is int and e['sequence'] == n, '/play sequence')
            require(type(e['actor']) is int and e['actor'] in (0, 1), '/play actor')
            require(type(e['turn']) is int and e['turn'] > 0, '/play turn')
            require(isinstance(e['step'], str) and isinstance(e['kind'], str), '/play kind/step')


def negative_inputs(doc):
    require(doc == json.loads(FIXTURE.read_text()), '/negative fixture binding')
    return json.loads(NEGATIVES.read_text())


def compare(actual, engine='native'):
    require(engine in ('native', 'xmage'), '/unknown engine')
    expected = EXPECTED if engine == 'native' else REFERENCE_EXPECTED
    diff = base.instant_reference.difference(json.loads(expected.read_text()), actual)
    require(diff is None, '/checkpoints ' + json.dumps(diff, sort_keys=True))


def native(folder):
    return base.native(folder, family='priority')


def check_run(result, doc, engine):
    compare(result['checkpoints'], engine)
    require(set(result['runs']) == {c['id'] for c in doc['cases']}, '/case coverage')
    for case in doc['cases']:
        name = case['id']
        observed = result['runs'][name]
        require(observed['points'] == result['checkpoints'][name] and observed['points'], '/empty/unbound observations')
        require(observed['consumed_play'] == case['play'], '/actual played choices')
        for field in ('chance', 'choices'):
            require(observed['opening']['consumed_' + field] == case[field], '/actual opening ' + field)
        for variant in ('repeat_runs', 'continued_runs'):
            again = result[variant][name]
            for field in ('points', 'consumed_play'):
                require(observed[field] == again[field], '/' + variant + '/' + name + '/' + field)
        stop = result['continued_runs'][name]['stops']
        require(len(stop) == 1, '/named stop count')
        expected = copy.deepcopy(observed['points'][stop[0]['next_sequence']])
        expected['boundary'] = 'first_cast_committed'
        require(expected == stop[0]['checkpoint'], '/named stop checkpoint')
        if engine == 'native':
            require(observed['records'] and observed['policy_capture'], '/scalar recording missing')
            require(observed['stale_candidates_rejected'] == len(observed['records']), '/stale policy controls')
            require(result['continued_runs'][name]['continuation_rejections'] == 2, '/continuation controls')
        else:
            require(len(observed['raw_occurrence_bindings']) == 80 and len(observed['raw_stack_ids']) == 4, '/identity provenance')
            casts = [e for e in case['play'] if e['kind'] == 'cast']
            checks = observed['selected_cast_checks']
            require(len(checks) == len(casts), '/selected cast legality coverage')
            for event, check in zip(casts, checks):
                require(check['source'] == event['source'] and check['action'] == event['sequence']
                        and check['candidate_count'] == 1 and check['candidate_id'], '/selected cast engine legality')
            incarnations = {b['occurrence']: 0 for b in observed['raw_occurrence_bindings'].values()}
            for transition in observed['identity_transitions']:
                source = transition['source']
                require(transition['from'] != transition['to'], '/unwitnessed incarnation transition')
                delta = transition['raw_after'] - transition['raw_before']
                require(delta == (0 if transition['direct_mulligan_return'] else 1), '/raw incarnation counter')
                if transition['direct_mulligan_return']:
                    require((transition['from'], transition['to']) == ('HAND', 'LIBRARY'), '/direct return provenance')
                incarnations[source] += 1
                require(incarnations[source] == transition['incarnation'], '/canonical incarnation sequence')
            require(incarnations == observed['points'][-1]['incarnations'], '/final incarnation provenance')
    require(set(result['rejections']) == set(negative_inputs(doc)), '/negative controls')
    require(all(v.startswith('first divergence:') for v in result['rejections'].values()), '/negative diagnostics')
    if engine == 'native':
        require(result['rejection_state_rng'] == 'unchanged', '/native atomic rejection')
    else:
        require(set(result['callback_controls']) == {'unscripted_mana', 'unexpected_target', 'unexpected_blockers'}, '/callback controls')
        for name in ('off_turn_creature', 'insufficient_mana'):
            require('/selected cast not playable' in result['rejections'][name], '/intended rejection boundary ' + name)


def comparator_controls(points):
    """Mutate actual observations; retain the exact first divergent field."""
    controls = {}
    case = next(iter(points))
    def probe(name, mutate, expected_path):
        bad = copy.deepcopy(points)
        mutate(bad[case])
        diff = base.instant_reference.difference(points, bad)
        require(diff is not None and expected_path in diff['path'], '/first field control ' + name)
        controls[name] = diff
    for field in points[case][0]:
        probe('missing-' + field, lambda x, f=field: x[0].pop(f), '')
    draw = next(i for i, p in enumerate(points[case]) if p['step'] == 'draw')
    probe('first-draw', lambda x: x[draw]['hand'][x[draw]['active']].__setitem__(-1, 'wrong/occurrence'), '.hand')
    probe('first-priority', lambda x: x[0].__setitem__('actor', 1-x[0]['actor']), '.actor')
    stack = next(i for i, p in enumerate(points[case]) if p['stack'])
    probe('stack-action', lambda x: x[stack]['stack'][0].__setitem__('action', -1), '.stack[0].action')
    probe('stack-incarnation', lambda x: x[stack]['stack'][0].__setitem__('incarnation', -1), '.stack[0].incarnation')
    source = points[case][stack]['stack'][0]['source']
    probe('source-incarnation', lambda x: x[stack]['incarnations'].__setitem__(source, -1), '.incarnations')
    def swap(row):
        row[0], row[1] = row[1], row[0]
    probe('ordered-library', lambda x: swap(x[0]['library'][0]), '.library[0][0]')
    probe('ordered-hand', lambda x: swap(x[0]['hand'][0]), '.hand[0][0]')
    battlefield = next(i for i, p in enumerate(points[case]) if len(p['battlefield']) >= 2)
    probe('ordered-battlefield', lambda x: swap(x[battlefield]['battlefield']), '.battlefield[0]')
    # Vanilla creatures cannot share a stack under sorcery timing. This ordered
    # comparator probe uses two separately witnessed stack entries; it does not
    # manufacture a played simultaneous-stack observation.
    entries = []
    for point in points[case]:
        for entry in point['stack']:
            if entry not in entries:
                entries.append(entry)
    require(len(entries) >= 2, '/distinct actual stack entries')
    diff = base.instant_reference.difference({'stack': entries[:2]}, {'stack': list(reversed(entries[:2]))})
    require(diff is not None and '.stack[0].' in diff['path'], '/ordered stack comparator')
    controls['stack-order-synthetic-from-witnessed-entries'] = diff
    return controls


def source_files():
    """Executable and oracle inputs covered by each priority receipt."""
    return [FIXTURE, EXPECTED, REFERENCE_EXPECTED, NEGATIVES, BRIDGE, opening.BRIDGE,
               Path(__file__).resolve(), ROOT / 'fixtures/reference/author_priority.py',
               ROOT / 'fixtures/reference/full-pool-priority-final.json',
               ROOT / 'fixtures/reference/full-pool-priority-oracle.md',
               ROOT / 'scripts/full_pool_reference.py', ROOT / 'scripts/mulligan_reference.py',
               ROOT / 'Cargo.lock', ROOT / 'Cargo.toml', ROOT / 'crates/mtg-core/Cargo.toml',
               ROOT / 'scripts/xmage.py', ROOT / 'scripts/instant_reference.py', ROOT / 'scripts/scenario.py',
               ROOT / 'references/xmage/pins.json', ROOT / 'references/xmage/dependencies.json',
               ROOT / 'references/xmage/mulligan-provenance.json',
               ROOT / 'references/xmage/priority-provenance.json',
               *sorted((ROOT / 'crates/mtg-core/src').rglob('*.rs'))]


def run(args):
    import os
    import shutil
    import subprocess
    import sys
    import tempfile
    import xmage
    output = args.output.resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    output.unlink(missing_ok=True)
    folder = Path(tempfile.mkdtemp(prefix=output.stem + '.privileged-', dir=output.parent))
    os.chmod(folder, 0o700)
    doc = json.loads(FIXTURE.read_text())
    validate(doc)
    for source, name in [(FIXTURE, 'input.json'), (EXPECTED, 'native-oracle.json'),
                         (REFERENCE_EXPECTED, 'xmage-oracle.json')]:
        (folder / name).write_bytes(source.read_bytes())
    sources = source_files()
    hashes = {str(p.relative_to(ROOT)): xmage.sha(p) for p in sources}
    native_result = native(folder)
    check_run(native_result, doc, 'native')
    cache = args.cache.resolve()
    xmage.verify_inputs(cache)
    require(xmage.dependencies(cache) == json.loads((ROOT / 'references/xmage/dependencies.json').read_text()), '/dependencies')
    provenance = json.loads((ROOT / 'references/xmage/mulligan-provenance.json').read_text())
    for p, sha in provenance['consulted_sources'].items():
        require(xmage.sha(cache / xmage.SOURCE / p) == sha, '/upstream source ' + p)
    provenance = json.loads((ROOT / 'references/xmage/priority-provenance.json').read_text())
    for p, sha in provenance['consulted_sources'].items():
        require(xmage.sha(cache / xmage.SOURCE / p) == sha, '/upstream source ' + p)
    for bridge in (opening.BRIDGE, BRIDGE):
        target = cache / xmage.SOURCE / 'Mage.Tests/src/test/java/org/mage/test/mtglab' / bridge.name
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(bridge.read_bytes())
    result_path = folder / 'xmage.json'
    command = xmage.maven(cache) + ['-o', '-pl', 'Mage.Tests', '-am', 'test',
        '-Dtest=org.mage.test.mtglab.FullPoolPriorityTest', '-Dsurefire.failIfNoSpecifiedTests=false',
        '-Dmtglab.fixture=' + str(FIXTURE), '-Dmtglab.root=' + str(ROOT),
        '-Dmtglab.negatives=' + str(folder / 'negative-inputs.json'), '-Dmtglab.output=' + str(result_path),
        '-DargLine=-Djava.awt.headless=true']
    xmage.bounded(command, cache / xmage.SOURCE, xmage.environment(cache), folder / 'xmage.log', 600)
    reference = json.loads(result_path.read_text())
    check_run(reference, doc, 'xmage')
    # Both engines are checked against separately authored raw staging oracles.
    # Only committed states and the explicit payment ledger are shared contracts.
    for name, native_points in native_result['checkpoints'].items():
        reference_points = reference['checkpoints'][name]
        require(len(native_points) == len(reference_points), '/checkpoint count')
        for n, (left, right) in enumerate(zip(native_points, reference_points)):
            require(left['payment'] == right['payment'], f'/payment/{name}/{n}')
            if left['payment'] is None:
                for field in left:
                    if field == 'hand':
                        require([sorted(h) for h in left[field]] == [sorted(h) for h in right[field]], f'/hand-membership/{name}/{n}')
                    else:
                        require(left[field] == right[field], f'/committed/{name}/{n}/{field}')
    controls = comparator_controls(native_result['checkpoints'])
    (folder / 'comparator-controls.json').write_text(json.dumps(controls, indent=2) + '\n')
    toolchain = {}
    for name, command in [('java', ['java', '-version']), ('maven', ['mvn', '-version']),
                          ('rustc', ['rustc', '--version']), ('python', [sys.executable, '--version'])]:
        p = subprocess.run(command, capture_output=True, text=True, timeout=30, check=True, stdin=subprocess.DEVNULL)
        toolchain[name] = {'version': p.stdout + p.stderr,
                          'executable_sha256': xmage.sha(Path(shutil.which(command[0])).resolve())}
    (folder / 'toolchain.json').write_text(json.dumps(toolchain, indent=2) + '\n')
    for path in folder.iterdir():
        if path.is_file():
            os.chmod(path, 0o600)
    require(hashes == {str(p.relative_to(ROOT)): xmage.sha(p) for p in sources}, '/source changed during execution')
    receipt = dict(schema_version=3, family='priority', status='agreed', cases=len(doc['cases']),
        runs_per_engine=3 * len(doc['cases']), completion='second_creature_resolved_prefix',
        limits='Raw in-flight transaction/announcement and opening hand presentation orders have distinct literal oracles. Canonical incarnations count witnessed zone transitions; raw counters are retained, including XMage direct mulligan returns without counter increments. Committed fields and payment ledgers agree, with hand membership compared across engines and each raw hand order independently asserted. No complete legal-set, reference rollback/internal RNG, nonempty combat, noncreature, trigger or full-game claim. Stack-order mutation is a synthetic comparator probe using separately witnessed entries.',
        pins=doc['pins'], upstream_commit=json.loads((ROOT / 'references/xmage/pins.json').read_text())['upstream_commit'],
        sources=hashes, privileged_directory=folder.name,
        privileged_artifacts={p.name: xmage.sha(p) for p in folder.iterdir() if p.is_file()},
        comparator_controls_detected=len(controls), negative_controls=list(reference['rejections']),
        acceptance_claims={
            'selected_actions': 'Native semantic records/scalar submissions and pinned XMage player commands accepted.',
            'native_rejections': 'Production decoding/application rejects illegal records and stale candidates with unchanged state/RNG.',
            'reference_rejections': 'Raw land rejection and selected-cast membership in the engine playable-action query, followed by accepted cast submission; the low-level cast method assumes caller legality. Payment guards inspect witnessed pool/unpaid cost. No raw off-turn cast-call or wrong-color payment-call rejection claim.',
            'complete_legal_set': False},
        callback_controls=list(reference['callback_controls']), stdin='closed', display='unset', offline=True)
    output.write_text(json.dumps(receipt, indent=2) + '\n')
    print('Six real played prefixes agreed; explicit choices, repeats, strict continuation and negative controls passed.')
