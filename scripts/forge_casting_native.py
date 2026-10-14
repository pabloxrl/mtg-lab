"""Translate committed Forge prefix choices for an independent native execution.

The source states are observations, never inputs to the native rules state.
Physical identity is preserved; hand and battlefield order are not rules order.
"""
import forge_opening as opening

STEPS = dict(UPKEEP='upkeep', DRAW='draw', MAIN1='precombat_main',
             COMBAT_BEGIN='begin_combat', COMBAT_DECLARE_ATTACKERS='declare_attackers',
             COMBAT_DECLARE_BLOCKERS='declare_blockers',
             COMBAT_DAMAGE='combat_damage', CLEANUP='cleanup',
             COMBAT_END='end_combat', MAIN2='postcombat_main', END_OF_TURN='end_turn')
COLORS = {1: 0, 2: 1, 4: 2, 8: 3, 16: 4, 0: 5}


def cleanup_selection(row, selected):
    """CR 514.1: explicit distinct hand cards, exactly the excess above seven."""
    state = row['state']
    actor = row['actor']['player_id']
    hand = state['players'][actor]['zones']['Hand']
    count = row['arguments'][0]
    opening.need(state['phase'] == 'CLEANUP' and state['active']['player_id'] == actor and
                 count == len(hand)-7 and count > 0, row['index'], 'cleanup discard context/count')
    keys = [(c['id'], c['timestamp']) for c in selected]
    available = {(c['id'], c['timestamp']) for c in hand}
    opening.need(len(keys) == count and len(set(keys)) == count and set(keys) <= available,
                 row['index'], 'cleanup discard occurrences')
    return selected


