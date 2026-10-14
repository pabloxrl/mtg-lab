"""Recompute L0-005 campaign accounting from retained immutable attempt archives.

Checks measurements and provenance, not gameplay admission. Source games remain
unqualified candidates; the separately pinned complete L0 corpus supplies native
replay measurements. Private source logs are hashed but not redistributed.
"""
import argparse
from collections import Counter
import hashlib
import json
import lzma
import math
from pathlib import Path
import tarfile

import xmage_campaign as campaign
import xmage_ai_pilot as pilot
import check_xmage_pilot as pilot_evidence

ROOT=Path(__file__).resolve().parents[1]
DEFAULT=ROOT/'doc/evidence/xmage-campaign'


def need(condition, message):
    if not condition: raise ValueError('campaign: '+message)


def distribution(values):
    need(bool(values) and all(math.isfinite(x) and x>=0 for x in values), 'finite measurements')
    values=sorted(values)
    return dict(count=len(values),total=sum(values),p50=values[(len(values)-1)//2],
                p95=values[math.ceil(len(values)*.95)-1],minimum=values[0],maximum=values[-1])


def read_attempt(path):
    with tarfile.open(path,'r:xz') as archive:
        files={}
        for member in archive:
            need(member.isfile() and Path(member.name).name==member.name, 'flat regular archive')
            need(member.name not in files,'duplicate archive member')
            files[member.name]=archive.extractfile(member).read()
    def load(name):return json.loads(files[name])
    receipt=load('attempt.json'); measurement=load('measurement.json'); config=load('config.json')
    need(config==receipt['config'],'config binding')
    need(measurement['seed']==receipt['seed']==config['seed'],'seed binding')
    need(measurement['status']==receipt['status'],'status binding')
    for name,digest in receipt['artifacts'].items():
        if name.endswith('.log'):
            need(name not in files,'private source logs must not be published')
        else:
            need(hashlib.sha256(files[name]).hexdigest()==digest,'original artifact checksum '+name)
    canonical=None
    if measurement['identity']:
        converted=load('converted.json')
        canonical=pilot.canonical_game(converted['transcript'])
        need(canonical==files['canonical.json'],'canonical bytes')
        need(hashlib.sha256(canonical).hexdigest()==measurement['identity'],'canonical hash')
        need(converted['source_receipt']==receipt,'converted source receipt')
        need(receipt['status']=='complete','identity requires source completion')
    if receipt['status']=='complete':
        timing=load('java-timing.json')
        need(timing['passed'] and load('result.json')['status']=='complete','real Java completion')
        need(receipt['equivalence']['equal_return'] and receipt['equivalence']['equal_semantic_state'],
             'original recording equivalence')
        need(receipt['validation']['played_spells']>0 and receipt['validation']['attacking_combats']>0,
             'actual spells/combat')
    return receipt,measurement,canonical,(load('java-timing.json') if 'java-timing.json' in files else None),load('started.json')['started_unix']


def check(base=DEFAULT):
    base=Path(base)
    index=json.loads((base/'index.json').read_text())
    required={'declaration.json','summary.json','phase-1.json','phase-2.json','prepared.json.xz',
              'native/summary.json','native/build.json','l0/summary.json','controls/summary.json'}|set(index['attempts'])
    need(required<=set(index['files']),'complete published inventory')
    for name,digest in index['files'].items():
        need(not Path(name).is_absolute() and '..' not in Path(name).parts,'local artifact path')
        need(hashlib.sha256((base/name).read_bytes()).hexdigest()==digest,'published checksum '+name)
    declaration=json.loads((base/'declaration.json').read_text())
    need(declaration['attempts']==campaign.schedule(),'declared deterministic schedule')
    prepared=lzma.decompress((base/'prepared.json.xz').read_bytes())
    prepared_hash=hashlib.sha256(prepared).hexdigest()
    need(declaration['prepared_sha256']==prepared_hash,'prepared execution binding')
    for name,digest in json.loads(prepared)['sources'].items():
        need('sources/'+name in index['files'] and not Path(name).is_absolute()
             and '..' not in Path(name).parts,'listed local source snapshot')
        need(hashlib.sha256((base/'sources'/name).read_bytes()).hexdigest()==digest,'measured source pin '+name)
    need(len(index['attempts'])==100,'all 100 attempts retained')
    results=[];receipts=[];timings=[];starts=[];seen={};duplicates=[];actions=0
    for row,name in zip(campaign.schedule(),index['attempts']):
        receipt,result,canonical,timing,started=read_attempt(base/name)
        starts.append(started)
        need(all(result[k]==v for k,v in row.items()),'assignment binding')
        need(receipt['campaign_prepared_sha256']==prepared_hash,'attempt execution binding')
        identity=result['identity']
        if identity:
            actions+=len(json.loads(canonical)['gameplay']['play'])
            if identity in seen:
                need(seen[identity][1]==canonical,'canonical hash collision')
                duplicates.append(dict(attempt=row['attempt'],duplicate_of=seen[identity][0],identity=identity))
            else:seen[identity]=(row['attempt'],canonical)
        receipts.append(receipt);results.append(result)
        if timing:timings.append(timing)
    pilot_manifest=json.loads(pilot_evidence.DEFAULT.read_text())
    overlap=[]
    for game in pilot_manifest['games']:
        if game['id'] in seen:
            converted=pilot_evidence.document(pilot_evidence.DEFAULT.parent,pilot_manifest,game['converted'])
            need(seen[game['id']][1]==pilot.canonical_game(converted['transcript']),'pilot canonical collision')
            overlap.append(game['id'])
    # Wall timestamps and monotonic duration clocks are distinct. Completion
    # ordering is enforced by the measured driver's joined phase futures.
    need(min(starts[16:])>=max(starts[:16]),'baseline starts precede concurrent phase')
    original=json.loads((base/'summary.json').read_text())
    need(original['results']==results,'original result inventory')
    phases=original['phases']
    need(phases==[json.loads((base/f'phase-{i}.json').read_text()) for i in (1,2)],'phase receipt binding')
    need([(p['workers'],p['attempts']) for p in phases]==[(1,16),(2,84)],'single baseline then bounded concurrency')
    need(all(math.isfinite(p['elapsed_seconds']) and p['elapsed_seconds']>0 for p in phases),'positive phase elapsed')
    summary=campaign.account(results,sum(p['elapsed_seconds'] for p in phases))
    need(all(original[k]==v for k,v in summary.items()),'original aggregate accounting')
    summary.update(duplicate_attempts=duplicates,pilot_overlap_ids=overlap,conversion_classes=dict(Counter(r.get('conversion_status','not-attempted') for r in results)),
        latency_seconds={k:distribution([r[k] for r in results if k in r]) for k in ('capture_seconds','conversion_seconds','total_seconds')},
        java_seconds={k:distribution([r[k] for r in timings]) for k in ('jvm_start_to_entry_seconds','capture_and_equivalence_seconds')},
        phase_projection=[dict(p,projection=campaign.account(results[:16] if p['workers']==1 else results[16:],p['elapsed_seconds']),
            latency_seconds=distribution([r['total_seconds'] for r in results if r['workers']==p['workers']])) for p in phases],
        peak_java_rss_kib=max(r['peak_java_rss_kib'] for r in receipts),
        artifact_bytes_including_private_logs=sum(r['artifact_bytes'] for r in results),
        normalized_actions=actions,
        raw_records=sum(r['validation']['records'] for r in receipts if 'validation' in r),
        source_record_count_scope='all raw callback/event records, not normalized semantic actions',
        new_provisional_games=0,final_admitted_games=0)
    native=json.loads((base/'native/summary.json').read_text())
    need(native['complete_accounting'] and native['exit_code']==1,'diagnostic partial outcome')
    need([r['attempt'] for r in native['results']]==list(range(100)),'complete native inventory')
    native_counts=Counter(r['status'] for r in native['results'])
    need(native_counts=={'passed':93,'not-convertible':5,'divergent-or-unsupported':2},'retained native outcomes')
    need(native['provisional_admissions']==native['final_admissions']==0,'no diagnostic admissions')
    for result,diagnostic in zip(results,native['results']):
        need((result['identity'] is None)==(diagnostic['status']=='not-convertible'),'native conversion binding')
        if result['identity']:
            need(f"native/{result['attempt']:03d}.tar.xz" in index['files'],'listed native archive')
            need(result['identity']==diagnostic['identity'],'native identity binding')
            with tarfile.open(base/f"native/{result['attempt']:03d}.tar.xz",'r:xz') as archive:
                receipt=json.load(archive.extractfile('receipt.json'))
                need(receipt==diagnostic,'native receipt binding')
    for name,digest in json.loads((base/'native/build.json').read_text())['sources'].items():
        need('sources/'+name in index['files'],'listed native source snapshot')
        need(hashlib.sha256((base/'sources'/name).read_bytes()).hexdigest()==digest,'native measured source pin')
    controls=json.loads((base/'controls/summary.json').read_text())
    need(controls['campaign_attempts']==controls['quota_games']==0,'controls excluded from campaign')
    need([(r['name'],r['status'],r['exit_code']) for r in controls['results']]==
         [('unsupported','unsupported',1),('truncated','truncated',1)],'source negative controls')
    for control in controls['results']:
        prefix='controls/'+control['name']+'/'
        need(all(prefix+n in index['files'] for n in ['result.json','config.json','java-timing.json']),'listed source control')
        actual=json.loads((base/prefix/'result.json').read_text())
        need(actual['status']==control['status'] and actual['error']==control['error'],'source control result binding')
        need(not json.loads((base/prefix/'java-timing.json').read_text())['passed'],'source control rejected')
    l0=json.loads((base/'l0/summary.json').read_text())
    need(len(l0['serial'])==5 and len(l0['parallel'])==2,'L0 repeated baseline and concurrency')
    need(len({r['receipt'] for r in l0['serial']+l0['parallel']})==7,'distinct L0 invocations')
    need(l0['distinct_corpus_games']==10 and l0['repeated_executions']==140,'distinct versus repeated games')
    corpus=hashlib.sha256((ROOT/'fixtures/l0/xmage-pilot-v1.json').read_bytes()).hexdigest()
    serial=[];parallel=[]
    for group,target in [('serial',serial),('parallel',parallel)]:
        for sample in l0[group]:
            need('l0/'+sample['receipt'] in index['files'],'listed L0 receipt')
            receipt=json.loads((base/'l0'/sample['receipt']).read_text())
            need(receipt['corpus_sha256']==corpus and receipt['full_comparable_run'] and
                 receipt['exit_code']==receipt['failed']==0 and receipt['successful']==10,'complete production L0')
            need(receipt['cold_elapsed_seconds']==sample['cold'] and
                 receipt['total_invocation_seconds']==sample['total'],'L0 elapsed binding')
            for run in [receipt['native'],receipt['warm_throughput']]:
                need(run['complete'] and run['successful']==10 and run['failed']==run['unattempted']==0,
                     'complete cold/warm inventory')
                need(len(run['games'])==10 and all(g['passed'] for g in run['games']), 'all L0 games pass')
                need(run['actions']==run['checkpoints']==4886 and run['artifact_bytes']==111210783,'L0 workload')
            target.append(receipt)
    summary['native_diagnostic_classes']=dict(native_counts)
    summary['l0_serial']={
        'cold_seconds':distribution([r['cold_elapsed_seconds'] for r in serial]),
        'warm_seconds':distribution([r['warm_throughput']['elapsed_seconds'] for r in serial]),
        'per_game_cold_seconds':distribution([g['elapsed_seconds'] for r in serial for g in r['native']['games']]),
        'per_game_warm_seconds':distribution([g['elapsed_seconds'] for r in serial for g in r['warm_throughput']['games']]),
        'peak_rss_kib':max(r['peak_rss_kib'] for r in serial),
        'loaded_replay_comparison_seconds':distribution([r['native']['loaded_replay_comparison_seconds'] for r in serial])}
    summary['l0_parallel_makespan_seconds']=l0['parallel_launcher_makespan_seconds']
    summary['source_components_seconds']={k:distribution([r[k] for r in receipts]) for k in
        ['java_process_seconds','validation_seconds','archive_seconds']}
    return summary


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--evidence',type=Path,default=DEFAULT)
    args=parser.parse_args()
    print(json.dumps(check(args.evidence),indent=2,sort_keys=True))
