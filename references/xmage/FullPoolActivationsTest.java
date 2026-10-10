package org.mage.test.mtglab;
import com.google.gson.*;
import mage.abilities.*;
import mage.abilities.mana.ActivatedManaAbilityImpl;
import mage.abilities.costs.mana.ManaCost;
import mage.constants.*;
import mage.cards.Card;
import mage.game.Game;
import mage.game.permanent.Permanent;
import mage.game.stack.Spell;
import mage.game.stack.StackObject;
import mage.target.Target;
import mage.players.Player;
import org.mage.test.player.TestPlayer;
import org.junit.Test;
import java.util.*;
import java.nio.file.*;
import java.nio.charset.StandardCharsets;
import static org.junit.Assert.*;
/** Version 5 strict activation callback extension of the same normal-reset runner. */
public class FullPoolActivationsTest extends FullPoolPriorityTest {
    private boolean pendingActivation,cancelRequested;
    private JsonArray activationChecks,manaChecks;
    @Override protected void prepareCase(JsonObject c){super.prepareCase(c);pendingActivation=false;cancelRequested=false;activationChecks=new JsonArray();manaChecks=new JsonArray();}
    @Override protected void validate(JsonObject doc){
        need(integer(doc.get("schema_version"),5)&&doc.get("family").getAsString().equals("activations"),"/version/family");
        JsonObject projection=doc.deepCopy();projection.addProperty("schema_version",3);projection.addProperty("family","priority");
        for(JsonElement e:projection.getAsJsonArray("cases")){JsonObject c=e.getAsJsonObject();need(c.get("stop").getAsString().equals("activations/"+c.get("id").getAsString()),"/stop");c.addProperty("stop","second_creature_resolved");}
        super.validate(projection);
    }
    @Override protected JsonObject next(int actor,Game g){
        need(cursor<current.getAsJsonArray("play").size(),"/missing choice before named stop");
        JsonObject e=current.getAsJsonArray("play").get(cursor).getAsJsonObject();
        need(integer(e.get("sequence"),cursor)&&integer(e.get("actor"),actor),"/play actor/sequence");
        need(integer(e.get("turn"),g.getTurnNum())&&e.get("step").getAsString().equals(g.getTurnStepType().name().toLowerCase(Locale.ROOT)),"/play position");
        String k=e.get("kind").getAsString();need(Arrays.asList("pay","activation_pay").contains(k)==!e.get("color").isJsonNull(),"/payment color field");
        boolean object=Arrays.asList("cast","play_land","tap_mana","activate","activation_target").contains(k);
        need(object==!e.get("source").isJsonNull()&&object==!e.get("incarnation").isJsonNull(),"/source/incarnation field");return e;
    }
    @Override protected UUID source(JsonObject e,Game g){
        UUID id=handles.get(e.get("source").getAsString());need(id!=null,"/source unknown");
        need(integer(e.get("incarnation"),incarnation(id,g)),"/source incarnation");return id;
    }
    private JsonObject reference(UUID id,Game g){JsonObject r=new JsonObject();need(bindings.containsKey(id),"/unbound object");r.addProperty("source",bindings.get(id));r.addProperty("incarnation",incarnation(id,g));return r;}
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
    @Override protected JsonObject state(Game g,int actor,String boundary,ManaCost unpaid) {
        observeIdentities(g,false);
        JsonObject p=new JsonObject();p.addProperty("boundary",boundary);p.addProperty("turn",g.getTurnNum());p.addProperty("step",g.getTurnStepType().name().toLowerCase(Locale.ROOT));
        p.addProperty("active",seat(g.getActivePlayerId()));p.addProperty("actor",actor);p.addProperty("land_plays",g.getPlayer(g.getActivePlayerId()).getLandsPlayed());
        JsonArray life=new JsonArray(),pool=new JsonArray(),hand=new JsonArray(),library=new JsonArray(),grave=new JsonArray();
        for(int s=0;s<2;s++){Player player=g.getPlayer(players[s].getId());life.add(player.getLife());pool.add(mana(player.getManaPool().getMana()));hand.add(ids(player.getHand(),false));library.add(ids(player.getLibrary().getCardList(),false));grave.add(ids(player.getGraveyard(),false));}
        p.add("life",life);p.add("mana",pool);p.add("hand",hand);p.add("library",library);p.add("graveyard",grave);
        List<UUID> exile=new ArrayList<>();for(Card c:g.getExile().getCardsOwned(g,null))exile.add(c.getId());p.add("exile",ids(exile,false));
        JsonArray battlefield=new JsonArray(),stack=new JsonArray();JsonObject permanents=new JsonObject(),incarnations=new JsonObject();
        for(UUID id:initialCounters.keySet())incarnations.addProperty(bindings.get(id),incarnation(id,g));
        for(Permanent o:g.getBattlefield().getAllActivePermanents()) {
            need(bindings.containsKey(o.getId()),"/unwitnessed permanent");String id=bindings.get(o.getId());battlefield.add(id);incarnations.addProperty(id,incarnation(o.getId(),g));
            JsonObject v=new JsonObject();v.addProperty("card",id.split("/")[1]);v.addProperty("owner",seat(o.getOwnerId()));v.addProperty("controller",seat(o.getControllerId()));v.addProperty("tapped",o.isTapped());v.addProperty("sick",o.isCreature(g)&&o.hasSummoningSickness());
            v.add("power",o.isCreature(g)?new JsonPrimitive(o.getPower().getValue()):JsonNull.INSTANCE);v.add("toughness",o.isCreature(g)?new JsonPrimitive(o.getToughness().getValue()):JsonNull.INSTANCE);v.add("damage",o.isCreature(g)?new JsonPrimitive(o.getDamage()):JsonNull.INSTANCE);
            v.addProperty("haste",o.getAbilities().containsKey(mage.abilities.keyword.HasteAbility.getInstance().getId()));v.addProperty("trample",o.getAbilities().containsKey(mage.abilities.keyword.TrampleAbility.getInstance().getId()));
            permanents.add(id,v);
        }
        List<StackObject> order=new ArrayList<>();for(StackObject o:g.getStack())order.add(o);Collections.reverse(order);
        for(StackObject o:order) {
            UUID source=o.getSourceId();need(bindings.containsKey(source),"/unbound stack source");
            boolean spell=o instanceof Spell;need(spell||o instanceof mage.game.stack.StackAbility,"/unsupported stack object");
            int action;
            if(spell){need(castActions.containsKey(source),"/unbound spell");action=castActions.get(source);}
            else {
                Integer known=stackActions.get(o.getId());
                if(known==null){need(pendingActivation&&pending!=null&&pending.equals(source),"/unwitnessed ability announcement");known=pendingAction;stackActions.put(o.getId(),known);}
                action=known;
            }
            JsonObject v=new JsonObject();v.addProperty("source",bindings.get(source));v.addProperty("incarnation",incarnation(source,g));v.addProperty("action",action);
            v.addProperty("object",spell?bindings.get(source):"ability/"+action);v.addProperty("ability",spell?"spell":abilityKind(source,g));
            JsonElement target=JsonNull.INSTANCE;
            if(!spell){Ability a=(Ability)o;for(Target t:a.getTargets())for(UUID id:t.getTargets()){need(target.isJsonNull(),"/extra activation target");target=reference(id,g);}v.addProperty("raw_id",o.getId().toString());}
            v.add("target",target);stack.add(v);rawStackIds.add(o.getId().toString(),v.deepCopy());
        }
        p.add("battlefield",battlefield);p.add("permanents",permanents);p.add("incarnations",incarnations);p.add("stack",stack);
        p.add("payment",JsonNull.INSTANCE);p.add("activation",JsonNull.INSTANCE);
        if(pending!=null) {
            JsonObject payment=new JsonObject();payment.addProperty("source",bindings.get(pending));payment.addProperty("actor",pendingActor);
            payment.add("pool",mana(g.getPlayer(players[pendingActor].getId()).getManaPool().getMana()));
            if(unpaid!=null){payment.add("colored",mana(unpaid.getMana()));payment.addProperty("generic",unpaid.getMana().getGeneric());}
            payment.add("sources",ids(pendingSources,false));p.add(pendingActivation?"activation":"payment",payment);
        }
        JsonArray effects=new JsonArray();
        for(mage.abilities.effects.ContinuousEffect effect:g.getContinuousEffects().getLayeredEffects(g)) {
            JsonObject observed=new JsonObject();observed.addProperty("raw_id",effect.getId().toString());observed.addProperty("class",effect.getClass().getName());observed.addProperty("duration",effect.getDuration().name());
            JsonArray sources=new JsonArray();for(Ability a:g.getContinuousEffects().getLayeredEffectAbilities(effect)){need(bindings.containsKey(a.getSourceId()),"/unbound effect source");sources.add(bindings.get(a.getSourceId()));}observed.add("sources",sources);effects.add(observed);
        }
        p.add("effects",effects);
        JsonArray haste=new JsonArray(),trample=new JsonArray();
        for(Permanent o:g.getBattlefield().getAllActivePermanents()){
            if(o.getAbilities().containsKey(mage.abilities.keyword.HasteAbility.getInstance().getId()))haste.add(reference(o.getId(),g));
            if(o.getAbilities().containsKey(mage.abilities.keyword.TrampleAbility.getInstance().getId()))trample.add(reference(o.getId(),g));
        }
        p.add("haste",haste);p.add("trample",trample);return p;
    }
    @Override protected boolean onSpellTarget(TestPlayer p,int s,Outcome outcome,Target target,Ability a,Game g){
        try{
            need(pendingActivation&&pending!=null&&a!=null&&pending.equals(a.getSourceId())&&s==pendingActor&&target!=null,"/unexpected activation target callback");
            pendingAbility=a;JsonObject e=next(s,g);if(e.get("kind").getAsString().equals("cancel_activation")){played.add(state(g,s,"before/"+cursor,null));accept(e);cancelRequested=true;return false;}need(e.get("kind").getAsString().equals("activation_target"),"/missing activation target");
            played.add(state(g,s,"before/"+cursor,null));UUID id=source(e,g);
            need(target.canTarget(p.getId(),id,a,g),"/selected target not legal");target.addTarget(id,a,g);need(target.getTargets().contains(id),"/selected target rejected");accept(e);return true;
        }catch(IllegalArgumentException error){failCallback(g,error);return false;}
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
    private boolean done(Game g){
        if(!g.getStack().isEmpty()||pending!=null)return false;
        String name=current.get("id").getAsString();
        if(name.startsWith("shivan")){Permanent p=g.getPermanent(handles.get("0/shivan-dragon/0"));return p!=null&&p.getPower().getValue()==(name.equals("shivan-repeated")?7:6);}
        if(name.startsWith("invoker")){Permanent p=g.getPermanent(handles.get("1/bear-cub/0"));return p!=null&&p.getPower().getValue()==7;}
        for(String id:Arrays.asList("1/llanowar-elves/1","1/druid-of-the-cowl/1")){Permanent p=g.getPermanent(handles.get(id));if(p==null||!p.isTapped()||!p.getAbilities().containsKey(mage.abilities.keyword.HasteAbility.getInstance().getId()))return false;}
        return g.getPlayer(players[1].getId()).getManaPool().getMana().getGreen()==2;
    }
    @Override protected boolean onPriority(TestPlayer p,int s,Game g) {
        try {
            need(!stopped && priorityFailure==null,"/repeated priority callback");
            if(!openingSeen){firstUpkeep(p,s,g);openingSeen=true;}
            need(p.getId().equals(g.getPriorityPlayerId()),"/priority provenance");
            if(done(g)){need(cursor==current.getAsJsonArray("play").size(),"/extra choice after named stop");played.add(state(g,s,current.get("stop").getAsString(),null));stopped=true;g.pause();return false;}
            if(g.getTurnStepType()==PhaseStep.DECLARE_ATTACKERS && !declarations.contains(g.getTurnNum()))emptyAttackers(p,s,g,true);
            JsonObject e=next(s,g);played.add(state(g,s,"before/"+cursor,null));String kind=e.get("kind").getAsString();
            switch(kind) {
                case "pass":accept(e);p.pass(g);return true;
                case "play_land":need(p.playLand(g.getCard(source(e,g)),g,false),"/selected land rejected");accept(e);return true;
                case "tap_mana":tap(p,e,g);return true;
                case "cast": {
                    UUID id=source(e,g);Card card=g.getCard(id);need(card.isCreature(g),"/unsupported noncreature spell");
                    List<mage.abilities.SpellAbility> candidates=new ArrayList<>();
                    for(mage.abilities.ActivatedAbility ability:p.getPlayableActivatedAbilities(card,Zone.HAND,g).values())
                        if(ability instanceof mage.abilities.SpellAbility && ability.getSourceId().equals(id))candidates.add((mage.abilities.SpellAbility)ability);
                    JsonObject check=new JsonObject();check.addProperty("source",bindings.get(id));check.addProperty("action",cursor);check.addProperty("candidate_count",candidates.size());
                    if(candidates.size()==1)check.addProperty("candidate_id",candidates.get(0).getId().toString());selectedCastChecks.add(check);
                    need(candidates.size()==1,"/selected cast not playable at /play/"+cursor);
                    pending=id;pendingAction=cursor;pendingOrigin=incarnation(id,g);pendingActor=s;pendingSources.clear();pendingAbility=null;castActions.put(id,cursor);
                    p.getManaPool().setAutoPayment(false);accept(e);
                    need(p.cast(candidates.get(0),g,false,null),"/selected cast rejected");
                    need(priorityFailure==null,"/failed payment callback");
                    JsonObject finish=next(s,g);need(finish.get("kind").getAsString().equals("finish_payment"),"/missing commit acknowledgement");
                    need(pendingAbility!=null,"/missing payment observation");
                    played.add(state(g,s,"before/"+cursor,pendingAbility.getManaCostsToPay().getUnpaid()));accept(finish);
                    pending=null;pendingAbility=null;pendingSources.clear();return true;
                }
                case "activate":{
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
                default:throw new IllegalArgumentException("first divergence: /unsupported priority callback");
            }
        } catch(IllegalArgumentException error){failCallback(g,error);return false;}
    }
    @Override protected JsonObject result(){JsonObject r=super.result();r.add("activation_checks",activationChecks);r.add("mana_checks",manaChecks);return r;}
    @Override @Test public void mulliganPrefixes() throws Exception {
        JsonObject doc=read(Paths.get(System.getProperty("mtglab.fixture")));validate(doc);
        JsonObject points=new JsonObject(),runs=new JsonObject(),repeat=new JsonObject();
        for(JsonElement element:doc.getAsJsonArray("cases")){JsonObject c=element.getAsJsonObject();String id=c.get("id").getAsString();JsonObject r=run(c);points.add(id,r.get("points"));runs.add(id,r);repeat.add(id,run(c));}
        JsonObject rejections=new JsonObject(),negativeRuns=new JsonObject();
        JsonObject negatives=read(Paths.get(System.getProperty("mtglab.root"),"fixtures/reference/full-pool-activations-xmage-negatives.json"));
        for(String name:negatives.keySet()){
            JsonObject spec=negatives.getAsJsonObject(name);JsonObject c=spec.getAsJsonObject("input").getAsJsonArray("cases").get(0).getAsJsonObject();
            try{run(c);fail("accepted negative "+name);}catch(IllegalArgumentException error){
                String prefix="first divergence: /play/"+spec.get("sequence").getAsInt()+" "+spec.get("category").getAsString();
                need(error.getMessage().startsWith(prefix),"/intended rejection boundary "+name+": expected "+prefix+" observed "+error.getMessage());
                rejections.addProperty(name,error.getMessage());negativeRuns.add(name,result());
            }
        }
        JsonObject cancelledRuns=new JsonObject();
        JsonObject cancellations=read(Paths.get(System.getProperty("mtglab.root"),"fixtures/reference/full-pool-activations-xmage-cancellations.json"));
        for(String name:cancellations.keySet()){
            JsonObject spec=cancellations.getAsJsonObject(name);JsonObject c=spec.getAsJsonObject("input").getAsJsonArray("cases").get(0).getAsJsonObject();
            JsonObject run=run(c);JsonObject value=new JsonObject();value.add(c.get("id").getAsString(),run);cancelledRuns.add(name,value);
        }
        JsonObject callbacks=new JsonObject();
        run(doc.getAsJsonArray("cases").get(0).getAsJsonObject());
        onSpellTarget(players[0],0,Outcome.Benefit,null,null,lastGame);need(priorityFailure!=null,"/unexpected target accepted");callbacks.addProperty("unexpected_target",priorityFailure.getMessage());
        run(doc.getAsJsonArray("cases").get(0).getAsJsonObject());
        onMana(players[0],0,null,null,lastGame);need(priorityFailure!=null,"/unexpected mana accepted");callbacks.addProperty("unexpected_mana",priorityFailure.getMessage());
        try{onSpellMode(players[0],0,null,null,lastGame);fail("unexpected mode accepted");}catch(IllegalArgumentException e){callbacks.addProperty("unexpected_mode",e.getMessage());}
        JsonObject output=new JsonObject();output.add("checkpoints",points);output.add("runs",runs);output.add("repeat_runs",repeat);output.add("rejections",rejections);output.add("negative_runs",negativeRuns);output.add("cancelled_runs",cancelledRuns);output.add("callback_controls",callbacks);
        Files.write(Paths.get(System.getProperty("mtglab.output")),JSON.toJson(output).getBytes(StandardCharsets.UTF_8));
    }
}
