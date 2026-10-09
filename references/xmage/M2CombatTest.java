package org.mage.test.mtglab;

import com.google.gson.*;
import mage.abilities.*;
import mage.abilities.costs.mana.ManaCost;
import mage.abilities.keyword.TrampleAbility;
import mage.abilities.keyword.FlyingAbility;
import mage.abilities.keyword.ReachAbility;
import mage.abilities.keyword.DeathtouchAbility;
import mage.constants.*;
import mage.game.Game;
import mage.game.permanent.Permanent;
import mage.util.MultiAmountMessage;
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

/** Original composition scripts using the same strict upstream operations as
 * InvokerTest and ShivanTest. Only the initial position and blocker departure
 * are synthetic. All modifiers, costs, damage and cleanup are computed by XMage.
 * Independent expectations live outside this executable bridge. */
@RunWith(Parameterized.class)
public class M2CombatTest extends CardTestPlayerBase {
    private final JsonObject spec;
    private final String id, targetName, blockerName;
    private final int boostCount, growthCount, blockerCount;
    private final boolean departure;
    private final JsonObject points = new JsonObject();
    private final List<UUID> blockers = new ArrayList<>();
    private UUID target, dragon;
    private boolean dragonBoosted, pendingDragon;
    private int boosts, growths, priorityCalls, manaCalls, assignments;
    private boolean initialized, pendingBoost, pendingGrowth, left, captured;

