"""Privileged raw Forge AI capture. This is not normalized replay or admission."""
import argparse
import copy
from collections import Counter
import gzip
import hashlib
import json
import os
import re
from pathlib import Path
import subprocess
import tarfile
import time

import forge
import forge_capture_patch as patch
import xmage

ROOT = Path(__file__).resolve().parents[1]
GAME = 'forge-game/src/main/java/forge/game/Game.java'
PLAYER = 'forge-game/src/main/java/forge/game/player/Player.java'
ACTION = 'forge-game/src/main/java/forge/game/GameAction.java'
COSTS = 'forge-ai/src/main/java/forge/ai/AiCostDecision.java'
RECORDER = 'forge-game/src/main/java/forge/game/MtgRawCapture.java'
TEST = 'forge-gui-desktop/src/test/java/forge/gamesimulationtests/mtglab/RawAiCaptureTest.java'
POM = 'forge-game/pom.xml'
TRANSFORMS = {patch.CONTROLLER: patch.instrument_controller,
              COSTS: patch.instrument_cost_decisions,
              GAME: patch.instrument_game, PLAYER: patch.instrument_shuffle, ACTION: patch.instrument_sba}


def sha_bytes(value):
    return hashlib.sha256(value).hexdigest()


def gson_dependency(text):
    # Same artifact/version already required by pinned forge-gui and in the
    # existing checksum inventory. This is an explicit external source patch.
    if text.count('</dependencies>') != 1:
        raise ValueError('Forge game dependency seam changed')
    return text.replace('</dependencies>', '<dependency><groupId>com.google.code.gson</groupId>'
                        '<artifactId>gson</artifactId><version>2.13.2</version></dependency></dependencies>')


def install(cache, uninstrumented=False):
    source = cache / forge.SOURCE
    pins = json.loads((ROOT / 'references/forge/pins.json').read_text())
    if xmage.sha(cache / 'forge.tar.gz') != pins['archive']['sha256']:
        raise ValueError('Forge archive pin mismatch')
    transformations = dict(TRANSFORMS, **{POM: gson_dependency})
    hashes, inventories = {}, {}
    # Always rebuild patches from immutable archive bytes, never a previously
    # edited source. Every unpatched upstream file is checked byte-for-byte.
    with tarfile.open(cache / 'forge.tar.gz') as archive:
        for member in archive:
            if not member.isfile():
                continue
            relative = str(Path(member.name).relative_to(forge.SOURCE))
            original = archive.extractfile(member).read()
            path = source / relative
            if relative in transformations:
                result = transformations[relative](original.decode())
                if isinstance(result, tuple):
                    result, inventories[relative] = result
                if uninstrumented and relative != POM:
                    result = original.decode()
                modified = result.encode()
                path.write_bytes(modified)
                hashes[relative] = dict(original=sha_bytes(original), modified=sha_bytes(modified))
            elif not path.is_file() or xmage.sha(path) != sha_bytes(original):
                raise ValueError('unpinned upstream source: ' + relative)
    for target, local in [(RECORDER, 'MtgRawCapture.java'), (TEST, 'RawAiCaptureTest.java')]:
        content = (ROOT / 'references/forge' / local).read_bytes()
        (source / target).parent.mkdir(parents=True, exist_ok=True)
        (source / target).write_bytes(content)
        hashes[target] = dict(original=None, modified=sha_bytes(content))
    return hashes, inventories


