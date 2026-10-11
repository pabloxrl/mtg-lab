"""Version 7 cleanup and terminal played transcripts; privileged real observations."""
import copy
import json
from pathlib import Path
import full_pool_reference as base
import spells_reference as spells
ROOT=base.ROOT
FIXTURE=ROOT/'fixtures/reference/full-pool-cleanup.json'
require=base.require

def same(left,right):
    return base.instant_reference.difference(left,right) is None

def validate(doc):
    require(type(doc.get('schema_version')) is int and doc['schema_version']==7 and doc.get('family')=='cleanup','/version/family')
    projected=copy.deepcopy(doc);projected.update(schema_version=4,family='spells')
    for original,c in zip(doc['cases'],projected['cases']):
        require(original['stop'] in ('terminal','prefix','concession'),'/completion kind')
        c['stop']='resolved/unused'
        for e in c['play']:
            if e['kind']=='cleanup_discard':
                require(set(e)=={'sequence','turn','step','actor','kind','selection'},'/cleanup fields')
                require(isinstance(e['selection'],list),'/selection')
                for selected in e['selection']:
                    require(set(selected)=={'source','incarnation'} and isinstance(selected['source'],str) and type(selected['incarnation']) is int and selected['incarnation']>=0,'/selected fields')
                del e['selection'];e.update(kind='discard',source=None,incarnation=None,color=None,role=None,mode=None)
    spells.validate(projected)

def negative_inputs(doc):
    return json.loads((ROOT/'fixtures/reference/full-pool-cleanup-negatives.json').read_text())

def compare(points,engine="native"):

    doc=json.loads(FIXTURE.read_text())
    require(set(points)=={c['id'] for c in doc['cases']},'/case coverage')
    for c in doc['cases']:
        rows=points[c['id']];require(bool(rows),'/empty observations')
        fields={'boundary','turn','step','active','actor','life','mana','land_plays','hand','library','graveyard','exile','battlefield','stack','incarnations','permanents','payment','targeting','effects','creations','departures','outcome'}
        require(all(set(p)==fields for p in rows),'/missing/extra observation fields')
        require(all(same(p['life'],[20,20]) for p in rows),'/independent life ledger')
        last=rows[-1]
        if c['stop']=='concession':
            require(same(last['outcome'],dict(winner=1,losses=['Concession',None])),'/separate concession outcome')
            continue
        if c['stop']=='prefix':
            require(last['outcome'] is None,'/premature terminal')
            if c['id']=='stacked-expiry':
                require((last['turn'],last['step'])==(9,'upkeep'),'/expiry stop')
                for token in ('token/0/0/0','token/0/0/1'):
                    p=last['permanents'][token];require((p['power'],p['toughness'],p['damage'])==(1,1,0),'/atomic expiration')
                before=next(p for p in rows if p['turn']==8 and p['step']=='end_turn')
                p=before['permanents']['token/0/0/0'];require((p['power'],p['toughness'],p['damage'])==(9,7,2),'/stacked independent ledger')
                if engine=='xmage':
                    effects=before['effects'];require(len(effects)==3 and all(e['duration']=='EndOfTurn' for e in effects),'/independent duration ledger')
                    require(sorted(source for e in effects for source in e['sources'])==['0/goblin-surprise/0','1/giant-growth/0','1/giant-growth/1'],'/independent effect sources')
            else:
                require(last['turn']==3 and last['step']=='upkeep','/discard finish')
                require(last['graveyard']==[[],[c['chance'][1]['after'][7]]],'/discard occurrence')
            continue
        starter=c['starter'];losses=[None,None];losses[1-starter]='EmptyDraw'
        require(same(last['outcome'],dict(winner=starter,losses=losses)),'/outcome')
        require(last['turn']==68 and last['step']=='draw' and last['actor'] is None,'/terminal boundary')
        require(last['library']==[[],[]],'/empty libraries')
        require(last['graveyard']==[([] if engine=='xmage' and s!=starter else c['chance'][s]['after'][7:]) for s in (0,1)],'/ordered discard ledger')
        require([sorted(h) for h in last['hand']]==[([] if engine=='xmage' and s!=starter else sorted(c['chance'][s]['after'][:7])) for s in (0,1)],'/retained opening hand')

def native(folder):return base.native(folder,family='cleanup')

def check_rejections(result,engine):
    specs=negative_inputs(json.loads(FIXTURE.read_text()))
    require(set(result['rejections'])==set(specs) and set(result['negative_runs'])==set(specs),'/negative coverage')
    for name,spec in specs.items():
        prefix=f"first divergence: /play/{spec['sequence']} " if engine=='xmage' else 'first divergence: '
        require(result['rejections'][name]==prefix+spec[engine],'/intended rejection/'+name)
        run=result['negative_runs'][name]
        require(len(run['consumed_play'])==spec['sequence'] and run['consumed_play']==spec['input']['cases'][0]['play'][:spec['sequence']],'/rejected tape consumption/'+name)
        require(bool(run['points']),'/empty rejected observations/'+name)

