"""GH-272 authored schedules. No engine execution/output supplies expectations.

CR 601.2: modes/targets, mana and additional costs before stack commitment;
608.2: last-in-first-out resolution and target revalidation; 111/704: tokens.
"""
import copy
import json
from pathlib import Path
from author_priority import author
ROOT = Path(__file__).resolve().parents[2]


def make(name, red_spell, final_turn, mode=None):
    c = author(0, False)
    c['id'] = name
    def swap(seat, a, b):
        row = c['chance'][seat]['after']
        i, j = row.index(a), row.index(b)
        row[i], row[j] = row[j], row[i]
    swap(0, '0/swab-goblin/0', '0/'+red_spell+'/0')
    if mode is not None:
        swap(0, '0/swab-goblin/1', '0/goblin-surprise/0')
    if name in ('growth-bite', 'departed-target'):
        swap(1, '1/bear-cub/1', '1/bite-down/0')
        swap(1, '1/forest/3', '1/giant-growth/0')
    out = []
    def event(t, s, actor, kind, source=None, incarnation=None, color=None, role=None, mode=None):
        e = dict(sequence=len(out), turn=t, step=s, actor=actor, kind=kind,
                 source=source, incarnation=incarnation, color=color, role=role, mode=mode)
        out.append(e)
    def cast(t, actor, card, lands, targets=(), discard=None, chosen_mode=None):
        s='precombat_main'
        event(t,s,actor,'cast',card,1)
        if chosen_mode is not None: event(t,s,actor,'mode',mode=chosen_mode)
        for target, incarnation, role in targets:
            event(t,s,actor,'target',target,incarnation,role=role)
        if targets: event(t,s,actor,'finish_targets')
        if discard: event(t,s,actor,'discard',discard,1)
        for land in lands: event(t,s,actor,'tap_mana',land,2)
        for _ in lands: event(t,s,actor,'pay',color=3 if actor==0 else 4)
        event(t,s,actor,'finish_payment')
    def resolve(t, actor):
        event(t,'precombat_main',actor,'pass')
        event(t,'precombat_main',1-actor,'pass')
    for turn in range(1,final_turn+1):
        # Reuse only the explicitly authored empty-step chronology, omitting
        # prior creature casts and payments; no runtime legality is consulted.
        events=[e for e in c['play'] if e['turn']==turn]
        first_main=next(i for i,e in enumerate(events) if e['kind']=='play_land')
        for e in events[:first_main+1]:
            event(**dict(t=e['turn'],s=e['step'],actor=e['actor'],kind=e['kind'],source=e['source'],incarnation=e['incarnation'],color=e['color']))
        if turn==3:
            cast(turn,0,'0/'+red_spell+'/0',['0/mountain/0','0/mountain/1'],
                 discard='0/mountain/2' if red_spell=='thrill-of-possibility' else None)
            resolve(turn,0)
        if turn==4:
            cast(turn,1,'1/bear-cub/0',['1/forest/0','1/forest/1'])
            resolve(turn,1)
        if turn==5 and mode is not None:
            cast(turn,0,'0/goblin-surprise/0',[f'0/mountain/{i}' for i in range(3)],chosen_mode=mode)
            resolve(turn,0)
        if turn==6:
            if name=='departed-target':
                cast(turn,1,'1/giant-growth/0',['1/forest/2'],targets=[('token/0/0/1',0,'growth_target')])
            cast(turn,1,'1/bite-down/0',['1/forest/0','1/forest/1'],
                 targets=[('1/bear-cub/0',3,'bite_source'),('token/0/0/1',0,'bite_destination')])
            if name=='growth-bite':
                cast(turn,1,'1/giant-growth/0',['1/forest/2'],targets=[('1/bear-cub/0',3,'growth_target')])
            resolve(turn,1)
            resolve(turn,1)
        if turn==final_turn: break
        # Two passes to leave main and the fixed empty combat/end sequence.
        tail=next(i for i,e in enumerate(events) if e['step']=='begin_combat')
        event(turn,'precombat_main',turn%2==0,'pass')
        event(turn,'precombat_main',turn%2!=0,'pass')
        for e in events[tail:]:
            event(e['turn'],e['step'],e['actor'],e['kind'],e['source'],e['incarnation'],e['color'])
    for e in out: e['actor']=int(e['actor'])
    c['play']=out
    last='1/giant-growth/0' if name=='departed-target' else '1/bite-down/0' if final_turn==6 else '0/goblin-surprise/0' if mode is not None else '0/'+red_spell+'/0'
    c['stop']='resolved/'+last
    return c


def fixtures():
    return dict(schema_version=4,family='spells',pins=json.loads((ROOT/'fixtures/reference/full-pool-priority.json').read_text())['pins'],cases=[
        make('fodder','dragon-fodder',3),
        make('thrill','thrill-of-possibility',3),
        make('surprise-boost','dragon-fodder',5,0),
        make('surprise-tokens','dragon-fodder',5,1),
        make('growth-bite','dragon-fodder',6),
        make('departed-target','dragon-fodder',6)])

