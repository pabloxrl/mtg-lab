"""Version 9 played combat; literal CR-derived oracles and privileged provenance."""
import copy
import json
from pathlib import Path
import full_pool_reference as base
import activations_reference as activations
ROOT=base.ROOT
FIXTURE=ROOT/'fixtures/reference/full-pool-combat.json'
BRIDGE=ROOT/'references/xmage/FullPoolCombatTest.java'
require=base.require

def validate(doc):
    require(type(doc.get('schema_version')) is int and doc['schema_version']==9 and doc.get('family')=='combat','/version/family')
    projected=copy.deepcopy(doc);projected.update(schema_version=5,family='activations')
    for c in projected['cases']:
        require(c['stop']=='combat/'+c['id'],'/stop');c['stop']='activations/'+c['id']
        for e in c['play']:
            selected=e['kind'] in ('attackers','blockers','damage','graveyard_order')
            require(('selection' in e)==selected,'/selection field')
            require(('event' in e)==(e['kind']=='graveyard_order'),'/graveyard event field')
            if e['kind']=='graveyard_order':require(e.pop('event')==f"combat-sba/{e['turn']}",'/graveyard event')
            if selected:require(isinstance(e.pop('selection'),list),'/selection array')
    activations.validate(projected)

def negative_inputs(doc):return json.loads((ROOT/'fixtures/reference/full-pool-combat-negatives.json').read_text())
def native(folder):return base.native(folder,family='combat')

