"""Recompute descriptive summaries from every retained GH-281 artifact."""
import json
from pathlib import Path
import sys

ROOT=Path(__file__).resolve().parents[3]
sys.path[:0]=[str(ROOT),str(ROOT/'tests')]
from scripts.m2_measurement import validate_archive
from test_m2_repair_timing import validate as validate_latency, EDGES
from test_m2_repair_profile import validate as validate_profile
from scripts.scalar_modes import validate_execution


def summarize(throughput, diagnostics):
    result=validate_archive(throughput)
    diagnostics=Path(diagnostics)
    native=json.loads((diagnostics/'native-latency.json').read_text())
    rows=native['runs']
    expected={(p,e,m,row) for p in ('heuristic-activation-mana-v1','legal-random-activation-mana-v1')
              for e in (False,True) for m in ('off','counters','sampled_trace','full_replay') for row in range(8)}
    if len(rows)!=128 or {(r['policy'],r['encoded'],r['mode'],r['row']) for r in rows}!=expected:
        raise ValueError('incomplete fixed-episode matrix')
    latency={};baselines={}
    for r in rows:
        identity=[r[k] for k in ('observations','history_sha256','observations_sha256','final_sha256')]
        if r['observations']<=0 or identity!=baselines.setdefault((r['policy'],r['row']),identity):
            raise ValueError('missing observations or unequal native semantics')
        if r['mode']=='off':
            if r['latency'] is not None: raise ValueError('fabricated off latency')
            continue
        validate_latency(r['latency'])
        group='/'.join((r['policy'],'encoded' if r['encoded'] else 'native',r['mode']))
        for label,phase in r['latency']['phases'].items():
            dest=latency.setdefault(group,{}).setdefault(label,dict(attempts=0,count=0,skipped=0,
                errors=0,sum_ns=0,buckets=[0]*8))
            for field in ('attempts','count','skipped','errors','sum_ns'): dest[field]+=phase[field]
            dest['buckets']=[a+b for a,b in zip(dest['buckets'],phase['buckets'])]
    for phases in latency.values():
        for phase in phases.values():
            for p in (50,95,99):
                interval=None
                if phase['count']:
                    rank=(phase['count']*p+99)//100;seen=0
                    for i,count in enumerate(phase['buckets']):
                        seen+=count
                        if seen>=rank:
                            interval=dict(lower_ns=0 if i==0 else EDGES[i-1]+1,
                                          upper_ns=EDGES[i] if i<7 else None)
                            break
                phase['p'+str(p)]=interval
    result['latency']=dict(scope='pooled deterministic samples from separate fixed episodes; not benchmark-window histograms',
        binary=native['hardware'],groups=latency,observations=sum(r['observations'] for r in rows))
    result['mode_capture']=validate_execution(json.loads((diagnostics/'mode-capture.json').read_text()))
    profiles=[];rejections=[]
    import hashlib
    for path in sorted((diagnostics/'profiles').glob('*.report.json')):
        report=json.loads(path.read_text())
        inp=path.with_name(path.name.replace('.report.json','.input.json'))
        if 'error' in report:
            rejections.append(dict(path=path.name,error=report['error']));continue
        validate_profile(report)
        if report['trace_sha256']!=hashlib.sha256(inp.read_bytes()).hexdigest():
            raise ValueError('profile input hash mismatch')
        public_fields=('trace_sha256','source_sha256','binary_sha256','toolchain_sha256',
            'dependency_sha256','rules_sha256','cards_sha256','history_sha256','observations_sha256',
            'elapsed_ns','allocations','allocated_bytes','costs','consumed','validated_checkpoints',
            'policy_ns','observations','legal_candidates','decision_kinds','encoding','effect_probe',
            'capture_sha256','captured_decisions','replay_bytes')
        profiles.append(dict(path=path.name,metrics={k:report[k] for k in public_fields if k in report}))
    if len(profiles)!=12 or len(rejections)!=7: raise ValueError('missing profile execution or rejection')
    result['profiles']=dict(valid=profiles,rejected=rejections)
    return result
