"""Version 5: strict played activations, with separate witnessed payment staging."""
import copy
import json
from pathlib import Path
import full_pool_reference as base
import priority_reference
ROOT=base.ROOT
FIXTURE=ROOT/'fixtures/reference/full-pool-activations.json'
BRIDGE=ROOT/'references/xmage/FullPoolActivationsTest.java'
require=base.require


def validate(doc):
    require(type(doc.get('schema_version')) is int and doc['schema_version']==5 and doc.get('family')=='activations','/version/family')
    projection=copy.deepcopy(doc);projection.update(schema_version=3,family='priority')
    for c in projection['cases']:
        require(c['stop']=='activations/'+c['id'],'/stop')
        c['stop']='second_creature_resolved'
    priority_reference.validate(projection)


def negative_inputs(doc):
    require(doc==json.loads(FIXTURE.read_text()),'/negative fixture binding')
    return json.loads((ROOT/'fixtures/reference/full-pool-activations-native-negatives.json').read_text())


def native(folder):return base.native(folder,family='activations')


def semantic(p):
    out={k:p[k] for k in ('turn','step','active','actor','life','mana','land_plays','library','graveyard','exile','permanents')}
    out['hand']=[sorted(h) for h in p['hand']];out['battlefield']=sorted(p['battlefield'])
    out['stack']=[{k:v for k,v in s.items() if k not in ('raw_birth','raw_id')} for s in p['stack']]
    out['incarnations']={k:v for k,v in p['incarnations'].items() if not k.startswith('ability/')}
    return out


def compare(points,engine='native'):
    doc=json.loads(FIXTURE.read_text());require(set(points)=={c['id'] for c in doc['cases']},'/case coverage')
    for c in doc['cases']:
        name=c['id'];rows=points[name]
        require(bool(rows),'/empty observations/'+name)
        for p in rows:
            require(set(p)=={'boundary','turn','step','active','actor','life','mana','land_plays','hand','library','graveyard','exile','battlefield','stack','incarnations','permanents','payment','activation','effects','haste','trample'},'/observation fields/'+name)
        p=rows[-1];require(p['boundary']==c['stop'] and p['turn']==c['play'][-1]['turn'] and p['step']=='precombat_main','/named stop/'+name)
        require(p['stack']==[] and p['payment'] is None and p['activation'] is None,'/unfinished activation/'+name)
        require(p['life']==[20,20] and p['graveyard']==[[],[]] and p['exile']==[],'/literal zones/life/'+name)
        # Independently count dealt/drawn cards, and named land/cast plays.
        played=[e['source'] for e in c['play'] if e['kind'] in ('cast','play_land')]
        require(sorted(p['battlefield'])==sorted(played),'/literal battlefield/'+name)
        for seat in (0,1):
            draws=(p['turn']+1)//2-1 if seat==0 else p['turn']//2
            order=c['chance'][seat]['after'];dealt=order[:7+draws]
            require(p['library'][seat]==order[7+draws:] and sorted(p['hand'][seat])==sorted(x for x in dealt if x not in played),'/literal draw ledger/'+name)
        if name.startswith('shivan'):
            creature=p['permanents']['0/shivan-dragon/0']
            require((creature['power'],creature['toughness'])==(7 if name=='shivan-repeated' else 6,5),'/literal Shivan boost')
        elif name.startswith('invoker'):
            creature=p['permanents']['1/bear-cub/0']
            require((creature['power'],creature['toughness'],creature['trample'])==(7,7,True),'/literal Invoker boost')
        else:
            for target in ('1/llanowar-elves/1','1/druid-of-the-cowl/1'):
                q=p['permanents'][target];require(q['haste'] and q['tapped'] and not q['sick'],'/haste permits sick mana creature')
        # The last untap/activation schedule is fixed independently of output.
        if name=='haste':tapped={'0/axgard-cavalry/0','0/axgard-cavalry/1','1/llanowar-elves/1','1/druid-of-the-cowl/1',*[f'1/forest/{i}' for i in range(3)]}
        elif name.startswith('invoker'):tapped={*[f'0/mountain/{i}' for i in range(6)],*[f'1/forest/{i}' for i in range(6)],'1/llanowar-elves/0','1/druid-of-the-cowl/0'}
        else:tapped={f'0/mountain/{i}' for i in range(1 if name=='shivan-single' else 2)}
        require({id for id,p in p['permanents'].items() if p['tapped']}==tapped,'/literal final taps/'+name)
        expected=[[0]*6,[0]*6]
        if name=='haste':expected[1][4]=2
        if name=='shivan-floating':expected[0][3]=1
        require(p['mana']==expected,'/literal surplus mana/'+name)


