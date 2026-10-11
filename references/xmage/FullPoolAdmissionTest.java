package org.mage.test.mtglab;
import com.google.gson.*;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.util.*;
import mage.abilities.*;
import mage.abilities.Ability;
import mage.abilities.costs.mana.ManaCost;
import mage.abilities.mana.ActivatedManaAbilityImpl;
import mage.cards.Card;
import mage.constants.*;
import mage.game.*;
import mage.game.Game;
import mage.game.combat.CombatGroup;
import mage.game.events.*;
import mage.game.events.GameEvent;
import mage.game.events.ZoneChangeEvent;
import mage.game.mulligan.LondonMulligan;
import mage.game.permanent.Permanent;
import mage.game.stack.Spell;
import mage.game.stack.StackObject;
import mage.players.Player;
import mage.target.Target;
import mage.util.MultiAmountMessage;
import mage.watchers.Watcher;
import org.junit.Test;
import org.mage.test.player.TestPlayer;
import static org.junit.Assert.*;
/** Version 10 full games. Composes the delivered strict v5/v6/v7/v9
 * callbacks; no policy runs here. Combat uses the retained reviewed patch. */
public class FullPoolAdmissionTest extends FullPoolTriggersTest {
    private JsonArray allocations,marked,legalChecks,damageCallbacks,activationChecks,manaChecks,cleanupPoints;
    private boolean observerInstalled,pendingActivation,cancelRequested;
    private JsonObject beforeDeparture;
    @Override protected void prepareCase(JsonObject c){
        super.prepareCase(c);allocations=new JsonArray();marked=new JsonArray();legalChecks=new JsonArray();damageCallbacks=new JsonArray();
        activationChecks=new JsonArray();manaChecks=new JsonArray();cleanupPoints=new JsonArray();beforeDeparture=null;
        observerInstalled=false;pendingActivation=false;cancelRequested=false;
    }
    @Override protected boolean triggerPrefixDone(Game g){return false;}
    @Override protected void validate(JsonObject doc){
        need(integer(doc.get("schema_version"),10)&&doc.get("family").getAsString().equals("admission"),"/version/family");
        JsonObject opening=new JsonObject();opening.addProperty("schema_version",2);opening.addProperty("family","mulligan");opening.add("pins",doc.get("pins"));
        JsonArray cases=new JsonArray();
        for(JsonElement raw:doc.getAsJsonArray("cases")){
            JsonObject source=raw.getAsJsonObject(),c=new JsonObject();
            for(String key:Arrays.asList("id","starter","decks","chance","choices"))c.add(key,source.get(key));
            need(source.get("stop").getAsString().equals("terminal"),"/nonterminal admission");
            need(source.getAsJsonArray("play").size()>0,"/empty transcript");
            c.addProperty("stop","first_upkeep");cases.add(c);
        }
        opening.add("cases",cases);new FullPoolMulliganTest().validate(opening);
    }
    @Override protected JsonObject next(int actor,Game g){
        need(cursor<current.getAsJsonArray("play").size(),"/omitted choice");
        JsonObject e=current.getAsJsonArray("play").get(cursor).getAsJsonObject();
        need(integer(e.get("sequence"),cursor)&&integer(e.get("actor"),actor),"/play actor/sequence");
        need(integer(e.get("turn"),g.getTurnNum())&&e.get("step").getAsString().equals(g.getTurnStepType().name().toLowerCase(Locale.ROOT)),"/play position");
        String k=e.get("kind").getAsString();
        Set<String> fields=new HashSet<>(Arrays.asList("sequence","turn","step","actor","kind","source","incarnation","color","role","mode","order","player"));
        if(Arrays.asList("attackers","blockers","damage","graveyard_order","cleanup_discard").contains(k))fields.add("selection");
        if(k.equals("graveyard_order"))fields.add("event");need(e.keySet().equals(fields),"/choice fields");
        boolean object=Arrays.asList("cast","play_land","tap_mana","target","discard","activate","activation_target").contains(k);
        need(object==!e.get("source").isJsonNull()&&object==!e.get("incarnation").isJsonNull(),"/source/incarnation field");
        need((k.equals("pay")||k.equals("activation_pay"))==!e.get("color").isJsonNull(),"/payment color field");
        need(k.equals("mode")==!e.get("mode").isJsonNull(),"/mode field");
        need(k.equals("target")==!e.get("role").isJsonNull(),"/target role field");
        need(k.equals("order_triggers")==!e.get("order").isJsonNull(),"/trigger order field");
        need(k.equals("target_player")==!e.get("player").isJsonNull(),"/player target field");return e;
    }

