"""Version 10 full-game admission. Inputs/oracles are frozen before comparison.

Privileged per-game artifacts are streamed separately. No policy runs in replay;
no engine result is promoted to an expected checkpoint.
"""
import copy
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

import full_pool_reference as base
import mulligan_reference
import triggers_reference
import admission_observations
from instant_reference import difference

ROOT=base.ROOT
FIXTURE=ROOT/'fixtures/reference/full-pool-admission.json'
ORACLE=ROOT/'fixtures/reference/full-pool-admission-expectations.json'
MANIFEST=ROOT/'fixtures/reference/full-pool-admission-manifest.json'
FIELDS=set('activation active actor allocations battlefield boundary combat creations departures effects exile graveyard hand incarnations land_plays library life mana outcome payment pending_triggers permanents stack step targeting trigger_boundary turn'.split())
EVENT_FIELDS=set('sequence turn step actor kind source incarnation color role mode order player'.split())
SELECTED={'attackers','blockers','damage','graveyard_order','cleanup_discard'}
KINDS=set('pass play_land cast tap_mana pay finish_payment target finish_targets mode discard target_player activate activation_target activation_pay finish_activation empty_attackers order_triggers finish'.split())|SELECTED


def require(ok,path):
    if not ok:raise ValueError('first divergence: '+path)


def digest(value):
    return hashlib.sha256(json.dumps(value,sort_keys=True,separators=(',',':')).encode()).hexdigest()


def reference_case(case):
    """CR 601.2h permits either order of mana and additional discard costs.

    As in the delivered spells family, XMage requests Thrill's discard after
    mana. Preserve every supplied choice, with an explicit index permutation;
    never fill a callback or derive a choice from observed engine state.
    """
    result=copy.deepcopy(case)
    indexed=list(enumerate(result['play']))
    for item in list(indexed):
        if item[1]['kind']!='discard':continue
        pos=indexed.index(item)
        start=next(j for j in range(pos-1,-1,-1) if indexed[j][1]['kind']=='cast')
        require('/thrill-of-possibility/' in indexed[start][1]['source'],'/discard transaction source')
        indexed.remove(item)
        finish=next(j for j in range(start+1,len(indexed)) if indexed[j][1]['kind']=='finish_payment')
        indexed.insert(finish,item)
    for j,(_,event) in enumerate(indexed):event['sequence']=j
    result['play']=[event for _,event in indexed]
    return result,[i for i,_ in indexed]


def validate(doc,quota=True):
    require(type(doc) is dict and set(doc)=={'schema_version','family','pins','cases','provenance'},'/envelope fields')
    require(type(doc['schema_version']) is int and doc['schema_version']==10 and doc['family']=='admission','/version/family')
    require(isinstance(doc['cases'],list) and doc['cases'],'/cases')
    projection=dict(schema_version=2,family='mulligan',pins=doc['pins'],cases=[])
    seen=set();content=set();rows=set()
    for c in doc['cases']:
        require(type(c) is dict and isinstance(c.get('id'),str),'/case id')
        required={'id','starter','decks','chance','choices','play','stop'}
        if c['id'].endswith('-heuristic'):required|={'native_records','native_indices'}
        require(set(c)==required,'/case fields')
        require(c['id'] not in seen,'/duplicate game id');seen.add(c['id'])
        require(c['stop']=='terminal','/nonterminal quota')
        identity=digest({k:c[k] for k in ('starter','decks','chance','choices','play')})
        require(identity not in content,'/duplicate game content');content.add(identity)
        category=c['id'].rsplit('-',1)[-1];require(category in ('heuristic','directed'),'/origin')
        row=(''.join(d['deck'][0].upper() for d in c['decks']),c['starter'],category);require(row not in rows,'/duplicate row/origin');rows.add(row)
        projection['cases'].append({**{k:c[k] for k in ('id','starter','decks','chance','choices')},'stop':'first_upkeep'})
        require(isinstance(c['play'],list) and c['play'],'/empty play')
        for i,e in enumerate(c['play']):
            kind=e.get('kind');expected=EVENT_FIELDS|({'selection'} if kind in SELECTED else set())|({'event'} if kind=='graveyard_order' else set())
            require(set(e)==expected,f'/play/{i}/fields')
            require(kind in KINDS,f'/play/{i}/kind')
            require(type(e['sequence']) is int and e['sequence']==i,f'/play/{i}/sequence')
            require(type(e['actor']) is int and e['actor'] in (0,1),f'/play/{i}/actor')
            require(type(e['turn']) is int and e['turn']>0,f'/play/{i}/turn')
            object_kind=kind in {'cast','play_land','tap_mana','target','discard','activate','activation_target'}
            require((isinstance(e['source'],str) and type(e['incarnation']) is int and e['incarnation']>=0) if object_kind else e['source'] is None and e['incarnation'] is None,f'/play/{i}/source/incarnation')
            for field,required in [('color',kind in ('pay','activation_pay')),('role',kind=='target'),('mode',kind=='mode'),('order',kind=='order_triggers'),('player',kind=='target_player')]:
                require((e[field] is not None)==required,f'/play/{i}/{field}')
            if e['color'] is not None:require(type(e['color']) is int and 0<=e['color']<6,f'/play/{i}/payment color')
            if e['mode'] is not None:require(type(e['mode']) is int and e['mode'] in (0,1),f'/play/{i}/mode')
            if e['player'] is not None:require(type(e['player']) is int and e['player'] in (0,1),f'/play/{i}/target player')
            require((kind=='finish')==(i==len(c['play'])-1),f'/play/{i}/finish')
        if category=='heuristic':
            require(len(c['native_indices'])==len(c['play']) and c['native_records'],'/native source chronology')
    mulligan_reference.validate(projection)
    if quota:
        require(rows=={(row,seat,kind) for row in ('RG','GR','RR','GG') for seat in (0,1) for kind in ('heuristic','directed')},'/sixteen distinct row/origin quotas')