def check_run(result,doc,engine):
    compare(result['checkpoints'],engine)
    for case in doc['cases']:
        name=case['id'];r=result['runs'][name];points=r['points']
        require(points==result['checkpoints'][name] and len(points)==len(case['play'])+1,'/complete witnessed observations')
        require(r['consumed_play']==case['play'],'/actual consumed actions')
        for key in ('chance','choices'):require(r['opening']['consumed_'+key]==case[key],'/actual opening '+key)
        for i,e in enumerate(case['play']):
            p=points[i]
            require((p['boundary'],p['turn'],p['step'],p['actor'])==(f'before/{i}',e['turn'],e['step'],e['actor']),'/choice boundary/'+name+'/'+str(i))
            if e['kind']=='activate':
                j=next(j for j in range(i+1,len(case['play'])) if case['play'][j]['kind']=='finish_activation')
                end=points[j+1];s=end['stack'][-1]
                require(s['source']==e['source'] and s['incarnation']==e['incarnation'] and s['action']==i and s['object']==f'ability/{i}','/source/ability/stack action')
                targets=[x for x in case['play'][i+1:j] if x['kind']=='activation_target']
                require(s['target']==({'source':targets[0]['source'],'incarnation':targets[0]['incarnation']} if targets else None),'/activation target')
                taps=[x for x in case['play'][i+1:j] if x['kind']=='tap_mana']
                for x in taps:require(end['permanents'][x['source']]['tapped'],'/committed source tap')
                require(len(end['stack'])==len(p['stack'])+1,'/exactly one ability on commit')
                expected_pool=p['mana'][e['actor']].copy();reserved=[0]*6;sources=[];target=None
                ability={'shivan-dragon':'power','wildheart-invoker':'invoker','axgard-cavalry':'haste'}[e['source'].split('/')[1]]
                for k in range(i+1,j+1):
                    q=points[k];stage=q['activation'];require(stage is not None,'/missing activation observation')
                    require(stage['actor']==e['actor'] and stage['pool']==expected_pool,'/witnessed payment pool')
                    if engine=='native':
                        cost=1 if ability=='power' else 8 if ability=='invoker' else 0
                        require(set(stage)=={'source','actor','target','paid','reserved','pool','sources','ability'},'/native activation fields')
                        require(type(stage['paid']) is bool and stage['paid']==(cost>0 and sum(reserved)==cost),'/native paid flag')
                        require(stage['source']=={'source':e['source'],'incarnation':e['incarnation']} and stage['ability']==ability and stage['target']==target and stage['reserved']==reserved and stage['sources']==sources,'/native staged source/target/payment')
                    else:
                        expected_fields={'source','actor','pool','sources'}
                        if case['play'][k]['kind']!='activation_target':
                            expected_fields|={'colored','generic'}
                            colored=[0]*6
                            if ability=='power':colored[3]=1-sum(reserved)
                            generic=8-sum(reserved) if ability=='invoker' else 0
                            require(stage.get('colored')==colored and stage.get('generic')==generic,'/reference unpaid cost')
                        require(set(stage)==expected_fields,'/reference activation fields')
                        require(stage['source']==e['source'] and stage['sources']==[x['source'] for x in sources],'/reference staged sources')
                        active=q['stack'][-1];require(active['action']==i and active['source']==e['source'] and active['target']==target,'/reference announced ability/target')
                    x=case['play'][k]
                    if x['kind']=='tap_mana':
                        color=3 if x['source'].split('/')[1]=='mountain' else 4
                        expected_pool[color]+=1;sources.append({'source':x['source'],'incarnation':x['incarnation']})
                    elif x['kind']=='activation_pay':expected_pool[x['color']]-=1;reserved[x['color']]+=1
                    elif x['kind']=='activation_target':target={'source':x['source'],'incarnation':x['incarnation']}
                if engine=='native':
                    for q in points[i+1:j+1]:
                        require(q['activation'] is not None,'/missing native activation stage')
                        for field in ('mana','permanents','stack','hand','battlefield','incarnations'):
                            require(q[field]==p[field],'/private transaction changed public '+field)
                    stage=points[j]['activation'];require(stage['sources']==[{'source':x['source'],'incarnation':x['incarnation']} for x in taps],'/reserved sources')
                    paid=[0]*6
                    for x in case['play'][i+1:j]:
                        if x['kind']=='activation_pay':paid[x['color']]+=1
                    require(stage['reserved']==paid,'/reserved payment')
            if e['kind']=='tap_mana' and p['payment'] is None and p['activation'] is None:
                q=points[i+1];require(q['stack']==p['stack'] and q['actor']==p['actor'],'/mana ability added stack or priority')
        final=points[-1];effects=final['effects']
        if engine=='native':
            wanted=[]
            if name.startswith('shivan'):wanted=[{'object':{'source':'0/shivan-dragon/0','incarnation':3},'boost':0,'power_boost':2 if name=='shivan-repeated' else 1,'damage':0}]
            elif name.startswith('invoker'):wanted=[{'object':{'source':'1/bear-cub/0','incarnation':3},'boost':5,'power_boost':0,'damage':0}]
            require(effects==wanted,'/actual temporary modification storage')
        else:
            count=2 if name in ('shivan-repeated','haste') or name.startswith('invoker') else 1
            require(len(effects)==count and all(e['duration']=='EndOfTurn' and e['raw_id'] for e in effects),'/actual effect duration/count')
            sources=sorted(s for e in effects for s in e['sources'])
            wanted=['0/axgard-cavalry/0','0/axgard-cavalry/1'] if name=='haste' else ['1/wildheart-invoker/0']*2 if name.startswith('invoker') else ['0/shivan-dragon/0']*count
            require(sources==wanted,'/actual effect source identity')
            require(len(r['activation_checks'])==sum(e['kind']=='activate' for e in case['play']),'/selected activation engine checks')
            require(len(r['mana_checks'])==sum(e['kind']=='tap_mana' for e in case['play']),'/actual mana ability checks')
            require(all(x['candidate_count']==1 and x['candidate_id'] for x in r['activation_checks']),'/selected activation membership')
        identities={}
        for p in points:
            for a in p['stack']:
                if a['ability']=='spell':continue
                raw=a['raw_birth' if engine=='native' else 'raw_id']
                require(raw not in identities or identities[raw]==a['action'],'/identical raw activation stack identities')
                identities[raw]=a['action']
        require(len(identities)==sum(e['kind']=='activate' for e in case['play']),'/witnessed unique activation stack objects')
        repeat=result['repeat_runs'][name]
        require(repeat['consumed_play']==r['consumed_play'] and [semantic(p) for p in repeat['points']]==[semantic(p) for p in points],'/repeat')
        if engine=='native':require(r['records'] and r['policy_capture'] and r['stale_candidates_rejected']==len(r['records']),'/real scalar recording')
    if engine=='xmage':
        categories={'unexpected_target':'/unexpected activation target callback','unexpected_mana':'/unscripted mana callback','unexpected_mode':'/unsupported mode callback'}
        require(set(result['callback_controls'])==set(categories),'/unsupported callback coverage')
        for name,category in categories.items():require(result['callback_controls'][name].endswith(category),'/unsupported callback boundary/'+name)
    if engine in ('native','xmage'):
        specs=negative_inputs(doc) if engine=='native' else json.loads((ROOT/'fixtures/reference/full-pool-activations-xmage-negatives.json').read_text());require(set(result['rejections'])==set(specs),'/negative coverage')
        for name,spec in specs.items():
            require(result['rejections'][name].startswith(f"first divergence: /play/{spec['sequence']} {spec['category']}"),'/intended rejection boundary/'+name)
            require(len(result['negative_runs'][name]['consumed_play'])==spec['sequence'],'/rejection cursor/'+name)
        if engine=='native':require(result['rejection_state_rng']=='unchanged','/rejection atomicity')

        cancels=json.loads((ROOT/('fixtures/reference/full-pool-activations-cancellations.json' if engine=='native' else 'fixtures/reference/full-pool-activations-xmage-cancellations.json')).read_text())
        require(set(result['cancelled_runs'])==set(cancels),'/every cancellation stage')
        for name,spec in cancels.items():
            case=spec['input']['cases'][0];r=result['cancelled_runs'][name][case['id']]
            require(r['consumed_play']==case['play'],'/actual cancelled choices')
            stage=spec['cancel_sequence'];start=max(i for i in range(stage) if case['play'][i]['kind']=='activate')
            before=r['points'][start];after=r['points'][stage+1]
            require(after['activation'] is None,'/cancel leaves pending activation')
            require(semantic(before)==semantic(after),'/cancel changed committed state')


