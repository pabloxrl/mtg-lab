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
public class FullPoolResetTest {
    private static final Gson JSON = new GsonBuilder().serializeNulls().setPrettyPrinting().create();
    private final Path root = Paths.get(System.getProperty("mtglab.root"));
    private final JsonObject manifest = read(root.resolve("data/cards/foundations_micro_v1.json"));
    private final Map<UUID,String> bindings = new LinkedHashMap<>();
    private final Map<String,UUID> handles = new LinkedHashMap<>();
    private final TestPlayer[] players = new TestPlayer[2];
    private JsonArray consumed;
    private JsonArray rawPreShuffleOrders;
    private final JsonObject rawPreShuffleEvidence = new JsonObject();
    private JsonObject observed;
    private JsonObject current;
    private Game lastGame;
    private static class PrefixStop extends RuntimeException { }
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
        need(integer(doc.get("schema_version"),1),"/schema_version");
        need(doc.get("family").getAsString().equals("reset"),"/family");
        JsonObject pins=doc.getAsJsonObject("pins");
        keys(pins,"data/cards/foundations_micro_v1.json","data/rules/cr-2026-09-25.json","references/xmage/pins.json");
        for(String p:pins.keySet()) need(pins.get(p).getAsString().equals(sha(root.resolve(p))),"/pins");
        need(doc.getAsJsonArray("cases").size()>0,"/cases empty");
        Set<String> ids=new HashSet<>();
        for(JsonElement element:doc.getAsJsonArray("cases")) {
            JsonObject c=element.getAsJsonObject();keys(c,"id","starter","decks","chance","choices","stop");
            need(c.get("id").isJsonPrimitive() && c.get("id").getAsJsonPrimitive().isString() && !c.get("id").getAsString().isEmpty() && ids.add(c.get("id").getAsString()),"/id");
            need(integer(c.get("starter"),0)||integer(c.get("starter"),1),"/starter");
            need(c.get("stop").getAsString().equals("first_declaration"),"/stop unsupported callback");
            need(c.getAsJsonArray("choices").size()==0,"/choices unsupported mulligan/ongoing callback");
            need(c.getAsJsonArray("decks").size()==2,"/decks");
            need(c.getAsJsonArray("chance").size()==2,"/chance length");
            for(int seat=0;seat<2;seat++) {
                JsonObject d=c.getAsJsonArray("decks").get(seat).getAsJsonObject();keys(d,"deck","occurrences");
                JsonArray expected=canonical(d.get("deck").getAsString(),seat);
                need(expected.equals(d.get("occurrences")),"/deck occurrences");
                JsonObject event=c.getAsJsonArray("chance").get(seat).getAsJsonObject();keys(event,"sequence","actor","kind","before","after");
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
    private JsonObject observe(Game game, UUID actor) {
        need(consumed.size()==2,"/unconsumed chance");
        need(!game.hasEnded(),"/unexpected terminal game");
        need(game.getCards().size()==bindings.size(),"/unbound engine cards");
        need(game.getState().getChoosingPlayerId().equals(actor),"/declaration actor");
        need(game.getActivePlayerId()==null,"/unexpected turn-active player");
        JsonObject point=new JsonObject();point.addProperty("completion","prefix");point.addProperty("boundary","first_declaration");
        point.add("turn_active_seat",JsonNull.INSTANCE);
        point.addProperty("starting_seat",seat(game.getStartingPlayerId()));point.addProperty("declaration_seat",seat(actor));
        JsonArray life=new JsonArray(),hand=new JsonArray(),library=new JsonArray();JsonObject occurrences=new JsonObject();
        for(int s=0;s<2;s++) {
            life.add(game.getPlayer(players[s].getId()).getLife());
            hand.add(ids(game.getPlayer(players[s].getId()).getHand(),true));library.add(ids(game.getPlayer(players[s].getId()).getLibrary().getCardList(),false));
        }
        for(Map.Entry<UUID,String> entry:bindings.entrySet()) {
            Card card=game.getCard(entry.getKey());need(card!=null,"/missing engine card");
            String[] parts=entry.getValue().split("/");
            JsonObject o=new JsonObject();o.addProperty("seat",seat(card.getOwnerId()));
            // Key was bound to the concrete pinned card class at construction.
            o.addProperty("card",parts[1]);o.addProperty("copy",Integer.parseInt(parts[2]));
            Zone zone=game.getState().getZone(card.getId());
            need(zone==Zone.HAND||zone==Zone.LIBRARY,"/unexpected zone");
            o.addProperty("zone",zone==Zone.HAND?"hand":"library");occurrences.add(entry.getValue(),o);
        }
        point.add("life",life);point.add("hand",hand);point.add("library",library);point.add("occurrences",occurrences);
        point.add("consumed_chance",consumed);point.add("consumed_choices",new JsonArray());return point;
    }
    private TestPlayer player(final int s) {
        TestPlayer p=new TestPlayer(new TestComputerPlayer("Seat"+s,RangeOfInfluence.ONE)) {
            @Override public void shuffleLibrary(Ability source,Game game) {
                need(source==null && consumed.size()<2,"/unsupported shuffle callback");
                JsonObject event=current.getAsJsonArray("chance").get(consumed.size()).getAsJsonObject();
                need(integer(event.get("actor"),s),"/runtime chance actor");
                JsonArray before=ids(getLibrary().getCardList(),false);
                // Deck.getMaindeckCards collects into an unordered Set. The envelope's
                // before field is a multiset in creation order, not a pre-shuffle
                // library-order promise. Retain the raw engine order separately.
                need(sorted(before).equals(sorted(event.getAsJsonArray("before"))),"/runtime chance before");
                rawPreShuffleOrders.add(before.deepCopy());
                List<String> creationOrder = new ArrayList<>(handles.keySet());
                List<String> witnessed = strings(before);
                witnessed.sort(Comparator.comparingInt(creationOrder::indexOf));
                before = JSON.toJsonTree(witnessed).getAsJsonArray();
                List<String> after=strings(event.getAsJsonArray("after"));
                for(int i=after.size()-1;i>=0;i--) getLibrary().putOnTop(game.getCard(handles.get(after.get(i))),game);
                JsonObject used=new JsonObject();used.addProperty("sequence",consumed.size());used.addProperty("kind","initial_shuffle");used.addProperty("actor",s);used.add("before",before);used.add("after",ids(getLibrary().getCardList(),false));consumed.add(used);
            }
            @Override public boolean chooseMulligan(Game game) {
                need(observed==null,"/unsupported mulligan callback");observed=observe(game,getId());throw new PrefixStop();
            }
            @Override public boolean priority(Game game) {throw new IllegalArgumentException("first divergence: unsupported priority callback");}
            @Override public boolean chooseTarget(Outcome outcome,Target target,Ability source,Game game) {throw new IllegalArgumentException("first divergence: unsupported target callback");}
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
        current=c;bindings.clear();handles.clear();consumed=new JsonArray();rawPreShuffleOrders=new JsonArray();observed=null;
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
        try {game.start(players[c.get("starter").getAsInt()].getId());fail("missing declaration callback");} catch(PrefixStop expected) {need(observed!=null,"/missing observation");}
        return observed;
    }
    @Test public void resetPrefixes() throws Exception {
        JsonObject doc=read(Paths.get(System.getProperty("mtglab.fixture")));validate(doc);
        JsonObject points=new JsonObject(),repeated=new JsonObject(),rejected=new JsonObject();
        for(JsonElement c:doc.getAsJsonArray("cases")) {
            String id=c.getAsJsonObject().get("id").getAsString();points.add(id,run(c.getAsJsonObject()));rawPreShuffleEvidence.add(id,rawPreShuffleOrders);repeated.add(id,run(c.getAsJsonObject()));rawPreShuffleEvidence.add(id+"/repeat",rawPreShuffleOrders);
        }
        JsonObject bad=read(Paths.get(System.getProperty("mtglab.negatives")));
        for(String name:bad.keySet()) {
            try {validate(bad.getAsJsonObject(name));fail("accepted negative "+name);}catch(IllegalArgumentException e) {need(e.getMessage().startsWith("first divergence:"),"/negative diagnostic");rejected.addProperty(name,e.getMessage());}
        }
        // Supplemental callback probes, not claimed as played continuation.
        JsonObject callbacks=new JsonObject();
        try {players[0].priority(lastGame);fail("accepted ongoing callback");} catch(IllegalArgumentException e) {callbacks.addProperty("priority",e.getMessage());}
        try {players[0].chooseMulligan(lastGame);fail("accepted second declaration callback");} catch(IllegalArgumentException e) {callbacks.addProperty("mulligan_again",e.getMessage());}
        JsonObject result=new JsonObject();result.add("raw_pre_shuffle_orders",rawPreShuffleEvidence);result.add("callback_controls",callbacks);result.add("checkpoints",points);result.add("repeat_checkpoints",repeated);result.add("rejections",rejected);
        Files.write(Paths.get(System.getProperty("mtglab.output")),JSON.toJson(result).getBytes(StandardCharsets.UTF_8));
    }
}
