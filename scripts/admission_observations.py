"""Rules/card-derived assertions for version 10 intermediate observations.

This audits delivered engine observations; it neither plays games nor supplies
state to an engine. Cost ledgers use the frozen choices and pinned card costs.
"""
import copy
import json
import re
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
DEFINITIONS={c['id']:c for c in json.loads((ROOT/'data/cards/foundations_micro_v1.json').read_text())['cards']}
CARDS={k:c['characteristics'] for k,c in DEFINITIONS.items()}


def require(ok,path):
    if not ok:raise ValueError('first divergence: '+path)


def ref(e):return {k:e[k] for k in ('source','incarnation')}


def cost(card):
    colored=[0]*6;generic=0
    for symbol in re.findall(r'\{([^}]+)\}',CARDS[card]['mana_cost']):
        if symbol.isdigit():generic+=int(symbol)
        else:colored['WUBRGC'.index(symbol)]+=1
    return colored,generic


def structure(p,case,engine,index,prefix):
    identities(p,case,engine,index)
    # Card/occurrence identity is checked at every callback, including costs.
    require(len(p['battlefield'])==len(set(p['battlefield'])) and set(p['permanents'])==set(p['battlefield']),prefix+'/permanent identity')
    require(len(p['mana'])==2 and all(len(row)==6 and all(type(n) is int and n>=0 for n in row) for row in p['mana']),prefix+'/mana')
    for seat in (0,1):
        order=case['chance'][seat]['after'];remaining=p['library'][seat]
        require(remaining==order[len(order)-len(remaining):],prefix+'/library occurrence order')
    if p['outcome'] is None:
        cards=[source for field in ('hand','library','graveyard') for zone in p[field] for source in zone]
        cards+=p['battlefield']+p['exile']+[s['source'] for s in p['stack'] if 'ability' not in s]
        cards=[s for s in cards if not s.startswith('token/')]
        expected_cards=[s for d in case['decks'] for s in d['occurrences']]
        require(sorted(cards)==sorted(expected_cards),prefix+'/physical occurrence conservation')
    for source,v in p['permanents'].items():
        fields={'card','owner','controller','tapped','sick','power','toughness','damage','flying','reach','vigilance','deathtouch','haste','trample'}
        require(set(v)==fields,prefix+'/permanent fields')
        require(v['card']==('goblin-token' if source.startswith('token/') else source.split('/')[1]),prefix+'/permanent card')
        require(type(v['owner']) is int and v['owner'] in (0,1) and type(v['controller']) is int and v['controller'] in (0,1),prefix+'/permanent owner/controller')
        require(all(type(v[k]) is bool for k in ('tapped','sick','flying','reach','vigilance','deathtouch','haste','trample')),prefix+'/permanent flags')
        require(source in p['incarnations'],prefix+'/permanent incarnation')
    check_effects(p,engine,prefix)


