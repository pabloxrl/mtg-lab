"""Version 6 real played cast/ETB trigger adapter; privileged test evidence."""
import copy
import json
from pathlib import Path
import full_pool_reference as base
ROOT=base.ROOT
FIXTURE=ROOT/'fixtures/reference/full-pool-triggers.json'
REFERENCE_FIXTURE=ROOT/'fixtures/reference/full-pool-triggers-xmage-input.json'
BRIDGE=ROOT/'references/xmage/FullPoolTriggersTest.java'
require=base.require

def negative_inputs(doc):
    return json.loads((ROOT/'fixtures/reference/full-pool-triggers-native-negatives.json').read_text())

def native(folder):
    return base.native(folder,'triggers')

def compare(points):
    require(set(points)=={'archer-cyclops','duplicate-archers','pyromancer-bite'},'/cases')
    for name,rows in points.items():
        require(bool(rows),'/empty observations')
        p=rows[-1]
        require(p['stack']==[] and p['pending_triggers']==[] and p['trigger_boundary']=='settled','/unsettled final')
        require(p['life']==[20,{'archer-cyclops':19,'duplicate-archers':16,'pyromancer-bite':18}[name]],'/literal trigger damage/'+name)
        if name=='archer-cyclops':require(p['permanents']['0/crackling-cyclops/0']['power']==3,'/Cyclops +3')
        if name=='pyromancer-bite':
            require('0/viashino-pyromancer/0' in p['graveyard'][0],'/source died to played Bite')
            require(p['incarnations']['0/viashino-pyromancer/0']==4,'/departed source incarnation')


def validate(doc):
    import priority_reference
    require(type(doc.get('schema_version')) is int and doc['schema_version']==6 and doc.get('family')=='triggers','/version/family')
    p=copy.deepcopy(doc);p.update(schema_version=3,family='priority')
    for c in p['cases']:
        require(c['stop'].startswith('triggers_settled/'),'/stop')
        c['stop']='second_creature_resolved'
        for e in c['play']:
            require(set(e)=={'sequence','turn','step','actor','kind','source','incarnation','color','role','mode','order','player'},'/choice fields')
            for k in ('role','mode','order','player'):del e[k]
    priority_reference.validate(p)

def semantic(p):
    p=copy.deepcopy(p)
    out={f:p[f] for f in ('turn','step','active','actor','life','mana','land_plays','library','graveyard','exile','stack','incarnations','permanents','trigger_boundary','pending_triggers')}
    out['pending_triggers'].sort(key=lambda a:json.dumps(a,sort_keys=True))
    for a in out['stack']:
        a.pop('raw_birth',None);a.pop('raw_uuid',None);a.pop('raw_source_handle',None);a.pop('raw_source',None);a.pop('raw_source_zcc',None)
    out['hand_membership']=[sorted(h) for h in p['hand']]
    out['battlefield_membership']=sorted(p['battlefield'])
    out['creation_order']=[b['id'] for b in p['creations']]
    return out

def specs(engine):
    return json.loads((ROOT/f'fixtures/reference/full-pool-triggers-{engine}-negatives.json').read_text())

