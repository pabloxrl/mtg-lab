"""Capture privileged source-native XMage AI games; never admission evidence."""
import argparse
import gzip
import hashlib
import lzma
from collections import Counter
import json
from pathlib import Path
import time

import xmage

ROOT = Path(__file__).resolve().parents[1]
BRIDGE = ROOT / 'references/xmage/RawAiCaptureTest.java'
CALLBACKS = set('priority pass shuffleLibrary cast chooseAbilityForCast chooseLandOrSpellAbility '
                'playLand activateAbility concede choose chooseTarget chooseTargetAmount chooseMulligan '
                'chooseUse choosePile announceX chooseReplacementEffect chooseTriggeredAbility chooseMode '
                'selectAttackers selectBlockers getAmount getMultiAmountWithIndividualConstraints playMana'.split())


def pack_trace(path):
    """Losslessly store the original JSONL bytes; verify before replacing codec."""
    path = Path(path)
    target = path.with_suffix('.xz')
    temporary = target.with_suffix('.xz.partial')
    if target.exists() or temporary.exists():
        raise FileExistsError('trace archive already exists: ' + str(target))
    original = hashlib.sha256()
    size = 0
    with gzip.open(path, 'rb') as source, lzma.open(temporary, 'xb', preset=3) as output:
        while chunk := source.read(1024 * 1024):
            original.update(chunk)
            size += len(chunk)
            output.write(chunk)
    restored = hashlib.sha256()
    with lzma.open(temporary, 'rb') as source:
        while chunk := source.read(1024 * 1024):
            restored.update(chunk)
    if restored.digest() != original.digest():
        raise ValueError('lossless trace archive verification failed')
    temporary.rename(target)
    path.unlink()
    return dict(path=target.name, jsonl_sha256=original.hexdigest(), jsonl_bytes=size)


