"""Test-only pinned XMage smoke bridge. Never imports the production engine."""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import time
import tarfile
import platform

import scenario

ROOT = Path(__file__).resolve().parents[1]
SOURCE = 'mage-000d8a7abc0ac31cc24af08691423e0c24dc59e7'
FIXTURE = ROOT / 'fixtures/scenarios/xmage-priority-pass.json'


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def bounded(command, cwd, env, log, timeout):
    """Close stdin, remove displays, kill the entire process group on timeout."""
    env = dict(env)
    for key in ('DISPLAY', 'WAYLAND_DISPLAY'):
        env.pop(key, None)
    with Path(log).open('w') as out:
        process = subprocess.Popen(command, cwd=cwd, env=env, stdin=subprocess.DEVNULL,
                                   stdout=out, stderr=subprocess.STDOUT, start_new_session=True)
        try:
            status = process.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait()
            raise ValueError(f'timeout after {timeout}s; log: {log}') from None
    if status:
        raise ValueError(f'command exited {status}; log: {log}')


def scope(fixture):
    scenario.validate(fixture)
    s = fixture['setup']
    scenario.require(s['kind'] == 'synthetic', 'smoke supports synthetic setup only')
    s = s['state']
    scenario.require((s['turn'], s['active_player'], s['priority'], s['phase'], s['step']) ==
                     (1, 0, 0, 'beginning', 'upkeep'), 'unsupported initial timing')
    scenario.require(not s['stack'] and not s['effects'], 'unsupported initial stack/effects')
    scenario.require(not fixture['invalid_actions'], 'invalid-action execution unsupported')
    for p in s['players']:
        scenario.require(0 < p['life'] < 1000 and p['land_plays_used'] == 0,
                         'unsupported life/land configuration')
        scenario.require(all(v == 0 for v in p['mana'].values()), 'initial mana unsupported')
        scenario.require(all(not v for k, v in p['zones'].items() if k != 'library'),
                         'only library objects supported')
    for o in s['objects']:
        scenario.require(o['card_id'] in ('forest', 'mountain') and o['generation'] == 0
                         and o['controller'] == o['owner'] and o['status'] ==
                         dict(tapped=False, controlled_since_turn=0, damage=0, power=None,
                              toughness=None, keywords=[], token=False), 'unsupported object')
    scenario.require(len(fixture['script']) == 1, 'exactly one explicit pass required')
    a = fixture['script'][0]
    scenario.require(a['kind'] == 'pass' and a['source'] is None, 'only pass supported')
    scenario.require(len(fixture['checkpoints']) == 1 and
                     fixture['checkpoints'][0]['name'] == 'priority-p1' and
                     fixture['checkpoints'][0]['after'] == a['id'], 'unsupported checkpoint')


def compare(fixture, output):
    scenario.require(output['bridge_version'] == 1, 'bridge version mismatch')
    unsupported = output.get('unsupported', {})
    required = {'object_characteristics_status_damage', 'legal_choices', 'outcome',
                'effects_and_private_views', 'land_plays_used'}
    scenario.require(set(unsupported) == required and
                     all(isinstance(v, str) and v.strip() for v in unsupported.values()),
                     'missing observability declarations')
    points = output['checkpoints']
    scenario.require([p['name'] for p in points] == ['initial', 'priority-p1'], 'missing/reordered checkpoints')
    # Initial state must be independently read from XMage, not merely echoed.
    initial = copy.deepcopy(fixture['setup']['state'])
    for field in ('turn', 'active_player', 'priority', 'phase', 'step', 'stack'):
        scenario.require(scenario.json_equal(initial[field], points[0]['state'][field]),
                         f'initial mismatch: /{field}')
    for i in range(2):
        for field in ('seat', 'life', 'mana', 'zones'):
            scenario.require(scenario.json_equal(initial['players'][i][field], points[0]['state']['players'][i][field]),
                             f'initial mismatch: /players/{i}/{field}')
    # CR 117.3d: one pass only transfers priority; every exported other field persists.
    expected_after = copy.deepcopy(points[0]['state'])
    expected_after['priority'] = 1
    scenario.require(scenario.json_equal(expected_after, points[1]['state']),
                     'priority-p1 mismatch: pass must preserve all other observed fields')
    for assertion in fixture['checkpoints'][0]['assertions']:
        actual = scenario.pointer(points[1]['state'], assertion['path'])
        scenario.require(scenario.json_equal(actual, assertion['expected']),
                         f'priority-p1 mismatch: {assertion["path"]}')


