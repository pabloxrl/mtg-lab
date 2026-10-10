package org.mage.test.mtglab;

import com.google.gson.*;
import mage.abilities.*;
import mage.abilities.costs.mana.ManaCost;
import mage.constants.*;
import mage.cards.Card;
import mage.game.Game;
import mage.game.events.GameEvent;
import mage.game.events.ZoneChangeEvent;
import mage.game.permanent.Permanent;
import mage.game.stack.Spell;
import mage.game.stack.StackObject;
import mage.players.Player;
import mage.target.Target;
import mage.watchers.Watcher;
import org.junit.Test;
import org.mage.test.player.TestPlayer;
import java.util.*;
import java.nio.file.*;
import java.nio.charset.StandardCharsets;
import static org.junit.Assert.*;

/** Version 6 extends actual played prefixes with strict trigger callbacks and identities. */
public class FullPoolTriggersTest extends FullPoolPriorityTest {
    private JsonArray creations,departures,callbackEvidence;
    private final Map<UUID,JsonObject> triggerKeys=new LinkedHashMap<>();
    private JsonArray triggerRaw=new JsonArray();
    private final List<UUID> placementOrder=new ArrayList<>();
    private int eventOrdinal;
    private boolean batchActive;
    private String triggerBoundary="settled";
    private mage.abilities.TriggeredAbility placing;

    private final Map<UUID,Integer> tokenCounters=new LinkedHashMap<>();
    private final Map<UUID,Integer> eventOrdinals=new LinkedHashMap<>();
    private final Map<UUID,Integer> tokenOrdinals=new LinkedHashMap<>();
    private boolean observerInstalled;
    private int targetIndex;
    private final Map<UUID,Map<UUID,JsonObject>> chosenTargets=new LinkedHashMap<>();