def check(run,case,engine):
    transaction=None;seen={};departed=[];created=[]
    for i,(p,e) in enumerate(zip(run['points'],case['play'])):
        prefix=f'/checkpoint/{i}'
        structure(p,case,engine,i,prefix)
        # Cumulative departure witnesses must reference a previously witnessed
        # incarnation, never merely a currently matching card name.
        require(p['departures'][:len(departed)]==departed,prefix+'/departure history')
        for d in p['departures'][len(departed):]:
            if engine=='native':
                require(set(d)=={'action','object','old_handle_valid'} and d['old_handle_valid'] is False,prefix+'/departure fields')
                source=d['object']['source'];inc=d['object']['incarnation']
            else:
                require(set(d)=={'source','from','to','action','departing_incarnation'} and d['from']=='BATTLEFIELD' and d['to'] in ('GRAVEYARD','EXILED'),prefix+'/departure fields')
                source=d['source'];inc=d['departing_incarnation']
            require(type(d['action']) is int and 0<=d['action']<=i,prefix+'/departure action')
            require((source,inc) in seen,prefix+'/departure identity')
            require(source not in p['permanents'] or p['incarnations'][source]!=inc,prefix+'/departure still live')
        departed=p['departures']
        require(p['creations'][:len(created)]==created,prefix+'/creation history')
        for c in p['creations'][len(created):]:
            require(c['id'] not in {v['id'] for v in created} and c['id'].startswith('token/'),prefix+'/creation identity')
            require(c['id'] in p['permanents'] and p['permanents'][c['id']]['card']=='goblin-token',prefix+'/creation observation')
        created=p['creations']
        for source in p['permanents']:seen[source,p['incarnations'][source]]=True
        for field in ('payment','activation'):
            value=p[field]
            if value is None:continue
            require(transaction is not None and transaction['kind']==field,prefix+'/'+field+' transaction')
            t=transaction
            if field=='activation' and engine=='native':
                require(set(value)=={'ability','actor','paid','pool','reserved','source','sources','target'},prefix+'/activation fields')
                expected=dict(ability=t['ability'],actor=t['actor'],paid=sum(t['colored'])+t['generic']==0,pool=t['pool'],reserved=t['reserved'],source=t['reference'],sources=t['source_refs'],target=t['target'])
                # Tap-only haste has no mana payment flag in the native API.
                if t['ability']=='haste':expected['paid']=False
                require(value==expected,prefix+'/activation ledger')
            else:
                fields={'source','actor','pool','sources'}
                has_cost=engine=='native' or e['kind'] in ('tap_mana','pay','activation_pay','finish_payment','finish_activation','finish_targets')
                if has_cost:fields|={'colored','generic'}
                if engine=='native':fields|={'targets','mode','discard'}
                require(set(value)==fields,prefix+'/'+field+' fields')
                expected=dict(source=t['reference']['source'],actor=t['actor'],pool=t['pool'],sources=[r['source'] for r in t['source_refs']])
                if has_cost:expected.update(colored=t['colored'],generic=t['generic'])
                if engine=='native':expected.update(targets=t['targets'],mode=t['mode'],discard=t['discard'])
                require(value==expected,prefix+'/'+field+' cost/choice ledger')
        if p['targeting'] is not None:
            require(engine=='native' and transaction is not None,prefix+'/targeting callback')
            t=transaction;roles={'target':'Growth' if e['role']=='growth_target' else 'BiteSource' if e['role']=='bite_source' else 'BiteDestination','finish_targets':'Complete'}
            require(e['kind'] in roles and p['targeting']==dict(source=t['reference']['source'],role=roles[e['kind']],selected=[a['object'] for a in t['targets']]),prefix+'/targeting selected identities')
        if transaction is not None:
            t=transaction;expected=copy.deepcopy(t['before'])
            stable=('active','turn','step','land_plays','life','mana','library','graveyard','exile','incarnations','permanents','combat','allocations')
            hands=copy.deepcopy(expected['hand']);stack=copy.deepcopy(expected['stack'])
            if engine=='xmage':
                seat=t['actor'];source=t['reference']['source']
                expected['mana'][seat]=t['pool']
                for mana_source in t['source_refs']:expected['permanents'][mana_source['source']]['tapped']=True
                if t['kind']=='payment':
                    hands[seat].remove(source);expected['incarnations'][source]+=1
                    stack.append(dict(source=source,incarnation=expected['incarnations'][source],action=t['action'],targets=t['targets'],mode=t['mode']))
                    if t['discard'] is not None:
                        discarded=t['discard']['source'];hands[seat].remove(discarded)
                        expected['graveyard'][seat].append(discarded);expected['incarnations'][discarded]+=1
                else:
                    if t['ability']=='haste' and e['kind']=='finish_activation':expected['permanents'][source]['tapped']=True
                    stack.append(dict(source=source,incarnation=t['reference']['incarnation'],action=t['action'],object=f"ability/{t['action']}",ability=t['ability'],target=t['target']))
            for field in stable:require(p[field]==expected[field],prefix+'/transaction footprint/'+field)
            require([sorted(h) for h in p['hand']]==[sorted(h) for h in hands],prefix+'/transaction footprint/hand')
            clean=lambda values:[{k:v for k,v in item.items() if k!='raw_id'} for item in values]
            require(clean(p['stack'])==clean(stack),prefix+'/transaction footprint/stack')
        # Update the independent cost/selection ledger only after checking the
        # before-choice observation. All values come from frozen input/card text.
        k=e['kind']
        if k in ('cast','activate'):
            require(transaction is None,prefix+'/nested transaction')
            card=e['source'].split('/')[1];colored,generic=cost(card)
            ability=None
            if k=='activate':
                ability={'shivan-dragon':'power','wildheart-invoker':'invoker','axgard-cavalry':'haste'}[card]
                colored=[0,0,0,int(ability=='power'),0,0];generic=8 if ability=='invoker' else 0
            transaction=dict(kind='payment' if k=='cast' else 'activation',reference=ref(e),actor=e['actor'],pool=list(p['mana'][e['actor']]),colored=colored,generic=generic,source_refs=[],targets=[],mode=None,discard=None,reserved=[0]*6,ability=ability,target=None,before=copy.deepcopy(p),action=i)
        elif transaction is not None:
            t=transaction
            if k=='tap_mana':
                color=3 if '/mountain/' in e['source'] else 4
                require(e['source'].split('/')[1] in ('mountain','forest','llanowar-elves','druid-of-the-cowl'),prefix+'/mana source type')
                t['pool'][color]+=1;t['source_refs'].append(ref(e))
            elif k in ('pay','activation_pay'):
                color=e['color'];require(t['pool'][color]>0,prefix+'/payment availability');t['pool'][color]-=1;t['reserved'][color]+=1
                if t['colored'][color]>0:t['colored'][color]-=1
                else:require(sum(t['colored'])==0 and t['generic']>0,prefix+'/payment color');t['generic']-=1
            elif k=='target':t['targets'].append(dict(role=e['role'],object=ref(e)))
            elif k=='activation_target':t['target']=ref(e)
            elif k=='mode':t['mode']=e['mode']
            elif k=='discard':t['discard']=ref(e)
            elif k in ('finish_payment','finish_activation'):
                require(sum(t['colored'])+t['generic']==0,prefix+'/unpaid commit');transaction=None
    require(transaction is None,'/unfinished transaction')