def check_run(result,doc,engine):
    compare(result['checkpoints'])
    names={c['id'] for c in doc['cases']}
    require(set(result['runs'])==names and set(result['repeat_runs'])==names,'/run coverage')
    fields={'boundary','turn','step','active','actor','life','mana','land_plays','hand','library','graveyard','exile','battlefield','stack','incarnations','permanents','payment','targeting','effects','creations','departures','pending_triggers','trigger_boundary'}
    for c in doc['cases']:
        name=c['id'];r=result['runs'][name];points=r['points'];play=c['play']
        require(points==result['checkpoints'][name] and len(points)==len(play)+1,'/complete observations/'+name)
        require(r['consumed_play']==play,'/actual choices/'+name)
        for f in ('chance','choices'):require(r['opening']['consumed_'+f]==c[f],'/actual opening/'+f)
        require(points[-1]['boundary']==c['stop'],'/named stop')
        first_order=next(i for i,e in enumerate(play) if e['kind']=='order_triggers')
        keys=[]
        for i,p in enumerate(points):
            require(set(p)==fields,'/missing/extra observation/'+name)
            if i<len(play):
                e=play[i]
                require(p['boundary']==f'before/{i}' and (p['turn'],p['step'],p['actor'])==(e['turn'],e['step'],e['actor']),'/choice alignment/'+name+'/'+str(i))
            # Validate every boundary from the supplied choices, including late
            # ordinary checkpoints and final settlement. These are adapter
            # staging differences, not inferred trigger creation.
            e=play[i] if i<len(play) else None
            kind=e['kind'] if e else None
            expected_boundary=('order' if kind=='order_triggers' else
                               'target' if kind=='target_player' else
                               None if engine=='native' and (p['payment'] is not None or p['targeting'] is not None) else 'settled')
            require(p['trigger_boundary']==expected_boundary,'/checkpoint trigger boundary/'+name+'/'+str(i))
            expected_pending=[]
            if kind=='order_triggers' and (engine=='native' or len(e['order'])>1):
                expected_pending=[{'key':k,'controller':e['actor']} for k in e['order']]
            elif kind=='target_player' and engine=='native':
                require(i>0 and play[i-1]['kind']=='order_triggers','/target order predecessor')
                expected_pending=[{'key':k,'controller':e['actor']} for k in play[i-1]['order']]
            elif engine=='xmage' and kind=='finish_payment' and i+1<len(play) and play[i+1]['kind']=='order_triggers':
                expected_pending=[{'key':k,'controller':play[i+1]['actor']} for k in play[i+1]['order']]
            canonical=lambda rows:sorted(rows,key=lambda a:json.dumps(a,sort_keys=True))
            require(canonical(p['pending_triggers'])==canonical(expected_pending),'/checkpoint pending source/event set/'+name+'/'+str(i))
            if i<first_order:
                require(not any(a.get('ability')=='trigger' for a in p['stack']),'/trigger before event')
            for a in p['stack']:
                if a.get('ability')=='trigger':
                    k=a['key'];require(set(k)=={'source','incarnation','ability','event'} and k['incarnation']==3,'/trigger source incarnation')
                    require(k in keys,'/unknown source/ability/event')
                    require(a['source_card']==k['source'].split('/')[1],'/source LKI card')
                    if engine=='native':
                        witness=next((w for w in r['trigger_provenance'] if w['key']==k),None)
                        require(witness is not None and a['raw_source_handle']==witness['raw_source_handle'],'/raw source rebound')
                    else:
                        witness=next((w for w in r['trigger_provenance'] if all(w[f]==k[f] for f in k)),None)
                        require(witness is not None and a['raw_source']==witness['raw_source'] and a['raw_source_zcc']==witness['raw_source_zcc'],'/raw source rebound')
                    if k['ability']=='pyromancer' and p['trigger_boundary']!='target':require(a['target']==1,'/ETB player target')
            if i==len(play):continue
            e=play[i]
            if e['kind']=='order_triggers':
                order=e['order'];keys.extend(order)
                require(p['trigger_boundary']=='order','/pending/settled boundary')
                pending=[a['key'] for a in p['pending_triggers']]
                if engine=='native' or len(order)>1:require(sorted(pending,key=lambda k:json.dumps(k,sort_keys=True))==sorted(order,key=lambda k:json.dumps(k,sort_keys=True)),'/pending source/event set')
                after=points[i+1]
                if name!='pyromancer-bite':
                    require([a['key'] for a in after['stack'] if a.get('ability')=='trigger']==order,'/trigger stack order')
                    require(after['trigger_boundary']=='settled' and after['pending_triggers']==[],'/placement settlement')
                    require(after['life']==p['life'],'/damage before trigger resolution')
                    # The noncreature spell remains under both abilities.
                    require(after['stack'][0]['source'].startswith('0/dragon-fodder/'),'/cast trigger before spell resolution')
            if e['kind']=='target_player':
                require(p['trigger_boundary']=='target' and '0/viashino-pyromancer/0' in p['permanents'],'/ETB timing')
                if engine=='native':require(len(p['pending_triggers'])==1 and p['stack']==[],'/native target pending')
                else:require(len(p['stack'])==1 and p['stack'][0]['target'] is None,'/reference target placement')
                require(points[i+1]['life']==[20,20] and points[i+1]['stack'][-1]['target']==1,'/ETB target before damage')
        require(len(keys)==(4 if name=='duplicate-archers' else 1 if name=='pyromancer-bite' else 2),'/trigger event cardinality')
        require(len({json.dumps(k,sort_keys=True) for k in keys})==len(keys),'/duplicate trigger identity')
        if name=='duplicate-archers':require({k['event'] for k in keys}=={0,1},'/distinct cast events')
        if name=='pyromancer-bite':
            departed=[p for p in points if '0/viashino-pyromancer/0' in p['graveyard'][0] and p['stack']]
            require(bool(departed),'/witnessed source death with ability on stack')
            for p in departed:
                require(p['life']==[20,20] and p['stack'][0]['key']==keys[0] and p['incarnations']['0/viashino-pyromancer/0']==4,'/source identity rebound after death')
        if engine=='native':
            require(r['records'] and r['policy_capture'] and r['stale_candidates_rejected']==len(r['records']),'/real scalar records')
            raw=r['trigger_provenance'];require(len(raw)==len(keys),'/native raw trigger provenance')
            for w in raw:
                require(w['key'] in keys and w['raw_source_identity'][1]==w['key']['incarnation'] and set(w['raw_source_handle'])=={'store','epoch','generation','slot'},'/native source incarnation provenance')
                require(play[w['after_choice']]['kind'] in ('finish_payment','pass'),'/native event boundary provenance')
        else:
            raw=r['trigger_provenance']
            require(len(raw)==len(keys) and all(v['raw_ability'] and v['raw_source'] and v['ability_class'] and type(v['raw_source_zcc']) is int for v in raw),'/witnessed trigger provenance')
            require([{f:v[f] for f in ('source','incarnation','ability','event')} for v in raw]==sorted(keys,key=lambda k:(k['event'],k['source'])) or sorted((json.dumps({f:v[f] for f in ('source','incarnation','ability','event')},sort_keys=True) for v in raw))==sorted(json.dumps(k,sort_keys=True) for k in keys),'/raw trigger bindings')
        rr=result['repeat_runs'][name]
        require(rr['consumed_play']==play and [semantic(p) for p in rr['points']]==[semantic(p) for p in points],'/repeat actual semantics')
    controls=specs(engine)
    require(set(result['rejections'])==set(controls) and set(result['negative_runs'])==set(controls),'/negative coverage')
    for name,s in controls.items():
        error=result['rejections'][name]
        prefix='first divergence: '+(f"/play/{s['sequence']} " if engine=='xmage' or name not in ('extra_tape','truncated_tape') else '')+s['category']
        require(error.startswith(prefix),'/intended rejection/'+name)
        require(name=='truncated_tape' or 'missing choice before named stop' not in error,'/late exhaustion/'+name)
        r=result['negative_runs'][name];require(r['points'],'/missing rejected observations')
        used=r['consumed_play'];require(used==s['input']['cases'][0]['play'][:len(used)],'/rejected choice ledger')
        if name=='failed_cast':
            require(r['post_run_trigger_counts']=={'pending':0,'stack':0},'/post-rejection cast trigger')
            p=r['points'][-1];require(p['pending_triggers']==[] and not any(a.get('ability')=='trigger' for a in p['stack']) and p['life']==[20,20],'/failed cast triggered')
    if engine=='native':require(result['rejection_state_rng']=='unchanged','/native transactional rejection')
    else:
        require(set(result['callback_controls'])=={'extra_order','extra_placement'},'/callback probes')
        for value in result['callback_controls'].values():require('unexpected trigger' in value,'/unsupported callback fails closed')

