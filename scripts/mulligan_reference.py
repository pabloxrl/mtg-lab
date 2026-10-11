"""Version 2 London extension of the test-only full-pool reference runner."""
import copy
import json
from pathlib import Path
import full_pool_reference as base

ROOT = base.ROOT
FIXTURE = ROOT / 'fixtures/reference/full-pool-mulligan.json'
EXPECTED = ROOT / 'fixtures/reference/full-pool-mulligan-expectations.json'
BRIDGE = ROOT / 'references/xmage/FullPoolMulliganTest.java'
require = base.require


def reset_projection(doc):
    result = copy.deepcopy(doc)
    result['schema_version'] = 1
    result['family'] = 'reset'
    for c in result['cases']:
        c['stop'] = 'first_declaration'
        c['choices'] = []
        c['chance'] = c['chance'][:2]
        for e in c['chance']:
            e.pop('source', None)
    return result


def validate(doc):
    require(type(doc.get('schema_version')) is int and doc['schema_version'] == 2, '/schema_version')
    require(doc.get('family') == 'mulligan', '/family')
    base.validate(reset_projection(doc))
    for c in doc['cases']:
        require(c['stop'] == 'first_upkeep', '/stop unsupported callback')
        for n, e in enumerate(c['chance']):
            require(set(e) == {'sequence', 'kind', 'actor', 'source', 'before', 'after'}, '/chance fields')
            require(type(e['sequence']) is int and e['sequence'] == n, '/chance sequence')
            require(type(e['actor']) is int and e['actor'] in (0, 1), '/chance actor')
            require(e['source'] is None, '/chance source')
            require(e['kind'] == ('initial_shuffle' if n < 2 else 'mulligan_shuffle'), '/chance kind')
            canonical = c['decks'][e['actor']]['occurrences']
            require(e['before'] == canonical and sorted(e['after']) == sorted(canonical), '/chance occurrences')
        require(isinstance(c['choices'], list) and bool(c['choices']), '/choices missing')
        for n, e in enumerate(c['choices']):
            require(set(e) == {'sequence', 'kind', 'actor', 'source', 'round', 'selection'}, '/choice fields')
            require(type(e['sequence']) is int and e['sequence'] == n, '/choice sequence')
            require(type(e['actor']) is int and e['actor'] in (0, 1), '/choice actor')
            require(e['source'] is None, '/choice source')
            require(type(e['round']) is int and 0 <= e['round'] <= 7, '/choice round')
            require(e['kind'] in ('declare', 'bottom'), '/choice kind')
            if e['kind'] == 'declare':
                require(e['selection'] in ('keep', 'mulligan'), '/declaration selection')
            else:
                require(isinstance(e['selection'], list) and all(isinstance(x, str) for x in e['selection']), '/bottom type')
                require(len(e['selection']) == e['round'] and len(set(e['selection'])) == len(e['selection']), '/bottom cardinality/duplicate')
                require(all(x in c['decks'][e['actor']]['occurrences'] for x in e['selection']), '/bottom occurrence')


