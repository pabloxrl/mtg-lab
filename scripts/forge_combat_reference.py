"""Bounded L0-010 all-three acceptance for a source-derived full-game fixture.

Forge execution and its immutable source validation precede this check. This
runner adds independent native/patched-XMage execution, not corpus admission.
"""
import argparse
import copy
import json
import os
from pathlib import Path
import time
import xml.etree.ElementTree as ET

import admission_reference as admission
import admission_observations
import forge_opening as opening
from instant_reference import difference
import xmage

ROOT = Path(__file__).resolve().parents[1]


def transcript(fixture):
    opening.need(fixture['schema_version'] == 3 and fixture['complete_game'], 0, 'full-game fixture')
    base = fixture['initial_incarnations']

    def absolute(value):
        if isinstance(value, list):
            return [absolute(v) for v in value]
        if not isinstance(value, dict):
            return value
        result = {k: absolute(v) for k,v in value.items()}
        if isinstance(result.get('source'), str) and 'incarnation' in result:
            source = result['source']
            opening.need(source in base or source.startswith('token/'), 0, 'unbound absolute identity')
            result['incarnation'] += base.get(source, 0)
        return result

    case = copy.deepcopy(fixture['opening']['cases'][0])
    case.update(id='forge-actual-combat', stop='terminal',
                play=[absolute(op['action']) for op in fixture['operations'] if 'action' in op])
    return dict(schema_version=10, family='admission', pins=fixture['opening']['pins'],
                cases=[case], provenance=dict(kind='actual-forge-ai', source_sha256=fixture['source_sha256']))


def compare(fixture, native, reference):
    case = transcript(fixture)['cases'][0]
    projected, indices = admission.reference_case(case)
    opening.need(native['consumed_play'] == case['play'], 0, 'native exact actions')
    opening.need(reference['consumed_play'] == projected['play'], 0, 'XMage exact actions')
    for engine, run, inputs in [('native',native,case), ('xmage',reference,projected)]:
        for kind in ('chance','choices'):
            opening.need(run['opening']['consumed_'+kind] == case[kind], 0, engine+' exact '+kind)
        opening.need(len(run['points']) == len(inputs['play']), 0, engine+' checkpoint count')
        admission_observations.check(run, inputs, engine)
    admission_observations.xmage_effect_lifecycle(reference, projected)
    admission_observations.xmage_trigger_provenance(reference, projected)
    aligned = dict(zip(indices, reference['points']))
    action = 0
    checkpoints = 0
    for op in fixture['operations']:
        if 'action' in op:
            action += 1
            continue
        index = min(action, len(case['play'])-1)
        left, right = native['points'][index], aligned[index]
        if index == len(case['play'])-1:
            right = dict(reference['before_departure'], outcome=reference['points'][-1]['outcome'], actor=None)
        delta = difference(admission.semantic(left), admission.semantic(right))
        opening.need(delta is None, op['source_cursor'], 'XMage checkpoint: '+str(delta))
        checkpoints += 1
    for kind in ('marked','cleanup_checkpoints'):
        opening.need(len(native[kind]) == len(reference[kind]), 0, kind+' count')
        for i,(left,right) in enumerate(zip(native[kind], reference[kind])):
            delta = difference(admission.semantic(left), admission.semantic(right))
            opening.need(delta is None, i, kind+': '+str(delta))
    return dict(actions=len(case['play']), checkpoints=checkpoints,
                marked=len(native['marked']), cleanups=len(native['cleanup_checkpoints']))


