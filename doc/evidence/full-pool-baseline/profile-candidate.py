"""Replay the recorded independent profiles into fresh output directories."""
import argparse,hashlib,json,os,subprocess
from pathlib import Path
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--output',type=Path,required=True)
parser.add_argument('--raw',type=Path,required=True)
parser.add_argument('--capture',type=Path,default=Path('doc/evidence/full-pool-baseline/resident/capture.json'))
args=parser.parse_args()
if 'MTG_SYMPHONY_LOCK_FD' not in os.environ:parser.error('shared heavy lock required')
root=Path.cwd();out=args.output.resolve();out.mkdir(parents=True,exist_ok=False)
raw=args.raw.resolve();raw.mkdir(parents=True,exist_ok=False)
capture=json.loads(args.capture.read_text())
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