def validate_trace(path):
    """Validate capture completeness, without certifying source rules or replay."""
    calls = []
    shuffles = Counter()
    counts = Counter()
    first = last = None
    players = []
    expected = {}
    bindings = {}
    manifest = json.loads((ROOT / 'data/cards/foundations_micro_v1.json').read_text())
    decks = {d['id']: d for d in manifest['decks']}
    definitions = {c['id']: c for c in manifest['cards']}
    def need(ok, message):
        if not ok:
            raise ValueError('incomplete raw capture: ' + message)
    def ability_fields(ability):
        need(isinstance(ability, dict), 'ability choice fields')
        need({'class', 'id', 'source', 'controller', 'source_zcc', 'type', 'targets', 'modes',
              'available_modes', 'mana_costs', 'mana_to_pay', 'costs'} <= set(ability), 'ability choice fields')
        mode_ids = set()
        for mode in ability['available_modes']:
            need(mode['key'] == mode['value']['id'], 'mode identity binding')
            mode_ids.add(mode['key'])
            for effect in mode['value']['effects']:
                need({'id', 'class', 'type'} <= set(effect), 'mode effect binding')
        need(set(ability['modes']) <= mode_ids, 'selected mode binding')
        for target in ability['targets']:
            need({'class', 'targets', 'min', 'max', 'not_target', 'zone_change_counters'} <= set(target),
                 'target choices/incarnation bindings')
            for selected in target['targets']:
                need(set(selected) == {'id', 'amount'}, 'target amount')
        for cost in ability['mana_costs'] + ability['mana_to_pay'] + ability['costs']:
            need({'class', 'id', 'paid', 'targets'} <= set(cost), 'cost/payment observations')
    def payload_value(value, schema):
        # These are pinned Player callback types, not inferred replay decisions.
        if schema.endswith('?'):
            return value is None or payload_value(value, schema[:-1])
        if schema == 'null':
            return value is None
        if schema in ('bool', 'int', 'str'):
            return type(value) is {'bool': bool, 'int': int, 'str': str}[schema]
        if schema.endswith('[]'):
            return isinstance(value, list) and all(payload_value(v, schema[:-2]) for v in value)
        if schema in ('modes', 'text_map', 'object_map'):
            item = {'modes': 'mode', 'text_map': 'str', 'object_map': 'card'}[schema]
            return isinstance(value, list) and all(isinstance(v, dict) and set(v) == {'key', 'value'}
                and payload_value(v['key'], 'str') and payload_value(v['value'], item) for v in value)
        if schema == 'map':
            return isinstance(value, list) and all(isinstance(v, dict) and set(v) == {'key', 'value'} for v in value)
        if not isinstance(value, dict):
            return False
        if schema == 'label':
            return set(value) == {'text_sha256'} and isinstance(value['text_sha256'], str) and len(value['text_sha256']) == 64
        if schema == 'ability':
            ability_fields(value)
            return True
        required = {
            'card': {'id', 'name', 'class'},
            'target': {'class', 'targets', 'min', 'max', 'not_target', 'zone_change_counters'},
            'cost': {'class', 'id', 'paid', 'targets', 'mana', 'payment', 'used'},
            'mode': {'id', 'targets', 'effects'},
            'message': {'message', 'min', 'max', 'default'},
            'amount_type': {'title', 'header', 'can_cancel'},
        }[schema]
        if not required <= set(value):
            return False
        if schema == 'target':
            return isinstance(value['targets'], list) and all(isinstance(v, dict) and set(v) == {'id', 'amount'} for v in value['targets'])
        if schema == 'cost':
            components = {'white', 'blue', 'black', 'red', 'green', 'colorless', 'generic', 'any', 'flag'}
            return all(isinstance(value[k], dict) and components <= set(value[k]) for k in ('mana', 'payment', 'used'))
        if schema == 'mode':
            return payload_value(value['targets'], 'target[]') and isinstance(value['effects'], list) and all(
                isinstance(e, dict) and {'id', 'class', 'type'} <= set(e) for e in value['effects'])
        if schema == 'message':
            return payload_value(value['message'], 'label') and all(type(value[k]) is int for k in ('min', 'max', 'default'))
        return True
    def callback_payload(point):
        method = point['kind'].split('/', 1)[1]
        returning = point['kind'].startswith('return/')
        detail = point.get('detail')
        if returning:
            need(isinstance(detail, dict) and set(detail) == {'arguments', 'result'}, 'callback payload return fields')
            args = detail['arguments']
        else:
            args = detail
        need(isinstance(args, list), 'callback payload arguments')
        signatures = {
            'priority': [('', 'bool')], 'pass': [('', 'null')],
            'shuffleLibrary': [('ability?', 'null')],
            'cast': [('ability bool null', 'bool')],
            'chooseAbilityForCast': [('card bool', 'ability?')],
            'chooseLandOrSpellAbility': [('card bool', 'ability?')],
            'playLand': [('card bool', 'bool')], 'activateAbility': [('ability', 'bool')],
            'choose': [('str target ability?', 'bool'), ('str target ability? map?', 'bool'),
                       ('str str[] target ability?', 'bool')],
            'chooseTarget': [('str target ability?', 'bool'), ('str str[] target ability?', 'bool')],
            'chooseTargetAmount': [('str target ability?', 'bool')],
            'chooseMulligan': [('', 'bool')],
            'chooseUse': [('str label ability?', 'bool'), ('str label label label label ability?', 'bool')],
            'choosePile': [('str label card[] card[]', 'bool')],
            'announceX': [('int int label ability? bool', 'int')],
            'chooseReplacementEffect': [('text_map object_map', 'int')],
            'chooseTriggeredAbility': [('ability[]', 'ability?')],
            'chooseMode': [('modes ability?', 'mode?')],
            'selectAttackers': [('str', 'null')], 'selectBlockers': [('ability? str', 'null')],
            'getAmount': [('int int label ability?', 'int')],
            'getMultiAmountWithIndividualConstraints': [('str message[] int int amount_type', 'int[]?')],
            'playMana': [('ability cost label', 'bool')],
        }
        matches = [(spec.split(), result) for spec, result in signatures.get(method, [])
                   if len(spec.split()) == len(args)]
        need(any(all(payload_value(v, t) for v, t in zip(args, types))
                 and (not returning or payload_value(detail['result'], result))
                 for types, result in matches), 'callback payload signature: ' + method)
    opener = lzma.open if Path(path).suffix == '.xz' else gzip.open
    with opener(path, 'rt') as stream:
        for sequence, line in enumerate(stream):
            point = json.loads(line)
            if first is None:
                need(point.get('kind') == 'configuration', 'configuration first')
                first = point
                detail = point['detail']
                need(detail['ai_class'] == 'mage.player.ai.ComputerPlayer7', 'concrete AI class')
                need(detail['test_mode'] is False and detail['skip_init_shuffling'] is False, 'normal opening')
                players = [p['id'] for p in point['state']['players']]
                need(len(players) == 2 and len(set(players)) == 2, 'both seats')
                for seat, deck in enumerate(detail['config']['decks']):
                    expected.update({f"{seat}/{c['card_id']}/{i}": players[seat]
                                     for c in decks[deck]['cards'] for i in range(c['copies'])})
                physical = {c['physical']: c['owner'] for c in point['state']['cards']}
                need(physical == expected and len(point['state']['cards']) == 80, 'both frozen 40-card decks')
                bindings = {c['id']: c['physical'] for c in point['state']['cards']}
            need(point.get('sequence') == sequence, 'record chronology')
            need(point.get('simulation') is False, 'simulation branch')
            need(last is None or last['kind'] != 'terminal', 'records after terminal')
            state = point['state']
            required = {'turn', 'step', 'active', 'priority', 'choosing', 'ended', 'players', 'cards',
                        'permanents', 'stack', 'combat', 'effects', 'pending_triggers', 'exile'}
            need(set(state) == required, 'semantic observation fields')
            need([p['id'] for p in state['players']] == players, 'seat identities')
            for p in state['players']:
                need({'hand', 'library', 'graveyard', 'mana', 'mana_native', 'life', 'lands_played',
                      'empty_draw', 'lost', 'won', 'quit', 'test_mode'} <= set(p), 'player observations')
                need(p['test_mode'] is False and p['quit'] is False, 'test-mode/concession')
                need(set(p['mana']) == {'white', 'blue', 'black', 'red', 'green', 'colorless', 'generic', 'any', 'flag'},
                     'mana components')
                need({'mana', 'items', 'auto_payment', 'restricted', 'unlocked_type'} <= set(p['mana_native']),
                     'mana source lots')
            for c in state['cards']:
                need(c['physical'] in expected and expected[c['physical']] == c['owner'], 'physical identity')
                need({'id', 'zcc', 'zone', 'name', 'types', 'subtypes', 'color', 'power', 'toughness', 'abilities'} <= set(c), 'card characteristics/incarnation')
                need(bindings.get(c['id']) == c['physical'], 'changed physical binding')
                need(c['name'] == definitions[c['physical'].split('/')[1]]['name'], 'physical card definition')
                for ability in c['abilities']:
                    ability_fields(ability)
            need(len({c['id'] for c in state['cards']}) == 80, 'physical card inventory')
            for permanent in state['permanents']:
                need({'id', 'physical', 'name', 'token', 'owner', 'controller', 'zcc', 'tapped', 'sick',
                      'damage', 'power', 'toughness', 'types', 'subtypes', 'color', 'abilities'} <= set(permanent),
                     'permanent characteristics/incarnation')
                if permanent['token']:
                    need(permanent['physical'] == 'token/' + permanent['id'], 'token physical identity')
                else:
                    need(bindings.get(permanent['id']) == permanent['physical'], 'permanent physical identity')
                for ability in permanent['abilities']:
                    ability_fields(ability)
            for item in state['stack']:
                need(set(item) == {'id', 'source', 'controller', 'ability'}, 'stack identity')
                ability_fields(item['ability'])
            for ability in state['pending_triggers']:
                ability_fields(ability)
            for effect in state['effects']:
                need({'id', 'class', 'duration', 'abilities', 'native_fields'} <= set(effect), 'active effect fields')
                need(effect['native_fields']['class'] == effect['class'], 'active effect native class')
                for ability in effect['abilities']:
                    ability_fields(ability)
            for group in state['combat']:
                need(set(group) == {'attackers', 'blockers', 'defender', 'blocked'}, 'combat observations')
            kind = point['kind']
            if kind.startswith(('call/', 'return/')):
                need(kind.split('/', 1)[1] in CALLBACKS, 'unsupported callback')
                need(point['actor'] in players, 'callback actor')
                callback_payload(point)
            else:
                need(kind in {'configuration', 'event/before', 'event/after', 'resolution/before',
                              'resolution/after', 'sba/after', 'terminal'}, 'unsupported boundary')
            counts[kind] += 1
            if kind.startswith('call/'):
                calls.append(point)
                if kind == 'call/chooseMulligan':
                    need(all(shuffles[p] >= 1 for p in players), 'mulligan before both initial shuffles')
            elif kind.startswith('return/'):
                need(bool(calls), 'return without choice')
                before = calls.pop()
                need(before['actor'] == point['actor'] and before['kind'][5:] == kind[7:], 'callback nesting/order')
                need(len(before['detail']) == len(point['detail']['arguments']), 'callback payload changed signature')
                if kind == 'return/shuffleLibrary':
                    actor = point['actor']
                    pre = next(p['library'] for p in before['state']['players'] if p['id'] == actor)
                    post = next(p['library'] for p in state['players'] if p['id'] == actor)
                    need(len(pre) == 40 and sorted(pre) == sorted(post), 'actual shuffle permutation')
                    shuffles[actor] += 1
                if kind == 'return/selectAttackers' and state['combat']:
                    counts['attacking_combats'] += 1
            if kind == 'event/before' and point['detail']['type'] == 'SPELL_CAST':
                counts['played_spells'] += 1
            last = point
    need(first is not None, 'configuration missing')
    need(last['kind'] == 'terminal' and not calls, 'terminal or complete callback stream missing')
    need(last['state']['ended'] is True, 'rules ending missing')
    losses = {p['id']: p for p in last['state']['players'] if p['lost']}
    need(losses and {p['player'] for p in last['detail']} == set(losses), 'terminal reasons')
    for loss in last['detail']:
        p = losses[loss['player']]
        need(loss['life_loss'] == (p['life'] <= 0) and loss['empty_draw'] == p['empty_draw']
             and (loss['life_loss'] or loss['empty_draw']), 'rules loss, not quota filler')
    need(counts['played_spells'] > 0 and counts['attacking_combats'] > 0, 'played spells/combat')
    need(counts['call/priority'] > 0 and counts['call/pass'] > 0
         and counts['call/chooseMulligan'] >= 2 and counts['resolution/after'] > 0, 'semantic boundaries')
    return dict(records=sum(v for k, v in counts.items() if k not in ('played_spells', 'attacking_combats')),
                played_spells=counts['played_spells'], attacking_combats=counts['attacking_combats'],
                shuffles=dict(shuffles), mulligan_decisions=counts['call/chooseMulligan'])


