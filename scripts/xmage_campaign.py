"""Declared L0-005 source campaign: unchanged ComputerPlayer7 capture, measured separately.

Build once; each attempt has its own JVM and output directory. Concurrent JVMs
read shared immutable classes/dependencies and never run Maven or compile.
"""
import argparse
from collections import Counter
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
import os
from pathlib import Path
import platform
import resource
import subprocess
import sys
import time
import xml.etree.ElementTree as ET

import xmage
import xmage_ai_capture as capture
import xmage_ai_normalize as normalize
import xmage_ai_pilot as pilot

ROOT = Path(__file__).resolve().parents[1]


def write(path, value):
    with Path(path).open('x') as stream:
        json.dump(value, stream, sort_keys=True, indent=2)
        stream.write('\n')


def schedule():
    rows = [(a,b,s) for a,b in [('red','green'),('green','red'),('red','red'),('green','green')] for s in (0,1)]
    return [dict(attempt=i, seed=323000+i, decks=list(rows[i%8][:2]), starter=rows[i%8][2],
                 workers=1 if i<16 else 2) for i in range(100)]


def account(attempts, elapsed):
    if any(a['status'] not in ('complete','failed','truncated','unsupported') for a in attempts):
        raise ValueError('unfinished or unknown source attempt')
    complete = [a for a in attempts if a['status']=='complete']
    identities = [a['identity'] for a in complete if a.get('identity')]
    distinct = len(set(identities))
    return dict(attempts=len(attempts), source_classes=dict(Counter(a['status'] for a in attempts)),
                source_complete=len(complete), source_completion_rate=len(complete)/len(attempts) if attempts else 0,
                distinct_complete=distinct, duplicates=len(identities)-distinct,
                identity_unavailable=len(complete)-len(identities), elapsed_seconds=elapsed,
                projected_1000_generation_seconds=elapsed*1000/distinct if distinct else None,
                projection_scope='observed source-complete canonical games, not admitted-game quota')


def prepare(cache, output):
    cache, output = Path(cache).resolve(), Path(output).resolve()
    output.mkdir(parents=True, exist_ok=False)
    start=time.perf_counter()
    xmage.verify_inputs(cache)
    if xmage.dependencies(cache)!=json.loads((ROOT/'references/xmage/dependencies.json').read_text()):
        raise ValueError('dependency lock mismatch')
    report=cache/xmage.SOURCE/'Mage.Tests/target/surefire-reports/TEST-org.mage.test.mtglab.NeutralSmokeTest.xml'
    properties={p.attrib['name']:p.attrib['value'] for p in ET.parse(report).findall('./properties/property')}
    classpath=properties['java.class.path']
    # Compile only the unchanged bridge and timing shell, outside upstream outputs.
    classes=output/'classes';classes.mkdir()
    command=['javac','-cp',classpath,'-d',str(classes),str(capture.BRIDGE),str(ROOT/'references/xmage/CampaignCapture.java')]
    xmage.bounded(command,ROOT,xmage.environment(cache),output/'compile.log',180)
    sources=[capture.BRIDGE,ROOT/'references/xmage/CampaignCapture.java',Path(__file__)]
    # Pin every file reachable through the classpath, including directory classes/resources.
    files={}
    for entry in [classes]+[Path(p) for p in classpath.split(os.pathsep)]:
        for path in sorted(entry.rglob('*')) if entry.is_dir() else [entry]:
            if path.is_file(): files[str(path)]=xmage.sha(path)
    result=dict(classpath=str(classes)+os.pathsep+classpath, cache=str(cache), files=files,
                sources={str(p.relative_to(ROOT)):xmage.sha(p) for p in sources},
                compilation_seconds=time.perf_counter()-start,
                peak_rss_kib=resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss)
    write(output/'prepared.json',result)
    return output/'prepared.json'


