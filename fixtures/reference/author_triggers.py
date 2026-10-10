"""Author fixed trigger tapes from CR/card schedules, never observed output."""
import json,copy
from pathlib import Path
root=Path('.')
base=json.load(open('fixtures/reference/full-pool-spells.json'))
cases=[]
def scenario(name,creatures):
 c=copy.deepcopy(base['cases'][0]);c['id']=name;c['stop']='triggers_settled/'+('0/viashino-pyromancer/0' if name=='pyromancer-bite' else '0/dragon-fodder/'+('1' if name=='duplicate-archers' else '0'));c['play']=[]
 prefixes=[['0/mountain/0','0/mountain/1','0/mountain/2',*creatures,'0/dragon-fodder/0'],['1/forest/0','1/forest/1','1/bear-cub/0','1/bite-down/0','1/forest/2']]
 for s,start in enumerate(prefixes):
  old=c['chance'][s]['after']; c['chance'][s]['after']=start+[v for v in old if v not in start]
 def add(t,step,actor,kind,source=None,inc=None,color=None,role=None,mode=None,order=None,player=None):
  c['play'].append(dict(sequence=len(c['play']),turn=t,step=step,actor=actor,kind=kind,source=source,incarnation=inc,color=color,role=role,mode=mode,order=order,player=player))
 def main(t,a,k,**kw):add(t,'precombat_main',a,k,**kw)
 def cast(t,a,card,cost):
  main(t,a,'cast',source=f'{a}/{card}/0',inc=1)
  for i in range(cost):main(t,a,'tap_mana',source=f'{a}/{"mountain" if a==0 else "forest"}/{i}',inc=2)
  for i in range(cost):main(t,a,'pay',color=3 if a==0 else 4)
  main(t,a,'finish_payment')
 def passes(t,a):main(t,a,'pass');main(t,1-a,'pass')
 def key(source,event,ability):return dict(source=source,incarnation=3,ability=ability,event=event)
 finalturn=9 if name=='duplicate-archers' else 7
 for t in range(1,finalturn+1):
  a=(t-1)%2
  for step in ['upkeep']+(['draw'] if t>1 else []):add(t,step,a,'pass');add(t,step,1-a,'pass')
  land=(t-1)//2
  main(t,a,'play_land',source=f'{a}/{"mountain" if a==0 else "forest"}/{land}',inc=1)
  if a==1 and t==4:cast(t,a,'bear-cub',2);passes(t,a)
  if name=='pyromancer-bite' and t==7:
   cast(t,0,'viashino-pyromancer',2);passes(t,0)
   main(t,0,'order_triggers',order=[key('0/viashino-pyromancer/0',0,'pyromancer')])
   main(t,0,'target_player',player=1)
   main(t,0,'pass')
   main(t,1,'cast',source='1/bite-down/0',inc=1)
   main(t,1,'target',source='1/bear-cub/0',inc=3,role='bite_source')
   main(t,1,'target',source='0/viashino-pyromancer/0',inc=3,role='bite_destination')
   main(t,1,'finish_targets')
   for i in range(2):main(t,1,'tap_mana',source=f'1/forest/{i}',inc=2)
   for i in range(2):main(t,1,'pay',color=4)
   main(t,1,'finish_payment');passes(t,1);passes(t,0)
  elif a==0 and name!='pyromancer-bite':
   if t==3:
    cast(t,0,'firebrand-archer',2);passes(t,0)
   if t==5:
    card='firebrand-archer' if name=='duplicate-archers' else 'crackling-cyclops'
    cast(t,0,card,2 if card=='firebrand-archer' else 3)
    if card=='firebrand-archer':c['play'][-6]['source']='0/firebrand-archer/1'
    passes(t,0)
   if t in (7,9):
    cast(t,0,'dragon-fodder',2)
    if t==9:c['play'][-6]['source']='0/dragon-fodder/1'
    second='0/firebrand-archer/1' if name=='duplicate-archers' else '0/crackling-cyclops/0'
    main(t,0,'order_triggers',order=[key(second,(t-7)//2,'archer' if name=='duplicate-archers' else 'cyclops'),key('0/firebrand-archer/0',(t-7)//2,'archer')])
    passes(t,0);passes(t,0);passes(t,0)
  if t==finalturn:break
  for step in ['precombat_main','begin_combat','declare_attackers','end_combat','postcombat_main','end_turn']:
   if step=='declare_attackers':add(t,step,a,'empty_attackers')
   add(t,step,a,'pass');add(t,step,1-a,'pass')
 cases.append(c)
scenario('archer-cyclops',['0/firebrand-archer/0','0/crackling-cyclops/0'])
scenario('duplicate-archers',['0/firebrand-archer/0','0/firebrand-archer/1','0/dragon-fodder/1'])
scenario('pyromancer-bite',['0/viashino-pyromancer/0'])
for c in cases:
 for i,e in enumerate(c['play']):e['sequence']=i
base.update(schema_version=6,family='triggers',cases=cases)
Path('fixtures/reference/full-pool-triggers.json').write_text(json.dumps(base,indent=2)+'\n')

Path('fixtures/reference/full-pool-triggers-xmage-input.json').write_text(json.dumps(base,indent=2)+'\n')
import copy,json
from pathlib import Path
base=json.load(open('fixtures/reference/full-pool-triggers.json'))
for engine in ('native','xmage'):
 out={}
 def add(name,case,select,mutate,category):
  doc=copy.deepcopy(base);doc['cases']=[doc['cases'][case]];c=doc['cases'][0]
  index=select(c['play']);mutate(c['play'],index)
  for i,e in enumerate(c['play']):e['sequence']=i
  out[name]=dict(input=doc,sequence=index,category=category)
 order=lambda p:next(i for i,e in enumerate(p) if e['kind']=='order_triggers')
 target=lambda p:next(i for i,e in enumerate(p) if e['kind']=='target_player')
 for name,field,value in [('wrong_source','source','0/firebrand-archer/2'),('wrong_event','event',99),('wrong_incarnation','incarnation',4),('wrong_ability','ability','pyromancer')]:
  add(name,0,order,lambda p,i,f=field,v=value:p[i]['order'][0].__setitem__(f,v),'/trigger source/event')
 add('duplicate_order',0,order,lambda p,i:p[i]['order'].__setitem__(1,copy.deepcopy(p[i]['order'][0])), '/native command' if engine=='native' else '/trigger source/event')
 add('extra_order',0,order,lambda p,i:p[i]['order'].append(copy.deepcopy(p[i]['order'][0])), '/native command' if engine=='native' else '/trigger order cardinality')
 add('missing_order',0,order,lambda p,i:p.pop(i),'/native command' if engine=='native' else '/missing trigger order')
 def swap(p,i):p[i],p[i+1]=p[i+1],p[i]
 add('reordered_choice',0,order,swap,'/native command' if engine=='native' else '/missing trigger order')
 add('extra_trigger_choice',0,lambda p:order(p)+1,lambda p,i:p.insert(i,copy.deepcopy(p[i-1])),'/trigger source/event' if engine=='native' else '/unsupported priority callback')
 add('invalid_player',2,target,lambda p,i:p[i].__setitem__('player',2),'/native command' if engine=='native' else '/illegal ETB player target')
 add('creature_as_player',2,target,lambda p,i:p[i].update(kind='target',source='1/bear-cub/0',incarnation=3,role='pyromancer_target',player=None),'/native command' if engine=='native' else '/illegal ETB player target')
 def early(p,i):p[i]=copy.deepcopy(p[target(p)])
 add('target_at_cast',2,lambda p:next(i+1 for i,e in enumerate(p) if e['kind']=='cast' and e['source']=='0/viashino-pyromancer/0'),early,'/native command' if engine=='native' else '/unsupported payment callback')
 add('failed_cast',0,lambda p:next(i for i,e in enumerate(p) if e['turn']==7 and e['kind']=='pay'),lambda p,i:p[i].__setitem__('color',4),'/native command' if engine=='native' else '/insufficient or wrong-color payment')
 add('unsupported_callback',0,lambda p:2,lambda p,i:p[i].update(kind='unsupported',source=None,incarnation=None),'/unsupported spell callback' if engine=='native' else '/unsupported priority callback')
 add('truncated_tape',0,lambda p:len(p)-1,lambda p,i:p.pop(),'/missing choice before named stop')
 def extra(p,i):
  e=copy.deepcopy(p[-1]);e['actor']=0;p.append(e)
 add('extra_tape',0,lambda p:len(p),extra,'/extra choice after named stop')
 Path(f'fixtures/reference/full-pool-triggers-{engine}-negatives.json').write_text(json.dumps(out,indent=2)+'\n')