def identities(p,case,engine,index):
    """Required nested identities; raw engine identifiers stay witnessed."""
    prefix=f'/checkpoint/{index}'
    births=set();uuids=set()
    for c in p['creations']:
        if engine=='native':
            require(set(c)=={'action','birth','card','definition_hash','id','incarnation'},prefix+'/creation fields')
            require(type(c['birth']) is int and c['birth']>=80 and c['birth'] not in births,prefix+'/creation birth');births.add(c['birth'])
            require(c['card']=='goblin-token' and c['definition_hash']==DEFINITIONS['goblin-token']['content_sha256'],prefix+'/creation definition')
        else:
            require(set(c)=={'id','raw_uuid','raw_counter','red','white','blue','black','green','subtypes','incarnation','action','source'},prefix+'/creation fields')
            require(isinstance(c['raw_uuid'],str) and len(c['raw_uuid'])==36 and c['raw_uuid'] not in uuids,prefix+'/creation raw identity');uuids.add(c['raw_uuid'])
            require(type(c['raw_counter']) is int and c['raw_counter']>=0,prefix+'/creation raw counter')
            require(c['red'] is True and all(c[k] is False for k in ('white','blue','black','green')) and c['subtypes']==['GOBLIN'],prefix+'/creation characteristics')
            require(c['source'].split('/')[1] in ('dragon-fodder','goblin-surprise'),prefix+'/creation source')
        require(c['incarnation']==0 and type(c['action']) is int and 0<=c['action']<=index,prefix+'/creation incarnation/action')
    for s in p['stack']:
        if s.get('ability')=='trigger':
            raw={'raw_birth','raw_source_handle'} if engine=='native' else {'raw_uuid','raw_source','raw_source_zcc'}
            require(set(s)=={'ability','key','controller','target','source_card'}|raw,prefix+'/stack trigger fields')
            key=s['key'];require(set(key)=={'source','incarnation','ability','event'},prefix+'/stack trigger key')
            require(any(key in e['order'] for e in case['play'][:index+1] if e['kind']=='order_triggers'),prefix+'/stack trigger event/source')
            require(s['source_card']==key['source'].split('/')[1] and s['controller'] in (0,1),prefix+'/stack trigger controller/card')
            require(s['target'] is None or type(s['target']) is int and s['target'] in (0,1),prefix+'/stack trigger target')
        elif 'ability' in s:
            raw={'raw_birth'} if engine=='native' else {'raw_id'}
            require(set(s)=={'source','incarnation','action','object','ability','target'}|raw,prefix+'/stack activation fields')
            action=s['action'];require(type(action) is int and 0<=action<index,prefix+'/stack activation action')
            event=case['play'][action]
            require(event['kind']=='activate' and ref(event)==ref(s) and s['object']==f'ability/{action}',prefix+'/stack activation identity')
            require(s['ability']=={'shivan-dragon':'power','wildheart-invoker':'invoker','axgard-cavalry':'haste'}.get(s['source'].split('/')[1]),prefix+'/stack activation ability')
            if s['target'] is not None:require(set(s['target'])=={'source','incarnation'},prefix+'/stack activation target')
        else:
            require(set(s)=={'source','incarnation','action','targets','mode'},prefix+'/stack spell fields')
            action=s['action'];require(type(action) is int and 0<=action<index,prefix+'/stack spell action')
            event=case['play'][action]
            require(event['kind']=='cast' and event['source']==s['source'] and s['incarnation']==event['incarnation']+1,prefix+'/stack spell identity')
            for t in s['targets']:
                require(set(t)=={'role','object'} and t['role'] in ('growth_target','bite_source','bite_destination'),prefix+'/stack spell target fields')
                if t['object'] is not None:require(set(t['object'])=={'source','incarnation'},prefix+'/stack spell target identity')
        if 'raw_birth' in s:require(type(s['raw_birth']) is int and s['raw_birth']>=80,prefix+'/stack raw birth')
        if 'raw_source_handle' in s:require(set(s['raw_source_handle'])=={'store','epoch','slot','generation'} and all(type(v) is int and v>=0 for v in s['raw_source_handle'].values()),prefix+'/stack raw source handle')
        for field in ('raw_uuid','raw_source','raw_id'):
            if field in s:require(isinstance(s[field],str) and len(s[field])==36,prefix+'/stack raw identity')


