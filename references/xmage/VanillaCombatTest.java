package org.mage.test.mtglab;

import com.google.gson.*;
import mage.abilities.Ability;
import mage.constants.*;
import mage.game.Game;
import mage.game.combat.CombatGroup;
import mage.game.permanent.Permanent;
import mage.util.MultiAmountMessage;
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

/** Original test-only combat bridge: explicit declarations, allocation and passes.
 * No upstream scenario copied; no production mtg-lab code used. */
@RunWith(Parameterized.class)
public class VanillaCombatTest extends CardTestPlayerBase {
    private final JsonObject fixture;
    private final JsonArray checkpoints = new JsonArray();
    private int passes, attackerCalls, blockerCalls, allocationCalls;
    private static JsonObject document() throws Exception {
        return JsonParser.parseString(new String(Files.readAllBytes(Paths.get(System.getProperty("mtglab.fixture"))),StandardCharsets.UTF_8)).getAsJsonObject();
    }
    @Parameterized.Parameters(name="{index}") public static Collection<Object[]> cases() throws Exception {
        List<Object[]> result = new ArrayList<>();
        for (JsonElement e : document().getAsJsonArray("cases")) result.add(new Object[]{e.getAsJsonObject()});
        return result;
    }
    public VanillaCombatTest(JsonObject fixture) { this.fixture = fixture; }
    private String attacker() { return fixture.get("attacker").getAsString(); }
    private Permanent permanent(Game game, int seat, String name) {
        UUID controller = seat == 0 ? playerA.getId() : playerB.getId();
        return game.getBattlefield().getAllActivePermanents().stream()
            .filter(p -> p.getControllerId().equals(controller) && p.getName().equals(name))
            .findFirst().orElseThrow(() -> new AssertionError("missing " + name));
    }
    @Override protected TestPlayer createPlayer(String name, RangeOfInfluence range) {
        return new TestPlayer(new TestComputerPlayer(name, range)) {
            @Override public boolean priority(Game game) {
                int seat = getId().equals(playerA.getId()) ? 0 : 1;
                assertEquals(1, game.getTurnNum());
                if (game.getTurnStepType() == PhaseStep.COMBAT_DAMAGE) {
                    assertEquals(0, seat);
                    checkpoints.add(checkpoint("damage", game));
                    game.pause(); return false;
                }
                try {
                    JsonArray steps = document().getAsJsonArray("priority_pass_steps");
                    assertTrue("extra priority decision", passes < steps.size()*2);
                    assertEquals(steps.get(passes/2).getAsString(), game.getTurnStepType().name());
                    assertEquals(passes%2, seat);
                } catch (Exception e) { throw new AssertionError(e); }
                if (seat == 0) {
                    if (game.getTurnStepType() == PhaseStep.UPKEEP) checkpoints.add(checkpoint("initial",game));
                    if (game.getTurnStepType() == PhaseStep.DECLARE_ATTACKERS) checkpoints.add(checkpoint("attackers",game));
                    if (game.getTurnStepType() == PhaseStep.DECLARE_BLOCKERS) checkpoints.add(checkpoint("blockers",game));
                }
                passes++; pass(game); return true;
            }
            @Override public void selectAttackers(Game game, UUID attackingPlayerId) {
                assertEquals(playerA.getId(), getId()); assertEquals(0, attackerCalls++);
                Permanent a = permanent(game,0,attacker());
                assertTrue(a.canAttack(playerB.getId(),game));
                declareAttacker(a.getId(),playerB.getId(),game,false);
            }
            @Override public void selectBlockers(Ability source, Game game, UUID defendingPlayerId) {
                assertEquals(playerB.getId(),getId()); assertEquals(0,blockerCalls++);
                for (JsonElement b : fixture.getAsJsonArray("blockers")) {
                    declareBlocker(getId(),permanent(game,1,b.getAsString()).getId(),permanent(game,0,attacker()).getId(),game);
                }
            }
            @Override public List<Integer> getMultiAmountWithIndividualConstraints(Outcome outcome,
                    List<MultiAmountMessage> messages, int totalMin, int totalMax, MultiAmountType type, Game game) {
                assertEquals(playerA.getId(),getId()); assertEquals(0,allocationCalls++);
                assertEquals(2,totalMin); assertEquals(2,totalMax);
                assertEquals(fixture.getAsJsonArray("blockers").size(),messages.size());
                List<Integer> result = new ArrayList<>();
                for (MultiAmountMessage m : messages) {
                    int found = -1;
                    for (int i=0; i<fixture.getAsJsonArray("blockers").size(); i++) {
                        if (m.message.contains(fixture.getAsJsonArray("blockers").get(i).getAsString())) {
                            assertEquals(-1,found); found=i;
                        }
                    }
                    assertTrue("unknown allocation recipient",found>=0);
                    int amount=fixture.getAsJsonArray("amounts").get(found).getAsInt();
                    assertTrue(amount>=m.min && amount<=m.max); result.add(amount);
                }
                return result;
            }
            @Override public boolean chooseMulligan(Game game) { throw new AssertionError("unexpected mulligan"); }
        };
    }
    private JsonObject checkpoint(String name,Game game) {
        JsonObject c=new JsonObject(); c.addProperty("name",name);
        JsonArray life=new JsonArray(), battlefield=new JsonArray(), graves=new JsonArray();
        for (int seat=0;seat<2;seat++) {
            UUID id=seat==0?playerA.getId():playerB.getId(); Player p=game.getPlayer(id);
            life.add(p.getLife()); JsonArray grave=new JsonArray();
            List<String> names=new ArrayList<>();
            for (UUID h:p.getGraveyard()) names.add(game.getCard(h).getName());
            Collections.sort(names); for(String n:names) grave.add(n); graves.add(grave);
            List<Permanent> permanents=new ArrayList<>();
            for(Permanent o:game.getBattlefield().getAllActivePermanents()) if(o.getControllerId().equals(id)) permanents.add(o);
            permanents.sort(Comparator.comparing(Permanent::getName));
            for(Permanent o:permanents) {
                JsonObject b=new JsonObject(); b.addProperty("seat",seat); b.addProperty("card",o.getName());
                b.addProperty("tapped",o.isTapped()); b.addProperty("damage",o.getDamage());
                b.addProperty("power",o.getPower().getValue()); b.addProperty("toughness",o.getToughness().getValue()); battlefield.add(b);
            }
        }
        c.add("life",life);c.add("battlefield",battlefield);c.add("graveyard",graves);
        if(name.equals("attackers") || name.equals("blockers")) {
            assertEquals(1,game.getCombat().getGroups().size());
            CombatGroup group=game.getCombat().getGroups().get(0);
            c.addProperty("blocked",group.getBlocked());
            List<String> names=new ArrayList<>(); for(UUID b:group.getBlockers()) names.add(game.getPermanent(b).getName());
            Collections.sort(names); JsonArray bs=new JsonArray();for(String b:names)bs.add(b);c.add("blockers",bs);
        }
        assertTrue(game.getStack().isEmpty()); assertEquals(playerA.getId(),game.getActivePlayerId());
        assertEquals(playerA.getId(),game.getPriorityPlayerId()); return c;
    }
    @Test public void combat() throws Exception {
        setStrictChooseMode(true); currentGame.setStartingPlayerId(playerA.getId());gameOptions.skipInitShuffling=true;
        removeAllCardsFromHand(playerA);removeAllCardsFromHand(playerB);
        removeAllCardsFromLibrary(playerA);removeAllCardsFromLibrary(playerB);
        addCard(Zone.LIBRARY,playerA,"FDN-Forest",1);addCard(Zone.LIBRARY,playerB,"FDN-Mountain",1);
        addCard(Zone.BATTLEFIELD,playerA,"FDN-"+attacker(),1);
        for(JsonElement b:fixture.getAsJsonArray("blockers"))addCard(Zone.BATTLEFIELD,playerB,"FDN-"+b.getAsString(),1);
        setLife(playerA,20);setLife(playerB,20);setStopAt(1,PhaseStep.END_COMBAT);execute();
        assertEquals(10,passes);assertEquals(1,attackerCalls);assertEquals(1,blockerCalls);
        assertEquals(fixture.getAsJsonArray("blockers").size()>1?1:0,allocationCalls);
        Files.write(Paths.get(System.getProperty("mtglab.output"),fixture.get("id").getAsString()+".json"),new GsonBuilder().setPrettyPrinting().create().toJson(checkpoints).getBytes(StandardCharsets.UTF_8));
        assertEquals(fixture.getAsJsonArray("expected"),checkpoints);
    }
}