def negative_inputs(doc):
    out = {}
    def add(name, mutate):
        bad = copy.deepcopy(doc)
        # Unequal multiple mulligans exercises both seats and a cumulative bottom.
        bad['cases'] = [bad['cases'][2]]
        mutate(bad, bad['cases'][0])
        out[name] = bad
    for kind in ('chance', 'choices'):
        add(kind + '_missing', lambda d, c, k=kind: c[k].pop())
        add(kind + '_extra', lambda d, c, k=kind: c[k].append(copy.deepcopy(c[k][-1])))
        add(kind + '_reordered', lambda d, c, k=kind: c[k].reverse())
        for field,value in [('actor',1),('kind','unexpected'),('source','0/mountain/0'),('sequence',False)]:
            add(kind+'_'+field, lambda d,c,k=kind,f=field,v=value:c[k][0].__setitem__(f,v))
    add('stale_round', lambda d,c:c['choices'][2].__setitem__('round',0))
    add('stale_occurrence', lambda d,c:c['choices'][2]['selection'].__setitem__(0,c['decks'][0]['occurrences'][-1]))
    add('bottom_missing', lambda d,c:c['choices'][-2]['selection'].pop())
    add('bottom_duplicate', lambda d,c:c['choices'][-2]['selection'].__setitem__(1,c['choices'][-2]['selection'][0]))
    add('bottom_before_declaration', lambda d,c:c['choices'].__setitem__(0,copy.deepcopy(c['choices'][2])))
    add('shuffle_before_wrong_seat', lambda d,c:c['chance'][2].__setitem__('actor',1))
    add('shuffle_wrong_permutation', lambda d,c:c['chance'][2]['after'].__setitem__(1,c['chance'][2]['after'][0]))
    add('shuffle_missing_redraw', lambda d,c:c['chance'].__delitem__(2))
    add('incomplete_ledger', lambda d,c:c['choices'].__delitem__(slice(-2,None)))
    add('unexpected_callback', lambda d,c:c.__setitem__('stop','first_draw'))
    add('schema', lambda d,c:d.__setitem__('schema_version',1))
    add('chance_missing_source', lambda d,c:c['chance'][2].pop('source'))
    add('choice_missing_source', lambda d,c:c['choices'][2].pop('source'))
    add('mulligan_shuffle_source', lambda d,c:c['chance'][2].__setitem__('source','0/mountain/0'))
    add('mulligan_shuffle_kind', lambda d,c:c['chance'][2].__setitem__('kind','initial_shuffle'))
    add('bottom_source', lambda d,c:c['choices'][2].__setitem__('source','0/mountain/0'))
    def swap(c, key, a, b):
        c[key][a], c[key][b] = c[key][b], c[key][a]
        for n, event in enumerate(c[key]):
            event['sequence'] = n
    add('runtime_shuffle_reorder', lambda d,c:swap(c,'chance',2,3))
    add('runtime_choice_reorder', lambda d,c:swap(c,'choices',1,2))
    # Keep numbering valid so these controls exercise live chronology and
    # complete consumption, rather than only failing a sequence-number parser.
    for name in ('chance_extra', 'chance_missing', 'choices_extra',
                 'choices_missing', 'choices_reordered', 'bottom_before_declaration'):
        key = 'chance' if name.startswith('chance_') else 'choices'
        for n, event in enumerate(out[name]['cases'][0][key]):
            event['sequence'] = n
    return out


def compare(actual):
    diff = base.instant_reference.difference(json.loads(EXPECTED.read_text()), actual)
    require(diff is None, '/checkpoints ' + json.dumps(diff, sort_keys=True))


