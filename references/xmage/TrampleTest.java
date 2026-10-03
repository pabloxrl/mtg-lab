package org.mage.test.mtglab;

import com.google.gson.*;
import mage.abilities.Ability;
import mage.constants.*;
import mage.game.Game;
import mage.game.permanent.Permanent;
import mage.game.permanent.PermanentImpl;
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

/** Original synthetic CR 510/702.19/702.20 cases. No upstream scenario copied.
 * Marked damage, sickness and departure are explicit test-only setup hooks. */
@RunWith(Parameterized.class)
public class TrampleTest extends CardTestPlayerBase {
    private final JsonObject spec;
    private final String mode;
    private boolean captured, adjusted, untapped=true, legal=true;
    private JsonObject result;
    private int assignments;
    @Parameterized.Parameters(name="{0}") public static Collection<Object[]> cases() throws Exception {
        JsonObject f=JsonParser.parseString(new String(Files.readAllBytes(Paths.get(System.getProperty("mtglab.fixture"))),StandardCharsets.UTF_8)).getAsJsonObject();
        assertEquals(1,f.get("version").getAsInt());
        List<Object[]> out=new ArrayList<>();for(JsonElement c:f.getAsJsonArray("cases"))out.add(new Object[]{c.getAsJsonObject()});return out;
    }
    public TrampleTest(JsonObject spec){this.spec=spec;mode=spec.get("id").getAsString();}
    private List<Permanent> creatures(Game g,int seat){UUID id=seat==0?playerA.getId():playerB.getId();List<Permanent> out=new ArrayList<>();for(Permanent p:g.getBattlefield().getAllActivePermanents())if(p.getControllerId().equals(id)&&p.isCreature(g))out.add(p);return out;}
    private Permanent attacker(Game g){return creatures(g,0).stream().findFirst().orElse(null);}
    private void capture(Game g){
        Permanent p=attacker(g);result=new JsonObject();
        if(p==null)result.add("attacker",JsonNull.INSTANCE);else{JsonArray s=new JsonArray();s.add(p.getPower().getValue());s.add(p.getToughness().getValue());s.add(p.getDamage());result.add("attacker",s);untapped &= !p.isTapped();}
        result.addProperty("untapped",untapped);result.addProperty("blockers",mode.equals("next-block")?0:creatures(g,1).size());result.addProperty("legal",legal);
        JsonArray life=new JsonArray(),graves=new JsonArray();life.add(g.getPlayer(playerA.getId()).getLife());life.add(g.getPlayer(playerB.getId()).getLife());graves.add(g.getPlayer(playerA.getId()).getGraveyard().size());graves.add(g.getPlayer(playerB.getId()).getGraveyard().size());result.add("life",life);result.add("graveyard",graves);captured=true;g.pause();
    }
    @Override protected TestPlayer createPlayer(String n,RangeOfInfluence r){return new TestPlayer(new TestComputerPlayer(n,r)){
        @Override public boolean priority(Game g){
            if(captured)return false;
            if(mode.equals("sick")&&g.getTurnNum()==1&&g.getTurnStepType()==PhaseStep.BEGIN_COMBAT){
                Permanent p=attacker(g);try{java.lang.reflect.Field f=PermanentImpl.class.getDeclaredField("controlledFromStartOfControllerTurn");f.setAccessible(true);f.setBoolean(p,false);}catch(Exception e){throw new AssertionError(e);}
                legal=p.canAttack(playerB.getId(),g);assertFalse(legal);capture(g);return false;
            }
            if(g.getTurnNum()==1&&g.getTurnStepType()==PhaseStep.DECLARE_ATTACKERS){assertFalse(attacker(g).isTapped());}
            if(!adjusted&&g.getTurnNum()==1&&g.getTurnStepType()==PhaseStep.DECLARE_BLOCKERS){
                adjusted=true;
                if(mode.equals("marked"))creatures(g,1).get(0).damage(1,attacker(g).getId(),null,g,false,false);
                if(mode.equals("departed"))assertTrue(creatures(g,1).get(0).destroy(null,g));
            }
            if(g.getTurnNum()==1&&g.getTurnStepType()==PhaseStep.COMBAT_DAMAGE&&!mode.equals("next-block")){capture(g);return false;}
            if(mode.equals("next-block")&&g.getTurnNum()==2&&g.getTurnStepType()==PhaseStep.DECLARE_BLOCKERS){capture(g);return false;}
            pass(g);return true;
        }
        @Override public void selectAttackers(Game g,UUID active){
            if(g.getTurnNum()==1){Permanent a=attacker(g);assertTrue(a.canAttack(playerB.getId(),g));declareAttacker(a.getId(),playerB.getId(),g,false);}
            else {assertEquals("next-block",mode);Permanent b=creatures(g,1).get(0);declareAttacker(b.getId(),playerA.getId(),g,false);}
        }
        @Override public void selectBlockers(Ability ability,Game g,UUID defending){
            if(g.getTurnNum()==1){if(!mode.equals("next-block"))for(Permanent b:creatures(g,1))declareBlocker(getId(),b.getId(),attacker(g).getId(),g);}
            else {Permanent a=attacker(g),b=creatures(g,1).get(0);assertFalse(a.isTapped());assertTrue(a.canBlock(b.getId(),g));declareBlocker(getId(),a.getId(),b.getId(),g);}
        }
        @Override public List<Integer> getMultiAmountWithIndividualConstraints(Outcome outcome,List<MultiAmountMessage> messages,int min,int max,MultiAmountType type,Game g){
            assertEquals(playerA.getId(),getId());assertEquals(0,assignments++);assertEquals(5,max);
            assertEquals(mode.equals("departed")?0:spec.get("blockers").getAsInt(),messages.size());
            List<Integer> amounts=new ArrayList<>();for(JsonElement v:spec.getAsJsonArray("amounts"))amounts.add(v.getAsInt());
            int sum=amounts.stream().mapToInt(Integer::intValue).sum();
            if(mode.equals("negative")){
                // Observe XMage's actual minimum: one damage cannot satisfy an undamaged 2/2.
                assertEquals(2,min);assertTrue(sum<min);legal=false;capture(g);
                // Stop before applying any allocation; this is a constraint rejection checkpoint.
                throw new RejectedAllocation();
            }
            assertTrue(sum>=min&&sum<=max);
            for(int i=0;i<amounts.size();i++)assertTrue(amounts.get(i)>=messages.get(i).min&&amounts.get(i)<=messages.get(i).max);
            return amounts;
        }
    };}
    private static class RejectedAllocation extends RuntimeException {}
    @Test public void executeCase() throws Exception {
        setStrictChooseMode(true);currentGame.setStartingPlayerId(playerA.getId());gameOptions.skipInitShuffling=true;
        removeAllCardsFromHand(playerA);removeAllCardsFromHand(playerB);removeAllCardsFromLibrary(playerA);removeAllCardsFromLibrary(playerB);
        addCard(Zone.LIBRARY,playerA,"FDN-Forest",2);addCard(Zone.LIBRARY,playerB,"FDN-Forest",2);
        addCard(Zone.BATTLEFIELD,playerA,"FDN-Tajuru Pathwarden",1);
        int count=mode.equals("next-block")?1:spec.get("blockers").getAsInt();if(count>0)addCard(Zone.BATTLEFIELD,playerB,"FDN-Bear Cub",count);
        setStopAt(2,PhaseStep.END_COMBAT);
        try{execute();}catch(RejectedAllocation e){assertEquals("negative",mode);}
        assertTrue("required checkpoint not reached",captured);
        Files.write(Paths.get(System.getProperty("mtglab.output"),mode+".json"),new GsonBuilder().serializeNulls().setPrettyPrinting().create().toJson(result).getBytes(StandardCharsets.UTF_8));
    }
}