def validate_trace(path):
    """Structural/source-completeness checks, never a rules-correctness oracle."""
    def need(condition, message):
        if not condition:
            raise ValueError('incomplete Forge raw capture: ' + message)
    inventories = json.loads((ROOT/'references/forge/capture-callbacks.json').read_text())
    callbacks = {m['id']: (('cost.' if source == COSTS else '') + m['name'],
                          len(patch.split_parameters(m['signature'].split('(', 1)[1][:-1])) + (source == COSTS))
                 for source, entries in inventories.items() for m in entries}
    card_fields = {'id','instance','timestamp','name','owner','controller','zone','token','tapped',
                   'printing','sick','damage','power','toughness','types','colors','keywords','pt_boosts','keyword_changes'}
    ability_fields = {'id','host','actor','api','spell','mana_ability','trigger','cost','targets','paid',
                      'paying_mana','paying_mana_abilities','optional_costs','trigger_context','sub'}
    def payload(value):
        if isinstance(value, list):
            for item in value:
                payload(item)
        elif isinstance(value, dict):
            if 'instance' in value or 'timestamp' in value or 'pt_boosts' in value:
                need(card_fields <= value.keys(), 'card fields/effects in callback or state')
            if ('host' in value and 'mode' not in value and 'ability_id' not in value) or 'api' in value or 'paying_mana' in value:
                need(ability_fields <= value.keys(), 'ability choices/payment/trigger fields')
            for item in value.values():
                payload(item)
    counts, methods = Counter(), Counter()
    pending, before = [], {}
    players, identities = None, {}
    started = ended = False
    card_data = json.loads((ROOT/'data/cards/foundations_micro_v1.json').read_text())
    names = {c['id']: c['name'] for c in card_data['cards']}
    decks = [Counter({names[c['card_id']]: c['copies'] for c in d['cards']}) for d in card_data['decks']]
    observed_opening = set()
    last = None
    with gzip.open(path, 'rt') as stream:
        for index, line in enumerate(stream):
            row = json.loads(line)
            need(not ended, 'extra records after footer')
            need(type(row.get('index')) is int and row['index'] == index, 'record index gap/order')
            kind = row.get('kind')
            need(kind in {'start', 'finish', 'enter', 'leave', 'event', 'shuffle', 'random', 'boundary'}, 'failure/unsupported/unknown record')
            need(started or kind == 'start', 'missing start')
            counts[kind] += 1
            if kind == 'start':
                need(not started and row.get('schema_version') == 1, 'duplicate start/unsupported schema'); started = True
            if kind == 'random':
                need(type(row.get('bits')) is int and 1 <= row['bits'] <= 32, 'random bit width')
                need(type(row.get('result')) is int and row.get('draw') == counts['random'], 'random draw')
                continue
            payload(row)
            state = row.get('state')
            need(isinstance(state, dict) and {'turn','phase','active','priority','terminal','players','stack','combat','pending_triggers'} <= state.keys(), 'state fields')
            ps = state['players']
            need(len(ps) == 2, 'two seats')
            ids = [p['player']['player_id'] for p in ps]
            if players is None:
                players = ids
                need(len(set(ids)) == 2 and [p['life'] for p in ps] == [20,20], 'normal starting life/seats')
            need(ids == players, 'seat identity changed')
            for p in ps:
                need({'zones','mana','outcome','life'} <= p.keys(), 'player fields')
                need({'Library','Hand','Battlefield','Graveyard','Exile','Command'} <= p['zones'].keys(), 'ordered zones')
                for cards in p['zones'].values():
                    for c in cards:
                        need({'id','instance','timestamp','name','owner','controller','zone','token','tapped','printing','sick','damage','power','toughness','types','colors','keywords','pt_boosts','keyword_changes'} <= c.keys(), 'card fields/effects')
                        identity = (c['name'], c['owner']['player_id'], c['token'])
                        need(identities.setdefault(c['id'], identity) == identity, 'physical identity changed')
            if kind == 'enter':
                need(row.get('actor', {}).get('player_id') in players, 'callback actor')
                need(isinstance(row.get('arguments'), list), 'callback arguments')
                need(row.get('call') == counts['enter'], 'callback index gap')
                need(isinstance(row.get('method_id'), str) and len(row['method_id']) == 16, 'callback signature')
                need(callbacks.get(row['method_id']) == (row.get('method'), len(row['arguments'])), 'unknown callback signature/name/arguments')
                methods[row['method']] += 1
                pending.append((row['call'], len(row['arguments'])))
            elif kind == 'leave':
                need(pending and pending[-1][0] == row.get('call'), 'unmatched callback return')
                _, arity = pending.pop()
                need('result' in row and isinstance(row.get('arguments'), list), 'callback return/after arguments')
                need(len(row['arguments']) == arity, 'callback return argument arity')
            elif kind == 'shuffle':
                actor = row.get('actor', {}).get('player_id')
                need(actor in players, 'shuffle actor')
                cards = row['library']; card_ids = [c['id'] for c in cards]
                need(len(card_ids) == len(set(card_ids)), 'duplicate physical shuffle identity')
                if row['stage'] == 'before':
                    need(actor not in before, 'nested shuffle')
                    before[actor] = set(card_ids)
                    if actor not in observed_opening:
                        need(Counter(c['name'] for c in cards) == decks[players.index(actor)], 'frozen 40-card opening')
                        observed_opening.add(actor)
                elif row['stage'] == 'after':
                    need(before.pop(actor, None) == set(card_ids), 'shuffle loses physical copies')
                else:
                    need(False, 'unknown shuffle stage')
            elif kind == 'boundary':
                need(row.get('boundary') in ('sba-before','sba-after'), 'unknown rules boundary')
                counts[row['boundary']] += 1
            elif kind == 'event':
                need(isinstance(row.get('event'), str) and isinstance(row.get('payload'), dict), 'source event payload')
                counts[row['event']] += 1
            elif kind == 'finish':
                need(row.get('callback_count') == counts['enter'] and row.get('random_count') == counts['random'] and row.get('record_count') == index + 1, 'footer completeness counts')
                need(row.get('status') == 'complete' and not row.get('first_error'), 'noncomplete footer')
                need(state['terminal'] is True and not pending and not before, 'unfinished game/callback/shuffle')
                outcomes = [p['outcome'] for p in ps]
                need(all(isinstance(o, dict) for o in outcomes), 'missing terminal reasons')
                need(sum(o['won'] for o in outcomes) == 1, 'unsupported terminal verdict')
                need(all(o['loss'] in (None,'LifeReachedZero','Milled') for o in outcomes), 'nonnormal termination')
                ended = True
            last = row
    need(ended, 'missing complete footer')
    need(set(players) == observed_opening, 'missing opening shuffle')
    need(methods['mulliganKeepHand'] >= 2 and methods['chooseStartingPlayer'] == 1, 'missing normal opening choices')
    need(counts['random'] > 0, 'missing actual chance draws')
    need(counts['sba-before'] > 0 and counts['sba-before'] == counts['sba-after'], 'missing SBA boundaries')
    for event in ['GameEventSpellAbilityCast','GameEventSpellResolved','GameEventCardTapped',
                  'GameEventPlayerDamaged','GameEventAttackersDeclared','GameEventGameOutcome']:
        need(counts[event] > 0, 'missing played witness: ' + event)
    return dict(records=last['record_count'],
                kinds=dict(counts), callbacks=dict(methods), terminal=last['state'])


