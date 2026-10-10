package org.mage.test.mtglab;

import com.google.gson.*;
import mage.Mana;
import mage.abilities.Ability;
import mage.abilities.costs.mana.ManaCost;
import mage.abilities.mana.ActivatedManaAbilityImpl;
import mage.cards.Card;
import mage.constants.*;
import mage.game.Game;
import mage.game.permanent.Permanent;
import mage.game.stack.Spell;
import mage.game.stack.StackObject;
import mage.players.Player;
import org.junit.Test;
import org.mage.test.player.TestPlayer;
import java.nio.file.*;
import java.nio.charset.StandardCharsets;
import java.util.*;
import static org.junit.Assert.*;

/** Continuous normal-reset CR117/305/601/608 client; never resolves the stack directly. */
public class FullPoolPriorityTest extends FullPoolMulliganTest {
    protected final Map<UUID,Integer> initialCounters=new LinkedHashMap<>();
    protected final Map<UUID,Integer> witnessedCounters=new LinkedHashMap<>(),semanticIncarnations=new LinkedHashMap<>();
    protected final Map<UUID,Zone> witnessedZones=new LinkedHashMap<>();
    protected JsonArray identityTransitions;
    protected final Map<UUID,Integer> castActions=new LinkedHashMap<>();
    protected final Map<UUID,Integer> stackActions=new LinkedHashMap<>();
    protected final Set<Integer> declarations=new HashSet<>();
    protected JsonArray played,usedPlay,stops,selectedCastChecks;
    protected JsonObject rawStackIds,openingResult;
    protected int cursor,pendingAction,pendingOrigin,pendingActor;
    protected UUID pending;
    protected Ability pendingAbility;
    protected final List<UUID> pendingSources=new ArrayList<>();
    protected boolean openingSeen;