def comparator_controls(result,doc,engine):
    out={}
    def mutate(name,case,index,path,value):
        bad=copy.deepcopy(result);row=bad['runs'][case]['points'][index]
        for key in path[:-1]:row=row[key]
        row[path[-1]]=value
        bad['checkpoints'][case]=bad['runs'][case]['points']
        try:check_run(bad,doc,engine)
        except ValueError as e:
            diff=base.instant_reference.difference(result,bad)
            require(diff is not None and f'[{index % len(result["runs"][case]["points"])}]' in diff['path'],'/first divergence mutation location')
            out[name]=dict(diff,rejection=str(e))
        else:raise ValueError('mutation accepted: '+name)
    c='archer-cyclops';rows=result['checkpoints'][c];i=next(i for i,p in enumerate(rows) if len(p['stack'])==3)
    mutate('stack-order',c,i,['stack'],[rows[i]['stack'][0],rows[i]['stack'][2],rows[i]['stack'][1]])
    for f,v in [('source','0/firebrand-archer/2'),('incarnation',4),('event',99),('ability','pyromancer')]:mutate('wrong-'+f,c,i,['stack',1,'key',f],v)
    j=next(i for i,p in enumerate(rows) if p['pending_triggers'] and p['trigger_boundary']=='order')
    mutate('pending-settled',c,j,['trigger_boundary'],'settled')
    mutate('missing-pending',c,j,['pending_triggers'],[])
    mutate('extra-pending',c,j,['pending_triggers'],rows[j]['pending_triggers']*2)
    # Review reproduction: ordinary checkpoints after the first trigger event
    # must not acquire a pending trigger or become an ordering boundary.
    mutate('late-order-boundary',c,145,['trigger_boundary'],'order')
    mutate('late-invented-pending',c,145,['pending_triggers'],[{'controller':0,'key':{'source':'0/firebrand-archer/99','incarnation':99,'ability':'archer','event':99}}])
    c='pyromancer-bite';rows=result['checkpoints'][c];i=next(i for i,p in enumerate(rows) if p['stack'] and '0/viashino-pyromancer/0' in p['graveyard'][0])
    mutate('source-rebound',c,i,['stack',0,'key','incarnation'],4)
    if engine=='native':
        wrong=copy.deepcopy(rows[i]['stack'][0]['raw_source_handle']);wrong['generation']+=1
        mutate('raw-source-rebound',c,i,['stack',0,'raw_source_handle'],wrong)
    else:mutate('raw-source-rebound',c,i,['stack',0,'raw_source_zcc'],99)
    mutate('source-lki-card',c,i,['stack',0,'source_card'],'firebrand-archer')
    mutate('wrong-player',c,i,['stack',0,'target'],0)
    mutate('early-damage',c,i,['life'],[20,18])
    mutate('wrong-damage',c,-1,['life'],[20,19])
    return out

