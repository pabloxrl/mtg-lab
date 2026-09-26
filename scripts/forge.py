"""Pinned external Forge harness. Reuses the neutral XMage smoke contract, not its engine."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import tarfile
import time
import xml.etree.ElementTree as ET

import scenario
import xmage
from xmage import bounded, sha, maven, dependencies

ROOT = Path(__file__).resolve().parents[1]
REFERENCE = ROOT / 'references/forge'
FIXTURE = xmage.FIXTURE  # Exactly the bytes executed by GH-13, including independent expectations.
SOURCE = 'forge-95dc682bf92460f49cebd7a9578f06ccf60d5569'
TEST_PATH = 'forge-gui-desktop/src/test/java/forge/gamesimulationtests/mtglab/NeutralSmokeTest.java'
REPORT_PATH = 'forge-gui-desktop/target/surefire-reports/TEST-forge.gamesimulationtests.mtglab.NeutralSmokeTest.xml'


def scope(fixture):
    xmage.scope(fixture)
    keys = [(o['owner'], o['card_id']) for o in fixture['setup']['state']['objects']]
    scenario.require(len(keys) == len(set(keys)), 'ambiguous card identity unsupported')


def compare(fixture, output):
    xmage.compare(fixture, output)
    identities = {o['id']: o['card_id'] for o in fixture['setup']['state']['objects']}
    scenario.require(scenario.json_equal(output.get('observed_cards'), identities),
                     'missing or mismatched card identity evidence')


def environment(cache):
    env = xmage.environment(cache)
    # Use an isolated Java home directory for Forge preferences/custom data. Never
    # reads the agent's settings or installs GUI software. Inherited by test JVMs.
    env['JAVA_TOOL_OPTIONS'] = '-Djava.awt.headless=true -Duser.home=' + str(cache/'home')
    return env


def verify_inputs(cache):
    pins = scenario.load(REFERENCE/'pins.json')
    scenario.require(platform.system() == 'Linux' and platform.machine() == 'aarch64',
                     'this smoke requires the pinned Linux ARM64 toolchain')
    java = subprocess.run([str(Path(os.environ['JAVA_HOME'])/'bin/java'), '-version'],
                          capture_output=True, text=True, check=True, timeout=15,
                          stdin=subprocess.DEVNULL, env=environment(cache))
    mvn = subprocess.run(['mvn', '-version'], capture_output=True, text=True, check=True,
                         timeout=15, stdin=subprocess.DEVNULL, env=environment(cache))
    scenario.require('Temurin-'+pins['jdk_runtime']+' ' in java.stderr and
                     'Apache Maven '+pins['maven']+' ' in mvn.stdout and
                     'Java version: 21.0.9, vendor: Eclipse Adoptium' in mvn.stdout,
                     'installed toolchain differs from pins')
    archive = cache/'forge.tar.gz'
    scenario.require(sha(archive) == pins['archive']['sha256'], 'source archive pin mismatch')
    with tarfile.open(archive) as stream:
        for member in stream:
            if member.isfile():
                scenario.require(hashlib.sha256(stream.extractfile(member).read()).hexdigest() ==
                                 sha(cache/member.name), 'pinned source changed: '+member.name)
    # Custom cards/preferences would change the reference. Only freshly created
    # empty directories are permitted as inputs to this synthetic test harness.
    home = cache/'home'
    scenario.require(not any(p.is_file() for p in home.rglob('*')),
                     'unexpected Forge user data; use an empty isolated home')


def command(cache, fixture, output, offline):
    return maven(cache) + (['-o'] if offline else []) + [
        '-pl', 'forge-gui-desktop', '-am', 'test',
        '-Dtest=forge.gamesimulationtests.mtglab.NeutralSmokeTest',
        '-Dsurefire.failIfNoSpecifiedTests=false',
        '-Dmtglab.fixture='+str(fixture.resolve()), '-Dmtglab.output='+str(output),
        '-Dmtglab.assets='+str(cache/SOURCE/'forge-gui')+'/']


def require_executed(report):
    root = ET.parse(report).getroot()
    scenario.require(root.attrib.get('tests') == '1' and
                     all(root.attrib.get(k) == '0' for k in ('failures', 'errors', 'skipped')),
                     'Forge smoke was not exactly one passing, unskipped test')
    cases = root.findall('testcase')
    scenario.require(len(cases) == 1 and cases[0].attrib.get('name') == 'neutralSmoke',
                     'Forge smoke test missing')


def execute(cache, fixture_path, name, offline=True):
    fixture = scenario.load(fixture_path)
    scope(fixture)
    out = cache/(name+'.json')
    report = cache/SOURCE/REPORT_PATH
    out.unlink(missing_ok=True)
    report.unlink(missing_ok=True)
    cmd = command(cache, fixture_path, out, offline)
    bounded(cmd, cache/SOURCE, environment(cache), cache/(name+'.log'), 180 if offline else 1200)
    require_executed(report)
    scenario.require(out.is_file(), 'Forge produced no checkpoints')
    result = scenario.load(out)
    compare(fixture, result)
    return result


def mutations(cache, fixture_path):
    results = {}
    for name in ('life', 'choice', 'checkpoint'):
        fixture = scenario.load(fixture_path)
        if name == 'life':
            fixture['setup']['state']['players'][0]['life'] = 19
            expected = 'priority-p1 mismatch: /players/0/life'
        elif name == 'choice':
            fixture['script'][0]['actor'] = 1
            fixture['script'][0]['choices'][0]['actor'] = 1
            expected = 'script actor mismatch'
        else:
            fixture['checkpoints'][0]['assertions'][0]['expected'] = 0
            expected = 'priority-p1 mismatch: /priority'
        fixture['provenance']['reproduction']['trace_sha256'] = scenario.digest(fixture['script'])
        del fixture['provenance']['fixture_revision']
        fixture['provenance']['fixture_revision'] = scenario.digest(fixture)
        path = cache/('mutant-'+name+'-fixture.json')
        path.write_text(json.dumps(fixture, indent=2)+'\n')
        try:
            execute(cache, path, 'mutant-'+name)
        except ValueError as error:
            evidence = str(error)
            if name == 'choice':
                scenario.require('command exited' in evidence, 'choice did not reach Forge')
                evidence = (cache/('mutant-'+name+'.log')).read_text()
            scenario.require(expected in evidence, f'{name} failed for unintended reason: {error}')
            results[name] = dict(status='detected', reason=expected,
                                 fixture_sha256=sha(path), log_sha256=sha(cache/('mutant-'+name+'.log')))
        else:
            raise ValueError('undetected '+name+' mutation')
    return results


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['prepare', 'build', 'run', 'acceptance'])
    parser.add_argument('--cache', type=Path, required=True)
    parser.add_argument('--fixture', type=Path, default=FIXTURE)
    args = parser.parse_args()
    cache = args.cache.resolve()
    try:
        scenario.require(not cache.is_relative_to(ROOT), 'cache must be outside source tree')
        scenario.require(' ' not in str(cache), 'cache path cannot contain spaces (JVM options)')
        pins = scenario.load(REFERENCE/'pins.json')
        if args.command == 'prepare':
            cache.mkdir(parents=True, exist_ok=True)
            archive = cache/'forge.tar.gz'
            if not archive.exists():
                bounded(['curl', '-fL', '--max-time', '180', pins['archive']['url'], '-o', str(archive)],
                        cache, os.environ, cache/'download.log', 190)
            scenario.require(sha(archive) == pins['archive']['sha256'], 'source archive pin mismatch')
            bounded(['tar', '-xzf', str(archive), '-C', str(cache)],
                    cache, os.environ, cache/'extract.log', 300)
            print(json.dumps(dict(status='prepared', cache=str(cache))))
            return 0
        (cache/'acceptance.json').unlink(missing_ok=True)
        scenario.require((cache/SOURCE).is_dir(), 'missing externally installed pinned Forge source')
        verify_inputs(cache)
        target = cache/SOURCE/TEST_PATH
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes((REFERENCE/'NeutralSmokeTest.java').read_bytes())
        if args.command == 'build':
            execute(cache, args.fixture, 'build-smoke', offline=False)
            scenario.require(dependencies(cache) == scenario.load(REFERENCE/'dependencies.json'),
                             'build dependency lock mismatch')
            print(json.dumps(dict(status='built', log=str(cache/'build-smoke.log'))))
            return 0
        scenario.require(dependencies(cache) == scenario.load(REFERENCE/'dependencies.json'),
                         'build dependency lock mismatch')
        started = time.monotonic()
        a = execute(cache, args.fixture, 'smoke-1')
        b = execute(cache, args.fixture, 'smoke-2')
        scenario.require(scenario.json_equal(a, b), 'nonreproducible checkpoints')
        # Compare actual normalized states with GH-13's committed, executed evidence.
        # Expectations still come from the neutral fixture and CR 117.3d above.
        xmage_points = scenario.load(ROOT/'references/xmage/checkpoints.json')['checkpoints']
        scenario.require(scenario.json_equal(a['checkpoints'], xmage_points), 'XMage/Forge checkpoint disagreement')
        report = dict(status='agreed', smoke_seconds=time.monotonic()-started,
                      checkpoint_sha256=sha(cache/'smoke-1.json'), fixture_sha256=sha(args.fixture),
                      bridge_sha256=sha(REFERENCE/'NeutralSmokeTest.java'), runner_sha256=sha(Path(__file__)),
                      shared_runner_sha256=sha(ROOT/'scripts/xmage.py'),
                      pins_sha256=sha(REFERENCE/'pins.json'), dependencies_sha256=sha(REFERENCE/'dependencies.json'),
                      xmage_checkpoint_sha256=sha(ROOT/'references/xmage/checkpoints.json'),
                      platform=platform.system()+'/'+platform.machine(), stdin='closed', display='unset',
                      offline=True, smoke_timeout_seconds=180,
                      command=command(cache, args.fixture, cache/'smoke-1.json', True),
                      logs={n: sha(cache/(n+'.log')) for n in ('smoke-1', 'smoke-2')})
        if args.command == 'acceptance':
            report['mutations'] = mutations(cache, args.fixture)
        (cache/'acceptance.json').write_text(json.dumps(report, indent=2)+'\n')
        print(json.dumps(report))
    except (ValueError, OSError, KeyError, ET.ParseError, subprocess.SubprocessError) as error:
        print(json.dumps(dict(status='failed', error=str(error))))
        return 2
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