def compare(points,engine='native'):
    doc=json.loads(FIXTURE.read_text());oracle=json.loads((ROOT/'fixtures/reference/full-pool-combat-expectations.json').read_text())
    require(set(points)==set(oracle),'/case coverage')
    for c in doc['cases']:
        name=c['id'];rows=points[name];o=oracle[name];require(bool(rows),'/empty observations')
        fields={'boundary','turn','step','active','actor','life','mana','land_plays','hand','library','graveyard','exile','battlefield','stack','incarnations','permanents','payment','activation','effects','haste','trample','combat','allocations'}
        require(all(set(p)==fields for p in rows),'/observation fields')
        for e,p in zip(c['play'][:-1],rows[:-1]):
            require(p['life']==(o['life'] if e['kind']=='graveyard_order' else [20,20]),'/independent precombat life')
            require(p['graveyard']==[[],[]] and p['exile']==[], '/independent precombat zones')
        p=rows[-1];require(p['step']=='combat_damage' and p['boundary']==f"before/{len(c['play'])-1}",'/finish')
        require(p['life']==o['life'],'/literal life/'+name)
        require(sorted(sum(p['graveyard'],[]))==sorted(o['dead']),'/literal deaths/'+name)
        ordered=[[x for x in o['dead'] if x.startswith(str(s)+'/')] for s in (0,1)]
        for e in c['play']:
            if e['kind']=='graveyard_order':ordered[e['actor']]=[x['source'] for x in e['selection']]
        require(p['graveyard']==ordered,'/literal ordered graveyard/'+name)
        for source,mark in o['marks'].items():
            if source not in o['dead']:require(p['permanents'][source]['damage']==mark,'/literal surviving damage/'+name)
        played=[e['source'] for e in c['play'] if e['kind'] in ('cast','play_land')]
        require(sorted(p['battlefield'])==sorted(set(played)-set(o['dead'])),'/literal battlefield/'+name)
        for seat in (0,1):
            draws=(p['turn']+1)//2-1 if seat==0 else p['turn']//2
            order=c['chance'][seat]['after'];dealt=order[:7+draws]
            require(p['library'][seat]==order[7+draws:] and sorted(p['hand'][seat])==sorted(x for x in dealt if x not in played),'/literal draw ledger/'+name)
        characteristics={
            'shivan-dragon':(5,5,{'flying'}),'magnigoth-sentry':(4,4,{'reach'}),
            'thornweald-archer':(2,1,{'reach','deathtouch'}),'tajuru-pathwarden':(5,4,{'vigilance','trample'}),
            'axgard-cavalry':(2,2,set()),'swab-goblin':(2,2,set()),'bear-cub':(2,2,set())}
        for q in rows:
            for source,v in q['permanents'].items():
                card=source.split('/')[1]
                if card in characteristics:
                    power,toughness,keywords=characteristics[card]
                    require((v['power'],v['toughness'])==(power,toughness),'/literal power toughness')
                    require({k for k in ('flying','reach','deathtouch','vigilance','trample') if v[k]}==keywords,'/literal keyword ledger')
        attack_sources=o['attacker'] if isinstance(o['attacker'],list) else [o['attacker']]
        for i,e in enumerate(c['play']):
            if e['kind']=='pass' and e['step']=='declare_blockers':
                require(rows[i]['combat']==[dict(attacker=dict(source=a,incarnation=3),blocked=bool(o['blockers']),blockers=[dict(source=b,incarnation=3) for b in (o['blockers'] if len(attack_sources)==1 else [o['blockers'][j]])]) for j,a in enumerate(attack_sources)],'/committed block links')
            if e['kind']=='blockers':
                q=rows[i]
                for a in attack_sources:
                    permanent=q['permanents'][a]
                    require(permanent['tapped']==('pathwarden' not in a),'/vigilance or attacker tap/'+name)
                    require(not permanent['sick'],'/sick attack/'+name)
            if e['kind']=='damage':
                q=rows[i];require(len(q['combat'])==1 and q['combat'][0]['blocked']==bool(o['blockers']),'/blocked status/'+name)
                require([v['source'] for v in q['combat'][0]['blockers']]==o['blockers'],'/block links/'+name)
        expected_allocations=[] if o['amounts'] is None else [dict(attacker=dict(source=o['attacker'],incarnation=3),amounts=[dict(target=dict(source=b,incarnation=3),amount=n) for b,n in zip(o['blockers'],o['amounts'])])]
        require(p['allocations']==expected_allocations,'/chosen allocations/'+name)
        for source in played:
            require(p['incarnations'][source]==(4 if source in o['dead'] else 2 if '/forest/' in source or '/mountain/' in source else 3),'/zone incarnation/'+name)
        if name=='pathwarden-haste':
            i=next(i for i,e in enumerate(c['play']) if e['kind']=='activate');q=rows[i]
            require(q['permanents']['1/tajuru-pathwarden/0']['sick'] and not q['permanents']['1/tajuru-pathwarden/0']['haste'],'/no printed haste')
            require(not q['permanents']['0/axgard-cavalry/0']['sick'] and not q['permanents']['0/axgard-cavalry/0']['tapped'],'/legal cavalry tap')
            after=next(q for q in rows[i:] if q['activation'] is None and q['stack'] and q['stack'][-1]['ability']=='haste')
            require(after['stack'][-1]['target']==dict(source='1/tajuru-pathwarden/0',incarnation=3),'/haste chosen target')
            require(after['permanents']['0/axgard-cavalry/0']['tapped'],'/paid cavalry tap')
            require(p['permanents']['1/tajuru-pathwarden/0']['haste'],'/resolved haste')

def check_rejections(r,engine):
    specs=negative_inputs(json.loads(FIXTURE.read_text()))
    require(set(r['rejections'])==set(specs) and set(r['negative_runs'])==set(specs),'/negative coverage')
    for name,spec in specs.items():
        require(r['rejections'][name].endswith(spec[engine]),'/rejection category/'+name)
        run=r['negative_runs'][name];i=spec['sequence']
        require(run['consumed_play']==spec['input']['cases'][0]['play'][:i],'/rejection consumed boundary/'+name)
        require(bool(run['points']),'/empty rejected observation/'+name)

