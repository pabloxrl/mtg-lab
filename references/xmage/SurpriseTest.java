package org.mage.test.mtglab;
import com.google.gson.*;
import mage.abilities.*;
import mage.constants.*;
import mage.game.Game;
import mage.game.permanent.Permanent;
import mage.game.permanent.token.GoblinToken;
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
/** Original pinned Oracle / CR 700.2, 601.2b, 611.2c, 302.6, 510 and 514.
 * Synthetic positions, actual XMage casting/continuous effects/combat/cleanup. */
@RunWith(Parameterized.class)
public class SurpriseTest extends CardTestPlayerBase {
 private final String scenario;private boolean initialized,pending,settled;private int choices,calls;private final JsonObject result=new JsonObject();
 @Parameterized.Parameters(name="{0}") public static Collection<Object[]> cases(){return Arrays.asList(new Object[][]{{"boost"},{"tokens"},{"later"},{"stacked"},{"block"},{"end"},{"swab"}});}
 public SurpriseTest(String s){scenario=s;}
 private boolean tokenMode(){return Arrays.asList("tokens","block","end").contains(scenario);}
 private List<Permanent> creatures(Game g){List<Permanent> out=new ArrayList<>();for(Permanent p:g.getBattlefield().getAllActivePermanents())if(p.isCreature(g))out.add(p);return out;}
 private JsonObject point(Game g){JsonObject out=new JsonObject();List<String> rows=new ArrayList<>();for(Permanent p:creatures(g))rows.add((p.getControllerId().equals(playerA.getId())?"0:":"1:")+(p.isToken()?"goblin-token":p.getName().toLowerCase(Locale.ROOT).replace(' ','-'))+":"+p.getPower().getValue()+"/"+p.getToughness().getValue()+":"+p.getDamage());Collections.sort(rows);JsonArray a=new JsonArray();rows.forEach(a::add);out.add("creatures",a);JsonArray life=new JsonArray();life.add(playerA.getLife());life.add(playerB.getLife());out.add("life",life);return out;}
 @Override protected TestPlayer createPlayer(String n,RangeOfInfluence r){return new TestPlayer(new TestComputerPlayer(n,r)){
  @Override public Mode chooseMode(Modes modes,Ability source,Game g){assertEquals("Goblin Surprise",g.getCard(source.getSourceId()).getName());assertEquals(2,modes.size());assertEquals(1,++choices);return new ArrayList<>(modes.values()).get(tokenMode()?1:0);}
  @Override public void selectAttackers(Game g,UUID active){
   if((scenario.equals("block")||scenario.equals("swab"))&&g.getTurnNum()==2){Permanent p=creatures(g).stream().filter(c->c.getControllerId().equals(playerB.getId())).findFirst().get();declareAttacker(p.getId(),playerA.getId(),g,false);}
   else if(scenario.equals("end")&&g.getTurnNum()==3){for(Permanent p:creatures(g))if(p.isToken())declareAttacker(p.getId(),playerB.getId(),g,false);}
  }
  @Override public void selectBlockers(Ability a,Game g,UUID defending){if((scenario.equals("block")||scenario.equals("swab"))&&g.getTurnNum()==2){Permanent b=creatures(g).stream().filter(c->c.getControllerId().equals(playerA.getId())).findFirst().get();Permanent at=creatures(g).stream().filter(c->c.getControllerId().equals(playerB.getId())).findFirst().get();declareBlocker(getId(),b.getId(),at.getId(),g);}}
  @Override public boolean priority(Game g){assertTrue("bounded script",++calls<500);
   if(!initialized){initialized=true;if(Arrays.asList("boost","tokens","later").contains(scenario)){Ability source=g.getPlayer(playerA.getId()).getHand().getCards(g).stream().filter(c->c.getName().equals("Goblin Surprise")).findFirst().get().getSpellAbility();assertTrue(new GoblinToken().putOntoBattlefield(scenario.equals("boost")?2:1,g,source,playerA.getId()));g.applyEffects();}}
   if(!g.getStack().isEmpty()&&g.getStack().peek().getName().equals("Goblin Surprise")){pending=true;assertEquals(1,choices);result.addProperty("mode",tokenMode()?1:0);}
   boolean grave=g.getPlayer(playerA.getId()).getGraveyard().getCards(g).stream().anyMatch(c->c.getName().equals("Goblin Surprise"));
   if(grave&&g.getStack().isEmpty()&&!settled){settled=true;result.add("resolved",point(g));}
   if(scenario.equals("later")&&g.getPlayer(playerA.getId()).getGraveyard().getCards(g).stream().anyMatch(c->c.getName().equals("Dragon Fodder"))&&g.getStack().isEmpty()&&!result.has("later"))result.add("later",point(g));
   if((scenario.equals("block")||scenario.equals("swab")||scenario.equals("end"))&&g.getTurnStepType()==PhaseStep.COMBAT_DAMAGE&&settled&&!result.has("combat"))result.add("combat",point(g));
   return super.priority(g);
  }
 };}
 @Test public void executeCase() throws Exception {
  setStrictChooseMode(true);currentGame.setStartingPlayerId(playerA.getId());gameOptions.skipInitShuffling=true;
  removeAllCardsFromHand(playerA);removeAllCardsFromHand(playerB);removeAllCardsFromLibrary(playerA);removeAllCardsFromLibrary(playerB);
  addCard(Zone.LIBRARY,playerA,"FDN-Mountain",8);addCard(Zone.LIBRARY,playerB,"FDN-Forest",8);
  addCard(Zone.BATTLEFIELD,playerA,"FDN-Mountain",6);addCard(Zone.BATTLEFIELD,playerA,"FDN-Forest",1);
  addCard(Zone.HAND,playerA,"FDN-Goblin Surprise",1);
  if(Arrays.asList("boost","tokens").contains(scenario)){addCard(Zone.BATTLEFIELD,playerA,"FDN-Bear Cub",1);addCard(Zone.BATTLEFIELD,playerB,"FDN-Bear Cub",1);}
  if(scenario.equals("stacked")){addCard(Zone.BATTLEFIELD,playerA,"FDN-Shivan Dragon",1);addCard(Zone.HAND,playerA,"FDN-Giant Growth",1);castSpell(1,PhaseStep.UPKEEP,playerA,"Giant Growth","Shivan Dragon");activateAbility(1,PhaseStep.PRECOMBAT_MAIN,playerA,"{R}: {this} gets +1/+0 until end of turn.");}
  if(scenario.equals("block"))addCard(Zone.BATTLEFIELD,playerB,"FDN-Bear Cub",1);
  if(scenario.equals("swab")){addCard(Zone.BATTLEFIELD,playerA,"FDN-Swab Goblin",1);addCard(Zone.BATTLEFIELD,playerB,"FDN-Magnigoth Sentry",1);}
  int turn=Arrays.asList("block","end","swab").contains(scenario)?2:1;
  PhaseStep step=scenario.equals("end")?PhaseStep.END_TURN:turn==2?PhaseStep.DECLARE_ATTACKERS:scenario.equals("stacked")?PhaseStep.POSTCOMBAT_MAIN:PhaseStep.PRECOMBAT_MAIN;
  castSpell(turn,step,playerA,"Goblin Surprise");
  if(scenario.equals("later")){addCard(Zone.HAND,playerA,"FDN-Dragon Fodder",1);castSpell(1,PhaseStep.POSTCOMBAT_MAIN,playerA,"Dragon Fodder");}
  setStopAt(scenario.equals("end")?4:turn+1,PhaseStep.UPKEEP);execute();assertTrue(pending);assertTrue(settled);assertEquals(1,choices);result.add("cleanup",point(currentGame));
  Files.write(Paths.get(System.getProperty("mtglab.output"),scenario+".json"),new GsonBuilder().setPrettyPrinting().create().toJson(result).getBytes(StandardCharsets.UTF_8));
 }
}
