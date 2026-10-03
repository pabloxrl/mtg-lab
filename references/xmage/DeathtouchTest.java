package org.mage.test.mtglab;

import com.google.gson.*;
import mage.abilities.Ability;
import mage.abilities.keyword.FlyingAbility;
import mage.abilities.keyword.TrampleAbility;
import mage.abilities.mana.ActivatedManaAbilityImpl;
import mage.abilities.costs.mana.ManaCost;
import mage.constants.*;
import mage.counters.CounterType;
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

/** Original CR 702.2/702.19/510/704.5h/608.2b cases. Synthetic flying
 * and boost/trample definitions are test-only, not Shivan/Invoker support. */
@RunWith(Parameterized.class)
public class DeathtouchTest extends CardTestPlayerBase {
    private final JsonObject spec;
    private final String mode;
    private boolean injected,captured,legal=true,growthMana;
    private JsonObject result;
    private int assignments;
    @Parameterized.Parameters(name="{0}") public static Collection<Object[]> cases() throws Exception {
        JsonObject f=JsonParser.parseString(new String(Files.readAllBytes(Paths.get(System.getProperty("mtglab.fixture"))),StandardCharsets.UTF_8)).getAsJsonObject();
        assertEquals(1,f.get("version").getAsInt());
        List<Object[]> out=new ArrayList<>();for(JsonElement c:f.getAsJsonArray("cases"))out.add(new Object[]{c.getAsJsonObject()});return out;
    }
    public DeathtouchTest(JsonObject spec){this.spec=spec;mode=spec.get("mode").getAsString();}
    private List<Permanent> creatures(Game g,int seat){UUID id=seat==0?playerA.getId():playerB.getId();List<Permanent> out=new ArrayList<>();for(Permanent p:g.getBattlefield().getAllActivePermanents())if(p.getControllerId().equals(id)&&p.isCreature(g))out.add(p);return out;}
    private Permanent attacker(Game g){return creatures(g,0).stream().findFirst().orElse(null);}
    private void capture(Game g){
        result=new JsonObject();JsonArray all=new JsonArray();
        for(int seat=0;seat<2;seat++){JsonArray cards=new JsonArray();for(Permanent p:creatures(g,seat)){JsonArray s=new JsonArray();s.add(p.getPower().getValue());s.add(p.getToughness().getValue());s.add(p.getDamage());cards.add(s);}all.add(cards);}
        result.add("creatures",all);result.addProperty("legal",legal);
        JsonArray life=new JsonArray(),graves=new JsonArray();life.add(g.getPlayer(playerA.getId()).getLife());life.add(g.getPlayer(playerB.getId()).getLife());graves.add(g.getPlayer(playerA.getId()).getGraveyard().size());graves.add(g.getPlayer(playerB.getId()).getGraveyard().size());result.add("life",life);result.add("graveyard",graves);captured=true;g.pause();
    }
    @Override protected TestPlayer createPlayer(String n,RangeOfInfluence r){return new TestPlayer(new TestComputerPlayer(n,r)){
        @Override public boolean priority(Game g){
            if(captured)return false;
            if(!injected){
                injected=true;Permanent a=attacker(g);
                if(mode.equals("flyer")){a.getCounters(g).addCounter(CounterType.P1P1.createInstance(3));g.getCard(a.getId()).addAbility(FlyingAbility.getInstance());a.getPower().setBoostedValue(5);a.getToughness().setBoostedValue(5);a.addAbility(FlyingAbility.getInstance(),a.getId(),g);}
                if(mode.equals("trample")||mode.equals("negative")){a.getCounters(g).addCounter(CounterType.P1P1.createInstance(5));g.getCard(a.getId()).addAbility(TrampleAbility.getInstance());a.getPower().setBoostedValue(7);a.getToughness().setBoostedValue(6);a.addAbility(TrampleAbility.getInstance(),a.getId(),g);}
            }
            // Float scripted mana before the harness queues spell targets; no AI payment.
            if((mode.equals("bite")||mode.equals("source-gone"))&&g.getTurnStepType()==PhaseStep.PRECOMBAT_MAIN){
                Permanent land=g.getBattlefield().getAllActivePermanents().stream().filter(p->p.getControllerId().equals(getId())&&p.isLand(g)&&!p.isTapped()).findFirst().orElse(null);
                if(land!=null){assertTrue(activateAbility(land.getAbilities().getActivatedManaAbilities(Zone.BATTLEFIELD).get(0),g));return true;}
            }
            if((mode.equals("bite")||mode.equals("source-gone"))&&g.getTurnStepType()==PhaseStep.PRECOMBAT_MAIN&&g.getStack().isEmpty()&&g.getPlayer(playerA.getId()).getGraveyard().size()>0){capture(g);return false;}
            if(mode.equals("growth-no-trample")&&!growthMana&&getId().equals(playerA.getId())&&g.getTurnStepType()==PhaseStep.DECLARE_BLOCKERS){
                growthMana=true;Permanent land=g.getBattlefield().getAllActivePermanents().stream().filter(p->p.getControllerId().equals(getId())&&p.isLand(g)).findFirst().get();assertTrue(activateAbility(land.getAbilities().getActivatedManaAbilities(Zone.BATTLEFIELD).get(0),g));return true;
            }
            if(g.getTurnStepType()==PhaseStep.COMBAT_DAMAGE){
                capture(g);return false;
            }
            return super.priority(g);
        }
        @Override public boolean playMana(Ability ability,ManaCost unpaid,String prompt,Game g){
            Permanent land=g.getBattlefield().getAllActivePermanents().stream().filter(p->p.getControllerId().equals(getId())&&p.isLand(g)&&!p.isTapped()).findFirst().orElse(null);
            if(land!=null){List<ActivatedManaAbilityImpl> a=land.getAbilities().getActivatedManaAbilities(Zone.BATTLEFIELD);assertEquals(1,a.size());assertTrue(activateAbility(a.get(0),g));return true;}
            assertTrue(getManaPool().getGreen()>0);getManaPool().unlockManaType(ManaType.GREEN);return true;
        }
        @Override public void selectAttackers(Game g,UUID active){assertEquals(playerA.getId(),getId());Permanent a=attacker(g);assertTrue(a.canAttack(playerB.getId(),g));declareAttacker(a.getId(),playerB.getId(),g,false);}
        @Override public void selectBlockers(Ability ability,Game g,UUID defending){
            assertEquals(playerB.getId(),getId());Permanent a=attacker(g);
            if(mode.equals("no-trample")||mode.equals("growth-no-trample")){legal=a.getAbilities().containsKey(TrampleAbility.getInstance().getId());assertFalse(legal);}
            for(Permanent b:creatures(g,1)){assertTrue(b.canBlock(a.getId(),g));declareBlocker(getId(),b.getId(),a.getId(),g);}
        }
        @Override public List<Integer> getMultiAmountWithIndividualConstraints(Outcome outcome,List<MultiAmountMessage> messages,int min,int max,MultiAmountType type,Game g){
            assertEquals(playerA.getId(),getId());assertEquals(0,assignments++);assertEquals(2,messages.size());assertEquals(mode.equals("zero-split")?2:7,max);assertEquals(2,min);
            List<Integer> amounts=new ArrayList<>();for(JsonElement v:spec.getAsJsonArray("amounts"))amounts.add(v.getAsInt());
            int sum=amounts.stream().mapToInt(Integer::intValue).sum();
            if(mode.equals("negative")){assertTrue(sum<min);legal=false;capture(g);throw new RejectedAllocation();}
            assertTrue(sum>=min&&sum<=max);for(int i=0;i<amounts.size();i++)assertTrue(amounts.get(i)>=messages.get(i).min&&amounts.get(i)<=messages.get(i).max);return amounts;
        }
    };}
    private static class RejectedAllocation extends RuntimeException {}
    @Test public void executeCase() throws Exception {
        setStrictChooseMode(true);currentGame.setStartingPlayerId(playerA.getId());gameOptions.skipInitShuffling=true;
        removeAllCardsFromHand(playerA);removeAllCardsFromHand(playerB);removeAllCardsFromLibrary(playerA);removeAllCardsFromLibrary(playerB);
        addCard(Zone.LIBRARY,playerA,"FDN-Forest",2);addCard(Zone.LIBRARY,playerB,"FDN-Forest",2);
        addCard(Zone.BATTLEFIELD,playerA,"FDN-"+(mode.equals("flyer")?"Bear Cub":"Thornweald Archer"),1);
        String blocker=mode.equals("flyer")?"Thornweald Archer":mode.equals("bite")||spec.has("blocker")&&spec.get("blocker").getAsString().equals("magnigoth-sentry")?"Magnigoth Sentry":"Bear Cub";
        addCard(Zone.BATTLEFIELD,playerB,"FDN-"+blocker,spec.get("blockers").getAsInt());
        if(mode.equals("bite")||mode.equals("source-gone")){
            addCard(Zone.BATTLEFIELD,playerA,"FDN-Forest",2);addCard(Zone.HAND,playerA,"FDN-Bite Down",1);
            castSpell(1,PhaseStep.PRECOMBAT_MAIN,playerA,"Bite Down","Thornweald Archer^"+blocker);
            if(mode.equals("source-gone")){addCard(Zone.BATTLEFIELD,playerB,"FDN-Forest",2);addCard(Zone.HAND,playerB,"FDN-Bite Down",1);castSpell(1,PhaseStep.PRECOMBAT_MAIN,playerB,"Bite Down","Bear Cub^Thornweald Archer","Bite Down");}
        }
        if(mode.equals("growth-no-trample")){addCard(Zone.BATTLEFIELD,playerA,"FDN-Forest",1);addCard(Zone.HAND,playerA,"FDN-Giant Growth",1);castSpell(1,PhaseStep.DECLARE_BLOCKERS,playerA,"Giant Growth","Thornweald Archer");}
        setStopAt(1,PhaseStep.END_COMBAT);try{execute();}catch(RejectedAllocation e){assertEquals("negative",mode);}assertTrue("required checkpoint not reached",captured);
        Files.write(Paths.get(System.getProperty("mtglab.output"),spec.get("id").getAsString()+".json"),new GsonBuilder().serializeNulls().setPrettyPrinting().create().toJson(result).getBytes(StandardCharsets.UTF_8));
    }
}
