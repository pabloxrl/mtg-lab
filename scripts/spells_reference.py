"""Version 4 played spell extension; privileged observations from real engines."""
import copy
import json
from pathlib import Path
import full_pool_reference as base

ROOT=base.ROOT
FIXTURE=ROOT/'fixtures/reference/full-pool-spells.json'
REFERENCE_FIXTURE=ROOT/'fixtures/reference/full-pool-spells-xmage-input.json'
FINAL=ROOT/'fixtures/reference/full-pool-spells-final.json'
BRIDGE=ROOT/'references/xmage/FullPoolSpellsTest.java'
require=base.require


def negative_specs(engine):
    return json.loads((ROOT/f'fixtures/reference/full-pool-spells-{engine}-negatives.json').read_text())


def negative_inputs(doc):
    require(doc==json.loads(FIXTURE.read_text()),'/negative fixture binding')
    return negative_specs('native')


def validate(doc):
    import priority_reference
    require(doc.get('schema_version')==4 and doc.get('family')=='spells','/version/family')
    projected=copy.deepcopy(doc);projected.update(schema_version=3,family='priority')
    for case in projected['cases']:
        require(case['stop'].startswith('resolved/'),'/stop')
        case['stop']='second_creature_resolved'
        for e in case['play']:
            require(set(e)=={'sequence','turn','step','actor','kind','source','incarnation','color','role','mode'},'/spell choice fields')
            del e['role'];del e['mode']
    priority_reference.validate(projected)


def native(folder):
    return base.native(folder,family='spells')


def compare(points,engine='native'):
    """Independent fixed final ledgers, never baselined from engine output."""
    wanted=json.loads(FINAL.read_text())
    require(set(points)==set(wanted),'/case coverage')
    fields={'boundary','turn','step','active','actor','life','mana','land_plays','hand','library','graveyard','exile','battlefield','stack','incarnations','permanents','payment','targeting','effects','creations','departures'}
    for name,rows in points.items():
        require(bool(rows),'/empty observations/'+name)
        for row in rows:require(set(row)==fields,'/missing/extra observation field/'+name)
        last=rows[-1]
        for field,value in wanted[name].items():
            if field=='hand_membership': actual=[sorted(h) for h in last['hand']]
            elif field=='battlefield_membership':actual=sorted(last['battlefield'])
            elif field=='creatures':
                actual={k:{f:v[f] for f in ('power','toughness','owner','controller','card')} for k,v in last['permanents'].items() if v['power'] is not None}
            else:actual=last[field]
            diff=base.instant_reference.difference(value,actual)
            require(diff is None,'/literal-final/'+name+'/'+field+' '+str(diff))


def check_rejections(actual,engine):
    specs=negative_specs(engine)
    require(set(actual)==set(specs),'/negative controls')
    for name,spec in specs.items():
        if engine=='native' and name=='truncated_tape':prefix='first divergence: /missing choice before named stop'
        elif engine=='native' and name=='extra_tape':prefix='first divergence: /extra choice after named stop'
        else:
            sequence=spec['sequence']+(1 if name=='extra_tape' else 0)
            prefix=f'first divergence: /play/{sequence} '+spec['category']
        require(actual[name].startswith(prefix),'/intended rejection boundary/'+name)
        require(name=='truncated_tape' or 'missing choice before named stop' not in actual[name],'/late exhaustion/'+name)


def semantic(point):
    """Common committed state. Raw in-flight observations remain privileged."""
    fields=('turn','step','active','actor','life','mana','land_plays','library','graveyard','exile','stack','incarnations','permanents')
    out={f:point[f] for f in fields}
    out['hand_membership']=[sorted(h) for h in point['hand']]
    out['battlefield_membership']=sorted(point['battlefield'])
    out['token_creation_order']=[b['id'] for b in point['creations']]
    return out