    @Override protected Game createGame(){
        return new CombatSbaGame(new CombatSbaGame.OwnerChoice(){
            public List<Permanent> choose(UUID owner,List<Permanent> arrivals,Game g){return graveyardOrder(seat(owner),arrivals,g);}
            public void failure(Game g,IllegalArgumentException error){failCallback(g,error);}
        });
    }
    private List<Permanent> graveyardOrder(int owner,List<Permanent> arrivals,Game g){
        JsonObject e=next(owner,g);
        need(e.get("kind").getAsString().equals("graveyard_order"),"/missing graveyard choice");
        need(e.has("event")&&e.get("event").getAsString().equals("combat-sba/"+g.getTurnNum()),"/graveyard event");
        played.add(state(g,owner,"before/"+cursor,null));
        List<Permanent> ordered=new ArrayList<>();Set<UUID> used=new HashSet<>();
        for(JsonElement v:e.getAsJsonArray("selection")){
            UUID id=combatSource(v.getAsJsonObject(),g);
            Permanent p=arrivals.stream().filter(x->x.getId().equals(id)).findFirst().orElse(null);
            need(p!=null&&used.add(id),"/graveyard permutation");ordered.add(p);
        }
        need(ordered.size()==arrivals.size(),"/graveyard permutation");accept(e);return ordered;
    }
    private JsonObject reference(UUID id,Game g){need(bindings.containsKey(id),"/unbound combat object");JsonObject r=new JsonObject();r.addProperty("source",bindings.get(id));r.addProperty("incarnation",incarnation(id,g));return r;}
    private UUID combatSource(JsonObject v,Game g){keys(v,"source","incarnation");return source(v,g);}
    @Override protected void emptyAttackers(TestPlayer p,int s,Game g,boolean forced){
        JsonObject e=next(s,g);if(e.get("kind").getAsString().equals("legacy_blocker_order"))legacyBlockerOrder();if(e.get("kind").getAsString().equals("empty_attackers")){super.emptyAttackers(p,s,g,forced);return;}
        need(g.getTurnStepType()==PhaseStep.DECLARE_ATTACKERS&&s==seat(g.getActivePlayerId())&&declarations.add(g.getTurnNum()),"/extra or misplaced declaration");
        need(e.get("kind").getAsString().equals("attackers"),"/missing attackers declaration");
        played.add(state(g,s,"before/"+cursor,null));JsonArray legal=new JsonArray();
        for(Permanent o:p.getAvailableAttackers(g))if(o.canAttack(players[1-s].getId(),g))legal.add(reference(o.getId(),g));
        JsonObject check=new JsonObject();check.addProperty("kind","attackers");check.addProperty("sequence",cursor);check.add("legal",legal);legalChecks.add(check);
        Set<UUID> used=new HashSet<>();
        for(JsonElement v:e.getAsJsonArray("selection")){
            UUID id=combatSource(v.getAsJsonObject(),g);Permanent o=g.getPermanent(id);
            need(used.add(id)&&o!=null&&p.getAvailableAttackers(g).stream().anyMatch(x->x.getId().equals(id))&&o.getControllerId().equals(p.getId())&&o.canAttack(players[1-s].getId(),g),"/selected attacker rejected");
            p.declareAttacker(id,players[1-s].getId(),g,false);
            need(g.getCombat().getGroups().stream().anyMatch(a->a.getAttackers().contains(id)),"/selected attacker not committed");
        }
        accept(e);
    }
    @Override protected void onBlockers(TestPlayer p,int s,Ability source,Game g,UUID defending){
        try{
            need(s!=seat(g.getActivePlayerId())&&defending.equals(p.getId()),"/blockers actor");JsonObject e=next(s,g);need(e.get("kind").getAsString().equals("blockers"),"/missing blockers declaration");
            played.add(state(g,s,"before/"+cursor,null));JsonArray legal=new JsonArray();
            for(Permanent o:g.getBattlefield().getAllActivePermanents())if(o.getControllerId().equals(p.getId()))for(CombatGroup group:g.getCombat().getGroups())for(UUID a:group.getAttackers())if(o.canBlock(a,g)){
                JsonObject pair=new JsonObject();pair.add("blocker",reference(o.getId(),g));pair.add("attacker",reference(a,g));legal.add(pair);
            }
            JsonObject check=new JsonObject();check.addProperty("kind","blockers");check.addProperty("sequence",cursor);check.add("legal",legal);legalChecks.add(check);
            Set<UUID> used=new HashSet<>();
            for(JsonElement v:e.getAsJsonArray("selection")){
                JsonObject pair=v.getAsJsonObject();keys(pair,"blocker","attacker");UUID b=combatSource(pair.getAsJsonObject("blocker"),g),a=combatSource(pair.getAsJsonObject("attacker"),g);Permanent o=g.getPermanent(b);
                need(used.add(b)&&o!=null&&o.getControllerId().equals(p.getId())&&!o.isTapped()&&o.canBlock(a,g),"/selected blocker rejected");
                p.declareBlocker(p.getId(),b,a,g);
                need(g.getCombat().getGroups().stream().anyMatch(group->group.getAttackers().contains(a)&&group.getBlockers().contains(b)),"/selected blocker not committed");
            }
            accept(e);
        }catch(IllegalArgumentException error){failCallback(g,error);}
    }
    @Override protected List<Integer> onCombatAmounts(TestPlayer p,int s,Outcome outcome,List<MultiAmountMessage> messages,int min,int max,MultiAmountType type,Game g){
        JsonObject e=next(s,g);need(e.get("kind").getAsString().equals("damage"),"/missing damage allocation");played.add(state(g,s,"before/"+cursor,null));
        JsonArray selected=e.getAsJsonArray("selection");need(selected.size()==1,"/damage cardinality");JsonObject choice=selected.get(0).getAsJsonObject();keys(choice,"attacker","amounts");UUID a=combatSource(choice.getAsJsonObject("attacker"),g);
        CombatGroup group=g.getCombat().getGroups().stream().filter(x->x.getAttackers().contains(a)).findFirst().orElseThrow(()->new IllegalArgumentException("first divergence: /damage source"));
        need(s==seat(g.getActivePlayerId())&&group.getAttackers().size()==1&&messages.size()==group.getBlockers().size(),"/damage callback identity");
        JsonArray amounts=choice.getAsJsonArray("amounts");need(amounts.size()==messages.size(),"/damage recipients");List<Integer> result=new ArrayList<>();JsonArray constraints=new JsonArray();int total=0;
        for(int i=0;i<messages.size();i++){
            JsonObject amount=amounts.get(i).getAsJsonObject();keys(amount,"target","amount");UUID b=combatSource(amount.getAsJsonObject("target"),g);need(b.equals(group.getBlockers().get(i)),"/reordered damage recipient");
            int value=amount.get("amount").getAsInt();MultiAmountMessage m=messages.get(i);need(integer(amount.get("amount"),value)&&value>=m.min&&value<=m.max,"/damage constraint");total+=value;result.add(value);
            JsonObject constraint=new JsonObject();constraint.add("target",reference(b,g));constraint.addProperty("min",m.min);constraint.addProperty("max",m.max);constraint.addProperty("default",m.defaultValue);constraint.addProperty("message",m.message);constraints.add(constraint);
        }
        JsonObject witness=new JsonObject();witness.add("attacker",reference(a,g));witness.addProperty("sequence",cursor);witness.addProperty("min",min);witness.addProperty("max",max);witness.add("constraints",constraints);damageCallbacks.add(witness);
        need(total>=min&&total<=max,"/damage total");
        // Upstream only exposes a total lethal minimum. CR702.19b additionally
        // requires lethal on EACH blocker before damage tramples to the player.
        if(total<max)for(int i=0;i<result.size();i++)need(result.get(i)>=g.getPermanent(group.getBlockers().get(i)).getLethalDamage(a,g),"/nonlethal before trample");
        allocations.add(choice.deepCopy());accept(e);return result;
    }
    protected void legacyBlockerOrder(){throw new IllegalArgumentException("first divergence: /legacy blocker-order callback");}
    private String abilityKind(UUID id,Game g){
        String card=g.getCard(id).getName();
        switch(card){case "Shivan Dragon":return "power";case "Wildheart Invoker":return "invoker";case "Axgard Cavalry":return "haste";default:throw new IllegalArgumentException("first divergence: /unsupported activation source");}
    }
    @Override protected void tap(TestPlayer p,JsonObject e,Game g){
        UUID id=source(e,g);Permanent o=g.getPermanent(id);need(o!=null,"/departed mana source");
        List<ActivatedManaAbilityImpl> abilities=o.getAbilities().getActivatedManaAbilities(Zone.BATTLEFIELD);
        need(abilities.size()==1,"/ambiguous mana ability");
        JsonObject witness=new JsonObject();witness.addProperty("source",bindings.get(id));witness.addProperty("action",cursor);witness.addProperty("ability_id",abilities.get(0).getId().toString());
        int stack=g.getStack().size();UUID priority=g.getPriorityPlayerId();
        need(p.activateAbility(abilities.get(0),g),"/selected mana ability rejected");
        need(g.getPermanent(id).isTapped()&&g.getStack().size()==stack&&Objects.equals(priority,g.getPriorityPlayerId()),"/mana stack/priority/tap");
        manaChecks.add(witness);if(pending!=null)pendingSources.add(id);accept(e);
    }
    @Override protected boolean onMana(TestPlayer p,int s,Ability a,ManaCost unpaid,Game g){
        if(!pendingActivation)return super.onMana(p,s,a,unpaid,g);
        try{
            need(!stopped&&pending!=null&&a!=null&&pending.equals(a.getSourceId())&&s==pendingActor,"/unexpected activation mana callback");
            need(!p.getManaPool().isAutoPayment(),"/automatic mana selection");pendingAbility=a;
            JsonObject e=next(s,g);played.add(state(g,s,"before/"+cursor,unpaid));String k=e.get("kind").getAsString();
            if(k.equals("cancel_activation")){accept(e);cancelRequested=true;return false;}
            if(k.equals("tap_mana")){tap(p,e,g);return true;}
            need(k.equals("activation_pay"),"/missing activation payment");int color=e.get("color").getAsInt();
            need(integer(e.get("color"),color)&&color>=0&&color<6,"/payment color");
            need(mana(p.getManaPool().getMana()).get(color).getAsInt()>0,"/insufficient or wrong-color payment");
            JsonArray colored=mana(unpaid.getMana());for(int i=0;i<6;i++)if(colored.get(i).getAsInt()>0)need(i==color,"/wrong-color payment");
            p.getManaPool().unlockManaType(new ManaType[]{ManaType.WHITE,ManaType.BLUE,ManaType.BLACK,ManaType.RED,ManaType.GREEN,ManaType.COLORLESS}[color]);accept(e);return true;
        }catch(IllegalArgumentException error){failCallback(g,error);return false;}
    }
    private class AdmissionObserver extends mage.watchers.Watcher {
        AdmissionObserver(){super(WatcherScope.GAME);} AdmissionObserver(AdmissionObserver o){super(o);}
        @Override public mage.watchers.Watcher copy(){return new AdmissionObserver(this);}
        @Override public void watch(GameEvent e,Game g){
            if(e.getType()==GameEvent.EventType.LOST)beforeDeparture=state(g,-1,"loss-before-departure",null);
            if(e.getType()==GameEvent.EventType.CLEANUP_STEP_POST)cleanupPoints.add(state(g,-1,"cleanup-settled",null));
            if(e.getType()==GameEvent.EventType.DAMAGED_BATCH_COULD_HAVE_FIRED&&g.getTurnStepType()==PhaseStep.COMBAT_DAMAGE)marked.add(state(g,-1,"simultaneous-before-sba",null));
        }
    }
    @Override protected JsonObject nonTriggerAbility(StackObject o,Game g){
        UUID source=o.getSourceId();need(bindings.containsKey(source),"/unbound ability source");
        Integer action=stackActions.get(o.getId());
        if(action==null){need(pendingActivation&&pending!=null&&pending.equals(source),"/unwitnessed ability announcement");action=pendingAction;stackActions.put(o.getId(),action);}
        JsonObject v=new JsonObject();v.addProperty("source",bindings.get(source));v.addProperty("incarnation",incarnation(source,g));v.addProperty("action",action);
        v.addProperty("object","ability/"+action);v.addProperty("ability",abilityKind(source,g));
        JsonElement target=JsonNull.INSTANCE;
        for(Target t:((Ability)o).getTargets())for(UUID id:t.getTargets()){need(target.isJsonNull(),"/extra activation target");target=reference(id,g);}
        v.add("target",target);v.addProperty("raw_id",o.getId().toString());return v;
    }
    private JsonElement outcome(Game g){
        if(!g.hasEnded())return JsonNull.INSTANCE;
        JsonObject out=new JsonObject();out.add("winner",JsonNull.INSTANCE);JsonArray losses=new JsonArray();
        for(int s=0;s<2;s++){
            Player p=g.getPlayer(players[s].getId());if(p.hasWon())out.addProperty("winner",s);
            if(!p.hasLost())losses.add(JsonNull.INSTANCE);
            else if(p.getLife()<=0&&!p.getLibrary().isEmptyDraw())losses.add("Life");
            else if(p.getLife()>0&&p.getLibrary().isEmptyDraw())losses.add("EmptyDraw");
            else throw new IllegalArgumentException("first divergence: /unsupported or ambiguous terminal reason");
        }
        out.add("losses",losses);return out;
    }
    @Override protected JsonObject state(Game g,int actor,String boundary,ManaCost unpaid){
        JsonObject p=super.state(g,actor,boundary,unpaid);p.add("outcome",outcome(g));if(actor<0||g.hasEnded()){p.add("actor",JsonNull.INSTANCE);p.add("trigger_boundary",JsonNull.INSTANCE);}
        JsonArray combat=new JsonArray();
        for(CombatGroup group:g.getCombat().getGroups())for(UUID id:group.getAttackers()){
            JsonObject a=new JsonObject();a.add("attacker",reference(id,g));a.addProperty("blocked",group.getBlocked());JsonArray blockers=new JsonArray();for(UUID b:group.getBlockers())blockers.add(reference(b,g));a.add("blockers",blockers);combat.add(a);
        }
        p.add("combat",combat);p.add("allocations",allocations.deepCopy());p.add("activation",JsonNull.INSTANCE);
        if(pendingActivation){p.add("activation",p.get("payment"));p.add("payment",JsonNull.INSTANCE);}
        for(Permanent o:g.getBattlefield().getAllActivePermanents()){
            JsonObject v=p.getAsJsonObject("permanents").getAsJsonObject(bindings.get(o.getId()));
            v.addProperty("flying",o.getAbilities().containsKey(mage.abilities.keyword.FlyingAbility.getInstance().getId()));
            v.addProperty("reach",o.getAbilities().containsKey(mage.abilities.keyword.ReachAbility.getInstance().getId()));
            v.addProperty("vigilance",o.getAbilities().containsKey(mage.abilities.keyword.VigilanceAbility.getInstance().getId()));
            v.addProperty("deathtouch",o.getAbilities().containsKey(mage.abilities.keyword.DeathtouchAbility.getInstance().getId()));
            v.addProperty("haste",o.getAbilities().containsKey(mage.abilities.keyword.HasteAbility.getInstance().getId()));
            v.addProperty("trample",o.getAbilities().containsKey(mage.abilities.keyword.TrampleAbility.getInstance().getId()));
        }
        return p;
    }

