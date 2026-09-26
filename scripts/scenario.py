"""Strict offline neutral-scenario v1 and scoped capability validation; no engine."""
import argparse
from collections import Counter
import copy
from datetime import date
import hashlib
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = {'positive', 'negative', 'interaction', 'regression'}
INVARIANTS = {'state_unchanged', 'rng_unchanged', 'decision_unchanged', 'private_information_unchanged'}
CHOICES = {
    'pass': ['pass'], 'cast': ['mode', 'targets', 'payment', 'discard'],
    'activate': ['targets', 'payment'], 'target_trigger': ['targets'], 'play_land': ['land'],
    'attack': ['attackers'], 'block': ['blockers'], 'assign_damage': ['damage'],
    'mulligan': ['mulligan'], 'keep': ['keep'], 'bottom': ['bottom'],
    'discard': ['discard'], 'order_triggers': ['trigger_order'], 'concede': ['concede'],
}
STEPS = {'beginning': {'untap', 'upkeep', 'draw'}, 'precombat_main': {'main'},
         'combat': {'begin_combat', 'declare_attackers', 'declare_blockers', 'combat_damage', 'end_combat'},
         'postcombat_main': {'main'}, 'ending': {'end', 'cleanup'}}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':'),
                                    ensure_ascii=False, allow_nan=False).encode()).hexdigest()


def _pairs(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, f'duplicate field: {key}')
        result[key] = value
    return result


def load(path):
    path = Path(path)
    require(path.is_file(), 'input must be a regular file')
    require(path.stat().st_size <= 4 * 1024 * 1024, 'input exceeds 4 MiB limit')
    def invalid(value):
        raise ValueError(f'nonfinite JSON value: {value}')
    return json.loads(path.read_text(encoding='utf-8'), object_pairs_hook=_pairs,
                      parse_constant=invalid)


def shape(value, schema, path='$'):
    """Implement exactly the keywords used by our bundled JSON Schema.

    No remote resolution, coercion, optional defaults or permissive unknown keywords.
    External draft-2020-12 validators can also validate the published shape schema;
    semantic checks below remain necessary.
    """
    known = {'$schema', '$id', 'title', 'type', 'properties', 'required',
             'additionalProperties', 'items', 'minItems', 'minLength', 'minimum',
             'pattern', 'enum', 'anyOf'}
    require(set(schema) <= known, 'unsupported schema keyword')
    if 'anyOf' in schema:
        for branch in schema['anyOf']:
            try:
                shape(value, branch, path)
                return
            except ValueError:
                pass
        raise ValueError(f'{path}: no valid shape alternative')
    if 'enum' in schema:
        require(any(type(value) is type(x) and value == x for x in schema['enum']), f'{path}: invalid enum')
    types = {'object': dict, 'array': list, 'string': str, 'integer': int,
             'boolean': bool, 'null': type(None)}
    if 'type' in schema:
        require(type(value) is types[schema['type']], f'{path}: expected {schema["type"]}')
    if schema.get('type') == 'object':
        require(set(value) == set(schema['properties']), f'{path}: missing or unknown fields')
        for key, item in value.items():
            shape(item, schema['properties'][key], path + '/' + key)
    if schema.get('type') == 'array':
        require(len(value) >= schema.get('minItems', 0), f'{path}: incomplete array')
        for index, item in enumerate(value):
            shape(item, schema['items'], f'{path}/{index}')
    if 'minLength' in schema:
        require(len(value.strip()) >= schema['minLength'], f'{path}: blank string')
    if 'minimum' in schema:
        require(value >= schema['minimum'], f'{path}: below minimum')
    if 'pattern' in schema:
        require(re.search(schema['pattern'], value), f'{path}: malformed value')


def fields(value, keys):
    require(type(value) is dict and set(value) == set(keys.split()), 'missing or unknown fields: ' + keys)


def text(value):
    require(type(value) is str and value.strip(), 'expected nonblank string')


def sha(value):
    require(type(value) is str and re.fullmatch('[0-9a-f]{64}', value), 'invalid SHA-256')


def unique(values, label):
    require(len(values) == len(set(values)), f'duplicate {label}')


def list_of_text(value, nonempty=True):
    require(type(value) is list and (value or not nonempty), 'expected string array')
    for item in value:
        text(item)
    unique(value, 'array entry')


