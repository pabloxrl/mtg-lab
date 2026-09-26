package org.mage.test.mtglab;

import com.google.gson.*;
import mage.Mana;
import mage.cards.Card;
import mage.constants.*;
import mage.game.Game;
import mage.players.Player;
import org.junit.Test;
import org.mage.test.player.TestComputerPlayer;
import org.mage.test.player.TestPlayer;
import org.mage.test.serverside.base.CardTestPlayerBase;
import java.nio.file.*;
import java.nio.charset.StandardCharsets;
import java.util.*;
import static org.junit.Assert.*;

/** Original mtg-lab test-only bridge. Upstream APIs consulted, no scenario copied. */
public class NeutralSmokeTest extends CardTestPlayerBase {
    private JsonObject fixture;
    private final Map<UUID, String> ids = new HashMap<>();
    private final JsonArray checkpoints = new JsonArray();
    private int consumed;

    @Override
    protected TestPlayer createPlayer(String name, RangeOfInfluence range) {
        return new TestPlayer(new TestComputerPlayer(name, range)) {
            @Override public boolean priority(Game game) {
                int seat = getId().equals(playerA.getId()) ? 0 : 1;
                assertEquals("unexpected step", PhaseStep.UPKEEP, game.getTurnStepType());
                assertEquals("unexpected turn", 1, game.getTurnNum());
                if (consumed == 0) {
                    checkpoints.add(checkpoint("initial", game));
                } else {
                    checkpoints.add(checkpoint("priority-p1", game));
                    game.pause(); // stop at next decision before taking another action
                    return false;
                }
                JsonObject action = fixture.getAsJsonArray("script").get(consumed).getAsJsonObject();
                assertEquals("script actor mismatch", seat, action.get("actor").getAsInt());
                assertEquals("unsupported action", "pass", action.get("kind").getAsString());
                JsonArray choices = action.getAsJsonArray("choices");
                assertEquals("missing/extra choice", 1, choices.size());
                JsonObject choice = choices.get(0).getAsJsonObject();
                assertEquals("choice actor mismatch", seat, choice.get("actor").getAsInt());
                assertEquals("unsupported choice", "pass", choice.get("kind").getAsString());
                assertEquals("unexpected choice values", 0, choice.getAsJsonArray("values").size());
                consumed++;
                pass(game); // explicitly requested pass, never TestPlayer.priority's default
                return true;
            }
            @Override public boolean chooseMulligan(Game game) {
                throw new AssertionError("unexpected mulligan in synthetic fixture");
            }
        };
    }

    private int seat(UUID id) {
        if (id.equals(playerA.getId())) return 0;
        if (id.equals(playerB.getId())) return 1;
        throw new AssertionError("unknown player");
    }

    private JsonArray zone(Collection<UUID> cards) {
        JsonArray out = new JsonArray();
        for (UUID id : cards) {
            assertTrue("unexpected object", ids.containsKey(id));
            out.add(ids.get(id));
        }
        return out;
    }

