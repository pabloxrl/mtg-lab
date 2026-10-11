"""Translate witnessed native heuristic choices, never author expected outcomes.

The input is privileged generation provenance, NOT a golden. Raw native
micro-choices remain alongside the aligned reference transactions. Only genuine
creation/choice observations provide identity; no name-based matching.
"""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
STEPS = dict(Upkeep='upkeep', Draw='draw', PrecombatMain='precombat_main',
             BeginningCombat='begin_combat', DeclareAttackers='declare_attackers',
             DeclareBlockers='declare_blockers', CombatDamage='combat_damage',
             EndCombat='end_combat', PostcombatMain='postcombat_main', End='end_turn', Cleanup='cleanup')


def convert(origin):
    births = {int(k): v for k, v in origin['birth_occurrences'].items()}
    handles = {}
    token_event = 0
    trigger_event = 0
    pending_keys = {}
    previous_pending = False
    events = []
    opening = []
    pending_combat = {}
    pending_order = []
    native_indices = []
    declarations = set()
    blocks = set()

    def handle_key(h):
        return tuple(h[k] for k in ('store', 'epoch', 'slot', 'generation'))

    def observe(state):
        nonlocal token_event, trigger_event, previous_pending, pending_keys
        new_tokens = [o for o in state['objects'] if o['birth'] not in births and o['card'] == 'goblin-token']
        for ordinal, o in enumerate(new_tokens):
            births[o['birth']] = f"token/{o['controller']}/{token_event}/{ordinal}"
        if new_tokens:
            token_event += 1
        for o in state['objects']:
            if o['birth'] in births:
                handles[handle_key(o['handle'])] = dict(source=births[o['birth']], incarnation=o['incarnation'])
        pending = state['turns']['pending_triggers']
        live = any(p is not None for p in pending)
        if live and not previous_pending:
            pending_keys = {}
            for i, p in enumerate(pending):
                if p is None:
                    continue
                source = handles[handle_key(p['source'])]
                kind = p['kind']
                name = kind if isinstance(kind, str) else next(iter(kind))
                pending_keys[i] = dict(source, ability={'Archer':'archer','Cyclops':'cyclops','Pyromancer':'pyromancer'}[name], event=trigger_event)
            trigger_event += 1
        previous_pending = live

    def reference(r):
        # Birth is the engine-issued physical identity; never match by card name.
        if r['birth'] not in births:
            raise ValueError(f"unwitnessed birth {r['birth']}")
        return dict(source=births[r['birth']], incarnation=r['incarnation'])

    def emit(before, actor, kind, indices, **fields):
        position = before['turns']['position']
        if position is None:
            raise ValueError('played event before first upkeep')
        event = dict(sequence=len(events),turn=position[0],step=STEPS[position[2]],actor=int(actor[1]),
                     kind=kind,source=None,incarnation=None,color=None,role=None,mode=None,order=None,player=None)
        event.update(fields)
        events.append(event)
        native_indices.append(indices)
        if kind in ('attackers','empty_attackers'):declarations.add(position[0])
        if kind=='blockers':blocks.add(position[0])

    for index, row in enumerate(origin['history']):
        before, after = row['observed_before'], row['observed_after']
        observe(before)
        r = row['choice']
        decision, actor, choices = r['decision'], r['actor'], r['choices']
        position=before['turns']['position']
        if decision=='priority' and position is not None:
            turn,active,step=position
            if step=='DeclareAttackers' and turn not in declarations:
                if before['turns']['combat']['attacks'] or before['turns']['combat']['selected']:
                    raise ValueError('unwitnessed attacker declaration')
                emit(before,active,'empty_attackers',[])
            if step=='DeclareBlockers' and turn not in blocks:
                if before['turns']['combat']['blocks']:
                    raise ValueError('unwitnessed blocker declaration')
                emit(before,'P'+str(1-int(active[1])),'blockers',[],selection=[])
        if decision == 'keep_or_mulligan':
            if choices != [{'kind':'keep'}]:
                raise ValueError('this heuristic source must explicitly keep; mulligan generation requires witnessed shuffle callbacks')
            opening.append(dict(sequence=len(opening),kind='declare',actor=int(actor[1]),source=None,round=0,selection='keep'))
        elif decision in ('attackers','blockers','combat_damage'):
            pending = pending_combat.setdefault(decision, dict(before=before,indices=[],selection=[]))
            pending['indices'].append(index)
            for c in choices:
                kind = c['kind']
                if kind == 'select_attackers':
                    pending['selection'] = [reference(x) for x in c['cards']]
                elif kind == 'select_blockers':
                    pending['selection'] = [dict(blocker=reference(b),attacker=reference(a)) for b,a in c['blocks']]
                elif kind == 'assign_damage':
                    pending['selection'].append(dict(attacker=reference(c['attacker']),amounts=[dict(target=reference(b),amount=n) for b,n in c['amounts']]))
                elif kind == 'finish_combat':
                    selected = pending['selection']
                    if decision == 'attackers':
                        emit(pending['before'],actor,'attackers' if selected else 'empty_attackers',pending['indices'],**({'selection':selected} if selected else {}))
                    elif decision == 'blockers':
                        emit(pending['before'],actor,'blockers',pending['indices'],selection=selected)
                    elif selected:
                        # Each engine callback names its actual combat group.
                        for selection in selected:
                            emit(pending['before'],actor,'damage',pending['indices'],selection=[selection])
                    pending_combat.pop(decision)
                else:
                    raise ValueError('unsupported combat choice '+kind)
        elif decision == 'trigger_order':
            for c in choices:
                if c['kind'] != 'order_trigger':
                    raise ValueError('unsupported trigger-order choice')
                pending_order.append((pending_keys[c['trigger']], index, before))
            next_decision = after['turns']['decision']
            if next_decision is None or next_decision['kind'] != 'TriggerOrder':
                emit(pending_order[0][2],actor,'order_triggers',[v[1] for v in pending_order],order=[v[0] for v in pending_order])
                pending_order=[]
        elif decision in ('cleanup_discard','graveyard_order'):
            if not all(c['kind'] == ('discard' if decision=='cleanup_discard' else 'order_graveyard') for c in choices):
                raise ValueError('unexpected ordered zone choice')
            fields=dict(selection=[reference(c['card']) for c in choices])
            if decision == 'graveyard_order':
                fields['event']=f"combat-sba/{before['turns']['position'][0]}"
            emit(before,actor,decision,[index],**fields)
        else:
            if len(choices)!=1:
                raise ValueError('unexpected noncombat choice cardinality')
            c=choices[0];kind=c['kind'];fields={}
            if 'card' in c:
                fields.update(reference(c['card']))
            if kind=='target':
                if decision=='activation_target':kind='activation_target'
                else:fields['role']=decision
            if kind=='pay':
                fields['color']=c['color']
                if decision=='activation_payment':kind='activation_pay'
            if kind=='mode':fields['mode']=c['mode']
            if kind=='target_player':fields['player']=c['seat']
            emit(before,actor,kind,[index],**fields)
        observe(after)
    if pending_combat or pending_order:
        raise ValueError('unconsumed native choice group')
    if origin['outcome'] is None or 'Concession' in origin['outcome']['losses']:
        raise ValueError('not a rules-terminal origin')
    emit(origin['history'][-1]['observed_after'],'P'+str(origin['config']['starting_seat']),'finish',[])
    decks=[dict(deck=s['deck'],occurrences=origin['chance'][i]['before']) for i,s in enumerate(origin['config']['seats'])]
    row=''.join(d['deck'][0].upper() for d in decks)
    return dict(id=f"GR-040-{row}-S{origin['config']['starting_seat']}-heuristic",starter=origin['config']['starting_seat'],
                decks=decks,chance=origin['chance'],choices=opening,play=events,stop='terminal',
                native_records=[r['choice'] for r in origin['history']],native_indices=native_indices)


if __name__ == '__main__':
    import argparse
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source',type=Path)
    parser.add_argument('destination',type=Path)
    args=parser.parse_args()
    cases=[];provenance=[]
    for path in sorted(args.source.glob('*.json')):
        origin=json.loads(path.read_text())
        cases.append(convert(origin))
        provenance.append(dict(case=cases[-1]['id'],path=path.name,sha256=hashlib.sha256(path.read_bytes()).hexdigest(),policy=origin['policy']))
    pins=json.loads((ROOT/'fixtures/reference/full-pool-combat.json').read_text())['pins']
    args.destination.write_text(json.dumps(dict(schema_version=10,family='admission',pins=pins,cases=cases,provenance=provenance),indent=2)+'\n')
