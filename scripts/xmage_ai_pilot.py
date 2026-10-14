"""Versioned content identity and accounting for the provisional XMage pilot."""
import hashlib
import json
import copy


def canonical_game(transcript):
    """Canonical gameplay/chance, excluding every source/provenance label.

    Equal-card physical copies are named by their first ordered shuffle position,
    preserving subsequent incarnation/choice identity. Token copies are named by
    first consequential use within their recorded creation group. Fixed seats are
    never swapped. This is content identity, not a qualification verdict.
    """
    if len(transcript['cases']) != 1:
        raise ValueError('pilot identity requires one game')
    case = transcript['cases'][0]
    mapping = {}
    counts = {}

    def bind(source):
        if source not in mapping:
            parts = source.split('/')
            group = '/'.join(parts[:-1])
            ordinal = counts.get(group, 0)
            counts[group] = ordinal + 1
            mapping[source] = f'{group}/{ordinal}'
        return mapping[source]

    for shuffle in case['chance']:
        for source in shuffle['after']:
            bind(source)

    def project(value):
        if isinstance(value, list):
            return [project(v) for v in value]
        if isinstance(value, dict):
            return {k:project(value[k]) for k in sorted(value)}
        if isinstance(value, str) and (value in mapping or value.startswith('token/')):
            return bind(value)
        return value

    gameplay = project({k:case[k] for k in ('starter','decks','chance','choices','play')})
    # These are construction multisets, not observed random outcomes. Preserve
    # the ordered shuffle *after* arrays and every subsequent consequential use.
    for deck in gameplay['decks']:
        deck['occurrences'].sort()
    for shuffle in gameplay['chance']:
        shuffle['before'].sort()
    definitions = {name:transcript['pins'][name] for name in (
        'data/cards/foundations_micro_v1.json','data/rules/cr-2026-09-25.json')}
    value = dict(canonical_version=1, transcript_schema=transcript['schema_version'],
                 definitions=definitions, gameplay=gameplay)
    return json.dumps(value,sort_keys=True,separators=(',',':'),ensure_ascii=True).encode()


def identity(transcript):
    return hashlib.sha256(canonical_game(transcript)).hexdigest()


def distinct_attempts(attempts):
    """Retain all attempts; first origin owns an identity, repeats never count."""
    seen = {}
    result = []
    for attempt in attempts:
        canonical = canonical_game(attempt['transcript'])
        digest = hashlib.sha256(canonical).hexdigest()
        previous = seen.get(digest)
        if previous is not None and previous[1] != canonical:
            raise ValueError('canonical hash collision')
        row = dict(attempt_id=attempt['attempt_id'], identity=digest,
                   duplicate_of=previous[0] if previous else None)
        if previous is None:
            seen[digest] = (attempt['attempt_id'],canonical)
        result.append(row)
    return result


def source_negative_inputs(converted):
    """Illegal controls tied to this tape's choices, rather than fixed cursors."""
    case=converted['transcript']['cases'][0]
    result={}

    def add(name,index,edit):
        bad=copy.deepcopy(case);edit(bad)
        result['pilot_'+name]=dict(case=bad,sequence=index)

    add('shuffle',None,lambda c:c['chance'][0]['after'].__setitem__(0,c['chance'][0]['after'][1]))
    add('unexpected_trigger_order',0,lambda c:c['play'][0].update(kind='order_triggers',order=[]))
    land=next(e for e in case['play'] if e['kind']=='play_land')
    point=next(c['state'] for c in converted['source_checkpoints'] if c['action']==land['sequence'])
    copies=[source for source in case['decks'][land['actor']]['occurrences']
            if source.split('/')[1]==land['source'].split('/')[1]
            and source not in point['hand_membership'][land['actor']]]
    if not copies:
        raise ValueError('no unheld duplicate for physical identity control')
    add('duplicate_card_identity',land['sequence'],lambda c:c['play'][land['sequence']].update(source=copies[0]))
    payment=next(e for e in case['play'] if e['kind']=='pay')
    # Frozen red/green decks produce no colorless mana (CR106/601.2h).
    add('cost',payment['sequence'],lambda c:c['play'][payment['sequence']].update(color=5))
    target=next((e for e in case['play'] if e['kind'] in ('target','activation_target')),None)
    if target:
        # Every supported targeted spell/activation targets a creature, never a land.
        add('target',target['sequence'],lambda c:c['play'][target['sequence']].update(
            source=land['source'],incarnation=land['incarnation']))
    order=next((e for e in case['play'] if e['kind']=='order_triggers' and len(e['order'])>1),None)
    if order:
        add('genuine_trigger_order',order['sequence'],lambda c:c['play'][order['sequence']]['order'].__setitem__(1,copy.deepcopy(order['order'][0])))
    grave=next((e for e in case['play'] if e['kind']=='graveyard_order'),None)
    if grave:
        add('graveyard_order',grave['sequence'],lambda c:c['play'][grave['sequence']]['selection'].__setitem__(1,copy.deepcopy(grave['selection'][0])))
    return result
