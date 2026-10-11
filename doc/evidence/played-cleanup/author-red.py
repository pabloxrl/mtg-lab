"""CR 514/104/121 fixed no-cast draw/discard schedule; no engine output."""
import json
from pathlib import Path
from author_priority import author
ROOT=Path(__file__).resolve().parents[2]

def empty_game(starter, turns=68):
    c=author(starter,False);c['id']=f'empty-library-{starter}';out=[]
    def event(t,step,actor,kind,source=None):
        out.append(dict(sequence=len(out),turn=t,step=step,actor=actor,kind=kind,source=source,incarnation=1 if source else None,color=None,role=None,mode=None))
    for turn in range(1,turns+1):
        active=starter if turn%2 else 1-starter
        for step in ['upkeep']+(['draw'] if turn>1 else [])+['precombat_main','begin_combat','declare_attackers','end_combat','postcombat_main','end_turn']:
            if step=='declare_attackers':event(turn,step,active,'empty_attackers')
            event(turn,step,active,'pass');event(turn,step,1-active,'pass')
            if turn==68 and step=='upkeep':break
        if turn>1 and turn<68:
            # Retain the opening seven, discard each newly drawn physical card.
            draw_index=(turn//2-1 if active!=starter else (turn-1)//2-1)+7
            event(turn,'cleanup',active,'discard',c['chance'][active]['after'][draw_index])
    c['play']=out;c['stop']='terminal'
    return c

if __name__=='__main__':
    doc=dict(schema_version=7,family='cleanup',pins=json.loads((ROOT/'fixtures/reference/full-pool-spells.json').read_text())['pins'],cases=[empty_game(0),empty_game(1)])
    (ROOT/'fixtures/reference/full-pool-cleanup.json').write_text(json.dumps(doc,indent=2)+'\n')
