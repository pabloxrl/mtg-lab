"""CR 602/605, 302.6, 702.10 schedules; no engine output is read.

Literal costs: Cavalry 1R, Elf G, Druid 1G, Cub 1G, Invoker 2GG,
Shivan 4RR. Activations: Cavalry tap/target, Shivan R/+1 power,
Invoker 8/target/+5/+5 trample. Empty combat is always explicit.
"""
import copy
import json
from pathlib import Path
from author_priority import author
ROOT=Path(__file__).resolve().parents[2]


def make(name):
    c=author(0,False);c['id']=name;c['play']=[]
    tops=[['0/mountain/'+str(i) for i in range(3)]+['0/axgard-cavalry/0','0/axgard-cavalry/1','0/shivan-dragon/0']+['0/mountain/'+str(i) for i in range(3,10)],
          ['1/forest/'+str(i) for i in range(3)]+['1/llanowar-elves/0','1/bear-cub/0','1/druid-of-the-cowl/0','1/wildheart-invoker/0']+['1/forest/3','1/forest/4','1/llanowar-elves/1','1/druid-of-the-cowl/1']+['1/forest/'+str(i) for i in range(5,10)]]
    # Green needs its sixth land by turn 12. For haste, the two newly cast
    # mana creatures are drawn by turn 10; all other cases draw lands first.
    if name!='haste':
        tops[1]=tops[1][:7]+['1/forest/'+str(i) for i in range(3,10)]+['1/llanowar-elves/1','1/druid-of-the-cowl/1']
    for seat,top in enumerate(tops):
        c['chance'][seat]['after']=top+[x for x in c['decks'][seat]['occurrences'] if x not in top]
    final=10 if name=='haste' else 12 if name.startswith('invoker') else 13
    def e(t,a,k,source=None,inc=None,color=None,step='precombat_main'):
        c['play'].append(dict(sequence=len(c['play']),turn=t,step=step,actor=a,kind=k,source=source,incarnation=inc,color=color))
    def passes(t,a,step='precombat_main'):e(t,a,'pass',step=step);e(t,1-a,'pass',step=step)
    def cast(t,a,card,cost):
        e(t,a,'cast',f'{a}/{card}',1)
        for i in range(cost):e(t,a,'tap_mana',f'{a}/'+('mountain' if a==0 else 'forest')+f'/{i}',2)
        for _ in range(cost):e(t,a,'pay',color=3 if a==0 else 4)
        e(t,a,'finish_payment');passes(t,a)
    def activate(t,a,source,target=None):
        e(t,a,'activate',source,3)
        if target:e(t,a,'activation_target',target,3)
    for t in range(1,final+1):
        a=0 if t%2 else 1;n=(t-1)//2
        passes(t,a,'upkeep')
        if t!=1:passes(t,a,'draw')
        e(t,a,'play_land',f'{a}/'+('mountain' if a==0 else 'forest')+f'/{n}',1)
        if t==2:cast(t,1,'llanowar-elves/0',1)
        if t==3:cast(t,0,'axgard-cavalry/0',2)
        if t==4:cast(t,1,'bear-cub/0',2)
        if t==5:cast(t,0,'axgard-cavalry/1',2)
        if t==6:cast(t,1,'druid-of-the-cowl/0',2)
        if t==8:cast(t,1,'wildheart-invoker/0',4)
        if t==11:cast(t,0,'shivan-dragon/0',6)
        if t==final:
            if name=='haste':
                # Three distinct lands pay the two spells; no default sources.
                for card,lands in [('llanowar-elves/1',[0]),('druid-of-the-cowl/1',[1,2])]:
                    e(t,1,'cast','1/'+card,1)
                    for i in lands:e(t,1,'tap_mana',f'1/forest/{i}',2)
                    for _ in lands:e(t,1,'pay',color=4)
                    e(t,1,'finish_payment');passes(t,1)
                for i,target in enumerate(['1/llanowar-elves/1','1/druid-of-the-cowl/1']):
                    e(t,1,'pass');activate(t,0,f'0/axgard-cavalry/{i}',target)
                    e(t,0,'finish_activation');passes(t,0)
                    e(t,1,'tap_mana',target,3)
            elif name.startswith('invoker'):
                sources=[(f'1/forest/{i}',2) for i in range(6)]+[('1/llanowar-elves/0',3),('1/druid-of-the-cowl/0',3)]
                if name.endswith('floating'):
                    for s,inc in sources:e(t,1,'tap_mana',s,inc)
                activate(t,1,'1/wildheart-invoker/0','1/bear-cub/0')
                if not name.endswith('floating'):
                    for s,inc in sources:e(t,1,'tap_mana',s,inc)
                for _ in range(8):e(t,1,'activation_pay',color=4)
                e(t,1,'finish_activation');passes(t,1)
            else:
                count=2 if name=='shivan-repeated' else 1
                if name=='shivan-floating':
                    for i in range(2):e(t,0,'tap_mana',f'0/mountain/{i}',2)
                for i in range(count):
                    activate(t,0,'0/shivan-dragon/0')
                    if name!='shivan-floating':e(t,0,'tap_mana',f'0/mountain/{i}',2)
                    e(t,0,'activation_pay',color=3);e(t,0,'finish_activation')
                for _ in range(count):passes(t,0)
            break
        passes(t,a)
        passes(t,a,'begin_combat');e(t,a,'empty_attackers',step='declare_attackers');passes(t,a,'declare_attackers')
        for step in ('end_combat','postcombat_main','end_turn'):passes(t,a,step)
    c['stop']='activations/'+name
    return c


