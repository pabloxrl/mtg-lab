"""Translate immutable source choices into the delivered strict replay interface.

Source observations are candidate evidence, not rules goldens. The original
compressed tape is never modified. Every translated action retains source cursors.
"""
import json
import lzma
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
COLORS = ('white', 'blue', 'black', 'red', 'green', 'colorless')


def need(ok, cursor, field):
    if not ok:
        raise ValueError(f'first divergence: /source/{cursor}/{field}')


def loss_reason(loss, cursor):
    need(loss['life_loss'] != loss['empty_draw'],cursor,'unsupported terminal reason')
    return 'Life' if loss['life_loss'] else 'EmptyDraw'


def validate_provenance(path, receipt):
    import xmage
    need(receipt['status']=='complete' and receipt['reference_variant']=='unmodified-upstream-raw',0,'source completion/variant')
    need(receipt['artifacts'].get(path.name)==xmage.sha(path),0,'source tape hash')
    required={'references/xmage/RawAiCaptureTest.java','scripts/xmage_ai_capture.py','scripts/xmage.py',
              'data/cards/foundations_micro_v1.json','data/rules/cr-2026-09-25.json',
              'references/xmage/pins.json','references/xmage/dependencies.json','references/xmage/raw-capture-provenance.json'}
    need(set(receipt['pins'])==required,0,'source pin inventory')
    for name,sha in receipt['pins'].items():
        need(xmage.sha(ROOT/name)==sha,0,'source pin '+name)
    upstream=json.loads((ROOT/'references/xmage/raw-capture-provenance.json').read_text())
    need(receipt['toolchain']['upstream_commit']==upstream['upstream_commit'],0,'source upstream commit')
    source_names={'Mage.Server.Plugins/Mage.Player.AI/src/mage/player/ai/ComputerPlayer.java',
                  'Mage.Server.Plugins/Mage.Player.AI.MAD/src/mage/player/ai/ComputerPlayer6.java',
                  'Mage.Server.Plugins/Mage.Player.AI.MAD/src/mage/player/ai/ComputerPlayer7.java',
                  'Mage/src/main/java/mage/util/RandomUtil.java','Mage/src/main/java/mage/game/GameImpl.java',
                  'Mage/src/main/java/mage/players/PlayerImpl.java'}
    need(set(receipt['upstream_sources'])==source_names,0,'source upstream inventory')
    for name,sha in receipt['upstream_sources'].items():
        need(upstream['consulted_sources'].get(name)==sha,0,'source upstream pin '+name)


def records(path):
    """Stream source states; discard only redundant printed ability catalogs."""
    zones = {}
    incarnations = {}
    with lzma.open(path, 'rt') as stream:
        for line in stream:
            r = json.loads(line)
            s = r['state']
            for card in s['cards']:
                uid, zone = card['id'], card['zone']
                if uid not in zones:
                    incarnations[uid] = 0
                elif zones[uid] != zone:
                    incarnations[uid] += 1
                zones[uid] = zone
            r['incarnations'] = dict(incarnations)
            s['cards'] = [{k: c[k] for k in ('id', 'physical', 'zone', 'zcc')}
                          for c in s['cards']]
            yield r


