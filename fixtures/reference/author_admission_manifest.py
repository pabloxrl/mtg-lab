"""Author admission membership/provenance from inputs, not played output."""
import hashlib
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]

def digest(value):return hashlib.sha256(json.dumps(value,sort_keys=True,separators=(',',':')).encode()).hexdigest()
def author():
    doc=json.loads((ROOT/'fixtures/reference/full-pool-admission.json').read_text())
    cards=json.loads((ROOT/'data/cards/foundations_micro_v1.json').read_text())['cards']
    provenance={p['case']:p for p in doc['provenance']};cases={};coverage={}
    for c in doc['cases']:
        kind=c['id'].rsplit('-',1)[-1]
        source=dict(kind=kind,author='fixtures/reference/author_admission.py') if kind=='directed' else dict(kind=kind,policy='heuristic-activation-mana-v1',master=42,episode=7,author='crates/mtg-core/src/admission_origin_tests.rs',raw=provenance[c['id']])
        cases[c['id']]=dict(input_sha256=digest(c),oracle_sha256=digest(json.loads((ROOT/'fixtures/reference/full-pool-admission-expectations.json').read_text())[c['id']]),provenance=source)
    for card in cards:
        if card['kind']!='card':continue
        events=[]
        for c in doc['cases']:
            for e in c['play']:
                if e['kind'] in ('cast','play_land') and e['source'].split('/')[1]==card['id']:
                    events.append(dict(case=c['id'],sequence=e['sequence'],source=e['source'],kind=e['kind']))
        if not events:raise ValueError('no played behavior witness: '+card['id'])
        coverage[card['id']]=events
    return dict(schema_version=10,family='admission',cases=cases,played_cards=coverage,
        basis=['GR-040 initial native/XMage subset','CR103 full normal reset','CR117 priority','CR601.2h cost ordering','CR704.5a life loss','CR121.4 empty draw','CR514 cleanup'],
        reference='Pinned patched XMage: retained CombatSbaGame owner-order patch; not unmodified upstream or Forge qualification.',
        legality='Selected actions must be accepted by both actual engines. Native stale/invalid candidate rejection is mandatory. Complete cross-engine legal-set coverage is not claimed.',
        quotas=dict(rows=['RG','GR','RR','GG'],starters=[0,1],origins=['heuristic','directed'],games=16,repeats_per_engine=2))

if __name__=='__main__':
    (ROOT/'fixtures/reference/full-pool-admission-manifest.json').write_text(json.dumps(author(),indent=2)+'\n')