def attempt(prepared_path, folder, row):
    prepared=json.loads(Path(prepared_path).read_text()); folder=Path(folder).resolve()
    folder.mkdir(parents=True,exist_ok=False,mode=0o700)
    started=time.perf_counter()
    config=dict(seed=row['seed'],starter=row['starter'],decks=row['decks'],max_records=100000,
                timeout_seconds=1800,unsupported_probe=False)
    write(folder/'config.json',config)
    # A started receipt is retained even if the worker is externally interrupted.
    write(folder/'started.json',dict(config=config,started_unix=time.time()))
    pins=['references/xmage/RawAiCaptureTest.java','scripts/xmage_ai_capture.py','scripts/xmage.py',
          'data/cards/foundations_micro_v1.json','data/rules/cr-2026-09-25.json',
          'references/xmage/pins.json','references/xmage/dependencies.json','references/xmage/raw-capture-provenance.json']
    upstream=['Mage.Server.Plugins/Mage.Player.AI/src/mage/player/ai/ComputerPlayer.java',
              'Mage.Server.Plugins/Mage.Player.AI.MAD/src/mage/player/ai/ComputerPlayer6.java',
              'Mage.Server.Plugins/Mage.Player.AI.MAD/src/mage/player/ai/ComputerPlayer7.java',
              'Mage/src/main/java/mage/util/RandomUtil.java','Mage/src/main/java/mage/game/GameImpl.java',
              'Mage/src/main/java/mage/players/PlayerImpl.java']
    receipt=dict(schema_version=1,seed=row['seed'],config=config,status='started',admitted=False,
                 reference_variant='unmodified-upstream-raw',pins={p:xmage.sha(ROOT/p) for p in pins},
                 toolchain=json.loads((ROOT/'references/xmage/pins.json').read_text()),
                 upstream_sources={p:xmage.sha(Path(prepared['cache'])/xmage.SOURCE/p) for p in upstream},
                 campaign_prepared_sha256=xmage.sha(prepared_path),jvm_limits={'game':'-Xmx2g'})
    command=['java','-Xmx2g','-Djava.awt.headless=true','-Dmtglab.root='+str(ROOT),
             '-Dmtglab.output='+str(folder),'-cp',prepared['classpath'],'org.mage.test.mtglab.CampaignCapture']
    try:
        t=time.perf_counter()
        try:
            xmage.bounded(command,folder,xmage.environment(Path(prepared['cache'])),folder/'xmage.log',1800)
        finally:
            receipt['java_process_seconds']=time.perf_counter()-t
            if (folder/'result.json').exists(): receipt.update(json.loads((folder/'result.json').read_text()))
        if 'Game end on critical error' in (folder/'xmage.log').read_text():
            raise ValueError('source game ended on critical error')
        if receipt['status']!='complete': raise ValueError('source capture did not complete')
        t=time.perf_counter()
        receipt['validation']=capture.validate_trace(folder/'trace.jsonl.gz')
        receipt['validation_seconds']=time.perf_counter()-t
    except Exception as error:
        if receipt['status'] in ('started','complete'):
            receipt['status']='truncated' if 'timeout after' in str(error) else 'failed'
        receipt['command_error']=str(error)
    t=time.perf_counter()
    receipt['trace_content']=[]
    try:
        for path in sorted(folder.glob('*.jsonl.gz')): receipt['trace_content'].append(capture.pack_trace(path))
    except Exception as error:
        receipt.update(status='failed',archive_error=str(error))
    receipt['archive_seconds']=time.perf_counter()-t
    receipt['elapsed_seconds']=time.perf_counter()-started
    receipt['peak_java_rss_kib']=resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
    receipt['artifacts']={p.name:xmage.sha(p) for p in folder.iterdir() if p.is_file()}
    write(folder/'attempt.json',receipt)
    result=dict(row,status=receipt['status'],identity=None,capture_seconds=receipt['elapsed_seconds'])
    if receipt['status']=='complete':
        t=time.perf_counter()
        try:
            converted=normalize.normalize(folder/'trace.jsonl.xz')
            write(folder/'converted.json',converted)
            canonical=pilot.canonical_game(converted['transcript'])
            (folder/'canonical.json').write_bytes(canonical)
            result.update(identity=hashlib.sha256(canonical).hexdigest(),conversion_status='complete')
        except Exception as error:
            result.update(conversion_status='unsupported-or-divergent',first_difference=str(error))
        result['conversion_seconds']=time.perf_counter()-t
    result['total_seconds']=time.perf_counter()-started
    result['artifact_bytes']=sum(p.stat().st_size for p in folder.rglob('*') if p.is_file())
    write(folder/'measurement.json',result)
    return result