    @Parameterized.Parameters(name="{0}")
    public static Collection<Object[]> cases() throws Exception {
        JsonObject f = JsonParser.parseString(new String(Files.readAllBytes(Paths.get(System.getProperty("mtglab.fixture"))),StandardCharsets.UTF_8)).getAsJsonObject();
        assertEquals(1, f.get("version").getAsInt());
        List<Object[]> out = new ArrayList<>();
        for (JsonElement c : f.getAsJsonArray("cases")) out.add(new Object[]{c.getAsJsonObject()});
        return out;
    }
    public M2CombatTest(JsonObject s) {
        spec=s; id=s.get("id").getAsString();
        targetName=name(s.get("target").getAsString()); blockerName=name(s.get("blocker").getAsString());
        boostCount=s.get("boosts").getAsInt(); growthCount=s.get("growths").getAsInt();
        blockerCount=s.get("blockers").getAsInt(); departure=s.get("departure").getAsBoolean();
    }
    private static String name(String key) {
        switch(key) {
            case "thornweald-archer": return "Thornweald Archer";
            case "bear-cub": return "Bear Cub";
            case "magnigoth-sentry": return "Magnigoth Sentry";
            default: throw new AssertionError("unsupported fixture card " + key);
        }
    }
    private Permanent find(Game g, String name, UUID controller) {
        return g.getBattlefield().getAllActivePermanents().stream()
            .filter(p->p.getName().equals(name)&&p.getControllerId().equals(controller)).findFirst().orElse(null);
    }
    private JsonElement stats(Permanent p) {
        if(p==null)return JsonNull.INSTANCE;
        JsonArray a=new JsonArray();a.add(p.getPower().getValue());a.add(p.getToughness().getValue());a.add(p.getDamage());return a;
    }
    private void point(String label, Game g) {
        assertFalse("duplicate checkpoint "+label,points.has(label));
        JsonObject p=new JsonObject(); Permanent t=g.getPermanent(target);
        p.add("target",stats(t)); JsonArray bs=new JsonArray();
        for(UUID b:blockers)bs.add(stats(g.getPermanent(b)));p.add("blockers",bs);
        p.addProperty("trample",t!=null&&t.getAbilities().containsKey(TrampleAbility.getInstance().getId()));
        JsonArray life=new JsonArray();life.add(g.getPlayer(playerA.getId()).getLife());life.add(g.getPlayer(playerB.getId()).getLife());p.add("life",life);
        p.addProperty("mana",g.getPlayer(playerA.getId()).getManaPool().count());p.addProperty("stack",g.getStack().size());
        if (id.equals("cleanup-shivan-thorn")) {
            Permanent d = g.getPermanent(dragon);
            p.add("dragon", stats(d));
            p.addProperty("flying", d.getAbilities().containsKey(FlyingAbility.getInstance().getId()));
            p.addProperty("reach", t.getAbilities().containsKey(ReachAbility.getInstance().getId()));
            p.addProperty("deathtouch", t.getAbilities().containsKey(DeathtouchAbility.getInstance().getId()));
        }
        points.add(label,p);
    }
    @Override protected TestPlayer createPlayer(String n,RangeOfInfluence r) {
        return new TestPlayer(new TestComputerPlayer(n,r)) {
            @Override public boolean priority(Game g) {
                assertTrue("bounded priority: "+id,++priorityCalls<500);
                if(captured)return false;
                if(g.getTurnNum()==2&&g.getTurnStepType()==PhaseStep.UPKEEP) {
                    point("after-cleanup",g);captured=true;g.pause();return false;
                }
                if(!getId().equals(playerA.getId())){pass(g);return false;}
                boolean main=g.getTurnNum()==1&&g.getTurnStepType()==PhaseStep.PRECOMBAT_MAIN;
                boolean blocked=g.getTurnNum()==1&&g.getTurnStepType()==PhaseStep.DECLARE_BLOCKERS;
                if(main&&!initialized) {
                    if(getManaPool().count()<boostCount*8+growthCount+(id.equals("cleanup-shivan-thorn")?1:0)) {
                        Permanent land=g.getBattlefield().getAllActivePermanents().stream().filter(p->p.getControllerId().equals(getId())&&p.isLand(g)&&!p.isTapped()).findFirst().orElseThrow(()->new AssertionError("missing scripted object/ability: "+id));
                        assertTrue(activateAbility(land.getAbilities().getActivatedManaAbilities(Zone.BATTLEFIELD).get(0),g));return true;
                    }
                    target=find(g,targetName,getId()).getId();
                    if (id.equals("cleanup-shivan-thorn")) dragon=find(g,"Shivan Dragon",getId()).getId();
                    for(Permanent p:g.getBattlefield().getAllActivePermanents())if(p.getControllerId().equals(playerB.getId())&&p.isCreature(g))blockers.add(p.getId());
                    assertEquals(blockerCount,blockers.size());initialized=true;point("initial",g);
                }
                if(main&&pendingBoost&&g.getStack().isEmpty()){pendingBoost=false;point("activation-"+boosts+"-resolved",g);}
                if((main||blocked)&&pendingGrowth&&g.getStack().isEmpty()){pendingGrowth=false;point("growth-"+growths,g);}
                if(main&&g.getStack().isEmpty()&&boosts<boostCount) {
                    Permanent source=find(g,"Wildheart Invoker",getId());
                    ActivatedAbility a=source.getAbilities().getActivatedAbilities(Zone.BATTLEFIELD).stream().filter(x->x.getRule().contains("+5/+5")).findFirst().orElseThrow(()->new AssertionError("missing scripted object/ability: "+id));
                    addTarget(targetName);assertTrue(activateAbility(a,g));boosts++;pendingBoost=true;
                    point("activation-"+boosts+"-stack",g);return true;
                }
                if(main&&pendingDragon&&g.getStack().isEmpty()){pendingDragon=false;point("dragon-resolved",g);}
                if(main&&id.equals("cleanup-shivan-thorn")&&boosts==boostCount&&g.getStack().isEmpty()&&!dragonBoosted) {
                    ActivatedAbility a=g.getPermanent(dragon).getAbilities().getActivatedAbilities(Zone.BATTLEFIELD).stream().filter(x->x.getRule().contains("+1/+0")).findFirst().orElseThrow(()->new AssertionError("missing scripted object/ability: "+id));
                    assertTrue(activateAbility(a,g));dragonBoosted=true;pendingDragon=true;point("dragon-stack",g);return true;
                }
                if(blocked&&departure&&!left) {
                    assertTrue(g.getPermanent(blockers.get(0)).destroy(null,g));left=true;
                    Permanent land=g.getBattlefield().getAllActivePermanents().stream().filter(p->p.getControllerId().equals(getId())&&p.isLand(g)&&!p.isTapped()).findFirst().orElseThrow(()->new AssertionError("missing scripted object/ability: "+id));
                    assertTrue(activateAbility(land.getAbilities().getActivatedManaAbilities(Zone.BATTLEFIELD).get(0),g));return true;
                }
                if((main&&!departure||blocked&&departure)&&g.getStack().isEmpty()&&boosts==boostCount&&growths<growthCount) {
                    UUID spell=getHand().stream().filter(h->g.getCard(h).getName().equals("Giant Growth")).findFirst().orElseThrow(()->new AssertionError("missing scripted object/ability: "+id));
                    addTarget(targetName);assertTrue(activateAbility(g.getCard(spell).getSpellAbility(),g));growths++;pendingGrowth=true;return true;
                }
                if(blocked&&g.getStack().isEmpty()&&growths==growthCount&&!points.has("before-damage"))point("before-damage",g);
                if(blockerCount>0&&g.getTurnStepType()==PhaseStep.COMBAT_DAMAGE&&!points.has("after-damage"))point("after-damage",g);
                pass(g);return false;
            }
            @Override public boolean playMana(Ability a,ManaCost unpaid,String prompt,Game g) {
                assertTrue("bounded payment",++manaCalls<100);
                if (dragon != null && dragon.equals(a.getSourceId())) {
                    assertTrue(getManaPool().getRed()>0);getManaPool().unlockManaType(ManaType.RED);
                } else {
                    assertTrue(getManaPool().getGreen()>0);getManaPool().unlockManaType(ManaType.GREEN);
                }
                return true;
            }
            @Override public void selectAttackers(Game g,UUID active) {
                if(blockerCount>0){assertTrue(g.getPermanent(target).canAttack(playerB.getId(),g));declareAttacker(target,playerB.getId(),g,false);}
            }
            @Override public void selectBlockers(Ability a,Game g,UUID defending) {
                for(UUID b:blockers){assertTrue(g.getPermanent(b).canBlock(target,g));declareBlocker(getId(),b,target,g);}
            }
            @Override public List<Integer> getMultiAmountWithIndividualConstraints(Outcome o,List<MultiAmountMessage> messages,int min,int max,MultiAmountType type,Game g) {
                assertEquals(0,assignments++);assertFalse(departure);assertEquals(2,messages.size());assertEquals(2,min);
                assertEquals(boostCount*5+growthCount*3+2,max);
                // 0+1 is below XMage's required lethal sum; then execute 1+1.
                if(id.equals("invoker-cubs-reject"))assertTrue(0+1<min);
                for(MultiAmountMessage m:messages)assertTrue(m.min<=1&&m.max>=1);
                return Arrays.asList(1,1);
            }
        };
    }
    @Test public void executeCase() throws Exception {
        setStrictChooseMode(true);currentGame.setStartingPlayerId(playerA.getId());gameOptions.skipInitShuffling=true;
        removeAllCardsFromHand(playerA);removeAllCardsFromHand(playerB);removeAllCardsFromLibrary(playerA);removeAllCardsFromLibrary(playerB);
        addCard(Zone.LIBRARY,playerA,"FDN-Forest",2);addCard(Zone.LIBRARY,playerB,"FDN-Forest",2);
        addCard(Zone.BATTLEFIELD,playerA,"FDN-"+targetName,1);addCard(Zone.BATTLEFIELD,playerA,"FDN-Wildheart Invoker",1);
        if(id.equals("cleanup-shivan-thorn")) {
            addCard(Zone.BATTLEFIELD,playerA,"FDN-Shivan Dragon",1);
            addCard(Zone.BATTLEFIELD,playerA,"FDN-Mountain",1);
        }
        if(blockerCount>0)addCard(Zone.BATTLEFIELD,playerB,"FDN-"+blockerName,blockerCount);
        addCard(Zone.BATTLEFIELD,playerA,"FDN-Forest",boostCount*8+growthCount+(departure?1:0));
        if(growthCount>0)addCard(Zone.HAND,playerA,"FDN-Giant Growth",growthCount);
        setStopAt(2,PhaseStep.UPKEEP);execute();assertTrue("cleanup checkpoint not reached",captured);
        assertEquals("activation script consumed",boostCount,boosts);
        assertEquals("Growth script consumed",growthCount,growths);
        assertFalse("no pending resolution",pendingBoost||pendingGrowth||pendingDragon);
        assertEquals("damage choices consumed",blockerCount==2?1:0,assignments);
        if(id.equals("cleanup-shivan-thorn"))assertTrue(dragonBoosted);
        Files.write(Paths.get(System.getProperty("mtglab.output"),id+".json"),new GsonBuilder().serializeNulls().setPrettyPrinting().create().toJson(points).getBytes(StandardCharsets.UTF_8));
    }
}