def source_diagnostic(log):
    """Identify upstream swallowed evaluation failures, not just Maven exit status."""
    for line in log.splitlines():
        if re.match(r'^\s*(?:Caused by:\s*)?[\w.$]+(?:Exception|Error)(?::|$)', line) or 'AI eval thread at timeout:' in line:
            return line.strip()
    return None


def verify_toolchain(cache):
    pins = json.loads((ROOT/'references/forge/pins.json').read_text())
    import platform
    if platform.system() != 'Linux' or platform.machine() != 'aarch64':
        raise ValueError('capture requires pinned Linux ARM64 toolchain')
    env = forge.environment(cache)
    java = subprocess.run([str(Path(os.environ['JAVA_HOME'])/'bin/java'), '-version'],
                          capture_output=True, text=True, check=True, timeout=15,
                          stdin=subprocess.DEVNULL, env=env)
    maven = subprocess.run(['mvn','-version'], capture_output=True, text=True, check=True,
                           timeout=15, stdin=subprocess.DEVNULL, env=env)
    if 'Temurin-'+pins['jdk_runtime']+' ' not in java.stderr or 'Apache Maven '+pins['maven']+' ' not in maven.stdout:
        raise ValueError('capture toolchain differs from pins')
    if any(p.is_file() for p in (cache/'home').rglob('*')):
        raise ValueError('unexpected Forge custom user data')
    return dict(java=java.stderr.strip(), maven=maven.stdout.strip(), platform=platform.platform())


