"""Bind captured trigger occurrences and placement order, without engine output."""


def index(rows, physical, seats, need):
    active = {}
    groups = []
    for r in rows:
        s = r['state']
        abilities = s['pending_triggers'] + [item['ability'] for item in s['stack']
                                             if item['ability']['type'] == 'Triggered']
        present = {a['id'] for a in abilities}
        fresh = [a for a in abilities if a['id'] not in active]
        if fresh:
            group = dict(event=len(groups), keys={}, order=[], source=r['sequence'], placements=[],selected=[])
            groups.append(group)
            for a in fresh:
                card = physical[a['source']].split('/')[1]
                kind = {'firebrand-archer':'archer', 'crackling-cyclops':'cyclops',
                        'viashino-pyromancer':'pyromancer'}.get(card)
                need(kind is not None, r['sequence'], 'unsupported captured trigger')
                key = dict(source=physical[a['source']], incarnation=r['incarnations'][a['source']],
                           ability=kind, event=group['event'])
                active[a['id']] = key
                group['keys'][a['id']] = key
        r['trigger_keys'] = dict(active)
        if r['kind']=='return/chooseTriggeredAbility':
            selected=r['detail']['result'];options=r['detail']['arguments'][0]
            need(selected is not None and selected['id'] in active,r['sequence'],'selected trigger occurrence')
            group=groups[active[selected['id']]['event']]
            remaining=set(group['keys'])-set(group['placements'])
            need({a['id'] for a in options}==remaining and len(options)==len(remaining),
                 r['sequence'],'trigger candidate set')
            need(any(a==selected for a in options),r['sequence'],'selected trigger candidate')
            group['selected'].append(selected['id'])
        if r['kind']=='event/before' and r['detail']['type']=='TRIGGERED_ABILITY':
            uid=r['detail']['target']
            need(uid in active,r['sequence'],'trigger placement without occurrence')
            key=active[uid];group=groups[key['event']]
            need(uid not in group['placements'],r['sequence'],'duplicate trigger placement')
            remaining=set(group['keys'])-set(group['placements'])
            if group['selected']:
                need(group['selected'].pop(0)==uid,r['sequence'],'selected trigger placement')
            else:
                need(len(remaining)==1,r['sequence'],'missing trigger order choice')
            group['placements'].append(uid)
            group['order'].append(key)
        # The source temporarily removes a pending instance while choosing its
        # target and placing it. Retain the binding through that transaction.
        if r['kind']=='call/priority' and not s['pending_triggers']:
            active={uid:key for uid,key in active.items() if uid in present}
    for group in groups:
        need(set(group['placements'])==set(group['keys']),group['source'],
             'unplaced captured trigger')
        need(not group['selected'],group['source'],'unconsumed trigger order choice')
    return groups