def source_contract():
    cards_path = ROOT / 'data/cards/foundations_micro_v1.json'
    cards = load(cards_path)
    rules = load(ROOT / 'data/rules/cr-2026-09-25.json')
    return cards, {'version': rules['rules_version'], 'sha256': rules['sha256']}, {
        'version': cards['revision'], 'sha256': hashlib.sha256(cards_path.read_bytes()).hexdigest()}


def rfc_requirements():
    rfc = (ROOT / 'doc/rfcs/0002-first-mvp.md').read_text()
    rows = rfc.split('### Required scenario families\n')[1].split('\nTest both')[0]
    families = {}
    for row in rows.splitlines():
        if row.startswith('| ') and not row.startswith(('| Family', '| ---')):
            name, clauses = [part.strip() for part in row.strip('|').split('|')]
            families[name] = clauses.split('; ')
    supported = rfc.split('Supported behavior:\n')[1].split('\nImplement these')[0]
    return families, [line[2:] for line in supported.strip().splitlines()]


def validate_registry(registry, fixtures=(), require_passed=False):
    shape(registry, load(ROOT / 'schemas/capability-registry-v1.json'))
    fields(registry, 'registry_version scope families supported_requirements capabilities')
    require(type(registry['registry_version']) is int and registry['registry_version'] == 1,
            'unsupported registry version')
    require(registry['scope'] == 'foundations_micro_v1', 'unsupported scope')
    for key in ('families', 'supported_requirements', 'capabilities'):
        require(type(registry[key]) is list and registry[key], f'missing {key}')
    families, supported = rfc_requirements()
    caps = {}
    for cap in registry['capabilities']:
        fields(cap, 'id family description rule_references implementation required_evidence')
        for key in ('id', 'family', 'description'):
            text(cap[key])
        require(cap['id'] not in caps, 'duplicate capability')
        list_of_text(cap['rule_references'])
        require(cap['implementation'] in ('planned', 'implemented'), 'unsupported implementation status')
        fields(cap['required_evidence'], 'positive negative interaction regression')
        caps[cap['id']] = cap
    used = set()
    def mappings(rows, expected):
        require(type(rows) is list, 'missing requirement mappings')
        texts = []
        for row in rows:
            fields(row, 'text capabilities')
            text(row['text'])
            list_of_text(row['capabilities'])
            require(set(row['capabilities']) <= caps.keys(), 'unresolved capability mapping')
            texts.append(row['text'])
            used.update(row['capabilities'])
        require(texts == expected, 'missing or changed RFC coverage requirement')
    family_ids, family_names = [], []
    for family in registry['families']:
        fields(family, 'id name requirements')
        text(family['id'])
        require(family['name'] in families, 'unknown scenario family')
        family_ids.append(family['id'])
        family_names.append(family['name'])
        mappings(family['requirements'], families[family['name']])
        for row in family['requirements']:
            require(all(caps[c]['family'] == family['id'] for c in row['capabilities']), 'wrong family mapping')
    unique(family_ids, 'family ID')
    require(family_names == list(families), 'missing or reordered RFC family')
    mappings(registry['supported_requirements'], supported)
    card_ids = {card['behavior_id'] for card in source_contract()[0]['cards']}
    require(card_ids <= caps.keys(), 'missing scoped card/token behavior')
    require(used | card_ids == caps.keys(), 'capability outside RFC/card scope')
    require(all(cap['family'] in family_ids for cap in caps.values()), 'unresolved family')
    fixture_map = {}
    for fixture in fixtures:
        validate(fixture, registry, check_registry=False)
        require(fixture['fixture_id'] not in fixture_map, 'duplicate fixture ID')
        fixture_map[fixture['fixture_id']] = fixture
    counts = {state: 0 for state in ('planned', 'executed', 'passed', 'failed', 'unsupported', 'unavailable')}
    gaps = []
    for cap in caps.values():
        for category, evidence in cap['required_evidence'].items():
            require(type(evidence) is list, 'evidence must be an array')
            passed = False
            keys = []
            for entry in evidence:
                fields(entry, 'fixture_id fixture_revision status engine engine_revision artifact_sha256 artifact_url')
                text(entry['fixture_id'])
                sha(entry['fixture_revision'])
                require(entry['status'] in counts, 'unknown evidence status')
                require(entry['engine'] in ('native', 'xmage', 'forge'), 'unsupported evidence engine')
                require(entry['fixture_id'] in fixture_map, 'unresolved evidence fixture')
                f = fixture_map[entry['fixture_id']]
                require(entry['fixture_revision'] == f['provenance']['fixture_revision'], 'stale evidence fixture')
                require(cap['id'] in f['required_capabilities'], 'evidence does not cover capability')
                require(f['provenance']['review']['status'] == 'accepted-for-m0', 'unreviewed evidence fixture')
                key = (entry['fixture_id'], entry['engine'])
                keys.append(key)
                if entry['status'] == 'planned':
                    require(all(entry[k] is None for k in ('engine_revision','artifact_sha256','artifact_url')), 'planned evidence claims execution')
                else:
                    require(type(entry['engine_revision']) is str and re.fullmatch('[0-9a-f]{40}',entry['engine_revision']), 'unresolved engine pin')
                    sha(entry['artifact_sha256'])
                    text(entry['artifact_url'])
                counts[entry['status']] += 1
                passed |= entry['status'] == 'passed'
            unique(keys, 'evidence result')
            if not passed:
                gaps.append(cap['id'] + ':' + category)
                require(cap['implementation'] == 'planned', 'implemented capability lacks mandatory passing evidence')
    require(not require_passed or not gaps, 'missing executed/passed coverage: ' + ', '.join(gaps))
    return {**counts, 'required_slots': 4 * len(caps), 'missing_passed': len(gaps), 'gaps': gaps}


