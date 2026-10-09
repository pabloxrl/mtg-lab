import hashlib,json,os,subprocess
from pathlib import Path
root=Path.cwd();out=root/'doc/evidence/full-pool-baseline/profiles';out.mkdir(exist_ok=False)
raw=root/'.agent-artifacts/profiles';raw.mkdir(exist_ok=False)
capture=json.loads((root/'doc/evidence/full-pool-baseline/resident/capture.json').read_text())
fd=int(os.environ['MTG_SYMPHONY_LOCK_FD']);receipts={}
for kind,specimen in capture['specimens'].items():
    config=out/(kind+'.config.json');config.write_text(json.dumps(specimen['config'],indent=2)+'\n')
    experiment=raw/(kind+'.er')
    command=['timeout','--kill-after=5s','180s','gprofng','collect','app','-p','on','-H','on','-a','off','-o',str(experiment),
        str(root/'target/release/mtg'),'simulate','--config',str(config),'--output',str(out/(kind+'.game.jsonl'))]
    print('Independent heap/PC profile:',kind,flush=True)
    with (out/(kind+'.stdout')).open('w') as stdout,(out/(kind+'.stderr')).open('w') as stderr:
        result=subprocess.run(command,stdout=stdout,stderr=stderr,pass_fds=(fd,))
    receipt=dict(command=command,returncode=result.returncode,ordinal=specimen['ordinal'],shape=specimen['shape'],
                 interpretation='diagnostic whole normal game containing the selected stress prefix; profiling overhead excluded from throughput artifacts')
    if experiment.exists():
        for display in ['metric_list','functions','heap','statistics','overview']:
            cmd=['timeout','60s','gprofng','display','text','-'+display,str(experiment)]
            with (out/(kind+'.'+display+'.txt')).open('w') as stream:
                shown=subprocess.run(cmd,stdout=stream,stderr=subprocess.STDOUT,pass_fds=(fd,))
            receipt[display+'_returncode']=shown.returncode
        with (out/(kind+'.commands.txt')).open('w') as stream:
            subprocess.run(['timeout','30s','gprofng','display','text',str(experiment)],input='help\nquit\n',text=True,stdout=stream,stderr=subprocess.STDOUT,pass_fds=(fd,))
        receipt['raw_files']={str(p.relative_to(experiment)):dict(bytes=p.stat().st_size,sha256=hashlib.sha256(p.read_bytes()).hexdigest()) for p in experiment.rglob('*') if p.is_file() and not p.is_symlink()}
    receipts[kind]=receipt;(out/'receipt.json').write_text(json.dumps(receipts,indent=2)+'\n')
# CPU-only profiles avoid attributing heap-interposer overhead to the engine.
for policy in ['heuristic','legal-random']:
    name='cpu-'+policy;experiment=raw/(name+'.er')
    command=['timeout','--kill-after=5s','360s','gprofng','collect','app','-p','on','-H','off','-a','off','-o',str(experiment),
        str(root/'target/release/mtg'),'bench','--workload','scalar-full-pool-v1','--config',
        str(root/f'fixtures/bench/full-pool/{policy}-off.json'),'--output',str(out/(name+'.benchmark.json'))]
    print('Independent CPU-only full-pool profile:',policy,flush=True)
    with (out/(name+'.stdout')).open('w') as stdout,(out/(name+'.stderr')).open('w') as stderr:
        result=subprocess.run(command,stdout=stdout,stderr=stderr,pass_fds=(fd,))
    receipt=dict(command=command,returncode=result.returncode,profiler=True,heap_tracing=False,
                 interpretation='CPU-only software sampling; full repeated windows retained, never admitted to the off/counters throughput comparison')
    if experiment.exists():
        for display in ['metric_list','functions','statistics','overview']:
            with (out/(name+'.'+display+'.txt')).open('w') as stream:
                shown=subprocess.run(['timeout','60s','gprofng','display','text','-'+display,str(experiment)],stdout=stream,stderr=subprocess.STDOUT,pass_fds=(fd,))
            receipt[display+'_returncode']=shown.returncode
    receipts[name]=receipt;(out/'receipt.json').write_text(json.dumps(receipts,indent=2)+'\n')
