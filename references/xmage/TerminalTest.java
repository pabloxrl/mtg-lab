package org.mage.test.mtglab;

import com.google.gson.*;
import com.google.gson.stream.*;
import java.io.StringReader;
import mage.constants.*;
import mage.game.Game;
import mage.players.Player;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.junit.runners.Parameterized;
import org.mage.test.player.TestComputerPlayer;
import org.mage.test.player.TestPlayer;
import org.mage.test.serverside.base.CardTestPlayerBase;
import java.nio.file.*;
import java.nio.charset.StandardCharsets;
import java.util.*;
import static org.junit.Assert.*;

/** Original synthetic boundaries. No expected result supplies an observation. */
@RunWith(Parameterized.class)
public class TerminalTest extends CardTestPlayerBase {
    private final JsonObject fixture;
    private final int version;
    private final boolean invalid;
    private final String invalidId, raw;
    private JsonObject observed, pending;
    private final JsonArray consumed = new JsonArray();
    private Player a, b;
    private int calls;

    private static void require(boolean ok, String message) {
        if (!ok) throw new IllegalArgumentException("terminal input: " + message);
    }
    private static void fields(JsonObject obj, String required, String optional) {
        Set<String> allowed = new HashSet<>(Arrays.asList((required + " " + optional).trim().split(" +")));
        for (String key : obj.keySet()) require(allowed.contains(key), "unknown field " + key);
        for (String key : required.split(" +")) require(obj.has(key), "missing field " + key);
    }
    private static int integer(JsonElement value) {
        require(value != null && value.isJsonPrimitive() && value.getAsJsonPrimitive().isNumber()
                && value.toString().matches("-?(0|[1-9][0-9]*)"), "integer required");
        try { return Integer.parseInt(value.toString()); }
        catch (NumberFormatException e) { throw new IllegalArgumentException("terminal input: integer range", e); }
    }
    private static String string(JsonElement value) {
        require(value != null && value.isJsonPrimitive() && value.getAsJsonPrimitive().isString(), "string required");
        return value.getAsString();
    }
    private static int seat(JsonElement value) {
        int n = integer(value); require(n == 0 || n == 1, "invalid seat"); return n;
    }
    private static JsonArray pair(JsonElement value) {
        require(value != null && value.isJsonArray() && value.getAsJsonArray().size() == 2, "pair required");
        return value.getAsJsonArray();
    }
    private static void expectation(JsonObject e, int version) {
        fields(e, "life lost library hand" + (version == 2 ? " outcome winner draw" : ""), "");
        for (String f : Arrays.asList("life", "lost", "library", "hand")) {
            for (JsonElement value : pair(e.get(f))) {
                if (f.equals("lost")) require(value.isJsonPrimitive() && value.getAsJsonPrimitive().isBoolean(), "boolean required");
                else integer(value);
            }
        }
        if (version == 2) {
            require(Arrays.asList("ongoing", "win", "draw").contains(string(e.get("outcome"))), "invalid outcome");
            if (!e.get("winner").isJsonNull()) seat(e.get("winner"));
            require(e.get("draw").isJsonPrimitive() && e.get("draw").getAsJsonPrimitive().isBoolean(), "invalid draw");
        }
    }
    private static int validate(JsonObject doc) {
        fields(doc, "version basis cases", "");
        int version = integer(doc.get("version")); require(version == 1 || version == 2, "unsupported version");
        require(!string(doc.get("basis")).isEmpty(), "basis required");
        require(doc.get("cases").isJsonArray() && doc.getAsJsonArray("cases").size() > 0, "cases required");
        Set<String> ids = new HashSet<>();
        for (JsonElement element : doc.getAsJsonArray("cases")) {
            require(element.isJsonObject(), "case object required"); JsonObject c = element.getAsJsonObject();
            fields(c, "id life action expected" + (version == 2 ? " life_order injection_order expected_pending expected_drawn" : ""),
                    "library_card" + (version == 2 ? " draw_seat concede_seat" : ""));
            String id = string(c.get("id")); require(id.matches("[a-z0-9][a-z0-9_-]*") && ids.add(id), "invalid/duplicate id");
            for (JsonElement value : pair(c.get("life"))) integer(value);
            String action = string(c.get("action")); require(Arrays.asList("settle", "draw", "concede").contains(action), "invalid action");
            if (c.has("library_card")) require(action.equals("draw") && string(c.get("library_card")).equals("Forest"), "conflicting library_card");
            expectation(c.getAsJsonObject("expected"), version);
            if (version == 2) {
                JsonArray order = pair(c.get("life_order")); require(seat(order.get(0)) != seat(order.get(1)), "invalid life_order");
                String injection = string(c.get("injection_order"));
                require(injection.equals("life_then_draw") || (injection.equals("draw_then_life") && action.equals("draw")), "invalid/conflicting injection_order");
                for (String kind : Arrays.asList("draw", "concede")) {
                    String key = kind + "_seat"; require(c.has(key) == action.equals(kind), "conflicting or missing " + key);
                    if (c.has(key)) seat(c.get(key));
                }
                if (action.equals("concede")) require(integer(c.getAsJsonArray("life").get(0)) == 20 && integer(c.getAsJsonArray("life").get(1)) == 20, "conflicting concession");
                if (action.equals("draw")) seat(c.get("expected_drawn"));
                else require(c.get("expected_drawn").isJsonNull(), "conflicting expected_drawn");
                expectation(c.getAsJsonObject("expected_pending"), 2);
            }
        }
        return version;
    }
    private static JsonElement readUnique(JsonReader r) throws java.io.IOException {
        switch (r.peek()) {
            case BEGIN_OBJECT:
                r.beginObject(); JsonObject object = new JsonObject();
                while (r.hasNext()) {
                    String key = r.nextName(); require(!object.has(key), "duplicate field " + key);
                    object.add(key, readUnique(r));
                }
                r.endObject(); return object;
            case BEGIN_ARRAY:
                r.beginArray(); JsonArray array = new JsonArray();
                while (r.hasNext()) array.add(readUnique(r));
                r.endArray(); return array;
            case STRING: return new JsonPrimitive(r.nextString());
            case NUMBER: return JsonParser.parseString(r.nextString());
            case BOOLEAN: return new JsonPrimitive(r.nextBoolean());
            case NULL: r.nextNull(); return JsonNull.INSTANCE;
            default: throw new IllegalArgumentException("terminal input: malformed JSON");
        }
    }
    private static JsonElement parse(String text) {
        try (JsonReader reader = new JsonReader(new StringReader(text))) {
            reader.setLenient(false);
            JsonElement result = readUnique(reader);
            require(reader.peek() == JsonToken.END_DOCUMENT, "trailing JSON");
            return result;
        } catch (java.io.IOException e) { throw new IllegalArgumentException("terminal input: malformed JSON", e); }
    }
    @Parameterized.Parameters(name="{index}") public static Collection<Object[]> cases() throws Exception {
        JsonElement doc = parse(new String(Files.readAllBytes(Paths.get(System.getProperty("mtglab.fixture"))), StandardCharsets.UTF_8));
        List<Object[]> out = new ArrayList<>();
        if (Boolean.getBoolean("mtglab.invalid")) {
            for (JsonElement row : doc.getAsJsonArray()) {
                JsonObject r = row.getAsJsonObject(); out.add(new Object[]{r.has("fixture") ? r.getAsJsonObject("fixture") : null, 0, true, r.get("id").getAsString(), r.has("text") ? r.get("text").getAsString() : null});
            }
        } else {
            int version = validate(doc.getAsJsonObject());
            for (JsonElement c : doc.getAsJsonObject().getAsJsonArray("cases")) out.add(new Object[]{c.getAsJsonObject(), version, false, "", null});
        }
        return out;
    }
    public TerminalTest(JsonObject f, int v, boolean bad, String id, String text) { fixture=f; version=v; invalid=bad; invalidId=id; raw=text; }
    private Player player(int seat) { return seat == 0 ? a : b; }
    private void record(String action, Integer seat, String valueKey, Integer value) {
        JsonObject step = new JsonObject(); step.addProperty("action", action);
        if (seat != null) step.addProperty("seat", seat);
        if (valueKey != null) step.addProperty(valueKey, value);
        consumed.add(step);
    }
    private JsonObject snapshot(Game game, boolean explicit) {
        JsonObject out = new JsonObject(); JsonArray life=new JsonArray(),lost=new JsonArray(),library=new JsonArray(),hand=new JsonArray();
        for (Player p : Arrays.asList(a,b)) {life.add(p.getLife());lost.add(p.hasLost());library.add(p.getLibrary().size());hand.add(p.getHand().size());}
        out.add("life",life);out.add("lost",lost);out.add("library",library);out.add("hand",hand);
        if (explicit) {
            boolean ended = game.hasEnded();
            // winnerId is finalized by XMage's game loop, so settled snapshot is taken after execute().
            boolean draw = game.isADraw();
            out.addProperty("outcome", !ended ? "ongoing" : draw ? "draw" : "win");
            out.addProperty("draw", draw);
            if (a.hasWon()) out.addProperty("winner", 0);
            else if (b.hasWon()) out.addProperty("winner", 1);
            else out.add("winner", JsonNull.INSTANCE);
        }
        return out;
    }
    private void injectLife(Game game) {
        JsonArray order = version == 1 ? JsonParser.parseString("[0,1]").getAsJsonArray() : fixture.getAsJsonArray("life_order");
        for (JsonElement n : order) {
            int seat = n.getAsInt(), life = fixture.getAsJsonArray("life").get(seat).getAsInt();
            player(seat).setLife(life, game, null); record("life", seat, "value", life);
        }
    }
    private void draw(Game game) {
        int seat = version == 1 ? 0 : fixture.get("draw_seat").getAsInt();
        if (System.getProperty("mtglab.fault", "none").equals("wrong_draw_seat")) seat = 1-seat;
        int drawn = player(seat).drawCards(1, null, game); record("draw", seat, "drawn", drawn);
    }
    @Override protected TestPlayer createPlayer(String name, RangeOfInfluence range) {
        return new TestPlayer(new TestComputerPlayer(name,range)) {
            @Override public boolean priority(Game game) {
                assertEquals(0,calls++);assertEquals(playerA.getId(),getId());assertEquals(PhaseStep.UPKEEP,game.getTurnStepType());
                a=game.getPlayer(playerA.getId()); b=game.getPlayer(playerB.getId());
                String action=fixture.get("action").getAsString();
                boolean drawFirst=version==2 && fixture.get("injection_order").getAsString().equals("draw_then_life");
                if (drawFirst) draw(game); else injectLife(game);
                if (System.getProperty("mtglab.fault", "none").equals("premature_settlement")) game.checkStateAndTriggered();
                if (drawFirst) injectLife(game); else if (action.equals("draw")) draw(game);
                pending=snapshot(game, version==2);
                if (action.equals("concede")) {
                    int seat=version==1 ? 1 : fixture.get("concede_seat").getAsInt();
                    player(seat).concede(game); record("concede",seat,null,null);
                }
                game.checkStateAndTriggered(); record("settle",null,null,null);
                observed=snapshot(game,false); game.pause(); return false;
            }
            @Override public boolean chooseMulligan(Game game) {throw new AssertionError("unexpected mulligan");}
        };
    }
    private void write(String id, JsonObject result) throws Exception {
        Files.write(Paths.get(System.getProperty("mtglab.output"),id+".json"),new GsonBuilder().serializeNulls().setPrettyPrinting().create().toJson(result).getBytes(StandardCharsets.UTF_8));
    }
    @Test public void terminal() throws Exception {
        if (invalid) {
            try { validate(raw == null ? fixture : parse(raw).getAsJsonObject()); fail("invalid terminal input accepted: " + invalidId); }
            catch (IllegalArgumentException e) { assertTrue(e.getMessage(), e.getMessage().startsWith("terminal input:")); }
            JsonObject result = new JsonObject(); result.addProperty("rejected",true); write(invalidId,result); return;
        }
        setStrictChooseMode(true);currentGame.setStartingPlayerId(playerA.getId());gameOptions.skipInitShuffling=true;
        removeAllCardsFromHand(playerA);removeAllCardsFromHand(playerB);removeAllCardsFromLibrary(playerA);removeAllCardsFromLibrary(playerB);
        if(fixture.has("library_card")) {
            int seat=version==1 ? 0 : fixture.get("draw_seat").getAsInt();
            addCard(Zone.LIBRARY,seat==0 ? playerA : playerB,"FDN-"+fixture.get("library_card").getAsString(),1);
        }
        setLife(playerA,20);setLife(playerB,20);setStopAt(1,PhaseStep.PRECOMBAT_MAIN);execute();assertEquals(1,calls);
        if (version == 2) {
            JsonObject result=new JsonObject(); result.addProperty("version",2); result.add("pending",pending);
            result.add("settled",snapshot(currentGame,true)); result.add("consumed",consumed); observed=result;
        }
        write(fixture.get("id").getAsString(),observed);
        if (version == 1) assertEquals(fixture.getAsJsonObject("expected"), observed);
    }
}
