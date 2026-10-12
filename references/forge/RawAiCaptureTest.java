package forge.gamesimulationtests.mtglab;

import com.google.gson.*;
import com.google.common.eventbus.Subscribe;
import forge.card.CardMockTestCase;
import forge.ai.*;
import forge.deck.Deck;
import forge.game.*;
import forge.game.event.Event;
import forge.game.player.RegisteredPlayer;
import forge.gamesimulationtests.util.CardDatabaseHelper;
import forge.localinstance.properties.ForgeConstants;
import forge.util.MyRandom;
import org.mockito.Mockito;
import org.testng.annotations.Test;
import java.nio.file.*;
import java.util.*;
import static org.testng.Assert.*;

/** Original capture harness: real controllers, normal opening, no manufactured ending. */
public class RawAiCaptureTest extends CardMockTestCase {
    private final Map<String, Integer> events = new TreeMap<>();
    private Game observedGame;
    private Throwable auditFailure;
    private final java.security.MessageDigest auditDigest;
    public RawAiCaptureTest() throws Exception { auditDigest = java.security.MessageDigest.getInstance("SHA-256"); }
    @Override protected void initForgeSingletons() {
        System.setIn(java.io.InputStream.nullInputStream());
        forge.gui.GuiBase.setInterface(Mockito.mock(forge.gui.interfaces.IGuiBase.class, call -> {
            switch (call.getMethod().getName()) {
                case "getAssetsDir": return System.getProperty("mtglab.assets");
                case "isRunningOnDesktop": return true;
                default: throw new AssertionError("unsupported GUI callback: " + call.getMethod().getName());
            }
        }));
        super.initForgeSingletons();
        forge.model.FModel.loadDynamicGamedata();
    }
    @Subscribe public void observe(Event event) {
        events.merge(event.getClass().getSimpleName(), 1, Integer::sum);
        try {
        String line = event.getClass().getSimpleName() + ":" + MtgRawCapture.audit(observedGame).toString() + "\n";
        auditDigest.update(line.getBytes(java.nio.charset.StandardCharsets.UTF_8));
        } catch (RuntimeException | Error error) { auditFailure = error; throw error; }
    }
    @Test public void rawAiCapture() throws Exception {
        assertEquals(System.in.read(), -1);
        assertTrue(java.awt.GraphicsEnvironment.isHeadless());
        AiProfileUtil.loadAllProfiles(ForgeConstants.AI_PROFILE_DIR);
        assertEquals(forge.StaticData.instance().getMulliganRule().name(), "London");
        assertFalse(forge.StaticData.instance().getFilteredHandsEnabled(), "normal unfiltered opening required");
        MyRandom.setRandom(new MtgRawCapture.ObservedRandom(Long.getLong("mtglab.seed", 324L)));
        JsonObject manifest = JsonParser.parseString(Files.readString(Path.of(System.getProperty("mtglab.manifest")))).getAsJsonObject();
        Map<String,String> names = new HashMap<>();
        for (JsonElement e : manifest.getAsJsonArray("cards")) {
            JsonObject c = e.getAsJsonObject(); names.put(c.get("id").getAsString(), c.get("name").getAsString());
        }
        List<RegisteredPlayer> seats = new ArrayList<>();
        for (JsonElement e : manifest.getAsJsonArray("decks")) {
            JsonObject d = e.getAsJsonObject();
            Deck deck = new Deck(d.get("id").getAsString());
            for (JsonElement entry : d.getAsJsonArray("cards")) {
                JsonObject c = entry.getAsJsonObject();
                deck.getMain().add(CardDatabaseHelper.getCard(names.get(c.get("card_id").getAsString())), c.get("copies").getAsInt());
            }
            assertEquals(deck.getMain().countAll(), 40);
            LobbyPlayerAi ai = new LobbyPlayerAi("seat-" + seats.size(), null);
            ai.setAiProfile("Default");
            seats.add(new RegisteredPlayer(deck).setPlayer(ai));
        }
        Match match = new Match(new GameRules(GameType.Constructed), seats, "raw capture");
        Game game = match.createGame();
        game.setNoGUIUser();
        for (forge.game.player.Player player : game.getPlayers()) {
            assertTrue(player.getController() instanceof PlayerControllerAi);
            assertFalse(((PlayerControllerAi)player.getController()).getAi().usesFullSimulation());
        }
        observedGame = game;
        game.subscribeToEvents(this);
        boolean record = Boolean.parseBoolean(System.getProperty("mtglab.record", "true"));
        if (record) MtgRawCapture.open(game, Path.of(System.getProperty("mtglab.output") + ".jsonl.gz"), Long.getLong("mtglab.maxCalls", 10000L));
        Files.writeString(Path.of(System.getProperty("mtglab.output") + ".started"), "normal-match-start\n", StandardOpenOption.CREATE_NEW);
        String status = "failed";
        try {
            match.startGame(game);
            status = game.isGameOver() ? "complete" : "truncated";
        } finally {
            if (record) MtgRawCapture.close(status);
        }
        assertNull(auditFailure, "event audit must not lose exceptions inside EventBus");
        assertTrue(game.isGameOver(), "normal match must terminate");
        JsonObject result = new JsonObject();
        result.addProperty("audit_sha256", java.util.HexFormat.of().formatHex(auditDigest.digest()));
        result.addProperty("terminal", game.isGameOver());
        result.addProperty("turn", game.getPhaseHandler().getTurn());
        result.add("events", new Gson().toJsonTree(events));
        result.add("final", MtgRawCapture.audit(game));
        Files.writeString(Path.of(System.getProperty("mtglab.output")), new Gson().toJson(result));
        assertTrue(events.getOrDefault("GameEventSpellAbilityCast", 0) > 0, "must cast spells");
        assertTrue(events.getOrDefault("GameEventCombatChanged", 0) > 0, "must reach combat");
    }
}
