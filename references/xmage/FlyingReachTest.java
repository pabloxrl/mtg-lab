package org.mage.test.mtglab;

import com.google.gson.*;
import mage.abilities.Ability;
import mage.abilities.keyword.FlyingAbility;
import mage.abilities.mana.ActivatedManaAbilityImpl;
import mage.abilities.costs.mana.ManaCost;
import mage.constants.*;
import mage.counters.CounterType;
import mage.game.Game;
import mage.game.permanent.Permanent;
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

/** Original synthetic fixture; pinned CR 509/510/702.9/702.17.
 * A test-only 5/5 flying Cub is NOT evidence of Shivan support. */
@RunWith(Parameterized.class)
public class FlyingReachTest extends CardTestPlayerBase {
    private final JsonObject spec;
    private final String mode;
    private boolean injected, captured, growthMana, legal=true;
    private JsonObject result;
    @Parameterized.Parameters(name="{0}") public static Collection<Object[]> cases() throws Exception {
        JsonObject f=JsonParser.parseString(new String(Files.readAllBytes(Paths.get(System.getProperty("mtglab.fixture"))),StandardCharsets.UTF_8)).getAsJsonObject();
        assertEquals(1,f.get("version").getAsInt());
        List<Object[]> out=new ArrayList<>();for(JsonElement c:f.getAsJsonArray("cases"))out.add(new Object[]{c.getAsJsonObject()});return out;
    }
    public FlyingReachTest(JsonObject spec){this.spec=spec;mode=spec.get("mode").getAsString();}
    private Permanent find(Game g,int seat){UUID id=seat==0?playerA.getId():playerB.getId();return g.getBattlefield().getAllActivePermanents().stream().filter(p->p.getControllerId().equals(id)&&p.isCreature(g)).findFirst().orElse(null);}
    private JsonElement state(Permanent p){if(p==null)return JsonNull.INSTANCE;JsonArray a=new JsonArray();a.add(p.getPower().getValue());a.add(p.getToughness().getValue());a.add(p.getDamage());return a;}
    private void capture(Game g){result=new JsonObject();result.add("attacker",state(find(g,0)));result.add("blocker",state(find(g,1)));JsonArray life=new JsonArray();life.add(g.getPlayer(playerA.getId()).getLife());life.add(g.getPlayer(playerB.getId()).getLife());result.add("life",life);result.addProperty("legal",legal);captured=true;g.pause();}
    @Override protected TestPlayer createPlayer(String n,RangeOfInfluence r){return new TestPlayer(new TestComputerPlayer(n,r)){
        @Override public boolean priority(Game g){
            if(!injected && !mode.equals("cast") && !mode.equals("ground")){
                injected=true;Permanent p=find(g,0);
                // Alter the underlying test card as well as its current permanent;
                // XMage's subsequent layer rebuilds retain the synthetic definition.
                p.getCounters(g).addCounter(CounterType.P1P1.createInstance(3));
                g.getCard(p.getId()).addAbility(FlyingAbility.getInstance());
                p.getPower().setBoostedValue(5);p.getToughness().setBoostedValue(5);p.addAbility(FlyingAbility.getInstance(),p.getId(),g);
                if(mode.equals("tapped"))find(g,1).setTapped(true);
            }
            if(mode.equals("cast") && g.getTurnStepType()==PhaseStep.PRECOMBAT_MAIN && find(g,0)!=null && g.getStack().isEmpty()){
                assertEquals(0,g.getPlayer(playerA.getId()).getManaPool().count());capture(g);return false;
            }
            if(mode.equals("growth") && !growthMana && getId().equals(playerB.getId()) && g.getTurnStepType()==PhaseStep.DECLARE_BLOCKERS){
                growthMana=true;Permanent land=g.getBattlefield().getAllActivePermanents().stream().filter(p->p.getControllerId().equals(getId())&&p.isLand(g)).findFirst().get();
                assertTrue(activateAbility(land.getAbilities().getActivatedManaAbilities(Zone.BATTLEFIELD).get(0),g));return true;
            }
            if(g.getTurnStepType()==PhaseStep.COMBAT_DAMAGE){assertEquals(playerA.getId(),getId());capture(g);return false;}
            return super.priority(g);
        }
        @Override public boolean playMana(Ability ability,ManaCost unpaid,String prompt,Game g){
            assertTrue(mode.equals("cast")||mode.equals("growth"));
            Permanent land=g.getBattlefield().getAllActivePermanents().stream().filter(p->p.getControllerId().equals(getId())&&p.isLand(g)&&!p.isTapped()).findFirst().orElse(null);
            if(land!=null){List<ActivatedManaAbilityImpl> a=land.getAbilities().getActivatedManaAbilities(Zone.BATTLEFIELD);assertEquals(1,a.size());assertTrue(activateAbility(a.get(0),g));return true;}
            if(getManaPool().getGreen()>0)getManaPool().unlockManaType(ManaType.GREEN);
            else {assertTrue(getManaPool().getRed()>0);getManaPool().unlockManaType(ManaType.RED);}return true;
        }
        @Override public void selectAttackers(Game g,UUID active){assertEquals(playerA.getId(),getId());Permanent a=find(g,0);assertTrue(a.canAttack(playerB.getId(),g));declareAttacker(a.getId(),playerB.getId(),g,false);}
        @Override public void selectBlockers(Ability ability,Game g,UUID defending){
            assertEquals(playerB.getId(),getId());Permanent a=find(g,0),b=find(g,1);legal=b.canBlock(a.getId(),g);
            assertEquals(!mode.equals("cub-illegal")&&!mode.equals("tapped"),legal);
            if(legal)declareBlocker(getId(),b.getId(),a.getId(),g);
        }
    };}
    @Test public void executeCase() throws Exception {
        setStrictChooseMode(true);currentGame.setStartingPlayerId(playerA.getId());gameOptions.skipInitShuffling=true;
        removeAllCardsFromHand(playerA);removeAllCardsFromHand(playerB);removeAllCardsFromLibrary(playerA);removeAllCardsFromLibrary(playerB);
        if(mode.equals("cast")){
            addCard(Zone.HAND,playerA,"FDN-Magnigoth Sentry",1);addCard(Zone.BATTLEFIELD,playerA,"FDN-Forest",1);addCard(Zone.BATTLEFIELD,playerA,"FDN-Mountain",3);
            castSpell(1,PhaseStep.PRECOMBAT_MAIN,playerA,"Magnigoth Sentry");
        }else{
            addCard(Zone.BATTLEFIELD,playerA,"FDN-"+(mode.equals("ground")?"Magnigoth Sentry":"Bear Cub"),1);
            addCard(Zone.BATTLEFIELD,playerB,"FDN-"+(mode.equals("ground")||mode.equals("cub-illegal")?"Bear Cub":"Magnigoth Sentry"),1);
            if(mode.equals("growth")){addCard(Zone.HAND,playerB,"FDN-Giant Growth",1);addCard(Zone.BATTLEFIELD,playerB,"FDN-Forest",1);castSpell(1,PhaseStep.DECLARE_BLOCKERS,playerB,"Giant Growth","Magnigoth Sentry");}
        }
        setStopAt(1,PhaseStep.END_TURN);execute();assertTrue("checkpoint not reached",captured);
        Files.write(Paths.get(System.getProperty("mtglab.output"),spec.get("id").getAsString()+".json"),new GsonBuilder().serializeNulls().setPrettyPrinting().create().toJson(result).getBytes(StandardCharsets.UTF_8));
    }
}
