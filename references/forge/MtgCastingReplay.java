package forge.game;

import com.google.gson.*;
import forge.game.card.*;
import forge.game.event.Event;
import forge.game.player.Player;
import forge.game.zone.ZoneType;
import forge.game.spellability.SpellAbility;
import forge.game.spellability.AbilitySub;
import forge.game.ability.ApiType;
import forge.game.ability.effects.CharmEffect;
import forge.game.cost.*;
import forge.game.mana.ManaCostBeingPaid;
import forge.card.mana.ManaCost;
import java.nio.file.*;
import java.util.*;

/** Test-only strict prefix controller. Forge alone applies the game rules. */
public final class MtgCastingReplay {
    private static MtgCastingReplay active;
    private final Game game;
    private final JsonObject tape;
    private final JsonArray rows;
    public interface CastExecutor { boolean play(Player player, SpellAbility ability); }
    public interface CostFactory { CostDecisionMakerBase create(Player player, SpellAbility ability); }
    public static CastExecutor castExecutor;
    public static CostFactory costFactory;
    private final Map<Integer,Integer> abilityIds=new HashMap<>();
    private final Map<Integer,Integer> cardIds=new HashMap<>();
    private final Map<String,Long> timestamps=new HashMap<>();
    private final Map<String,Long> reverseTimestamps=new HashMap<>();
    private final Map<Long,Long> effectTimestamps=new HashMap<>();
    private final Map<Long,Long> reverseEffectTimestamps=new HashMap<>();
    private int cursor;
    private int randoms;
    private int decisions;
    public static final class Endpoint extends RuntimeException { private Endpoint() { super("casting prefix endpoint"); } }
    private MtgCastingReplay(Game g, JsonObject t) { game=g; tape=t; rows=t.getAsJsonArray("prefix"); }
    public static void open(Game g, Path file) throws Exception {
        if (active != null) throw new IllegalStateException("replay already open");
        active=new MtgCastingReplay(g, JsonParser.parseString(Files.readString(file)).getAsJsonObject());
        active.take("start");
    }
    private void require(boolean ok, String field) {
        if (!ok) throw new IllegalArgumentException("first divergence: /forge/"+cursor+"/"+field);
    }
    private static JsonElement inspect(Game game, Object value) {
        try {
            java.lang.reflect.Constructor<MtgRawCapture> constructor=MtgRawCapture.class.getDeclaredConstructor(Game.class,Path.class,long.class);
            constructor.setAccessible(true);
            MtgRawCapture observer=constructor.newInstance(game,null,Long.MAX_VALUE);
            java.lang.reflect.Method method=MtgRawCapture.class.getDeclaredMethod("value",Object.class);
            method.setAccessible(true);
            return (JsonElement)method.invoke(observer,value);
        } catch (ReflectiveOperationException error) { throw new IllegalStateException("capture value observer unavailable",error); }
    }
    private static JsonElement semantic(JsonElement e) {
        if (e.isJsonArray()) { JsonArray a=new JsonArray(); for(JsonElement v:e.getAsJsonArray()) a.add(semantic(v)); return a; }
        if (e.isJsonObject()) {
            JsonObject o=new JsonObject();
            for(Map.Entry<String,JsonElement> v:e.getAsJsonObject().entrySet())
                if (!v.getKey().equals("instance")) {
                    JsonElement normalized=semantic(v.getValue());
                    if(v.getKey().equals("paying_mana_abilities")) {
                        // Forge keeps live SpellAbility references here. AI cost
                        // probes mutate their activating player after payment.
                        // This is not the historical payer: the enclosing spell
                        // actor, mana lots, source controller and committed cost
                        // callback are independently compared and remain intact.
                        for(JsonElement ability:normalized.getAsJsonArray())ability.getAsJsonObject().remove("actor");
                    }
                    o.add(v.getKey(),normalized);
                }
            if(o.has("token") && o.get("token").getAsBoolean() && o.has("printing")) {
                // TokenInfo pins a cosmetic PaperToken edition on the first
                // prototype query, including skipped AI simulation queries.
                // Keep token name and every rules-bearing characteristic;
                // paper edition, collector number and artwork are presentation.
                JsonObject printing=o.getAsJsonObject("printing");
                for(String key:List.of("edition","collector_number","art_index"))printing.remove(key);
            }
            return o;
        }
        return e;
    }
    private static String difference(JsonElement expected,JsonElement actual,String path) {
        if(expected.equals(actual))return null;
        if(expected.isJsonObject() && actual.isJsonObject()) {
            JsonObject e=expected.getAsJsonObject(),a=actual.getAsJsonObject();
            if(!e.keySet().equals(a.keySet()))return path+"/fields";
            for(String key:e.keySet()) { String d=difference(e.get(key),a.get(key),path+"/"+key); if(d!=null)return d; }
        }
        if(expected.isJsonArray() && actual.isJsonArray()) {
            JsonArray e=expected.getAsJsonArray(),a=actual.getAsJsonArray();
            if(e.size()!=a.size())return path+"/length";
            for(int i=0;i<e.size();i++) { String d=difference(e.get(i),a.get(i),path+"/"+i); if(d!=null)return d; }
        }
        return path+(expected.isJsonPrimitive() || actual.isJsonPrimitive() ? " expected="+expected+" actual="+actual : "");
    }
    private JsonElement observed(JsonElement value) {
        JsonElement result=semantic(value);
        bindObserved(result);
        return result;
    }
    private void bindObserved(JsonElement value) {
        if(value.isJsonArray()) { for(JsonElement e:value.getAsJsonArray())bindObserved(e); }
        if(value.isJsonObject()) {
            JsonObject o=value.getAsJsonObject();
            if(o.has("timestamp") && o.has("id")) {
                String key=o.get("id").getAsString()+"/"+o.get("timestamp").getAsString();
                if(timestamps.containsKey(key))o.addProperty("timestamp",timestamps.get(key));
                int id=o.get("id").getAsInt();
                if(cardIds.containsKey(id))o.addProperty("id",cardIds.get(id));
            }
            for(String key:List.of("source_ability","ability_id")) if(o.has(key) && !o.get(key).isJsonNull()) {
                int id=o.get(key).getAsInt(); if(abilityIds.containsKey(id))o.addProperty(key,abilityIds.get(id));
            }
            if(o.has("api") && o.has("id")) {
                int id=o.get("id").getAsInt(); if(abilityIds.containsKey(id))o.addProperty("id",abilityIds.get(id));
            }
            if(o.has("pt_boosts"))for(JsonElement cell:o.getAsJsonArray("pt_boosts")) {
                JsonArray boost=cell.getAsJsonArray();long stamp=boost.get(0).getAsLong();
                if(effectTimestamps.containsKey(stamp))boost.set(0,new JsonPrimitive(effectTimestamps.get(stamp)));
            }
            for(JsonElement e:o.asMap().values())bindObserved(e);
        }
    }
    private void bindCardTimestamp(JsonObject wanted,JsonObject actual) {
        int actualId=actual.get("id").getAsInt();
        if(wanted==null || wanted.get("id").getAsInt()!=cardIds.getOrDefault(actualId,actualId) || !wanted.get("zone").equals(actual.get("zone")))return;
        String actualKey=actual.get("id").getAsString()+"/"+actual.get("timestamp").getAsString();
        String expectedKey=wanted.get("id").getAsString()+"/"+wanted.get("timestamp").getAsString();
        Long old=timestamps.putIfAbsent(actualKey,wanted.get("timestamp").getAsLong());
        Long reverse=reverseTimestamps.putIfAbsent(expectedKey,actual.get("timestamp").getAsLong());
        require((old==null || old==wanted.get("timestamp").getAsLong()) &&
                (reverse==null || reverse==actual.get("timestamp").getAsLong()),"incarnation binding");
        JsonArray expectedBoosts=wanted.getAsJsonArray("pt_boosts"),actualBoosts=actual.getAsJsonArray("pt_boosts");
        if(expectedBoosts.size()==actualBoosts.size())for(int i=0;i<actualBoosts.size();i++) {
            long e=expectedBoosts.get(i).getAsJsonArray().get(0).getAsLong();
            long a=actualBoosts.get(i).getAsJsonArray().get(0).getAsLong();
            Long previous=effectTimestamps.putIfAbsent(a,e),back=reverseEffectTimestamps.putIfAbsent(e,a);
            require((previous==null || previous==e) && (back==null || back==a),"effect identity binding");
        }
    }
    private void bindStateCards(JsonElement expected,JsonElement actual) {
        if(expected.isJsonObject() && actual.isJsonObject()) {
            JsonObject e=expected.getAsJsonObject(),a=actual.getAsJsonObject();
            if(e.has("timestamp") && a.has("timestamp") && e.has("id") && a.has("id"))bindCardTimestamp(e,a);
            if(e.has("api") && a.has("api") && e.get("api").getAsString().equals("Charm") && e.get("api").equals(a.get("api")) &&
                    !e.get("sub").isJsonNull() && !a.get("sub").isJsonNull()) {
                int root=a.get("id").getAsInt();
                require(abilityIds.getOrDefault(root,root)==e.get("id").getAsInt(),"modal root identity");
                JsonObject es=e.getAsJsonObject("sub"),as=a.getAsJsonObject("sub");
                require(es.get("api").equals(as.get("api")),"mode clone selection");
                Integer previous=abilityIds.putIfAbsent(as.get("id").getAsInt(),es.get("id").getAsInt());
                require(previous==null || previous==es.get("id").getAsInt(),"changed modal clone identity");
            }
            for(String key:e.keySet())if(a.has(key))bindStateCards(e.get(key),a.get(key));
        } else if(expected.isJsonArray() && actual.isJsonArray() && expected.getAsJsonArray().size()==actual.getAsJsonArray().size()) {
            for(int i=0;i<expected.getAsJsonArray().size();i++)bindStateCards(expected.getAsJsonArray().get(i),actual.getAsJsonArray().get(i));
        }
    }
    private void state(JsonObject row) {
        JsonObject raw=MtgRawCapture.audit(game);
        Map<Integer,JsonObject> expectedCards=new HashMap<>();
        for(JsonElement p:row.getAsJsonObject("state").getAsJsonArray("players"))
            for(JsonElement zone:p.getAsJsonObject().getAsJsonObject("zones").asMap().values())
                for(JsonElement c:zone.getAsJsonArray())expectedCards.put(c.getAsJsonObject().get("id").getAsInt(),c.getAsJsonObject());
        List<JsonObject> expectedBorn=new ArrayList<>(),actualBorn=new ArrayList<>();
        for(JsonObject c:expectedCards.values())
            if(c.get("token").getAsBoolean() && !cardIds.containsValue(c.get("id").getAsInt()))expectedBorn.add(c);
        for(JsonElement p:raw.getAsJsonArray("players"))
            for(JsonElement zone:p.getAsJsonObject().getAsJsonObject("zones").asMap().values())
                for(JsonElement value:zone.getAsJsonArray()) {
                    JsonObject c=value.getAsJsonObject();
                    if(c.get("token").getAsBoolean() && !cardIds.containsKey(c.get("id").getAsInt()))actualBorn.add(c);
                }
        require(expectedBorn.size()==actualBorn.size(),"token birth count");
        Comparator<JsonObject> birthOrder=Comparator.comparingInt(c->c.get("id").getAsInt());
        expectedBorn.sort(birthOrder);actualBorn.sort(birthOrder);
        for(int i=0;i<expectedBorn.size();i++) {
            JsonObject e=expectedBorn.get(i),a=actualBorn.get(i);
            require(e.get("name").getAsString().equals("Goblin Token") && e.get("name").equals(a.get("name")) &&
                    e.get("owner").equals(a.get("owner")) && e.get("controller").equals(a.get("controller")) &&
                    e.get("zone").getAsString().equals("Battlefield") && e.get("zone").equals(a.get("zone")),"unsupported token birth");
            cardIds.put(a.get("id").getAsInt(),e.get("id").getAsInt());
        }
        // During a zone-change event the host can already have its new
        // timestamp while temporarily absent from every zone collection.
        // Stack and pending event observations retain both current and
        // last-known objects. Bind every state observation bijectively;
        // selected choice results are never used to create these bindings.
        bindStateCards(row.get("state"),raw);
        JsonElement expected=semantic(row.get("state")), actual=observed(raw);
        String difference=difference(expected,actual,"");
        require(difference==null,"state"+difference);
    }
    private JsonObject take(String kind) {
        require(cursor<rows.size(),"unrecorded "+kind);
        JsonObject row=rows.get(cursor).getAsJsonObject();
        require(row.get("index").getAsInt()==cursor && row.get("kind").getAsString().equals(kind),"record kind "+kind);
        if (!kind.equals("random")) state(row);
        cursor++;
        return row;
    }
    public static void event(Game g, Event e) {
        if(active==null)return;
        JsonObject row=active.take("event");
        active.require(g==active.game && row.get("event").getAsString().equals(e.getClass().getSimpleName()),"event");
    }
    public static void boundary(Game g,String name) {
        if(active==null)return;
        JsonObject row=active.take("boundary");
        active.require(g==active.game && row.get("boundary").getAsString().equals(name),"boundary");
    }
    public static void shuffle(Player p,String stage) {
        if(active==null)return;
        JsonObject row=active.take("shuffle");
        active.require(row.get("stage").getAsString().equals(stage) && row.getAsJsonObject("actor").get("player_id").getAsInt()==p.getId(),"shuffle actor/stage");
        active.require(semantic(row.get("library")).equals(semantic(inspect(p.getGame(),p.getCardsIn(ZoneType.Library)))),"shuffle order");
    }
    public static final class RecordedRandom extends Random {
        public RecordedRandom() { super(0); }
        @Override protected int next(int bits) {
            if(active==null) throw new IllegalStateException("random outside casting replay");
            JsonObject r=active.take("random");
            active.require(r.get("bits").getAsInt()==bits && r.get("draw").getAsInt()==++active.randoms,"random bits/order");
            return r.get("result").getAsInt();
        }
    }
    @SuppressWarnings("unchecked")
    public static <T> T choose(Player p,String method,Object[] args) {
        if(active==null)throw new IllegalStateException("choice outside casting replay: "+method);
        MtgCastingReplay a=active;
        if(a.cursor==a.rows.size()) {
            JsonObject endpoint=a.tape.getAsJsonObject("endpoint");
            a.require(method.equals(endpoint.get("method").getAsString()) &&
                endpoint.getAsJsonObject("actor").get("player_id").getAsInt()==p.getId(),"unconsumed casting choice/endpoint");
            a.state(endpoint);
            throw new Endpoint();
        }
        JsonObject entry=a.take("enter");
        a.require(entry.get("method").getAsString().equals(method) && entry.getAsJsonObject("actor").get("player_id").getAsInt()==p.getId(),"callback actor/method");
        // CharmEffect clones the selected mode while announcing the spell.
        // Bind that clone's allocation ID; mode/effect contents stay compared.
        JsonArray expectedArgs=entry.getAsJsonArray("arguments");
        for(int i=0;i<args.length;i++)if(args[i] instanceof SpellAbility sa && sa.getApi()==ApiType.Charm && sa.getSubAbility()!=null) {
            JsonObject expected=expectedArgs.get(i).getAsJsonObject();
            a.require(!expected.get("sub").isJsonNull(),"unexpected modal subability");
            JsonObject sub=expected.getAsJsonObject("sub");
            a.require(String.valueOf(sa.getSubAbility().getApi()).equals(sub.get("api").getAsString()),"mode clone selection");
            Integer old=a.abilityIds.putIfAbsent(sa.getSubAbility().getId(),sub.get("id").getAsInt());
            a.require(old==null || old==sub.get("id").getAsInt(),"changed modal clone identity");
        }
        a.require(semantic(entry.get("arguments")).equals(a.observed(inspect(p.getGame(),Arrays.asList(args)))),"callback arguments");
        int end=a.tape.getAsJsonObject("paired").get(entry.get("index").getAsString()).getAsInt();
        if(method.equals("playChosenSpellAbility") || method.equals("payManaCost")) {
            Object answer;
            if(method.equals("playChosenSpellAbility")) {
                SpellAbility sa=(SpellAbility)args[0];
                a.require(sa.canPlay(),"illegal announced ability");
                if(sa.isLandAbility()) { sa.resolve(); answer=true; }
                else { a.require(sa.isSpell(),"unsupported nonmana activation"); answer=castExecutor.play(p,sa); }
            } else { answer=a.pay(p,args,end); }
            a.require(a.cursor==end,"unconsumed executable callback");
            JsonObject leave=a.take("leave");
            a.require(leave.get("call").equals(entry.get("call")),"callback return");
            a.require(semantic(leave.get("result")).equals(a.observed(inspect(a.game,answer))),"execution result");
            a.require(semantic(leave.get("arguments")).equals(a.observed(inspect(a.game,Arrays.asList(args)))),"execution arguments");
            a.decisions++; return (T)answer;
        }
        // Internal AI deliberation is retained and accounted for, never rerun.
        // The source parser pairs it; no rules transition may be hidden here.
        while(a.cursor<end) {
            JsonObject inner=a.rows.get(a.cursor).getAsJsonObject();
            String kind=inner.get("kind").getAsString();
            a.require(Set.of("enter","leave","random").contains(kind),"rules event inside AI choice");
            inner=a.take(kind);
            if(kind.equals("random")) a.require(inner.get("draw").getAsInt()==++a.randoms,"internal random order");
        }
        JsonObject leave=a.take("leave");
        a.require(leave.get("call").equals(entry.get("call")),"callback return");
        JsonElement result=leave.get("result");
        a.decisions++;
        Object answer;
        switch(method) {
            case "chooseModeForAbility":
                a.require(((Number)args[2]).intValue()==1 && ((Number)args[3]).intValue()==1 && Boolean.FALSE.equals(args[4]) &&
                          result.isJsonArray() && result.getAsJsonArray().size()==1,"unsupported mode cardinality");
                JsonObject wanted=result.getAsJsonArray().get(0).getAsJsonObject();
                List<AbilitySub> modes=new ArrayList<>();
                for(AbilitySub candidate:(List<AbilitySub>)args[1])
                    if(String.valueOf(candidate.getApi()).equals(wanted.get("api").getAsString()))modes.add(candidate);
                a.require(modes.size()==1,"ambiguous/illegal mode");
                a.configure(p,modes.get(0),wanted,true);
                a.require(semantic(wanted).equals(a.observed(inspect(a.game,modes.get(0)))),"mode selection");
                answer=modes;break;
            case "chooseSpellAbilityToPlay":
                if(result.isJsonNull()) { answer=null; break; }
                List<SpellAbility> selected=new ArrayList<>();
                for(JsonElement e:result.getAsJsonArray())selected.add(a.ability(p,e.getAsJsonObject(),true));
                answer=selected; break;
            case "cost.paysRightAfterDecision": answer=result.getAsBoolean(); break;
            case "cost.visit":
                a.require(args[0] instanceof CostPartMana || args[0] instanceof CostTap || args[0] instanceof CostDiscard,"unsupported cost part");
                if(result.isJsonNull()) { answer=null; break; }
                JsonObject pd=result.getAsJsonObject();
                for(String key:List.of("type","mana","players","abilities","colors","counter_table","matrix"))
                    a.require(pd.get(key).isJsonNull(),"unsupported payment decision "+key);
                PaymentDecision decision=new PaymentDecision(pd.get("amount").getAsInt());
                for(JsonElement e:pd.getAsJsonArray("cards"))decision.cards.add(a.card(e.getAsJsonObject()));
                answer=decision; break;
            case "declareAttackers":
                a.require(result.isJsonNull() && leave.getAsJsonObject("state").getAsJsonArray("combat").isEmpty(),"unsupported nonempty combat");
                answer=null; break;
            case "resetAtEndOfTurn": case "autoPassCancel":
                a.require(result.isJsonNull(),"void gameplay result"); answer=null; break;
            case "mulliganKeepHand": answer=result.getAsBoolean(); break;
            case "chooseStartingPlayer":
                answer=p.getGame().getPlayers().stream().filter(q->q.getId()==result.getAsJsonObject().get("player_id").getAsInt()).findFirst().orElseThrow(); break;
            case "cheatShuffle": case "tuckCardsViaMulligan":
                CardCollection cards=new CardCollection(); Set<Integer> ids=new HashSet<>();
                CardCollectionView available=(CardCollectionView)args[0];
                for(JsonElement e:result.getAsJsonArray()) {
                    JsonObject c=e.getAsJsonObject(); int id=c.get("id").getAsInt();
                    a.require(ids.add(id),"duplicate occurrence");
                    Card card=null;
                    for(Card candidate:available) if(candidate.getId()==id) card=candidate;
                    a.require(card!=null && card.getGameTimestamp()==c.get("timestamp").getAsLong(),"stale occurrence/incarnation");
                    cards.add(card);
                }
                a.require(method.equals("cheatShuffle") ? cards.size()==available.size() : cards.size()==((Number)args[1]).intValue(),"choice cardinality");
                answer=cards; break;
            case "complainCardsCantPlayWell": a.require(result.isJsonArray() && result.getAsJsonArray().isEmpty(),"unsupported complaint"); answer=Collections.emptyMap(); break;
            case "awaitNextInput": case "revealUnsupported": a.require(result.isJsonNull(),"void result"); answer=null; break;
            default: throw new IllegalArgumentException("unsupported casting choice: "+method);
        }
        return (T)answer;
    }
    private Card card(JsonObject expected) {
        int id=expected.get("id").getAsInt();
        Card found=null;
        for(Card c:game.getCardsInGame())if(cardIds.getOrDefault(c.getId(),c.getId())==id) { require(found==null,"ambiguous physical card"); found=c; }
        require(found!=null && timestamps.getOrDefault(found.getId()+"/"+found.getGameTimestamp(),Long.MIN_VALUE)==expected.get("timestamp").getAsLong(),"stale occurrence/incarnation");
        require(found.getName().equals(expected.get("name").getAsString()),"card definition");
        return found;
    }
    private SpellAbility ability(Player player,JsonObject expected,boolean choices) {
        Card host=card(expected.getAsJsonObject("host"));
        List<SpellAbility> candidates=new ArrayList<>();
        for(SpellAbility sa:host.getSpellAbilities()) {
            if(String.valueOf(sa.getApi()).equals(expected.get("api").getAsString()) &&
               sa.isSpell()==expected.get("spell").getAsBoolean() && sa.isManaAbility()==expected.get("mana_ability").getAsBoolean())candidates.add(sa);
        }
        require(candidates.size()==1,"ambiguous/unsupported ability");
        SpellAbility sa=candidates.get(0);
        configure(player,sa,expected,choices);
        require(semantic(expected).equals(observed(inspect(game,sa))),"ability selection");
        return sa;
    }
    private void configure(Player player,SpellAbility sa,JsonObject expected,boolean choices) {
        sa.setActivatingPlayer(player);
        Integer previous=abilityIds.putIfAbsent(sa.getId(),expected.get("id").getAsInt());
        require(previous==null || previous==expected.get("id").getAsInt(),"changed ability identity");
        if(choices) {
            require(expected.getAsJsonArray("optional_costs").isEmpty(),"unsupported optional costs");
            sa.resetTargets();
            for(JsonElement e:expected.getAsJsonArray("targets")) {
                JsonObject target=e.getAsJsonObject(); require(target.get("allocation").isJsonNull(),"unsupported divided target");
                JsonObject ref=target.getAsJsonObject("target");
                if(ref.has("player_id")) {
                    Player p=game.getPlayers().stream().filter(q->q.getId()==ref.get("player_id").getAsInt()).findFirst().orElseThrow();
                    require(sa.canTarget(p),"illegal player target");sa.getTargets().add(p);
                } else { Card c=card(ref);require(sa.canTarget(c),"illegal card target");sa.getTargets().add(c); }
            }
        }
        if(!expected.get("sub").isJsonNull()) {
            if(sa.getApi()==ApiType.Charm && sa.getSubAbility()==null && choices) {
                JsonObject wanted=expected.getAsJsonObject("sub");
                List<AbilitySub> modes=new ArrayList<>();
                for(AbilitySub candidate:CharmEffect.makePossibleOptions(sa))
                    if(String.valueOf(candidate.getApi()).equals(wanted.get("api").getAsString()))modes.add(candidate);
                require(modes.size()==1,"illegal announced mode");
                CharmEffect.chainAbilities(sa,modes);
            }
            require(sa.getSubAbility()!=null,"missing subability");
            configure(player,sa.getSubAbility(),expected.getAsJsonObject("sub"),choices);
        } else require(sa.getSubAbility()==null,"extra subability");
    }
    private boolean pay(Player player,Object[] args,int end) {
        require(args[0] instanceof ManaCost && args[2] instanceof SpellAbility,"unsupported mana callback");
        require(args[4]==null,"unsupported mana conversion");
        SpellAbility spell=(SpellAbility)args[2];
        ManaCostBeingPaid cost=new ManaCostBeingPaid((ManaCost)args[0]);
        player.getManaPool().restoreColorReplacements();
        player.getManaPool().payManaCostFromPool(cost,spell,false,spell.getPayingMana());
        while(!cost.isPaid()) {
            // AI source selection randomness is retained but never used to choose.
            while(cursor<end && rows.get(cursor).getAsJsonObject().get("kind").getAsString().equals("random")) {
                JsonObject r=take("random");require(r.get("draw").getAsInt()==++randoms,"payment random order");
            }
            require(cursor<end,"missing mana payment");
            JsonObject next=rows.get(cursor).getAsJsonObject();
            require(next.get("kind").getAsString().equals("enter") && next.get("method").getAsString().equals("cost.visit"),"ordered mana source");
            SpellAbility mana=ability(player,next.getAsJsonArray("arguments").get(1).getAsJsonObject(),false);
            require(mana.isManaAbility() && mana.canPlay(),"illegal mana source");
            require(new CostPayment(mana.getPayCosts(),mana).payComputerCosts(costFactory.create(player,mana)),"mana source costs");
            game.getStack().addAndUnfreeze(mana);
            player.getManaPool().payManaFromAbility(spell,cost,mana);
        }
        return true;
    }
    public static void finish(Path output) throws Exception {
        MtgCastingReplay a=active;
        a.require(a.cursor==a.rows.size() && !a.game.isGameOver(),"prefix consumption/terminal");
        JsonObject result=new JsonObject();
        result.addProperty("status","casting-replayed"); result.addProperty("complete_game",false);
        result.addProperty("consumed_records",a.cursor); result.addProperty("consumed_random",a.randoms);
        result.addProperty("controller_choices",a.decisions); result.add("endpoint",a.tape.getAsJsonObject("endpoint").get("index"));
        result.add("remainder",a.tape.get("remainder")); result.add("state",MtgRawCapture.audit(a.game));
        Files.writeString(output,result.toString(),StandardOpenOption.CREATE_NEW); active=null;
    }
}
