import hashlib,json,subprocess,sys,time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
root=Path(__file__).resolve().parents[2]
output=root/'.agent-artifacts/campaign/l0';output.mkdir(exist_ok=False)
start=time.perf_counter()
subprocess.run([sys.executable,'scripts/package_l0.py','--output',str(output/'package')],cwd=root,check=True)
(output/'package-time.json').write_text(json.dumps(dict(elapsed_seconds=time.perf_counter()-start)))
pin=hashlib.sha256((root/'fixtures/l0/xmage-pilot-v1.json').read_bytes()).hexdigest()
def run(name):
 start=time.perf_counter()
 receipt=output/(name+'.json')
 subprocess.run([sys.executable,'scripts/run_l0.py','--runner','target/release/mtg',
                 '--manifest',str(output/'package/manifest.json'),'--manifest-sha256',pin,
                 '--receipt',str(receipt)],cwd=root,check=True)
 result=json.loads(receipt.read_text())
 assert result['full_comparable_run']
 return dict(receipt=receipt.name,launcher_inclusive_seconds=time.perf_counter()-start,
             cold=result['cold_elapsed_seconds'],total=result['total_invocation_seconds'])
serial=[run(f'serial-{i}') for i in range(5)]
start=time.perf_counter()
with ThreadPoolExecutor(max_workers=2) as pool:
 parallel=list(pool.map(run,['parallel-0','parallel-1']))
summary=dict(serial=serial,parallel=parallel,parallel_launcher_makespan_seconds=time.perf_counter()-start,
             distinct_corpus_games=10,final_admitted_games=0,
             repeated_executions=140,scope='each invocation replays all ten provisional games cold and warm')
(output/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps(summary),flush=True)