def normalize(path):
    import xmage_ai_capture
    import xmage
    receipt=json.loads(path.with_name('attempt.json').read_text())
    validate_provenance(path,receipt)
    xmage_ai_capture.validate_trace(path)
    rows = list(records(path))
    first = rows[0]
    need(first['kind'] == 'configuration', 0, 'configuration')
    config = first['detail']['config']
    physical = {x['key']: x['value'] for x in first['detail']['physical']}
    seats = {p['id']: i for i, p in enumerate(first['state']['players'])}
    manifest = json.loads((ROOT / 'data/cards/foundations_micro_v1.json').read_text())
    decks = []
    for seat, name in enumerate(config['decks']):
        d = next(d for d in manifest['decks'] if d['id'] == name)
        decks.append(dict(deck=name, occurrences=[f'{seat}/{c["card_id"]}/{i}'
                     for c in d['cards'] for i in range(c['copies'])]))
    need(set(physical.values()) == {v for d in decks for v in d['occurrences']}, 0, 'physical')
    # Pair nested calls by their original chronology, never by method-name search.
    calls, paired = [], {}
    for r in rows:
        k = r['kind']
        if k.startswith('call/'):
            need(k[5:] in {'priority','pass','activateAbility','playMana','cast','playLand',
                          'selectAttackers','selectBlockers','getMultiAmountWithIndividualConstraints',
                          'shuffleLibrary','chooseMulligan','choose','chooseTarget'},
                 r['sequence'], 'unsupported callback '+k)
            calls.append(r)
        elif k.startswith('return/'):
            need(bool(calls) and calls[-1]['kind'][5:] == k[7:], r['sequence'], 'callback nesting')
            paired[calls.pop()['sequence']] = r
    need(not calls, len(rows), 'unconsumed callbacks')
    token_event = 0
    for r in rows:
        born = [p for p in r['state']['permanents'] if p['token'] and p['id'] not in physical]
        for ordinal,p in enumerate(born):
            need(p['name']=='Goblin Token', r['sequence'], 'unsupported token')
            physical[p['id']] = f"token/{seats[p['controller']]}/{token_event}/{ordinal}"
        if born:
            token_event += 1
        for p in r['state']['permanents']:
            if p['token']:
                r['incarnations'][p['id']] = 0
    play, mappings, chance, choices = [], [], [], []
    rounds = [0, 0]
    action_sources = {}

    def ref(uid, r):
        need(uid in physical, r['sequence'], 'unbound physical identity')
        return dict(source=physical[uid], incarnation=r['incarnations'][uid])

    def target_ref(target, r):
        need(len(target['targets'])==1, r['sequence'], 'target cardinality')
        uid = target['targets'][0]['id']
        counters = {x['key']:x['value'] for x in target['zone_change_counters']}
        current = next((c for c in r['state']['cards']+r['state']['permanents'] if c['id']==uid), None)
        need(current is not None and uid in counters, r['sequence'], 'target incarnation provenance')
        result = ref(uid,r)
        result['incarnation'] += counters[uid] - current['zcc']
        return result

    def observe(r, actor):
        s=r['state']
        # Settled source priority/announcement boundaries are comparable;
        # transaction micro-callbacks have a separately retained cursor map.
        checkpoint = dict(turn=s['turn'], step=s['step'].lower().replace(' ', '_'), active=seats[s['active']], actor=actor,
                          life=[p['life'] for p in s['players']],
                          mana=[[p['mana'][c] for c in COLORS] for p in s['players']],
                          library=[[physical[v] for v in p['library']] for p in s['players']],
                          graveyard=[[physical[v] for v in p['graveyard']] for p in s['players']],
                          hand_membership=[sorted(physical[v] for v in p['hand']) for p in s['players']],
                          battlefield_membership=sorted(physical[p['id']] for p in s['permanents']))
        checkpoint['incarnations'] = {physical[c['id']]:r['incarnations'][c['id']] for c in s['cards']}
        checkpoint['permanents'] = {}
        for p in s['permanents']:
            card = physical[p['id']].split('/')[1] if not p['token'] else 'goblin-token'
            creature = 'Creature' in p['types']
            v = dict(card=card, owner=seats[p['owner']], controller=seats[p['controller']],
                     tapped=p['tapped'], sick=p['sick'] if creature else False,
                     power=p['power'] if creature else None, toughness=p['toughness'] if creature else None,
                     damage=p['damage'] if creature else None)
            for keyword in ('flying','reach','vigilance','deathtouch','haste','trample'):
                v[keyword] = any(a['class']=='mage.abilities.keyword.'+keyword.title()+'Ability' for a in p['abilities'])
            checkpoint['permanents'][physical[p['id']]] = v
            if p['token']:
                checkpoint['incarnations'][physical[p['id']]] = 0
        checkpoint['stack'] = []
        for item in reversed(s['stack']):
            a = item['ability']
            uid = a['source']
            need(uid in action_sources, r['sequence'], 'stack source action')
            action = action_sources[uid]
            key = physical[uid].split('/')[1]
            if a['type']=='Spell':
                roles = {'giant-growth':['growth_target'], 'bite-down':['bite_source','bite_destination']}.get(key, [])
                need(len(roles)==len(a['targets']), r['sequence'], 'stack target roles')
                v = dict(**ref(uid,r),action=action,mode=None,
                         targets=[dict(object=target_ref(t,r),role=role) for t,role in zip(a['targets'],roles)])
            else:
                need(a['type']=='Activated' and key in ('wildheart-invoker','shivan-dragon','axgard-cavalry'), r['sequence'], 'unsupported stack ability')
                v = dict(**ref(uid,r),action=action,object=f'ability/{action}',
                         ability={'wildheart-invoker':'invoker','shivan-dragon':'power','axgard-cavalry':'haste'}[key],
                         target=target_ref(a['targets'][0],r) if a['targets'] else None)
            checkpoint['stack'].append(v)
        need(not s['pending_triggers'], r['sequence'], 'unsupported pending trigger context')
        checkpoint['pending_triggers'] = []
        checkpoint['exile'] = [physical[uid] for uid in s['exile']]
        checkpoint['combat'] = [dict(attacker=ref(uid,r),blocked=g['blocked'],
                                    blockers=[ref(b,r) for b in g['blockers']])
                                for g in s['combat'] for uid in g['attackers']]
        # Aggregate the source's explicitly observed continuous modifications
        # into the existing native observer representation; damage is a mark,
        # not a continuous boost. No computed native state supplies this data.
        mods = {}
        grants = []
        def modification(uid):
            return mods.setdefault(uid, dict(object=ref(uid,r),boost=0,power_boost=0,damage=0))
        for p in s['permanents']:
            if p['damage']:
                modification(p['id'])['damage'] = p['damage']
        prefix = 'mage.abilities.effects.'
        for effect in s['effects']:
            native_fields = effect['native_fields']
            need(effect['duration']=='until end of turn' and not native_fields[prefix+'ContinuousEffectImpl.discarded'], r['sequence'], 'effect lifetime')
            pointer = native_fields[prefix+'EffectImpl.targetPointer']
            need(pointer['class']=='mage.target.targetpointer.FirstTargetPointer', r['sequence'], 'unsupported effect pointer')
            counters = pointer['mage.target.targetpointer.NthTargetPointer.zoneChangeCounter']['entries']
            need(len(counters)==1, r['sequence'], 'effect target identity')
            uid = counters[0]['key']
            target = dict(targets=[dict(id=uid,amount=0)],zone_change_counters=counters)
            bound = target_ref(target,r)
            need(bound==ref(uid,r), r['sequence'], 'departed effect target')
            cls = effect['class']
            if cls==prefix+'common.continuous.BoostTargetEffect':
                power = native_fields[cls+'.power']; toughness = native_fields[cls+'.toughness']
                if toughness=={'reference':power.get('node')}:
                    toughness=power
                need(power.get('class')==toughness.get('class')=='mage.abilities.dynamicvalue.common.StaticValue', r['sequence'], 'unsupported dynamic boost')
                p,t = power['mage.abilities.dynamicvalue.common.StaticValue.value'], toughness['mage.abilities.dynamicvalue.common.StaticValue.value']
                modification(uid)['boost'] += t
                modification(uid)['power_boost'] += p-t
            elif cls==prefix+'common.continuous.GainAbilityTargetEffect':
                ability=native_fields[cls+'.ability']['class']
                need(ability in ('mage.abilities.keyword.TrampleAbility','mage.abilities.keyword.HasteAbility'), r['sequence'], 'unsupported grant')
                grants.append(dict(object=bound,keyword='trample' if ability.endswith('TrampleAbility') else 'haste'))
            else:
                need(False,r['sequence'],'unsupported effect '+cls)
        checkpoint['effects'] = sorted(mods.values(),key=lambda m:m['object']['source'])
        checkpoint['grants'] = sorted(grants,key=lambda g:(g['object']['source'],g['keyword']))
        return checkpoint

    def emit(r, kind, actor=None, **fields):
        s = r['state']
        e = dict(sequence=len(play), turn=s['turn'], step=s['step'].lower().replace(' ', '_'),
                 actor=seats[r['actor']] if actor is None else actor, kind=kind,
                 source=None, incarnation=None, color=None, role=None, mode=None, order=None, player=None)
        e.update(fields)
        if kind in ('pass','cast','play_land','activate'):
            checkpoints.append(dict(action=len(play), source=r['sequence'], state=observe(r,e['actor'])))
        if kind in ('cast','activate'):
            action_sources[next(uid for uid,name in physical.items() if name==fields['source'])] = len(play)
        play.append(e)
        mappings.append(r['sequence'])

    def opening(r, kind, selection):
        actor = seats[r['actor']]
        choices.append(dict(sequence=len(choices), kind=kind, actor=actor, source=None,
                            round=rounds[actor], selection=selection))
        if selection == 'mulligan':
            rounds[actor] += 1

    opening_checkpoints = []
    # CR103.5: retain every declaration, redraw and bottom in source order.
    for r in rows:
        k = r['kind']
        if k == 'call/chooseMulligan' or (k == 'call/chooseTarget' and r['state']['step'] is None):
            a=seats[r['actor']]; p=r['state']['players'][a]
            opening_checkpoints.append(dict(source=r['sequence'],state=dict(actor=a,
                boundary='declaration' if k=='call/chooseMulligan' else 'bottom',
                hand=sorted(physical[v] for v in p['hand']),
                library=[physical[v] for v in p['library']],
                life=[p['life'] for p in r['state']['players']],mulligans=rounds[a])))
        if k == 'return/shuffleLibrary':
            a = seats[r['actor']]
            chance.append(dict(sequence=len(chance), kind='initial_shuffle' if len(chance)<2 else 'mulligan_shuffle',
                               actor=a, source=None, before=decks[a]['occurrences'],
                               after=[physical[x] for x in r['state']['players'][a]['library']]))
        elif k == 'return/chooseMulligan':
            opening(r, 'declare', 'mulligan' if r['detail']['result'] else 'keep')
        elif k == 'return/chooseTarget' and r['state']['step'] is None:
            opening(r, 'bottom', [physical[t['id']] for t in r['detail']['arguments'][1]['targets']])
    source_opening_choices = json.loads(json.dumps(choices))

    checkpoints = []
    auxiliary = dict(marked=[],cleanup_checkpoints=[],terminal=[])
    loss_departures = []
    skip_until = -1
    declarations = set()
    for r in rows:
        i, k, s = r['sequence'], r['kind'], r['state']
        if k=='event/before' and r['detail']['type']=='LOST':
            actor=seats[r['actor']]
            loss_departures.append(dict(source=i,actor=actor,zones={
                field:[physical[uid] for uid in s['players'][actor][field]]
                for field in ('hand','library','graveyard')}))
        if k=='event/before' and r['detail']['type']=='COMBAT_DAMAGE_STEP_PRIORITY':
            auxiliary['marked'].append(dict(source=i,state=observe(r,None)))
        if k=='event/after' and r['detail']['type']=='CLEANUP_STEP_POST':
            auxiliary['cleanup_checkpoints'].append(dict(source=i,state=observe(r,None)))
        if k=='terminal':
            auxiliary['terminal'].append(dict(source=i,state=observe(r,None)))
        if s['step'] is None or i <= skip_until:
            continue
        if k == 'call/playLand':
            end = paired[i]
            need(end['detail']['result'] is True, i, 'failed land choice')
            emit(r, 'play_land', **ref(r['detail'][0]['id'], r))
        elif k == 'call/cast' or (k == 'call/activateAbility' and r['detail'][0]['type']=='Activated' and r['detail'][0]['source'] is not None):
            end = paired[i]
            need(end['detail']['result'] is True, i, 'failed transaction')
            ability = r['detail'][0]
            uid = ability['source']
            activation = k != 'call/cast'
            emit(r, 'activate' if activation else 'cast', **ref(uid, r))
            stacked = [v['ability'] for v in end['state']['stack'] if v['source']==uid]
            need(len(stacked)==1, i, 'committed ability')
            committed = stacked[0]
            key = physical[uid].split('/')[1]
            if key == 'goblin-surprise':
                effects = [e['class'] for m in committed['available_modes'] if m['key'] in committed['modes'] for e in m['value']['effects']]
                need(False, i, 'unsupported mode '+str(effects))
            for n, target in enumerate(committed['targets']):
                need(len(target['targets'])==1, i, 'target cardinality')
                target_id = target['targets'][0]['id']
                role = {'giant-growth': ['growth_target'], 'bite-down': ['bite_source','bite_destination']}.get(key)
                need(activation or role is not None, i, 'unsupported target role')
                emit(r, 'activation_target' if activation else 'target', **ref(target_id,r),
                     **({} if activation else {'role':role[n]}))
            if committed['targets'] and not activation:
                emit(r, 'finish_targets')
            costs = [x for x in rows[i+1:end['sequence']] if x['kind']=='event/after' and x['detail']['type']=='DISCARDED_CARD']
            for x in costs:
                emit(r, 'discard', **ref(x['detail']['target'],r))
            taps = [x for x in rows[i+1:end['sequence']] if x['kind']=='event/after' and x['detail']['type']=='TAPPED_FOR_MANA']
            for x in taps:
                emit(r, 'tap_mana', **ref(x['detail']['source'],r))
            for cost in committed['mana_to_pay']:
                need(cost['paid'], i, 'unpaid cost')
                for color,name in enumerate(COLORS):
                    for _ in range(cost['payment'][name]):
                        emit(r, 'activation_pay' if activation else 'pay', color=color)
            emit(r, 'finish_activation' if activation else 'finish_payment')
            skip_until = end['sequence']
        elif k == 'call/pass':
            emit(r, 'pass')
        elif k == 'event/after' and r['detail']['type']=='DECLARE_ATTACKERS_STEP':
            need(s['turn'] not in declarations, i, 'duplicate attackers')
            declarations.add(s['turn'])
            selected=[ref(a,r) for g in s['combat'] for a in g['attackers']]
            emit(r,'attackers' if selected else 'empty_attackers', actor=seats[s['active']], **({'selection':selected} if selected else {}))
        elif k == 'return/selectBlockers':
            selected=[dict(blocker=ref(b,r),attacker=ref(g['attackers'][0],r)) for g in s['combat'] for b in g['blockers']]
            emit(r,'blockers',selection=selected)
        elif k == 'return/getMultiAmountWithIndividualConstraints':
            args, amounts = r['detail']['arguments'], r['detail']['result']
            # Pinned source callback embeds the attacker UUID in its typed
            # MultiAmountType header. Do not identify cards by display name.
            ids = re.findall(r"object_id='([0-9a-f-]{36})'", args[4]['header'])
            need(len(ids)==1, i, 'damage attacker binding')
            groups = [g for g in s['combat'] if g['attackers']==ids]
            need(len(groups)==1, i, 'damage combat group')
            group = groups[0]
            # Source message text is hashed; multi-blocker recipient ordering
            # needs an additional typed capture seam, not a guessed permutation.
            need(len(group['blockers'])==1 and len(amounts)==1, i, 'unsupported multi-blocker damage recipient binding')
            emit(r,'damage',selection=[dict(attacker=ref(ids[0],r), amounts=[dict(target=ref(group['blockers'][0],r),amount=amounts[0])])])
        elif k == 'terminal':
            emit(r,'finish',actor=seats[s['active']])
        elif k.startswith('call/'):
            wrappers={'call/priority','call/selectAttackers','call/selectBlockers',
                      'call/getMultiAmountWithIndividualConstraints'}
            if k=='call/activateAbility':
                a=r['detail'][0]
                need(a['type'] in ('Spell','Play land') or a['class']=='mage.abilities.common.PassAbility',i,'unsupported standalone ability')
            else:
                need(k in wrappers,i,'unsupported standalone choice '+k)
    case=dict(id='L0-XMage-captured',starter=config['starter'],decks=decks,chance=chance,choices=choices,play=play,stop='terminal')
    pins=json.loads((ROOT/'fixtures/reference/full-pool-admission.json').read_text())['pins']
    terminal=rows[-1]
    need(terminal['kind']=='terminal',terminal['sequence'],'missing terminal')
    losses=[None,None]
    for loss in terminal['detail']:
        losses[seats[loss['player']]]=loss_reason(loss,terminal['sequence'])
    winners=[i for i,p in enumerate(terminal['state']['players']) if p['won']]
    need(len(winners)==1,terminal['sequence'],'unsupported draw')
    provenance=dict(source='XMage ComputerPlayer7',normalization_version=1,
                    trace_sha256=xmage.sha(path),receipt_sha256=xmage.sha(path.with_name('attempt.json')))
    return dict(transcript=dict(schema_version=10,family='admission',pins=pins,cases=[case],provenance=provenance),auxiliary=auxiliary, source_checkpoints=checkpoints, source_cursors=mappings, source_opening_choices=source_opening_choices,opening_checkpoints=opening_checkpoints,outcome=dict(winner=winners[0],losses=losses),loss_departures=loss_departures,source_receipt=receipt)


