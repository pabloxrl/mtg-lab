"""Frozen GH-281 run order. Uses the delivered collector without adding a CLI."""
import json
import os
from pathlib import Path
import sys
import time

ROOT=Path(__file__).resolve().parents[3]
sys.path.insert(0,str(ROOT))
from scripts.scalar_modes import collect_report, MODES
from scripts.collect_scalar_baseline import digest, system
from scripts.m2_measurement import POLICIES, needs_extension, validate_campaign


def run(binary, destination):
    directory=Path(destination)
    directory.mkdir(parents=True,exist_ok=False)
    binary=Path(binary).resolve()
    cpu=min(os.sched_getaffinity(0));os.sched_setaffinity(0,{cpu})
    manifest=dict(schema_version=1,plan_sha256=digest(Path(__file__).with_name('plan.md')),
                  script_sha256=digest(Path(__file__)), binary_sha256=digest(binary),
                  started_utc=time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime()),system=system(),runs=[])
    path=directory/'campaign.json'
    def save(): path.write_text(json.dumps(manifest,indent=2)+'\n')
    save()
    reports=[]
    for encoded in (False,True):
        for policy in POLICIES:
            for mode in MODES:
                config=json.loads((ROOT/'fixtures/bench/scalar-four-modes-v1.json').read_text())
                config.update(encoding=encoded,policies=[policy]*2,instrumentation=mode)
                if mode=='sampled_trace': config['trace']=dict(every=64,capacity=256)
                label=f'{policy}-{int(encoded)}-{mode}'
                for extension in range(2):
                    name=label+('-extended' if extension else '')
                    row=dict(name=name,config=config.copy(),status='running')
                    manifest['runs'].append(row);save()
                    print('Collecting '+name,flush=True)
                    try:
                        report=collect_report(binary,config,directory/name)
                        reports.append(report)
                        row.update(status='collected',report_sha256=digest(directory/name/'report.json'),
                                   needs_extension=needs_extension(report['completed_games_per_second']['samples']))
                    except (ValueError,OSError) as error:
                        row.update(status='failed',error=str(error));save();break
                    save()
                    if extension or not row['needs_extension']: break
                    config['windows']=10
    try:
        if any(r['status']!='collected' for r in manifest['runs']):
            raise ValueError('campaign contains unsuccessful attempts; see every retained run')
        manifest['summary']=validate_campaign(reports)
        manifest['status']='validated-needs-independent-review'
    except ValueError as error:
        manifest.update(status='failed',error=str(error))
    manifest['finished_utc']=time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime())
    save()
    return manifest