def check_run(result,engine="native"):
    check_rejections(result,engine)
    compare(result['checkpoints'],engine)
    for c in json.loads(FIXTURE.read_text())['cases']:
        r=result['runs'][c['id']];rows=r['points']
        if engine=='native':require(r['policy_capture']['header']['policies']==['strict-cleanup-tape-v7']*2,'/capture policy provenance')
        require(rows==result['checkpoints'][c['id']] and len(rows)==len(c['play']),'/complete checkpoint stream')
        require(same(r['consumed_play'],c['play']),'/consumed tape')
        require(rows[-1]['boundary']=='finish','/omitted final checkpoint')
        for i,e in enumerate(c['play'][:-1]):
            p=rows[i];require(p['boundary']==f'before/{i}' and p['outcome'] is None,'/premature terminal')
            require((p['turn'],p['step'],p['actor'])==(e['turn'],e['step'],e['actor']),'/choice position')
        for k in ('chance','choices'):require(r['opening']['consumed_'+k]==c[k],'/opening consumption')
        if c['stop']=='terminal':require(any(not p['library'][1-c['starter']] and p['outcome'] is None for p in rows),'/empty library is not a loss')
        cleanup=r['cleanup_checkpoints']
        expected_turns=list(range(1,68)) if c['stop']=='terminal' else list(range(1,9)) if c['id']=='stacked-expiry' else [] if c['stop']=='concession' else [1,2]
        require([p['turn'] for p in cleanup]==expected_turns,'/missing/reordered cleanup observation')
        for p in cleanup:
            require(p['boundary']=='cleanup-settled' and p['step']=='cleanup' and p['actor'] is None,'/settled cleanup boundary')
            require(p['outcome'] is None,'/intermediate lethal cleanup')
            require(all(len(h)<=7 for h in p['hand']),'/discard settlement')
            require(all(v['damage'] in (None,0) for v in p['permanents'].values()),'/damage removal')
            require(not p['effects'],'/effect expiration')
        if c['id']=='stacked-expiry':
            require('token/0/0/0' in cleanup[-1]['permanents'],'/intermediate lethal cleanup')
            p=cleanup[-1]['permanents']['token/0/0/0']
            require((p['power'],p['toughness'],p['damage'])==(1,1,0),'/simultaneous cleanup')
        if engine=='native' and c['stop']=='terminal':
            footer=r['policy_capture']['footer'];rewards=[-1,-1];rewards[c['starter']]=1
            require(same(footer['returns'],rewards) and footer['complete'] is True and footer['end']=='Completed','/native terminal accounting')
            decisions=r['policy_capture']['decisions']
            require(sum(d['terminated'] for d in decisions)==1,'/terminal exactly once')
            for seat in (0,1):require(sum(d['reward'][seat] for d in decisions)+footer['boundary_reward'][seat]==rewards[seat],'/terminal reward sum')
        if engine=='native' and c['stop']=='concession':
            footer=r['policy_capture']['footer'];require(footer['returns']==[-1,1] and footer['boundary_reward']==[-1,1],'/concession accounting')
        repeat=result['repeat_runs'][c['id']]
        require(same(repeat['consumed_play'],r['consumed_play']),'/repeat choices')
        require(same([semantic(p) for p in repeat['points']],[semantic(p) for p in rows]),'/repeat checkpoints')

def semantic(p):
    out=spells.semantic(p);out['outcome']=p['outcome'];return out

def check_additional_cleanup(points):
    cases=json.loads((ROOT/'fixtures/reference/cleanup.json').read_text())['cases']
    require(set(points)=={c['id'] for c in cases},'/synthetic cleanup coverage')
    for c in cases:
        rows=points[c['id']];expected=2 if c['trigger'] else 1
        require(len(rows)==expected,'/missing additional cleanup observation/'+c['id'])
        require([p['ordinal'] for p in rows]==list(range(1,expected+1)),'/additional cleanup order')
        require(all(p['step']=='CLEANUP' and p['turn']==1 for p in rows),'/additional cleanup boundary')

