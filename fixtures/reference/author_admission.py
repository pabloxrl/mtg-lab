"""Independent CR/card-derived directed full games; no engine output is read.

Each player casts a 2/2 on its second turn. On its third turn red casts
Viashino Pyromancer (ETB: two to opponent); green casts Druid of the Cowl.
On its fourth turn red selects Surprise's +2/+0 mode; green casts Growth
on its own Cub. Both expire at cleanup. Thereafter both explicitly pass,
retaining their remaining opening cards and discarding newest draws over seven.
No extra draws occur: the nonstarter loses on the turn-68 empty draw.
"""
import json
from pathlib import Path

ROOT=Path(__file__).resolve().parents[2]
MANIFEST=json.loads((ROOT/'data/cards/foundations_micro_v1.json').read_text())


def author(names,starter):
    decks=[];chance=[];hands=[];libraries=[];play=[];life=[20,20];trigger_event=0
    for s,name in enumerate(names):
        d=next(d for d in MANIFEST['decks'] if d['id']==name)
        ids=[f'{s}/{c["card_id"]}/{i}' for c in d['cards'] for i in range(c['copies'])]
        land='mountain' if name=='red' else 'forest'
        vanilla='swab-goblin' if name=='red' else 'bear-cub'
        second='viashino-pyromancer' if name=='red' else 'druid-of-the-cowl'
        spell='goblin-surprise' if name=='red' else 'giant-growth'
        top=[f'{s}/{land}/{i}' for i in range(3)]+[f'{s}/{vanilla}/0',f'{s}/{second}/0',f'{s}/{spell}/0',f'{s}/{land}/3']
        order=top+[x for x in ids if x not in top]
        decks.append(dict(deck=name,occurrences=ids))
        chance.append(dict(sequence=s,kind='initial_shuffle',actor=s,source=None,before=ids,after=order))
        hands.append(order[:7]);libraries.append(order[7:])
    opening=[dict(sequence=i,kind='declare',actor=s,source=None,round=0,selection='keep') for i,s in enumerate((starter,1-starter))]
    def e(t,step,a,k,**fields):
        v=dict(sequence=len(play),turn=t,step=step,actor=a,kind=k,source=None,incarnation=None,color=None,role=None,mode=None,order=None,player=None)
        v.update(fields);play.append(v)
    def pair(t,step,a):e(t,step,a,'pass');e(t,step,1-a,'pass')
    for t in range(1,69):
        a=starter if t%2 else 1-starter;n=(t+1)//2
        pair(t,'upkeep',a)
        if t==68:
            e(t,'draw',starter,'finish');break
        if t>1:
            hands[a].append(libraries[a].pop(0));pair(t,'draw',a)
        name=names[a];land='mountain' if name=='red' else 'forest';color=3 if name=='red' else 4
        if n<=4:
            source=f'{a}/{land}/{n-1}';hands[a].remove(source)
            e(t,'precombat_main',a,'play_land',source=source,incarnation=1)
        if n in (2,3,4):
            card=('swab-goblin' if name=='red' else 'bear-cub') if n==2 else ('viashino-pyromancer' if name=='red' else 'druid-of-the-cowl') if n==3 else ('goblin-surprise' if name=='red' else 'giant-growth')
            source=f'{a}/{card}/0';hands[a].remove(source)
            e(t,'precombat_main',a,'cast',source=source,incarnation=1)
            cost=2 if n<4 else 3 if name=='red' else 1
            if n==4:
                if name=='red':e(t,'precombat_main',a,'mode',mode=0)
                else:
                    e(t,'precombat_main',a,'target',source=f'{a}/bear-cub/0',incarnation=3,role='growth_target')
                    e(t,'precombat_main',a,'finish_targets')
            for i in range(cost):e(t,'precombat_main',a,'tap_mana',source=f'{a}/{land}/{i}',incarnation=2)
            for _ in range(cost):e(t,'precombat_main',a,'pay',color=color)
            e(t,'precombat_main',a,'finish_payment');pair(t,'precombat_main',a)
            if n==3 and name=='red':
                e(t,'precombat_main',a,'order_triggers',order=[dict(source=source,incarnation=3,ability='pyromancer',event=trigger_event)])
                trigger_event+=1
                e(t,'precombat_main',a,'target_player',player=1-a)
                pair(t,'precombat_main',a);life[1-a]-=2
        pair(t,'precombat_main',a);pair(t,'begin_combat',a)
        e(t,'declare_attackers',a,'empty_attackers');pair(t,'declare_attackers',a)
        for step in ('end_combat','postcombat_main','end_turn'):pair(t,step,a)
        if len(hands[a])>7:
            source=hands[a].pop()
            e(t,'cleanup',a,'cleanup_discard',selection=[dict(source=source,incarnation=1)])
    row=''.join(name[0].upper() for name in names)
    id=f'GR-040-{row}-S{starter}-directed'
    losses=[None,None];losses[1-starter]='EmptyDraw'
    oracle=dict(basis=['CR103.8a','CR121.4','CR104.3c','CR514.2','pinned Viashino Pyromancer, Goblin Surprise and Giant Growth'],
                life=life,outcome=dict(winner=starter,losses=losses),turn=68,step='draw',library=[[],[]],
                cleanup_turns=list(range(1,68)),spells_per_seat=3,land_plays_per_seat=4,
                terminal_hand=hands)
    return dict(id=id,starter=starter,decks=decks,chance=chance,choices=opening,play=play,stop='terminal'),oracle


if __name__=='__main__':
    import argparse
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('destination',type=Path);args=parser.parse_args()
    cases=[];oracle={}
    for names in [('red','green'),('green','red'),('red','red'),('green','green')]:
        for starter in (0,1):
            case,expected=author(names,starter);cases.append(case);oracle[case['id']]=expected
    pins=json.loads((ROOT/'fixtures/reference/full-pool-combat.json').read_text())['pins']
    args.destination.write_text(json.dumps(dict(schema_version=10,family='admission',pins=pins,cases=cases,oracle=oracle),indent=2)+'\n')


def heuristic_oracle(case):
    """Derive a literal pre-first-cast checkpoint from INPUTS, never observations.

    No spell or ability has resolved at this boundary. CR103.8/121 draws and
    the explicitly played lands completely determine these fields.
    """
    first=next(e for e in case['play'] if e['kind']=='cast')
    prefix=case['play'][:first['sequence']]
    if any(e['kind'] not in ('pass','play_land','empty_attackers') for e in prefix):
        raise ValueError('first-cast oracle requires no prior consequential effect')
    lands=[e['source'] for e in prefix if e['kind']=='play_land']
    turn=first['turn'];hands=[];libraries=[];incarnations={}
    for seat in (0,1):
        draws=(turn-1)//2 if seat==case['starter'] else turn//2
        order=case['chance'][seat]['after'];dealt=order[:7+draws]
        hands.append(sorted(x for x in dealt if x not in lands));libraries.append(order[7+draws:])
        for source in order:incarnations[source]=2 if source in lands else 1 if source in dealt else 0
    return dict(basis=['CR103.8a','CR121.2','CR305.2','CR400.7'],before_sequence=first['sequence'],
                life=[20,20],stack=[],battlefield_membership=sorted(lands),hand_membership=hands,
                library=libraries,incarnations=incarnations,
                terminal_predicate='CR704.5a: losers exactly seats with observed life <= 0; sole survivor is winner; no concession or truncation')
