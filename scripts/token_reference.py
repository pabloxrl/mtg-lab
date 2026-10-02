"""Execute original token scenarios against the existing pinned XMage cache."""
import argparse
import json
import os
from pathlib import Path
import xmage

ROOT = Path(__file__).resolve().parents[1]


def compare(results):
    expected = json.loads((ROOT/'fixtures/reference/token-expectations.json').read_text())
    xmage.scenario.require(results == expected, 'token checkpoint mismatch')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cache', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    cache = args.cache.resolve()
    xmage.verify_inputs(cache)
    xmage.scenario.require(xmage.dependencies(cache) == xmage.scenario.load(ROOT/'references/xmage/dependencies.json'), 'dependency lock mismatch')
    bridge = ROOT/'references/xmage/TokenLifecycleTest.java'
    target = cache/xmage.SOURCE/'Mage.Tests/src/test/java/org/mage/test/mtglab/TokenLifecycleTest.java'
    target.write_bytes(bridge.read_bytes())
    args.output.mkdir(parents=True, exist_ok=True)
    for p in args.output.glob('*.json'):
        p.unlink()
    cmd = xmage.maven(cache) + ['-o', '-pl', 'Mage.Tests', '-am', 'test',
        '-Dtest=org.mage.test.mtglab.TokenLifecycleTest', '-Dsurefire.failIfNoSpecifiedTests=false',
        '-Dmtglab.output='+str(args.output.resolve()), '-DargLine=-Djava.awt.headless=true']
    xmage.bounded(cmd, cache/xmage.SOURCE, xmage.environment(cache), args.output/'xmage.log', 300)
    results = {p.stem: json.loads(p.read_text()) for p in args.output.glob('*.json')}
    env=dict(os.environ, MTG_TOKEN_OUTPUT=str((args.output/'native.json').resolve()))
    xmage.bounded(['cargo', 'test', '-p', 'mtg-core', '--locked', '--lib',
        'game::token_tests::tokens_reference_observations_match_literal_oracle', '--', '--exact'],
        ROOT, env, args.output/'native.log', 180)
    native=json.loads((args.output/'native.json').read_text())
    compare(native)
    compare(results)
    report = dict(status='agreed', upstream_commit=xmage.scenario.load(ROOT/'references/xmage/pins.json')['upstream_commit'],
        bridge_sha256=xmage.sha(bridge), runner_sha256=xmage.sha(Path(__file__)), native_log_sha256=xmage.sha(args.output/'native.log'), native_sources={str(p.relative_to(ROOT)): xmage.sha(p) for p in sorted((ROOT/'crates/mtg-core/src').glob('*.rs'))}, native_sha256=xmage.sha(ROOT/'crates/mtg-core/src/token_tests.rs'), expectations_sha256=xmage.sha(ROOT/'fixtures/reference/token-expectations.json'), log_sha256=xmage.sha(args.output/'xmage.log'), checkpoints=results,
        stdin='closed', display='unset', offline=True,
        limitations='Synthetic XMage setup; no full-game or Forge agreement. Matched native checkpoints; timing/overflow rejection tests are native-only.')
    (args.output/'receipt.json').write_text(json.dumps(report, indent=2)+'\n')
    print('Five pinned XMage token scenarios agreed.')


if __name__ == '__main__':
    main()