def environment(cache):
    env = dict(os.environ, JAVA_HOME=os.environ['JAVA_HOME'],
               MAVEN_OPTS='-Xmx6g -Djava.awt.headless=true')
    for key in ('DISPLAY', 'WAYLAND_DISPLAY'):
        env.pop(key, None)
    return env


def maven(cache):
    settings = cache/'settings.xml'
    settings.write_text('<settings xmlns="http://maven.apache.org/SETTINGS/1.0.0"/>\n')
    return ['mvn', '-B', '-ntp', '-s', str(settings),
            '-gs', str(settings), '-Dmaven.repo.local='+str(cache/'m2')]


def execute(cache, fixture_path, name):
    fixture = scenario.load(fixture_path)
    scope(fixture)
    out = cache/(name+'.json')
    out.unlink(missing_ok=True)
    cmd = maven(cache) + ['-o', '-pl', 'Mage.Tests', '-am', 'test',
                         '-Dtest=org.mage.test.mtglab.NeutralSmokeTest',
                         '-Dsurefire.failIfNoSpecifiedTests=false',
                         '-Dmtglab.fixture='+str(fixture_path.resolve()),
                         '-Dmtglab.output='+str(out), '-DargLine=-Djava.awt.headless=true']
    bounded(cmd, cache/SOURCE, environment(cache), cache/(name+'.log'), 180)
    scenario.require(out.is_file(), 'XMage produced no checkpoints')
    result = scenario.load(out)
    compare(fixture, result)
    return result


def verify_inputs(cache):
    pins = scenario.load(ROOT/'references/xmage/pins.json')
    scenario.require(platform.system() == 'Linux' and platform.machine() == 'aarch64',
                     'this smoke requires the pinned Linux ARM64 toolchain')
    java = subprocess.run([str(Path(os.environ['JAVA_HOME'])/'bin/java'), '-version'],
                          capture_output=True, text=True, timeout=15, check=True,
                          stdin=subprocess.DEVNULL, env=environment(cache))
    mvn = subprocess.run(['mvn', '-version'], capture_output=True, text=True,
                         timeout=15, check=True, stdin=subprocess.DEVNULL, env=environment(cache))
    scenario.require('Temurin-'+pins['jdk_runtime']+' ' in java.stderr and
                     'Apache Maven '+pins['maven']+' ' in mvn.stdout and
                     'Java version: 21.0.9, vendor: Eclipse Adoptium' in mvn.stdout,
                     'installed toolchain differs from pins')
    for name, pin in pins['archives'].items():
        scenario.require(sha(cache/name) == pin['sha256'], f'archive pin mismatch: {name}')
    # Verify original upstream source bytes, including POMs and card definitions.
    # Generated target/db files and our single new test are outside the archive.
    for name in pins['archives']:
        with tarfile.open(cache/name) as archive:
            for member in archive:
                if member.isfile():
                    scenario.require(hashlib.sha256(archive.extractfile(member).read()).hexdigest() ==
                                     sha(cache/member.name), f'pinned source/tool changed: {member.name}')


def dependencies(cache):
    return {str(p.relative_to(cache/'m2')): sha(p) for p in sorted((cache/'m2').rglob('*'))
            if p.is_file() and p.suffix in ('.jar', '.pom', '.exe')}