if __name__=='__main__':
    doc=fixtures()
    (ROOT/'fixtures/reference/full-pool-spells.json').write_text(json.dumps(doc,indent=2)+'\n')
    reference=copy.deepcopy(doc)
    for case in reference['cases']:
        play=case['play']
        discards=[e for e in play if e['kind']=='discard']
        for discard in discards:
            play.remove(discard)
            start=next(i for i,e in enumerate(play) if e['kind']=='cast' and e['source']=='0/thrill-of-possibility/0')
            finish=next(i for i in range(start,len(play)) if play[i]['kind']=='finish_payment')
            play.insert(finish,discard)
        for i,e in enumerate(play): e['sequence']=i
    (ROOT/'fixtures/reference/full-pool-spells-xmage-input.json').write_text(json.dumps(reference,indent=2)+'\n')


def negative_cases(doc, engine):
    out={}
    def add(name,case_name,kind,occurrence,change,reason):
        d=copy.deepcopy(doc);c=next(c for c in d['cases'] if c['id']==case_name);d['cases']=[c]
        matches=[i for i,e in enumerate(c['play']) if e['kind']==kind]
        i=matches[occurrence];change(c['play'],i)
        for n,e in enumerate(c['play']): e['sequence']=n
        out[name]={'input':d,'sequence':i,'requirement':reason}
    def replace(kind,**fields):
        def f(p,i):
            e=p[i];e.update(kind=kind,source=None,incarnation=None,color=None,role=None,mode=None);e.update(fields)
        return f
    add('wrong_target_role','growth-bite','target',0,lambda p,i:p[i].update(role='bite_destination'),'CR601.2c: own source target precedes opposing destination')
    add('wrong_source_controller','growth-bite','target',0,lambda p,i:p[i].update(source='token/0/0/0',incarnation=0),'Bite own creature target')
    add('wrong_destination_controller','growth-bite','target',1,lambda p,i:p[i].update(source='1/bear-cub/0',incarnation=3),'Bite opposing creature target')
    add('departed_incarnation','growth-bite','target',0,lambda p,i:p[i].update(incarnation=2),'CR400.7: former stack incarnation cannot designate battlefield incarnation')
    add('missing_target','growth-bite','target',0,replace('finish_targets'),'CR601.2c: both targets required')
    add('disordered_targets','growth-bite','target',0,lambda p,i:p.__setitem__(slice(i,i+2),list(reversed(p[i:i+2]))),'CR601.2c: role and sequence required')
    add('unknown_token','growth-bite','target',1,lambda p,i:p[i].update(source='token/0/0/2'),'CR111: only actually created physical tokens exist')
    add('duplicate_token_selection','growth-bite','target',0,lambda p,i:p[i].update(source='token/0/0/1',incarnation=0),'An opposing token cannot also fill the own-creature target role')
    add('illegal_mode','surprise-boost','mode',0,lambda p,i:p[i].update(mode=2),'Oracle/CR700.2: exactly one of two modes')
    add('missing_discard','thrill','discard',0,replace('finish_payment'),'CR601.2h: additional cost required before commit')
    add('discard_spell_itself','thrill','discard',0,lambda p,i:p[i].update(source='0/thrill-of-possibility/0',incarnation=2 if engine=='xmage' else 1),'CR601: cannot discard the announced spell as its own cost')
    add('wrong_discard_controller','thrill','discard',0,lambda p,i:p[i].update(source='1/forest/2'),'CR601.2h: discard from caster hand')
    add('unpaid_commit','fodder','pay',0,replace('finish_payment'),'CR601.2h: unpaid spell cannot commit')
    add('insufficient_resources','fodder','tap_mana',1,replace('pay',color=3),'CR601.2h: one tapped source cannot pay two mana')
    # The first pay consumes the one available red; the following original pay
    # is the intended failing step in this control.
    out['insufficient_resources']['sequence']+=1
    add('extra_callback','fodder','finish_payment',0,replace('mode',mode=0),'Nonmodal Fodder has no mode callback')
    add('cancel_then_commit','fodder','pay',0,replace('cancel_payment'),'Cancelled action never commits')
    # Cancellation itself is a valid native action; the next attempted pay is rejected.
    out['cancel_then_commit']['sequence']+=1
    add('truncated_tape','fodder','pass',-1,lambda p,i:p.__delitem__(slice(i,None)),'CR117/608: missing resolution pass cannot count as completion')
    add('extra_tape','fodder','pass',-1,lambda p,i:p.append(copy.deepcopy(p[i])),'Named stop rejects every unconsumed action')
    # Categories are the authored adapter rejection contract: native semantic
    # action kind/reference/mask errors; reference target/mode/cost callbacks.
    native={
        'wrong_target_role':'/native command WrongKind',
        'wrong_source_controller':'/native command Policy(InvalidSelection)',
        'wrong_destination_controller':'/native command Policy(InvalidSelection)',
        'departed_incarnation':'/native command InvalidReference',
        'missing_target':'/native command WrongKind',
        'disordered_targets':'/native command WrongKind',
        'unknown_token':'/source unknown',
        'duplicate_token_selection':'/native command Policy(InvalidSelection)',
        'illegal_mode':'/native command Policy(InvalidSelection)',
        'missing_discard':'/native command WrongKind',
        'discard_spell_itself':'/native command Policy(InvalidSelection)',
        'wrong_discard_controller':'/native command InvalidReference',
        'unpaid_commit':'/native command Policy(InvalidSelection)',
        'insufficient_resources':'/native command Policy(InvalidSelection)',
        'extra_callback':'/native command WrongKind',
        'cancel_then_commit':'/native command WrongKind',
        'truncated_tape':'/missing choice before named stop',
        'extra_tape':'/extra choice after named stop'}
    reference={
        'wrong_target_role':'/target role',
        'wrong_source_controller':'/selected target not legal',
        'wrong_destination_controller':'/selected target not legal',
        'departed_incarnation':'/source incarnation',
        'missing_target':'/missing target/discard choice',
        'disordered_targets':'/target role',
        'unknown_token':'/source unknown',
        'duplicate_token_selection':'/selected target not legal',
        'illegal_mode':'/illegal mode',
        'missing_discard':'/missing target/discard choice',
        'discard_spell_itself':'/discard source',
        'wrong_discard_controller':'/discard source',
        'unpaid_commit':'/unsupported payment callback',
        'insufficient_resources':'/insufficient or wrong-color payment',
        'extra_callback':'/missing commit acknowledgement',
        'cancel_then_commit':'/selected cast rejected',
        'truncated_tape':'/missing choice before named stop',
        'extra_tape':'/extra choice after named stop'}
    for name,spec in out.items():
        spec['category']=(native if engine=='native' else reference)[name]
        if name not in ('truncated_tape','extra_tape'):
            spec['input']['cases'][0]['play']=spec['input']['cases'][0]['play'][:spec['sequence']+1]
    return out

