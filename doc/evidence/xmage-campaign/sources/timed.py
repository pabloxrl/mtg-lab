import json,resource,subprocess,sys,time
from pathlib import Path
start=time.perf_counter()
r=subprocess.run(sys.argv[2:],stdin=subprocess.DEVNULL)
Path(sys.argv[1]).write_text(json.dumps(dict(command=sys.argv[2:],elapsed_seconds=time.perf_counter()-start,peak_rss_kib=resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss,exit_code=r.returncode),indent=2)+'\n')
raise SystemExit(r.returncode)