def check_run(r,doc,engine):
    check_rejections(r,engine)
    compare(r['checkpoints'],engine)
    compare({k:v['points'] for k,v in r['repeat_runs'].items()},engine)
    for c in doc['cases']:
        run=r['runs'][c['id']];points=run['points']
        require(points==r['checkpoints'][c['id']] and len(points)==len(c['play']),'/complete observations')
        require(run['consumed_play']==c['play'],'/consumed choices')
        for key in ('chance','choices'):require(run['opening']['consumed_'+key]==c[key],'/opening '+key)
        o=json.loads((ROOT/'fixtures/reference/full-pool-combat-expectations.json').read_text())[c['id']]
        require(len(run['marked'])==1,'/simultaneous observation count')
        marked=run['marked'][0]
        require(marked['life']==o['life'],'/simultaneous life')
        for source,damage in o['marks'].items():
            require(source in marked['permanents'] and marked['permanents'][source]['damage']==damage,'/simultaneous marked damage/'+source)
            incarnation=marked['permanents'][source]['incarnation'] if engine=='native' else marked['incarnations'][source]
            require(incarnation==3,'/simultaneous source incarnation')
        require(len(run['legal_checks'])==2,'/missing legal checks')
        base_name=o['base']
        if base_name.startswith('shivan'):
            attackers=['0/axgard-cavalry/0','0/swab-goblin/0','0/shivan-dragon/0'];blockers=['1/thornweald-archer/0','1/magnigoth-sentry/0']
        elif base_name=='both-owners':
            attackers=['0/axgard-cavalry/0','0/swab-goblin/0'];blockers=['1/thornweald-archer/0','1/bear-cub/0','1/magnigoth-sentry/0']
        else:
            attackers=['1/thornweald-archer/0','1/bear-cub/0']
            if base_name!='thorn-multiple':attackers+=['1/magnigoth-sentry/0','1/tajuru-pathwarden/0']
            blockers=['0/swab-goblin/0'] if base_name=='pathwarden-haste' else ['0/axgard-cavalry/0','0/swab-goblin/0']
        # CR302.6 restricts attacking/tap costs, not blocking: Shivan cast
        # on turn eleven is an eligible blocker on green turn twelve.
        if base_name in ('pathwarden-single','pathwarden-multiple'):blockers+=['0/shivan-dragon/0']
        if c['id'].endswith('-identities'):
            swap=lambda s:s[:-1]+'1' if any(k in s for k in ('axgard-cavalry','swab-goblin','thornweald-archer','magnigoth-sentry')) else s
            attackers=list(map(swap,attackers));blockers=list(map(swap,blockers))
        attack_sources=o['attacker'] if isinstance(o['attacker'],list) else [o['attacker']]
        ref=lambda s:dict(source=s,incarnation=3)
        want=[dict(kind='attackers',sequence=next(e['sequence'] for e in c['play'] if e['kind']=='attackers'),legal=[ref(s) for s in attackers]),
              dict(kind='blockers',sequence=next(e['sequence'] for e in c['play'] if e['kind']=='blockers'),legal=[dict(blocker=ref(s),attacker=ref(a)) for s in blockers for a in attack_sources])]
        canonical=lambda checks:[dict(v,legal=sorted(v['legal'],key=lambda e:json.dumps(e,sort_keys=True))) for v in checks]
        require(canonical(run['legal_checks'])==canonical(want),'/literal legal declarations/'+c['id'])
        if engine=='native':require(points==r['repeat_runs'][c['id']]['points'],'/repeat exact native points')
        else:require([semantic(p) for p in points]==[semantic(p) for p in r['repeat_runs'][c['id']]['points']],'/repeat semantic points')
        for field in ('consumed_play','legal_checks'):
            require(run[field]==r['repeat_runs'][c['id']][field],'/repeat '+field)
        if engine=='native':
            require(run['policy_capture']['header']['policies']==['strict-combat-tape-v9']*2,'/capture provenance')
            require(run['records'] and run['stale_candidates_rejected']==len(run['records']),'/semantic recording or stale candidates')
        else:require(len(run['raw_occurrence_bindings'])==80,'/physical occurrence provenance')
        for i,e in enumerate(c['play']):
            p=points[i];require((p['turn'],p['step'],p['actor'])==(e['turn'],e['step'],e['actor']),'/choice boundary')

