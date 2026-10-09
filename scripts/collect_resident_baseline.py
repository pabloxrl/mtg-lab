"""Collect normal-play samples and fresh-process core-state RSS curves."""
import argparse
import json
import os
from pathlib import Path
if __package__:
    from .collect_scalar_baseline import measured_process, digest, system
    from .scalar_artifact import validate_resident
else:
    from collect_scalar_baseline import measured_process, digest, system
    from scalar_artifact import validate_resident


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary',type=Path,default=Path('target/release/mtg'))
    p.add_argument('--output',type=Path,required=True)
    a=p.parse_args()
    if 'MTG_SYMPHONY_LOCK_FD' not in os.environ: p.error('shared heavy lock required')
    fd=int(os.environ['MTG_SYMPHONY_LOCK_FD']);os.fstat(fd)
    a.output.mkdir(parents=True,exist_ok=False)
    def run(name,request):
        config=a.output/f'{name}.config.json';config.write_text(json.dumps(request,indent=2)+'\n')
        output=a.output/f'{name}.json'
        command=[str(a.binary.resolve()),
                 'bench','--workload','scalar-resident-v1','--config',str(config),'--output',str(output)]
        with (a.output/f'{name}.stdout').open('w') as out,(a.output/f'{name}.stderr').open('w') as err:
            receipt=measured_process(command,out,err,fd,1800)
        (a.output/f'{name}.process.json').write_text(json.dumps(receipt,indent=2)+'\n')
        if receipt['returncode']: raise RuntimeError(f'{name} failed; evidence retained')
        return json.loads(output.read_text())
    manifest=dict(binary_sha256=digest(a.binary),system=system(),runs={})
    (a.output/'collection.json').write_text(json.dumps(manifest,indent=2)+'\n')
    captured=run('capture',dict(operation='capture'))
    manifest['capture_sha256']=digest(a.output/'capture.json')
    manifest['source_sha256']=captured['source_sha256']
    summaries={}
    for name,specimen in captured['specimens'].items():
        path=a.output/f'{name}.specimen.json';path.write_text(json.dumps(specimen,indent=2)+'\n')
        report=run(name,dict(operation='sweep',specimen_path=str(path.resolve())))
        if digest(a.binary)!=manifest['binary_sha256']:raise RuntimeError('binary changed during resident collection')
        summaries[name]=validate_resident(report)
        manifest['runs'][name]=dict(specimen_sha256=digest(path),report_sha256=digest(a.output/f'{name}.json'))
        (a.output/'collection.json').write_text(json.dumps(manifest,indent=2)+'\n')
        (a.output/'summary.json').write_text(json.dumps(summaries,indent=2)+'\n')
    return 0


if __name__=='__main__':
    raise SystemExit(main())