def capture(cache, output, seed=1, starter=0, decks=('red', 'green'),
            max_records=100000, timeout=1800, unsupported_probe=False):
    cache, output = Path(cache).resolve(), Path(output).resolve()
    output.mkdir(parents=True, exist_ok=False, mode=0o700)
    config = dict(seed=seed, starter=starter, decks=list(decks),
                  max_records=max_records, timeout_seconds=timeout, unsupported_probe=unsupported_probe)
    result = dict(schema_version=1, seed=seed, status='started', config=config)
    receipt = output / 'attempt.json'
    receipt.write_text(json.dumps(result, indent=2) + '\n')
    (output / 'config.json').write_text(json.dumps(config) + '\n')
    start = time.monotonic()
    try:
        if not (cache / xmage.SOURCE).is_dir():
            raise ValueError('missing pinned XMage cache: ' + str(cache))
        if type(seed) is not int or not -(2**63) <= seed < 2**63:
            raise ValueError('seed must be a signed 64-bit integer')
        if starter not in (0, 1) or len(decks) != 2 or any(d not in ('red', 'green') for d in decks):
            raise ValueError('unsupported frozen-deck configuration')
        if max_records <= 0 or timeout <= 0:
            raise ValueError('limits must be positive')
        xmage.verify_inputs(cache)
        if xmage.dependencies(cache) != json.loads((ROOT / 'references/xmage/dependencies.json').read_text()):
            raise ValueError('dependency lock mismatch')
        provenance = ROOT / 'references/xmage/raw-capture-provenance.json'
        for name, expected_hash in json.loads(provenance.read_text())['consulted_sources'].items():
            if xmage.sha(cache / xmage.SOURCE / name) != expected_hash:
                raise ValueError('capture provenance source mismatch: ' + name)
        sources = [BRIDGE, Path(__file__), ROOT / 'scripts/xmage.py',
                   ROOT / 'data/cards/foundations_micro_v1.json', ROOT / 'data/rules/cr-2026-09-25.json',
                   ROOT / 'references/xmage/pins.json', ROOT / 'references/xmage/dependencies.json', provenance]
        source_folder = output / 'sources'
        source_folder.mkdir()
        for source in (BRIDGE, Path(__file__)):
            (source_folder / source.name).write_bytes(source.read_bytes())
        result['pins'] = {str(p.relative_to(ROOT)): xmage.sha(p) for p in sources}
        upstream = cache / xmage.SOURCE
        source_paths = [
            'Mage.Server.Plugins/Mage.Player.AI/src/mage/player/ai/ComputerPlayer.java',
            'Mage.Server.Plugins/Mage.Player.AI.MAD/src/mage/player/ai/ComputerPlayer6.java',
            'Mage.Server.Plugins/Mage.Player.AI.MAD/src/mage/player/ai/ComputerPlayer7.java',
            'Mage/src/main/java/mage/util/RandomUtil.java',
            'Mage/src/main/java/mage/game/GameImpl.java',
            'Mage/src/main/java/mage/players/PlayerImpl.java',
        ]
        result['upstream_sources'] = {p: xmage.sha(upstream / p) for p in source_paths}
        result['toolchain'] = json.loads((ROOT / 'references/xmage/pins.json').read_text())
        result['jvm_limits'] = dict(maven='-Xmx2g', game='-Xmx3g')
        result['reference_variant'] = 'unmodified-upstream-raw'
        result['admitted'] = False
        target = cache / xmage.SOURCE / 'Mage.Tests/src/test/java/org/mage/test/mtglab/RawAiCaptureTest.java'
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(BRIDGE.read_bytes())
        command = xmage.maven(cache) + ['-o', '-pl', 'Mage.Tests', '-am', 'test',
                    '-Dtest=org.mage.test.mtglab.RawAiCaptureTest', '-Dsurefire.failIfNoSpecifiedTests=false',
                    '-Dmtglab.root=' + str(ROOT), '-Dmtglab.output=' + str(output),
                    '-DargLine=-Xmx3g -Djava.awt.headless=true']
        env = xmage.environment(cache)
        env['MAVEN_OPTS'] = '-Xmx2g -Djava.awt.headless=true'
        try:
            xmage.bounded(command, cache / xmage.SOURCE, env, output / 'xmage.log', timeout)
        finally:
            if (output / 'result.json').is_file():
                result.update(json.loads((output / 'result.json').read_text()))
        if 'Game end on critical error' in (output / 'xmage.log').read_text():
            result['status'] = 'failed'
            raise ValueError('source game ended on critical error')
        if result['status'] != 'complete':
            raise ValueError('source capture did not complete')
        result['validation'] = validate_trace(output / 'trace.jsonl.gz')
    except Exception as error:
        if result['status'] in ('started', 'complete'):
            result['status'] = 'truncated' if 'timeout after' in str(error) else 'failed'
        result['command_error'] = str(error)
        result.setdefault('error', str(error))
    result['trace_content'] = []
    try:
        for trace in sorted(output.glob('*.jsonl.gz')):
            result['trace_content'].append(pack_trace(trace))
    except Exception as error:
        result['archive_error'] = str(error)
        if result['status'] == 'complete':
            result['status'] = 'failed'
    result['elapsed_seconds'] = time.monotonic() - start
    result['artifacts'] = {str(p.relative_to(output)): xmage.sha(p)
                           for p in sorted(output.rglob('*')) if p.is_file() and p != receipt}
    receipt.write_text(json.dumps(result, indent=2) + '\n')
    return result