def native(folder):
    return base.native(folder, family='mulligan')


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
    (folder / 'input.json').write_bytes(FIXTURE.read_bytes())
    (folder / 'independent-oracle.json').write_bytes(EXPECTED.read_bytes())
    sources = [FIXTURE, EXPECTED, BRIDGE, Path(__file__).resolve(),
               ROOT / 'scripts/full_pool_reference.py', ROOT / 'Cargo.lock', ROOT / 'Cargo.toml',
               ROOT / 'crates/mtg-core/Cargo.toml', ROOT / 'scripts/xmage.py',
               ROOT / 'scripts/instant_reference.py', ROOT / 'scripts/scenario.py',
               ROOT / 'fixtures/reference/full-pool-mulligan-negative-inputs.json',
               ROOT / 'references/xmage/pins.json', ROOT / 'references/xmage/dependencies.json',
               ROOT / 'references/xmage/mulligan-provenance.json',
               *sorted((ROOT / 'crates/mtg-core/src').glob('*.rs'))]
    hashes = {str(p.relative_to(ROOT)): xmage.sha(p) for p in sources}
    native_result = native(folder)
    cache = args.cache.resolve()
    xmage.verify_inputs(cache)
    require(xmage.dependencies(cache) == json.loads((ROOT / 'references/xmage/dependencies.json').read_text()), '/dependencies')
    provenance = json.loads((ROOT / 'references/xmage/mulligan-provenance.json').read_text())
    for p, sha in provenance['consulted_sources'].items():
        require(xmage.sha(cache / xmage.SOURCE / p) == sha, '/upstream source ' + p)
    target = cache / xmage.SOURCE / 'Mage.Tests/src/test/java/org/mage/test/mtglab/FullPoolMulliganTest.java'
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(BRIDGE.read_bytes())
    result_path = folder / 'xmage.json'
    command = xmage.maven(cache) + ['-o', '-pl', 'Mage.Tests', '-am', 'test',
        '-Dtest=org.mage.test.mtglab.FullPoolMulliganTest', '-Dsurefire.failIfNoSpecifiedTests=false',
        '-Dmtglab.fixture=' + str(FIXTURE), '-Dmtglab.root=' + str(ROOT),
        '-Dmtglab.negatives=' + str(folder / 'negative-inputs.json'), '-Dmtglab.output=' + str(result_path),
        '-DargLine=-Djava.awt.headless=true']
    xmage.bounded(command, cache / xmage.SOURCE, xmage.environment(cache), folder / 'xmage.log', 600)
    reference = json.loads(result_path.read_text())
    compare(reference['checkpoints'])
    for result in (native_result, reference):
        for name, observed in result['runs'].items():
            again = result['repeat_runs'][name]
            # Raw states before the other seat's initial shuffle contain the
            # pinned engine's unordered Deck set. Retain, but do not normalize
            # or claim repeat equality for that explicitly unordered data.
            for field in ('points', 'consumed_chance', 'consumed_choices'):
                require(observed[field] == again[field], '/repeat ' + name + '/' + field)
        for case in doc['cases']:
            run_result = result['runs'][case['id']]
            require(run_result['consumed_chance'] == case['chance'], '/actual chance ledger')
            require(run_result['consumed_choices'] == case['choices'], '/actual choice ledger')
            require(run_result['raw_callbacks'], '/raw callbacks missing')
        require(set(result['rejections']) == set(negative_inputs(doc)), '/negative controls')
        require(all(v.startswith('first divergence:') for v in result['rejections'].values()), '/rejection diagnostics')
    require(set(reference['callback_controls']) == {'extra_declaration', 'unexpected_target', 'extra_shuffle'}, '/callback controls')
    controls = []
    points = native_result['checkpoints']
    for field in next(iter(points.values()))[0]:
        bad = copy.deepcopy(points)
        del next(iter(bad.values()))[0][field]
        try:
            compare(bad)
        except ValueError as error:
            controls.append(str(error))
        else:
            raise ValueError('undetected missing checkpoint field: ' + field)
    for name, mutate in [
        ('bottom-order', lambda x: x['red-green-0-both'][-2]['library'].reverse()),
        ('missing-boundary', lambda x: x['red-green-0-both'].pop(2)),
        ('reordered-boundaries', lambda x: x['red-green-0-both'].reverse())]:
        bad = copy.deepcopy(points)
        mutate(bad)
        try:
            compare(bad)
        except ValueError as error:
            controls.append(name + ': ' + str(error))
        else:
            raise ValueError('undetected comparator mutation: ' + name)
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
    receipt = dict(schema_version=2, family='mulligan', status='agreed', cases=len(doc['cases']),
        runs_per_engine=2 * len(doc['cases']), completion='first_upkeep_prefix',
        limits='Per-player CR103 boundaries and independent ordered chance/choice streams. Raw cross-stream callback scheduling is retained, not claimed equal. Reference internal RNG/rollback and complete legal-set equivalence are unobservable/unclaimed.',
        pins=doc['pins'], upstream_commit=json.loads((ROOT / 'references/xmage/pins.json').read_text())['upstream_commit'],
        sources=hashes, privileged_directory=folder.name,
        privileged_artifacts={p.name: xmage.sha(p) for p in folder.iterdir() if p.is_file()},
        comparator_controls_detected=len(controls), negative_controls=list(reference['rejections']),
        callback_controls=list(reference['callback_controls']), stdin='closed', display='unset', offline=True)
    output.write_text(json.dumps(receipt, indent=2) + '\n')
    print('32 real London opening prefixes agreed; exact ledgers, repeats and strict negative controls passed.')
