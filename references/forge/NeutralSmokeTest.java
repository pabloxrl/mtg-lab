package forge.gamesimulationtests.mtglab;

import com.google.gson.*;
import forge.game.Game;
import forge.game.card.Card;
import forge.game.player.Player;
import forge.game.phase.PhaseType;
import forge.game.zone.ZoneType;
import forge.gamesimulationtests.BaseGameSimulationTest;
import forge.gamesimulationtests.util.GameWrapper;
import forge.gamesimulationtests.util.PlayerControllerForTests;
import forge.gamesimulationtests.util.card.CardSpecificationBuilder;
import forge.gamesimulationtests.util.gamestate.GameStateSpecificationBuilder;
import forge.gamesimulationtests.util.player.PlayerSpecification;
import forge.gamesimulationtests.util.player.PlayerSpecificationBuilder;
import org.mockito.MockedConstruction;
import org.mockito.Mockito;
import org.testng.annotations.Test;
import java.nio.file.*;
import java.util.*;
import static org.testng.Assert.*;

/** Original mtg-lab adapter, no upstream scenario copied or translated. */
public class NeutralSmokeTest extends BaseGameSimulationTest {
    private JsonObject fixture;
    private final JsonArray checkpoints = new JsonArray();
    private final JsonObject observedCards = new JsonObject();
    private final Map<Integer, String> ids = new HashMap<>();
    private int consumed;
    private static final class CheckpointReached extends RuntimeException { }

    @Override protected void initForgeSingletons() {
        // Surefire owns the fork's OS stdin for its control protocol. Expose EOF
        // to all test/engine code instead of allowing reads from that pipe.
        System.setIn(java.io.InputStream.nullInputStream());
        forge.gui.GuiBase.setInterface(Mockito.mock(forge.gui.interfaces.IGuiBase.class, call -> {
            switch (call.getMethod().getName()) {
                case "getAssetsDir": return System.getProperty("mtglab.assets");
                case "isRunningOnDesktop": return true;
                default: throw new AssertionError("unexpected GUI call: " + call.getMethod().getName());
            }
        }));
        super.initForgeSingletons();
    }

    private int seat(Player player) {
        return player.getGame().getPlayers().indexOf(player);
    }

    private Object priority(Player player) {
        Game game = player.getGame();
        assertEquals(game.getPhaseHandler().getPhase(), PhaseType.UPKEEP, "unexpected step");
        assertEquals(game.getPhaseHandler().getTurn(), 1, "unexpected turn");
        assertSame(game.getPhaseHandler().getPriorityPlayer(), player, "controller is not priority holder");
        if (consumed == 1) {
            checkpoints.add(checkpoint("priority-p1", game));
            // Stop BEFORE another decision; never invent a concession or second pass.
            throw new CheckpointReached();
        }
        bindIdentities(game);
        checkpoints.add(checkpoint("initial", game));
        JsonObject action = fixture.getAsJsonArray("script").get(consumed).getAsJsonObject();
        assertEquals(action.get("actor").getAsInt(), seat(player), "script actor mismatch");
        assertEquals(action.get("kind").getAsString(), "pass", "unsupported action");
        assertTrue(action.get("source").isJsonNull(), "unexpected source");
        JsonArray choices = action.getAsJsonArray("choices");
        assertEquals(choices.size(), 1, "missing/extra choice");
        JsonObject choice = choices.get(0).getAsJsonObject();
        assertEquals(choice.get("actor").getAsInt(), seat(player), "choice actor mismatch");
        assertEquals(choice.get("kind").getAsString(), "pass", "unsupported choice");
        assertEquals(choice.getAsJsonArray("values").size(), 0, "unexpected choice values");
        consumed++;
        return null; // Forge's explicit pass API; PhaseHandler performs the handoff.
    }

    private void bindIdentities(Game game) {
        for (Player player : game.getPlayers()) {
            for (Card card : player.getCardsIn(ZoneType.Library)) {
                String id = null;
                for (JsonElement element : fixture.getAsJsonObject("setup").getAsJsonObject("state").getAsJsonArray("objects")) {
                    JsonObject object = element.getAsJsonObject();
                    if (object.get("owner").getAsInt() == seat(card.getOwner()) &&
                            object.get("card_id").getAsString().equals(card.getName().toLowerCase(Locale.ROOT))) {
                        assertNull(id, "ambiguous object identity");
                        id = object.get("id").getAsString();
                    }
                }
                assertNotNull(id, "unexpected card identity");
                assertFalse(ids.containsValue(id), "duplicate object identity");
                ids.put(card.getId(), id);
                observedCards.addProperty(id, card.getName().toLowerCase(Locale.ROOT));
            }
        }
    }