def fixtures():
    return dict(schema_version=5,family='activations',pins=json.loads((ROOT/'fixtures/reference/full-pool-priority.json').read_text())['pins'],cases=[make(n) for n in ('shivan-single','shivan-repeated','shivan-floating','invoker-empty','invoker-floating','haste')])

if __name__=='__main__':
    (ROOT/'fixtures/reference/full-pool-activations.json').write_text(json.dumps(fixtures(),indent=2)+'\n')


def negatives(doc):
    out={}
    def add(name,case,kind,n,fields,category,requirement):
        d=copy.deepcopy(doc);c=next(x for x in d['cases'] if x['id']==case);d['cases']=[c]
        i=[i for i,e in enumerate(c['play']) if e['kind']==kind][n]
        c['play'][i].update(fields);c['play']=c['play'][:i+1]
        out[name]=dict(input=d,sequence=i,category=category,requirement=requirement)
    illegal='/native command Policy(InvalidSelection)'
    add('wrong_actor','shivan-single','activate',0,{'actor':1},'/native command Policy(WrongActor)','CR602: controller activates its source')
    add('wrong_color','shivan-single','activation_pay',0,{'color':4},illegal,'Oracle Shivan costs R')
    add('missing_mana_source','shivan-single','tap_mana',-1,{'kind':'activation_pay','source':None,'incarnation':None,'color':3},illegal,'CR602/605: no default mana source')
    add('enemy_mana','invoker-empty','tap_mana',-1,{'source':'0/mountain/0','incarnation':2},illegal,'CR605: actor controls the mana source')
    add('enemy_untapped_mana','shivan-floating','tap_mana',-2,{'source':'1/forest/0','incarnation':2},illegal,'CR605: an untapped opposing Forest is unusable solely because the actor does not control it')
    add('tapped_mana','invoker-empty','tap_mana',-1,{'source':'1/forest/0','incarnation':2},illegal,'CR605: source already reserved')
    add('departed_incarnation','invoker-empty','tap_mana',-1,{'incarnation':2},'/native command InvalidReference','CR400.7: past hand/stack incarnation is not the battlefield object')
    add('wrong_target','invoker-empty','activation_target',0,{'source':'1/forest/0','incarnation':2},illegal,'Oracle Invoker targets a creature')
    add('target_after_payment','invoker-empty','activation_target',0,{'kind':'activation_pay','source':None,'incarnation':None,'color':4},'/native command WrongKind','CR602.2b: target precedes payment')
    add('extra_priority','shivan-single','activation_pay',0,{'kind':'pass','color':None},'/native command WrongKind','CR605: no priority during mana payment')
    add('unpaid_commit','invoker-empty','activation_pay',-1,{'kind':'finish_activation','color':None},illegal,'Oracle Invoker requires eight mana, not seven')
    add('missing_target','invoker-empty','activation_target',0,{'kind':'finish_activation','source':None,'incarnation':None},illegal,'CR602: required target before commitment')
    add('unknown_source','shivan-single','activate',0,{'source':'0/shivan-dragon/99'},'/source unknown','Every selected source is a witnessed physical occurrence')
    d=copy.deepcopy(doc);c=next(c for c in d['cases'] if c['id']=='haste');d['cases']=[c]
    i=next(i for i,e in enumerate(c['play']) if e['kind']=='activate')-1
    c['play'][i].update(kind='tap_mana',source='1/llanowar-elves/1',incarnation=3)
    c['play']=c['play'][:i+1]
    out['sick_mana']=dict(input=d,sequence=i,category=illegal,requirement='CR302.6: own newly cast Elf cannot tap before haste resolves')
    d=copy.deepcopy(doc);c=next(c for c in d['cases'] if c['id']=='invoker-empty');d['cases']=[c]
    i=next(i for i,e in enumerate(c['play']) if e['turn']==10 and e['step']=='precombat_main' and e['kind']=='pass')
    c['play'][i].update(kind='activate',source='1/wildheart-invoker/0',incarnation=3)
    c['play']=c['play'][:i+1]
    out['insufficient_eight']=dict(input=d,sequence=i,category=illegal,requirement='Oracle Invoker costs eight; five Forests and two mana creatures provide only seven')
    return out