def pointer(value, path):
    for part in path.split('/')[1:]:
        part = part.replace('~1', '/').replace('~0', '~')
        try:
            value = value[int(part)] if type(value) is list else value[part]
        except (KeyError, IndexError, ValueError, TypeError) as error:
            raise ValueError('unresolved assertion path: ' + path) from error
    return value


def validate_state(state, cards):
    require(state['step'] in STEPS[state['phase']], 'phase/step mismatch')
    require([p['seat'] for p in state['players']] == [0, 1], 'exactly seats 0 and 1 required')
    objects = {o['id']: o for o in state['objects']}
    require(len(objects) == len(state['objects']), 'duplicate object identity')
    card_map = {c['id']: c for c in cards['cards']}
    locations = []
    for player in state['players']:
        for zone, ids in player['zones'].items():
            for oid in ids:
                require(oid in objects, 'unresolved zone object')
                obj = objects[oid]
                require(obj['owner'] == player['seat'], 'zone owner mismatch')
                if zone != 'battlefield':
                    require(not obj['status']['token'], 'token outside battlefield in settled synthetic state')
                locations.append(oid)
    historic = [x['last_known_source'] for x in state['stack'] + state['effects'] if x['last_known_source'] is not None]
    for obj in list(objects.values()) + historic:
        require(obj['card_id'] in card_map, 'unsupported card')
        require(obj['status']['token'] == (card_map[obj['card_id']]['kind'] == 'token'), 'token definition mismatch')
        require(obj['status']['controlled_since_turn'] <= state['turn'], 'future control timestamp')
        require(set(obj['status']['keywords']) <= {'Flying','Reach','Haste','Vigilance','Trample','Deathtouch'}, 'unsupported keyword')
        require(obj['owner'] == obj['controller'], 'unsupported control change')
    for item in state['stack']:
        require(item['source'] in objects or (item['last_known_source'] is not None and
                item['last_known_source']['id'] == item['source']), 'unresolved stack source')
        if item['kind'] == 'spell':
            locations.append(item['source'])
    unique([item['id'] for item in state['stack']], 'stack identity')
    unique(locations, 'zone membership')
    require(set(locations) == set(objects), 'object without zone membership')
    for effect in state['effects']:
        require((effect['source'] in objects or (effect['last_known_source'] is not None and
                effect['last_known_source']['id'] == effect['source'])) and
                set(effect['targets']) <= objects.keys(), 'unresolved effect object')
        require(effect['keyword'] is None or effect['keyword'] in {'Flying','Reach','Haste','Vigilance','Trample','Deathtouch'}, 'unsupported effect keyword')


