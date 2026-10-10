import json,os,subprocess,sys
from pathlib import Path
sys.path[:0]=[str(Path.cwd()),str(Path.cwd()/'tests')]
from test_m2_repair_profile import FixedTraceProfile
cpu=min(os.sched_getaffinity(0));os.sched_setaffinity(0,{cpu})
os.environ['MTG_PROFILE_ARTIFACT_DIR']=str(Path('.agent-artifacts/measurement-diagnostics/profiles').resolve())
result=subprocess.run(['cargo','test','-p','mtg-cli','--release','--locked','--no-run','--message-format=json'],capture_output=True,text=True,check=True)
binary=next(Path(r['executable']) for line in result.stdout.splitlines() if (r:=json.loads(line)).get('reason')=='compiler-artifact' and r.get('executable') and r['profile']['test'] and r['target']['name']=='mtg')
for name in ('test_real_native_traces_validate_and_measure','test_all_modes_and_capture_preserve_semantics_and_player_inputs','test_real_client_rejects_mutated_input','test_real_export_rejects_accounting_tampering'):
    adapter=FixedTraceProfile(name);adapter.binary=binary
    getattr(adapter,name)()
    print(name+' PASS',flush=True)