def comparator_controls(result,doc,engine):
    controls={}
    def probe(name,mutate):
        bad=dict(result);bad["runs"]=copy.deepcopy(result["runs"]);mutate(bad)
        # Preserve report/checkpoint binding; mutations target the real reports.
        bad['checkpoints']={k:r['points'] for k,r in bad['runs'].items()}
        try:check_run(bad,doc,engine)
        except ValueError as error:controls[name]=str(error)
        else:raise ValueError('missed comparator control '+name)
    probe('missing-payment',lambda r:r['runs']['shivan-single']['points'][0].pop('activation'))
    probe('empty-observations',lambda r:r['runs']['shivan-single'].__setitem__('points',[]))
    probe('wrong-stats',lambda r:r['runs']['invoker-empty']['points'][-1]['permanents']['1/bear-cub/0'].__setitem__('power',6))
    probe('wrong-surplus',lambda r:r['runs']['shivan-floating']['points'][-1]['mana'][0].__setitem__(3,0))
    probe('missing-trample',lambda r:r['runs']['invoker-empty']['points'][-1]['permanents']['1/bear-cub/0'].__setitem__('trample',False))
    def stack_swap(r):
        for p in r['runs']['shivan-repeated']['points']:
            if len(p['stack'])==2:p['stack'][1]['action']=p['stack'][0]['action']
    probe('identical-stack-actions',stack_swap)
    def sources(r):
        for p in r['runs']['invoker-empty']['points']:
            if p['activation'] and p['activation']['sources']:
                if engine=='native':p['activation']['sources'][0]['source']='1/forest/5'
                else:p['activation']['sources'][0]='1/forest/5'
    probe('swapped-same-name-source',sources)
    def target(r):
        for p in r['runs']['invoker-empty']['points']:
            for s in p['stack']:
                if s['ability']=='invoker':s['target']=None
    probe('missing-target',target)
    def payment_metadata(r):
        for p in r['runs']['shivan-single']['points']:
            if p['activation'] is not None:
                if engine=='native':p['activation']['paid']=not p['activation']['paid'];return
                if 'colored' in p['activation']:p['activation']['colored']=[0]*6;p['activation']['generic']=1;return
        raise ValueError('missing real payment stage for control')
    probe('wrong-payment-metadata',payment_metadata)

    for field in ('incarnations','effects','mana','library','stack','permanents'):
        probe('missing-'+field,lambda r,f=field:r['runs']['haste']['points'][0].pop(f))
    return controls