def validate_action(action):
    require(action['kind'] in CHOICES, 'unsupported action kind')
    choices = action['choices']
    require([c['kind'] for c in choices] == CHOICES[action['kind']], 'incomplete or reordered choice script')
    require(all(c['actor'] == action['actor'] for c in choices), 'choice actor mismatch')
    require((action['source'] is not None) == (action['kind'] in ('cast','activate','play_land','target_trigger')), 'action source mismatch')
    for choice in choices:
        if choice['kind'] in ('pass', 'keep', 'mulligan', 'concede'):
            require(choice['values'] == [], 'unexpected choice values')
        else:
            for value in choice['values']:
                if choice['kind'] == 'blockers':
                    fields(value, 'blocker attacker')
                    text(value['blocker'])
                    text(value['attacker'])
                elif choice['kind'] == 'damage':
                    fields(value, 'source target amount')
                    text(value['source'])
                    text(value['target'])
                    require(type(value['amount']) is int and value['amount'] >= 0, 'invalid damage allocation')
                elif choice['kind'] == 'payment':
                    fields(value, 'resource amount')
                    text(value['resource'])
                    require(type(value['amount']) is int and value['amount'] > 0, 'invalid payment allocation')
                else:
                    text(value)
                    require(value not in ('auto', 'default', 'ai', 'random', '*'), 'unscripted choice')
            if choice['kind'] not in ('damage', 'payment'):
                unique([digest(v) for v in choice['values']], 'semantic selection')