def check_manifest(doc):
    manifest=json.loads(MANIFEST.read_text());oracles=json.loads(ORACLE.read_text())
    require(set(manifest['cases'])=={c['id'] for c in doc['cases']},'/manifest membership')
    for c in doc['cases']:
        m=manifest['cases'][c['id']]
        require(m['input_sha256']==digest(c) and m['oracle_sha256']==digest(oracles[c['id']]),'/frozen input/oracle hash')
        require(m['provenance']['kind']==c['id'].rsplit('-',1)[-1],'/origin provenance')
    require(set(manifest['played_cards'])==set(admission_observations.CARDS)-{'goblin-token'},'/twenty played card behaviors')
    return manifest


def check_negatives(folder,engine):
    specs=json.loads((ROOT/f'fixtures/reference/full-pool-admission-{engine}-negatives.json').read_text())
    actual=json.loads((Path(folder)/f'{engine}-negatives.json').read_text())
    require(set(actual)==set(specs),'/real negative coverage')
    for name,spec in specs.items():
        r=actual[name];require('first divergence:' in r['error'],'/unclassified negative '+name)
        if spec['sequence'] is not None:
            require(r['run']['consumed_play']==spec['case']['play'][:spec['sequence']],'/negative first-divergence consumption '+name)
    return actual


def semantic(point):
    p=triggers_reference.semantic(point)
    for a in p['stack']:
        a.pop('raw_id',None)
    p['combat']=point['combat'];p['allocations']=point['allocations'];p['outcome']=point['outcome']
    # CR 113.7: a stack ability has its own identity, not a physical card's
    # zone-change history. Compare its witnessed identity/source in `stack`;
    # validate native's additional arena incarnation separately below.
    p['incarnations']={k:v for k,v in p['incarnations'].items() if not k.startswith('ability/')}
    return p


