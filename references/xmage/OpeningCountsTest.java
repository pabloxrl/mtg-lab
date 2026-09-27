package org.mage.test.mtglab;

import com.google.gson.*;
import mage.abilities.Ability;
import mage.constants.*;
import mage.game.Game;
import mage.target.Target;
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

/** Original CR 103.5 count ledger; synthetic seven-card hands, real mulligan phase. */
@RunWith(Parameterized.class)
public class OpeningCountsTest extends CardTestPlayerBase {
    private final JsonObject fixture;
    private final JsonArray declarations = new JsonArray();
    private int round, bottoms, priorityCalls;
    private JsonObject observed;
    @Parameterized.Parameters(name="{index}") public static Collection<Object[]> cases() throws Exception {
        JsonObject doc=JsonParser.parseString(new String(Files.readAllBytes(Paths.get(System.getProperty("mtglab.fixture"))),StandardCharsets.UTF_8)).getAsJsonObject();
        List<Object[]> out=new ArrayList<>();
        for(JsonElement c:doc.getAsJsonArray("cases")) out.add(new Object[]{c.getAsJsonObject()});
        return out;
    }
    public OpeningCountsTest(JsonObject f) {fixture=f;}
    private int seat(UUID id) {return id.equals(playerA.getId()) ? 0 : 1;}
    @Override protected TestPlayer createPlayer(String name, RangeOfInfluence range) {
        return new TestPlayer(new TestComputerPlayer(name,range)) {
            // Explicit chance hook: retain the supplied all-basic permutation.
            // No independently seeded reference shuffle is part of this comparison.
            @Override public void shuffleLibrary(Ability source, Game game) { }
            @Override public boolean chooseMulligan(Game game) {
                assertTrue("unbounded declaration",declarations.size()<10);
                JsonArray row=new JsonArray(); row.add(seat(getId())); row.add(getHand().size()); row.add(getLibrary().size()); declarations.add(row);
                if(seat(getId()) != fixture.get("starter").getAsInt()) return false;
                return round++ < fixture.get("rounds").getAsInt();
            }
            @Override public boolean chooseTarget(Outcome outcome, Target target, Ability source, Game game) {
                assertEquals("only mulligan bottom choices",fixture.get("starter").getAsInt(),seat(getId()));
                assertTrue(bottoms++<28);
                assertEquals(1,target.getMinNumberOfTargets());
                target.addTarget(getHand().iterator().next(),source,game);
                assertEquals(1,target.getTargets().size());return true;
            }
            @Override public boolean priority(Game game) {
                assertEquals(0,priorityCalls++);
                assertEquals(fixture.get("starter").getAsInt(),seat(getId()));
                assertEquals(PhaseStep.UPKEEP,game.getTurnStepType());
                observed=new JsonObject(); observed.add("declarations",declarations);
                JsonArray hand=new JsonArray(),library=new JsonArray();
                for(UUID id:Arrays.asList(playerA.getId(),playerB.getId())) {
                    hand.add(game.getPlayer(id).getHand().size()); library.add(game.getPlayer(id).getLibrary().size());
                    assertEquals(20,game.getPlayer(id).getLife());
                }
                observed.add("hand",hand);observed.add("library",library);observed.addProperty("bottom_choices",bottoms);
                game.pause();return false;
            }
        };
    }
    @Test public void openingCounts() throws Exception {
        setStrictChooseMode(true);gameOptions.skipInitShuffling=true;
        currentGame.setStartingPlayerId(fixture.get("starter").getAsInt()==0 ? playerA.getId() : playerB.getId());
        // The upstream test constructor skips initial draws. Inject that explicit
        // prefix only; its real London phase performs declarations/redraw/bottom.
        removeAllCardsFromHand(playerA);removeAllCardsFromHand(playerB);
        removeAllCardsFromLibrary(playerA);removeAllCardsFromLibrary(playerB);
        addCard(Zone.HAND,playerA,"FDN-Forest",7);addCard(Zone.LIBRARY,playerA,"FDN-Forest",33);
        addCard(Zone.HAND,playerB,"FDN-Mountain",7);addCard(Zone.LIBRARY,playerB,"FDN-Mountain",33);
        setLife(playerA,20);setLife(playerB,20);setStopAt(1,PhaseStep.PRECOMBAT_MAIN);execute();
        assertEquals(1,priorityCalls);
        Files.write(Paths.get(System.getProperty("mtglab.output"),fixture.get("id").getAsString()+".json"),new GsonBuilder().setPrettyPrinting().create().toJson(observed).getBytes(StandardCharsets.UTF_8));
        assertEquals(fixture.getAsJsonObject("expected"),observed);
    }
}