def comparator_controls(result,engine):
    controls={}
    probes=[('winner',['checkpoints','empty-library-0',-1,'outcome','winner'],1),
            ('reason',['checkpoints','empty-library-0',-1,'outcome','losses',1],'Concession'),
            ('premature-terminal',['runs','empty-library-0','points',0,'outcome'],{'winner':0,'losses':[None,'EmptyDraw']}),
            ('unused-suffix',['runs','discard-prefix','consumed_play'],[]),
            ('missing-cleanup',['runs','stacked-expiry','cleanup_checkpoints'],[]),
            ('damage-not-removed',['runs','stacked-expiry','cleanup_checkpoints',-1,'permanents','token/0/0/0','damage'],2),
            ('boost-not-expired',['runs','stacked-expiry','cleanup_checkpoints',-1,'permanents','token/0/0/0','toughness'],7),
            ('discard-order',['checkpoints','discard-prefix',-1,'graveyard'],[['0/mountain/0'],[]]),
            ('final-checkpoint',['runs','discard-prefix','points',-1,'boundary'],'before/33')]
    if engine=='native':probes += [('limit-as-completion',['runs','empty-library-0','policy_capture','footer','end'],{'Truncated':'limit'}),('wrong-reward',['runs','empty-library-0','policy_capture','footer','returns'],[-1,1])]
    for name,path,value in probes:
        # Copy the mutated path only; unchanged full-game negative captures are
        # large, read-only evidence and need not be duplicated for every fault.
        bad=copy.copy(result);row=bad;original=result
        for k in path[:-1]:
            row[k]=copy.copy(original[k]);row=row[k];original=original[k]
        row[path[-1]]=value
        diff=base.instant_reference.difference(result,bad);require(diff is not None,'/ineffective control/'+name)
        try:check_run(bad,engine)
        except ValueError as error:controls[name]=dict(first_difference=diff,rejection=str(error))
        else:raise ValueError('undetected comparator control: '+name)
    return controls