def semantic(p):
    out=activations.semantic(p)
    out['combat']=p['combat'];out['allocations']=p['allocations']
    return out

def comparator_controls(r,doc,engine):
    controls={}
    def probe(name,case,field,mutate):
        # Only the mutated field needs ownership. Large recordings, repeats and
        # other cases remain read-only; every probe starts from pristine input.
        bad=dict(r,runs=dict(r['runs']))
        bad['runs'][case]=dict(r['runs'][case])
        bad['runs'][case][field]=copy.deepcopy(r['runs'][case][field])
        mutate(bad)
        # Aliases in actual JSON files are not shared. Bind both views so that
        # rejection tests the semantic assertion, not a duplicate-view mismatch.
        bad['checkpoints']={k:v['points'] for k,v in bad['runs'].items()}
        try:check_run(bad,doc,engine)
        except (ValueError,KeyError,TypeError,IndexError) as error:controls[name]=str(error)
        else:raise ValueError('undetected comparator mutation '+name)
    probe('early-life','shivan-unblocked','points',lambda b:b['runs']['shivan-unblocked']['points'][0]['life'].__setitem__(0,19))
    probe('final-life','shivan-unblocked','points',lambda b:b['runs']['shivan-unblocked']['points'][-1]['life'].__setitem__(1,20))
    probe('simultaneous-damage','shivan-thorn','marked',lambda b:b['runs']['shivan-thorn']['marked'][0]['permanents']['0/shivan-dragon/0'].__setitem__('damage',0))
    probe('missing-simultaneous','shivan-thorn','marked',lambda b:b['runs']['shivan-thorn'].__setitem__('marked',[]))
    def combat(b):
        p=next(p for p in b['runs']['shivan-multiple']['points'] if p['step']=='combat_damage' and p['combat'])
        p['combat'][0]['blocked']=False
    probe('blocked-status','shivan-multiple','points',combat)
    probe('allocation','shivan-multiple','points',lambda b:b['runs']['shivan-multiple']['points'][-1]['allocations'][0]['amounts'][0].__setitem__('amount',3))
    probe('graveyard-order','shivan-multiple','points',lambda b:b['runs']['shivan-multiple']['points'][-1]['graveyard'][1].reverse())
    probe('zone-identity','shivan-thorn','points',lambda b:b['runs']['shivan-thorn']['points'][-1]['graveyard'][0].__setitem__(0,'0/axgard-cavalry/0'))
    probe('incarnation','shivan-thorn','points',lambda b:b['runs']['shivan-thorn']['points'][-1]['incarnations'].__setitem__('0/shivan-dragon/0',3))
    probe('library-identity','shivan-unblocked','points',lambda b:b['runs']['shivan-unblocked']['points'][-1]['library'][0].reverse())
    probe('missing-consumption','pathwarden-haste','consumed_play',lambda b:b['runs']['pathwarden-haste']['consumed_play'].pop())
    probe('legal-observation','pathwarden-haste','legal_checks',lambda b:b['runs']['pathwarden-haste'].__setitem__('legal_checks',[]))
    for f in ('stack','permanents','life','combat','allocations'):
        probe('missing-'+f,'shivan-unblocked','points',lambda b,f=f:b['runs']['shivan-unblocked']['points'][0].pop(f))
    return controls

def source_files():
    return list(dict.fromkeys([*activations.source_files(),Path(__file__),BRIDGE,ROOT/'references/xmage/CombatSbaGame.java',FIXTURE,ROOT/'tests/test_m2_repair_combat.py',ROOT/'references/xmage/played-combat-provenance.json',*sorted((ROOT/'fixtures/reference').glob('*combat*'))]))