def native_checkpoint(point):
    """Representation normalization only; never change game-derived values."""
    result = dict(point)
    result['hand_membership'] = [sorted(h) for h in point['hand']]
    result['battlefield_membership'] = sorted(point['battlefield'])
    result['incarnations'] = {k:v for k,v in point['incarnations'].items() if not k.startswith('ability/')}
    result['stack'] = [{k:v for k,v in item.items() if k!='raw_birth'} for item in point['stack']]
    # Zero-valued damage bookkeeping has no rules effect (CR120.6/514.2).
    result['effects'] = sorted((m for m in point['effects'] if any(m[k] for k in ('boost','power_boost','damage'))),key=lambda m:m['object']['source'])
    result['grants'] = sorted(point['grants'],key=lambda g:(g['object']['source'],g['keyword']))
    return result


def verify_native(converted, run):
    from instant_reference import difference
    import admission_observations
    import admission_reference
    case = converted['transcript']['cases'][0]
    for name in ('chance','choices'):
        need(run['opening']['consumed_'+name]==case[name], 0, 'consumed '+name)
    need(run['consumed_play']==case['play'],0,'consumed play')
    need(len(run['points'])==len(case['play']),0,'checkpoint count')
    for i,p in enumerate(run['points']):
        required=admission_reference.FIELDS|{'grants'}
        for field in sorted(required-set(p)):
            need(False,converted['source_cursors'][i],f'checkpoint/action/{i}/{field} missing')
    admission_observations.check(run,case,'native')
    need(len(run['opening']['points'])==len(converted['opening_checkpoints'])+2,0,'opening checkpoint count')
    need(run['points'][-1]['outcome']==converted['outcome'],converted['source_cursors'][-1],'outcome')
    for c,p in zip(converted['opening_checkpoints'],run['opening']['points']):
        delta=difference(c['state'],p)
        need(delta is None,c['source'],f'opening checkpoint: {delta}')
    for c in converted['source_checkpoints']:
        actual = native_checkpoint(run['points'][c['action']])
        for key,value in c['state'].items():
            need(key in actual,c['source'],f'checkpoint/action/{c["action"]}/{key} missing')
            delta = difference(value,actual[key])
            need(delta is None,c['source'],f"checkpoint/action/{c['action']}/{key}: {delta}")
    for kind,expected in converted['auxiliary'].items():
        actual=run[kind] if kind!='terminal' else [run['points'][-1]]
        need(len(expected)==len(actual),0,'auxiliary/'+kind+'/count')
        for c,p in zip(expected,actual):
            observed=native_checkpoint(p)
            if kind=='terminal':
                # Pinned XMage clears the loser's existing private zones at
                # LOST, before this same SBA moves lethal creatures. Preserve
                # both original snapshots; project only the witnessed removals.
                import copy
                observed=copy.deepcopy(observed)
                for departure in converted['loss_departures']:
                    actor=departure['actor']
                    need(converted['outcome']['losses'][actor] is not None,departure['source'],'loser zone departure')
                    for field,removed in departure['zones'].items():
                        key='hand_membership' if field=='hand' else field
                        need(all(uid in observed[key][actor] for uid in removed),departure['source'],'terminal departure/'+field)
                        observed[key][actor]=[uid for uid in observed[key][actor] if uid not in removed]
            for key,value in c['state'].items():
                delta=difference(value,observed[key])
                need(delta is None,c['source'],f'auxiliary/{kind}/{key}: {delta}')
    return dict(checkpoints=len(converted['source_checkpoints']),actions=len(case['play']),chance=len(case['chance']))


