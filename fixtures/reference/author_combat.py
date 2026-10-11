"""Literal CR 508–510/702 combat tapes. Reads pins/decks, never engine output.

Shivan 5/5 flying; Sentry 4/4 reach; Thornweald 2/1 reach/deathtouch;
Pathwarden 5/4 vigilance/trample; Cavalry, Cub and Swab are 2/2.
"""
import copy
import json
from pathlib import Path
from author_priority import author
ROOT=Path(__file__).resolve().parents[2]

def ref(s):return dict(source=s,incarnation=3)
S='0/shivan-dragon/0';C='0/axgard-cavalry/0';G='0/swab-goblin/0'
P='1/tajuru-pathwarden/0';T='1/thornweald-archer/0';R='1/magnigoth-sentry/0';B='1/bear-cub/0'
SPECS={
 'shivan-unblocked':(13,S,[],None,[20,15],{},[]),
 'shivan-sentry':(13,S,[R],None,[20,20],{S:4,R:5},[R]),
 'shivan-thorn':(13,S,[T],None,[20,20],{S:2,T:5},[S,T]),
 'shivan-multiple':(13,S,[R,T],[4,1],[20,20],{S:6,R:4,T:1},[S,R,T]),
 'shivan-modern-split':(13,S,[R,T],[3,2],[20,20],{S:6,R:3,T:2},[S,T]),
 'pathwarden-single':(12,P,[G],[2],[17,20],{P:2,G:2},[G]),
 'pathwarden-multiple':(12,P,[C,G],[2,2],[19,20],{P:4,C:2,G:2},[P,C,G]),
 'pathwarden-haste':(10,P,[G],[2],[17,20],{P:2,G:2},[G]),
 'thorn-multiple':(8,T,[C,G],[1,1],[20,20],{T:4,C:1,G:1},[T,C,G]),
 'both-owners':(9,[C,G],[B,T],None,[20,20],{C:2,G:2,B:2,T:2},[C,G,B,T]),
}

def make(name):
    final,attacker,blockers,amounts,life,marks,dead=SPECS[name]
    c=author(0,False);c.update(id=name,play=[],stop='combat/'+name)
    tops=[['0/mountain/'+str(i) for i in range(3)]+[C,G,S]+['0/mountain/'+str(i) for i in range(3,10)],
          ['1/forest/'+str(i) for i in range(3)]+[T,B,R,P]+['1/forest/'+str(i) for i in range(3,10)]]
    for s,top in enumerate(tops):c['chance'][s]['after']=top+[x for x in c['decks'][s]['occurrences'] if x not in top]
    def e(t,a,k,source=None,inc=None,color=None,step='precombat_main',**kw):
        v=dict(sequence=len(c['play']),turn=t,step=step,actor=a,kind=k,source=source,incarnation=inc,color=color);v.update(kw);c['play'].append(v)
    def passes(t,a,step='precombat_main'):e(t,a,'pass',step=step);e(t,1-a,'pass',step=step)
    def cast(t,a,source,cost):
        e(t,a,'cast',source,1)
        for i in range(cost):e(t,a,'tap_mana',f'{a}/'+('mountain' if a==0 else 'forest')+f'/{i}',2)
        for _ in range(cost):e(t,a,'pay',color=3 if a==0 else 4)
        e(t,a,'finish_payment');passes(t,a)
    casts={3:(C,2),4:(T,2),5:(G,2),6:(B,2),8:(R,4),10:(P,5),11:(S,6)}
    for t in range(1,final+1):
        a=0 if t%2 else 1;n=(t-1)//2
        passes(t,a,'upkeep')
        if t!=1:passes(t,a,'draw')
        e(t,a,'play_land',f'{a}/'+('mountain' if a==0 else 'forest')+f'/{n}',1)
        if t in casts:cast(t,a,*casts[t])
        if t==final and name=='pathwarden-haste':
            e(t,1,'pass');e(t,0,'activate',C,3);e(t,0,'activation_target',P,3);e(t,0,'finish_activation');passes(t,0)
        passes(t,a);passes(t,a,'begin_combat')
        if t==final:
            e(t,a,'attackers',step='declare_attackers',selection=[ref(x) for x in (attacker if isinstance(attacker,list) else [attacker])])
            passes(t,a,'declare_attackers')
            e(t,1-a,'blockers',step='declare_blockers',selection=[dict(blocker=ref(b),attacker=ref(attacker[i] if isinstance(attacker,list) else attacker)) for i,b in enumerate(blockers)])
            passes(t,a,'declare_blockers')
            if amounts is not None:e(t,a,'damage',step='combat_damage',selection=[dict(attacker=ref(attacker),amounts=[dict(target=ref(b),amount=v) for b,v in zip(blockers,amounts)])])
            for owner in (a,1-a):
                arrivals=[x for x in dead if x.startswith(str(owner)+'/')]
                if len(arrivals)>1:e(t,owner,'graveyard_order',step='combat_damage',selection=[ref(x) for x in arrivals],event=f'combat-sba/{t}')
            e(t,a,'finish',step='combat_damage')
            break
        e(t,a,'empty_attackers',step='declare_attackers');passes(t,a,'declare_attackers')
        for step in ('end_combat','postcombat_main','end_turn'):passes(t,a,step)
    return c