    @Override protected boolean onPriority(TestPlayer p,int s,Game g){
        if(!observerInstalled){g.getState().addWatcher(new AdmissionObserver());observerInstalled=true;}
        if(cursor>=current.getAsJsonArray("play").size()||!current.getAsJsonArray("play").get(cursor).getAsJsonObject().get("kind").getAsString().equals("activate"))return super.onPriority(p,s,g);
        try{
            beginPriority();
            JsonObject e=next(s,g);played.add(state(g,s,"before/"+cursor,null));
        {
                    UUID id=source(e,g);Permanent o=g.getPermanent(id);need(o!=null,"/departed activation source");
                    List<ActivatedAbility> candidates=new ArrayList<>();
                    for(ActivatedAbility a:p.getPlayableActivatedAbilities(o,Zone.BATTLEFIELD,g).values())if(!(a instanceof ActivatedManaAbilityImpl)&&a.getSourceId().equals(id))candidates.add(a);
                    need(candidates.size()==1,"/selected activation not playable");
                    JsonObject witness=new JsonObject();witness.addProperty("source",bindings.get(id));witness.addProperty("action",cursor);witness.addProperty("candidate_id",candidates.get(0).getId().toString());witness.addProperty("candidate_count",candidates.size());witness.addProperty("ability",abilityKind(id,g));activationChecks.add(witness);
                    pending=id;pendingAction=cursor;pendingOrigin=incarnation(id,g);pendingActor=s;pendingSources.clear();pendingAbility=null;pendingActivation=true;
                    p.getManaPool().setAutoPayment(false);accept(e);
                    boolean activated=p.activateAbility(candidates.get(0),g);
                    if(cancelRequested){need(!activated&&priorityFailure==null,"/cancel failed");cancelRequested=false;pending=null;pendingAbility=null;pendingActivation=false;pendingSources.clear();return true;}
                    need(activated,"/selected activation rejected");need(priorityFailure==null,"/failed activation callback");
                    JsonObject finish=next(s,g);need(finish.get("kind").getAsString().equals("finish_activation"),"/missing activation commit acknowledgement");
                    need(pendingAbility!=null,"/missing activation callback observation");
                    played.add(state(g,s,"before/"+cursor,pendingAbility.getManaCostsToPay().getUnpaid()));accept(finish);
                    pending=null;pendingAbility=null;pendingActivation=false;pendingSources.clear();return true;
                }
        }catch(IllegalArgumentException error){failCallback(g,error);return false;}
    }