def check_game(run,case,engine):
    points=run.get('points')
    require(isinstance(points,list) and len(points)==len(case['play']) and points,'/checkpoint stream length')
    require(run.get('consumed_play')==case['play'],'/consumed play')
    opening=run.get('opening');require(isinstance(opening,dict),'/opening')
    require(set(opening)=={'points','consumed_chance','consumed_choices','raw_callbacks','native_rng' if engine=='native' else 'reference_rng'},'/opening fields')
    for field in ('chance','choices'):require(opening.get('consumed_'+field)==case[field],'/consumed '+field)
    # CR103: this frozen initial set explicitly keeps seven. The retained London
    # family separately exercises repeated mulligans/bottoming in both engines.
    require(len(case['chance'])==2 and all(e['selection']=='keep' for e in case['choices']),'/initial admission keep contract')
    expected_opening=[]
    for boundary in ('declaration','first_upkeep'):
        for seat in (case['starter'],1-case['starter']):
            order=case['chance'][seat]['after']
            expected_opening.append(dict(actor=seat,boundary=boundary,hand=sorted(order[:7]),library=order[7:],life=[20,20],mulligans=0))
    require(opening['points']==expected_opening,'/opening literal checkpoints')
    require(opening['raw_callbacks'],'/opening callback provenance')
    for i,(p,e) in enumerate(zip(points,case['play'])):
        require(isinstance(p,dict) and set(p)==FIELDS,f'/checkpoint/{i}/fields')
        require(p['boundary']==('finish' if e['kind']=='finish' else f'before/{i}'),f'/checkpoint/{i}/boundary')
        require((p['turn'],p['step'])==(e['turn'],e['step']),f'/checkpoint/{i}/position')
        require(type(p['life']) is list and len(p['life'])==2 and all(type(v) is int for v in p['life']),f'/checkpoint/{i}/life')
        require((p['outcome'] is not None)==(e['kind']=='finish'),f'/checkpoint/{i}/premature outcome')
        ability_ids={s['object'] for s in p['stack'] if 'object' in s}
        arena_ids={k:v for k,v in p['incarnations'].items() if k.startswith('ability/')}
        # Native objects start at incarnation zero (objects::create), and a
        # newly created stack ability has not crossed a zone boundary.
        if engine=='native':require(arena_ids=={k:0 for k in ability_ids},f'/checkpoint/{i}/ability arena identity')
        else:require(not arena_ids,f'/checkpoint/{i}/unexpected ability card incarnation')
        if e['kind']!='finish':require(p['actor']==e['actor'],f'/checkpoint/{i}/actor')
        expected=[]
        if e['kind']=='order_triggers' and (engine=='native' or len(e['order'])>1):expected=[dict(key=k,controller=e['actor']) for k in e['order']]
        elif e['kind']=='target_player' and engine=='native':
            require(i>0 and case['play'][i-1]['kind']=='order_triggers','/target before order')
            expected=[dict(key=k,controller=e['actor']) for k in case['play'][i-1]['order']]
        elif engine=='xmage' and e['kind']=='finish_payment' and i+1<len(points) and case['play'][i+1]['kind']=='order_triggers':
            expected=[dict(key=k,controller=case['play'][i+1]['actor']) for k in case['play'][i+1]['order']]
        canonical=lambda v:sorted(v,key=lambda a:json.dumps(a,sort_keys=True))
        require(canonical(p['pending_triggers'])==canonical(expected),f'/checkpoint/{i}/pending source/event set')
        for effect in p['effects']:
            if engine=='xmage':require(effect.get('duration')=='EndOfTurn',f'/checkpoint/{i}/effect duration')
            else:require(set(effect)=={'object','boost','power_boost','damage'},f'/checkpoint/{i}/effect fields')
    admission_observations.check(run,case,engine)
    marked=run.get('marked');require(isinstance(marked,list),'/missing simultaneous damage observations')
    combat_turns=[e['turn'] for e in case['play'] if e['kind']=='attackers']
    require([p['turn'] for p in marked]==combat_turns,'/simultaneous damage chronology')
    for p in marked:
        require(set(p)==FIELDS and p['boundary']=='simultaneous-before-sba' and p['step']=='combat_damage' and p['actor'] is None and p['outcome'] is None,'/simultaneous damage fields/boundary')
        end=max(e['sequence'] for e in case['play'] if e['turn']==p['turn'])
        auxiliary(p,run,case,engine,end,'/simultaneous damage')
    last=points[-1];outcome=last['outcome']
    require(type(outcome) is dict and set(outcome)=={'winner','losses'},'/outcome fields')
    require(type(outcome['winner']) is int and outcome['winner'] in (0,1),'/winner type')
    require(len(outcome['losses'])==2 and outcome['losses'][outcome['winner']] is None,'/winner loss')
    loser=1-outcome['winner'];reason=outcome['losses'][loser]
    require(reason in ('Life','EmptyDraw'),'/concession/timeout quota')
    if reason=='Life':require(last['life'][loser]<=0 and last['life'][outcome['winner']]>0,'/CR704.5a life outcome')
    else:require(not last['library'][loser] and last['step']=='draw','/CR121 empty draw')
    expected=json.loads(ORACLE.read_text())[case['id']]
    if case['id'].endswith('directed'):
        for field in ('life','outcome','turn','step','library'):require(last[field]==expected[field],'/authored terminal/'+field)
        require([p['turn'] for p in run['cleanup_checkpoints']]==expected['cleanup_turns'],'/cleanup chronology')
        hands=last['hand'] if engine=='native' else run['before_departure']['hand']
        require([sorted(h) for h in hands]==[sorted(h) for h in expected['terminal_hand']],'/authored terminal hand ledger')
        for seat in (0,1):
            for kind,field in [('cast','spells_per_seat'),('play_land','land_plays_per_seat')]:
                require(sum(e['kind']==kind and e['actor']==seat for e in case['play'])==expected[field],'/authored played behavior count')
    else:
        first=semantic(points[expected['before_sequence']])
        for field in ('life','stack','battlefield_membership','hand_membership','library','incarnations'):
            require(first[field]==expected[field],'/authored first cast/'+field)
    for p in run.get('cleanup_checkpoints',[]):
        require(set(p)==FIELDS and p['boundary']=='cleanup-settled' and p['step']=='cleanup','/cleanup fields')
        require(not p['effects'] and all(v['damage'] in (None,0) for v in p['permanents'].values()),'/cleanup expiry/damage')
        require(all(len(h)<=7 for h in p['hand']),'/cleanup hand size')
        end=max(e['sequence'] for e in case['play'] if e['turn']==p['turn'])
        auxiliary(p,run,case,engine,end,'/cleanup')
    if engine=='native':
        capture=run.get('policy_capture');require(isinstance(capture,dict),'/native capture')
        footer=capture.get('footer');require(footer is not None and footer['complete'] is True and footer['end']=='Completed','/truncated capture')
        require(run['records'] and run['stale_candidates_rejected']==len(run['records']),'/native real application')
        if case['id'].endswith('heuristic'):
            require(run['records']==[r for r in case['native_records'] if r['decision']!='keep_or_mulligan'],'/exact frozen heuristic choices')
        rewards=[-1,-1];rewards[outcome['winner']]=1;require(footer['returns']==rewards,'/outcome-derived reward')
    else:
        require(run.get('before_departure') is not None and set(run['before_departure'])==FIELDS,'/missing pre-departure state/fields')
        auxiliary(run['before_departure'],run,case,engine,len(case['play'])-1,'/pre-departure')
        admission_observations.xmage_trigger_provenance(run,case)
        admission_observations.xmage_effect_lifecycle(run,case)
        bindings=run.get('raw_occurrence_bindings');require(isinstance(bindings,dict),'/physical occurrence provenance')
        originals=[v['occurrence'] for v in bindings.values() if not v['occurrence'].startswith('token/')]
        require(sorted(originals)==sorted(o for d in case['decks'] for o in d['occurrences']),'/physical occurrence provenance')