def validate(fixture, registry=None, check_registry=True):
    # Reject non-JSON values, including NaN passed through the Python API.
    digest(fixture)
    shape(fixture, load(ROOT / 'schemas/neutral-scenario-v1.json'))
    registry = registry if registry is not None else load(ROOT / 'data/capabilities-v1.json')
    if check_registry:
        validate_registry(registry, [fixture])
    cards, rules_pin, cards_pin = source_contract()
    require(fixture['rules'] == rules_pin and fixture['cards'] == cards_pin, 'unresolved rules/card pin')
    unique(fixture['required_capabilities'], 'required capability')
    require(set(fixture['required_capabilities']) <= {c['id'] for c in registry['capabilities']}, 'unsupported capability')
    p = fixture['provenance']
    require(p['fixture_id'] == fixture['fixture_id'], 'provenance fixture ID mismatch')
    raw = copy.deepcopy(fixture)
    del raw['provenance']['fixture_revision']
    require(p['fixture_revision'] == digest(raw), 'fixture revision digest mismatch')
    require(p['rules_basis']['rules'] == rules_pin and p['rules_basis']['cards'] == cards_pin, 'provenance pin mismatch')
    require(p['rules_basis']['numbered_rules'] == fixture['rule_references'], 'rule reference mismatch')
    for ref in fixture['rule_references']:
        require(re.fullmatch(r'[0-9]{3}(?:\.[0-9]+[a-z]?)?', ref), 'malformed numbered rule')
    basis = {entry['id'] for entry in p['expected_result_basis']}
    require(len(basis) == len(p['expected_result_basis']), 'duplicate expectation basis')
    actions = fixture['script']
    ids = [a['id'] for a in actions]
    unique(ids, 'action ID')
    require('initial' not in ids, 'reserved action ID')
    for action in actions:
        validate_action(action)
    unique([c['id'] for a in actions for c in a['choices']], 'choice ID')
    cp_names = [c['name'] for c in fixture['checkpoints']]
    unique(cp_names, 'checkpoint name')
    checkpoints = set(cp_names)
    require('initial' not in checkpoints, 'reserved checkpoint name')
    require(fixture['checkpoints'][-1]['after'] == ids[-1], 'script tail has no checkpoint')
    assertions = []
    order = []
    for cp in fixture['checkpoints']:
        require(cp['after'] in ['initial'] + ids, 'unresolved checkpoint action')
        order.append((['initial'] + ids).index(cp['after']))
        assertions.extend(cp['assertions'])
    require(order == sorted(order), 'checkpoint order mismatch')
    for inv in fixture['invalid_actions']:
        require(inv['at'] in checkpoints | {'initial'}, 'unresolved invalid-action checkpoint')
        require(set(inv['invariants']) == INVARIANTS and len(inv['invariants']) == 4, 'missing invalid-action invariants')
        require(inv['basis'] in basis, 'unresolved invariant basis')
        validate_action(inv['action'])
    setup = fixture['setup']
    used_cards = set()
    if setup['kind'] == 'synthetic':
        validate_state(setup['state'], cards)
        used_cards = {o['card_id'] for o in setup['state']['objects']}
        used_cards.update(x['last_known_source']['card_id'] for x in setup['state']['stack'] + setup['state']['effects'] if x['last_known_source'] is not None)
        for assumption in setup['assumptions']:
            for assertion in assumption['checks']:
                require(pointer(setup['state'],assertion['path']) == assertion['expected'], 'synthetic assumption failed')
            assertions.extend(assumption['checks'])
    else:
        require(len(setup['decks']) == 2 and len(setup['ordered_libraries']) == 2, 'normal reset requires two decks')
        decks = {d['id']: Counter({c['card_id']:c['copies'] for c in d['cards']}) for d in cards['decks']}
        for seat in (0, 1):
            require(Counter(setup['ordered_libraries'][seat]) == decks[setup['decks'][seat]], 'normal reset deck/order mismatch')
            used_cards.update(setup['ordered_libraries'][seat])
        unique([x['id'] for x in setup['shuffle_results']], 'shuffle result ID')
        for shuffle in setup['shuffle_results']:
            require(Counter(shuffle['order']) == decks[setup['decks'][shuffle['seat']]], 'invalid mulligan shuffle result')
        # The opening reset must expose an explicit mulligan/keep decision for both seats.
        require({a['actor'] for a in actions if a['kind'] in ('keep','mulligan')} == {0,1}, 'incomplete normal-reset mulligan script')
        mulligans = Counter(a['actor'] for a in actions if a['kind'] == 'mulligan')
        require(Counter(x['seat'] for x in setup['shuffle_results']) == mulligans, 'missing/extra ordered mulligan shuffles')
        kept, bottomed, taken = set(), set(), Counter()
        for action in actions:
            if len(bottomed) == 2:
                break
            actor, kind = action['actor'], action['kind']
            require(kind in ('keep', 'mulligan', 'bottom'), 'incomplete opening choices before play')
            if kind == 'bottom':
                require(actor in kept and actor not in bottomed and
                        len(action['choices'][0]['values']) == min(taken[actor], 7), 'wrong London bottom count')
                bottomed.add(actor)
            else:
                require(actor not in kept, 'opening decision after keep')
                if kind == 'mulligan':
                    taken[actor] += 1
                else:
                    kept.add(actor)
                    if taken[actor] == 0:
                        bottomed.add(actor)
        require(len(bottomed) == 2, 'incomplete London mulligan/bottom script')
    for assertion in assertions:
        require(assertion['basis'] in basis, 'unresolved expectation basis')
        require(not re.search(r'~(?![01])', assertion['path']), 'invalid JSON pointer escape')
        value, field = assertion['expected'], assertion['field']
        if field in ('life', 'marked_damage'):
            require(type(value) is int, 'expected integer assertion')
            require(field != 'marked_damage' or value >= 0, 'negative marked damage')
        if field in ('priority', 'active_player'):
            require((field == 'priority' and value is None) or
                    (type(value) is int and value in (0, 1)), 'expected seat assertion')
        if field in ('stack', 'legal_choices'):
            require(type(value) is list, 'expected ordered array assertion')
        if field in ('zones', 'object_identity', 'power_toughness', 'mana', 'status', 'player_visible_information', 'rewards', 'outcome'):
            require(type(value) is dict, 'expected structured assertion')
    oracle = {c['card_id']: c['sha256'] for c in p['rules_basis']['oracle_sources']}
    require(len(oracle) == len(p['rules_basis']['oracle_sources']), 'duplicate Oracle source')
    card_map = {c['id']: c for c in cards['cards']}
    require(set(oracle) == used_cards, 'missing/extra Oracle pin')
    require(all(oracle[c] == card_map[c]['oracle_text_sha256'] for c in oracle), 'unresolved Oracle text pin')
    require({card_map[c]['behavior_id'] for c in used_cards} <= set(fixture['required_capabilities']), 'missing card capability')
    require(p['reproduction']['setup_kind'] == setup['kind'], 'setup provenance mismatch')
    require(p['reproduction']['trace_sha256'] == digest(actions), 'action trace digest mismatch')
    for when in (p['created_at'], p['license_review']['date'], p['vintage_audit']['date'], p['review']['date']):
        require(date.fromisoformat(when).isoformat() == when, 'invalid provenance date')
    require(p['vintage_audit']['target'] == rules_pin['version'], 'wrong target vintage')
    for origin in p['origins'] + p['license_review']['sources'] + p['rules_basis']['rulings']:
        require(origin['url'].startswith('https://'), 'origin must use exact source URL')
        date.fromisoformat(origin['retrieved_at'])
        require(origin['version'] not in ('HEAD','main','master','latest','unknown','unresolved'), 'unresolved source revision')
    if p['source_kind'] == 'original':
        require(p['no_upstream_code_adapted'] is not None and not p['ancestry'], 'original provenance needs no-adaptation declaration')
    if p['source_kind'] == 'adapted' or p['ancestry']:
        require(p['license_review']['distribution_decision'] == 'reviewed-m0-adaptation', 'unreviewed adaptation')
        require(p['license_review']['retained_notice_paths'] and p['license_review']['file_notices'], 'missing adaptation notices')
        for origin in p['origins']:
            require('card-forge' not in origin['url'].lower(), 'Forge adaptation forbidden by M0 policy')
        require(any(re.fullmatch('[0-9a-f]{40}', o['version']) for o in p['origins']), 'missing adapted source commit')
        require('Permission is hereby granted' in p['license_review']['full_notice_text'] and
                'THE SOFTWARE IS PROVIDED' in p['license_review']['full_notice_text'], 'missing full MIT notice')
    require((p['regression'] is not None) == (p['source_kind'] == 'regression'), 'missing/inapplicable regression provenance')
    require((p['differential'] is not None) == (p['source_kind'] == 'differential'), 'missing/inapplicable differential provenance')
    if p['differential'] is not None:
        require(p['differential']['first_divergent_checkpoint'] in checkpoints, 'unresolved divergence checkpoint')
        require({e['engine'] for e in p['reference_evidence']} == {'xmage','forge'}, 'missing differential engines')
    unique([e['engine'] for e in p['reference_evidence']], 'reference engine')
    for evidence in p['reference_evidence']:
        require(evidence['cards'] == cards_pin, 'reference card mismatch')
        if evidence['status'] == 'planned':
            require(not evidence['artifacts'], 'planned reference claims execution')
        else:
            require(evidence['artifacts'] and evidence['observable_fields'], 'missing reference evidence')
    if p['review']['status'] == 'accepted-for-m0':
        require(p['vintage_audit']['status'] == 'accepted' and p['review']['evidence_links'] and
                not p['review']['blocker_ids'] and p['license_review']['distribution_decision'] != 'blocked', 'incomplete admission review')