def run(args):
    import os,shutil,subprocess,sys,tempfile
    import xmage
    output=args.output.resolve();output.parent.mkdir(parents=True,exist_ok=True);output.unlink(missing_ok=True)
    folder=Path(tempfile.mkdtemp(prefix=output.stem+'.privileged-',dir=output.parent));os.chmod(folder,0o700)
    doc=json.loads(FIXTURE.read_text());validate(doc)
    for path in (ROOT/'fixtures/reference').glob('*combat*'):
        if path.is_file():(folder/path.name).write_bytes(path.read_bytes())
    sources=source_files();hashes={str(p.relative_to(ROOT)):xmage.sha(p) for p in sources}
    (folder/'source-hashes.json').write_text(json.dumps(hashes,indent=2)+'\n')
    actual=native(folder);check_run(actual,doc,'native')
    cache=args.cache.resolve();xmage.verify_inputs(cache)
    require(xmage.dependencies(cache)==json.loads((ROOT/'references/xmage/dependencies.json').read_text()),'/dependencies')
    for family in ('mulligan','priority','activations','played-combat'):
        provenance=json.loads((ROOT/f'references/xmage/{family}-provenance.json').read_text())
        for name,sha in provenance['consulted_sources'].items():require(xmage.sha(cache/xmage.SOURCE/name)==sha,'/upstream source '+name)
    for name in ('FullPoolMulliganTest.java','FullPoolPriorityTest.java','FullPoolActivationsTest.java','FullPoolCombatTest.java','CombatSbaGame.java'):
        target=cache/xmage.SOURCE/'Mage.Tests/src/test/java/org/mage/test/mtglab'/name
        target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes((ROOT/'references/xmage'/name).read_bytes())
    result_path=folder/'xmage.json'
    command=xmage.maven(cache)+['-o','-pl','Mage.Tests','-am','test','-Dtest=org.mage.test.mtglab.FullPoolCombatTest','-Dsurefire.failIfNoSpecifiedTests=false',
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
    receipt=dict(schema_version=9,family='combat',status='agreed',reference_semantics='patched-combat-sba-v1',reference_integration_sha256=xmage.sha(ROOT/'references/xmage/CombatSbaGame.java'),bridge_sha256=xmage.sha(BRIDGE),cases=len(doc['cases']),runs_per_engine=2*len(doc['cases']),
        completion='bounded_played_combat_prefixes',pins=doc['pins'],
        upstream_commit=json.loads((ROOT/'references/xmage/pins.json').read_text())['upstream_commit'],sources=hashes,
        privileged_directory=folder.name,privileged_artifacts={p.name:xmage.sha(p) for p in folder.iterdir() if p.is_file()},
        compared_committed_checkpoints={k:len(v) for k,v in agreed.items()},negative_controls=list(actual['rejections']),callback_controls=list(reference['callback_controls']),
        
        comparator_controls_detected={engine:len(v) for engine,v in controls.items()},
        acceptance_claims={'selected_actions':'Actual scalar semantic decode/apply and pinned XMage player callbacks accepted; complete choices consumed.',
            'native_rejections':'Rejected semantic actions and envelopes preserve state/RNG.',
            'reference_rejections':'Engine playable-action membership and actual target/mana callbacks. Exact input cursor and category; no complete legal-set claim.',
            'complete_legal_set':False},
        limits='Bounded played combat only. Separate witnessed activation staging; eligible attacker and blocker-pair sets are compared, not all subset configurations. The upstream trample total-only constraint gap is explicit. Explicit owner orders are compared using the separately hashed test-only combat SBA integration. This is patched-reference agreement, not unmodified-upstream agreement; the original upstream mismatch remains archived. No complete game or M2 completion claim.',
        stdin='closed',display='unset',offline=True)
    output.write_text(json.dumps(receipt,indent=2)+'\n')
    print('Actual played combat prefixes, repeats, invalid tapes and comparator controls passed.')