def check_run(result,doc,engine):
    compare(result['checkpoints'],engine)
    require(set(result['runs'])=={c['id'] for c in doc['cases']},'/runs coverage')
    for case in doc['cases']:
        name=case['id'];run=result['runs'][name];points=run['points']
        require(points==result['checkpoints'][name] and len(points)==len(case['play'])+1,'/complete observations/'+name)
        require(run['consumed_play']==case['play'],'/actual choices/'+name)
        for field in ('chance','choices'):require(run['opening']['consumed_'+field]==case[field],'/actual opening '+field)
        require(points[-1]['boundary']==case['stop'],'/actual stop')
        for i,e in enumerate(case['play']):
            p=points[i]
            require(p['boundary']==f'before/{i}' and (p['turn'],p['step'],p['actor'])==(e['turn'],e['step'],e['actor']),'/choice boundary/'+name+'/'+str(i))
            if e['kind']=='cast':
                end=next(j for j in range(i+1,len(case['play'])) if case['play'][j]['kind']=='finish_payment')
                selected=case['play'][i+1:end];committed=points[end+1]
                if engine=='native':
                    for stage in points[i+1:end+1]:
                        for field in ('mana','hand','graveyard','battlefield','stack','incarnations','permanents','effects'):
                            require(stage[field]==p[field],'/partial native cast commit/'+field)

                entry=committed['stack'][-1]
                require(entry['source']==e['source'] and entry['incarnation']==e['incarnation']+1 and entry['action']==i,'/committed spell identity')
                wanted=[dict(role=s['role'],object=dict(source=s['source'],incarnation=s['incarnation'])) for s in selected if s['kind']=='target']
                require(entry['targets']==wanted,'/committed target roles/order')
                modes=[s['mode'] for s in selected if s['kind']=='mode']
                require(entry['mode']==(modes[0] if modes else None),'/committed mode')
                for s in selected:
                    if s['kind']=='tap_mana':require(committed['permanents'][s['source']]['tapped'],'/uncommitted mana source')
                    if s['kind']=='discard':require(s['source'] in committed['graveyard'][s['actor']] and s['source'] not in committed['hand'][s['actor']],'/discard cost before resolution')
                require(committed['mana'][e['actor']]==[0]*6,'/mana cost commit')
                # No spell effect can occur at commitment: births and target
                # characteristics remain the pre-announcement observations.
                require([b['id'] for b in p['creations']]==[b['id'] for b in committed['creations']],'/token before resolution')
        births=points[-1]['creations'];ids=[b['id'] for b in births]
        expected_count=0 if name=='thrill' else 4 if name=='surprise-tokens' else 2
        require(len(ids)==expected_count and len(set(ids))==expected_count,'/physical token births')
        require(ids==[f'token/0/{i//2}/{i%2}' for i in range(expected_count)],'/creation event/ordinal')
        rawkey='birth' if engine=='native' else 'raw_uuid'
        require(len({b[rawkey] for b in births})==expected_count,'/raw token identity')
        if engine=='native':
            manifest=json.loads((ROOT/'data/cards/foundations_micro_v1.json').read_text())
            token=next(c for c in manifest['cards'] if c['id']=='goblin-token')
            require(all(b['card']=='goblin-token' and b['definition_hash']==token['content_sha256'] for b in births),'/witnessed token definition identity')
        else:
            require(all(b['red'] is True and all(b[c] is False for c in ('white','blue','black','green')) and b['subtypes']==['GOBLIN'] for b in births),'/witnessed token color/subtype')
        if name=='thrill':
            require(points[-1]['hand'][0][-2:]==['0/mountain/6','0/mountain/7'],'/ordered two-card draw')
            first=next(j for j,e in enumerate(case['play']) if e['kind']=='finish_payment')+1
            require(points[first]['library'][0][:2]==['0/mountain/6','0/mountain/7'],'/library before Thrill resolution')
        if name in ('growth-bite','departed-target'):
            chain=[p for p in points if len(p['stack'])==2 and p['payment'] is None and p['targeting'] is None]
            require(bool(chain),'/real response stack')
            sources=[s['source'] for s in chain[0]['stack']]
            require(sources==(['1/bite-down/0','1/giant-growth/0'] if name=='growth-bite' else ['1/giant-growth/0','1/bite-down/0']),'/response stack order')
            if name=='departed-target':
                require(any(p['stack'] and p['stack'][0]['source']=='1/giant-growth/0' and p['stack'][0]['targets']==[{'role':'growth_target','object':None}] for p in points),'/departed target revalidation')
            require(len(points[-1]['departures'])==1,'/token disappearance event')
            d=points[-1]['departures'][0]
            if engine=='native':require(d['object']==dict(source='token/0/0/1',incarnation=0) and d['old_handle_valid'] is False,'/departed incarnation')
            else:require(d['source']=='token/0/0/1' and d['departing_incarnation']==0 and d['from']=='BATTLEFIELD' and d['to']=='GRAVEYARD','/departed incarnation')
        final_effects=points[-1]['effects']
        if engine=='native':
            require(run['records'] and run['policy_capture'] and run['stale_candidates_rejected']==len(run['records']),'/real scalar capture')
            boosts=[e for e in final_effects if e['boost'] or e['power_boost']]
            require(len(boosts)==(1 if name=='growth-bite' else 2 if name=='surprise-boost' else 0),'/actual native temporary effects')
            if name=='growth-bite':require(boosts==[{'object':{'source':'1/bear-cub/0','incarnation':3},'boost':3,'power_boost':0,'damage':0}],'/literal Growth modification')
            if name=='surprise-boost':
                require({e['object']['source'] for e in boosts}=={'token/0/0/0','token/0/0/1'} and all(e['boost']==0 and e['power_boost']==2 and e['damage']==0 for e in boosts),'/literal Surprise recipient modifications')
        else:
            require(len(run['raw_occurrence_bindings'])==80+expected_count,'/witnessed occurrences')
            require(len(run['selected_cast_checks'])==sum(e['kind']=='cast' for e in case['play']),'/selected cast legality')
            require(all(e['duration']=='EndOfTurn' for e in final_effects),'/observed effect duration')
            require(len(final_effects)==(1 if name in ('growth-bite','surprise-boost') else 0),'/actual reference temporary effects')
            if final_effects:require(final_effects[0]['sources']==(['1/giant-growth/0'] if name=='growth-bite' else ['0/goblin-surprise/0']),'/actual effect source identity')
        repeat=result['repeat_runs'][name]
        require(repeat['consumed_play']==run['consumed_play'],'/repeat choices')
        require([semantic(p) for p in repeat['points']]==[semantic(p) for p in points],'/repeat semantic checkpoints')
    check_rejections(result['rejections'],engine)
    specs=negative_specs(engine)
    require(set(result['negative_runs'])==set(specs),'/negative observation coverage')
    for name,run in result['negative_runs'].items():
        case=specs[name]['input']['cases'][0]
        used=run['consumed_play']
        require(run['points'] and used==case['play'][:len(used)],'/actual rejected-prefix choices/'+name)
        require(len(used)==specs[name]['sequence']+(1 if name=='extra_tape' else 0),'/rejected-prefix stop/'+name)
        for field in ('chance','choices'):require(run['opening']['consumed_'+field]==case[field],'/actual rejected opening/'+name+'/'+field)

    if engine=='native':require(result['rejection_state_rng']=='unchanged','/native rejection nonmutation')
    else:
        expected={'extra_target':'/unexpected target callback','extra_mode':'/unexpected mode callback','extra_mana':'/unscripted mana callback','duplicate_token_birth':'/duplicate token id'}
        require(set(result['callback_controls'])==set(expected),'/strict callback probes')
        for name,category in expected.items():require(result['callback_controls'][name].endswith(category),'/callback category/'+name)