if __name__=='__main__':
    for engine,name in [('native','full-pool-spells.json'),('xmage','full-pool-spells-xmage-input.json')]:
        d=json.loads((ROOT/'fixtures/reference'/name).read_text())
        (ROOT/f'fixtures/reference/full-pool-spells-{engine}-negatives.json').write_text(json.dumps(negative_cases(d,engine),indent=2)+'\n')


def final_ledger(case):
    """Fixed independently counted zone ledger at the specified main-phase stop."""
    name=case['id'];turn=case['play'][-1]['turn']
    red_turns=(turn+1)//2;green_turns=turn//2
    played=[['0/mountain/'+str(i) for i in range(red_turns)],['1/forest/'+str(i) for i in range(green_turns)]]
    graves=[[],[]]
    red_spell='thrill-of-possibility' if name=='thrill' else 'dragon-fodder'
    graves[0]=['0/'+red_spell+'/0']
    if name=='thrill':graves[0]=['0/mountain/2']+graves[0]
    if name.startswith('surprise'):graves[0].append('0/goblin-surprise/0')
    if turn>=4:played[1].append('1/bear-cub/0')
    if name=='growth-bite':graves[1]=['1/giant-growth/0','1/bite-down/0']
    if name=='departed-target':graves[1]=['1/bite-down/0','1/giant-growth/0']
    hands=[];libraries=[]
    for seat,draws in [(0,red_turns-1+(2 if name=='thrill' else 0)),(1,green_turns)]:
        order=case['chance'][seat]['after']
        hands.append([x for x in order[:7+draws] if x not in played[seat]+graves[seat]])
        libraries.append(order[7+draws:])
    creatures={}
    if name!='thrill':
        for i in range(2):creatures[f'token/0/0/{i}']={'power':3 if name=='surprise-boost' else 1,'toughness':1,'owner':0,'controller':0,'card':'goblin-token'}
    if name=='surprise-tokens':
        for i in range(2):creatures[f'token/0/1/{i}']={'power':1,'toughness':1,'owner':0,'controller':0,'card':'goblin-token'}
    if turn>=4:creatures['1/bear-cub/0']={'power':5 if name=='growth-bite' else 2,'toughness':5 if name=='growth-bite' else 2,'owner':1,'controller':1,'card':'bear-cub'}
    if turn==6:del creatures['token/0/0/1']
    return dict(turn=turn,step='precombat_main',active=int(turn%2==0),actor=int(turn%2==0),life=[20,20],mana=[[0]*6,[0]*6],land_plays=1,
                hand_membership=[sorted(h) for h in hands],library=libraries,graveyard=graves,exile=[],stack=[],payment=None,targeting=None,
                battlefield_membership=sorted(played[0]+played[1]+[t for t in creatures if t.startswith('token/')]),creatures=creatures)

if __name__=='__main__':
    (ROOT/'fixtures/reference/full-pool-spells-final.json').write_text(json.dumps({c['id']:final_ledger(c) for c in fixtures()['cases']},indent=2)+'\n')