def variants():
    # Literal permutations of the very same event. Physical /1 replacements are
    # authored into shuffle/played inputs, never inferred from engine output.
    out=[]
    for base in ('shivan-multiple','pathwarden-multiple','both-owners'):
        for reverse,swapped in ((True,False),(False,True),(True,True)):
            c=make(base);c['id']=base+('-reverse' if reverse else '')+('-identities' if swapped else '');c['stop']='combat/'+c['id']
            if reverse:
                for e in c['play']:
                    if e['kind']=='graveyard_order':e['selection'].reverse()
            if swapped:
                mapping={x:x[:-1]+'1' for x in (C,G,T,R)}
                mapping.update({v:k for k,v in list(mapping.items())})
                def remap(x):
                    if isinstance(x,str):return mapping.get(x,x)
                    if isinstance(x,list):return [remap(v) for v in x]
                    if isinstance(x,dict):return {k:remap(v) for k,v in x.items()}
                    return x
                c['play']=remap(c['play'])
                for chance in c['chance']:chance['after']=remap(chance['after'])
            out.append((base,c,swapped))
    return out

def fixtures():
    return dict(schema_version=9,family='combat',pins=json.loads((ROOT/'fixtures/reference/full-pool-priority.json').read_text())['pins'],cases=[make(n) for n in SPECS]+[c for _,c,_ in variants()])

def oracle():
    out={n:dict(life=v[4],marks=v[5],dead=v[6],attacker=v[1],blockers=v[2],amounts=v[3],base=n) for n,v in SPECS.items()}
    for base,c,swapped in variants():
        o=copy.deepcopy(out[base])
        if swapped:
            mapping={x:x[:-1]+'1' for x in (C,G,T,R)}
            def remap(x):
                if isinstance(x,str):return mapping.get(x,x)
                if isinstance(x,list):return [remap(v) for v in x]
                if isinstance(x,dict):return {mapping.get(k,k):remap(v) for k,v in x.items()}
                return x
            o=remap(o)
        out[c['id']]=o
    return out

if __name__=='__main__':
    for name,doc in [('full-pool-combat.json',fixtures()),('full-pool-combat-expectations.json',oracle())]:
        (ROOT/'fixtures/reference'/name).write_text(json.dumps(doc,indent=2)+'\n')

