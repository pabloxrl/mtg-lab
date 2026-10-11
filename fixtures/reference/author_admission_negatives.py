"""Minimized strict-client mutations of frozen games, with required fail cursor.

Expectations come from identity/permutation contracts and CR 117/601/510;
this author reads inputs only, never captured results.
"""
import copy
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]


def author(doc,project=lambda c:c):
    out={}
    def add(name,source,index,mutate,basis):
        c=project(copy.deepcopy(source));c.pop('native_records',None);c.pop('native_indices',None)
        mutate(c)
        for i,e in enumerate(c['play']):e['sequence']=i
        # Retain sufficient context through the invalid callback, not a fake
        # reset state. Every setup action is replayed through the actual client.
        if index is not None:c['play']=c['play'][:index+2]
        out[name]=dict(case=c,sequence=index,basis=basis)
    c=project(copy.deepcopy(doc['cases'][0]))
    # Already projected row below; avoid double projection in mutation selection.
    for name,mutate in [('missing_action',lambda c:c['play'].pop(0)),('extra_action',lambda c:c['play'].insert(1,copy.deepcopy(c['play'][0]))),('reordered_action',lambda c:c['play'].__setitem__(slice(0,2),list(reversed(c['play'][:2]))))]:
        add(name,doc['cases'][0],1 if name=='extra_action' else 0,mutate,'CR117: exact selected actor and chronological action consumption')
    add('truncated',doc['cases'][0],3,lambda c:c.update(play=c['play'][:3]),'GR-040: a nonterminal prefix cannot fill a terminal-game quota')
    for name in ('concede','timeout'):
        add(name,doc['cases'][0],0,lambda c,k=name:c['play'][0].update(kind=k),'GR-040: only nonconcession rules-terminal games qualify')
    add('missing_required_field',doc['cases'][0],0,lambda c:c['play'][0].pop('color'),'version10: even required null fields must be present')
    for kind,name,mutate in [
        ('pay','wrong_payment',lambda e:e.update(color=3 if e['color']==4 else 4)),
        ('target','wrong_target',lambda e:e.update(source='0/forest/0',incarnation=0)),
        ('cast','wrong_incarnation',lambda e:e.update(incarnation=e['incarnation']+1)),
        ('damage','wrong_damage',lambda e:e['selection'][0]['amounts'][0].update(amount=1000)),
        ('mode','oversized_mode',lambda e:e.update(mode=256)),
        ('target_player','oversized_player_target',lambda e:e.update(player=256)),
    ]:
        source=next(s for s in doc['cases'] if any(e['kind']==kind for e in s['play']))
        index=next(i for i,e in enumerate(project(copy.deepcopy(source))['play']) if e['kind']==kind)
        add(name,source,index,lambda c,i=index,f=mutate:f(c['play'][i]),'CR601/510 and physical occurrence contract: selected payment, target, incarnation and damage must be legal')
    for name,mutate in [('duplicate_identity',lambda c:c['chance'][0]['after'].__setitem__(1,c['chance'][0]['after'][0])),('missing_chance',lambda c:c['chance'].pop()),('extra_chance',lambda c:c['chance'].append(copy.deepcopy(c['chance'][0]))),('reordered_chance',lambda c:c['chance'].reverse())]:
        add(name,doc['cases'][0],None,mutate,'CR103 and strict chance contract: full physical permutation and exact event consumption')
    add('mismatched_pin',doc['cases'][0],None,lambda c:None,'Pinned source/card/rules inputs are mandatory')
    out['mismatched_pin']['pins']=dict(doc['pins'])
    out['mismatched_pin']['pins']['references/xmage/pins.json']='0'*64
    return out

if __name__=='__main__':
    import sys
    sys.path.insert(0,str(ROOT/'scripts'))
    from admission_reference import reference_case
    doc=json.loads((ROOT/'fixtures/reference/full-pool-admission.json').read_text())
    for engine,project in [('native',lambda c:c),('xmage',lambda c:reference_case(c)[0])]:
        (ROOT/f'fixtures/reference/full-pool-admission-{engine}-negatives.json').write_text(json.dumps(author(doc,project),indent=2)+'\n')