def comparator_controls(points,engine="native"):
    controls={}
    def probe(name,case,index,path,value):
        bad=copy.deepcopy(points);row=bad[case][index]
        for key in path[:-1]:row=row[key]
        row[path[-1]]=value
        diff=base.instant_reference.difference(points,bad)
        prefix=f'$.{case}[{index % len(points[case])}]'
        for item in path:prefix+=f'[{item}]' if isinstance(item,int) else '.'+item
        require(diff is not None and diff['path'].startswith(prefix),'/first divergence comparator control/'+name)
        controls[name]=diff
    n=next(i for i,p in enumerate(points['growth-bite']) if len(p['stack'])==2)
    probe('stack-order','growth-bite',n,['stack'],list(reversed(points['growth-bite'][n]['stack'])))
    probe('target-role','growth-bite',n,['stack',0,'targets',0,'role'],'bite_destination')
    probe('target-identity','growth-bite',n,['stack',0,'targets',1,'object','source'],'token/0/0/0')
    probe('target-incarnation','growth-bite',n,['stack',0,'targets',0,'object','incarnation'],2)
    probe('target-order','growth-bite',n,['stack',0,'targets'],list(reversed(points['growth-bite'][n]['stack'][0]['targets'])))
    probe('token-id','fodder',-1,['creations',1,'id'],'token/0/0/0')
    probe('token-order','fodder',-1,['creations'],list(reversed(points['fodder'][-1]['creations'])))
    probe('token-stats','fodder',-1,['permanents','token/0/0/0','power'],2)
    probe('growth-effect','growth-bite',-1,['permanents','1/bear-cub/0','toughness'],2)
    probe('discard-cost','thrill',-1,['graveyard',0],['0/thrill-of-possibility/0'])
    probe('draw-order','thrill',-1,['hand',0],list(reversed(points['thrill'][-1]['hand'][0])))
    if engine=='native':
        probe('effect-identity','growth-bite',-1,['effects',0,'object','source'],'token/0/0/0')
        probe('effect-amount','growth-bite',-1,['effects',0,'boost'],2)
        probe('effect-order','surprise-boost',-1,['effects'],list(reversed(points['surprise-boost'][-1]['effects'])))
    else:
        probe('effect-source','growth-bite',-1,['effects',0,'sources'],['1/bite-down/0'])
        probe('effect-duration','growth-bite',-1,['effects',0,'duration'],'EndOfGame')
    return controls


