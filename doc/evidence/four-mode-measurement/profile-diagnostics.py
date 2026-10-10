import hashlib,json,os,subprocess,sys
from pathlib import Path
sys.path[:0]=[str(Path.cwd()),str(Path.cwd()/'tests')]
from test_m2_repair_profile import FixedTraceProfile
from scripts.collect_scalar_baseline import system
cpu=min(os.sched_getaffinity(0));os.sched_setaffinity(0,{cpu})
os.environ['MTG_PROFILE_ARTIFACT_DIR']=str(Path('.agent-artifacts/measurement-diagnostics/profiles').resolve())
result=subprocess.run(['cargo','test','-p','mtg-cli','--release','--locked','--no-run','--message-format=json'],capture_output=True,text=True,check=True)
binary=next(Path(r['executable']) for line in result.stdout.splitlines() if (r:=json.loads(line)).get('reason')=='compiler-artifact' and r.get('executable') and r['profile']['test'] and r['target']['name']=='mtg')
before=system()
binary_hash=hashlib.sha256(binary.read_bytes()).hexdigest()
for name in ('test_real_native_traces_validate_and_measure','test_all_modes_and_capture_preserve_semantics_and_player_inputs','test_real_client_rejects_mutated_input','test_real_export_rejects_accounting_tampering'):
    adapter=FixedTraceProfile(name);adapter.binary=binary
    getattr(adapter,name)()
    print(name+' PASS',flush=True)

after=system()
assert hashlib.sha256(binary.read_bytes()).hexdigest()==binary_hash
assert all(before[k]==after[k] for k in ('affinity','cpu_quota','memory_limit'))
receipt=dict(binary_sha256=binary_hash,command=['cargo','test','-p','mtg-cli','--release','--locked','--no-run','--message-format=json'],
    before=before,after=after,scope='release test-target diagnostic adapter; separate from production throughput')
Path('.agent-artifacts/measurement-diagnostics/profile-collection.json').write_text(json.dumps(receipt,indent=2)+'\n')
