"""Execute the two original shared instant scripts in native Rust and pinned XMage."""
import argparse
import copy
import subprocess
import json
import os
from pathlib import Path
import sys
import xmage

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / 'fixtures/reference/instant-responses.json'
EXPECTATIONS = ROOT / 'fixtures/reference/instant-expectations.json'
BRIDGE = ROOT / 'references/xmage/InstantResponseTest.java'
NATIVE = ROOT / 'crates/mtg-core/src/instant_reference_tests.rs'


def difference(expected, actual, path='$'):
    """Return the first exact semantic difference, including missing/extra fields."""
    if type(expected) is not type(actual):
        return {'path': path, 'expected': expected, 'actual': actual}
    if isinstance(expected, dict):
        if expected.keys() != actual.keys():
            return {'path': path, 'expected_keys': sorted(expected), 'actual_keys': sorted(actual)}
        for key in expected:
            found = difference(expected[key], actual[key], f'{path}.{key}')
            if found:
                return found
    elif isinstance(expected, list):
        for i, (left, right) in enumerate(zip(expected, actual)):
            found = difference(left, right, f'{path}[{i}]')
            if found:
                return found
        if len(expected) != len(actual):
            return {'path': path, 'expected_length': len(expected), 'actual_length': len(actual)}
    elif expected != actual:
        return {'path': path, 'expected': expected, 'actual': actual}
    return None


def compare(document, results):
    # Success expectations belong to the checked-in rules corpus, never the engine
    # receipt or command caller. Both setup AND ordered input are acceptance data.
    canonical = json.loads(FIXTURE.read_text())
    expected = json.loads(EXPECTATIONS.read_text())
    failure = difference(canonical, document, '$.input')
    if not failure:
        wanted = {c['id']: {'checkpoints': expected[c['id']], 'consumed': c['script']}
                  for c in canonical['cases']}
        failure = difference(wanted, results)
    if failure:
        raise ValueError('first divergence: ' + json.dumps(failure, sort_keys=True))


def run_native(output, log, fixture=FIXTURE):
    env = dict(os.environ, MTG_INSTANT_INPUT=str(fixture), MTG_INSTANT_OUTPUT=str(output))
    env.pop('DISPLAY', None)
    env.pop('WAYLAND_DISPLAY', None)
    output.unlink(missing_ok=True)
    xmage.bounded(['cargo', 'test', '-p', 'mtg-core', '--locked', '--lib',
                  'game::instant_reference_tests::instant_export_observations', '--', '--exact'],
                 ROOT, env, log, 180)
    return json.loads(output.read_text())


def run_xmage(cache, output, log, fixture=FIXTURE):
    output.mkdir(exist_ok=True)
    for old in output.glob('*.json'):
        old.unlink()
    command = xmage.maven(cache) + ['-o', '-pl', 'Mage.Tests', '-am', 'test',
        '-Dtest=org.mage.test.mtglab.InstantResponseTest',
        '-Dsurefire.failIfNoSpecifiedTests=false', '-Dmtglab.fixture=' + str(fixture),
        '-Dmtglab.output=' + str(output), '-DargLine=-Djava.awt.headless=true']
    xmage.bounded(command, cache / xmage.SOURCE, xmage.environment(cache), log, 300)
    return {p.stem: json.loads(p.read_text()) for p in sorted(output.glob('*.json'))}


