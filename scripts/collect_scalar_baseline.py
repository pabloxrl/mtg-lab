"""Collect all four declared scalar runs under the caller's shared heavy lock.

No build or lock substitution is performed. Run from the repository root with
an already verified release executable. Failed reports/logs remain in the new
output directory; no retry overwrites or fastest-run selection.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import statistics
import subprocess
import time

if __package__:
    from .scalar_artifact import validate_report, validate_suite
else:
    from scalar_artifact import validate_report, validate_suite

ROOT=Path(__file__).resolve().parents[1]
ORDER=('heuristic-off','legal-random-counters','legal-random-off','heuristic-counters')


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def system():
    result={}
    for key,path in [('cpu_quota','/sys/fs/cgroup/cpu.max'),('memory_limit','/sys/fs/cgroup/memory.max'),
                     ('cpu_usage','/sys/fs/cgroup/cpu.stat'),('memory_current','/sys/fs/cgroup/memory.current'),
                     ('loadavg','/proc/loadavg')]:
        try: result[key]=Path(path).read_text().strip()
        except OSError as error: result[key]={'unavailable':str(error)}
    result['affinity']=sorted(os.sched_getaffinity(0))
    return result


def measured_process(command, out, err, lock_fd, timeout):
    """Linux wait4 supplies per-child CPU seconds and RSS without GNU time."""
    started=time.monotonic()
    process=subprocess.Popen(command,stdout=out,stderr=err,pass_fds=(lock_fd,))
    expired=False
    while True:
        pid,status,usage=os.wait4(process.pid,os.WNOHANG)
        if pid:
            process.returncode=os.waitstatus_to_exitcode(status)
            return dict(returncode=process.returncode,timed_out=expired,
                        wall_seconds=time.monotonic()-started,
                        process_user_seconds=usage.ru_utime,process_system_seconds=usage.ru_stime,
                        peak_rss_bytes=usage.ru_maxrss*1024,minor_faults=usage.ru_minflt,
                        major_faults=usage.ru_majflt,voluntary_switches=usage.ru_nvcsw,
                        involuntary_switches=usage.ru_nivcsw,oracle='Linux wait4 rusage; ru_maxrss KiB converted to bytes')
        if time.monotonic()-started>timeout and not expired:
            process.kill();expired=True
        time.sleep(.05)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary',type=Path,default=ROOT/'target/release/mtg')
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args()
    if 'MTG_SYMPHONY_LOCK_FD' not in os.environ:
        parser.error('caller must hold the shared heavy-work lock')
    lock_fd=int(os.environ['MTG_SYMPHONY_LOCK_FD']); os.fstat(lock_fd)
    if any(os.environ.get(k) for k in ('LD_PRELOAD','LD_AUDIT','GPROFNG_COLLECTOR_EXPNAME')):
        parser.error('profiler/loader instrumentation contaminates throughput')
    args.output.mkdir(parents=True,exist_ok=False)
    args.binary=args.binary.resolve()
    cpu=min(os.sched_getaffinity(0)); os.sched_setaffinity(0,{cpu})
    manifest=dict(schema_version=1,plan='doc/evidence/full-pool-baseline/plan.md',
                  plan_sha256=digest(ROOT/'doc/evidence/full-pool-baseline/plan.md'),
                  binary_sha256=digest(args.binary),started_utc=time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime()),
                  context='shared Linux container; one logical CPU; no physical-core reservation or designated-host qualification',
                  runs=[],system=system())
    manifest_path=args.output/'collection.json'
    manifest_path.write_text(json.dumps(manifest,indent=2)+'\n')
    failure=False
    for name in ORDER:
        original=json.loads((ROOT/f'fixtures/bench/full-pool/{name}.json').read_text())
        for extension in range(2):
            config=dict(original)
            if extension: config['windows']=10
            label=name+('-extended' if extension else '')
            config_path=args.output/f'{label}.config.json';config_path.write_text(json.dumps(config,indent=2)+'\n')
            raw=args.output/f'{label}.raw.json';report_path=args.output/f'{label}.json'
            command=[str(args.binary),'bench','--workload',config['workload'],
                     '--config',str(config_path),'--output',str(raw)]
            before=system()
            print(f'Collecting {label}; every raw interval is retained',flush=True)
            with (args.output/f'{label}.stdout').open('w') as out,(args.output/f'{label}.stderr').open('w') as err:
                receipt=measured_process(command,out,err,lock_fd,3700)
            receipt.update(contaminated=False,profiler=False,heavy_lock=True,
                           before=before,after=system(),command=command)
            row=dict(name=label,returncode=receipt['returncode'],receipt=receipt)
            if raw.exists():
                report=json.loads(raw.read_text())
                receipt['contaminated']=(digest(args.binary)!=manifest['binary_sha256']
                    or report.get('hardware',{}).get('binary_sha256')!=manifest['binary_sha256']
                    or report.get('hardware',{}).get('cpu_affinity')!=str(cpu)
                    or any(receipt['before'][k]!=receipt['after'][k] for k in ('cpu_quota','memory_limit','affinity')))
                report['collection']=receipt
                report_path.write_text(json.dumps(report,indent=2)+'\n')
                row.update(raw_sha256=digest(raw),report_sha256=digest(report_path))
                try: row['summary']=validate_report(report)
                except ValueError as error: row['validation_error']=str(error);failure=True
                samples=report.get('completed_games_per_second',{}).get('samples',[])
                cv=statistics.pstdev(samples)/statistics.mean(samples) if samples and statistics.mean(samples)>0 else None
                row['window_cv']=cv
            else: row['validation_error']='missing raw artifact';failure=True;cv=None
            manifest['runs'].append(row);manifest_path.write_text(json.dumps(manifest,indent=2)+'\n')
            if receipt['returncode'] or cv is None or cv<=.1: break
    try:
        reports=[json.loads((args.output/(row['name']+'.json')).read_text()) for row in manifest['runs']]
        manifest['comparison']=validate_suite(reports)
    except (ValueError,OSError) as error:
        manifest['comparison_error']=str(error);failure=True
    manifest['status']='failed' if failure else 'collected-needs-correctness-review'
    manifest_path.write_text(json.dumps(manifest,indent=2)+'\n')
    return int(failure)


if __name__=='__main__':
    raise SystemExit(main())