if __name__=='__main__':
    (ROOT/'fixtures/reference/full-pool-activations-native-negatives.json').write_text(json.dumps(negatives(fixtures()),indent=2)+'\n')


def cancellations(doc):
    out={}
    for case_name in ('shivan-single','invoker-empty','haste'):
        original=next(c for c in doc['cases'] if c['id']==case_name)
        start=next(i for i,e in enumerate(original['play']) if e['kind']=='activate')
        end=next(i for i in range(start,len(original['play'])) if original['play'][i]['kind']=='finish_activation')
        # Every native continuation boundary, including paid-but-uncommitted.
        for stage in range(start+1,end+1):
            d=copy.deepcopy(doc);c=copy.deepcopy(original);d['cases']=[c]
            cancel=copy.deepcopy(c['play'][stage]);cancel.update(kind='cancel_activation',source=None,incarnation=None,color=None,actor=c['play'][start]['actor'])
            c['play']=c['play'][:stage]+[cancel]+copy.deepcopy(c['play'][start:])
            for i,e in enumerate(c['play']):e['sequence']=i
            out[f'{case_name}-{stage-start}']=dict(input=d,cancel_sequence=stage,requirement='Native #254 private reservation cancellation restores the complete pre-announcement public state at every continuation stage')
    return out

if __name__=='__main__':
    (ROOT/'fixtures/reference/full-pool-activations-cancellations.json').write_text(json.dumps(cancellations(fixtures()),indent=2)+'\n')

if __name__=='__main__':
    reference=negatives(fixtures())
    categories={'wrong_actor':'/play actor/sequence','wrong_color':'/insufficient or wrong-color payment',
        'missing_mana_source':'/insufficient or wrong-color payment','enemy_mana':'/selected mana ability rejected',
        'enemy_untapped_mana':'/selected mana ability rejected','tapped_mana':'/selected mana ability rejected','departed_incarnation':'/source incarnation',
        'wrong_target':'/selected target not legal','target_after_payment':'/missing activation target',
        'extra_priority':'/missing activation payment','unpaid_commit':'/missing activation payment',
        'missing_target':'/missing activation target','unknown_source':'/source unknown',
        'sick_mana':'/selected mana ability rejected','insufficient_eight':'/selected activation not playable'}
    for name,spec in reference.items():spec['category']=categories[name]
    (ROOT/'fixtures/reference/full-pool-activations-xmage-negatives.json').write_text(json.dumps(reference,indent=2)+'\n')

if __name__=='__main__':
    reference_cancels={}
    for name,spec in cancellations(fixtures()).items():
        c=spec['input']['cases'][0];stage=spec['cancel_sequence']
        # The canceled original instruction is immediately before the retry
        # suffix; recover it from the authored original and stage offset.
        original=next(x for x in fixtures()['cases'] if x['id']==c['id'])
        if original['play'][stage]['kind'] in ('activation_target','tap_mana','activation_pay'):
            reference_cancels[name]=spec
    (ROOT/'fixtures/reference/full-pool-activations-xmage-cancellations.json').write_text(json.dumps(reference_cancels,indent=2)+'\n')