    @Override protected void validate(JsonObject doc) {
        need(integer(doc.get("schema_version"),6)&&doc.get("family").getAsString().equals("triggers"),"/version/family");
        JsonObject projection=doc.deepCopy();projection.addProperty("schema_version",3);projection.addProperty("family","priority");
        for(JsonElement element:projection.getAsJsonArray("cases")) {
            JsonObject c=element.getAsJsonObject();need(c.get("stop").getAsString().startsWith("triggers_settled/"),"/stop");c.addProperty("stop","second_creature_resolved");
            for(JsonElement action:c.getAsJsonArray("play")) {
                JsonObject e=action.getAsJsonObject();keys(e,"sequence","turn","step","actor","kind","source","incarnation","color","role","mode","order","player");e.remove("role");e.remove("mode");e.remove("order");e.remove("player");
            }
        }
        super.validate(projection);
    }
    @Override protected void prepareCase(JsonObject c) {
        super.prepareCase(c);triggerKeys.clear();triggerRaw=new JsonArray();placementOrder.clear();eventOrdinal=0;batchActive=false;placing=null;triggerBoundary="settled";creations=new JsonArray();departures=new JsonArray();callbackEvidence=new JsonArray();
        tokenCounters.clear();eventOrdinals.clear();tokenOrdinals.clear();observerInstalled=false;targetIndex=0;chosenTargets.clear();
    }
    private class SpellObserver extends Watcher {
        SpellObserver(){super(WatcherScope.GAME);}
        SpellObserver(SpellObserver other){super(other);}
        @Override public Watcher copy(){return new SpellObserver(this);}
        @Override public void watch(GameEvent e,Game g) {
            if(e.getType()==GameEvent.EventType.CREATED_TOKEN) {
                Permanent p=g.getPermanent(e.getTargetId());need(p!=null&&p.isToken()&&p.getName().equals("Goblin Token"),"/unsupported token creation event");
                need(!bindings.containsKey(p.getId()),"/duplicate token id");
                UUID source=e.getSourceId();need(source!=null&&castActions.containsKey(source),"/unwitnessed creation source");
                int event=eventOrdinals.computeIfAbsent(source,k->eventOrdinals.size());
                int ordinal=tokenOrdinals.getOrDefault(source,0);tokenOrdinals.put(source,ordinal+1);
                String id="token/"+seat(p.getControllerId())+"/"+event+"/"+ordinal;
                need(!bindings.containsKey(p.getId())&&!handles.containsKey(id),"/duplicate token id");
                bindings.put(p.getId(),id);handles.put(id,p.getId());tokenCounters.put(p.getId(),p.getZoneChangeCounter(g));
                JsonObject birth=new JsonObject();birth.addProperty("id",id);birth.addProperty("raw_uuid",p.getId().toString());birth.addProperty("raw_counter",p.getZoneChangeCounter(g));
                birth.addProperty("red",p.getColor(g).isRed());birth.addProperty("white",p.getColor(g).isWhite());birth.addProperty("blue",p.getColor(g).isBlue());birth.addProperty("black",p.getColor(g).isBlack());birth.addProperty("green",p.getColor(g).isGreen());
                JsonArray subtypes=new JsonArray();for(SubType subtype:p.getSubtype(g))subtypes.add(subtype.name());birth.add("subtypes",subtypes);
                birth.addProperty("incarnation",0);birth.addProperty("action",cursor-1);birth.addProperty("source",bindings.get(source));creations.add(birth);
            }
            if(e instanceof ZoneChangeEvent) {
                ZoneChangeEvent z=(ZoneChangeEvent)e;
                if(z.getFromZone()==Zone.BATTLEFIELD && bindings.containsKey(e.getTargetId())) {
                    UUID id=e.getTargetId();JsonObject d=new JsonObject();d.addProperty("source",bindings.get(id));
                    d.addProperty("from",z.getFromZone().name());d.addProperty("to",z.getToZone().name());d.addProperty("action",cursor-1);
                    d.addProperty("departing_incarnation",tokenCounters.containsKey(id)?0:semanticIncarnations.get(id));departures.add(d);
                }
            }
        }
        @Override public void reset(){super.reset();}
    }
    @Override protected int incarnation(UUID id,Game g) {
        if(tokenCounters.containsKey(id)) {
            Permanent p=g.getPermanent(id);need(p!=null&&p.getZoneChangeCounter(g)==tokenCounters.get(id),"/departed token incarnation");return 0;
        }
        return super.incarnation(id,g);
    }
    @Override protected JsonObject next(int actor,Game g) {
        need(cursor<current.getAsJsonArray("play").size(),"/missing choice before named stop");
        JsonObject e=current.getAsJsonArray("play").get(cursor).getAsJsonObject();
        need(integer(e.get("sequence"),cursor)&&integer(e.get("actor"),actor),"/play actor/sequence");
        need(integer(e.get("turn"),g.getTurnNum())&&e.get("step").getAsString().equals(g.getTurnStepType().name().toLowerCase(Locale.ROOT)),"/play position");
        String k=e.get("kind").getAsString();need(k.equals("pay")==!e.get("color").isJsonNull(),"/payment color field");
        need(k.equals("order_triggers")==!e.get("order").isJsonNull(),"/trigger order field");need(k.equals("target_player")==!e.get("player").isJsonNull(),"/player target field");
        need(k.equals("mode")==!e.get("mode").isJsonNull(),"/mode field");need(k.equals("target")==!e.get("role").isJsonNull(),"/target role field");
        boolean object=Arrays.asList("cast","play_land","tap_mana","target","discard").contains(k);
        need(object==!e.get("source").isJsonNull() && object==!e.get("incarnation").isJsonNull(),"/source/incarnation field");return e;
    }
    @Override protected UUID source(JsonObject e,Game g) {
        need(e.get("source").isJsonPrimitive()&&e.get("source").getAsJsonPrimitive().isString(),"/source type");
        UUID id=handles.get(e.get("source").getAsString());need(id!=null,"/source unknown");
        need(integer(e.get("incarnation"),incarnation(id,g)),"/source incarnation");return id;
    }
    private JsonObject ref(UUID id,Game g) {
        if(tokenCounters.containsKey(id)&&g.getPermanent(id)==null)return null;
        JsonObject v=new JsonObject();need(bindings.containsKey(id),"/unbound target");v.addProperty("source",bindings.get(id));
        if(tokenCounters.containsKey(id))v.addProperty("incarnation",0);else v.addProperty("incarnation",semanticIncarnations.get(id));return v;
    }
    private JsonArray targets(Ability a,Game g) {
        JsonArray result=new JsonArray();int index=0;
        for(Target t:a.getTargets())for(UUID id:t.getTargets()) {
            JsonObject v=new JsonObject();String name=g.getCard(a.getSourceId()).getName();
            String role=name.equals("Giant Growth")?"growth_target":index++==0?"bite_source":"bite_destination";
            need(name.equals("Giant Growth")||name.equals("Bite Down"),"/unsupported spell target roles");
            v.addProperty("role",role);
            JsonObject live=ref(id,g),selected=chosenTargets.getOrDefault(a.getSourceId(),Collections.emptyMap()).get(id);
            need(selected!=null,"/target has no selection provenance");
            v.add("object",g.getPermanent(id)!=null && selected.equals(live)?live:JsonNull.INSTANCE);result.add(v);
        }
        return result;
    }
    @Override protected JsonObject state(Game g,int actor,String boundary,ManaCost unpaid) {
        observeIdentities(g,false);capturePending(g);
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
            JsonObject v=new JsonObject();v.addProperty("card",o.isToken()?"goblin-token":id.split("/")[1]);v.addProperty("owner",seat(o.getOwnerId()));v.addProperty("controller",seat(o.getControllerId()));v.addProperty("tapped",o.isTapped());v.addProperty("sick",o.isCreature(g)&&o.hasSummoningSickness());
            v.add("power",o.isCreature(g)?new JsonPrimitive(o.getPower().getValue()):JsonNull.INSTANCE);v.add("toughness",o.isCreature(g)?new JsonPrimitive(o.getToughness().getValue()):JsonNull.INSTANCE);v.add("damage",o.isCreature(g)?new JsonPrimitive(o.getDamage()):JsonNull.INSTANCE);
            if(o.isToken()){need(o.getColor(g).isRed()&&o.getSubtype(g).contains(SubType.GOBLIN),"/token characteristics");}
            permanents.add(id,v);
        }
        List<StackObject> order=new ArrayList<>();for(StackObject o:g.getStack())order.add(o);Collections.reverse(order);
        for(StackObject o:order) {
            if(o instanceof mage.game.stack.StackAbility) {
                mage.game.stack.StackAbility stackAbility=(mage.game.stack.StackAbility)o;
                Ability a=stackAbility.getStackAbility();need(triggerKeys.containsKey(a.getId()),"/unwitnessed stack trigger");
                JsonObject v=new JsonObject();v.addProperty("ability","trigger");v.add("key",triggerKeys.get(a.getId()).deepCopy());v.addProperty("controller",seat(a.getControllerId()));
                JsonArray targets=new JsonArray();for(Target t:a.getTargets())for(UUID id:t.getTargets())targets.add(seat(id));
                need(targets.size()<=1,"/trigger target cardinality");v.add("target",targets.size()==1?targets.get(0):JsonNull.INSTANCE);
                v.addProperty("raw_uuid",o.getId().toString());v.addProperty("raw_source",a.getSourceId().toString());v.addProperty("raw_source_zcc",a.getStackMomentSourceZCC());v.addProperty("source_card",bindings.get(a.getSourceId()).split("/")[1]);stack.add(v);rawStackIds.add(o.getId().toString(),v.deepCopy());continue;
            }
            need(o instanceof Spell && castActions.containsKey(o.getSourceId()),"/unsupported stack object");Spell spell=(Spell)o;
            JsonObject v=new JsonObject();v.addProperty("source",bindings.get(o.getSourceId()));v.addProperty("incarnation",incarnation(o.getSourceId(),g));v.addProperty("action",castActions.get(o.getSourceId()));
            v.add("targets",targets(spell.getSpellAbility(),g));
            if(spell.getSpellAbility().getModes().size()>1 && !spell.getSpellAbility().getModes().getSelectedModes().isEmpty()) {
                List<UUID> modes=new ArrayList<>(spell.getSpellAbility().getModes().keySet());
                need(spell.getSpellAbility().getModes().getSelectedModes().size()==1,"/mode cardinality");
                v.addProperty("mode",modes.indexOf(spell.getSpellAbility().getModes().getSelectedModes().get(0)));
            } else v.add("mode",JsonNull.INSTANCE);
            stack.add(v);rawStackIds.add(o.getId().toString(),v.deepCopy());
        }
        p.add("battlefield",battlefield);p.add("permanents",permanents);p.add("incarnations",incarnations);p.add("stack",stack);
        p.add("payment",JsonNull.INSTANCE);p.add("targeting",JsonNull.INSTANCE);
        if(pending!=null) {
            JsonObject payment=new JsonObject();payment.addProperty("source",bindings.get(pending));payment.addProperty("actor",pendingActor);
            payment.add("pool",mana(g.getPlayer(players[pendingActor].getId()).getManaPool().getMana()));
            if(unpaid!=null){payment.add("colored",mana(unpaid.getMana()));payment.addProperty("generic",unpaid.getMana().getGeneric());}
            payment.add("sources",ids(pendingSources,false));p.add("payment",payment);
        }
        JsonArray effects=new JsonArray();
        for(mage.abilities.effects.ContinuousEffect effect:g.getContinuousEffects().getLayeredEffects(g)) {
            JsonObject observed=new JsonObject();observed.addProperty("raw_id",effect.getId().toString());observed.addProperty("class",effect.getClass().getName());observed.addProperty("duration",effect.getDuration().name());
            JsonArray sources=new JsonArray();for(Ability a:g.getContinuousEffects().getLayeredEffectAbilities(effect)){need(bindings.containsKey(a.getSourceId()),"/unbound effect source");sources.add(bindings.get(a.getSourceId()));}observed.add("sources",sources);effects.add(observed);
        }
        p.add("effects",effects);
        p.add("creations",creations.deepCopy());p.add("departures",departures.deepCopy());
        JsonArray pendingTriggers=new JsonArray();for(int seat=0;seat<2;seat++)for(TriggeredAbility a:g.getState().getTriggered(players[seat].getId())) {
            JsonObject v=new JsonObject();v.add("key",triggerKeys.get(a.getId()));v.addProperty("controller",seat);pendingTriggers.add(v);
        }
        p.add("pending_triggers",pendingTriggers);p.addProperty("trigger_boundary",triggerBoundary);return p;
    }
    @Override protected boolean onSpellTarget(TestPlayer player,int s,Outcome outcome,Target target,Ability ability,Game g) {
        try {
            if(ability instanceof TriggeredAbility) {
                need(!stopped&&placing!=null&&ability.getId().equals(placing.getId()),"/unexpected ETB target callback");
                triggerBoundary="target";JsonObject e=next(s,g);played.add(state(g,s,"before/"+cursor,null));
                if(e.get("kind").getAsString().equals("target")) {
                    UUID invalid=source(e,g);need(target.canTarget(players[s].getId(),invalid,ability,g),"/illegal ETB player target");
                    throw new IllegalArgumentException("first divergence: /unexpected legal creature ETB target");
                }
                need(e.get("kind").getAsString().equals("target_player"),"/missing ETB player target");
                int targetSeat=e.get("player").getAsInt();need(integer(e.get("player"),targetSeat)&&targetSeat>=0&&targetSeat<2,"/illegal ETB player target");
                UUID id=players[targetSeat].getId();need(target.canTarget(players[s].getId(),id,ability,g),"/illegal ETB player target");
                target.addTarget(id,ability,g);need(target.getTargets().contains(id),"/ETB target rejected");
                JsonObject witness=e.deepCopy();witness.addProperty("callback","chooseTarget");witness.addProperty("raw_ability",ability.getId().toString());callbackEvidence.add(witness);accept(e);return true;
            }
            need(!stopped&&pending!=null&&ability!=null&&ability.getSourceId().equals(pending)&&s==pendingActor&&target!=null,"/unexpected target callback");
            JsonObject e=next(s,g);String kind=e.get("kind").getAsString();need(kind.equals("target")||kind.equals("discard"),"/missing target/discard choice");
            played.add(state(g,s,"before/"+cursor,null));UUID id=source(e,g);
            if(kind.equals("target")) {
                String card=g.getCard(pending).getName();String role=card.equals("Giant Growth")?"growth_target":targetIndex==0?"bite_source":"bite_destination";
                need((card.equals("Giant Growth")||card.equals("Bite Down"))&&e.get("role").getAsString().equals(role),"/target role");targetIndex++;
            } else need(outcome==Outcome.Discard&&player.getHand().contains(id)&&!id.equals(pending),"/discard source");
            need(target.canTarget(player.getId(),id,ability,g),"/selected target not legal");target.addTarget(id,ability,g);
            need(target.getTargets().contains(id),"/selected target rejected");
            if(kind.equals("target"))chosenTargets.computeIfAbsent(pending,k->new LinkedHashMap<>()).put(id,ref(id,g));
            JsonObject witness=e.deepCopy();witness.addProperty("callback_target_class",target.getClass().getName());witness.addProperty("raw_target",id.toString());callbackEvidence.add(witness);accept(e);return true;
        } catch(IllegalArgumentException error){failCallback(g,error);return false;}
    }
    @Override protected Mode onSpellMode(TestPlayer p,int s,Modes modes,Ability ability,Game g) {
        try {
            need(!stopped&&pending!=null&&ability!=null&&pending.equals(ability.getSourceId())&&s==pendingActor,"/unexpected mode callback");
            JsonObject e=next(s,g);need(e.get("kind").getAsString().equals("mode"),"/missing mode");int n=e.get("mode").getAsInt();
            need(integer(e.get("mode"),n)&&n>=0&&n<modes.size(),"/illegal mode");played.add(state(g,s,"before/"+cursor,null));
            Mode m=new ArrayList<>(modes.values()).get(n);JsonObject witness=e.deepCopy();witness.addProperty("raw_mode",m.getId().toString());callbackEvidence.add(witness);accept(e);return m;
        } catch(IllegalArgumentException error){failCallback(g,error);return null;}
    }
    @Override protected boolean onMana(TestPlayer p,int s,Ability ability,ManaCost unpaid,Game g) {
        if(priorityFailure!=null)return false;
        try {
            need(!stopped && pending!=null && ability!=null && ability.getSourceId().equals(pending) && s==pendingActor,"/unscripted mana callback");
            if(targetIndex>0 && cursor<current.getAsJsonArray("play").size() && current.getAsJsonArray("play").get(cursor).getAsJsonObject().get("kind").getAsString().equals("finish_targets")) {
                JsonObject e=next(s,g);played.add(state(g,s,"before/"+cursor,unpaid));accept(e);
            }
            JsonObject next=next(s,g);
            if(next.get("kind").getAsString().equals("cancel_payment")) {
                played.add(state(g,s,"before/"+cursor,unpaid));accept(next);return false;
            }
            return super.onMana(p,s,ability,unpaid,g);
        }catch(IllegalArgumentException error){failCallback(g,error);return false;}
    }
    @Override protected boolean onPriority(TestPlayer p,int s,Game g) {
        try {
            need(!stopped&&priorityFailure==null,"/repeated priority callback");
            if(!openingSeen){firstUpkeep(p,s,g);openingSeen=true;}
            if(!observerInstalled){g.getState().addWatcher(new SpellObserver());observerInstalled=true;}
            triggerBoundary="settled";placing=null;
            if(eventOrdinal>0&&g.getState().getZone(handles.get(current.get("stop").getAsString().substring("triggers_settled/".length())))==Zone.GRAVEYARD&&g.getStack().isEmpty()&&pending==null) {
                need(cursor==current.getAsJsonArray("play").size(),"/extra choice after named stop");need(placementOrder.isEmpty(),"/unconsumed trigger order");played.add(state(g,s,current.get("stop").getAsString(),null));stopped=true;g.pause();return false;
            }
            if(g.getTurnStepType()==PhaseStep.DECLARE_ATTACKERS&&!declarations.contains(g.getTurnNum()))emptyAttackers(p,s,g,true);
            JsonObject e=next(s,g);played.add(state(g,s,"before/"+cursor,null));String kind=e.get("kind").getAsString();
            switch(kind) {
                case "pass":accept(e);p.pass(g);return true;
                case "play_land":need(p.playLand(g.getCard(source(e,g)),g,false),"/selected land rejected");accept(e);return true;
                case "tap_mana":tap(p,e,g);return true;
                case "cast": {
                    UUID id=source(e,g);Card card=g.getCard(id);List<SpellAbility> candidates=new ArrayList<>();
                    for(ActivatedAbility a:p.getPlayableActivatedAbilities(card,Zone.HAND,g).values())if(a instanceof SpellAbility&&a.getSourceId().equals(id))candidates.add((SpellAbility)a);
                    need(candidates.size()==1,"/selected cast not playable");
                    JsonObject check=new JsonObject();check.addProperty("source",bindings.get(id));check.addProperty("action",cursor);check.addProperty("candidate_id",candidates.get(0).getId().toString());check.addProperty("candidate_count",candidates.size());selectedCastChecks.add(check);
                    pending=id;pendingAction=cursor;pendingOrigin=incarnation(id,g);pendingActor=s;pendingSources.clear();pendingAbility=null;targetIndex=0;chosenTargets.remove(id);castActions.put(id,cursor);
                    p.getManaPool().setAutoPayment(false);accept(e);need(p.cast(candidates.get(0),g,false,null),"/selected cast rejected");need(priorityFailure==null,"/failed spell callback");
                    JsonObject finish=next(s,g);need(finish.get("kind").getAsString().equals("finish_payment"),"/missing commit acknowledgement");need(pendingAbility!=null,"/missing payment observation");
                    played.add(state(g,s,"before/"+cursor,pendingAbility.getManaCostsToPay().getUnpaid()));accept(finish);pending=null;pendingAbility=null;pendingSources.clear();return true;
                }
                default:throw new IllegalArgumentException("first divergence: /unsupported priority callback");
            }
        }catch(IllegalArgumentException error){failCallback(g,error);return false;}
    }
    private void capturePending(Game g) {
        List<TriggeredAbility> all=new ArrayList<>();for(int s=0;s<2;s++)all.addAll(g.getState().getTriggered(players[s].getId()));
        if(all.isEmpty())return;
        if(!batchActive) {
            observeIdentities(g,false);batchActive=true;
            for(TriggeredAbility a:all)registerTrigger(a,g,eventOrdinal);
            eventOrdinal++;
        }
    }
    private void registerTrigger(TriggeredAbility a,Game g,int event) {
        UUID id=a.getSourceId();need(bindings.containsKey(id),"/unwitnessed trigger source");
        String card=g.getCard(id).getName();String kind=card.equals("Firebrand Archer")?"archer":card.equals("Crackling Cyclops")?"cyclops":card.equals("Viashino Pyromancer")?"pyromancer":null;
        need(kind!=null&&g.getPermanent(id)!=null,"/unsupported trigger source");
        JsonObject key=new JsonObject();key.addProperty("source",bindings.get(id));key.addProperty("incarnation",incarnation(id,g));key.addProperty("ability",kind);key.addProperty("event",event);triggerKeys.put(a.getId(),key);
        JsonObject raw=key.deepCopy();raw.addProperty("raw_ability",a.getId().toString());raw.addProperty("raw_source",id.toString());raw.addProperty("raw_source_zcc",a.getStackMomentSourceZCC());raw.addProperty("ability_class",a.getClass().getName());raw.addProperty("observed_choice",cursor);triggerRaw.add(raw);
    }
    private void selectOrder(int s,List<TriggeredAbility> abilities,Game g) {
        need(!stopped&&placementOrder.isEmpty()&&!abilities.isEmpty(),"/unexpected trigger order callback");
        observeIdentities(g,false);capturePending(g);
        if(!batchActive) {batchActive=true;for(TriggeredAbility a:abilities)registerTrigger(a,g,eventOrdinal);eventOrdinal++;}
        triggerBoundary="order";JsonObject e=next(s,g);played.add(state(g,s,"before/"+cursor,null));
        need(e.get("kind").getAsString().equals("order_triggers"),"/missing trigger order");JsonArray order=e.getAsJsonArray("order");need(order.size()==abilities.size(),"/trigger order cardinality");
        for(JsonElement key:order) {
            TriggeredAbility match=null;for(TriggeredAbility a:abilities)if(key.equals(triggerKeys.get(a.getId()))) {need(match==null,"/ambiguous trigger identity");match=a;}
            need(match!=null&&!placementOrder.contains(match.getId()),"/trigger source/event");placementOrder.add(match.getId());
        }
        JsonObject witness=e.deepCopy();witness.addProperty("callback",abilities.size()>1?"chooseTriggeredAbility":"triggerAbility_single");callbackEvidence.add(witness);accept(e);
    }
    @Override protected TriggeredAbility onTriggerOrder(TestPlayer p,int s,List<TriggeredAbility> abilities,Game g) {
        try {if(placementOrder.isEmpty())selectOrder(s,abilities,g);
            UUID wanted=placementOrder.get(0);for(TriggeredAbility a:abilities)if(a.getId().equals(wanted))return a;
            throw new IllegalArgumentException("first divergence: /trigger source/event");
        }catch(IllegalArgumentException e){failCallback(g,e);throw e;}
    }
    @Override protected void onTriggerPlacement(TestPlayer p,int s,TriggeredAbility a,Game g) {
        try {
            need(!stopped&&a!=null,"/unexpected trigger placement callback");
            if(placementOrder.isEmpty())selectOrder(s,Collections.singletonList(a),g);
            need(placementOrder.remove(0).equals(a.getId()),"/trigger placement order");placing=a;
            if(placementOrder.isEmpty())batchActive=false;
        }catch(IllegalArgumentException e){failCallback(g,e);throw e;}
    }
    @Override protected JsonObject result() {
        JsonObject r=super.result();r.add("spell_callback_evidence",callbackEvidence);r.add("trigger_provenance",JSON.toJsonTree(triggerRaw));
        int pendingCount=0,stackCount=0;for(int s=0;s<2;s++)pendingCount+=lastGame.getState().getTriggered(players[s].getId()).size();
        for(StackObject o:lastGame.getStack())if(o instanceof mage.game.stack.StackAbility)stackCount++;
        JsonObject counts=new JsonObject();counts.addProperty("pending",pendingCount);counts.addProperty("stack",stackCount);r.add("post_run_trigger_counts",counts);
        if(r.get("opening").isJsonNull()) {
            JsonObject opening=new JsonObject();opening.add("points",observed);opening.add("consumed_chance",consumed);opening.add("consumed_choices",consumedChoices);opening.add("raw_callbacks",rawCallbacks);r.add("opening",opening);
        }
        return r;
    }
    @Override @Test public void mulliganPrefixes() throws Exception {
        JsonObject doc=read(Paths.get(System.getProperty("mtglab.fixture")));validate(doc);
        JsonObject points=new JsonObject(),runs=new JsonObject(),repeats=new JsonObject(),rejections=new JsonObject(),negativeRuns=new JsonObject(),callbacks=new JsonObject();
        for(JsonElement element:doc.getAsJsonArray("cases")) {
            JsonObject c=element.getAsJsonObject();JsonObject r=run(c);points.add(c.get("id").getAsString(),r.get("points"));runs.add(c.get("id").getAsString(),r);repeats.add(c.get("id").getAsString(),run(c));
        }
        JsonObject negatives=read(root.resolve("fixtures/reference/full-pool-triggers-xmage-negatives.json"));
        for(String name:negatives.keySet()) {
            JsonObject control=negatives.getAsJsonObject(name);
            try {JsonObject d=control.getAsJsonObject("input");validate(d);run(d.getAsJsonArray("cases").get(0).getAsJsonObject());fail("accepted negative "+name);}
            catch(IllegalArgumentException error) {
                String expected="first divergence: /play/"+control.get("sequence").getAsInt()+" "+control.get("category").getAsString();
                need(error.getMessage().startsWith(expected),"/intended rejection boundary "+name+": "+error.getMessage());
                need(name.equals("truncated_tape")||!error.getMessage().contains("missing choice before named stop"),"/late exhaustion "+name);
                rejections.addProperty(name,error.getMessage());negativeRuns.add(name,read(Paths.get(System.getProperty("mtglab.output")+".failure.json")));
            }
        }
        run(doc.getAsJsonArray("cases").get(0).getAsJsonObject());
        try {onTriggerOrder(players[0],0,Collections.emptyList(),lastGame);fail("extra trigger callback accepted");}
        catch(IllegalArgumentException error){callbacks.addProperty("extra_order",error.getMessage());}
        priorityFailure=null;
        try {onTriggerPlacement(players[0],0,null,lastGame);fail("extra placement callback accepted");}
        catch(IllegalArgumentException error){callbacks.addProperty("extra_placement",error.getMessage());}
        JsonObject output=new JsonObject();output.add("checkpoints",points);output.add("runs",runs);output.add("repeat_runs",repeats);output.add("rejections",rejections);output.add("negative_runs",negativeRuns);output.add("callback_controls",callbacks);
        Files.write(Paths.get(System.getProperty("mtglab.output")),JSON.toJson(output).getBytes(StandardCharsets.UTF_8));
    }
}