def mutations(cache, fixture_path):
    """Exercise real XMage, requiring the intended mismatch, not any failure."""
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
                scenario.require('command exited' in evidence, 'choice did not reach XMage')
                evidence = (cache/('mutant-'+name+'.log')).read_text()
            scenario.require(expected in evidence, f'{name} failed for unintended reason: {error}')
            results[name] = {'status': 'detected', 'reason': expected,
                             'log_sha256': sha(cache/('mutant-'+name+'.log'))}
        else:
            raise ValueError(f'undetected {name} mutation')
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
        if args.command == 'prepare':
            cache.mkdir(parents=True, exist_ok=True)
            pins = scenario.load(ROOT/'references/xmage/pins.json')
            for name, pin in pins['archives'].items():
                archive = cache/name
                if not archive.exists():
                    # curl's total deadline bounds slow transfers as well as idle sockets.
                    bounded(['curl', '-fL', '--max-time', '180', pin['url'], '-o', str(archive)],
                            cache, os.environ, cache/(name+'.download.log'), 190)
                scenario.require(sha(archive) == pin['sha256'], f'archive pin mismatch: {name}')
                bounded(['tar', '-xzf', str(archive), '-C', str(cache)], cache, os.environ,
                        cache/(name+'.extract.log'), 300)
            print(json.dumps({'status':'prepared', 'cache':str(cache)}))
            return 0
        if args.command in ('run', 'acceptance'):
            (cache/'acceptance.json').unlink(missing_ok=True)
        source = cache/SOURCE
        scenario.require(source.is_dir(), 'missing externally extracted pinned XMage source')
        verify_inputs(cache)
        target = source/'Mage.Tests/src/test/java/org/mage/test/mtglab/NeutralSmokeTest.java'
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes((ROOT/'references/xmage/NeutralSmokeTest.java').read_bytes())
        scope(scenario.load(args.fixture))
        if args.command == 'build':
            (cache/'build-smoke.json').unlink(missing_ok=True)
            bounded(maven(cache)+['-pl', 'Mage.Tests', '-am', 'test',
                    '-Dtest=org.mage.test.mtglab.NeutralSmokeTest',
                    '-Dsurefire.failIfNoSpecifiedTests=false',
                    '-Dmtglab.fixture='+str(args.fixture.resolve()),
                    '-Dmtglab.output='+str(cache/'build-smoke.json'),
                    '-DargLine=-Djava.awt.headless=true'],
                    source, environment(cache), cache/'build.log', 1200)
            compare(scenario.load(args.fixture), scenario.load(cache/'build-smoke.json'))
            scenario.require(dependencies(cache) == scenario.load(ROOT/'references/xmage/dependencies.json'),
                             'build dependency lock mismatch')
            print(json.dumps({'status':'built', 'log':str(cache/'build.log')}))
        else:
            lock = scenario.load(ROOT/'references/xmage/dependencies.json')
            scenario.require(dependencies(cache) == lock, 'build dependency lock mismatch')
            started = time.monotonic()
            a = execute(cache, args.fixture, 'smoke-1')
            b = execute(cache, args.fixture, 'smoke-2')
            scenario.require(a == b, 'nonreproducible checkpoints')
            report = {'status':'agreed', 'smoke_seconds':time.monotonic()-started,
                      'checkpoint_sha256':sha(cache/'smoke-1.json'),
                      'fixture_sha256':sha(args.fixture),
                      'bridge_sha256':sha(ROOT/'references/xmage/NeutralSmokeTest.java'),
                      'runner_sha256':sha(Path(__file__)),
                      'pins_sha256':sha(ROOT/'references/xmage/pins.json'),
                      'dependencies_sha256':sha(ROOT/'references/xmage/dependencies.json'),
                      'platform':platform.system()+'/'+platform.machine(),
                      'stdin':'closed', 'display':'unset', 'offline':True,
                      'smoke_timeout_seconds':180,
                      'logs':{n:sha(cache/(n+'.log')) for n in ('smoke-1', 'smoke-2')}}
            if args.command == 'acceptance':
                report['mutations'] = mutations(cache, args.fixture)
            (cache/'acceptance.json').write_text(json.dumps(report, indent=2)+'\n')
            print(json.dumps(report))
    except (ValueError, OSError, KeyError, subprocess.SubprocessError) as error:
        print(json.dumps({'status':'failed', 'error':str(error)}))
        return 2
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