def negatives(doc):
    out={}
    def add(name,case,kind,edit,native,xmage,rule):
        d=copy.deepcopy(doc);c=next(x for x in d['cases'] if x['id']==case);d['cases']=[c]
        i=next(i for i,e in enumerate(c['play']) if e['kind']==kind)
        edit(c,i);out[name]=dict(input=d,sequence=i,native=native,xmage=xmage,requirement=rule)
    illegal='/native command Policy(InvalidSelection)'
    add('wrong-actor','shivan-unblocked','attackers',lambda c,i:c['play'][i].update(actor=1),'/wrong combat actor','/play actor/sequence','CR508.1 active player declares attackers')
    add('stale-attacker','shivan-unblocked','attackers',lambda c,i:c['play'][i]['selection'][0].update(incarnation=2),'/native command InvalidReference','/source incarnation','CR400.7 stack and battlefield are distinct incarnations')
    add('duplicate-attacker','shivan-unblocked','attackers',lambda c,i:c['play'][i]['selection'].append(ref(S)),illegal,'/selected attacker rejected','CR508.1 each attacking creature selected once')
    add('enemy-blocker','pathwarden-single','blockers',lambda c,i:c['play'][i]['selection'][0].update(blocker=ref(B)),illegal,'/selected blocker rejected','CR509.1 defending player controls each blocker')
    add('duplicate-blocker','pathwarden-multiple','blockers',lambda c,i:c['play'][i]['selection'][1].update(blocker=ref(C)),illegal,'/selected blocker rejected','CR509.1 one blocker cannot block two times')
    add('tapped-blocker','pathwarden-haste','blockers',lambda c,i:c['play'][i]['selection'][0].update(blocker=ref(C)),illegal,'/selected blocker rejected','CR509.1a Cavalry tapped to grant haste cannot block')
    add('cub-blocks-flyer','shivan-sentry','blockers',lambda c,i:c['play'][i]['selection'][0].update(blocker=ref(B)),illegal,'/selected blocker rejected','CR702.9 flying requires flying or reach to block')
    add('stale-blocker','shivan-sentry','blockers',lambda c,i:c['play'][i]['selection'][0]['blocker'].update(incarnation=2),'/native command InvalidReference','/source incarnation','CR400.7 stale blocker identity rejected')
    for name,case,values,nat,refcat,rule in [
        ('excess-damage','shivan-multiple',[4,2],illegal,'/damage total','CR510.1 all assigned damage equals power'),
        ('wrong-total','shivan-multiple',[3,1],illegal,'/damage total','CR510.1 nontrampling attacker assigns all power'),
        ('negative-damage','shivan-multiple',[-1,6],'/negative or oversized damage','/damage constraint','CR107/510 damage allocations cannot be negative'),
        ('nonlethal-trample','pathwarden-multiple',[1,3],illegal,'/nonlethal before trample','CR702.19b EACH blocker must receive lethal before remainder reaches player'),
    ]:
        def mutate(c,i,values=values):
            for v,n in zip(c['play'][i]['selection'][0]['amounts'],values):v['amount']=n
        add(name,case,'damage',mutate,nat,refcat,rule)
    add('reordered-allocation','shivan-multiple','damage',lambda c,i:c['play'][i]['selection'][0]['amounts'].reverse(),'/reordered damage recipient','/reordered damage recipient','Version8 allocation recipients follow witnessed combat group identities, never names')
    def missing(c,i):c['play'].pop(i)
    add('missing-declaration','shivan-sentry','attackers',missing,'/play actor/sequence','/play actor/sequence','Version8 consumes every explicit declaration in sequence')
    add('missing-blockers','shivan-sentry','blockers',missing,'/play actor/sequence','/play actor/sequence','Version8 consumes every blocker declaration')
    add('missing-allocation','shivan-multiple','damage',missing,'/play actor/sequence','/play actor/sequence','Version8 consumes every explicit allocation in sequence')
    def extra(c,i):
        c['play'].insert(i+1,copy.deepcopy(c['play'][i]))
        for j,e in enumerate(c['play']):e['sequence']=j
    add('extra-declaration','shivan-sentry','attackers',extra,'/extra declaration','/unsupported priority callback','Version8 declarations cannot be replayed as priority commands')
    out['extra-declaration']['sequence']+=1
    add('extra-blockers','shivan-sentry','blockers',extra,'/wrong combat actor','/play actor/sequence','CR509.1 exactly one defending-player declaration callback')
    out['extra-blockers']['sequence']+=1
    add('extra-allocation','shivan-multiple','damage',extra,'/wrong combat actor','/play actor/sequence','CR510.2 no second assignment callback or response window')
    out['extra-allocation']['sequence']+=1
    add('stale-damage-source','shivan-multiple','damage',lambda c,i:c['play'][i]['selection'][0]['attacker'].update(incarnation=2),'/native command InvalidReference','/source incarnation','CR400.7 allocation source is the battlefield incarnation')
    def reordered(c,i):
        j=next(j for j,e in enumerate(c['play']) if e['kind']=='blockers')
        c['play'][i],c['play'][j]=c['play'][j],c['play'][i]
        for j,e in enumerate(c['play']):e['sequence']=j
    add('reordered-declarations','shivan-sentry','attackers',reordered,'/play position','/play actor/sequence','CR508 before CR509; no declaration reordering')
    def legacy(c,i):c['play'][i].pop('selection');c['play'][i]['kind']='legacy_blocker_order'
    add('legacy-order','shivan-sentry','attackers',legacy,'/legacy blocker-order callback','/legacy blocker-order callback','Modern CR509/510 removed damage assignment order')
    d=copy.deepcopy(doc);c=next(x for x in d['cases'] if x['id']=='pathwarden-haste');d['cases']=[c]
    start=next(i for i,e in enumerate(c['play']) if e['kind']=='activate')-1
    end=next(i for i,e in enumerate(c['play']) if e['kind']=='finish_activation')+3
    del c['play'][start:end]
    for i,e in enumerate(c['play']):e['sequence']=i
    seq=next(i for i,e in enumerate(c['play']) if e['kind']=='attackers')
    out['sick-pathwarden']=dict(input=d,sequence=seq,native=illegal,xmage='/selected attacker rejected',requirement='CR302.6/702.10: Pathwarden has no printed haste; same-turn attack is illegal before a real haste grant')
    # Move the identical legal Cavalry cast from its second turn to turn nine.
    # It has not begun another controller turn when asked to tap on turn ten.
    d=copy.deepcopy(doc);c=next(x for x in d['cases'] if x['id']=='pathwarden-haste');d['cases']=[c]
    start=next(i for i,e in enumerate(c['play']) if e['kind']=='cast' and e['source']==C)
    end=next(i for i in range(start,len(c['play'])) if c['play'][i]['kind']=='finish_payment')+3
    cast=c['play'][start:end];del c['play'][start:end]
    at=next(i for i,e in enumerate(c['play']) if e['turn']==9 and e['kind']=='play_land')+1
    for e in cast:e['turn']=9
    c['play'][at:at]=cast
    for i,e in enumerate(c['play']):e['sequence']=i
    seq=next(i for i,e in enumerate(c['play']) if e['kind']=='activate')
    out['sick-cavalry']=dict(input=d,sequence=seq,native=illegal,xmage='/selected activation not playable',requirement='CR302.6/602: Cavalry cannot pay its tap cost before its next controller turn')
    for name,edit,native,xmage in [
        ('missing-graveyard',lambda c,i:c['play'].pop(i),'/play actor/sequence','/play actor/sequence'),
        ('empty-graveyard',lambda c,i:c['play'][i].update(selection=[]),illegal,'/graveyard permutation'),
        ('duplicate-graveyard',lambda c,i:c['play'][i]['selection'].__setitem__(1,ref(R)),illegal,'/graveyard permutation'),
        ('foreign-graveyard',lambda c,i:c['play'][i]['selection'].__setitem__(1,ref(S)),illegal,'/graveyard permutation'),
        ('extra-graveyard-card',lambda c,i:c['play'][i]['selection'].append(ref(B)),illegal,'/graveyard permutation'),
        ('stale-graveyard',lambda c,i:c['play'][i]['selection'][0].update(incarnation=2),'/native command InvalidReference','/source incarnation'),
        ('wrong-graveyard-event',lambda c,i:c['play'][i].update(event='combat-sba/12'),'/graveyard event','/graveyard event'),
        ('wrong-graveyard-owner',lambda c,i:c['play'][i].update(actor=0),'/wrong combat actor','/play actor/sequence'),
    ]:add(name,'shivan-multiple','graveyard_order',edit,native,xmage,'CR101.4/404.3/704.3 exact event-bound owner permutation before simultaneous moves')
    add('extra-graveyard-choice','shivan-multiple','graveyard_order',extra,'/wrong combat actor','/play actor/sequence','CR404.3 consumes the owner choice exactly once')
    out['extra-graveyard-choice']['sequence']+=1
    return out

if __name__=='__main__':
    (ROOT/'fixtures/reference/full-pool-combat-negatives.json').write_text(json.dumps(negatives(fixtures()),indent=2)+'\n')