    private JsonObject checkpoint(String name, Game game) {
        JsonObject state = new JsonObject();
        state.addProperty("turn", game.getTurnNum());
        state.addProperty("active_player", seat(game.getActivePlayerId()));
        state.addProperty("priority", seat(game.getPriorityPlayerId()));
        state.addProperty("phase", game.getTurnPhaseType().name().toLowerCase(Locale.ROOT));
        state.addProperty("step", game.getTurnStepType().name().toLowerCase(Locale.ROOT));
        JsonArray players = new JsonArray();
        for (TestPlayer original : Arrays.asList(playerA, playerB)) {
            Player player = game.getPlayer(original.getId());
            JsonObject p = new JsonObject();
            p.addProperty("seat", seat(player.getId()));
            p.addProperty("life", player.getLife());
            Mana m = player.getManaPool().getMana();
            JsonObject mana = new JsonObject();
            mana.addProperty("W", m.getWhite()); mana.addProperty("U", m.getBlue());
            mana.addProperty("B", m.getBlack()); mana.addProperty("R", m.getRed());
            mana.addProperty("G", m.getGreen()); mana.addProperty("C", m.getColorless());
            p.add("mana", mana);
            JsonObject zones = new JsonObject();
            zones.add("library", zone(player.getLibrary().getCardList()));
            zones.add("hand", zone(player.getHand()));
            zones.add("graveyard", zone(player.getGraveyard()));
            assertTrue("unsupported battlefield", game.getBattlefield().getAllActivePermanents().isEmpty());
            zones.add("battlefield", new JsonArray());
            assertTrue("unsupported exile", game.getExile().getAllCards(game).isEmpty());
            zones.add("exile", new JsonArray());
            p.add("zones", zones); players.add(p);
        }
        state.add("players", players);
        assertTrue("unsupported stack", game.getStack().isEmpty());
        state.add("stack", new JsonArray());
        JsonObject result = new JsonObject(); result.addProperty("name", name); result.add("state", state);
        return result;
    }

    @Test public void neutralSmoke() throws Exception {
        fixture = JsonParser.parseString(new String(Files.readAllBytes(Paths.get(System.getProperty("mtglab.fixture"))), StandardCharsets.UTF_8)).getAsJsonObject();
        JsonObject state = fixture.getAsJsonObject("setup").getAsJsonObject("state");
        assertEquals(1, fixture.getAsJsonArray("script").size());
        setStrictChooseMode(true);
        currentGame.setStartingPlayerId(playerA.getId());
        gameOptions.skipInitShuffling = true;
        Map<String, String> names = new HashMap<>();
        for (JsonElement e : state.getAsJsonArray("objects")) {
            JsonObject o = e.getAsJsonObject();
            String card = o.get("card_id").getAsString();
            assertTrue("unsupported card", card.equals("forest") || card.equals("mountain"));
            names.put(o.get("id").getAsString(), card.equals("forest") ? "Forest" : "Mountain");
        }
        for (int i = 0; i < 2; i++) {
            TestPlayer player = i == 0 ? playerA : playerB;
            JsonObject p = state.getAsJsonArray("players").get(i).getAsJsonObject();
            removeAllCardsFromLibrary(player); removeAllCardsFromHand(player);
            setLife(player, p.get("life").getAsInt());
            JsonArray library = p.getAsJsonObject("zones").getAsJsonArray("library");
            for (int j = library.size() - 1; j >= 0; j--) {
                String id = library.get(j).getAsString();
                addCard(Zone.LIBRARY, player, "FDN-" + names.get(id), 1);
                List<Card> cards = getLibraryCards(player);
                ids.put(cards.get(cards.size() - 1).getId(), id);
            }
        }
        setStopAt(1, PhaseStep.PRECOMBAT_MAIN);
        execute();
        assertEquals("script unconsumed", 1, consumed);
        assertEquals("checkpoint missing", 2, checkpoints.size());
        JsonObject output = new JsonObject(); output.addProperty("bridge_version", 1);
        output.add("checkpoints", checkpoints);
        JsonObject unsupported = new JsonObject();
        unsupported.addProperty("object_characteristics_status_damage", "Library-only cards; permanent state unsupported in smoke v1");
        unsupported.addProperty("legal_choices", "Enumeration not exported by smoke v1; only explicit pass executed");
        unsupported.addProperty("outcome", "Stopped at intermediate decision, not game end");
        unsupported.addProperty("effects_and_private_views", "Not exported by smoke v1");
        unsupported.addProperty("land_plays_used", "Not exported by smoke v1");
        output.add("unsupported", unsupported);
        Files.write(Paths.get(System.getProperty("mtglab.output")), new GsonBuilder().setPrettyPrinting().create().toJson(output).getBytes(StandardCharsets.UTF_8));
    }
}