def check_effects(p,engine,prefix):
    if engine=='native':
        modifications={}
        for effect in p['effects']:
            require(set(effect)=={'object','boost','power_boost','damage'},prefix+'/effect fields')
            source=effect['object']['source'];inc=effect['object']['incarnation']
            require(source not in modifications and source in p['permanents'] and p['incarnations'][source]==inc,prefix+'/effect identity')
            require(all(type(effect[k]) is int and effect[k]>=0 for k in ('boost','power_boost','damage')),prefix+'/effect values')
            modifications[source]=effect
        for source,v in p['permanents'].items():
            card=CARDS[v['card']]
            if card['power'] is None:continue
            m=modifications.get(source,dict(boost=0,power_boost=0,damage=0))
            require((v['power'],v['toughness'],v['damage'])==(int(card['power'])+m['boost']+m['power_boost'],int(card['toughness'])+m['boost'],m['damage']),prefix+'/effect characteristics')
    else:
        raw_ids=set()
        for effect in p['effects']:
            require(set(effect)=={'raw_id','class','duration','sources'},prefix+'/effect fields')
            require(effect['raw_id'] and effect['raw_id'] not in raw_ids and effect['duration']=='EndOfTurn',prefix+'/effect identity/duration');raw_ids.add(effect['raw_id'])
            require(effect['sources'] and all(s in p['incarnations'] for s in effect['sources']),prefix+'/effect source')
            # Independently inspected pinned card constructors; hashes are
            # retained in references/xmage/admission-provenance.json.
            classes={'BoostTargetEffect':{'giant-growth','wildheart-invoker'},'BoostControlledEffect':{'goblin-surprise'},'GainAbilityTargetEffect':{'wildheart-invoker','axgard-cavalry'},'BoostSourceEffect':{'shivan-dragon','crackling-cyclops'}}
            name=effect['class'].removeprefix('mage.abilities.effects.common.continuous.')
            require(name in classes and all(s.split('/')[1] in classes[name] for s in effect['sources']),prefix+'/effect class/source')


