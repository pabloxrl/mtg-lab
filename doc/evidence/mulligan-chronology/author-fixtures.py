"""Fixture authoring only: explicit CR 103.5 round plans, no engine output."""
import copy,json
from pathlib import Path
root=Path.cwd()
reset=json.loads((root/'fixtures/reference/full-pool-reset.json').read_text())
cases=[]; expected={}
for base in reset['cases']:
    if base['id'].endswith('-swap'): continue
    starter=base['starter']; seats=[starter,1-starter]
    for profile,limits in [('keep',[0,0]),('one',[1,0]),('unequal',[2,1]),('both',[3,3])]:
        c=copy.deepcopy(base);c['id']+='-'+profile;c['stop']='first_upkeep';c['choices']=[]
        for e in c['chance']:e['source']=None
        hands=[e['after'][:7] for e in c['chance']];libs=[e['after'][7:] for e in c['chance']]
        rounds=[0,0];kept=[False,False];seq=2;points=[];used=copy.deepcopy(c['chance']);choices=[]
        def point(boundary,actor):
            return dict(boundary=boundary,actor=actor,life=[20,20],hand=sorted(hands[actor]),library=libs[actor][:],mulligans=rounds[actor])
        for round_no in range(max(limits)+1):
            takes=[]
            for pos,s in enumerate(seats):
                if kept[s]:continue
                take=round_no<limits[pos]
                points.append(point('declaration',s))
                choice=dict(sequence=len(choices),kind='declare',actor=s,source=None,round=round_no,selection='mulligan' if take else 'keep')
                seq+=1;c['choices'].append(choice);choices.append(copy.deepcopy(choice))
                if take:takes.append(s)
                else:kept[s]=True
            for s in takes:
                # Explicit permutations: rotations of immutable creation order.
                # Starts 0/1/2 across redraws force distinct duplicate basics in
                # hand; final two positions of a multi-bottom reverse choice order.
                canonical=c['decks'][s]['occurrences'];offset=round_no+s
                after=canonical[offset:]+canonical[:offset]
                event=dict(sequence=len(used),kind='mulligan_shuffle',actor=s,source=None,before=canonical[:],after=after)
                seq+=1;c['chance'].append(event);used.append(copy.deepcopy(event))
                hands[s]=after[:7];libs[s]=after[7:];rounds[s]+=1
            for s in takes:
                points.append(point('bottom',s))
                n=rounds[s];selected=list(reversed(hands[s][:n]))
                choice=dict(sequence=len(choices),kind='bottom',actor=s,source=None,round=round_no+1,selection=selected)
                seq+=1;c['choices'].append(choice);choices.append(copy.deepcopy(choice))
                hands[s]=[h for h in hands[s] if h not in selected];libs[s]+=selected
        points.extend(point('first_upkeep',s) for s in seats)
        cases.append(c);expected[c['id']]=points
(root/'fixtures/reference/full-pool-mulligan.json').write_text(json.dumps(dict(schema_version=2,family='mulligan',pins=reset['pins'],cases=cases),indent=2)+'\n')
(root/'fixtures/reference/full-pool-mulligan-expectations.json').write_text(json.dumps(expected,indent=2)+'\n')
