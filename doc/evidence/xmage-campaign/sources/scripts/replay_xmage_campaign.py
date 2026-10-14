"""Check every convertible campaign candidate against actual native execution.

This diagnostic two-repeat path preserves original/source-bound negative controls.
It does not admit games or replace the separately measured full production L0.
"""
import argparse
import hashlib
import json
import os
import subprocess
from pathlib import Path
import time

import xmage_ai_normalize as normalize
import xmage_ai_pilot as pilot
import xmage_campaign as campaign


def run(source, output):
    source=Path(source).resolve(); output=Path(output).resolve()
    completed=json.loads((source/'summary.json').read_text())
    if len(completed['results'])!=100:
        raise ValueError('complete declared campaign required before diagnostic replay')
    lock_fd=int(os.environ['MTG_SYMPHONY_LOCK_FD'])
    os.fstat(lock_fd)
    output.mkdir(parents=True,exist_ok=False)
    started=time.perf_counter()
    with (output/'build.log').open('w') as log:
        subprocess.run(['cargo','test','--release','--locked','-p','mtg-core','--lib','--no-run'],
                       cwd=campaign.ROOT,stdin=subprocess.DEVNULL,stdout=log,stderr=subprocess.STDOUT,
                       pass_fds=(lock_fd,),timeout=3600,check=True)
    pins=[Path(__file__),campaign.ROOT/'scripts/xmage_ai_normalize.py',campaign.ROOT/'scripts/xmage_ai_pilot.py',
          campaign.ROOT/'scripts/admission_observations.py',campaign.ROOT/'scripts/admission_reference.py',
          campaign.ROOT/'Cargo.lock']+sorted((campaign.ROOT/'crates/mtg-core/src').glob('*.rs'))
    campaign.write(output/'build.json',dict(elapsed_seconds=time.perf_counter()-started,status='passed',
        profile='release',sources={str(p.relative_to(campaign.ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in pins}))
    results=[]
    for row in campaign.schedule():
        original=source/f"{row['attempt']:03d}"
        measurement=json.loads((original/'measurement.json').read_text())
        if measurement.get('conversion_status')!='complete':
            results.append(dict(attempt=row['attempt'],status='not-convertible',first_difference=measurement.get('first_difference')))
            continue
        folder=output/original.name;folder.mkdir()
        started=time.perf_counter()
        receipt=dict(attempt=row['attempt'],identity=measurement['identity'],status='started')
        converted=json.loads((original/'converted.json').read_text())
        campaign.write(folder/'input.json',converted['transcript'])
        try:
            negatives=json.loads((campaign.ROOT/'fixtures/reference/xmage-ai-negative.json').read_text())
            negatives.update(pilot.source_negative_inputs(converted))
            campaign.write(folder/'negatives.json',negatives)
            os.environ['MTG_AI_REPLAY_INPUT']=str(folder/'input.json')
            os.environ['MTG_AI_REPLAY_NEGATIVES']=str(folder/'negatives.json')
            env=dict(os.environ,MTG_AI_REPLAY_OUTPUT=str(folder))
            with (folder/'native.log').open('w') as log:
                subprocess.run(['cargo','test','--release','--locked','-p','mtg-core','--lib',
                                'captured_xmage_ai_strict_replay'],cwd=campaign.ROOT,env=env,
                               stdin=subprocess.DEVNULL,stdout=log,stderr=subprocess.STDOUT,
                               pass_fds=(lock_fd,),timeout=3600,check=True)
            runs=[json.loads((folder/f'native-{i}.json').read_text())['run'] for i in (1,2)]
            receipt['repeats']=[normalize.verify_native(converted,game) for game in runs]
            actual=json.loads((folder/'native-negatives.json').read_text())
            for name,control in negatives.items():
                normalize.need(name in actual,0,'missing negative '+name)
                if control['sequence'] is not None:
                    normalize.need(len(actual[name]['run']['consumed_play'])==control['sequence'],control['sequence'],'negative cursor '+name)
            receipt.update(status='passed',negative_controls=list(negatives))
        except Exception as error:
            detail=str(error)
            native_failure=folder/'native-1.json'
            if native_failure.exists():
                detail=json.loads(native_failure.read_text()).get('error',detail)
            native_log=folder/'native.log'
            if isinstance(error,subprocess.CalledProcessError) and native_log.exists():
                lines=native_log.read_text().splitlines()
                panic=next((i for i,line in enumerate(lines) if 'panicked at' in line),None)
                if panic is not None:
                    detail='\n'.join(lines[panic:panic+4])[:2000]
                receipt['native_log_sha256']=hashlib.sha256(native_log.read_bytes()).hexdigest()
            receipt.update(status='divergent-or-unsupported',first_difference=detail)
        receipt['elapsed_seconds']=time.perf_counter()-started
        campaign.write(folder/'receipt.json',receipt)
        results.append(receipt)
        print(json.dumps(receipt),flush=True)
    exit_code=0 if all(r['status']=='passed' for r in results) else 1
    campaign.write(output/'summary.json',dict(results=results,provisional_admissions=0,final_admissions=0,
        exit_code=exit_code,complete_accounting=True,
        scope='diagnostic native/source checks only; patched reference requalification not performed'))
    return exit_code


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args();raise SystemExit(run(args.source,args.output))