def auxiliary(point,run,case,engine,index,path):
    """Audit raw auxiliary fields before semantic projection removes identifiers.

    Creation/departure histories are cumulative real-engine witnesses. Their raw
    entries must agree with the independently validated primary observation
    stream, including fields that intentionally differ between engines.
    """
    admission_observations.structure(point,case,engine,index,path)
    for field in ('creations','departures'):
        history=run['points'][-1][field]
        require(point[field]==history[:len(point[field])],path+'/'+field+' raw history')
    require(point['payment'] is None and point['activation'] is None and point['targeting'] is None,path+'/unfinished transaction')


def compare_game(native,reference,case):
    projected,indices=reference_case(case)
    check_game(native,case,'native');check_game(reference,projected,'xmage')
    aligned=[None]*len(indices)
    for p,i in zip(reference['points'],indices):aligned[i]=p
    compared=[];staging=[]
    for i,(left,right) in enumerate(zip(native['points'],aligned)):
        if i==len(native['points'])-1:
            # XMage removes losing-player objects after LOST; preserve and compare
            # its actual earlier event separately from the finalized winner.
            right=copy.deepcopy(reference['before_departure']);right['outcome']=reference['points'][-1]['outcome'];right['actor']=None
        if any(p['payment'] is not None or p['activation'] is not None or p['targeting'] is not None for p in (left,right)):
            staging.append(i);continue
        a,b=semantic(left),semantic(right)
        if case['play'][i]['kind']=='target_player':
            keys=case['play'][i-1]['order'];require(len(keys)==1 and b['stack'],'/targeted trigger announcement')
            announced=b['stack'][-1]
            require(announced==dict(ability='trigger',key=keys[0],controller=case['play'][i]['actor'],target=None,source_card=keys[0]['source'].split('/')[1]),'/targeted trigger preselection')
            # XMage announces the witnessed stack ability before choosing its
            # target; native retains that same key in the pending-trigger set.
            b['stack'].pop()
        if (left['trigger_boundary']=='order' and len(left['pending_triggers'])==1) or case['play'][i]['kind']=='target_player':
            # Exact engine-specific pending sets were independently checked above.
            a.pop('pending_triggers');b.pop('pending_triggers')
        diff=difference(a,b)
        require(diff is None,f'/committed/{case["id"]}/{i} {diff}')
        compared.append(i)
    require(compared,'/no nonempty committed observations')
    require(len(native['cleanup_checkpoints'])==len(reference['cleanup_checkpoints']),'/cleanup count')
    for i,(a,b) in enumerate(zip(native['cleanup_checkpoints'],reference['cleanup_checkpoints'])):
        diff=difference(semantic(a),semantic(b));require(diff is None,f'/cleanup/{i} {diff}')
    require(len(native['marked'])==len(reference['marked']),'/simultaneous damage count')
    for i,(a,b) in enumerate(zip(native['marked'],reference['marked'])):
        diff=difference(semantic(a),semantic(b));require(diff is None,f'/simultaneous damage/{i} {diff}')
    return dict(compared=compared,separate_staging=staging)