def negative_inputs(converted):
    """Illegal edits justified by the immutable source and CR 400.7/601/603.

    The chosen captured game has no trigger order decision. Inserting one at
    its first priority must fail there, rather than silently consuming a pass.
    Existing admission trigger permutations remain mandatory regressions.
    """
    import copy
    case=converted['transcript']['cases'][0]
    result={}
    def add(name,index,edit):
        bad=copy.deepcopy(case);edit(bad)
        result[name]=dict(case=bad,sequence=index)
    add('duplicate_card_identity',2,lambda c:c['play'][2].update(source='0/mountain/0'))
    add('shuffle',None,lambda c:c['chance'][0]['after'].__setitem__(0,c['chance'][0]['after'][1]))
    add('cost',23,lambda c:c['play'][23].update(color=3))
    add('target',137,lambda c:c['play'][137].update(source='0/mountain/8',incarnation=2))
    add('trigger_order',0,lambda c:c['play'][0].update(kind='order_triggers',order=[]))
    return result


def verify_pair(converted,native,reference):
    import admission_reference as admission
    import admission_observations
    from instant_reference import difference
    verify_native(converted,native)
    case=converted['transcript']['cases'][0]
    projected,indices=admission.reference_case(case)
    need(reference['consumed_play']==projected['play'],0,'XMage consumed play')
    for kind in ('chance','choices'):
        need(reference['opening']['consumed_'+kind]==case[kind],0,'XMage consumed '+kind)
    need(len(reference['points'])==len(indices),0,'XMage point count')
    admission_observations.check(reference,projected,'xmage')
    admission_observations.xmage_effect_lifecycle(reference,projected)
    admission_observations.xmage_trigger_provenance(reference,projected)
    aligned=dict(zip(indices,reference['points']))
    for c in converted['source_checkpoints']:
        i=c['action']
        delta=difference(admission.semantic(native['points'][i]),admission.semantic(aligned[i]))
        need(delta is None,c['source'],f'XMage checkpoint/action/{i}: {delta}')
    for kind in ('marked','cleanup_checkpoints'):
        need(len(native[kind])==len(reference[kind]),0,'XMage '+kind+' count')
        for i,(p,q) in enumerate(zip(native[kind],reference[kind])):
            delta=difference(admission.semantic(p),admission.semantic(q))
            need(delta is None,i,f'XMage {kind}: {delta}')
    need(reference['points'][-1]['outcome']==converted['outcome'],converted['source_cursors'][-1],'XMage outcome')
    terminal=dict(reference['before_departure'],outcome=reference['points'][-1]['outcome'],actor=None)
    delta=difference(admission.semantic(native['points'][-1]),admission.semantic(terminal))
    need(delta is None,converted['source_cursors'][-1],f'XMage terminal state: {delta}')
    return dict(actions=len(case['play']),chance=len(case['chance']),source_checkpoints=len(converted['source_checkpoints']))


