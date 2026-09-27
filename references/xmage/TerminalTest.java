package org.mage.test.mtglab;

import com.google.gson.*;
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

/** Original synthetic terminal boundary checks. No copied upstream test. */
@RunWith(Parameterized.class)
public class TerminalTest extends CardTestPlayerBase {
    private final JsonObject fixture;
    private JsonObject observed;
    private int calls;
    @Parameterized.Parameters(name="{index}") public static Collection<Object[]> cases() throws Exception {
        JsonObject doc=JsonParser.parseString(new String(Files.readAllBytes(Paths.get(System.getProperty("mtglab.fixture"))),StandardCharsets.UTF_8)).getAsJsonObject();
        List<Object[]> out=new ArrayList<>();for(JsonElement c:doc.getAsJsonArray("cases"))out.add(new Object[]{c.getAsJsonObject()});return out;
    }
    public TerminalTest(JsonObject f) {fixture=f;}
    @Override protected TestPlayer createPlayer(String name, RangeOfInfluence range) {
        return new TestPlayer(new TestComputerPlayer(name,range)) {
            @Override public boolean priority(Game game) {
                assertEquals(0,calls++);assertEquals(playerA.getId(),getId());assertEquals(PhaseStep.UPKEEP,game.getTurnStepType());
                Player a=game.getPlayer(playerA.getId()), b=game.getPlayer(playerB.getId());
                // Explicit synthetic checkpoint injection, then the real XMage SBA.
                a.setLife(fixture.getAsJsonArray("life").get(0).getAsInt(),game,null);
                b.setLife(fixture.getAsJsonArray("life").get(1).getAsInt(),game,null);
                String action=fixture.get("action").getAsString();
                if(action.equals("draw"))a.drawCards(1,null,game);
                else if(action.equals("concede"))b.concede(game);
                else assertEquals("settle",action);
                game.checkStateAndTriggered();
                observed=new JsonObject();JsonArray life=new JsonArray(),lost=new JsonArray(),library=new JsonArray(),hand=new JsonArray();
                for(Player p:Arrays.asList(a,b)) {life.add(p.getLife());lost.add(p.hasLost());library.add(p.getLibrary().size());hand.add(p.getHand().size());}
                observed.add("life",life);observed.add("lost",lost);observed.add("library",library);observed.add("hand",hand);
                game.pause();return false;
            }
            @Override public boolean chooseMulligan(Game game) {throw new AssertionError("unexpected mulligan");}
        };
    }
    @Test public void terminal() throws Exception {
        setStrictChooseMode(true);currentGame.setStartingPlayerId(playerA.getId());gameOptions.skipInitShuffling=true;
        removeAllCardsFromHand(playerA);removeAllCardsFromHand(playerB);removeAllCardsFromLibrary(playerA);removeAllCardsFromLibrary(playerB);
        if(fixture.has("library_card"))addCard(Zone.LIBRARY,playerA,"FDN-"+fixture.get("library_card").getAsString(),1);
        setLife(playerA,20);setLife(playerB,20);setStopAt(1,PhaseStep.PRECOMBAT_MAIN);execute();assertEquals(1,calls);
        Files.write(Paths.get(System.getProperty("mtglab.output"),fixture.get("id").getAsString()+".json"),new GsonBuilder().setPrettyPrinting().create().toJson(observed).getBytes(StandardCharsets.UTF_8));
        assertEquals(fixture.getAsJsonObject("expected"),observed);
    }
}