def run(args):
    import os,sys,tempfile,shutil,subprocess
    import xmage
    output=args.output.resolve();output.parent.mkdir(parents=True,exist_ok=True);output.unlink(missing_ok=True)
    folder=Path(tempfile.mkdtemp(prefix=output.stem+'.privileged-',dir=output.parent));os.chmod(folder,0o700)
    sources=spells.source_files()+[FIXTURE,Path(__file__).resolve(),ROOT/'fixtures/reference/author_cleanup.py',ROOT/'references/xmage/FullPoolCleanupTest.java',ROOT/'references/xmage/CleanupTest.java',ROOT/'tests/test_m2_repair_cleanup.py',ROOT/'fixtures/reference/full-pool-cleanup-negatives.json',ROOT/'fixtures/reference/full-pool-cleanup-oracle.md',ROOT/'references/xmage/played-cleanup-provenance.json',ROOT/'scripts/cleanup_reference.py',ROOT/'scripts/terminal_reference.py',ROOT/'references/xmage/TerminalTest.java',ROOT/'fixtures/reference/cleanup.json',ROOT/'fixtures/reference/cleanup-expectations.json']
    hashes={str(p.relative_to(ROOT)):xmage.sha(p) for p in sources}
    validate(json.loads(FIXTURE.read_text()))
    actual=native(folder);check_run(actual)
    cache=args.cache.resolve();xmage.verify_inputs(cache)
    require(xmage.dependencies(cache)==json.loads((ROOT/'references/xmage/dependencies.json').read_text()),'/dependencies')
    provenance=json.loads((ROOT/'references/xmage/played-cleanup-provenance.json').read_text())
    for name,digest in provenance['consulted_sources'].items():require(xmage.sha(cache/xmage.SOURCE/name)==digest,'/upstream source '+name)
    for name in ('FullPoolMulliganTest.java','FullPoolPriorityTest.java','FullPoolSpellsTest.java','FullPoolCleanupTest.java'):
        target=cache/xmage.SOURCE/'Mage.Tests/src/test/java/org/mage/test/mtglab'/name
        target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes((ROOT/'references/xmage'/name).read_bytes())
    result_path=folder/'xmage.json'
    command=xmage.maven(cache)+['-o','-pl','Mage.Tests','-am','test','-Dtest=org.mage.test.mtglab.FullPoolCleanupTest','-Dsurefire.failIfNoSpecifiedTests=false','-Dmtglab.fixture='+str(FIXTURE),'-Dmtglab.root='+str(ROOT),'-Dmtglab.output='+str(result_path),'-DargLine=-Djava.awt.headless=true']
    xmage.bounded(command,cache/xmage.SOURCE,xmage.environment(cache),folder/'xmage.log',600)
    reference=json.loads(result_path.read_text());check_run(reference,"xmage")
    for name,rows in actual['checkpoints'].items():
        other=reference['checkpoints'][name]
        left=[semantic(p) for p in rows];right=[semantic(p) for p in other]
        if rows[-1]['outcome'] is not None:
            observed=reference['runs'][name]['before_concession' if name=='concession-only' else 'before_departure']
            require(observed is not None,'/missing loss-before-departure observation')
            right[-1]=semantic(observed);right[-1]['outcome']=other[-1]['outcome'];right[-1]['actor']=other[-1]['actor']
        for i,(a,b) in enumerate(zip(rows,other)):
            if any(p['payment'] is not None or p['targeting'] is not None for p in (a,b)):
                left[i]=right[i]=None
        diff=base.instant_reference.difference(left,right)
        require(diff is None,'/committed/'+name+' '+str(diff))
        diff=base.instant_reference.difference([semantic(p) for p in actual['runs'][name]['cleanup_checkpoints']],[semantic(p) for p in reference['runs'][name]['cleanup_checkpoints']])
        require(diff is None,'/cleanup/'+name+' '+str(diff))
    controls={engine:comparator_controls(r,engine) for engine,r in [('native',actual),('xmage',reference)]}
    (folder/'comparator-controls.json').write_text(json.dumps(controls,indent=2)+'\n')
    # Reexecute preserved synthetic real-engine evidence; never count it as
    # normal-reset reachability or mix it into the two natural game endings.
    xmage.bounded([sys.executable,str(ROOT/'scripts/cleanup_reference.py'),'--cache',str(cache),'--output',str(folder/'synthetic-cleanup')],ROOT,os.environ,folder/'synthetic-cleanup.log',600)
    for repeat in ('0','1'):
        observed={p.stem:json.loads(p.read_text()) for p in (folder/'synthetic-cleanup'/repeat/'cleanup-observations').glob('*.json')}
        check_additional_cleanup(observed)
        bad=copy.deepcopy(observed);bad['rules-continuous-hand-size-cleanup-interaction'].pop()
        try:check_additional_cleanup(bad)
        except ValueError as error:(folder/('additional-cleanup-control-'+repeat+'.txt')).write_text(str(error)+'\n')
        else:raise ValueError('missing additional cleanup observation accepted')
    xmage.bounded([sys.executable,str(ROOT/'scripts/terminal_reference.py'),'--cache',str(cache),'--output',str(folder/'synthetic-terminal.json')],ROOT,os.environ,folder/'synthetic-terminal.log',600)
    for path in (FIXTURE,ROOT/'fixtures/reference/full-pool-cleanup-negatives.json',ROOT/'fixtures/reference/full-pool-cleanup-oracle.md'):(folder/path.name).write_bytes(path.read_bytes())
    toolchain={}
    for name,cmd in [('java',['java','-version']),('maven',['mvn','-version']),('rustc',['rustc','-Vv']),('python',[sys.executable,'--version'])]:
        r=subprocess.run(cmd,capture_output=True,text=True,timeout=30,check=True,stdin=subprocess.DEVNULL)
        toolchain[name]=dict(version=r.stdout+r.stderr,executable_sha256=xmage.sha(Path(shutil.which(cmd[0])).resolve()))
    (folder/'toolchain.json').write_text(json.dumps(toolchain,indent=2)+'\n')
    require(hashes=={str(p.relative_to(ROOT)):xmage.sha(p) for p in sources},'/source changed during execution')
    for p in folder.rglob("*"):
        os.chmod(p,0o700 if p.is_dir() else 0o600)
    receipt=dict(schema_version=7,family='cleanup',status='agreed',cases=5,natural_terminal_games=2,concessions=1,prefixes=2,repeats_per_engine=2,
        pins=json.loads(FIXTURE.read_text())['pins'],upstream_commit=json.loads((ROOT/'references/xmage/pins.json').read_text())['upstream_commit'],
        sources=hashes,privileged_directory=folder.name,privileged_artifacts={str(p.relative_to(folder)):xmage.sha(p) for p in folder.rglob('*') if p.is_file()},
        actual_choices={k:len(r['consumed_play']) for k,r in actual['runs'].items()},cleanup_observations={k:len(r['cleanup_checkpoints']) for k,r in actual['runs'].items()},
        negative_controls=list(actual['rejections']),comparator_controls={k:len(v) for k,v in controls.items()},
        synthetic_evidence={'additional_cleanup':'synthetic-cleanup/receipt.json','simultaneous_loss_257':'synthetic-terminal.json'},
        limits='Normal reset explicit no-cast empty-library games and stacked spell/cleanup prefix only. Synthetic pending-trigger additional cleanup and simultaneous-loss fixtures remain separate. XMage departure clears zones: pre-departure/pre-concession state and finalized outcome have separate raw provenance. Committed states compare; raw spell announcement/payment staging differs. No complete legal-set, reference internal-RNG/rollback, corpus admission, new trigger/combat support or M2 completion claim.',stdin='closed',display='unset',offline=True)
    output.write_text(json.dumps(receipt,indent=2)+'\n')
    print('Played cleanup/terminal family, strict negatives and separate synthetic evidence passed.')
