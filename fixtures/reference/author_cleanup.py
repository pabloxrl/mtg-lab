"""CR 514/104/121 fixed no-cast draw/discard schedule; no engine output."""
import json
from pathlib import Path
from author_priority import author
ROOT=Path(__file__).resolve().parents[2]

def empty_game(starter, turns=68):
    c=author(starter,False);c['id']=f'empty-library-{starter}';out=[]
    def event(t,step,actor,kind,source=None):
        out.append(dict(sequence=len(out),turn=t,step=step,actor=actor,kind=kind,source=source,incarnation=1 if source else None,color=None,role=None,mode=None))
    for turn in range(1,turns+1):
        active=starter if turn%2 else 1-starter
        for step in ['upkeep']+(['draw'] if turn>1 else [])+['precombat_main','begin_combat','declare_attackers','end_combat','postcombat_main','end_turn']:
            if step=='declare_attackers':event(turn,step,active,'empty_attackers')
            event(turn,step,active,'pass');event(turn,step,1-active,'pass')
            if turn==68 and step=='upkeep':break
        if turn>1 and turn<68:
            # Retain the opening seven, discard each newly drawn physical card.
            draw_index=(turn//2-1 if active!=starter else (turn-1)//2-1)+7
            event(turn,'cleanup',active,'discard',c['chance'][active]['after'][draw_index])
    event(68,'draw',starter,'finish')
    c['play']=out;c['stop']='terminal'
    return c

if __name__=='__main__':
    doc=dict(schema_version=7,family='cleanup',pins=json.loads((ROOT/'fixtures/reference/full-pool-spells.json').read_text())['pins'],cases=[empty_game(0),empty_game(1)])
    (ROOT/'fixtures/reference/full-pool-cleanup.json').write_text(json.dumps(doc,indent=2)+'\n')

def stacked():
    c=author(0,False);c['id']='stacked-expiry';out=[]
    tops=[['0/mountain/0','0/mountain/1','0/mountain/2','0/dragon-fodder/0','0/goblin-surprise/0','0/mountain/3','0/mountain/4'],['1/forest/0','1/forest/1','1/forest/2','1/bear-cub/0','1/giant-growth/0','1/giant-growth/1','1/bite-down/0','1/forest/3']]
    for s in (0,1):c['chance'][s]['after']=tops[s]+[x for x in c['decks'][s]['occurrences'] if x not in tops[s]]
    def e(t,step,a,k,source=None,inc=None,color=None,role=None,mode=None):
        out.append(dict(sequence=len(out),turn=t,step=step,actor=a,kind=k,source=source,incarnation=inc,color=color,role=role,mode=mode))
    def passes(t,step,a):e(t,step,a,'pass');e(t,step,1-a,'pass')
    def cast(t,a,card,lands,targets=(),mode=None):
        step='precombat_main';e(t,step,a,'cast',card,1)
        if mode is not None:e(t,step,a,'mode',mode=mode)
        for source,inc,role in targets:e(t,step,a,'target',source,inc,role=role)
        if targets:e(t,step,a,'finish_targets')
        for land in lands:e(t,step,a,'tap_mana',land,2)
        for land in lands:e(t,step,a,'pay',color=3 if a==0 else 4)
        e(t,step,a,'finish_payment')
    for t in range(1,9):
        a=0 if t%2 else 1
        passes(t,'upkeep',a)
        if t>1:passes(t,'draw',a)
        land='mountain' if a==0 else 'forest';e(t,'precombat_main',a,'play_land',f'{a}/{land}/{(t-1)//2}',1)
        if t==3:
            cast(t,0,'0/dragon-fodder/0',['0/mountain/0','0/mountain/1']);passes(t,'precombat_main',0)
        if t==4:
            cast(t,1,'1/bear-cub/0',['1/forest/0','1/forest/1']);passes(t,'precombat_main',1)
        if t==8:
            for i in (0,1):
                cast(t,1,f'1/giant-growth/{i}',[f'1/forest/{i}'],[('token/0/0/0',0,'growth_target')]);passes(t,'precombat_main',1)
            cast(t,1,'1/bite-down/0',['1/forest/2','1/forest/3'],[('1/bear-cub/0',3,'bite_source'),('token/0/0/0',0,'bite_destination')])
            e(t,'precombat_main',1,'pass')
            cast(t,0,'0/goblin-surprise/0',['0/mountain/0','0/mountain/1','0/mountain/2'],mode=0)
            passes(t,'precombat_main',0);passes(t,'precombat_main',1)
        passes(t,'precombat_main',a);passes(t,'begin_combat',a);e(t,'declare_attackers',a,'empty_attackers');passes(t,'declare_attackers',a)
        for step in ('end_combat','postcombat_main','end_turn'):passes(t,step,a)
    e(9,'upkeep',0,'finish');c['play']=out;c['stop']='prefix';return c

def fixtures():
    cases=[empty_game(0),empty_game(1)]
    c=json.loads(json.dumps(cases[0]));c['id']='discard-prefix';c['stop']='prefix'
    end=next(i for i,e in enumerate(c['play']) if e['kind']=='discard')
    c['play']=c['play'][:end+1];finish=dict(c['play'][-1],sequence=end+1,turn=3,step='upkeep',actor=0,kind='finish',source=None,incarnation=None);c['play'].append(finish)
    concession=author(0,False);concession['id']='concession-only';concession['stop']='concession'
    concession['play']=[dict(sequence=i,turn=1,step='upkeep',actor=0,kind=k,source=None,incarnation=None,color=None,role=None,mode=None) for i,k in enumerate(('concede','finish'))]
    cases += [c,stacked(),concession]
    for c in cases:
        for i,e in enumerate(c['play']):
            if e['kind']=='discard':
                c['play'][i]={k:e[k] for k in ('sequence','turn','step','actor')}
                c['play'][i].update(kind='cleanup_discard',selection=[dict(source=e['source'],incarnation=e['incarnation'])])
    return dict(schema_version=7,family='cleanup',pins=json.loads((ROOT/'fixtures/reference/full-pool-spells.json').read_text())['pins'],cases=cases)

if __name__=='__main__':
    (ROOT/'fixtures/reference/full-pool-cleanup.json').write_text(json.dumps(fixtures(),indent=2)+'\n')

def negatives(doc):
    import copy
    prefix=next(c for c in doc['cases'] if c['id']=='discard-prefix');i=len(prefix['play'])-2;out={}
    def add(name,mutate,native,xmage,sequence=i):
        c=copy.deepcopy(prefix);mutate(c)
        for j,e in enumerate(c['play']):e['sequence']=j
        out[name]=dict(input=dict(doc,cases=[c]),sequence=sequence,native=native,xmage=xmage)
    add('missing_discard',lambda c:c['play'].__setitem__(i,dict(sequence=i,turn=2,step='cleanup',actor=1,kind='pass',source=None,incarnation=None,color=None,role=None,mode=None)),'/native command WrongKind','/missing cleanup discard')
    add('zero_cardinality',lambda c:c['play'][i].update(selection=[]),'/native command Policy(InvalidSelection)','/cleanup cardinality')
    add('extra_cardinality',lambda c:c['play'][i]['selection'].append(dict(source='1/forest/0',incarnation=1)),'/native command Policy(InvalidSelection)','/cleanup cardinality')
    add('wrong_actor',lambda c:c['play'][i].update(actor=0),'/native command Policy(WrongActor)','/play actor/sequence')
    add('wrong_incarnation',lambda c:c['play'][i]['selection'][0].update(incarnation=0),'/native command InvalidReference','/source incarnation')
    add('foreign_card',lambda c:c['play'][i]['selection'][0].update(source='0/mountain/0'),'/native command InvalidReference','/cleanup discard source')
    add('reordered_discards',lambda c:c['play'][i]['selection'][0].update(source=c['chance'][1]['after'][8]),'/native command InvalidReference','/source incarnation')
    add('extra_discard',lambda c:c['play'].insert(i+1,dict(copy.deepcopy(c['play'][i]),turn=3,step='upkeep',actor=0)),'/native command WrongKind','/unsupported priority callback',i+1)
    add('premature_terminal',lambda c:c.update(stop='terminal'),'/premature terminal','/premature terminal',i+1)
    add('omitted_checkpoint',lambda c:c['play'].pop(),'/omitted final checkpoint','/omitted final checkpoint',i+1)
    add('unused_suffix',lambda c:c['play'].append(copy.deepcopy(c['play'][-1])),'/unused tape suffix','/unused tape suffix',i+1)
    c=copy.deepcopy(next(c for c in doc['cases'] if c['id']=='concession-only'));c['stop']='terminal'
    out['concession_as_natural']=dict(input=dict(doc,cases=[c]),sequence=1,native='/concession completion classification',xmage='/concession completion classification')
    for name,suffix in [('terminal_omitted_checkpoint',False),('terminal_unused_suffix',True)]:
        c=copy.deepcopy(doc['cases'][0]);sequence=len(c['play'])-1
        if suffix:c['play'].append(dict(c['play'][-1],sequence=sequence+1))
        else:c['play'].pop()
        category='/unused tape suffix' if suffix else '/omitted final checkpoint'
        out[name]=dict(input=dict(doc,cases=[c]),sequence=sequence,native=category,xmage=category)
    return out

if __name__=='__main__':
    (ROOT/'fixtures/reference/full-pool-cleanup-negatives.json').write_text(json.dumps(negatives(fixtures()),indent=2)+'\n')