def native(folder):
    folder=Path(folder).resolve();folder.mkdir(parents=True,exist_ok=True)
    env=dict(os.environ,MTG_ADMISSION_OUTPUT=str(folder))
    command=['cargo','test','-p','mtg-core','--locked','--lib','game::full_pool_reference_tests::full_pool_admission_literal_checkpoints','--','--exact']
    fd=env.get('MTG_SYMPHONY_LOCK_FD');fds=()
    if fd is not None:
        try:os.fstat(int(fd));fds=(int(fd),)
        except OSError:fd=None
    if fd is None:command=[sys.executable,str(ROOT/'scripts/symphony/resource_lock.py'),'heavy','--']+command
    with (folder/'native.log').open('w') as log:
        subprocess.run(command,cwd=ROOT,env=env,stdin=subprocess.DEVNULL,stdout=log,stderr=subprocess.STDOUT,timeout=3600,check=True,pass_fds=fds)
    index=json.loads((folder/'native-index.json').read_text());doc=json.loads(FIXTURE.read_text());validate(doc)
    require(set(index)=={c['id'] for c in doc['cases']},'/native case coverage')
    for case in doc['cases']:
        require(len(index[case['id']])==2,'/native repeats')
        for name in index[case['id']]:check_game(json.loads((folder/name).read_text()),case,'native')
    check_manifest(doc);check_negatives(folder,'native')
    return index


def comparator_controls(run,case,engine):
    """Fault injection against actual observations, never replacement positives."""
    controls={}
    def reject(name,mutate):
        bad=copy.deepcopy(run);mutate(bad)
        try:check_game(bad,case,engine)
        except ValueError as error:controls[name]=str(error)
        else:raise ValueError('undetected observation mutation: '+name)
    for field in FIELDS:
        reject('missing_'+field,lambda r,f=field:r['points'][-1].pop(f))
    reject('missing_checkpoint',lambda r:r['points'].pop())
    reject('extra_checkpoint',lambda r:r['points'].append(copy.deepcopy(r['points'][-1])))
    reject('reordered_checkpoint',lambda r:r['points'].__setitem__(slice(0,2),list(reversed(r['points'][:2]))))
    reject('consumed_action',lambda r:r['consumed_play'].pop())
    reject('consumed_chance',lambda r:r['opening']['consumed_chance'].pop())
    reject('payment_pool',lambda r:next(p for p in r['points'] if p['payment'])['payment'].pop('pool'))
    def swap_duplicate(r):
        row=r['points'][0]['library'][0]
        a,b=next((a,b) for a in range(len(row)) for b in range(a+1,len(row)) if row[a].split('/')[1]==row[b].split('/')[1])
        row[a],row[b]=row[b],row[a]
    reject('same_name_identity',swap_duplicate)
    reject('missing_opening_checkpoint',lambda r:r['opening']['points'].pop())
    reject('missing_damage_checkpoint',lambda r:r['marked'].pop())
    reject('reordered_damage_checkpoint',lambda r:r['marked'].reverse())
    reject('missing_damage_field',lambda r:r['marked'][0].pop('permanents'))
    reject('missing_cleanup_permanent_field',lambda r:next(iter(next(p for p in r['cleanup_checkpoints'] if p['permanents'])['permanents'].values())).pop('flying'))
    if engine=='xmage':
        reject('missing_predeparture_permanent_field',lambda r:next(iter(r['before_departure']['permanents'].values())).pop('flying'))
    for reason in ('Concession','Timeout'):
        reject(reason,lambda r,v=reason:r['points'][-1]['outcome']['losses'].__setitem__(1-r['points'][-1]['outcome']['winner'],v))
    reject('forged_winner',lambda r:r['points'][-1]['outcome'].__setitem__('winner',1-r['points'][-1]['outcome']['winner']))
    return controls


