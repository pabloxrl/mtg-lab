package org.mage.test.mtglab;

import com.google.gson.*;
import mage.abilities.Ability;
import mage.cards.Card;
import mage.cards.CardSetInfo;
import mage.cards.decks.Deck;
import mage.constants.*;
import mage.game.*;
import mage.game.mulligan.LondonMulligan;
import mage.target.Target;
import org.junit.Test;
import org.mage.test.player.TestComputerPlayer;
import org.mage.test.player.TestPlayer;
import java.nio.file.*;
import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.util.*;
import static org.junit.Assert.*;

/** Original CR 103 test client: real normal initialization, no injected hands. */
public class FullPoolMulliganTest {
    private static final Gson JSON = new GsonBuilder().serializeNulls().setPrettyPrinting().create();
    private final Path root = Paths.get(System.getProperty("mtglab.root"));
    private final JsonObject manifest = read(root.resolve("data/cards/foundations_micro_v1.json"));
    private final Map<UUID,String> bindings = new LinkedHashMap<>();
    private final Map<String,UUID> handles = new LinkedHashMap<>();
    private final TestPlayer[] players = new TestPlayer[2];
    private JsonArray consumed;
    private JsonArray rawPreShuffleOrders;
    private final JsonObject rawPreShuffleEvidence = new JsonObject();
    private JsonArray observed;
    private JsonArray consumedChoices;
    private JsonArray rawCallbacks;
    private int[] rounds;
    private int bottomIndex;
    private JsonObject current;
    private Game lastGame;
    private boolean stopped;
    private IllegalArgumentException priorityFailure;
    private static JsonObject read(Path p) {
        try { return JsonParser.parseString(new String(Files.readAllBytes(p),StandardCharsets.UTF_8)).getAsJsonObject(); }
        catch(Exception e) { throw new IllegalArgumentException(e); }
    }
    private static void need(boolean ok, String path) {
        if(!ok) throw new IllegalArgumentException("first divergence: "+path);
    }
    private static void keys(JsonObject o, String... names) {
        need(o.keySet().equals(new HashSet<>(Arrays.asList(names))),"/fields");
    }
    private static boolean integer(JsonElement e, int value) {
        return e.isJsonPrimitive() && e.getAsJsonPrimitive().isNumber() && e.toString().equals(Integer.toString(value));
    }
    private String sha(Path p) {
        try {
            byte[] hash=MessageDigest.getInstance("SHA-256").digest(Files.readAllBytes(p));
            StringBuilder s=new StringBuilder(); for(byte b:hash) s.append(String.format("%02x",b & 255)); return s.toString();
        } catch(Exception e) {throw new IllegalArgumentException(e);}
    }
    private JsonArray canonical(String deck, int seat) {
        JsonArray result=new JsonArray();
        for(JsonElement d:manifest.getAsJsonArray("decks")) if(d.getAsJsonObject().get("id").getAsString().equals(deck)) {
            for(JsonElement item:d.getAsJsonObject().getAsJsonArray("cards")) {
                JsonObject c=item.getAsJsonObject();
                for(int i=0;i<c.get("copies").getAsInt();i++) result.add(seat+"/"+c.get("card_id").getAsString()+"/"+i);
            }
        }
        need(result.size()==40,"/deck"); return result;
    }
    private static List<String> strings(JsonArray a) {
        List<String> out=new ArrayList<>(); for(JsonElement e:a) {need(e.isJsonPrimitive() && e.getAsJsonPrimitive().isString(),"/occurrence type");out.add(e.getAsString());}return out;
    }
    private static List<String> sorted(JsonArray a) {List<String> out=strings(a);Collections.sort(out);return out;}
    private void validate(JsonObject doc) {
        keys(doc,"schema_version","family","pins","cases");
        need(integer(doc.get("schema_version"),2),"/schema_version");
        need(doc.get("family").getAsString().equals("mulligan"),"/family");
        JsonObject pins=doc.getAsJsonObject("pins");
        keys(pins,"data/cards/foundations_micro_v1.json","data/rules/cr-2026-09-25.json","references/xmage/pins.json");
        for(String p:pins.keySet()) need(pins.get(p).getAsString().equals(sha(root.resolve(p))),"/pins");
        need(doc.getAsJsonArray("cases").size()>0,"/cases empty");
        Set<String> ids=new HashSet<>();
        for(JsonElement element:doc.getAsJsonArray("cases")) {
            JsonObject c=element.getAsJsonObject();keys(c,"id","starter","decks","chance","choices","stop");
            need(c.get("id").isJsonPrimitive() && c.get("id").getAsJsonPrimitive().isString() && !c.get("id").getAsString().isEmpty() && ids.add(c.get("id").getAsString()),"/id");
            need(integer(c.get("starter"),0)||integer(c.get("starter"),1),"/starter");
            need(c.get("stop").getAsString().equals("first_upkeep"),"/stop unsupported callback");
            need(c.getAsJsonArray("choices").size()>0,"/choices missing");
            need(c.getAsJsonArray("decks").size()==2,"/decks");
            need(c.getAsJsonArray("chance").size()>=2,"/chance length");
            for(int seat=0;seat<2;seat++) {
                JsonObject d=c.getAsJsonArray("decks").get(seat).getAsJsonObject();keys(d,"deck","occurrences");
                JsonArray expected=canonical(d.get("deck").getAsString(),seat);
                need(expected.equals(d.get("occurrences")),"/deck occurrences");
                JsonObject event=c.getAsJsonArray("chance").get(seat).getAsJsonObject();keys(event,"sequence","actor","kind","source","before","after");need(event.get("source").isJsonNull(),"/chance source");
                need(integer(event.get("sequence"),seat),"/chance sequence");
                need(integer(event.get("actor"),seat),"/chance actor");
                need(event.get("kind").getAsString().equals("initial_shuffle"),"/chance kind");
                need(event.get("before").equals(expected),"/chance before");
                need(sorted(event.getAsJsonArray("after")).equals(sorted(expected)),"/chance permutation");
            }
        }
    }
    private int seat(UUID id) {
        for(int i=0;i<2;i++) if(players[i].getId().equals(id)) return i;
        throw new IllegalArgumentException("first divergence: unknown player");
    }
    private JsonArray ids(Iterable<UUID> values, boolean sort) {
        List<String> result=new ArrayList<>();for(UUID id:values) {need(bindings.containsKey(id),"/unknown engine occurrence");result.add(bindings.get(id));}
        if(sort) Collections.sort(result);return JSON.toJsonTree(result).getAsJsonArray();
    }
    private JsonObject point(Game game, int s, String boundary) {
        need(!game.hasEnded(),"/unexpected terminal game");
        need(game.getCards().size()==bindings.size(),"/unbound engine cards");
        JsonObject point=new JsonObject(); point.addProperty("boundary",boundary);point.addProperty("actor",s);
        JsonArray life=new JsonArray();for(TestPlayer p:players)life.add(game.getPlayer(p.getId()).getLife());
        point.add("life",life);point.add("hand",ids(game.getPlayer(players[s].getId()).getHand(),true));
        point.add("library",ids(game.getPlayer(players[s].getId()).getLibrary().getCardList(),false));
        point.addProperty("mulligans",rounds[s]);return point;
    }
    private void raw(Game game,int s,String kind) {
        JsonObject row=new JsonObject();row.addProperty("kind",kind);row.addProperty("actor",s);
        JsonArray state=new JsonArray();for(int i=0;i<2;i++)state.add(point(game,i,kind));
        row.add("state",state);rawCallbacks.add(row);
    }
    private JsonObject choice(int s,String kind) {
        need(consumedChoices.size()<current.getAsJsonArray("choices").size(),"/missing choice or unexpected callback");
        JsonObject e=current.getAsJsonArray("choices").get(consumedChoices.size()).getAsJsonObject();
        keys(e,"sequence","kind","actor","source","round","selection");
        need(integer(e.get("sequence"),consumedChoices.size()),"/choice sequence");
        need(integer(e.get("actor"),s),"/choice actor");
        need(e.get("kind").getAsString().equals(kind),"/choice kind");
        need(e.get("source").isJsonNull(),"/choice source");
        need(integer(e.get("round"),rounds[s]),"/stale choice round");return e;
    }
    private TestPlayer player(final int s) {
        TestPlayer p=new TestPlayer(new TestComputerPlayer("Seat"+s,RangeOfInfluence.ONE)) {
            @Override public void shuffleLibrary(Ability source,Game game) {
                need(source==null && consumed.size()<current.getAsJsonArray("chance").size(),"/unsupported or missing shuffle callback");
                JsonObject event=current.getAsJsonArray("chance").get(consumed.size()).getAsJsonObject();
                keys(event,"sequence","actor","kind","source","before","after");
                String kind=consumed.size()<2?"initial_shuffle":"mulligan_shuffle";
                need(integer(event.get("sequence"),consumed.size()),"/chance sequence");
                need(integer(event.get("actor"),s),"/runtime chance actor");
                need(event.get("kind").getAsString().equals(kind) && event.get("source").isJsonNull(),"/chance kind/source");
                JsonArray before=ids(getLibrary().getCardList(),false);
                need(sorted(before).equals(sorted(event.getAsJsonArray("before"))),"/runtime chance before");
                JsonArray canonical=canonical(current.getAsJsonArray("decks").get(s).getAsJsonObject().get("deck").getAsString(),s);
                need(canonical.equals(event.get("before")),"/chance canonical multiset");
                need(sorted(before).equals(sorted(event.getAsJsonArray("after"))),"/chance permutation");
                rawPreShuffleOrders.add(before.deepCopy());
                List<String> creationOrder=new ArrayList<>(handles.keySet());List<String>witnessed=strings(before);
                witnessed.sort(Comparator.comparingInt(creationOrder::indexOf));before=JSON.toJsonTree(witnessed).getAsJsonArray();
                List<String> after=strings(event.getAsJsonArray("after"));
                for(int i=after.size()-1;i>=0;i--)getLibrary().putOnTop(game.getCard(handles.get(after.get(i))),game);
                JsonObject used=new JsonObject();used.addProperty("sequence",consumed.size());used.addProperty("kind",kind);used.addProperty("actor",s);used.add("source",JsonNull.INSTANCE);used.add("before",before);used.add("after",ids(getLibrary().getCardList(),false));consumed.add(used);
                if(kind.equals("mulligan_shuffle"))rounds[s]++;
                raw(game,s,kind);
            }
            @Override public boolean chooseMulligan(Game game) {
                need(bottomIndex==0 && getId().equals(game.getState().getChoosingPlayerId()),"/declaration boundary");
                JsonObject e=choice(s,"declare");
                need(e.get("selection").isJsonPrimitive() && e.get("selection").getAsJsonPrimitive().isString(),"/declaration type");
                String selected=e.get("selection").getAsString();need(selected.equals("keep")||selected.equals("mulligan"),"/declaration selection");
                observed.add(point(game,s,"declaration"));raw(game,s,"declare");consumedChoices.add(e.deepCopy());return selected.equals("mulligan");
            }
            @Override public boolean chooseTarget(Outcome outcome,Target target,Ability source,Game game) {
                need(source==null && outcome==Outcome.Discard && target!=null && target instanceof mage.target.common.TargetCardInHand && target.getMinNumberOfTargets()==1 && target.getMaxNumberOfTargets()==1,"/unsupported target callback");
                JsonObject e=choice(s,"bottom");JsonArray selected=e.getAsJsonArray("selection");
                need(selected.size()==rounds[s] && rounds[s]>0,"/bottom cardinality");
                List<String> names=strings(selected);need(new HashSet<>(names).size()==names.size(),"/duplicate bottom");
                if(bottomIndex==0) {
                    need(getHand().size()==7,"/redraw seven");
                    for(String id:names)need(handles.containsKey(id) && getHand().contains(handles.get(id)),"/stale bottom occurrence");
                    observed.add(point(game,s,"bottom"));
                }
                UUID chosen=handles.get(names.get(bottomIndex));need(chosen!=null && getHand().contains(chosen),"/stale bottom occurrence");
                need(target.canTarget(getId(),chosen,source,game),"/illegal bottom target");
                target.addTarget(chosen,source,game);need(target.getTargets().size()==1 && target.getTargets().contains(chosen),"/selected target rejected");
                raw(game,s,"bottom_card");bottomIndex++;
                if(bottomIndex==selected.size()){consumedChoices.add(e.deepCopy());bottomIndex=0;}return true;
            }
            @Override public boolean priority(Game game) {
                try {
                need(!stopped,"/repeated priority callback");
                need(bottomIndex==0 && game.getTurnNum()==1 && game.getTurnStepType()==PhaseStep.UPKEEP && getId().equals(game.getActivePlayerId()) && getId().equals(game.getStartingPlayerId()),"/unexpected priority callback");
                need(consumed.size()==current.getAsJsonArray("chance").size() && consumedChoices.size()==current.getAsJsonArray("choices").size(),"/incomplete ledger");
                int starter=seat(game.getStartingPlayerId());observed.add(point(game,starter,"first_upkeep"));observed.add(point(game,1-starter,"first_upkeep"));raw(game,s,"first_upkeep");
                } catch(IllegalArgumentException error) { priorityFailure=error; }
                stopped=true;game.pause();return false;
            }
        };
        p.setChooseStrictMode(true);return p;
    }
    private Card create(String key,UUID owner) throws Exception {
        JsonObject metadata=null;for(JsonElement e:manifest.getAsJsonArray("cards")) if(e.getAsJsonObject().get("id").getAsString().equals(key)) metadata=e.getAsJsonObject();
        need(metadata!=null && metadata.get("kind").getAsString().equals("card"),"/unknown card");
        JsonObject print=metadata.getAsJsonObject("printing");
        need(print.get("set").getAsString().equals("fdn"),"/unsupported printing set");
        mage.cards.ExpansionSet.SetCardInfo info=null;
        for(mage.cards.ExpansionSet.SetCardInfo candidate:mage.sets.Foundations.getInstance().getSetCardInfo()) {
            if(candidate.getCardNumber().equals(print.get("collector_number").getAsString()) && candidate.getName().equals(metadata.get("name").getAsString())) {need(info==null,"/ambiguous pinned printing");info=candidate;}
        }
        need(info!=null,"/unknown pinned printing");
        Card card=(Card)info.getCardClass().getConstructor(UUID.class,CardSetInfo.class).newInstance(owner,new CardSetInfo(info.getName(),"FDN",info.getCardNumber(),info.getRarity(),info.getGraphicInfo()));
        need(card.getName().equals(metadata.get("name").getAsString()),"/card class/name");return card;
    }
    private JsonObject run(JsonObject c) throws Exception {
        current=c;stopped=false;priorityFailure=null;bindings.clear();handles.clear();consumed=new JsonArray();consumedChoices=new JsonArray();rawCallbacks=new JsonArray();rounds=new int[2];bottomIndex=0;rawPreShuffleOrders=new JsonArray();observed=new JsonArray();
        Game game=new TwoPlayerDuel(MultiplayerAttackOption.LEFT,RangeOfInfluence.ONE,new LondonMulligan(0),40,20,7);
        lastGame=game;
        for(int s=0;s<2;s++) {
            players[s]=player(s);players[s].init(game);Deck deck=new Deck();
            for(JsonElement e:c.getAsJsonArray("decks").get(s).getAsJsonObject().getAsJsonArray("occurrences")) {
                String occurrence=e.getAsString();Card card=create(occurrence.split("/")[1],players[s].getId());
                // Bind at creation, before useDeck or any permutation.
                need(bindings.put(card.getId(),occurrence)==null && handles.put(occurrence,card.getId())==null,"/duplicate creation");deck.getCards().add(card);
            }
            game.loadCards(deck.getCards(),players[s].getId());game.addPlayer(players[s],deck);
        }
        GameOptions options=new GameOptions();need(!options.testMode && !options.skipInitShuffling,"/normal options");game.setGameOptions(options);
        game.setStartingPlayerId(players[c.get("starter").getAsInt()].getId());
        game.start(players[c.get("starter").getAsInt()].getId());
        if(priorityFailure!=null)throw priorityFailure;
        need(stopped && game.isPaused() && observed.size()>0,"/missing first upkeep stop");
        JsonObject result=new JsonObject();result.add("points",observed);result.add("consumed_chance",consumed);result.add("consumed_choices",consumedChoices);result.add("raw_callbacks",rawCallbacks);result.addProperty("reference_rng","unobservable; explicit shuffle callbacks witnessed, internal RNG not compared");return result;
    }
    @Test public void mulliganPrefixes() throws Exception {
        JsonObject doc=read(Paths.get(System.getProperty("mtglab.fixture")));validate(doc);
        JsonObject points=new JsonObject(),runs=new JsonObject(),repeated=new JsonObject(),rejected=new JsonObject();
        for(JsonElement c:doc.getAsJsonArray("cases")) {
            String id=c.getAsJsonObject().get("id").getAsString();JsonObject result=run(c.getAsJsonObject());
            points.add(id,result.get("points"));runs.add(id,result);rawPreShuffleEvidence.add(id,rawPreShuffleOrders);
            JsonObject again=run(c.getAsJsonObject());
            // Unordered pre-shuffle iteration is retained separately, not claimed equal.
            repeated.add(id,again);rawPreShuffleEvidence.add(id+"/repeat",rawPreShuffleOrders);
        }
        JsonObject bad=read(Paths.get(System.getProperty("mtglab.negatives")));
        for(String name:bad.keySet()) {
            try {JsonObject input=bad.getAsJsonObject(name);validate(input);for(JsonElement c:input.getAsJsonArray("cases"))run(c.getAsJsonObject());fail("accepted negative "+name);}
            catch(IllegalArgumentException e) {need(e.getMessage().startsWith("first divergence:"),"/negative diagnostic");rejected.addProperty(name,e.getMessage());}
        }
        // Explicit callback probes supplement, never replace, the played prefixes.
        JsonObject callbacks=new JsonObject();
        run(doc.getAsJsonArray("cases").get(0).getAsJsonObject());
        try {players[0].chooseMulligan(lastGame);fail("accepted extra declaration");}catch(IllegalArgumentException e){callbacks.addProperty("extra_declaration",e.getMessage());}
        try {players[0].chooseTarget(Outcome.Discard,null,null,lastGame);fail("accepted unexpected target");}catch(IllegalArgumentException e){callbacks.addProperty("unexpected_target",e.getMessage());}
        try {players[0].shuffleLibrary(null,lastGame);fail("accepted extra shuffle");}catch(IllegalArgumentException e){callbacks.addProperty("extra_shuffle",e.getMessage());}
        JsonObject result=new JsonObject();result.add("raw_pre_shuffle_orders",rawPreShuffleEvidence);result.add("callback_controls",callbacks);result.add("checkpoints",points);result.add("runs",runs);result.add("repeat_runs",repeated);result.add("rejections",rejected);
        result.addProperty("rejection_state_rng","reference stops at first divergence; rollback and internal RNG not observable");
        Files.write(Paths.get(System.getProperty("mtglab.output")),JSON.toJson(result).getBytes(StandardCharsets.UTF_8));
    }
}
