import hashlib,json,re,subprocess
from pathlib import Path
base='503ab665372122fd7a8c08b7ba8e6ba77ce525c2'
def old(p):return subprocess.check_output(['git','show',f'{base}:{p}'])
def current(p):return Path(p).read_bytes()
def obj(p):return json.loads(current(p))
mp='doc/programs/rfc-0002.json';rp='doc/programs/rfc-0002-requirements.json';cp='doc/testing/capability-test-plan.json'
m=obj(mp);before=json.loads(old(mp));r=obj(rp);rb=json.loads(old(rp))
assert {k:v for k,v in m.items() if k!='tasks'}=={k:v for k,v in before.items() if k!='tasks'}
oldtasks={t['issue']:t for t in before['tasks']};tasks={t['issue']:t for t in m['tasks']}
assert set(oldtasks)<=set(tasks)
for n,t in oldtasks.items():
 if n not in [23,24,25]:assert tasks[n]==t,n
 else:assert {k:v for k,v in tasks[n].items() if k!='depends_on'}=={k:v for k,v in t.items() if k!='depends_on'}
for a,b in zip(rb['blocks'],r['blocks']):
 assert {k:v for k,v in a.items() if k!='owners'}=={k:v for k,v in b.items() if k!='owners'}
 assert set(a['owners'])<=set(b['owners'])
for p in [cp,'doc/rfcs/0001-project-charter.md','doc/rfcs/0002-first-mvp.md','WORKFLOW.md','README.md']:
 assert old(p)==current(p),p
new=set(tasks)-set(oldtasks)
assert len(new)==27 and {131,132,133}<=new
for n in new:assert tasks[n]['milestone']=='M2' and 80 in tasks[n]['depends_on']
from functools import lru_cache
@lru_cache(None)
def ancestors(n):
 return set(tasks[n]['depends_on'])|set().union(*(ancestors(d) for d in tasks[n]['depends_on']))
assert new<=ancestors(26)
for n in new:assert 22 in ancestors(n)
for n in [194,195,196,197,198,202,204]:assert {131,132,133}<=ancestors(n)
for n in [23,24,25]:assert set(oldtasks[n]['depends_on'])<=ancestors(n)
rows=re.findall(r'`([^`]+)` \| #(\d+) \| #(\d+)',Path('doc/programs/m2-test-crosswalk.md').read_text())
cases=[c for c in obj(cp)['cases'] if c['milestone']=='M2']
assert len(rows)==len(cases)==175 and len({row[0] for row in rows})==175
for c,(case,owner,execution) in zip(cases,rows):
 assert case==c['id'] and int(owner)==c['owner_issue'] and int(execution) in [209,210,211,212]
 assert int(execution) in ancestors(24)
from collections import Counter
report={'base_sha':base,'registered_children':sorted(new),'preserved_original_tasks':len(oldtasks),'verbatim_requirement_blocks':len(r['blocks']),'unchanged_catalog_cases':len(obj(cp)['cases']),'m2_exact_case_crosswalk':len(rows),'execution_pack_counts':dict(Counter(row[2] for row in rows)),'rfc_sha256':hashlib.sha256(current('doc/rfcs/0002-first-mvp.md')).hexdigest(),'catalog_sha256':hashlib.sha256(current(cp)).hexdigest(),'result':'PASS','claim':'planning-preservation checks only; no new engine behavior executed'}
print(json.dumps(report,indent=2))
