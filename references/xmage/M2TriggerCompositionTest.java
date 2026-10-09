package org.mage.test.mtglab;

import com.google.gson.*;
import mage.abilities.*;
import mage.abilities.costs.mana.ManaCost;
import mage.cards.Card;
import mage.constants.*;
import mage.game.Game;
import mage.game.permanent.Permanent;
import mage.game.stack.StackObject;
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

/** Original composition of existing cast/trigger/departure bridge operations.
 * Synthetic positions; pending APNAP injection is explicitly test-only.
 * No expected ledger is read here. Stack labels retain physical source identity. */
@RunWith(Parameterized.class)
public class M2TriggerCompositionTest extends CardTestPlayerBase {
    private final JsonObject spec;
    private final JsonArray points = new JsonArray();
    private final Map<UUID, String> sources = new LinkedHashMap<>();
    private final Map<String, UUID> creatures = new LinkedHashMap<>();
    private boolean captured;
    private int calls, targetCalls, manaCalls;
    @Parameterized.Parameters(name="{0}") public static Collection<Object[]> cases() throws Exception {
        JsonObject f = JsonParser.parseString(new String(Files.readAllBytes(Paths.get(System.getProperty("mtglab.fixture"))), StandardCharsets.UTF_8)).getAsJsonObject();
        assertEquals(1, f.get("version").getAsInt());
        List<Object[]> out = new ArrayList<>();
        for (JsonElement c : f.getAsJsonArray("cases")) out.add(new Object[]{c.getAsJsonObject()});
        return out;
    }
    public M2TriggerCompositionTest(JsonObject s) { spec = s; }
    private String kind() { return spec.get("kind").getAsString(); }
    private TestPlayer seat(int i) { return i == 0 ? playerA : playerB; }
    private UUID alias(int i, String name) { return seat(i).getAliasByName(kind().equals("apnap") ? "p" + i + "-" + name : name); }
    private void label(int i, String alias, String label) { sources.put(alias(i, alias), label); }
    private JsonObject point(Game g) {
        JsonObject p = new JsonObject(), stats = new JsonObject();
        JsonArray life = new JsonArray(), lost = new JsonArray(), stack = new JsonArray();
        for (int i = 0; i < 2; i++) {
            life.add(g.getPlayer(seat(i).getId()).getLife());
            lost.add(g.getPlayer(seat(i).getId()).hasLost());
        }
        List<String> topFirst = new ArrayList<>();
        for (StackObject s : g.getStack()) {
            UUID source = s.getStackAbility().getSourceId();
            if (sources.containsKey(source)) topFirst.add("trigger:" + sources.get(source));
            else {
                Card c = g.getCard(source);
                assertNotNull(c);
                String key;
                switch (c.getName()) {
                    case "Giant Growth": key = "giant-growth"; break;
                    case "Bite Down": key = "bite-down"; break;
                    case "Dragon Fodder": key = "dragon-fodder"; break;
                    default: throw new AssertionError("unexpected stack source " + c.getName());
                }
                topFirst.add("spell:" + key);
            }
        }
        Collections.reverse(topFirst);
        for (String s : topFirst) stack.add(s);
        for (Map.Entry<String, UUID> entry : creatures.entrySet()) {
            Permanent c = g.getPermanent(entry.getValue());
            if (c == null) stats.add(entry.getKey(), JsonNull.INSTANCE);
            else {
                JsonArray values = new JsonArray();
                values.add(c.getPower().getValue()); values.add(c.getToughness().getValue()); values.add(c.getDamage());
                stats.add(entry.getKey(), values);
            }
        }
        p.add("life", life); p.add("lost", lost); p.add("stack", stack); p.add("creatures", stats);
        p.addProperty("goblins", g.getBattlefield().getAllActivePermanents().stream().filter(x -> x.isToken() && x.getName().equals("Goblin Token")).count());
        return p;
    }
    @Override protected TestPlayer createPlayer(String n, RangeOfInfluence r) {
        return new TestPlayer(new TestComputerPlayer(n,r)) {
            @Override public boolean priority(Game g) {
                assertTrue("bounded scripted priority", ++calls < 150);
                if (captured) return false;
                int active = spec.get("active").getAsInt();
                if (g.getTurnNum() != 1 || g.getTurnStepType() != PhaseStep.PRECOMBAT_MAIN || !getId().equals(seat(active).getId())) { pass(g); return false; }
                if (kind().equals("apnap")) {
                    for (int i = 0; i < 2; i++) for (int row = 0; row < spec.get("per_seat").getAsInt(); row++) {
                        label(i, "archer" + row, "p" + i + "-archer-" + row);
                        Permanent c = g.getPermanent(alias(i, "archer" + row));
                        g.getState().addTriggeredAbility(c.getAbilities().getTriggeredAbilities(Zone.BATTLEFIELD).get(0).copy());
                    }
                } else {
                    label(0, "archer", "p0-archer");
                    if (!kind().equals("lethal_fodder")) {
                        label(0, "cyclops", "p0-cyclops"); creatures.put("cyclops", alias(0, "cyclops"));
                    }
                    if (kind().equals("growth_cub")) creatures.put("cub", alias(0, "cub"));
                    if (kind().startsWith("cyclops_bite")) creatures.put("enemy", alias(1, "enemy"));
                    if (kind().equals("holdout")) {
                        label(0, "archer-b", "p0-archer-b");
                        creatures.put("archer", alias(0, "archer")); creatures.put("archer-b", alias(0, "archer-b"));
                    }
                    for (Permanent land : g.getBattlefield().getAllActivePermanents()) if (land.isLand(g) && land.getControllerId().equals(getId())) assertTrue(activateAbility(land.getAbilities().getActivatedManaAbilities(Zone.BATTLEFIELD).get(0),g));
                    assertEquals(1, getHand().size());
                    Card c = getHand().getCards(g).iterator().next();
                    assertTrue(cast(c.getSpellAbility(),g,false,null));
                }
                if (spec.has("dead_sources") && spec.get("dead_sources").getAsBoolean()) {
                    for (UUID source : sources.keySet()) assertTrue(g.getPermanent(source).destroy(null,g));
                }
                g.checkStateAndTriggered();
                if (kind().equals("holdout")) {
                    assertTrue(g.getPermanent(alias(0,"cyclops")).destroy(null,g)); g.checkStateAndTriggered();
                }
                points.add(point(g));
                int resolutions = 0;
                while (!g.getStack().isEmpty() && !g.getPlayer(playerA.getId()).hasLost() && !g.getPlayer(playerB.getId()).hasLost()) {
                    assertTrue(++resolutions < 8);
                    g.getStack().resolve(g); g.checkStateAndTriggered(); points.add(point(g));
                }
                captured = true; g.pause(); return false;
            }
            @Override public TriggeredAbility chooseTriggeredAbility(List<TriggeredAbility> abilities, Game g) {
                List<UUID> order = new ArrayList<>();
                if (kind().equals("apnap")) {
                    int i = getId().equals(playerA.getId()) ? 0 : 1;
                    int position = i == spec.get("active").getAsInt() ? 0 : 1;
                    for (JsonElement row : spec.getAsJsonArray("orders").get(position).getAsJsonArray()) order.add(alias(i,"archer" + row.getAsInt()));
                } else {
                    assertEquals(playerA.getId(),getId());
                    order.add(alias(0,"archer"));
                    if (!kind().equals("lethal_fodder")) order.add(alias(0,"cyclops"));
                    if (kind().equals("holdout")) order.add(alias(0,"archer-b"));
                }
                for (UUID source : order) for (TriggeredAbility a : abilities) if (a.getSourceId().equals(source)) return a;
                throw new AssertionError("unscripted trigger choice");
            }
            @Override public boolean chooseTarget(Outcome outcome, Target t, Ability a, Game g) {
                assertTrue(++targetCalls <= (kind().startsWith("cyclops_bite") ? 2 : 1));
                UUID selected = kind().equals("growth_cub") ? alias(0,"cub") : kind().equals("holdout") || targetCalls == 1 ? alias(0,"cyclops") : alias(1,"enemy");
                assertTrue(t.canTarget(selected,a,g)); t.addTarget(selected,a,g); return true;
            }
            @Override public boolean playMana(Ability a, ManaCost unpaid, String prompt, Game g) {
                assertTrue(++manaCalls < 8);
                assertTrue(getManaPool().getRed() > 0 || getManaPool().getGreen() > 0);
                if (getManaPool().getRed() > 0) getManaPool().unlockManaType(ManaType.RED);
                if (getManaPool().getGreen() > 0) getManaPool().unlockManaType(ManaType.GREEN);
                return true;
            }
        };
    }
    @Test public void executeCase() throws Exception {
        assertEquals("synthetic",spec.get("setup").getAsString());
        setStrictChooseMode(true); currentGame.setStartingPlayerId(seat(spec.get("active").getAsInt()).getId()); gameOptions.skipInitShuffling = true;
        for (TestPlayer p : Arrays.asList(playerA,playerB)) {
            removeAllCardsFromHand(p); removeAllCardsFromLibrary(p); addCard(Zone.LIBRARY,p,"FDN-Forest",3);
        }
        if (kind().equals("apnap")) {
            for (int i = 0; i < 2; i++) for (int row = 0; row < spec.get("per_seat").getAsInt(); row++) addCard(Zone.BATTLEFIELD,seat(i),"FDN-Firebrand Archer@p"+i+"-archer"+row,1);
        } else {
            addCard(Zone.BATTLEFIELD,playerA,"FDN-Firebrand Archer@archer",1);
            if (!kind().equals("lethal_fodder")) addCard(Zone.BATTLEFIELD,playerA,"FDN-Crackling Cyclops@cyclops",1);
            if (kind().equals("growth_cub")) addCard(Zone.BATTLEFIELD,playerA,"FDN-Bear Cub@cub",1);
            if (kind().startsWith("cyclops_bite")) addCard(Zone.BATTLEFIELD,playerB,kind().equals("cyclops_bite_survivor") ? "FDN-Magnigoth Sentry@enemy" : "FDN-Bear Cub@enemy",1);
            if (kind().equals("holdout")) addCard(Zone.BATTLEFIELD,playerA,"FDN-Firebrand Archer@archer-b",1);
            boolean fodder = kind().equals("lethal_fodder"), bite = kind().startsWith("cyclops_bite");
            addCard(Zone.HAND,playerA,"FDN-" + (fodder ? "Dragon Fodder" : bite ? "Bite Down" : "Giant Growth"),1);
            addCard(Zone.BATTLEFIELD,playerA,fodder ? "FDN-Mountain" : "FDN-Forest",fodder || bite ? 2 : 1);
            if (fodder) setLife(playerB,1);
        }
        setStopAt(2,PhaseStep.PRECOMBAT_MAIN); execute(); assertTrue("checkpoint not reached",captured);
        Files.write(Paths.get(System.getProperty("mtglab.output"),spec.get("id").getAsString()+".json"),new GsonBuilder().serializeNulls().setPrettyPrinting().create().toJson(points).getBytes(StandardCharsets.UTF_8));
    }
}