def capture(args):
    if not -(2**63) <= args.seed < 2**63 or args.max_calls < 1 or args.timeout < 1:
        raise ValueError('invalid seed/capture budget/timeout')
    if args.cache.resolve().is_relative_to(ROOT):
        raise ValueError('Forge source cache must be outside checkout')
    os.umask(0o077)
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    cache = args.cache.resolve()
    receipt = dict(schema_version=1, source='forge', status='failed', seed=args.seed,
                   mode='uninstrumented-audit' if args.uninstrumented else 'record',
                   started_game=False, raw_only=True,
                   upstream_commit='95dc682bf92460f49cebd7a9578f06ccf60d5569',
                   archive_sha256='54ecef193bf0f1ff85afa827e2e1d0fab78594dd0234d5fc461223ce5db5cd20',
                   ai=dict(controller='forge.ai.PlayerControllerAi', brain='forge.ai.AiController',
                           profile='Default', option=None, full_simulation=False, evaluation_timeout_seconds=5),
                   rng='java.util.Random (48-bit LCG), observed next(bits) returns',
                   decks=['red','green'], starting_seat='source random toss and AI choice',
                   opening=dict(life=20, hand=7, mulligan='London', filtered=False),
                   rules_version='cr-2026-09-25', privileged=True)
    receipt['status'] = 'started'
    receipt['max_calls'] = args.max_calls
    receipt['timeout_seconds'] = args.timeout
    receipt['reject_callback'] = args.reject_callback
    (output/'receipt.json').write_text(json.dumps(receipt, indent=2)+'\n')
    started = time.monotonic()
    try:
        receipt['toolchain'] = verify_toolchain(cache)
        patches, inventories = install(cache, args.uninstrumented)
        if inventories != json.loads((ROOT/'references/forge/capture-callbacks.json').read_text()):
            raise ValueError('capture callback inventory mismatch')
        receipt['patches'] = patches
        receipt['callbacks'] = inventories
        receipt['pins_sha256'] = xmage.sha(ROOT/'references/forge/pins.json')
        receipt['dependencies_sha256'] = xmage.sha(ROOT/'references/forge/dependencies.json')
        receipt['card_manifest_sha256'] = xmage.sha(ROOT/'data/cards/foundations_micro_v1.json')
        receipt['runner_sha256'] = xmage.sha(Path(__file__))
        receipt['patch_runner_sha256'] = xmage.sha(ROOT/'scripts/forge_capture_patch.py')
        result = output/'result.json'
        command = xmage.maven(cache) + ['-o', '-pl', 'forge-gui-desktop', '-am', 'test',
            '-Dtest=forge.gamesimulationtests.mtglab.RawAiCaptureTest',
            '-Dsurefire.failIfNoSpecifiedTests=false',
            '-Dmtglab.assets='+str(cache/forge.SOURCE/'forge-gui')+'/',
            '-Dmtglab.manifest='+str(ROOT/'data/cards/foundations_micro_v1.json'),
            '-Dmtglab.output='+str(result), '-Dmtglab.seed='+str(args.seed),
            '-Dmtglab.record='+str(not args.uninstrumented).lower(),
            '-Dmtglab.maxCalls='+str(args.max_calls)]
        if args.reject_callback:
            command.append('-Dmtglab.rejectCallback='+args.reject_callback)
        receipt['command'] = command
        env = forge.environment(cache)
        env['MAVEN_OPTS'] = '-Xmx3g -Djava.awt.headless=true'
        report = cache/forge.SOURCE/'forge-gui-desktop/target/surefire-reports/TEST-forge.gamesimulationtests.mtglab.RawAiCaptureTest.xml'
        report.unlink(missing_ok=True)
        xmage.bounded(command, cache/forge.SOURCE, env, output/'run.log', args.timeout)
        (output/'test-report.xml').write_bytes(report.read_bytes())
        diagnostic = source_diagnostic((output/'run.log').read_text(errors='replace'))
        if diagnostic:
            receipt['source_diagnostic'] = diagnostic
            raise ValueError('source evaluation failed: ' + diagnostic)
        import xml.etree.ElementTree as ET
        suite = ET.parse(report).getroot()
        if [case.get('name') for case in suite.findall('testcase')] != ['rawAiCapture'] or suite.get('tests') != '1' or any(suite.get(k) != '0' for k in ('failures','errors','skipped')):
            raise ValueError('missing actual passing unskipped source test')
        receipt['audit'] = json.loads(result.read_text())
        if not args.uninstrumented:
            receipt['validation'] = validate_trace(output/'result.json.jsonl.gz')
        if xmage.dependencies(cache) != json.loads((ROOT/'references/forge/dependencies.json').read_text()):
            raise ValueError('Forge dependency inventory mismatch')
        receipt['status'] = 'complete'
    except (ValueError, OSError, KeyError, TypeError, subprocess.SubprocessError) as error:
        receipt['error'] = str(error)
        log = (output/'run.log').read_text(errors='replace') if (output/'run.log').exists() else ''
        receipt['status'] = ('unsupported' if 'unsupported raw' in log else
                             'truncated' if 'raw capture budget: truncated' in log or 'timeout after' in str(error)
                             or 'TimeoutException' in log or 'AI eval thread at timeout:' in log
                             else 'failed')
    finally:
        receipt['elapsed_seconds'] = time.monotonic() - started
        receipt['started_game'] = (output/'result.json.started').exists()
        receipt['artifacts'] = {p.name: xmage.sha(p) for p in sorted(output.iterdir()) if p.is_file() and p.name != 'receipt.json'}
        (output/'receipt.json').write_text(json.dumps(receipt, indent=2)+'\n')
        for path in output.iterdir():
            if path.is_file():
                path.chmod(0o400)
    print(json.dumps(dict(status=receipt['status'], output=str(output), error=receipt.get('error'))))
    return 0 if receipt['status'] == 'complete' else 2