def coverage(runs,doc):
    manifest=check_manifest(doc);observed={}
    observed={card:[] for card in manifest['played_cards']}
    for case in doc['cases']:
        r=runs(case['id'])
        for card,witnesses in manifest['played_cards'].items():
            for w in witnesses:
                if w['case']!=case['id']:continue
                e=r['consumed_play'][w['sequence']]
                require(e['kind']==w['kind'] and e['source']==w['source'],'/played behavior input witness')
                # Mere deck membership or an uncommitted announcement cannot count.
                resolved=any(w['source'] in p['battlefield'] or any(w['source'] in g for g in p['graveyard']) for p in r['points'][w['sequence']+1:])
                if resolved:observed[card].append(w)
    for card,witnesses in observed.items():require(witnesses,'/unplayed card behavior '+card)
    return observed


def provenance_controls(run,case):
    """Independent review reproductions against fresh XMage observations."""
    controls={}
    def reject(name,mutate):
        bad=copy.deepcopy(run);mutate(bad)
        try:check_game(bad,case,'xmage')
        except ValueError as error:controls[name]=str(error)
        else:raise ValueError('first divergence: /accepted provenance corruption '+name)
    def all_points(r):return r['points']+r['marked']+r['cleanup_checkpoints']+[r['before_departure']]
    def omit_effects(r):
        for p in all_points(r):p['effects']=[]
    def expire(r):
        before=next(p for p in r['points'] if p['effects'])
        after=next(p for p in r['points'] if p['turn']>before['turn'] and not p['effects'])
        after['effects']=copy.deepcopy(before['effects'])
    def rebind(r,source=False):
        for trace in r['trigger_provenance']:
            trace['raw_source_zcc']+=1
            if source:trace['raw_source']='00000000-0000-0000-0000-000000000000'
        for p in all_points(r):
            for ability in p['stack']:
                if ability.get('ability')=='trigger':
                    ability['raw_source_zcc']+=1
                    if source:ability['raw_source']='00000000-0000-0000-0000-000000000000'
    reject('missing_active_effects',omit_effects)
    reject('expired_effect_reappears',expire)
    reject('missing_trigger_provenance',lambda r:r.pop('trigger_provenance'))
    reject('rebound_trigger_source',lambda r:rebind(r,True))
    reject('rebound_trigger_counter',rebind)
    return controls