def source_files():
    return [Path(__file__),BRIDGE,FIXTURE,
        *sorted((ROOT/'fixtures/reference').glob('*activations*')),
        ROOT/'references/xmage/FullPoolMulliganTest.java',ROOT/'references/xmage/FullPoolPriorityTest.java',
        ROOT/'references/xmage/activations-provenance.json',ROOT/'references/xmage/priority-provenance.json',ROOT/'references/xmage/mulligan-provenance.json',
        ROOT/'references/xmage/pins.json',ROOT/'references/xmage/dependencies.json',
        ROOT/'scripts/full_pool_reference.py',ROOT/'scripts/priority_reference.py',ROOT/'scripts/mulligan_reference.py',
        ROOT/'scripts/xmage.py',ROOT/'scripts/instant_reference.py',ROOT/'scripts/scenario.py',
        ROOT/'tests/test_m2_repair_activations.py',ROOT/'Cargo.lock',ROOT/'Cargo.toml',
        *sorted((ROOT/'crates').rglob('Cargo.toml')),*sorted((ROOT/'crates').rglob('*.rs'))]


def run(args):
    import os,shutil,subprocess,sys,tempfile
    import xmage
    output=args.output.resolve();output.parent.mkdir(parents=True,exist_ok=True);output.unlink(missing_ok=True)
    folder=Path(tempfile.mkdtemp(prefix=output.stem+'.privileged-',dir=output.parent));os.chmod(folder,0o700)
    doc=json.loads(FIXTURE.read_text());validate(doc)
    for path in (ROOT/'fixtures/reference').glob('*activations*'):
        if path.is_file():(folder/path.name).write_bytes(path.read_bytes())
    sources=source_files();hashes={str(p.relative_to(ROOT)):xmage.sha(p) for p in sources}
    actual=native(folder);check_run(actual,doc,'native')
    cache=args.cache.resolve();xmage.verify_inputs(cache)
    require(xmage.dependencies(cache)==json.loads((ROOT/'references/xmage/dependencies.json').read_text()),'/dependencies')
    for family in ('mulligan','priority','activations'):
        provenance=json.loads((ROOT/f'references/xmage/{family}-provenance.json').read_text())
        for name,sha in provenance['consulted_sources'].items():require(xmage.sha(cache/xmage.SOURCE/name)==sha,'/upstream source '+name)
    for name in ('FullPoolMulliganTest.java','FullPoolPriorityTest.java','FullPoolActivationsTest.java'):
        target=cache/xmage.SOURCE/'Mage.Tests/src/test/java/org/mage/test/mtglab'/name
        target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes((ROOT/'references/xmage'/name).read_bytes())
    result_path=folder/'xmage.json'
    command=xmage.maven(cache)+['-o','-pl','Mage.Tests','-am','test','-Dtest=org.mage.test.mtglab.FullPoolActivationsTest','-Dsurefire.failIfNoSpecifiedTests=false',
        '-Dmtglab.fixture='+str(FIXTURE),'-Dmtglab.root='+str(ROOT),'-Dmtglab.output='+str(result_path),'-DargLine=-Djava.awt.headless=true']
    xmage.bounded(command,cache/xmage.SOURCE,xmage.environment(cache),folder/'xmage.log',600)
    reference=json.loads(result_path.read_text());check_run(reference,doc,'xmage')
    agreed={}
    for name,points in actual['checkpoints'].items():
        other=reference['checkpoints'][name];require(len(points)==len(other),'/checkpoint cardinality')
        indices=[]
        for i,(left,right) in enumerate(zip(points,other)):
            if any(p['payment'] is not None or p['activation'] is not None for p in (left,right)):continue
            diff=base.instant_reference.difference(semantic(left),semantic(right))
            require(diff is None,'/committed/'+name+'/'+str(i)+' '+str(diff));indices.append(i)
        require(indices,'/no committed comparisons');agreed[name]=indices
    controls={engine:comparator_controls(r,doc,engine) for engine,r in [('native',actual),('xmage',reference)]}
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
    receipt=dict(schema_version=5,family='activations',status='agreed',cases=len(doc['cases']),runs_per_engine=2*len(doc['cases']),
        completion='bounded_main_phase_activation_prefixes',pins=doc['pins'],
        upstream_commit=json.loads((ROOT/'references/xmage/pins.json').read_text())['upstream_commit'],sources=hashes,
        privileged_directory=folder.name,privileged_artifacts={p.name:xmage.sha(p) for p in folder.iterdir() if p.is_file()},
        compared_committed_checkpoints={k:len(v) for k,v in agreed.items()},negative_controls=list(actual['rejections']),callback_controls=list(reference['callback_controls']),
        cancellations={engine:len(r['cancelled_runs']) for engine,r in [('native',actual),('xmage',reference)]},
        comparator_controls_detected={engine:len(v) for engine,v in controls.items()},
        acceptance_claims={'selected_actions':'Actual scalar semantic decode/apply and pinned XMage player callbacks accepted; complete choices consumed.',
            'native_rejections':'Rejected semantic actions and envelopes preserve state/RNG.',
            'reference_rejections':'Engine playable-action membership and actual target/mana callbacks. Exact input cursor and category; no complete legal-set claim.',
            'complete_legal_set':False},
        limits='Private native reservation stages and raw reference announcement/payment stages are separately asserted. Native paid-but-uncommitted cancellation is a scalar continuation; XMage cancellation is exercised at each actual target/payment callback. Source incarnation, distinct activation stack objects, observed temporary effect storage/durations retained; stop before cleanup. No reference internal RNG, noncreature/trigger/combat callback, full-game or M2 completion claim.',
        stdin='closed',display='unset',offline=True)
    output.write_text(json.dumps(receipt,indent=2)+'\n')
    print('Actual played activation prefixes, repeats, cancellations, invalid tapes and comparator controls passed.')