    private boolean activationTarget(TestPlayer p,int s,Outcome outcome,Target target,Ability a,Game g){
        try{
            need(pendingActivation&&pending!=null&&a!=null&&pending.equals(a.getSourceId())&&s==pendingActor&&target!=null,"/unexpected activation target callback");
            pendingAbility=a;JsonObject e=next(s,g);if(e.get("kind").getAsString().equals("cancel_activation")){played.add(state(g,s,"before/"+cursor,null));accept(e);cancelRequested=true;return false;}need(e.get("kind").getAsString().equals("activation_target"),"/missing activation target");
            played.add(state(g,s,"before/"+cursor,null));UUID id=source(e,g);
            need(target.canTarget(p.getId(),id,a,g),"/selected target not legal");target.addTarget(id,a,g);need(target.getTargets().contains(id),"/selected target rejected");accept(e);return true;
        }catch(IllegalArgumentException error){failCallback(g,error);return false;}
    }
    @Override protected boolean onSpellTarget(TestPlayer p,int s,Outcome o,Target target,Ability a,Game g) {
        if(pendingActivation)return activationTarget(p,s,o,target,a,g);
        if(g.getTurnStepType()!=PhaseStep.CLEANUP || a!=null)return super.onSpellTarget(p,s,o,target,a,g);
        try {
            need(!stopped&&o==Outcome.Discard&&target!=null,"/unexpected cleanup callback");
            JsonObject e=next(s,g);need(e.get("kind").getAsString().equals("cleanup_discard"),"/missing cleanup discard");
            JsonArray selected=e.getAsJsonArray("selection");
            need(selected.size()==target.getMinNumberOfTargets()&&selected.size()==target.getMaxNumberOfTargets(),"/cleanup cardinality");
            played.add(state(g,s,"before/"+cursor,null));
            for(JsonElement raw:selected) {
                JsonObject card=raw.getAsJsonObject();keys(card,"source","incarnation");UUID id=source(card,g);
                need(p.getHand().contains(id)&&!target.getTargets().contains(id)&&target.canTarget(p.getId(),id,a,g),"/cleanup discard source");
                target.addTarget(id,a,g);need(target.getTargets().contains(id),"/cleanup discard rejected");
            }
            need(target.getTargets().size()==selected.size(),"/cleanup discard cardinality");accept(e);return true;
        }catch(IllegalArgumentException e){failCallback(g,e);return false;}
    }
    @Override protected void afterGame(Game g){
        need(g.hasEnded(),"/truncated game");
        need(cursor<current.getAsJsonArray("play").size(),"/omitted final checkpoint");
        JsonObject e=current.getAsJsonArray("play").get(cursor).getAsJsonObject();
        need(e.get("kind").getAsString().equals("finish")&&integer(e.get("sequence"),cursor),"/choice after terminal");
        need(cursor+1==current.getAsJsonArray("play").size(),"/unused tape suffix");
        need(integer(e.get("turn"),g.getTurnNum())&&e.get("step").getAsString().equals(g.getTurnStepType().name().toLowerCase(Locale.ROOT)),"/final checkpoint position");
        need(beforeDeparture!=null,"/missing terminal-before-departure observation");
        played.add(state(g,-1,"finish",null));accept(e);stopped=true;
    }
    @Override protected JsonObject result(){
        JsonObject r=super.result();r.add("before_departure",beforeDeparture);r.add("cleanup_checkpoints",cleanupPoints);
        r.add("marked",marked);r.add("legal_checks",legalChecks);r.add("damage_callbacks",damageCallbacks);
        r.add("activation_checks",activationChecks);r.add("mana_checks",manaChecks);return r;
    }
    @Override @Test public void mulliganPrefixes() throws Exception{
        JsonObject doc=read(Paths.get(System.getProperty("mtglab.fixture")));
        validate(doc);
        JsonObject index=new JsonObject();Path output=Paths.get(System.getProperty("mtglab.output"));
        for(JsonElement v:doc.getAsJsonArray("cases")){
            JsonObject c=v.getAsJsonObject();String id=c.get("id").getAsString();need(id.matches("[A-Za-z0-9-]+"),"/case id");
            JsonArray paths=new JsonArray();
            for(int repeat=1;repeat<=2;repeat++){
                JsonObject result=run(c);String name=id+".xmage-"+repeat+".json";
                try(java.io.Writer writer=Files.newBufferedWriter(output.resolveSibling(name),StandardCharsets.UTF_8)){JSON.toJson(result,writer);}
                paths.add(name);
            }
            index.add(id,paths);
        }
        JsonObject specs=read(Paths.get(System.getProperty("mtglab.root")).resolve("fixtures/reference/full-pool-admission-xmage-negatives.json"));
        JsonObject negatives=new JsonObject();
        for(String name:specs.keySet()){
            JsonObject spec=specs.getAsJsonObject(name);JsonObject c=spec.getAsJsonObject("case");
            try{
                JsonObject input=doc.deepCopy();JsonArray cases=new JsonArray();cases.add(c);input.add("cases",cases);if(spec.has("pins"))input.add("pins",spec.get("pins"));validate(input);
                run(c);fail("accepted invalid admission tape "+name);
            }catch(IllegalArgumentException error){
                need(error.getMessage().contains("first divergence:"),"/unclassified rejection "+name);
                JsonObject rejected=new JsonObject();rejected.addProperty("error",error.getMessage());
                if(!spec.get("sequence").isJsonNull()){
                    need(usedPlay.size()==spec.get("sequence").getAsInt(),"/rejection cursor "+name+": "+error.getMessage());rejected.add("run",result());
                }else rejected.add("run",JsonNull.INSTANCE);
                negatives.add(name,rejected);
            }
        }
        try(java.io.Writer writer=Files.newBufferedWriter(output.resolveSibling("xmage-negatives.json"),StandardCharsets.UTF_8)){JSON.toJson(negatives,writer);}
        Files.write(output,JSON.toJson(index).getBytes(StandardCharsets.UTF_8));
    }
}