def source_files():
    return [FIXTURE,REFERENCE_FIXTURE,FINAL,BRIDGE,Path(__file__).resolve(),
        ROOT/'fixtures/reference/author_spells.py',ROOT/'fixtures/reference/author_priority.py',
        ROOT/'fixtures/reference/full-pool-spells-native-negatives.json',
        ROOT/'fixtures/reference/full-pool-spells-xmage-negatives.json',
        ROOT/'fixtures/reference/full-pool-spells-oracle.md',
        ROOT/'references/xmage/FullPoolPriorityTest.java',ROOT/'references/xmage/FullPoolMulliganTest.java',
        ROOT/'references/xmage/spells-provenance.json',ROOT/'references/xmage/priority-provenance.json',ROOT/'references/xmage/mulligan-provenance.json',
        ROOT/'references/xmage/pins.json',ROOT/'references/xmage/dependencies.json',
        ROOT/'scripts/full_pool_reference.py',ROOT/'scripts/priority_reference.py',ROOT/'scripts/mulligan_reference.py',
        ROOT/'scripts/xmage.py',ROOT/'scripts/instant_reference.py',ROOT/'scripts/scenario.py',
        ROOT/'tests/test_m2_repair_spells.py',ROOT/'Cargo.lock',ROOT/'Cargo.toml',
        *sorted((ROOT/'crates').rglob('Cargo.toml')),
        *sorted((ROOT/'crates').rglob('*.rs'))]


