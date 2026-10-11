package org.mage.test.mtglab;
import com.google.gson.*;
import mage.abilities.*;
import mage.abilities.costs.mana.ManaCost;
import mage.constants.*;
import mage.game.*;
import mage.game.events.*;
import mage.game.mulligan.LondonMulligan;
import mage.game.combat.CombatGroup;
import mage.game.permanent.Permanent;
import mage.util.MultiAmountMessage;
import org.mage.test.player.TestPlayer;
import org.junit.Test;
import java.util.*;
import java.nio.file.*;
import java.nio.charset.StandardCharsets;
import static org.junit.Assert.*;
/** Version 9 normal-play combat; the inherited v5 client owns Cavalry activation. */
public class FullPoolCombatTest extends FullPoolActivationsTest {
    private JsonArray allocations,marked,legalChecks,damageCallbacks;
    private boolean observerInstalled;
    @Override protected void prepareCase(JsonObject c){super.prepareCase(c);allocations=new JsonArray();marked=new JsonArray();legalChecks=new JsonArray();damageCallbacks=new JsonArray();observerInstalled=false;}
    @Override protected void validate(JsonObject d){
        need(integer(d.get("schema_version"),9)&&d.get("family").getAsString().equals("combat"),"/version/family");
        JsonObject p=d.deepCopy();p.addProperty("schema_version",5);p.addProperty("family","activations");
        for(JsonElement raw:p.getAsJsonArray("cases")){
            JsonObject c=raw.getAsJsonObject();need(c.get("stop").getAsString().equals("combat/"+c.get("id").getAsString()),"/stop");c.addProperty("stop","activations/"+c.get("id").getAsString());
            for(JsonElement v:c.getAsJsonArray("play")){JsonObject e=v.getAsJsonObject();boolean selected=Arrays.asList("attackers","blockers","damage","graveyard_order").contains(e.get("kind").getAsString());need(e.has("selection")==selected,"/selection field");need(e.has("event")==e.get("kind").getAsString().equals("graveyard_order"),"/graveyard event field");if(e.has("event"))e.remove("event");if(selected){need(e.get("selection").isJsonArray(),"/selection array");e.remove("selection");}}
        }
        super.validate(p);
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
    @Override protected JsonObject state(Game g,int actor,String boundary,ManaCost unpaid){
        JsonObject p=super.state(g,actor,boundary,unpaid);JsonArray combat=new JsonArray();
        for(CombatGroup group:g.getCombat().getGroups())for(UUID id:group.getAttackers()){
            JsonObject a=new JsonObject();a.add("attacker",reference(id,g));a.addProperty("blocked",group.getBlocked());JsonArray blockers=new JsonArray();for(UUID b:group.getBlockers())blockers.add(reference(b,g));a.add("blockers",blockers);combat.add(a);
        }
        for(Permanent o:g.getBattlefield().getAllActivePermanents()){
            JsonObject v=p.getAsJsonObject("permanents").getAsJsonObject(bindings.get(o.getId()));
            v.addProperty("flying",o.getAbilities().containsKey(mage.abilities.keyword.FlyingAbility.getInstance().getId()));
            v.addProperty("reach",o.getAbilities().containsKey(mage.abilities.keyword.ReachAbility.getInstance().getId()));
            v.addProperty("vigilance",o.getAbilities().containsKey(mage.abilities.keyword.VigilanceAbility.getInstance().getId()));
            v.addProperty("deathtouch",o.getAbilities().containsKey(mage.abilities.keyword.DeathtouchAbility.getInstance().getId()));
        }
        p.add("combat",combat);p.add("allocations",allocations.deepCopy());return p;
    }
    private class Observer extends mage.watchers.Watcher {
        Observer(){super(WatcherScope.GAME);} Observer(Observer o){super(o);}
        @Override public mage.watchers.Watcher copy(){return new Observer(this);}
        @Override public void watch(mage.game.events.GameEvent e,Game g){
            if(e.getType()==mage.game.events.GameEvent.EventType.DAMAGED_BATCH_COULD_HAVE_FIRED && g.getTurnStepType()==PhaseStep.COMBAT_DAMAGE)marked.add(state(g,-1,"simultaneous-before-sba",null));
        }
    }
    @Override protected boolean done(Game g){return false;}
    @Override protected boolean onPriority(TestPlayer p,int s,Game g){
        if(!observerInstalled){g.getState().addWatcher(new Observer());observerInstalled=true;}
        if(cursor<current.getAsJsonArray("play").size()&&current.getAsJsonArray("play").get(cursor).getAsJsonObject().get("kind").getAsString().equals("finish")){
            try{
                JsonObject e=next(s,g);need(g.getTurnStepType()==PhaseStep.COMBAT_DAMAGE&&pending==null&&g.getStack().isEmpty(),"/finish before settled damage");
                played.add(state(g,s,"before/"+cursor,null));accept(e);need(cursor==current.getAsJsonArray("play").size(),"/extra choice after named stop");stopped=true;g.pause();return false;
            }catch(IllegalArgumentException error){failCallback(g,error);return false;}
        }
        if(cursor<current.getAsJsonArray("play").size()&&current.getAsJsonArray("play").get(cursor).getAsJsonObject().get("kind").getAsString().equals("legacy_blocker_order"))legacyBlockerOrder();
        return super.onPriority(p,s,g);
    }
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
    @Override protected JsonObject result(){JsonObject r=super.result();r.add("marked",marked);r.add("legal_checks",legalChecks);r.add("damage_callbacks",damageCallbacks);return r;}
    @Override @Test public void mulliganPrefixes() throws Exception{
        JsonObject d=read(Paths.get(System.getProperty("mtglab.fixture")));validate(d);JsonObject points=new JsonObject(),runs=new JsonObject(),repeat=new JsonObject();
        for(JsonElement v:d.getAsJsonArray("cases")){JsonObject c=v.getAsJsonObject();String id=c.get("id").getAsString();JsonObject r=run(c);runs.add(id,r);points.add(id,r.get("points"));repeat.add(id,run(c));}
        JsonObject negatives=read(root.resolve("fixtures/reference/full-pool-combat-negatives.json")),rejections=new JsonObject(),negativeRuns=new JsonObject();
        for(String name:negatives.keySet()){
            JsonObject spec=negatives.getAsJsonObject(name);
            try{run(spec.getAsJsonObject("input").getAsJsonArray("cases").get(0).getAsJsonObject());fail("accepted negative "+name);}
            catch(IllegalArgumentException error){need(error.getMessage().endsWith(spec.get("xmage").getAsString()),"/intended rejection "+name+": "+error.getMessage());need(usedPlay.size()==spec.get("sequence").getAsInt(),"/rejection cursor "+name);rejections.addProperty(name,error.getMessage());negativeRuns.add(name,result());}
        }
        JsonObject callbacks=new JsonObject();
        try{legacyBlockerOrder();fail("legacy order accepted");}catch(IllegalArgumentException error){callbacks.addProperty("legacy_order",error.getMessage());}
        run(d.getAsJsonArray("cases").get(0).getAsJsonObject());
        onBlockers(players[0],0,null,lastGame,players[0].getId());need(priorityFailure!=null,"/unscripted blockers accepted");callbacks.addProperty("unscripted_blockers",priorityFailure.getMessage());
        run(d.getAsJsonArray("cases").get(0).getAsJsonObject());
        try{onCombatAmounts(players[0],0,Outcome.Damage,new ArrayList<>(),0,0,null,lastGame);fail("unscripted damage accepted");}catch(IllegalArgumentException error){callbacks.addProperty("unscripted_damage",error.getMessage());}
        JsonObject output=new JsonObject();output.add("callback_controls",callbacks);output.add("rejections",rejections);output.add("negative_runs",negativeRuns);output.add("checkpoints",points);output.add("runs",runs);output.add("repeat_runs",repeat);
        Files.write(Paths.get(System.getProperty("mtglab.output")),JSON.toJson(output).getBytes(StandardCharsets.UTF_8));
    }
}