    private JsonObject checkpoint(String name, Game game) {
        JsonObject state = new JsonObject();
        state.addProperty("turn", game.getPhaseHandler().getTurn());
        state.addProperty("active_player", seat(game.getPhaseHandler().getPlayerTurn()));
        state.addProperty("priority", seat(game.getPhaseHandler().getPriorityPlayer()));
        assertEquals(game.getPhaseHandler().getPhase(), PhaseType.UPKEEP, "unsupported phase");
        state.addProperty("phase", "beginning");
        state.addProperty("step", "upkeep");
        JsonArray players = new JsonArray();
        for (Player player : game.getPlayers()) {
            JsonObject p = new JsonObject();
            p.addProperty("seat", seat(player));
            p.addProperty("life", player.getLife());
            JsonObject mana = new JsonObject();
            String[] colors = {"W", "U", "B", "R", "G", "C"};
            byte[] masks = {1, 2, 4, 8, 16, 0};
            for (int i = 0; i < colors.length; i++) {
                mana.addProperty(colors[i], player.getManaPool().getAmountOfColor(masks[i]));
            }
            p.add("mana", mana);
            JsonObject zones = new JsonObject();
            ZoneType[] types = {ZoneType.Library, ZoneType.Hand, ZoneType.Battlefield, ZoneType.Graveyard, ZoneType.Exile};
            for (ZoneType type : types) {
                JsonArray cards = new JsonArray();
                for (Card card : player.getCardsIn(type)) {
                    assertTrue(ids.containsKey(card.getId()), "unmapped card");
                    cards.add(ids.get(card.getId()));
                }
                zones.add(type.name().toLowerCase(Locale.ROOT), cards);
            }
            p.add("zones", zones);
            players.add(p);
        }
        state.add("players", players);
        assertTrue(game.getStack().isEmpty(), "unsupported stack");
        state.add("stack", new JsonArray());
        JsonObject point = new JsonObject();
        point.addProperty("name", name);
        point.add("state", state);
        return point;
    }

    @Test public void neutralSmoke() throws Exception {
        assertNull(System.getenv("DISPLAY"), "display must be unset");
        assertNull(System.getenv("WAYLAND_DISPLAY"), "display must be unset");
        assertTrue(java.awt.GraphicsEnvironment.isHeadless(), "headless JVM required");
        assertEquals(System.in.read(), -1, "stdin must be closed");
        fixture = JsonParser.parseString(Files.readString(Path.of(System.getProperty("mtglab.fixture")))).getAsJsonObject();
        assertEquals(fixture.getAsJsonArray("script").size(), 1, "exactly one scripted pass");
        JsonObject state = fixture.getAsJsonObject("setup").getAsJsonObject("state");
        GameStateSpecificationBuilder setup = new GameStateSpecificationBuilder();
        PlayerSpecification[] seats = {PlayerSpecification.PLAYER_1, PlayerSpecification.PLAYER_2};
        Map<String, String> names = new HashMap<>();
        for (JsonElement element : state.getAsJsonArray("objects")) {
            JsonObject object = element.getAsJsonObject();
            String card = object.get("card_id").getAsString();
            assertTrue(card.equals("forest") || card.equals("mountain"), "unsupported card");
            names.put(object.get("id").getAsString(), card.equals("forest") ? "Forest" : "Mountain");
        }
        for (int i = 0; i < seats.length; i++) {
            JsonObject p = state.getAsJsonArray("players").get(i).getAsJsonObject();
            setup.addPlayerFact(new PlayerSpecificationBuilder(seats[i].getName()).life(p.get("life").getAsInt()));
            for (JsonElement id : p.getAsJsonObject("zones").getAsJsonArray("library")) {
                setup.addCard(new CardSpecificationBuilder(names.get(id.getAsString())).owner(seats[i]).library());
            }
        }
        // Replace EVERY controller method with a fail-closed answer. Only metadata,
        // bookkeeping and the single scripted priority decision are allowed.
        try (MockedConstruction<PlayerControllerForTests> controllers = Mockito.mockConstruction(
                PlayerControllerForTests.class, context -> Mockito.withSettings().defaultAnswer(call -> {
                    Player player = (Player) context.arguments().get(1);
                    switch (call.getMethod().getName()) {
                        case "getPlayer": return player;
                        case "getGame": return player.getGame();
                        case "getLobbyPlayer": return context.arguments().get(2);
                        case "isAI": return false;
                        case "setPlayerActions": return null;
                        case "chooseSpellAbilityToPlay": return priority(player);
                        default: throw new AssertionError("unscripted controller call: " + call.getMethod().getName());
                    }
                }))) {
            GameWrapper wrapper = new GameWrapper(setup.build(), null);
            try {
                wrapper.runGame();
                fail("missing intermediate checkpoint");
            } catch (CheckpointReached expected) {
                assertEquals(consumed, 1, "unconsumed script");
                assertEquals(checkpoints.size(), 2, "missing checkpoint");
            }
            assertEquals(controllers.constructed().size(), 2, "unexpected controller count");
        }
        JsonObject output = new JsonObject();
        output.addProperty("bridge_version", 1);
        output.add("checkpoints", checkpoints);
        output.add("observed_cards", observedCards);
        JsonObject unsupported = new JsonObject();
        unsupported.addProperty("object_characteristics_status_damage", "Library-only smoke; permanent attributes not exported");
        unsupported.addProperty("legal_choices", "Only explicit pass; enumeration not exported");
        unsupported.addProperty("outcome", "Stopped at intermediate priority, not game end");
        unsupported.addProperty("effects_and_private_views", "Not exported by smoke v1");
        unsupported.addProperty("land_plays_used", "Not exported by smoke v1");
        output.add("unsupported", unsupported);
        Files.writeString(Path.of(System.getProperty("mtglab.output")), new GsonBuilder().setPrettyPrinting().create().toJson(output));
    }
}