def run_native(folder):
    import os
    import subprocess
    import sys
    folder=Path(folder).resolve()
    env=dict(os.environ,MTG_AI_REPLAY_OUTPUT=str(folder))
    command=['cargo','test','-p','mtg-core','--locked','--lib','captured_xmage_ai_strict_replay']
    fd=env.get('MTG_SYMPHONY_LOCK_FD');fds=()
    if fd is not None:
        try:
            os.fstat(int(fd));fds=(int(fd),)
        except OSError:
            fd=None
    if fd is None:
        command=[sys.executable,str(ROOT/'scripts/symphony/resource_lock.py'),'heavy','--']+command
    with (folder/'native.log').open('w') as log:
        subprocess.run(command,cwd=ROOT,env=env,stdin=subprocess.DEVNULL,stdout=log,stderr=subprocess.STDOUT,pass_fds=fds,timeout=3600,check=True)
    return [json.loads((folder/f'native-{i}.json').read_text())['run'] for i in (1,2)]


def main():
    import argparse
    import hashlib
    import os
    import admission_reference as admission
    import xmage
    import time
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source',type=Path,default=ROOT/'doc/evidence/xmage-ai-capture/acceptance-005/game/trace.jsonl.xz')
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--cache',type=Path,required=True)
    args=parser.parse_args()
    folder=args.output.resolve();folder.mkdir(parents=True,exist_ok=False)
    os.chmod(folder,0o700)
    def write(name,value):(folder/name).write_text(json.dumps(value,indent=2)+'\n')
    started=time.monotonic()
    paths=[Path(__file__),ROOT/'scripts/admission_observations.py',ROOT/'scripts/admission_reference.py',
           ROOT/'references/xmage/FullPoolAdmissionTest.java',ROOT/'references/xmage/FullPoolTriggersTest.java',
           ROOT/'references/xmage/FullPoolMulliganTest.java',ROOT/'references/xmage/FullPoolPriorityTest.java',
           ROOT/'references/xmage/CombatSbaGame.java',ROOT/'crates/mtg-core/src/admission_reference_tests.rs']
    paths=sorted(set(paths)|set((ROOT/'crates/mtg-core/src').glob('*.rs'))|
                 set((ROOT/'scripts').glob('*reference*.py'))|
                 {ROOT/'Cargo.toml',ROOT/'Cargo.lock',ROOT/'crates/mtg-core/Cargo.toml',
                  ROOT/'scripts/xmage.py',ROOT/'scripts/xmage_ai_capture.py',ROOT/'scripts/scenario.py'})
    hashes={str(p.relative_to(ROOT)):xmage.sha(p) for p in paths}
    receipt=dict(status='started',source_sha256=xmage.sha(args.source),reference='patched-combat-sba-v1',sources=hashes)
    write('receipt.json',receipt)
    try:
        converted=normalize(args.source)
        case=converted['transcript']['cases'][0]
        canonical=dict(normalization_version=1,**{k:case[k] for k in ('starter','decks','chance','choices','play')})
        receipt['game_identity_sha256']=hashlib.sha256(json.dumps(canonical,sort_keys=True,separators=(',',':')).encode()).hexdigest()
        write('canonical-game.json',canonical)
        # Freeze source-derived inputs before either consumer executes.
        write('converted.json',converted)
        write('input.json',converted['transcript'])
        negatives=negative_inputs(converted);write('negatives.json',negatives)
        os.environ['MTG_AI_REPLAY_INPUT']=str(folder/'input.json')
        os.environ['MTG_AI_REPLAY_NEGATIVES']=str(folder/'negatives.json')
        native=run_native(folder)
        for run in native:verify_native(converted,run)
        doc=json.loads(json.dumps(converted['transcript']))
        doc['cases']=[admission.reference_case(c)[0] for c in doc['cases']]
        write('xmage-input.json',doc)
        write('xmage-extra-negatives.json',{name:dict(case=admission.reference_case(v['case'])[0],sequence=v['sequence']) for name,v in negatives.items()})
        cache=args.cache.resolve();xmage.verify_inputs(cache)
        need(xmage.dependencies(cache)==json.loads((ROOT/'references/xmage/dependencies.json').read_text()),0,'reference dependencies')
        for family in ('mulligan','priority','spells','triggers','activations','played-combat','admission'):
            for name,sha in json.loads((ROOT/f'references/xmage/{family}-provenance.json').read_text())['consulted_sources'].items():
                need(xmage.sha(cache/xmage.SOURCE/name)==sha,0,'upstream source '+name)
        for name in ('FullPoolMulliganTest.java','FullPoolPriorityTest.java','FullPoolTriggersTest.java','FullPoolAdmissionTest.java','CombatSbaGame.java'):
            target=cache/xmage.SOURCE/'Mage.Tests/src/test/java/org/mage/test/mtglab'/name
            target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes((ROOT/'references/xmage'/name).read_bytes())
        command=xmage.maven(cache)+['-o','-pl','Mage.Tests','-am','test','-Dtest=org.mage.test.mtglab.FullPoolAdmissionTest','-Dsurefire.failIfNoSpecifiedTests=false','-Dmtglab.fixture='+str(folder/'xmage-input.json'),'-Dmtglab.root='+str(ROOT),'-Dmtglab.extraNegatives='+str(folder/'xmage-extra-negatives.json'),'-Dmtglab.output='+str(folder/'xmage-index.json'),'-DargLine=-Djava.awt.headless=true']
        xmage.bounded(command,cache/xmage.SOURCE,xmage.environment(cache),folder/'xmage.log',600)
        index=json.loads((folder/'xmage-index.json').read_text())
        results=[]
        for i,name in enumerate(index[doc['cases'][0]['id']]):
            reference=json.loads((folder/name).read_text())
            results.append(verify_pair(converted,native[i],reference))
        for engine in ('native','xmage'):
            actual=json.loads((folder/f'{engine}-negatives.json').read_text())
            for name,v in negatives.items():
                need(name in actual,0,engine+' missing negative '+name)
                if v['sequence'] is not None:
                    need(len(actual[name]['run']['consumed_play'])==v['sequence'],v['sequence'],engine+' negative cursor '+name)
        receipt.update(status='passed',repeats=results,negative_controls=list(negatives))
        need(hashes=={str(p.relative_to(ROOT)):xmage.sha(p) for p in paths},0,'execution sources changed')
    except Exception as error:
        receipt.update(status='failed',error=str(error));raise
    finally:
        receipt['elapsed_seconds']=time.monotonic()-started
        receipt['artifacts']={p.name:xmage.sha(p) for p in folder.iterdir() if p.is_file() and p.name!='receipt.json'}
        write('receipt.json',receipt)


if __name__=='__main__':
    main()