def source_files():
    import spells_reference
    paths=set(spells_reference.source_files())
    paths.update([Path(__file__).resolve(),ROOT/'crates/mtg-core/src/triggers_reference_tests.rs',BRIDGE,ROOT/'tests/test_m2_repair_triggers.py',ROOT/'references/xmage/triggers-provenance.json',ROOT/'fixtures/reference/author_triggers.py'])
    paths.update((ROOT/'fixtures/reference').glob('full-pool-triggers*'))
    return sorted(paths)

def run(args):
    import os,shutil,subprocess,sys,tempfile
    import xmage
    output=args.output.resolve();output.parent.mkdir(parents=True,exist_ok=True);output.unlink(missing_ok=True)
    folder=Path(tempfile.mkdtemp(prefix=output.stem+'.privileged-',dir=output.parent));os.chmod(folder,0o700)
    native_doc=json.loads(FIXTURE.read_text());reference_doc=json.loads(REFERENCE_FIXTURE.read_text())
    validate(native_doc);validate(reference_doc)
    sources=source_files();hashes={str(p.relative_to(ROOT)):xmage.sha(p) for p in sources}
    for p in (ROOT/'fixtures/reference').glob('full-pool-triggers*'):(folder/p.name).write_bytes(p.read_bytes())
    actual=native(folder);check_run(actual,native_doc,'native')
    cache=args.cache.resolve();xmage.verify_inputs(cache)
    require(xmage.dependencies(cache)==json.loads((ROOT/'references/xmage/dependencies.json').read_text()),'/dependencies')
    for family in ('mulligan','priority','spells','triggers'):
        provenance=json.loads((ROOT/f'references/xmage/{family}-provenance.json').read_text())
        for name,sha in provenance['consulted_sources'].items():require(xmage.sha(cache/xmage.SOURCE/name)==sha,'/upstream source '+name)
    for name in ('FullPoolMulliganTest.java','FullPoolPriorityTest.java','FullPoolTriggersTest.java'):
        target=cache/xmage.SOURCE/'Mage.Tests/src/test/java/org/mage/test/mtglab'/name
        target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes((ROOT/'references/xmage'/name).read_bytes())
    result_path=folder/'xmage.json'
    command=xmage.maven(cache)+['-o','-pl','Mage.Tests','-am','test','-Dtest=org.mage.test.mtglab.FullPoolTriggersTest','-Dsurefire.failIfNoSpecifiedTests=false',
        '-Dmtglab.fixture='+str(REFERENCE_FIXTURE),'-Dmtglab.root='+str(ROOT),'-Dmtglab.output='+str(result_path),'-DargLine=-Djava.awt.headless=true']
    xmage.bounded(command,cache/xmage.SOURCE,xmage.environment(cache),folder/'xmage.log',600)
    reference=json.loads(result_path.read_text());check_run(reference,reference_doc,'xmage')
    agreed={}
    for name,points in actual['checkpoints'].items():
        other=reference['checkpoints'][name];require(len(points)==len(other),'/checkpoint cardinality')
        indices=[]
        for i,(left,right) in enumerate(zip(points,other)):
            if any(p['payment'] is not None or p['targeting'] is not None or p['trigger_boundary']=='target' for p in (left,right)):continue
            lhs,rhs=semantic(left),semantic(right)
            # A lone XMage trigger bypasses chooseTriggeredAbility; native
            # still exposes it as pending. check_run asserts both exact states.
            # Retain comparison of every other field at this ordering boundary.
            if left['trigger_boundary']=='order' and len(left['pending_triggers'])==1:
                lhs.pop('pending_triggers');rhs.pop('pending_triggers')
            diff=base.instant_reference.difference(lhs,rhs)
            require(diff is None,'/committed/'+name+'/'+str(i)+' '+str(diff));indices.append(i)
        require(indices,'/no committed comparisons');agreed[name]=indices
    controls={'native':comparator_controls(actual,native_doc,'native'),'xmage':comparator_controls(reference,reference_doc,'xmage')}
    (folder/'comparator-controls.json').write_text(json.dumps(controls,indent=2)+'\n')
    (folder/'agreed-committed-indices.json').write_text(json.dumps(agreed,indent=2)+'\n')
    toolchain={}
    for name,command in [('java',['java','-version']),('maven',['mvn','-version']),('rustc',['rustc','--version']),('python',[sys.executable,'--version'])]:
        p=subprocess.run(command,capture_output=True,text=True,timeout=30,check=True,stdin=subprocess.DEVNULL)
        toolchain[name]={'version':p.stdout+p.stderr,'executable_sha256':xmage.sha(Path(shutil.which(command[0])).resolve())}
    (folder/'toolchain.json').write_text(json.dumps(toolchain,indent=2)+'\n')
    for p in folder.iterdir():
        if p.is_file():os.chmod(p,0o600)
    require(hashes=={str(p.relative_to(ROOT)):xmage.sha(p) for p in sources},'/source changed during execution')
    receipt=dict(schema_version=6,family='triggers',status='agreed',cases=len(native_doc['cases']),runs_per_engine=2*len(native_doc['cases']),
        completion='bounded_normal_reset_played_trigger_prefixes',pins=native_doc['pins'],
        upstream_commit=json.loads((ROOT/'references/xmage/pins.json').read_text())['upstream_commit'],sources=hashes,
        privileged_directory=folder.name,privileged_artifacts={p.name:xmage.sha(p) for p in folder.iterdir() if p.is_file()},
        compared_committed_checkpoints={k:len(v) for k,v in agreed.items()},negative_controls=list(actual['rejections']),callback_controls=list(reference['callback_controls']),comparator_controls_detected={engine:len(values) for engine,values in controls.items()},
        acceptance_claims={'selected_actions':'Actual scalar semantic decode/apply and pinned XMage callbacks; every supplied choice consumed.',
            'native_rejections':'Rejected actions preserve live state/RNG; failed envelopes preserve caller state.',
            'reference_rejections':'Actual played cast/payment/trigger-order/target callbacks fail at intended boundaries; no internal-RNG or rollback claim.',
            'complete_legal_set':False},
        limits='Pending and target-placement staging is witnessed separately. XMage selects order for multiple triggers and directly places lone triggers; its ETB target callback sees an announced stack ability while native retains the pending trigger. Semantic comparison covers settled checkpoints; strict engine-specific assertions cover in-flight states. Existing synthetic APNAP evidence remains supplemental, not reachable played evidence. No combat, cleanup-expiry, terminal-game or M2 completion claim.',stdin='closed',display='unset',offline=True)
    output.write_text(json.dumps(receipt,indent=2)+'\n')
    print('Three actual played trigger prefixes, repeats, strict negatives and comparator controls passed.')