def run(args):
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    started = time.monotonic()
    receipt = dict(status='started', admitted_games=0, fixture_sha256=xmage.sha(args.fixture))
    paths = ['scripts/forge_combat_reference.py', 'scripts/forge_casting_native.py',
             'scripts/admission_reference.py', 'scripts/admission_observations.py',
             'scripts/forge_opening.py', 'scripts/instant_reference.py', 'scripts/xmage.py',
             'references/xmage/FullPoolAdmissionTest.java', 'references/xmage/CombatSbaGame.java',
             'references/xmage/FullPoolMulliganTest.java', 'references/xmage/FullPoolPriorityTest.java',
             'references/xmage/FullPoolTriggersTest.java', 'references/xmage/pins.json',
             'references/xmage/dependencies.json', 'Cargo.lock']
    receipt['implementation'] = {p:xmage.sha(ROOT/p) for p in paths}
    receipt['native_sources'] = {str(p.relative_to(ROOT)):xmage.sha(p)
                                for p in sorted((ROOT/'crates/mtg-core/src').glob('*.rs'))}
    try:
        fixtures = json.loads(args.fixture.read_text())
        opening.need(len(fixtures) == 1, 0, 'one bounded game')
        fixture = fixtures[0]
        doc = transcript(fixture)
        projected = copy.deepcopy(doc)
        projected['cases'] = [admission.reference_case(doc['cases'][0])[0]]
        (output/'xmage-input.json').write_text(json.dumps(projected))
        env = dict(os.environ, MTG_FORGE_COMBAT_INPUT=str(args.fixture.resolve()),
                   MTG_FORGE_COMBAT_OUTPUT=str(output/'native.json'))
        xmage.bounded(['cargo','test','-p','mtg-core','--locked','--lib','captured_forge_combat_strict_replay'],
                      ROOT, env, output/'native.log', 300)
        native = json.loads((output/'native.json').read_text())[0]['gameplay']['admission']
        cache = args.cache.resolve()
        xmage.verify_inputs(cache)
        opening.need(xmage.dependencies(cache) == json.loads((ROOT/'references/xmage/dependencies.json').read_text()), 0, 'XMage dependencies')
        for family in ('mulligan','priority','spells','triggers','activations','played-combat','admission'):
            for name, sha in json.loads((ROOT/f'references/xmage/{family}-provenance.json').read_text())['consulted_sources'].items():
                opening.need(xmage.sha(cache/xmage.SOURCE/name) == sha, 0, 'XMage pinned source '+name)
        for name in ('FullPoolMulliganTest.java','FullPoolPriorityTest.java','FullPoolTriggersTest.java','FullPoolAdmissionTest.java','CombatSbaGame.java'):
            target = cache/xmage.SOURCE/'Mage.Tests/src/test/java/org/mage/test/mtglab'/name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes((ROOT/'references/xmage'/name).read_bytes())
        command = xmage.maven(cache)+['-o','-pl','Mage.Tests','-am','test',
            '-Dtest=org.mage.test.mtglab.FullPoolAdmissionTest','-Dsurefire.failIfNoSpecifiedTests=false',
            '-Dmtglab.fixture='+str(output/'xmage-input.json'), '-Dmtglab.root='+str(ROOT),
            '-Dmtglab.output='+str(output/'xmage-index.json'), '-DargLine=-Djava.awt.headless=true']
        receipt['command'] = command
        report = cache/xmage.SOURCE/'Mage.Tests/target/surefire-reports/TEST-org.mage.test.mtglab.FullPoolAdmissionTest.xml'
        report.unlink(missing_ok=True)
        xmage.bounded(command, cache/xmage.SOURCE, xmage.environment(cache), output/'xmage.log', 600)
        (output/'test-report.xml').write_bytes(report.read_bytes())
        suite = ET.parse(report).getroot()
        opening.need(suite.get('tests') == '1' and all(suite.get(k) == '0' for k in ('failures','errors','skipped'))
                     and [c.get('name') for c in suite.findall('testcase')] == ['mulliganPrefixes'],
                     0, 'actual unskipped XMage test')
        index = json.loads((output/'xmage-index.json').read_text())
        receipt['runs'] = [compare(fixture,native,json.loads((output/name).read_text()))
                           for name in index[doc['cases'][0]['id']]]
        opening.need(len(receipt['runs']) == 2, 0, 'two reference replays')
        receipt['status'] = 'native-xmage-agreed'
    except (ValueError, OSError, KeyError, TypeError) as error:
        receipt.update(status='failed', error=str(error))
    receipt['elapsed_seconds'] = time.monotonic()-started
    receipt['artifacts'] = {p.name:xmage.sha(p) for p in output.iterdir() if p.is_file()}
    (output/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
    print(json.dumps(dict(status=receipt['status'], error=receipt.get('error'), output=str(output))))
    return 0 if receipt['status'] == 'native-xmage-agreed' else 2


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--fixture', type=Path, required=True)
    parser.add_argument('--cache', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    raise SystemExit(run(parser.parse_args()))