def fixture(converted, opening_native):
    full_game = converted.get('scope') == 'full-game'
    abilities = full_game or converted.get('scope') == 'abilities-prefix'
    opening.need(abilities or converted['endpoint']['method'] not in
                 ('orderAndPlaySimultaneousSa', 'orderSimultaneousSa', 'playTrigger'),
                 converted['endpoint']['index'], 'unsupported pending triggers')
    bindings = {int(k): v for k, v in converted['bindings'].items()}
    rows = converted['prefix']
    paired = converted['paired']
    ops, zones, incarnations = [], {}, {v: 0 for v in bindings.values()}
    action_count = 0
    auxiliary = dict(marked=[], cleanup_checkpoints=[])
    marked_turns = set()
    creation_count = 0
    declared = set()
    blocked = set()
    blocked_creatures = set()
    trigger_keys = {}
    activation_keys = {}
    witnessed_incarnations = {}
    trigger_event = 0
    in_trigger_batch = False

    def identity(card):
        return bindings[card['id']]

    def ref(card):
        if abilities:
            key = (card['id'], card['timestamp'])
            opening.need(key in witnessed_incarnations, card['id'], 'unwitnessed source incarnation')
            incarnation = witnessed_incarnations[key]
        else:
            incarnation = incarnations[identity(card)]
        return dict(source=identity(card), incarnation=incarnation)

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
        if abilities:
            value.update(order=None, player=None)
        if card is not None:
            value.update(ref(card))
        value.update(kwargs)
        ops.append(dict(source_cursor=row['index'], action=value))
        action_count += 1

    def track(row):
        nonlocal creation_count
        s = row['state']
        cards = [c for p in s['players'] for cs in p['zones'].values() for c in cs]
        cards += [a['host'] for a in s['stack'] if a['spell']]
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
            if abilities:
                witnessed_incarnations[(card['id'], card['timestamp'])] = incarnations[name]
        return cards

    def point(row, auxiliary_kind=None):
        s = row['state']
        opening.need(not s['pending_triggers'] or (auxiliary_kind and all(not g['triggers'] for g in s['pending_triggers'])), row['index'], 'unsupported pending triggers')
        cards = track(row)
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
            for change in card['keyword_changes']:
                grant = change[2]
                opening.need(abilities and len(change) == 3 and
                             set(grant) == {'added', 'removed', 'remove_all'} and
                             grant['added'] and set(grant['added']) <= {'Haste', 'Trample'} and
                             not grant['removed'] and not grant['remove_all'],
                             row['index'], 'unsupported keyword grant')
        mana = [[0] * 6 for _ in ps]
        for i, p in enumerate(ps):
            for lot in p['mana']:
                mana[i][COLORS[lot['color']]] += 1
        stack = []
        for ability in reversed(s['stack']):
            if abilities and ability['trigger']:
                opening.need(ability['id'] in trigger_keys, row['index'], 'unwitnessed trigger')
                targets = ability['targets']
                opening.need(len(targets) <= 1 and all('player_id' in t['target'] for t in targets),
                             row['index'], 'unsupported trigger target')
                stack.append(dict(ability='trigger', key=trigger_keys[ability['id']],
                                  controller=ability['actor']['player_id'],
                                  target=targets[0]['target']['player_id'] if targets else None,
                                  source_card=identity(ability['host']).split('/')[1]))
                continue
            if abilities and not ability['spell']:
                kinds = {'Shivan Dragon': 'power', 'Wildheart Invoker': 'invoker',
                         'Axgard Cavalry': 'haste'}
                opening.need(ability['host']['name'] in kinds and not ability['mana_ability'],
                             row['index'], 'unsupported stack activation')
                targets = ability['targets']
                opening.need(len(targets) <= 1, row['index'], 'activation target count')
                opening.need(ability['id'] in activation_keys, row['index'], 'unwitnessed activation')
                retained = activation_keys[ability['id']]
                opening.need(identity(ability['host']) == retained['source'], row['index'],
                             'activation source identity')
                stack.append(dict(retained, ability=kinds[ability['host']['name']]))
                continue
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
                        actor=None if s['terminal'] or auxiliary_kind else row['actor']['player_id'], life=[p['life'] for p in ps], mana=mana,
                        library=[[identity(c) for c in p['zones']['Library']] for p in ps],
                        graveyard=[[identity(c) for c in p['zones']['Graveyard']] for p in ps],
                        hand=[sorted(identity(c) for c in p['zones']['Hand']) for p in ps],
                        exile=sorted(identity(c) for p in ps for c in p['zones']['Exile']),
                        battlefield=sorted(per), permanents=per, incarnations={identity(c): incarnations[identity(c)] for c in cards},
                        stack=stack, effects=effects)
        if full_game:
            expected['combat'] = [dict(attacker=ref(g['attacker']), blocked=g['attacker']['id'] in blocked_creatures,
                blockers=[ref(b) for b in g['blockers']]) for g in (s['combat'] or [])]
            losses = []
            for player in ps:
                outcome = player['outcome']
                loss = outcome['loss'] if outcome else None
                opening.need(loss in (None, 'LifeReachedZero', 'Milled'), row['index'], 'unsupported terminal reason')
                losses.append({'LifeReachedZero': 'Life', 'Milled': 'EmptyDraw'}.get(loss))
            expected['outcome'] = dict(winner=next((i for i,p in enumerate(ps) if p['outcome'] and p['outcome']['won']), None), losses=losses) if s['terminal'] else None
        (auxiliary[auxiliary_kind] if auxiliary_kind else ops).append(dict(source_cursor=row['index'], checkpoint=expected))

    def payments(row, committed, activation=False):
        cursor = row['index']
        sources = committed['paying_mana_abilities']
        lots = committed['paying_mana']
        opening.need(len(sources) == len(lots), cursor,
                     'unsupported floating or multi-unit mana payment')
        for source, lot in zip(sources, lots, strict=True):
            opening.need(source['id'] == lot['source_ability'] and
                         source['host']['id'] == lot['source']['id'] and
                         source['host']['name'] in (('Forest', 'Mountain', 'Llanowar Elves', 'Druid of the Cowl')
                                                  if abilities else ('Forest', 'Mountain')) and
                         not lot['restricted'] and not lot['restrictions'] and
                         not lot['extra_restrictions'], cursor,
                         'unsupported mana source/payment order')
            action(row, 'tap_mana', source['host'])
            action(row, 'activation_pay' if activation else 'pay', color=COLORS[lot['color']])

    cursor = converted['checkpoint_sources'][-1]
    while cursor < len(rows):
        row = rows[cursor]
        if row['kind'] != 'enter':
            if full_game and row['kind'] == 'boundary':
                state = row['state']
                if row['boundary'] == 'sba-before' and state['phase'] == 'COMBAT_DAMAGE' and state['turn'] not in marked_turns:
                    marked_turns.add(state['turn'])
                    point(row, 'marked')
                elif row['boundary'] == 'sba-after' and state['phase'] == 'CLEANUP':
                    point(row, 'cleanup_checkpoints')
            cursor += 1
            continue
        method = row['method']
        end = paired[row['index']]
        leave = rows[end]
        if method == 'chooseSpellAbilityToPlay':
            in_trigger_batch = False
            state = row['state']
            if state['phase'] == 'COMBAT_DECLARE_ATTACKERS' and state['turn'] not in declared:
                action(dict(row, actor=state['active']), 'empty_attackers')
                declared.add(state['turn'])
            if full_game and state['phase'] == 'COMBAT_DECLARE_BLOCKERS' and state['turn'] not in blocked:
                opening.need(all(not g['blockers'] for g in state['combat']), cursor, 'missing blocker callback')
                opening.need(any(r.get('event') == 'GameEventBlockersDeclared' and r['state']['turn'] == state['turn'] for r in rows[:cursor]), cursor, 'missing empty blocker declaration event')
                action(dict(row, actor=dict(player_id=1-state['active']['player_id'])), 'blockers', selection=[])
                blocked.add(state['turn'])
            point(row)
            if leave['result'] is None:
                action(row, 'pass')
        elif method == 'playChosenSpellAbility':
            ability = row['arguments'][0]
            host = ability['host']
            if 'Land' in host['types']:
                action(row, 'play_land', host)
            elif abilities and not ability['spell']:
                committed = leave['arguments'][0]
                if ability['mana_ability']:
                    opening.need(host['name'] in ('Llanowar Elves', 'Druid of the Cowl'),
                                 cursor, 'unsupported standalone mana source')
                    action(row, 'tap_mana', host)
                else:
                    opening.need(host['name'] in ('Shivan Dragon', 'Wildheart Invoker', 'Axgard Cavalry'),
                                 cursor, 'unsupported activation definition')
                    action(row, 'activate', host)
                    before_ids = {a['id'] for a in row['state']['stack']}
                    added = [a for a in leave['state']['stack'] if a['id'] not in before_ids]
                    opening.need(len(added) == 1 and not added[0]['spell'] and
                                 not added[0]['trigger'] and added[0]['host']['id'] == host['id'],
                                 cursor, 'committed activation stack identity')
                    # Forge clears the reusable activation's targets after making
                    # its stack copy. The committed stack entry retains the choice.
                    targets = added[0]['targets']
                    opening.need(len(targets) == (0 if host['name'] == 'Shivan Dragon' else 1),
                                 cursor, 'activation target count')
                    opening.need([ref(t['target']) for t in targets] ==
                                 [ref(t['target']) for t in ability['targets']],
                                 cursor, 'committed activation targets')
                    for target in targets:
                        action(row, 'activation_target', target['target'])
                    activation_keys[added[0]['id']] = dict(
                        **ref(host), target=ref(targets[0]['target']) if targets else None)
                    payments(row, committed, activation=True)
                    action(row, 'finish_activation')
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
                payments(row, committed)
                action(row, 'finish_payment')
        elif abilities and method == 'orderAndPlaySimultaneousSa':
            track(row)
            # CR 603.3b: each controller chooses their own bottom-to-top order.
            # The real engine determines which APNAP controller acts next.
            orders = [rows[paired[r['index']]]['result'] for r in rows[cursor:end]
                      if r.get('method') == 'orderSimultaneousSa']
            opening.need(len(orders) == 1, cursor, 'one explicit trigger order')
            requested = orders[0]
            available = row['arguments'][0]
            opening.need(sorted(a['id'] for a in requested) == sorted(a['id'] for a in available)
                         and len({a['id'] for a in requested}) == len(requested),
                         cursor, 'trigger order permutation')
            if not in_trigger_batch:
                event = trigger_event
                trigger_event += 1
                in_trigger_batch = True
            kinds = {'Firebrand Archer': 'archer', 'Crackling Cyclops': 'cyclops',
                     'Viashino Pyromancer': 'pyromancer'}
            keys = []
            for a in requested:
                opening.need(a['trigger'] and a['host']['name'] in kinds,
                             cursor, 'unsupported trigger definition')
                key = dict(**ref(a['host']), ability=kinds[a['host']['name']], event=event)
                trigger_keys[a['id']] = key
                keys.append(key)
            action(row, 'order_triggers', order=keys)
            preparation = [r['arguments'][1] for r in rows[cursor:end]
                           if r.get('method') == 'cost.visit' and r['arguments'][1]['trigger']]
            opening.need(sorted(a['id'] for a in preparation) == sorted(a['id'] for a in requested),
                         cursor, 'exact prepared trigger choices')
            prepared = {a['id']: a for a in preparation}
            for requested_ability in requested:
                opening.need(requested_ability['id'] in prepared, cursor, 'prepared trigger choice')
                a = prepared[requested_ability['id']]
                if a['targets']:
                    opening.need(len(a['targets']) == 1 and 'player_id' in a['targets'][0]['target'],
                                 cursor, 'unsupported trigger target')
                    action(row, 'target_player', player=a['targets'][0]['target']['player_id'])
        elif abilities and method == 'playSpellAbilityNoStack':
            # Wrapped trigger resolution is already caused by the recorded
            # consecutive priority passes; it creates no extra native action.
            opening.need(row['arguments'][0]['trigger'], cursor, 'unsupported no-stack effect')
        elif method == 'declareAttackers':
            blocked_creatures.clear()
            track(row)
            selected = leave['state']['combat']
            opening.need(full_game or not selected, cursor, 'unsupported nonempty combat')
            if selected:
                action(row, 'attackers', selection=[ref(g['attacker']) for g in selected])
            else:
                action(row, 'empty_attackers')
            declared.add(row['state']['turn'])
        elif full_game and method == 'declareBlockers':
            blocked.add(row['state']['turn'])
            blocked_creatures.update(g['attacker']['id'] for g in leave['state']['combat'] if g['blockers'])
            track(row)
            action(row, 'blockers', selection=[dict(blocker=ref(b), attacker=ref(g['attacker']))
                for g in leave['state']['combat'] for b in g['blockers']])
        elif full_game and method == 'assignCombatDamage':
            track(row)
            source, blockers, _, power, defender, _ = row['arguments']
            allocation = leave['result']
            opening.need(sum(amount for _,amount in allocation) == power and all(amount >= 0 for _,amount in allocation), cursor, 'exact damage total')
            opening.need(len({c['id'] if c else None for c,_ in allocation}) == len(allocation), cursor, 'duplicate damage target')
            opening.need(all(c is None or c['id'] in {b['id'] for b in blockers} for c,_ in allocation), cursor, 'damage target')
            groups = [g for g in row['state']['combat'] if g['attacker']['id'] == source['id']]
            if groups and (len(blockers) > 1 or 'Trample' in source['keywords']):
                # Forge returns a map; current CR 510 has no damage-assignment
                # order. Bind each amount to its physical recipient and emit
                # stable slots in the recorded blocker-list order for XMage.
                by_target = {c['id']: n for c,n in allocation if c}
                action(row, 'damage', selection=[dict(attacker=ref(source), amounts=[
                    dict(target=ref(c), amount=by_target.get(c['id'], 0)) for c in blockers])])
            else:
                # Single-recipient nontrample assignments are forced by CR 510.1.
                # Retain and check the recorded allocation, without inventing a
                # native decision for a forced rules transition.
                opening.need(len(blockers)==1 and allocation == [[blockers[0], power]], cursor, 'forced combat allocation')
        elif full_game and method == 'orderMoveToZoneList':
            track(row)
            opening.need(row['arguments'][1] == 'Graveyard', cursor, 'unsupported zone order')
            opening.need(sorted(c['id'] for c in row['arguments'][0]) == sorted(c['id'] for c in leave['result']), cursor, 'graveyard permutation')
            # CR 111.6/404.3: tokens share the move event but never occupy
            # positions in the ordered graveyard. The native choice contains
            # only persistent cards; a singleton has no ordering decision.
            cards = [c for c in leave['result'] if not c['token']]
            if len(cards) > 1:
                action(row, 'graveyard_order', selection=[ref(c) for c in cards], event='combat-sba/' + str(row['state']['turn']))
        elif full_game and method == 'chooseCardsToDiscardToMaximumHandSize':
            track(row)
            selected = cleanup_selection(row, leave['result'])
            action(row, 'cleanup_discard', selection=[ref(c) for c in selected])
        elif method not in ('resetAtEndOfTurn', 'autoPassCancel'):
            opening.need(False, cursor, 'unsupported native callback ' + method)
        cursor = end + 1
    endpoint = converted['endpoint']
    if full_game:
        endpoint = dict(endpoint, actor=endpoint['state']['active'])
        action(endpoint, 'finish')
    point(endpoint)
    result = dict(schema_version=3 if full_game else 2 if abilities else 1, complete_game=full_game, source_sha256=converted['source_sha256'],
                  opening=opening_native, operations=ops, actions=action_count,
                  endpoint=converted['endpoint']['index'], remainder=converted['remainder'])
    if full_game:
        # Absolute semantic incarnations start in the initial library, before
        # London choices. Count witnessed zone transitions, not source timestamps.
        initial_zones, initial_incarnations = {}, {}
        for row in rows[:converted['checkpoint_sources'][-1] + 1]:
            for player in row.get('state', {}).get('players', []):
                for cards in player['zones'].values():
                    for card in cards:
                        uid = identity(card)
                        initial_incarnations.setdefault(uid, 0)
                        if uid in initial_zones and initial_zones[uid] != card['zone']:
                            initial_incarnations[uid] += 1
                        initial_zones[uid] = card['zone']
        result['initial_incarnations'] = initial_incarnations
        result['auxiliary'] = auxiliary
    return result