def strict_mutations(cache, output, document):
    """Execute malformed choices in both real adapters; unrelated failures do not pass."""
    definitions = [('omitted-target', 'IllegalTarget', 'illegal target'),
                   ('extra-target', 'unexpected target choice', 'choice order'),
                   ('reordered-payment', 'payment requested before targets', 'choice order'),
                   ('wrong-actor', 'WrongActor', 'wrong actor')]
    receipts = {}
    for name, native_reason, xmage_reason in definitions:
        mutant = copy.deepcopy(document)
        script = mutant['cases'][0]['script']
        if name == 'omitted-target':
            script.pop(4)
        elif name == 'extra-target':
            script.insert(6, copy.deepcopy(script[5]))
        elif name == 'reordered-payment':
            script[4], script[6] = script[6], script[4]
        else:
            script[3]['actor'] = 1
        path = output / (name + '-input.json')
        path.write_text(json.dumps(mutant, indent=2) + '\n')
        receipts[name] = {}
        for engine, reason in [('native', native_reason), ('xmage', xmage_reason)]:
            stem = name + '-' + engine
            log = output / (stem + '.log')
            try:
                if engine == 'native':
                    run_native(output / (stem + '.json'), log, path)
                else:
                    run_xmage(cache, output / stem, log, path)
            except ValueError:
                evidence = log.read_text()
                xmage.scenario.require(reason in evidence, 'unrelated mutant failure: ' + stem)
                receipts[name][engine] = {'status': 'rejected', 'reason': reason,
                                         'input_sha256': xmage.sha(path), 'log_sha256': xmage.sha(log)}
            else:
                raise ValueError('strict choice mutant survived: ' + stem)
    # A legal changed Growth target must execute, then disagree with the independent
    # destination expectation. This proves detection is not just syntax validation.
    mutant = copy.deepcopy(document)
    mutant['cases'][0]['script'][13]['target'] = 'source'
    path = output / 'changed-target-input.json'
    path.write_text(json.dumps(mutant, indent=2) + '\n')
    receipts['changed-target'] = {}
    for engine in ('native', 'xmage'):
        stem = 'changed-target-' + engine
        log = output / (stem + '.log')
        result = (run_native(output / (stem + '.json'), log, path) if engine == 'native'
                  else run_xmage(cache, output / stem, log, path))
        # Compare observed state against the original request, preserving first
        # divergence (including consumed choice mismatch) rather than editing truth.
        try:
            compare(document, result)
        except ValueError as error:
            failure = {'status': 'detected', 'error': str(error), 'log_sha256': xmage.sha(log)}
            (output / (stem + '-divergence.json')).write_text(json.dumps(failure, indent=2) + '\n')
            receipts['changed-target'][engine] = failure
        else:
            raise ValueError('changed target mutant survived: ' + engine)
    return receipts


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cache', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True,
                        help='Receipt directory; retains observations, scripts, logs and first divergence')
    args = parser.parse_args()
    cache, output = args.cache.resolve(), args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    receipt = output / 'acceptance.json'
    receipt.unlink(missing_ok=True)
    failure_path = output / 'first-divergence.json'
    failure_path.unlink(missing_ok=True)
    try:
        xmage.scenario.require(not cache.is_relative_to(ROOT), 'cache must be external')
        xmage.verify_inputs(cache)
        xmage.scenario.require(xmage.dependencies(cache) == xmage.scenario.load(ROOT / 'references/xmage/dependencies.json'),
                               'dependency lock mismatch')
        target = cache / xmage.SOURCE / 'Mage.Tests/src/test/java/org/mage/test/mtglab/InstantResponseTest.java'
        target.write_bytes(BRIDGE.read_bytes())
        document = json.loads(FIXTURE.read_text())
        (output / 'input.json').write_text(json.dumps(document, indent=2) + '\n')
        runs = {}
        for repeat in (1, 2):
            for engine in ('native', 'xmage'):
                name = f'{engine}-{repeat}'
                if engine == 'native':
                    results = run_native(output / (name + '.json'), output / (name + '.log'))
                else:
                    results = run_xmage(cache, output / name, output / (name + '.log'))
                    (output / (name + '.json')).write_text(json.dumps(results, indent=2) + '\n')
                compare(document, results)
                runs[name] = results
        for name, result in runs.items():
            xmage.scenario.require(result == runs['native-1'], 'repeat/cross-engine divergence: ' + name)
        mutations = strict_mutations(cache, output, document)
        files = [FIXTURE, EXPECTATIONS, BRIDGE, NATIVE, Path(__file__), ROOT / 'references/xmage/pins.json',
                 ROOT / 'references/xmage/dependencies.json', ROOT / 'Cargo.lock', ROOT / 'data/rules/cr-2026-09-25.json',
                 ROOT / 'data/cards/foundations_micro_v1.json']
        report = {'status': 'agreed', 'schema_version': 1, 'cases': list(runs['native-1']),
                  'executions_per_engine': 2, 'mutations': mutations, 'input_boundary': 'turn-1-upkeep-priority',
                  'pins': {str(p.relative_to(ROOT)): xmage.sha(p) for p in files},
                  'upstream_commit': xmage.scenario.load(ROOT / 'references/xmage/pins.json')['upstream_commit'],
                  'artifacts': {p.name: xmage.sha(p) for p in sorted(output.iterdir()) if p.is_file()},
                  'native_sources': {str(p.relative_to(ROOT)): xmage.sha(p) for p in sorted((ROOT / 'crates/mtg-core/src').glob('*.rs'))},
                  'rustc': subprocess.check_output(['rustc', '--version'], text=True, timeout=10).strip(),
                  'python': sys.version.split()[0], 'native_profile': 'debug',
                  'stdin': 'closed', 'display': 'unset', 'xmage_offline': True,
                  'observability': 'Settled priority, bottom-to-top stack and targets, named zones, creature stats/damage, life, mana, taps; damage event observed before lethal SBA.',
                  'limitations': 'Synthetic setup; no legal-action enumeration, hidden views, departed targets, combat, cleanup, full games or Forge agreement.'}
        receipt.write_text(json.dumps(report, indent=2) + '\n')
        print('Both instant-response cases agreed with independent expectations in both engines, twice.')
    except (ValueError, OSError, KeyError) as error:
        failure_path.write_text(json.dumps({'status': 'failed', 'error': str(error),
                                          'input': str(FIXTURE), 'artifacts': str(output)}, indent=2) + '\n')
        print(str(error), file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