def run(args):
    import shutil
    import xmage
    output=args.output.resolve();output.parent.mkdir(parents=True,exist_ok=True)
    # A repeated invocation always gets a fresh attempt directory. Failed runs
    # and prior receipts remain named and retained, including partial artifacts.
    folder=Path(tempfile.mkdtemp(prefix=output.stem+'.privileged-',dir=output.parent));os.chmod(folder,0o700)
    if output.exists():
        previous=folder/'previous-receipt.json';previous.write_bytes(output.read_bytes())
    sources=sorted(set([*ROOT.glob('crates/mtg-core/src/*.rs'),*ROOT.glob('crates/mtg-policy/src/*.rs'),*ROOT.glob('fixtures/reference/*admission*'),*ROOT.glob('references/xmage/*'),*ROOT.glob('scripts/*reference.py'),ROOT/'scripts/admission_observations.py',ROOT/'scripts/xmage.py',ROOT/'scripts/scenario.py',ROOT/'Cargo.lock',ROOT/'Cargo.toml',ROOT/'crates/mtg-core/Cargo.toml',ROOT/'tests/test_m2_repair_admission.py']))
    sources=[p for p in sources if p.is_file()];hashes={str(p.relative_to(ROOT)):xmage.sha(p) for p in sources}
    def write(name,value):(folder/name).write_text(json.dumps(value,indent=2)+'\n')
    def progress(current,next):
        subprocess.run([sys.executable,str(ROOT/'scripts/symphony/report_progress.py'),'--issue','277','--current',current,'--why','I’m checking that complete toy-deck games and invalid replays have independently verifiable evidence.','--next',next],cwd=ROOT,check=True,timeout=30)
    fd=os.environ.get('MTG_SYMPHONY_LOCK_FD');fds=()
    if fd is not None:
        try:os.fstat(int(fd));fds=(int(fd),)
        except OSError:pass
    receipt=dict(schema_version=10,family='admission',status='running',privileged_directory=folder.name,sources=hashes)
    output.write_text(json.dumps(receipt,indent=2)+'\n')
    try:
        doc=json.loads(FIXTURE.read_text());validate(doc);manifest=check_manifest(doc)
        for path in (FIXTURE,ORACLE,MANIFEST):(folder/path.name).write_bytes(path.read_bytes())
        for engine in ('native','xmage'):
            path=ROOT/f'fixtures/reference/full-pool-admission-{engine}-negatives.json';(folder/path.name).write_bytes(path.read_bytes())
        projected=copy.deepcopy(doc);maps={}
        for i,c in enumerate(doc['cases']):projected['cases'][i],maps[c['id']]=reference_case(c)
        write('xmage-input.json',projected);write('reference-cost-order.json',maps)
        write('input-oracle-freeze.json',dict(inputs=xmage.sha(FIXTURE),oracle=xmage.sha(ORACLE),manifest=xmage.sha(MANIFEST),reference_input=xmage.sha(folder/'xmage-input.json'),sources=hashes))
        progress('I’m replaying all sixteen frozen games twice in the native engine and checking invalid tapes.','I’ll run the same inputs through pinned XMage and compare the observed checkpoints.')
        native_index=native(folder)
        cache=args.cache.resolve();xmage.verify_inputs(cache)
        require(xmage.dependencies(cache)==json.loads((ROOT/'references/xmage/dependencies.json').read_text()),'/dependencies')
        for family in ('mulligan','priority','spells','triggers','activations','played-combat','admission'):
            for name,sha in json.loads((ROOT/f'references/xmage/{family}-provenance.json').read_text())['consulted_sources'].items():
                require(xmage.sha(cache/xmage.SOURCE/name)==sha,'/upstream source '+name)
        for name in ('FullPoolMulliganTest.java','FullPoolPriorityTest.java','FullPoolTriggersTest.java','FullPoolAdmissionTest.java','CombatSbaGame.java'):
            target=cache/xmage.SOURCE/'Mage.Tests/src/test/java/org/mage/test/mtglab'/name;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes((ROOT/'references/xmage'/name).read_bytes())
        progress('The native admission replays and invalid-tape checks passed. I’m running pinned XMage twice per game.','I’ll compare every committed checkpoint and verify the separately observed transaction stages.')
        command=xmage.maven(cache)+['-o','-pl','Mage.Tests','-am','test','-Dtest=org.mage.test.mtglab.FullPoolAdmissionTest','-Dsurefire.failIfNoSpecifiedTests=false','-Dmtglab.fixture='+str(folder/'xmage-input.json'),'-Dmtglab.root='+str(ROOT),'-Dmtglab.output='+str(folder/'xmage-index.json'),'-DargLine=-Djava.awt.headless=true']
        xmage.bounded(command,cache/xmage.SOURCE,xmage.environment(cache),folder/'xmage.log',600)
        index=json.loads((folder/'xmage-index.json').read_text());require(set(index)==set(native_index),'/reference membership')
        check_negatives(folder,'xmage')
        agreement={};controls={};counts={}
        for c in doc['cases']:
            name=c['id'];require(len(index[name])==2,'/reference repeats');pairs=[]
            for repeat in (0,1):
                n=json.loads((folder/native_index[name][repeat]).read_text());r=json.loads((folder/index[name][repeat]).read_text())
                pairs.append((n,r));agreement[f'{name}/{repeat+1}']=compare_game(n,r,c)
            for engine,j in [('native',0),('xmage',1)]:
                a,b=pairs[0][j],pairs[1][j]
                require([semantic(p) for p in a['points']]==[semantic(p) for p in b['points']],'/repeat observations/'+engine+'/'+name)
                require(a['consumed_play']==b['consumed_play'] and a['opening']['consumed_chance']==b['opening']['consumed_chance'],'/repeat consumption')
            counts[name]=len(pairs[0][0]['points'])
            if name==doc['cases'][0]['id']:
                controls['native']=comparator_controls(pairs[0][0],c,'native');controls['xmage']=comparator_controls(pairs[0][1],reference_case(c)[0],'xmage')
            if name=='GR-040-RG-S0-heuristic':controls['xmage_provenance']=provenance_controls(pairs[0][1],reference_case(c)[0])
        write('agreement.json',agreement);write('comparator-controls.json',controls)
        played=coverage(lambda name:json.loads((folder/native_index[name][0]).read_text()),doc);write('played-behavior-witnesses.json',played)
        # Retain all existing reviewed families. These are actual engine runs,
        # not copied receipts, because the shared observer gained extension hooks.
        compatibility={}
        for family in ('reset','mulligan','priority','spells','activations','triggers','cleanup','combat'):
            progress(f'The terminal-game comparisons passed. I’m rerunning the existing {family} reference family.','I’ll preserve its receipt and continue the compatibility checks before reporting admission.')
            path=folder/f'compatibility-{family}.json'
            with (folder/f'compatibility-{family}.log').open('w') as log:
                subprocess.run([sys.executable,str(ROOT/'scripts/full_pool_reference.py'),'--family',family,'--cache',str(cache),'--output',str(path)],cwd=ROOT,stdin=subprocess.DEVNULL,stdout=log,stderr=subprocess.STDOUT,check=True,timeout=3600,pass_fds=fds)
            prior=json.loads(path.read_text());require(prior['status']=='agreed','/compatibility '+family);compatibility[family]=xmage.sha(path)
        toolchain={}
        for name,command in [('java',['java','-version']),('maven',['mvn','-version']),('rustc',['rustc','--version']),('python',[sys.executable,'--version'])]:
            p=subprocess.run(command,capture_output=True,text=True,timeout=30,check=True,stdin=subprocess.DEVNULL)
            toolchain[name]=dict(version=p.stdout+p.stderr,executable_sha256=xmage.sha(Path(shutil.which(command[0])).resolve()))
        write('toolchain.json',toolchain)
        require(hashes=={str(p.relative_to(ROOT)):xmage.sha(p) for p in sources},'/source changed during execution')
        receipt.update(status='agreed',cases=16,runs_per_engine=32,completion='rules_terminal',checkpoints_per_game=counts,played_cards=sorted(played),compatibility_receipts=compatibility,
            pins=doc['pins'],upstream_commit=json.loads((ROOT/'references/xmage/pins.json').read_text())['upstream_commit'],reference_semantics='patched-combat-sba-v1',reference_integration_sha256=xmage.sha(ROOT/'references/xmage/CombatSbaGame.java'),bridge_sha256=xmage.sha(ROOT/'references/xmage/FullPoolAdmissionTest.java'),
            acceptance_claims=dict(selected_action_legality=True,complete_cross_engine_legal_set=False,raw_rejection='native production decode/apply rejection and stale-submission rollback; XMage selected-action legality queries, engine acceptance and explicit callback constraints, not a blanket raw-rejection equivalence claim',transaction_fields='independent pinned-card cost/choice ledgers and complete transaction footprints; raw observations retained',oracle='authored first-cast and terminal checkpoints, rules-derived invariants and independent review'),
            limits='Initial two-engine GR-040 subset only. Patched XMage, no Forge qualification, reference-AI generation, campaign/corpus or M2 verdict. Native privacy/capture/snapshot discovery remains mandatory.',stdin='closed',display='unset',offline=True)
        progress('The terminal-game admission and all retained reference families passed this run.','I’ll retain the receipts and complete full discovery, torture and independent review before delivery.')
    except Exception as error:
        receipt.update(status='failed',first_divergence=str(error));write('failure.json',dict(error=str(error)))
        raise
    finally:
        for p in folder.rglob('*'):
            if p.is_file():os.chmod(p,0o600)
            elif p.is_dir():os.chmod(p,0o700)
        receipt['privileged_artifacts']={str(p.relative_to(folder)):xmage.sha(p) for p in folder.rglob('*') if p.is_file()}
        output.write_text(json.dumps(receipt,indent=2)+'\n')
    print('Sixteen distinct rules-terminal games agreed twice per engine; strict invalid tapes and retained families passed.')