def run(prepared_path, output):
    prepared=json.loads(Path(prepared_path).read_text())
    for path,digest in prepared['files'].items():
        if xmage.sha(path)!=digest: raise ValueError('prepared classpath changed: '+path)
    for path,digest in prepared['sources'].items():
        if xmage.sha(ROOT/path)!=digest: raise ValueError('prepared source changed: '+path)
    output=Path(output).resolve();output.mkdir(parents=True,exist_ok=False,mode=0o700)
    rows=schedule()
    write(output/'declaration.json',dict(schema_version=1,attempts=rows,prepared_sha256=xmage.sha(prepared_path),
          hardware=dict(platform=platform.platform(),cpu_count=os.cpu_count(),
                        cpu_info=Path('/proc/cpuinfo').read_text(),
                        memory_info=Path('/proc/meminfo').read_text(),
                        cgroup={p:Path('/sys/fs/cgroup',p).read_text() for p in ('cpu.max','memory.max')}),
          policy='16 single-worker attempts then 84 at two workers; no retries or replacements'))
    def worker(row):
        folder=output/f"{row['attempt']:03d}"
        command=[sys.executable,str(Path(__file__).resolve()),'attempt','--prepared',str(Path(prepared_path).resolve()),
                 '--output',str(folder),'--index',str(row['attempt'])]
        subprocess.run(command,check=True,stdin=subprocess.DEVNULL)
        return json.loads((folder/'measurement.json').read_text())
    phases=[];results=[]
    for workers,batch in [(1,rows[:16]),(2,rows[16:])]:
        start=time.perf_counter()
        with ThreadPoolExecutor(max_workers=workers) as pool:
            for result in pool.map(worker,batch):
                results.append(result)
                print(json.dumps(result), flush=True)
        phases.append(dict(workers=workers,elapsed_seconds=time.perf_counter()-start,attempts=len(batch)))
        write(output/f'phase-{workers}.json',phases[-1])
        if workers == 1:
            rss=max(json.loads((output/f"{r['attempt']:03d}"/'attempt.json').read_text())['peak_java_rss_kib'] for r in results)
            if not any(r['status']=='complete' for r in results) or rss>2500000:
                raise ValueError('single-worker baseline does not support bounded two-worker probe')
    # Canonical equality, not just a hash match, establishes duplicates.
    seen={}
    for result in results:
        identity=result['identity']
        if identity:
            content=(output/f"{result['attempt']:03d}"/'canonical.json').read_bytes()
            if identity in seen and seen[identity]!=content: raise ValueError('canonical hash collision')
            seen[identity]=content
    summary=account(results,sum(p['elapsed_seconds'] for p in phases))
    summary.update(phases=phases,results=results)
    write(output/'summary.json',summary)


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command',choices=['prepare','run','attempt'])
    parser.add_argument('--cache',type=Path)
    parser.add_argument('--prepared',type=Path)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--index',type=int)
    args=parser.parse_args()
    if args.command=='prepare': print(prepare(args.cache,args.output))
    elif args.command=='run': run(args.prepared,args.output)
    else: attempt(args.prepared,args.output,schedule()[args.index])
