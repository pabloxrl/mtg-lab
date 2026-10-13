"""Real Forge AI capture using explicitly selected, unchanged frozen deck rows.

The original L0-006 capture and receipts remain unchanged. This additive profile
uses the same source bridge, AI, pins and normal reset; only deck seat assignment
is selected. Every source game is retained, including unsuccessful attempts.
"""
import argparse
from collections import Counter
import gzip
import json
import os
from pathlib import Path
import subprocess
import time

import forge
import forge_ai_capture as base
import forge_capture_patch as patch
import xmage

ROOT = Path(__file__).resolve().parents[1]
COSTS = base.COSTS
install = base.install
verify_toolchain = base.verify_toolchain
source_diagnostic = base.source_diagnostic

def validate_trace(path, selected_decks):
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
    decks = [Counter({names[c['card_id']]: c['copies'] for c in next(d for d in card_data['decks'] if d['id'] == key)['cards']}) for key in selected_decks]
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


def capture(args):
    if not isinstance(args.decks, list) or len(args.decks) != 2 or any(d not in ('red', 'green') for d in args.decks):
        raise ValueError('select exactly two unchanged frozen deck rows')
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
                   decks=args.decks, starting_seat='source random toss and AI choice',
                   opening=dict(life=20, hand=7, mulligan='London', filtered=False),
                   rules_version='cr-2026-09-25', privileged=True)
    receipt['source_profile'] = 'frozen-deck-selection-v1'
    receipt['capture_base_sha256'] = xmage.sha(ROOT/'scripts/forge_ai_capture.py')
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
        manifest = json.loads((ROOT/'data/cards/foundations_micro_v1.json').read_text())
        original_decks = manifest['decks']
        manifest['decks'] = [next(d for d in original_decks if d['id'] == key) for key in args.decks]
        selected_manifest = output/'selected-manifest.json'
        selected_manifest.write_text(json.dumps(manifest, indent=2)+'\n')
        result = output/'result.json'
        command = xmage.maven(cache) + ['-o', '-pl', 'forge-gui-desktop', '-am', 'test',
            '-Dtest=forge.gamesimulationtests.mtglab.RawAiCaptureTest',
            '-Dsurefire.failIfNoSpecifiedTests=false',
            '-Dmtglab.assets='+str(cache/forge.SOURCE/'forge-gui')+'/',
            '-Dmtglab.manifest='+str(selected_manifest),
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
            receipt['validation'] = validate_trace(output/'result.json.jsonl.gz', args.decks)
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



def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cache', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--seed', type=int, required=True)
    parser.add_argument('--decks', choices=['red,green', 'green,red', 'red,red', 'green,green'], required=True)
    parser.add_argument('--max-calls', type=int, default=10000)
    parser.add_argument('--timeout', type=int, default=300)
    parser.add_argument('--uninstrumented', action='store_true', help='Original-source audit control; not a recorded game')
    args = parser.parse_args()
    args.decks = args.decks.split(',')
    args.reject_callback = None
    return capture(args)


if __name__ == '__main__':
    raise SystemExit(main())
