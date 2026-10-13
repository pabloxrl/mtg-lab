"""Translate committed Forge prefix choices for an independent native execution.

The source states are observations, never inputs to the native rules state.
Physical identity is preserved; hand and battlefield order are not rules order.
"""
import forge_opening as opening

STEPS = dict(UPKEEP='upkeep', DRAW='draw', MAIN1='precombat_main',
             COMBAT_BEGIN='begin_combat', COMBAT_DECLARE_ATTACKERS='declare_attackers',
             COMBAT_END='end_combat', MAIN2='postcombat_main', END_OF_TURN='end_turn')
COLORS = {1: 0, 2: 1, 4: 2, 8: 3, 16: 4, 0: 5}


def fixture(converted, opening_native):
    opening.need(converted['endpoint']['method'] not in
                 ('orderAndPlaySimultaneousSa', 'orderSimultaneousSa', 'playTrigger'),
                 converted['endpoint']['index'], 'unsupported pending triggers')
    bindings = {int(k): v for k, v in converted['bindings'].items()}
    rows = converted['prefix']
    paired = converted['paired']
    ops, zones, incarnations = [], {}, {v: 0 for v in bindings.values()}
    action_count = 0
    creation_count = 0
    declared = set()

    def identity(card):
        return bindings[card['id']]

    def ref(card):
        return dict(source=identity(card), incarnation=incarnations[identity(card)])

    def mode(ability, cursor):
        if ability['api'] != 'Charm':
            return None
        opening.need(ability['host']['name'] == 'Goblin Surprise' and ability['sub'] is not None,
                     cursor, 'unsupported modal spell')
        api = ability['sub']['api']
        opening.need(api in ('PumpAll', 'Token'), cursor, 'unsupported mode')
        return 0 if api == 'PumpAll' else 1

    def action(row, kind, card=None, **kwargs):
        nonlocal action_count
        state = row['state']
        value = dict(sequence=action_count, turn=state['turn'], step=STEPS[state['phase']],
                     actor=row['actor']['player_id'], kind=kind, source=None,
                     incarnation=None, color=None, role=None, mode=None)
        if card is not None:
            value.update(ref(card))
        value.update(kwargs)
        ops.append(dict(source_cursor=row['index'], action=value))
        action_count += 1

    def point(row):
        nonlocal creation_count
        s = row['state']
        opening.need(not s['pending_triggers'], row['index'], 'unsupported pending triggers')
        cards = [c for p in s['players'] for cs in p['zones'].values() for c in cs]
        cards += [a['host'] for a in s['stack']]
        born = {c['id']: c for c in cards if c['id'] not in bindings}
        for ordinal, card in enumerate(sorted(born.values(), key=lambda c: c['id'])):
            opening.need(card['token'] and card['name'] == 'Goblin Token' and card['zone'] == 'Battlefield',
                         row['index'], 'unsupported created object')
            name = f"token/{card['controller']['player_id']}/{creation_count}/{ordinal}"
            bindings[card['id']] = name
            incarnations[name] = 0
        if born:
            creation_count += 1
        for card in cards:
            name = identity(card)
            zone = card['zone']
            if name in zones and zones[name] != zone:
                incarnations[name] += 1
            zones[name] = zone
        ps = s['players']
        per, effects = {}, {}
        for card in cards:
            if card['zone'] != 'Battlefield':
                continue
            creature = 'Creature' in card['types']
            per[identity(card)] = dict(card='goblin-token' if card['token'] else identity(card).split('/')[1],
                owner=card['owner']['player_id'], controller=card['controller']['player_id'],
                tapped=card['tapped'], sick=card['sick'] if creature else False,
                power=card['power'] if creature else None, toughness=card['toughness'] if creature else None,
                damage=card['damage'] if creature else None)
            for keyword in ('flying', 'reach', 'vigilance', 'deathtouch', 'haste', 'trample'):
                per[identity(card)][keyword] = any(k.lower() == keyword for k in card['keywords'])
            if card['pt_boosts']:
                amounts = [sum(cell[2][i] for cell in card['pt_boosts']) for i in (0, 1)]
                if amounts != [0, 0]:
                    effects[identity(card)] = amounts
            opening.need(not card['keyword_changes'], row['index'], 'unsupported keyword grant')
        mana = [[0] * 6 for _ in ps]
        for i, p in enumerate(ps):
            for lot in p['mana']:
                mana[i][COLORS[lot['color']]] += 1
        stack = []
        for ability in reversed(s['stack']):
            opening.need(ability['spell'], row['index'], 'unsupported stack ability')
            targets = []
            api = ability['api']
            roles = ['growth_target'] if api == 'Pump' and ability['sub'] is None else ['bite_source', 'bite_destination'] if api == 'Pump' and ability['sub']['api'] == 'DealDamage' else []
            selected_targets = ability['targets'] + (ability['sub']['targets'] if ability['sub'] is not None else [])
            for i, target in enumerate(selected_targets):
                opening.need(i < len(roles), row['index'], 'unsupported target role')
                targets.append(dict(role=roles[i], object=ref(target['target'])))
            stack.append(dict(**ref(ability['host']), targets=targets, mode=mode(ability, row['index'])))
        expected = dict(turn=s['turn'], step=STEPS[s['phase']], active=s['active']['player_id'],
                        actor=row['actor']['player_id'], life=[p['life'] for p in ps], mana=mana,
                        library=[[identity(c) for c in p['zones']['Library']] for p in ps],
                        graveyard=[[identity(c) for c in p['zones']['Graveyard']] for p in ps],
                        hand=[sorted(identity(c) for c in p['zones']['Hand']) for p in ps],
                        exile=sorted(identity(c) for p in ps for c in p['zones']['Exile']),
                        battlefield=sorted(per), permanents=per, incarnations=dict(incarnations),
                        stack=stack, effects=effects)
        ops.append(dict(source_cursor=row['index'], checkpoint=expected))

    cursor = converted['checkpoint_sources'][-1]
    while cursor < len(rows):
        row = rows[cursor]
        if row['kind'] != 'enter':
            cursor += 1
            continue
        method = row['method']
        end = paired[row['index']]
        leave = rows[end]
        if method == 'chooseSpellAbilityToPlay':
            state = row['state']
            if state['phase'] == 'COMBAT_DECLARE_ATTACKERS' and state['turn'] not in declared:
                action(dict(row, actor=state['active']), 'empty_attackers')
                declared.add(state['turn'])
            point(row)
            if leave['result'] is None:
                action(row, 'pass')
        elif method == 'playChosenSpellAbility':
            ability = row['arguments'][0]
            host = ability['host']
            if 'Land' in host['types']:
                action(row, 'play_land', host)
            else:
                opening.need(ability['spell'], cursor, 'unsupported activation')
                action(row, 'cast', host)
                committed = leave['arguments'][0]
                if ability['api'] == 'Charm':
                    selected = [rows[paired[r['index']]]['result'] for r in rows[cursor:end]
                                if r.get('method') == 'chooseModeForAbility']
                    opening.need(len(selected) == 1 and len(selected[0]) == 1 and
                                 selected[0][0]['api'] == committed['sub']['api'], cursor, 'committed mode choice')
                    action(row, 'mode', mode=mode(committed, cursor))
                if ability['api'] == 'Draw':
                    opening.need(host['name'] == 'Thrill of Possibility', cursor, 'unsupported additional cost')
                    selected = [rows[paired[r['index']]]['result']['cards'] for r in rows[cursor:end]
                                if r.get('method') == 'cost.visit' and
                                r['arguments'][0].get('class') == 'forge.game.cost.CostDiscard']
                    opening.need(len(selected) == 1 and len(selected[0]) == 1, cursor, 'one additional discard')
                    paid = [item[2] for item in committed['paid'] if item[:2] == ['Discarded', True]]
                    opening.need(len(paid) == 1 and [c['id'] for c in paid[0]] == [selected[0][0]['id']],
                                 cursor, 'committed discard choice')
                    action(row, 'discard', selected[0][0])
                if ability['targets']:
                    roles = ['growth_target'] if ability['sub'] is None else ['bite_source', 'bite_destination']
                    targets = ability['targets'] + (ability['sub']['targets'] if ability['sub'] is not None else [])
                    opening.need(ability['api'] == 'Pump' and len(targets) == len(roles), cursor, 'unsupported targets')
                    for role, target in zip(roles, targets, strict=True):
                        action(row, 'target', target['target'], role=role)
                    action(row, 'finish_targets')
                else:
                    opening.need(ability['api'] in ('PermanentCreature', 'Charm', 'Draw', 'Token'), cursor, 'unsupported spell')
                sources = committed['paying_mana_abilities']
                lots = committed['paying_mana']
                opening.need(len(sources) == len(lots), cursor,
                             'unsupported floating or multi-unit mana payment')
                for source, lot in zip(sources, lots, strict=True):
                    opening.need(source['id'] == lot['source_ability'] and
                                 source['host']['id'] == lot['source']['id'] and
                                 source['host']['name'] in ('Forest', 'Mountain') and
                                 not lot['restricted'] and not lot['restrictions'] and
                                 not lot['extra_restrictions'], cursor,
                                 'unsupported mana source/payment order')
                    action(row, 'tap_mana', source['host'])
                    action(row, 'pay', color=COLORS[lot['color']])
                action(row, 'finish_payment')
        elif method == 'declareAttackers':
            opening.need(not leave['state']['combat'], cursor, 'unsupported nonempty combat')
            action(row, 'empty_attackers')
            declared.add(row['state']['turn'])
        elif method not in ('resetAtEndOfTurn', 'autoPassCancel'):
            opening.need(False, cursor, 'unsupported native callback ' + method)
        cursor = end + 1
    point(converted['endpoint'])
    return dict(schema_version=1, complete_game=False, source_sha256=converted['source_sha256'],
                opening=opening_native, operations=ops, actions=action_count,
                endpoint=converted['endpoint']['index'], remainder=converted['remainder'])