def xmage_trigger_provenance(run,case):
    """Bind canonical trigger keys to actual construction/trigger witnesses.

    The delivered bridge normalizes a zone-change counter by subtracting the
    counter witnessed at construction. StackMomentSourceZCC preserves that
    source incarnation even if the permanent subsequently leaves (CR400.7,
    603.3 and 113.7a). It is not an arbitrary UUID-shaped diagnostic field.
    """
    import uuid
    def valid_uuid(value):
        try:return isinstance(value,str) and str(uuid.UUID(value))==value
        except (ValueError,AttributeError):return False
    bindings=run.get('raw_occurrence_bindings')
    require(isinstance(bindings,dict),'/trigger provenance physical bindings')
    by_occurrence={}
    for raw,value in bindings.items():
        require(valid_uuid(raw) and set(value)=={'occurrence','initial_counter'},'/trigger provenance binding fields')
        initial=value['initial_counter']
        if value['occurrence'].startswith('token/'):
            # Tokens were not present at deck construction. Their independent
            # CREATED_TOKEN callback carries the actual initial counter.
            births=[c for c in run['points'][-1]['creations'] if c['id']==value['occurrence'] and c['raw_uuid']==raw]
            require(initial is None and len(births)==1,'/trigger provenance token birth binding')
            initial=births[0]['raw_counter']
        require(type(initial) is int and initial>=0,'/trigger provenance initial counter')
        require(value['occurrence'] not in by_occurrence,'/trigger provenance duplicate binding')
        by_occurrence[value['occurrence']]=(raw,initial)
    key_fields=('source','incarnation','ability','event')
    key=lambda t:tuple(t[f] for f in key_fields)
    expected={key(k):(k,e['sequence']) for e in case['play'] if e['kind']=='order_triggers' for k in e['order']}
    traces=run.get('trigger_provenance')
    require(isinstance(traces,list),'/missing trigger provenance')
    require(len(traces)==len(expected),'/trigger provenance count')
    seen={}
    classes={'archer':'mage.abilities.common.SpellCastControllerTriggeredAbility','cyclops':'mage.abilities.common.SpellCastControllerTriggeredAbility','pyromancer':'mage.abilities.common.EntersBattlefieldTriggeredAbility'}
    for trace in traces:
        require(set(trace)==set(key_fields)|{'raw_ability','raw_source','raw_source_zcc','ability_class','observed_choice'},'/trigger provenance fields')
        k=key(trace);require(k in expected and k not in seen,'/trigger provenance key/event')
        canonical,ordered=expected[k];source=trace['source']
        require(source in by_occurrence,'/trigger provenance source binding')
        raw,initial=by_occurrence[source]
        require(trace['raw_source']==raw and type(trace['raw_source_zcc']) is int and trace['raw_source_zcc']==initial+trace['incarnation'],'/trigger provenance source incarnation binding')
        require(valid_uuid(trace['raw_ability']) and trace['ability_class']==classes[trace['ability']],'/trigger provenance ability identity/class')
        observed=trace['observed_choice']
        require(type(observed) is int and 0<=observed<=ordered and case['play'][observed]['turn']==case['play'][ordered]['turn'],'/trigger provenance observation chronology')
        # One targeted trigger is announced before its target callback. Preserve
        # that actual staging; its stack witness appears by the next checkpoint.
        nearby=run['points'][observed:min(ordered+2,len(run['points']))]
        require(any(canonical in [v['key'] for v in p['pending_triggers']]+[v['key'] for v in p['stack'] if v.get('ability')=='trigger'] for p in nearby),'/trigger provenance callback witness')
        seen[k]=trace
    witnessed=set();raw_stack=run.get('raw_stack_ids')
    require(isinstance(raw_stack,dict),'/trigger provenance raw stack index')
    points=run['points']+run['marked']+run['cleanup_checkpoints']+[run['before_departure']]
    for point in points:
        for ability in point['stack']:
            if ability.get('ability')!='trigger':continue
            k=key(ability['key']);require(k in seen,'/trigger provenance missing stack key')
            trace=seen[k]
            require((ability['raw_uuid'],ability['raw_source'],ability['raw_source_zcc'])==(trace['raw_ability'],trace['raw_source'],trace['raw_source_zcc']),'/trigger provenance stack source binding')
            require(ability['raw_uuid'] in raw_stack,'/trigger provenance raw stack witness')
            record=raw_stack[ability['raw_uuid']]
            require(all(record[f]==ability[f] for f in ('key','raw_uuid','raw_source','raw_source_zcc','source_card')),'/trigger provenance raw stack consistency')
            witnessed.add(k)
    require(witnessed==set(expected),'/trigger provenance unobserved stack instance')


