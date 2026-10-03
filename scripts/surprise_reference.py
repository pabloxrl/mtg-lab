"""Execute the shared surprise cases in native Rust and pinned XMage."""
import argparse
import copy
import json
import os
from pathlib import Path
import instant_reference
import xmage

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / 'fixtures/reference/surprise-expectations.json'
EXPECTED = ROOT / 'fixtures/reference/surprise-expectations.json'
BRIDGE = ROOT / 'references/xmage/SurpriseTest.java'


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
    (output / "receipt.json").unlink(missing_ok=True)
    target = cache / xmage.SOURCE / 'Mage.Tests/src/test/java/org/mage/test/mtglab/SurpriseTest.java'
    target.write_bytes(BRIDGE.read_bytes())
    runs = []
    for repeat in range(2):
        folder = output / str(repeat)
        folder.mkdir(exist_ok=True)
        for old in folder.glob('*.json'):
            old.unlink()
        cmd = xmage.maven(cache) + ['-o', '-pl', 'Mage.Tests', '-am', 'test',
            '-Dtest=org.mage.test.mtglab.SurpriseTest', '-Dsurefire.failIfNoSpecifiedTests=false',
            '-Dmtglab.fixture=' + str(FIXTURE), '-Dmtglab.output=' + str(folder),
            '-DargLine=-Djava.awt.headless=true']
        xmage.bounded(cmd, cache / xmage.SOURCE, xmage.environment(cache), folder / 'xmage.log', 300)
        actual = {p.stem: json.loads(p.read_text()) for p in folder.glob('*.json')}
        compare(actual)
        native = folder / 'native.json'
        env = dict(os.environ, MTG_SURPRISE_OUTPUT=str(native))
        xmage.bounded(['cargo', 'test', '-p', 'mtg-core', '--locked', '--lib',
            'game::casting::tests::surprise_reference_literal_checkpoints', '--', '--exact'],
            ROOT, env, folder / 'native.log', 180)
        compare(json.loads(native.read_text()))
        runs.append({'xmage_log_sha256': xmage.sha(folder / 'xmage.log'), 'native_log_sha256': xmage.sha(folder / 'native.log')})
    controls = []
    canonical = json.loads(EXPECTED.read_text())
    for field in ['mode', 'resolved', 'cleanup']:
        wrong = copy.deepcopy(canonical)
        wrong['boost'][field] = 99
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
        limitations='Synthetic setup with real pinned modal casts, continuous effects, token creation, combat and cleanup. Exact mode and creature/life ledgers at resolution, later creation, combat and cleanup. Native tests cover stale/private continuation rejection and real normal-reset Driver play. No Forge/full-game agreement.')
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print('7 Surprise cases agreed twice in native Rust and pinned XMage.')


if __name__ == '__main__':
    main()