def acceptance(cache, output, seed=319):
    """Three real source invocations, with complete attempted-seed accounting."""
    output = Path(output).resolve()
    output.mkdir(parents=True, exist_ok=False, mode=0o700)
    attempts = []
    for name, options, expected, reason in (
            ('game', {}, 'complete', None),
            ('unsupported', {'unsupported_probe': True}, 'unsupported', 'unsupported callback: chooseRingBearer'),
            ('truncated', {'max_records': 12}, 'truncated', 'truncated: record limit')):
        result = capture(cache, output / name, seed=seed, **options)
        attempts.append(dict(name=name, seed=seed, expected=expected,
                             status=result['status'], reason_matched=reason is None or reason in result.get('error', ''),
                             receipt_sha256=xmage.sha(output / name / 'attempt.json')))
    report = dict(status='passed' if all(a['status'] == a['expected'] and a['reason_matched'] for a in attempts) else 'failed',
                  attempts=attempts, admitted_games=0)
    (output / 'acceptance.json').write_text(json.dumps(report, indent=2) + '\n')
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cache', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--seed', type=int, default=1)
    parser.add_argument('--starter', type=int, choices=(0, 1), default=0)
    parser.add_argument('--decks', nargs=2, choices=('red', 'green'), default=('red', 'green'))
    parser.add_argument('--max-records', type=int, default=100000)
    parser.add_argument('--timeout', type=int, default=1800)
    parser.add_argument('--unsupported-probe', action='store_true')
    parser.add_argument('--acceptance', action='store_true', help='run a real game plus unsupported/truncation controls')
    args = parser.parse_args()
    options = vars(args)
    check = options.pop('acceptance')
    if check and (args.starter != 0 or tuple(args.decks) != ('red', 'green') or
                  args.max_records != 100000 or args.timeout != 1800 or args.unsupported_probe):
        parser.error('--acceptance takes only --cache, --output and --seed')
    result = acceptance(args.cache, args.output, args.seed) if check else capture(**options)
    print(json.dumps(result))
    return 0 if result['status'] in ('complete', 'passed') else 1


if __name__ == '__main__':
    raise SystemExit(main())