def acceptance(args):
    os.umask(0o077)
    base = args.output.resolve()
    base.mkdir(parents=True, exist_ok=False)
    summary = dict(status='started', seed=args.seed, attempts=[])
    path = base/'acceptance.json'
    path.write_text(json.dumps(summary, indent=2)+'\n')
    try:
        for name, expected in [('game','complete'), ('uninstrumented','complete'),
                               ('unsupported','unsupported'), ('truncated','truncated')]:
            child = copy.copy(args)
            child.output = base/name
            child.uninstrumented = name == 'uninstrumented'
            child.reject_callback = 'chooseSpellAbilityToPlay' if name == 'unsupported' else None
            child.max_calls = 20 if name == 'truncated' else args.max_calls
            code = capture(child)
            receipt = json.loads((child.output/'receipt.json').read_text())
            summary['attempts'].append(dict(name=name, status=receipt['status'],
                                           receipt_sha256=xmage.sha(child.output/'receipt.json')))
            if receipt['status'] != expected or code != (0 if expected == 'complete' else 2):
                raise ValueError(name + ' did not produce intended status: ' + str(receipt.get('error')))
            if name in ('unsupported', 'truncated'):
                kinds = Counter()
                last = None
                with gzip.open(child.output/'result.json.jsonl.gz', 'rt') as stream:
                    for line in stream:
                        last = json.loads(line); kinds[last['kind']] += 1
                expected_kind = 'unsupported' if name == 'unsupported' else 'truncation'
                if not kinds[expected_kind] or last['state']['terminal'] or last['status'] == 'complete':
                    raise ValueError(name + ' did not retain a nonterminal source failure')
        game = json.loads((base/'game/result.json').read_text())
        audit = json.loads((base/'uninstrumented/result.json').read_text())
        if game != audit:
            raise ValueError('recording changed full event-state audit or terminal state')
        receipts = [json.loads((base/a['name']/'receipt.json').read_text()) for a in summary['attempts']]
        for target in (RECORDER, TEST):
            if len({r['patches'][target]['modified'] for r in receipts}) != 1:
                raise ValueError('acceptance mixed capture implementations')
        summary.update(status='passed', audit_sha256=game['audit_sha256'])
    except (ValueError, OSError, KeyError) as error:
        summary.update(status='failed', error=str(error))
    path.write_text(json.dumps(summary, indent=2)+'\n')
    path.chmod(0o400)
    print(json.dumps(summary))
    return 0 if summary['status'] == 'passed' else 2


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cache', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--seed', type=int, default=324)
    parser.add_argument('--max-calls', type=int, default=10000)
    parser.add_argument('--timeout', type=int, default=300)
    parser.add_argument('--uninstrumented', action='store_true')
    parser.add_argument('--reject-callback', help='Explicit negative-control fault injection')
    parser.add_argument('--acceptance', action='store_true', help='Run a real game, uninstrumented audit and two real negative controls')
    args = parser.parse_args()
    return acceptance(args) if args.acceptance else capture(args)


if __name__ == '__main__':
    raise SystemExit(main())