def run(args):
    import os,shutil,subprocess,sys,tempfile
    import xmage
    output=args.output.resolve();output.parent.mkdir(parents=True,exist_ok=True);output.unlink(missing_ok=True)
    folder=Path(tempfile.mkdtemp(prefix=output.stem+'.privileged-',dir=output.parent));os.chmod(folder,0o700)
    native_doc=json.loads(FIXTURE.read_text());reference_doc=json.loads(REFERENCE_FIXTURE.read_text())
    validate(native_doc);validate(reference_doc)
    for path in (FIXTURE,REFERENCE_FIXTURE,FINAL,ROOT/'fixtures/reference/full-pool-spells-native-negatives.json',ROOT/'fixtures/reference/full-pool-spells-xmage-negatives.json',ROOT/'fixtures/reference/full-pool-spells-oracle.md'): (folder/path.name).write_bytes(path.read_bytes())
    sources=source_files();hashes={str(p.relative_to(ROOT)):xmage.sha(p) for p in sources}
    actual=native(folder);check_run(actual,native_doc,'native')
    cache=args.cache.resolve();xmage.verify_inputs(cache)
    require(xmage.dependencies(cache)==json.loads((ROOT/'references/xmage/dependencies.json').read_text()),'/dependencies')
    for family in ('mulligan','priority','spells'):
        provenance=json.loads((ROOT/f'references/xmage/{family}-provenance.json').read_text())
        for name,sha in provenance['consulted_sources'].items():require(xmage.sha(cache/xmage.SOURCE/name)==sha,'/upstream source '+name)
    for name in ('FullPoolMulliganTest.java','FullPoolPriorityTest.java','FullPoolSpellsTest.java'):
        target=cache/xmage.SOURCE/'Mage.Tests/src/test/java/org/mage/test/mtglab'/name
        target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes((ROOT/'references/xmage'/name).read_bytes())
    result_path=folder/'xmage.json'
    command=xmage.maven(cache)+['-o','-pl','Mage.Tests','-am','test','-Dtest=org.mage.test.mtglab.FullPoolSpellsTest','-Dsurefire.failIfNoSpecifiedTests=false',
        '-Dmtglab.fixture='+str(REFERENCE_FIXTURE),'-Dmtglab.root='+str(ROOT),'-Dmtglab.output='+str(result_path),'-DargLine=-Djava.awt.headless=true']
    xmage.bounded(command,cache/xmage.SOURCE,xmage.environment(cache),folder/'xmage.log',600)
    reference=json.loads(result_path.read_text());check_run(reference,reference_doc,'xmage')
    agreed={}
    for name,points in actual['checkpoints'].items():
        other=reference['checkpoints'][name];require(len(points)==len(other),'/checkpoint cardinality')
        indices=[]
        for i,(left,right) in enumerate(zip(points,other)):
            if any(p['payment'] is not None or p['targeting'] is not None for p in (left,right)):continue
            diff=base.instant_reference.difference(semantic(left),semantic(right))
            require(diff is None,'/committed/'+name+'/'+str(i)+' '+str(diff));indices.append(i)
        require(len(indices)>0,'/no committed comparisons');agreed[name]=indices
    controls={'native':comparator_controls(actual['checkpoints']),'xmage':comparator_controls(reference['checkpoints'],'xmage')}
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
    receipt=dict(schema_version=4,family='spells',status='agreed',cases=len(native_doc['cases']),runs_per_engine=2*len(native_doc['cases']),
        completion='bounded_main_phase_spell_prefixes',pins=native_doc['pins'],
        upstream_commit=json.loads((ROOT/'references/xmage/pins.json').read_text())['upstream_commit'],sources=hashes,
        privileged_directory=folder.name,privileged_artifacts={p.name:xmage.sha(p) for p in folder.iterdir() if p.is_file()},
        compared_committed_checkpoints={k:len(v) for k,v in agreed.items()},negative_controls=list(actual['rejections']),callback_controls=list(reference['callback_controls']),comparator_controls_detected={engine:len(values) for engine,values in controls.items()},
        acceptance_claims={'selected_actions':'Actual scalar semantic decode/apply and pinned XMage player callbacks accepted; complete choices consumed.',
            'native_rejections':'Each rejected semantic action preserves live state/RNG; complete failed envelopes preserve caller state.',
            'reference_rejections':'Selected cast membership in the engine playable-action query; actual mode/target/discard callbacks and observed payment resources. No raw off-turn cast rejection or complete legal-set claim.',
            'complete_legal_set':False},
        limits='Different raw announcement/payment staging is retained. Native selects its private discard before payment; XMage consumes its supplied discard during additional-cost payment after mana. Committed states compare with hand/battlefield membership; draw/library/stack/targets and creation-event order are ordered. Raw native modification slots and actual XMage continuous-effect durations are separately checked. No cleanup-expiry, trigger/activated-choice, combat, full-game, reference internal-RNG/rollback or M2 completion claim.',
        stdin='closed',display='unset',offline=True)
    output.write_text(json.dumps(receipt,indent=2)+'\n')
    print('Six actual played spell prefixes, repeats, strict invalid tapes and comparator controls passed.')