    @Override protected void validate(JsonObject doc) {
        need(integer(doc.get("schema_version"),3),"/schema_version");
        need(doc.get("family").getAsString().equals("priority"),"/family");
        JsonObject projection=doc.deepCopy();projection.addProperty("schema_version",2);projection.addProperty("family","mulligan");
        for(JsonElement element:projection.getAsJsonArray("cases")) {
            JsonObject c=element.getAsJsonObject();
            need(c.has("play") && c.get("play").isJsonArray(),"/missing play");c.remove("play");c.addProperty("stop","first_upkeep");
        }
        super.validate(projection);
        for(JsonElement element:doc.getAsJsonArray("cases")) {
            JsonObject c=element.getAsJsonObject();
            need(Arrays.asList("first_cast_committed","second_creature_resolved").contains(c.get("stop").getAsString()),"/unsupported stop");
            for(int i=0;i<c.getAsJsonArray("play").size();i++) {
                JsonObject e=c.getAsJsonArray("play").get(i).getAsJsonObject();
                keys(e,"sequence","turn","step","actor","kind","source","incarnation","color");
                need(integer(e.get("sequence"),i),"/play sequence");
                need(integer(e.get("actor"),0)||integer(e.get("actor"),1),"/play actor");
                need(e.get("turn").isJsonPrimitive() && e.get("turn").getAsJsonPrimitive().isNumber(),"/play turn type");
            }
        }
    }
    @Override protected void prepareCase(JsonObject c) {
        initialCounters.clear();witnessedCounters.clear();semanticIncarnations.clear();witnessedZones.clear();identityTransitions=new JsonArray();castActions.clear();stackActions.clear();declarations.clear();pendingSources.clear();
        cursor=0;pending=null;pendingAbility=null;openingSeen=false;
        played=new JsonArray();usedPlay=new JsonArray();stops=new JsonArray();selectedCastChecks=new JsonArray();rawStackIds=new JsonObject();openingResult=null;
    }
    @Override protected void initialLibrary(int s,Game g) {
        need(g.getPlayer(players[s].getId()).getLibrary().size()==40,"/initial library count");
        for(UUID id:g.getPlayer(players[s].getId()).getLibrary().getCardList()) {
            need(bindings.containsKey(id) && !initialCounters.containsKey(id),"/initial physical binding");
            initialCounters.put(id,g.getCard(id).getZoneChangeCounter(g));
            need(g.getState().getZone(id)==Zone.LIBRARY,"/initial zone provenance");
            witnessedCounters.put(id,g.getCard(id).getZoneChangeCounter(g));witnessedZones.put(id,Zone.LIBRARY);semanticIncarnations.put(id,0);
        }
    }
    @Override protected void raw(Game g,int s,String kind) {
        super.raw(g,s,kind);observeIdentities(g,kind.equals("mulligan_shuffle"));
    }
    protected void observeIdentities(Game g,boolean mulliganReturn) {
        for(UUID id:initialCounters.keySet()) {
            Zone before=witnessedZones.get(id),after=g.getState().getZone(id);
            int previous=witnessedCounters.get(id),counter=g.getCard(id).getZoneChangeCounter(g);
            if(before==after)need(counter==previous,"/unobserved round-trip zone change");
            else {
                // LondonMulligan uses Library.addAll + hand.clear: the actual
                // return is witnessed at shuffle, but has no raw ZCC increment.
                boolean directReturn=mulliganReturn && before==Zone.HAND && after==Zone.LIBRARY;
                need(counter-previous==(directReturn?0:1),"/unsupported zone-counter transition");
                int incarnation=semanticIncarnations.get(id)+1;semanticIncarnations.put(id,incarnation);
                JsonObject event=new JsonObject();event.addProperty("source",bindings.get(id));event.addProperty("from",before.name());event.addProperty("to",after.name());
                event.addProperty("raw_before",previous);event.addProperty("raw_after",counter);event.addProperty("incarnation",incarnation);event.addProperty("direct_mulligan_return",directReturn);identityTransitions.add(event);
            }
            witnessedZones.put(id,after);witnessedCounters.put(id,counter);
        }
    }
    protected int incarnation(UUID id,Game g) {
        need(initialCounters.containsKey(id) && g.getCard(id)!=null,"/unwitnessed incarnation");
        return semanticIncarnations.get(id);
    }
    protected static JsonArray mana(Mana m) {
        JsonArray result=new JsonArray();
        for(int n:new int[]{m.getWhite(),m.getBlue(),m.getBlack(),m.getRed(),m.getGreen(),m.getColorless()})result.add(n);
        return result;
    }
    protected JsonObject state(Game g,int actor,String boundary,ManaCost unpaid) {
        observeIdentities(g,false);
        need(!g.hasEnded() && g.getCards().size()==80 && initialCounters.size()==80,"/unbound or terminal state");
        JsonObject point=new JsonObject();point.addProperty("boundary",boundary);point.addProperty("turn",g.getTurnNum());
        point.addProperty("step",g.getTurnStepType().name().toLowerCase(Locale.ROOT));point.addProperty("active",seat(g.getActivePlayerId()));point.addProperty("actor",actor);
        JsonArray life=new JsonArray(),pools=new JsonArray(),hands=new JsonArray(),libraries=new JsonArray(),graves=new JsonArray();
        for(int s=0;s<2;s++) {
            Player p=g.getPlayer(players[s].getId());life.add(p.getLife());pools.add(mana(p.getManaPool().getMana()));
            hands.add(ids(p.getHand(),false));libraries.add(ids(p.getLibrary().getCardList(),false));graves.add(ids(p.getGraveyard(),false));
        }
        point.add("life",life);point.add("mana",pools);point.addProperty("land_plays",g.getPlayer(g.getActivePlayerId()).getLandsPlayed());
        List<UUID> exile=new ArrayList<>();for(Card card:g.getExile().getCardsOwned(g,null))exile.add(card.getId());
        point.add("hand",hands);point.add("library",libraries);point.add("graveyard",graves);point.add("exile",ids(exile,false));
        JsonArray battlefield=new JsonArray(),stack=new JsonArray();JsonObject incarnations=new JsonObject(),permanents=new JsonObject();
        Set<UUID> witnessed=new HashSet<>();
        for(int s=0;s<2;s++){Player p=g.getPlayer(players[s].getId());witnessed.addAll(p.getHand());witnessed.addAll(p.getLibrary().getCardList());witnessed.addAll(p.getGraveyard());}
        witnessed.addAll(exile);
        for(Permanent p:g.getBattlefield().getAllActivePermanents()) {
            need(bindings.containsKey(p.getId()),"/unknown permanent");witnessed.add(p.getId());String id=bindings.get(p.getId());battlefield.add(id);
            need(p.getOwnerId().equals(p.getControllerId()),"/unsupported controller change");
            JsonObject value=new JsonObject();value.addProperty("tapped",p.isTapped());value.addProperty("sick",p.isCreature(g)&&p.hasSummoningSickness());
            if(p.isCreature(g)){value.addProperty("power",p.getPower().getValue());value.addProperty("toughness",p.getToughness().getValue());}
            else {value.add("power",JsonNull.INSTANCE);value.add("toughness",JsonNull.INSTANCE);}
            permanents.add(id,value);
        }
        List<StackObject> ordered=new ArrayList<>();for(StackObject item:g.getStack())ordered.add(item);Collections.reverse(ordered);
        for(StackObject item:ordered) {
            need(item instanceof Spell && bindings.containsKey(item.getSourceId()),"/unsupported stack object");
            UUID source=item.getSourceId();witnessed.add(source);need(castActions.containsKey(source),"/unbound stack action");
            int action=castActions.get(source);Integer old=stackActions.putIfAbsent(item.getId(),action);need(old==null||old==action,"/stack action identity changed");
            JsonObject value=new JsonObject();value.addProperty("source",bindings.get(source));value.addProperty("incarnation",incarnation(source,g));value.addProperty("action",action);
            stack.add(value);rawStackIds.add(item.getId().toString(),value.deepCopy());
        }
        need(witnessed.equals(bindings.keySet()),"/missing or extra zone occurrence");
        for(Map.Entry<UUID,String> binding:bindings.entrySet()) {
            UUID id=binding.getKey();need(g.getCard(id).getOwnerId().equals(players[Integer.parseInt(binding.getValue().split("/")[0])].getId()),"/owner identity");
            incarnations.addProperty(binding.getValue(),incarnation(id,g));
        }
        point.add("battlefield",battlefield);point.add("stack",stack);point.add("incarnations",incarnations);point.add("permanents",permanents);
        if(pending==null)point.add("payment",JsonNull.INSTANCE);
        else {
            need(unpaid!=null && pendingAbility!=null && pendingAbility.getSourceId().equals(pending),"/unwitnessed payment");
            Mana remaining=unpaid.getMana();JsonObject p=new JsonObject();p.addProperty("actor",pendingActor);p.addProperty("source",bindings.get(pending));
            p.addProperty("incarnation",pendingOrigin);p.addProperty("action",pendingAction);p.add("pool",mana(g.getPlayer(players[pendingActor].getId()).getManaPool().getMana()));
            p.add("colored",mana(remaining));p.addProperty("generic",remaining.getGeneric());p.add("sources",ids(pendingSources,false));point.add("payment",p);
        }
        return point;
    }
    protected JsonObject next(int actor,Game g) {
        need(cursor<current.getAsJsonArray("play").size(),"/missing choice before named stop");
        JsonObject e=current.getAsJsonArray("play").get(cursor).getAsJsonObject();
        need(integer(e.get("sequence"),cursor),"/play sequence");need(integer(e.get("actor"),actor),"/play actor");
        need(integer(e.get("turn"),g.getTurnNum()),"/play turn");
        need(e.get("step").getAsString().equals(g.getTurnStepType().name().toLowerCase(Locale.ROOT)),"/play step");
        String kind=e.get("kind").getAsString();
        need(kind.equals("pay")||e.get("color").isJsonNull(),"/unexpected color");
        if(!Arrays.asList("play_land","cast","tap_mana").contains(kind))need(e.get("source").isJsonNull()&&e.get("incarnation").isJsonNull(),"/unexpected source");
        return e;
    }
    protected void accept(JsonObject e) {usedPlay.add(e.deepCopy());cursor++;}
    protected UUID source(JsonObject e,Game g) {
        need(e.get("source").isJsonPrimitive() && e.get("source").getAsJsonPrimitive().isString(),"/source type");
        UUID id=handles.get(e.get("source").getAsString());need(id!=null,"/source unknown");
        need(integer(e.get("incarnation"),incarnation(id,g)),"/play/"+cursor+"/incarnation expected "+e.get("incarnation")+" observed "+incarnation(id,g)+" raw "+g.getCard(id).getZoneChangeCounter(g)+" initial "+initialCounters.get(id));return id;
    }
    protected void tap(TestPlayer p,JsonObject e,Game g) {
        UUID id=source(e,g);Permanent land=g.getPermanent(id);
        need(land!=null && land.isLand(g) && land.getSuperType(g).contains(SuperType.BASIC),"/unsupported mana source");
        List<ActivatedManaAbilityImpl> abilities=land.getAbilities().getActivatedManaAbilities(Zone.BATTLEFIELD);
        need(abilities.size()==1,"/ambiguous mana ability");need(p.activateAbility(abilities.get(0),g),"/selected mana ability rejected");
        need(g.getPermanent(id).isTapped(),"/unwitnessed mana tap");
        if(pending!=null)pendingSources.add(id);
        accept(e);
    }
    protected void failCallback(Game g,IllegalArgumentException error) {
        if(priorityFailure==null)priorityFailure=new IllegalArgumentException(
            "first divergence: /play/"+cursor+" "+error.getMessage().replaceFirst("^first divergence: ",""));
        stopped=true;g.pause();
    }
    @Override protected boolean onMana(TestPlayer p,int s,Ability ability,ManaCost unpaid,Game g) {
        try {
            need(!stopped && pending!=null && ability!=null && ability.getSourceId().equals(pending) && s==pendingActor,"/unscripted mana callback");
            need(!p.getManaPool().isAutoPayment(),"/automatic mana payment");pendingAbility=ability;
            JsonObject e=next(s,g);played.add(state(g,s,"before/"+cursor,unpaid));String kind=e.get("kind").getAsString();
            if(kind.equals("tap_mana")){tap(p,e,g);return true;}
            need(kind.equals("pay"),"/unsupported payment callback");
            int color=e.get("color").getAsInt();need(integer(e.get("color"),color) && color>=0 && color<6,"/payment color");
            JsonArray pool=mana(p.getManaPool().getMana());need(pool.get(color).getAsInt()>0,"/insufficient or wrong-color payment");
            Mana remaining=unpaid.getMana();JsonArray colored=mana(remaining);int required=-1;
            for(int i=0;i<6;i++)if(colored.get(i).getAsInt()>0){required=i;break;}
            need(required<0||required==color,"/wrong-color payment");
            p.getManaPool().unlockManaType(new ManaType[]{ManaType.WHITE,ManaType.BLUE,ManaType.BLACK,ManaType.RED,ManaType.GREEN,ManaType.COLORLESS}[color]);
            accept(e);return true;
        } catch(IllegalArgumentException error){failCallback(g,error);return false;}
    }
    protected void emptyAttackers(TestPlayer p,int s,Game g,boolean forced) {
        need(g.getTurnStepType()==PhaseStep.DECLARE_ATTACKERS && p.getId().equals(g.getActivePlayerId()) && declarations.add(g.getTurnNum()),"/extra or misplaced declaration");
        JsonObject e=next(s,g);need(e.get("kind").getAsString().equals("empty_attackers"),"/missing empty declaration");
        if(forced)for(Permanent permanent:g.getBattlefield().getAllActivePermanents())if(permanent.isCreature(g)&&permanent.getControllerId().equals(p.getId()))need(!permanent.canAttack(players[1-s].getId(),g),"/unwitnessed attackers choice");
        need(g.getCombat().getGroups().isEmpty(),"/unsupported nonempty combat");played.add(state(g,s,"before/"+cursor,null));accept(e);
    }
    @Override protected void onAttackers(TestPlayer p,int s,Game g,UUID active) {
        try {need(active.equals(p.getId()),"/attackers actor");emptyAttackers(p,s,g,false);}
        catch(IllegalArgumentException error){failCallback(g,error);}
    }
    @Override protected boolean onPriority(TestPlayer p,int s,Game g) {
        try {
            need(!stopped && priorityFailure==null,"/repeated priority callback");
            if(!openingSeen){firstUpkeep(p,s,g);openingSeen=true;}
            need(p.getId().equals(g.getPriorityPlayerId()),"/priority provenance");
            if(current.get("stop").getAsString().equals("first_cast_committed") && castActions.size()==1 && !g.getStack().isEmpty() && pending==null) {
                JsonObject stop=new JsonObject();stop.addProperty("next_sequence",cursor);stop.add("checkpoint",state(g,s,"first_cast_committed",null));stops.add(stop);stopped=true;g.pause();return false;
            }
            long creatures=g.getBattlefield().getAllActivePermanents().stream().filter(x->x.isCreature(g)).count();
            if(g.getTurnNum()==6 && g.getStack().isEmpty() && pending==null && castActions.size()==4 && creatures==4) {
                need(cursor==current.getAsJsonArray("play").size(),"/extra choice after named stop");played.add(state(g,s,"second_creature_resolved",null));stopped=true;g.pause();return false;
            }
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
                default:throw new IllegalArgumentException("first divergence: /unsupported priority callback");
            }
        } catch(IllegalArgumentException error){failCallback(g,error);return false;}
    }
    protected JsonObject result() {
        JsonObject result=new JsonObject();result.add("points",played);result.add("consumed_play",usedPlay);result.add("opening",openingResult);
        result.add("stops",stops);result.add("raw_stack_ids",rawStackIds);result.add("raw_pre_shuffle_orders",rawPreShuffleOrders);result.add("identity_transitions",identityTransitions);
        result.add("selected_cast_checks",selectedCastChecks);
        JsonObject identities=new JsonObject();for(Map.Entry<UUID,String> b:bindings.entrySet()){JsonObject v=new JsonObject();v.addProperty("occurrence",b.getValue());v.addProperty("initial_counter",initialCounters.get(b.getKey()));identities.add(b.getKey().toString(),v);}result.add("raw_occurrence_bindings",identities);
        return result;
    }
    @Override protected JsonObject run(JsonObject c) throws Exception {
        try {openingResult=super.run(c);return result();}
        catch(Exception error) {
            JsonObject diagnostic=result();diagnostic.addProperty("case",c.get("id").getAsString());diagnostic.addProperty("error",error.toString());
            Files.write(Paths.get(System.getProperty("mtglab.output")+".failure.json"),JSON.toJson(diagnostic).getBytes(StandardCharsets.UTF_8));
            throw error;
        }
    }
    @Override @Test public void mulliganPrefixes() throws Exception {
        JsonObject doc=read(Paths.get(System.getProperty("mtglab.fixture")));validate(doc);
        JsonObject points=new JsonObject(),runs=new JsonObject(),repeated=new JsonObject(),continued=new JsonObject(),rejected=new JsonObject();
        for(JsonElement element:doc.getAsJsonArray("cases")) {
            JsonObject c=element.getAsJsonObject();String id=c.get("id").getAsString();JsonObject r=run(c);points.add(id,r.get("points"));runs.add(id,r);repeated.add(id,run(c));
            JsonObject bounded=c.deepCopy();bounded.addProperty("stop","first_cast_committed");run(bounded);
            need(stopped && cursor>0 && cursor<c.getAsJsonArray("play").size(),"/missing bounded stop");
            current=c;stopped=false;lastGame.resume();if(priorityFailure!=null)throw priorityFailure;need(stopped && lastGame.isPaused(),"/missing continuation stop");continued.add(id,result());
        }
        JsonObject bad=read(Paths.get(System.getProperty("mtglab.negatives")));
        JsonObject expectedRejections=read(Paths.get(System.getProperty("mtglab.root"),"fixtures/reference/full-pool-priority-negative-expectations.json"));
        for(String name:bad.keySet()) {
            try {JsonObject input=bad.getAsJsonObject(name);validate(input);for(JsonElement c:input.getAsJsonArray("cases"))run(c.getAsJsonObject());fail("accepted negative "+name);}
            catch(IllegalArgumentException e){need(e.getMessage().startsWith(expectedRejections.getAsJsonObject(name).get("xmage").getAsString()),"/intended rejection boundary "+name+": "+e.getMessage());rejected.addProperty(name,e.getMessage());}
        }
        JsonObject callbacks=new JsonObject();run(doc.getAsJsonArray("cases").get(0).getAsJsonObject());
        onMana(players[0],0,null,null,lastGame);need(priorityFailure!=null,"/unscripted mana probe accepted");callbacks.addProperty("unscripted_mana",priorityFailure.getMessage());
        try{players[0].chooseTarget(Outcome.Discard,null,null,lastGame);fail("unexpected target accepted");}catch(IllegalArgumentException e){callbacks.addProperty("unexpected_target",e.getMessage());}
        try{players[0].selectBlockers(null,lastGame,players[0].getId());fail("unexpected blockers accepted");}catch(IllegalArgumentException e){callbacks.addProperty("unexpected_blockers",e.getMessage());}
        JsonObject output=new JsonObject();output.add("checkpoints",points);output.add("runs",runs);output.add("repeat_runs",repeated);output.add("continued_runs",continued);output.add("rejections",rejected);output.add("callback_controls",callbacks);
        output.addProperty("rejection_state_rng","reference rollback/internal RNG not compared");
        Files.write(Paths.get(System.getProperty("mtglab.output")),JSON.toJson(output).getBytes(StandardCharsets.UTF_8));
    }
}
