package org.mage.test.mtglab;

import com.google.gson.*;
import mage.abilities.*;
import mage.abilities.keyword.HasteAbility;
import mage.abilities.costs.mana.ManaCost;
import mage.constants.*;
import mage.game.Game;
import mage.game.permanent.Permanent;
import mage.game.permanent.PermanentImpl;
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

/** Original synthetic positions, CR 602/302.6/702.10/113.7a/611.2.
 * Fixture sickness/departure hooks set the stated position; XMage performs
 * activation, targeting, tap payment, resolution and expiration itself. */
@RunWith(Parameterized.class)
public class HasteTest extends CardTestPlayerBase {
    private final JsonObject spec;
    private final String mode, targetName;
    private boolean initialized, started, captured;
    private JsonObject result;
    @Parameterized.Parameters(name="{0}") public static Collection<Object[]> cases() throws Exception {
        JsonObject f=JsonParser.parseString(new String(Files.readAllBytes(Paths.get(System.getProperty("mtglab.fixture"))),StandardCharsets.UTF_8)).getAsJsonObject();
        assertEquals(1,f.get("version").getAsInt());
        List<Object[]> out=new ArrayList<>();for(JsonElement c:f.getAsJsonArray("cases"))out.add(new Object[]{c.getAsJsonObject()});return out;
    }
    public HasteTest(JsonObject spec){this.spec=spec;mode=spec.get("mode").getAsString();targetName=spec.get("target").getAsString().equals("swab-goblin")?"Swab Goblin":"Bear Cub";}
    private Permanent find(Game g,String name){return g.getBattlefield().getAllActivePermanents().stream().filter(p->p.getName().equals(name)).findFirst().orElse(null);}
    private ActivatedAbility ability(Permanent p){return p.getAbilities().getActivatedAbilities(Zone.BATTLEFIELD).stream().filter(a->a.getRule().contains("haste")).findFirst().get();}
    private void sick(Permanent p){try {java.lang.reflect.Field f=PermanentImpl.class.getDeclaredField("controlledFromStartOfControllerTurn");f.setAccessible(true);f.setBoolean(p,false);}catch(Exception e){throw new AssertionError(e);}}
    private void capture(Game g){
        Permanent p=find(g,"Axgard Cavalry"),t=find(g,targetName);
        result=new JsonObject();result.addProperty("source",p!=null);result.addProperty("source_tapped",p!=null&&p.isTapped());result.addProperty("target",t!=null);result.addProperty("haste",t!=null&&t.getAbilities().containsKey(HasteAbility.getInstance().getId()));result.addProperty("stack",g.getStack().size());
        JsonArray life=new JsonArray();life.add(g.getPlayer(playerA.getId()).getLife());life.add(g.getPlayer(playerB.getId()).getLife());result.add("life",life);
        if(p==null)result.add("source_stats",JsonNull.INSTANCE);else{JsonArray stats=new JsonArray();stats.add(p.getPower().getValue());stats.add(p.getToughness().getValue());result.add("source_stats",stats);}
        captured=true;g.pause();
    }
    @Override protected TestPlayer createPlayer(String n,RangeOfInfluence r){return new TestPlayer(new TestComputerPlayer(n,r)){
        @Override public boolean priority(Game g){
            if(captured)return false;
            Permanent p=find(g,"Axgard Cavalry"),t=find(g,targetName);
            if(g.getTurnNum()==1&&g.getTurnStepType()==PhaseStep.PRECOMBAT_MAIN&&getId().equals(playerA.getId())){
                if(!initialized){initialized=true;sick(t);if(mode.equals("sick"))sick(p);if(mode.equals("tapped"))p.setTapped(true);}
                if(mode.equals("cast")){
                    if(p!=null&&g.getStack().isEmpty()){assertFalse(ability(p).canActivate(getId(),g).canActivate());capture(g);return false;}
                }else if(!started){
                    started=true;
                    if(mode.equals("dead_before")){ActivatedAbility a=ability(p).copy();assertTrue(p.destroy(null,g));assertFalse(a.canActivate(getId(),g).canActivate());capture(g);return false;}
                    if(mode.equals("sick")||mode.equals("tapped")){assertFalse(ability(p).canActivate(getId(),g).canActivate());capture(g);return false;}
                    addTarget(targetName);assertTrue(activateAbility(ability(p),g));assertTrue(p.isTapped());assertEquals(1,g.getStack().size());assertFalse(t.getAbilities().containsKey(HasteAbility.getInstance().getId()));
                    if(mode.equals("source_dies"))assertTrue(p.destroy(null,g));
                    if(mode.equals("target_dies"))assertTrue(t.destroy(null,g));
                    return true;
                }else if(g.getStack().isEmpty()){
                    if(!mode.equals("target_dies"))assertTrue(t.getAbilities().containsKey(HasteAbility.getInstance().getId()));
                    if(!Arrays.asList("cleanup","opposing","cub","swab","source_dies").contains(mode)){capture(g);return false;}
                }
            }
            if((mode.equals("cleanup")||mode.equals("opposing"))&&g.getTurnNum()==2&&g.getTurnStepType()==PhaseStep.UPKEEP){capture(g);return false;}
            if(Arrays.asList("cub","swab","source_dies").contains(mode)&&g.getTurnStepType()==PhaseStep.COMBAT_DAMAGE){capture(g);return false;}
            return super.priority(g);
        }
        @Override public boolean playMana(Ability a,ManaCost unpaid,String prompt,Game g){
            assertEquals("cast",mode);Permanent land=g.getBattlefield().getAllActivePermanents().stream().filter(p->p.isLand(g)&&!p.isTapped()).findFirst().orElse(null);
            if(land!=null){assertTrue(activateAbility(land.getAbilities().getActivatedManaAbilities(Zone.BATTLEFIELD).get(0),g));return true;}
            assertTrue(getManaPool().getRed()>0);getManaPool().unlockManaType(ManaType.RED);return true;
        }
        @Override public void selectAttackers(Game g,UUID active) { if(Arrays.asList("cub","swab","source_dies").contains(mode)){Permanent t=find(g,targetName);assertTrue(t.canAttack(playerB.getId(),g));declareAttacker(t.getId(),playerB.getId(),g,false);} }
        @Override public void selectBlockers(Ability a,Game g,UUID defending) { }
    };}
    @Test public void executeCase() throws Exception {
        setStrictChooseMode(true);currentGame.setStartingPlayerId(playerA.getId());gameOptions.skipInitShuffling=true;
        removeAllCardsFromHand(playerA);removeAllCardsFromHand(playerB);removeAllCardsFromLibrary(playerA);removeAllCardsFromLibrary(playerB);
        addCard(Zone.BATTLEFIELD,mode.equals("opposing")?playerB:playerA,"FDN-"+targetName,1);
        if(mode.equals("cast")){addCard(Zone.HAND,playerA,"FDN-Axgard Cavalry",1);addCard(Zone.BATTLEFIELD,playerA,"FDN-Mountain",2);castSpell(1,PhaseStep.PRECOMBAT_MAIN,playerA,"Axgard Cavalry");}
        else addCard(Zone.BATTLEFIELD,playerA,"FDN-Axgard Cavalry",1);
        setStopAt(2,PhaseStep.UPKEEP);execute();assertTrue("required checkpoint not reached",captured);
        Files.write(Paths.get(System.getProperty("mtglab.output"),spec.get("id").getAsString()+".json"),new GsonBuilder().serializeNulls().setPrettyPrinting().create().toJson(result).getBytes(StandardCharsets.UTF_8));
    }
}