class ChoiceScript:
    """Test/bridge utility. No legality/result calculation and no fallback choices."""
    def __init__(self, script):
        self.choices = [copy.deepcopy(c) for a in script for c in a['choices']]
        self.index = 0

    def consume(self, actor, kind):
        require(self.index < len(self.choices), 'incomplete choice script: unexpected decision')
        choice = self.choices[self.index]
        require(type(actor) is int and choice['actor'] == actor and choice['kind'] == kind,
                'unexpected decision actor/kind')
        self.index += 1
        return copy.deepcopy(choice['values'])

    def finish(self):
        require(self.index == len(self.choices), 'incomplete choice script: unused choices')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=('validate','coverage'))
    parser.add_argument('fixtures', nargs='*', type=Path)
    parser.add_argument('--registry', type=Path, default=ROOT / 'data/capabilities-v1.json')
    parser.add_argument('--require-passed', action='store_true')
    args = parser.parse_args()
    try:
        registry = load(args.registry)
        fixtures = [load(path) for path in args.fixtures]
        require(args.command != 'validate' or fixtures, 'validate needs at least one fixture')
        report = validate_registry(registry, fixtures, args.require_passed)
        result = {'status':'valid', 'scenario_version':1, 'fixtures':len(fixtures)}
        if args.command == 'coverage':
            result['coverage'] = report
        print(json.dumps(result, sort_keys=True))
    except (ValueError, OSError, RecursionError) as error:
        print(json.dumps({'status':'error','code':'invalid_scenario_or_coverage','message':str(error)}))
        return 2
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
