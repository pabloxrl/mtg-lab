import json,os,subprocess,time
from pathlib import Path
root=Path(__file__).resolve().parents[2]
prepared=json.loads((root/'.agent-artifacts/campaign/prepared/prepared.json').read_text())
output=root/'.agent-artifacts/campaign/controls';output.mkdir(exist_ok=False)
results=[]
for name,extra,status,reason in [('unsupported',{'unsupported_probe':True},'unsupported','unsupported callback: chooseRingBearer'),('truncated',{'max_records':12},'truncated','truncated: record limit')]:
 folder=output/name;folder.mkdir()
 config=dict(seed=323999,starter=0,decks=['red','green'],max_records=100000,timeout_seconds=1800,unsupported_probe=False,**{})
 config.update(extra);(folder/'config.json').write_text(json.dumps(config))
 command=['java','-Xmx2g','-Djava.awt.headless=true','-Dmtglab.root='+str(root),'-Dmtglab.output='+str(folder),'-cp',prepared['classpath'],'org.mage.test.mtglab.CampaignCapture']
 start=time.perf_counter()
 with (folder/'java.log').open('w') as log:
  result=subprocess.run(command,cwd=folder,stdin=subprocess.DEVNULL,stdout=log,stderr=subprocess.STDOUT,timeout=60)
 actual=json.loads((folder/'result.json').read_text())
 assert result.returncode!=0 and actual['status']==status and reason in actual['error'],actual
 results.append(dict(name=name,exit_code=result.returncode,status=actual['status'],error=actual['error'],elapsed_seconds=time.perf_counter()-start))
(output/'summary.json').write_text(json.dumps(dict(results=results,campaign_attempts=0,quota_games=0),indent=2)+'\n')
print(json.dumps(results))
