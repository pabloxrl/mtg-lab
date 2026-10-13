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

/** Normal frozen-deck setup with strict recorded opening controllers. */
public class OpeningReplayTest extends CardMockTestCase {
    private final Map<String, Integer> events = new TreeMap<>();
    private Game observedGame;
    private Throwable auditFailure;
    private final java.security.MessageDigest auditDigest;
    public OpeningReplayTest() throws Exception { auditDigest = java.security.MessageDigest.getInstance("SHA-256"); }
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
    @Test public void openingReplay() throws Exception {
        assertEquals(System.in.read(), -1);
        assertTrue(java.awt.GraphicsEnvironment.isHeadless());
        AiProfileUtil.loadAllProfiles(ForgeConstants.AI_PROFILE_DIR);
        assertEquals(forge.StaticData.instance().getMulliganRule().name(), "London");
        assertFalse(forge.StaticData.instance().getFilteredHandsEnabled(), "normal unfiltered opening required");
        MyRandom.setRandom(new java.util.Random(0));
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
        Path output=Path.of(System.getProperty("mtglab.output"));
        MtgOpeningReplay.open(game,Path.of(System.getProperty("mtglab.input")));
        MyRandom.setRandom(new MtgOpeningReplay.RecordedRandom());
        boolean reached=false;
        try { match.startGame(game); }
        catch (MtgOpeningReplay.Endpoint expected) { reached=true; }
        catch (IllegalArgumentException rejected) {
            String wanted=System.getProperty("mtglab.expectedRejection");
            if(wanted==null)throw rejected;
            assertTrue(rejected.getMessage().contains(wanted),"wrong rejection: "+rejected.getMessage());
            JsonObject result=new JsonObject(); result.addProperty("status","negative-detected");
            result.addProperty("reason",rejected.getMessage()); result.addProperty("complete_game",false);
            Files.writeString(output,result.toString(),StandardOpenOption.CREATE_NEW);
            return;
        }
        assertNull(System.getProperty("mtglab.expectedRejection"),"malformed tape was accepted");
        assertTrue(reached,"must reach exact opening endpoint");
        assertFalse(game.isGameOver(),"prefix cannot count as complete game");
        MtgOpeningReplay.finish(output);
    }
}
