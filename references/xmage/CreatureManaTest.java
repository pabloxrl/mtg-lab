package org.mage.test.mtglab;

import com.google.gson.*;
import mage.abilities.Ability;
import mage.abilities.mana.ActivatedManaAbilityImpl;
import mage.abilities.costs.mana.ManaCost;
import mage.constants.*;
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

/** Original synthetic cases, CR 302.6/601/605/508-510 and pinned card text.
 * All activations, casting and combat execute XMage rules; no outcome calculator. */
@RunWith(Parameterized.class)
public class CreatureManaTest extends CardTestPlayerBase {
    private final JsonObject spec;
    private final String mode, name;
    private boolean started, activatedInPayment, forestInPayment, captured;
    private JsonObject result;
    @Parameterized.Parameters(name="{0}") public static Collection<Object[]> cases() throws Exception {
        JsonObject f=JsonParser.parseString(new String(Files.readAllBytes(Paths.get(System.getProperty("mtglab.fixture"))),StandardCharsets.UTF_8)).getAsJsonObject();
        assertEquals(1,f.get("version").getAsInt());
        List<Object[]> out=new ArrayList<>();for(JsonElement c:f.getAsJsonArray("cases"))out.add(new Object[]{c.getAsJsonObject()});return out;
    }
    public CreatureManaTest(JsonObject spec){this.spec=spec;mode=spec.get("mode").getAsString();name=spec.get("card").getAsString().equals("llanowar-elves")?"Llanowar Elves":"Druid of the Cowl";}
    private Permanent find(Game game,String name){return game.getBattlefield().getAllActivePermanents().stream().filter(p->p.getName().equals(name)).findFirst().orElse(null);}
    private ActivatedManaAbilityImpl mana(Permanent p){List<ActivatedManaAbilityImpl> a=p.getAbilities().getActivatedManaAbilities(Zone.BATTLEFIELD);assertEquals(1,a.size());return a.get(0);}
    private void capture(Game g){
        Permanent p=find(g,name),cub=find(g,"Bear Cub");assertNotNull(p);
        result=new JsonObject();result.addProperty("power",p.getPower().getValue());result.addProperty("toughness",p.getToughness().getValue());result.addProperty("damage",p.getDamage());result.addProperty("tapped",p.isTapped());result.addProperty("green",g.getPlayer(playerA.getId()).getManaPool().getGreen());result.addProperty("stack",g.getStack().size());result.addProperty("priority",g.getPriorityPlayerId().equals(playerA.getId())?0:1);result.addProperty("cub",cub!=null);
        if(mode.equals("block"))result.addProperty("cub_damage",cub.getDamage());else result.add("cub_damage",JsonNull.INSTANCE);
        JsonArray life=new JsonArray();life.add(g.getPlayer(playerA.getId()).getLife());life.add(g.getPlayer(playerB.getId()).getLife());result.add("life",life);captured=true;g.pause();
    }
    @Override protected TestPlayer createPlayer(String n,RangeOfInfluence r){return new TestPlayer(new TestComputerPlayer(n,r)){
        @Override public boolean priority(Game game){
            Permanent p=find(game,name);
            boolean own=getId().equals(playerA.getId());
            if(game.getTurnNum()==1 && game.getTurnStepType()==PhaseStep.PRECOMBAT_MAIN){
                if(mode.equals("priority") && own && !started){
                    started=true;assertTrue(activateAbility(mana(p),game));
                    assertFalse(mana(p).canActivate(getId(),game).canActivate());capture(game);return false;
                }
                if(mode.equals("opponent") && !own){
                    // The owner cannot activate outside its priority or a payment.
                    assertFalse(game.getPriorityPlayerId().equals(p.getControllerId()));capture(game);return false;
                }
                if((mode.equals("payment")||mode.equals("floating")) && own && !started){
                    started=true;
                    if(mode.equals("floating"))assertTrue(activateAbility(mana(find(game,"Forest")),game));
                    getManaPool().setAutoPayment(false);
                    UUID card=getHand().stream().filter(id->game.getCard(id).getName().equals("Bear Cub")).findFirst().get();
                    assertTrue(cast(game.getCard(card).getSpellAbility(),game,false,null));
                    assertTrue(activatedInPayment);assertEquals(mode.equals("payment"),forestInPayment);
                    assertEquals(1,game.getStack().size());assertEquals(playerA.getId(),game.getPriorityPlayerId());
                    return true;
                }
                if((mode.equals("payment")||mode.equals("floating")) && own && started && game.getStack().isEmpty()) {capture(game);return false;}
                if(mode.equals("sick") && own && p!=null && game.getStack().isEmpty()) {
                    assertFalse(mana(p).canActivate(getId(),game).canActivate());capture(game);return false;
                }
            }
            if(mode.equals("attacked") && own && game.getTurnStepType()==PhaseStep.POSTCOMBAT_MAIN){
                assertTrue(p.isTapped());assertFalse(mana(p).canActivate(getId(),game).canActivate());capture(game);return false;
            }
            if(mode.equals("block") && own && game.getTurnStepType()==PhaseStep.COMBAT_DAMAGE){capture(game);return false;}
            return super.priority(game);
        }
        @Override public boolean playMana(Ability ability,ManaCost unpaid,String prompt,Game game){
            assertEquals(playerA.getId(),getId());
            if(mode.equals("sick")) {
                Permanent forest=game.getBattlefield().getAllActivePermanents().stream().filter(p->p.getName().equals("Forest")&&!p.isTapped()).findFirst().orElse(null);
                if(forest!=null){assertTrue(activateAbility(mana(forest),game));return true;}
                assertTrue(getManaPool().getGreen()>0);getManaPool().unlockManaType(ManaType.GREEN);return true;
            }
            assertTrue(mode.equals("payment")||mode.equals("floating"));
            if(!activatedInPayment){activatedInPayment=true;assertTrue(activateAbility(mana(find(game,name)),game));return true;}
            if(mode.equals("payment")&&!forestInPayment){forestInPayment=true;assertTrue(activateAbility(mana(find(game,"Forest")),game));return true;}
            assertTrue(getManaPool().getGreen()>0);getManaPool().unlockManaType(ManaType.GREEN);return true;
        }
        @Override public void selectAttackers(Game game,UUID active){
            assertEquals(playerA.getId(),getId());
            if(mode.equals("attacked"))declareAttacker(find(game,name).getId(),playerB.getId(),game,false);
            else if(mode.equals("block"))declareAttacker(find(game,"Bear Cub").getId(),playerB.getId(),game,false);
        }
        @Override public void selectBlockers(Ability ability,Game game,UUID defending){
            if(mode.equals("block"))declareBlocker(getId(),find(game,name).getId(),find(game,"Bear Cub").getId(),game);
        }
    };}
    @Test public void executeCase() throws Exception {
        setStrictChooseMode(true);currentGame.setStartingPlayerId(playerA.getId());gameOptions.skipInitShuffling=true;
        removeAllCardsFromHand(playerA);removeAllCardsFromHand(playerB);removeAllCardsFromLibrary(playerA);removeAllCardsFromLibrary(playerB);
        if(mode.equals("sick")){
            addCard(Zone.HAND,playerA,"FDN-"+name,1);addCard(Zone.BATTLEFIELD,playerA,"FDN-Forest",name.equals("Llanowar Elves")?1:2);
            castSpell(1,PhaseStep.PRECOMBAT_MAIN,playerA,name);
        }else addCard(Zone.BATTLEFIELD,mode.equals("block")?playerB:playerA,"FDN-"+name,1);
        if(mode.equals("payment")||mode.equals("floating")){addCard(Zone.BATTLEFIELD,playerA,"FDN-Forest",1);addCard(Zone.HAND,playerA,"FDN-Bear Cub",1);}
        if(mode.equals("block"))addCard(Zone.BATTLEFIELD,playerA,"FDN-Bear Cub",1);
        setStopAt(1,PhaseStep.END_TURN);execute();assertTrue("required checkpoint not reached",captured);
        Files.write(Paths.get(System.getProperty("mtglab.output"),spec.get("id").getAsString()+".json"),new GsonBuilder().serializeNulls().setPrettyPrinting().create().toJson(result).getBytes(StandardCharsets.UTF_8));
    }
}