def xmage_effect_lifecycle(run,case):
    """Independent presence/lifetime ledger for the pinned temporary effects.

    CR608.2b/611.2c: a legally resolving spell/ability creates the card-defined
    effects, with fixed recipients. CR514.2 expires them at cleanup. This ledger
    reads witnessed stack removal, not the effect list it validates. It never
    creates a game state or supplies a choice to an engine.

    Pinned BoostTarget/Source/ControlledEffect records persist until cleanup,
    including when their recipients leave. GainAbilityTargetEffect additionally
    discards itself when its fixed target leaves; ContinuousEffectsList removes
    discarded records. The consulted source hashes are retained in provenance.
    """
    from collections import Counter
    steps='untap upkeep draw precombat_main begin_combat declare_attackers declare_blockers combat_damage end_combat postcombat_main end_turn cleanup'.split()
    rank={s:i for i,s in enumerate(steps)}
    points=run['points'][:-1]+[run['before_departure']]
    def identity(s):
        if s.get('ability')=='trigger':return ('trigger',json.dumps(s['key'],sort_keys=True))
        if 'ability' in s:return ('activation',s['action'])
        return ('spell',s['source'],s['incarnation'],s['action'])
    def live(target,p):
        return target is not None and target['source'] in p['permanents'] and p['incarnations'][target['source']]==target['incarnation']
    entries=[]
    for i in range(1,len(points)):
        before,after=points[i-1],points[i]
        remaining={identity(s) for s in after['stack']}
        removed=[s for s in before['stack'] if identity(s) not in remaining]
        if not removed:continue
        event=case['play'][i-1]
        require(event['kind']=='pass' and len(removed)==1 and removed[0]==before['stack'][-1],'/effect resolution witness')
        s=removed[0];effects=[];target=None
        if s.get('ability')=='trigger':
            source=s['key']['source']
            if s['key']['ability']=='cyclops':effects=['BoostSourceEffect']
        elif 'ability' in s:
            source=s['source'];target=s['target']
            if s['ability']=='power':effects=['BoostSourceEffect']
            elif live(target,before):
                effects=['BoostTargetEffect','GainAbilityTargetEffect'] if s['ability']=='invoker' else ['GainAbilityTargetEffect']
        else:
            source=s['source'];card=source.split('/')[1]
            if card=='giant-growth':
                require(len(s['targets'])==1,'/effect Growth target witness');target=s['targets'][0]['object']
                if live(target,before):effects=['BoostTargetEffect']
            elif card=='goblin-surprise' and s['mode']==0:effects=['BoostControlledEffect']
        for effect in effects:
            entries.append(dict(kind=effect,source=source,target=target,turn=event['turn'],step=event['step'],ready=i))
    def check_at(p,selected,path):
        expected=Counter((e['kind'],e['source']) for e in selected if e['kind']!='GainAbilityTargetEffect' or live(e['target'],p))
        actual=Counter((effect['class'].removeprefix('mage.abilities.effects.common.continuous.'),source) for effect in p['effects'] for source in effect['sources'])
        require(actual==expected,path+'/effect presence/lifecycle')
    for i,p in enumerate(points):
        check_at(p,[e for e in entries if e['turn']==p['turn'] and e['ready']<=i],f'/checkpoint/{i}')
    for p in run['marked']:
        check_at(p,[e for e in entries if e['turn']==p['turn'] and rank[e['step']]<rank['combat_damage']],'/simultaneous damage')
    for p in run['cleanup_checkpoints']:check_at(p,[],'/cleanup')
    # The actual LOST callback and completed-game artifact remain distinct.
    # ContinuousEffectsList deliberately preserves the last effects at game over.
    require(run['points'][-1]['effects']==run['before_departure']['effects'],'/terminal effect provenance')
