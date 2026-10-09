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
/** Original Oracle/CR 603.2/603.3d/115/113.7a/704 scenarios. Synthetic
 * positions with real casts; explicitly synthetic end-step queue only. */
@RunWith(Parameterized.class)
public class EtbTriggersTest extends CardTestPlayerBase {
 private final JsonObject spec; private JsonArray points=new JsonArray(); private boolean captured, resolvedEndStep; private int calls, targetCalls,manaCalls;
 @Parameterized.Parameters(name="{0}") public static Collection<Object[]> cases() throws Exception {
  JsonObject f=JsonParser.parseString(new String(Files.readAllBytes(Paths.get(System.getProperty("mtglab.fixture"))),StandardCharsets.UTF_8)).getAsJsonObject();assertEquals(1,f.get("version").getAsInt());List<Object[]> out=new ArrayList<>();for(JsonElement c:f.getAsJsonArray("cases"))out.add(new Object[]{c.getAsJsonObject()});return out;
 }
 public EtbTriggersTest(JsonObject s){spec=s;}
 private Permanent pyro(Game g){return g.getBattlefield().getAllActivePermanents().stream().filter(p->p.getName().equals("Viashino Pyromancer")).findFirst().orElse(null);}
 private JsonArray point(Game g){JsonArray p=new JsonArray(),life=new JsonArray(),targets=new JsonArray(),lost=new JsonArray();life.add(g.getPlayer(playerA.getId()).getLife());life.add(g.getPlayer(playerB.getId()).getLife());p.add(life);Permanent c=pyro(g);if(c==null)p.add(JsonNull.INSTANCE);else {JsonArray stats=new JsonArray();stats.add(c.getPower().getValue());stats.add(c.getToughness().getValue());p.add(stats);}p.add(g.getStack().size());for(StackObject s:g.getStack())for(Target t:s.getStackAbility().getTargets())for(UUID id:t.getTargets()) {if(id.equals(playerA.getId()))targets.add(0);else if(id.equals(playerB.getId()))targets.add(1);}p.add(targets);lost.add(g.getPlayer(playerA.getId()).hasLost());lost.add(g.getPlayer(playerB.getId()).hasLost());p.add(lost);return p;}
 @Override protected TestPlayer createPlayer(String n,RangeOfInfluence r){return new TestPlayer(new TestComputerPlayer(n,r)){
  @Override public boolean priority(Game g){
   assertTrue("bounded scripted priority",++calls<150);if(captured)return false;
   if(resolvedEndStep){
    if(g.getTurnNum()==2){assertEquals(PhaseStep.UPKEEP,g.getTurnStepType());assertEquals(playerB.getId(),getId());assertTrue(g.getStack().isEmpty());assertEquals(18,g.getPlayer(playerB.getId()).getLife());captured=true;g.pause();return false;}
    assertEquals(PhaseStep.END_TURN,g.getTurnStepType());pass(g);return false;
   }
   PhaseStep step=spec.get("end_step").getAsBoolean()?PhaseStep.END_TURN:PhaseStep.PRECOMBAT_MAIN;
   if(g.getTurnNum()!=1||g.getTurnStepType()!=step||!getId().equals(playerA.getId())){pass(g);return false;}
   if(spec.get("end_step").getAsBoolean()){
    points.add(point(g));TriggeredAbility a=pyro(g).getAbilities().getTriggeredAbilities(Zone.BATTLEFIELD).get(0).copy();g.getState().addTriggeredAbility(a);
   }else {
    for(Permanent land:g.getBattlefield().getAllActivePermanents())if(land.isLand(g)&&land.getControllerId().equals(getId()))assertTrue(activateAbility(land.getAbilities().getActivatedManaAbilities(Zone.BATTLEFIELD).get(0),g));
    Card c=getHand().getCards(g).stream().filter(x->x.getName().equals("Viashino Pyromancer")).findFirst().get();assertTrue(c.getSpellAbility().getTargets().isEmpty());assertTrue(cast(c.getSpellAbility(),g,false,null));assertEquals(0,targetCalls);points.add(point(g));g.getStack().resolve(g);
   }
   g.checkStateAndTriggered();assertEquals(1,targetCalls);points.add(point(g));
   String death=spec.get("death").getAsString();
   if(death.equals("bite")){
    for(Permanent land:g.getBattlefield().getAllActivePermanents())if(land.isLand(g)&&land.getControllerId().equals(playerB.getId()))assertTrue(playerB.activateAbility(land.getAbilities().getActivatedManaAbilities(Zone.BATTLEFIELD).get(0),g));
    Card c=playerB.getHand().getCards(g).stream().filter(x->x.getName().equals("Bite Down")).findFirst().get();assertTrue(playerB.cast(c.getSpellAbility(),g,false,null));g.getStack().resolve(g);g.checkStateAndTriggered();points.add(point(g));
   }else if(death.equals("hook")){assertTrue(pyro(g).destroy(null,g));g.checkStateAndTriggered();points.add(point(g));}
   g.getStack().resolve(g);g.checkStateAndTriggered();points.add(point(g));
   if(spec.get("end_step").getAsBoolean()){resolvedEndStep=true;pass(g);}else{captured=true;g.pause();}return false;
  }
  @Override public boolean chooseTarget(Outcome outcome,Target t,Ability a,Game g){
   assertTrue(++targetCalls<4);
   UUID selected;
   if(getId().equals(playerB.getId())){selected=targetCalls==2?playerB.getAliasByName("cub"):pyro(g).getId();}
   else {
    assertEquals(1,t.getMinNumberOfTargets());assertTrue(t.getTargets().isEmpty());assertFalse(t.canTarget(playerB.getAliasByName("cub"),a,g));assertTrue(t.canTarget(playerA.getId(),a,g));assertTrue(t.canTarget(playerB.getId(),a,g));selected=spec.get("target").getAsInt()==0?playerA.getId():playerB.getId();
   }
   assertTrue(t.canTarget(selected,a,g));t.addTarget(selected,a,g);return true;
  }
  @Override public boolean playMana(Ability a,ManaCost unpaid,String prompt,Game g){assertTrue(++manaCalls<10);assertTrue(getManaPool().getRed()>0||getManaPool().getGreen()>0);if(getManaPool().getRed()>0)getManaPool().unlockManaType(ManaType.RED);if(getManaPool().getGreen()>0)getManaPool().unlockManaType(ManaType.GREEN);return true;}
  @Override public void selectAttackers(Game g,UUID actor){assertEquals(getId(),actor);}
 };}
 @Test public void executeCase() throws Exception {
  setStrictChooseMode(true);currentGame.setStartingPlayerId(playerA.getId());gameOptions.skipInitShuffling=true;
  removeAllCardsFromHand(playerA);removeAllCardsFromHand(playerB);removeAllCardsFromLibrary(playerA);removeAllCardsFromLibrary(playerB);addCard(Zone.LIBRARY,playerA,"FDN-Forest",3);addCard(Zone.LIBRARY,playerB,"FDN-Forest",3);
  addCard(spec.get("end_step").getAsBoolean()?Zone.BATTLEFIELD:Zone.HAND,playerA,"FDN-Viashino Pyromancer",1);addCard(Zone.BATTLEFIELD,playerA,"FDN-Mountain",2);addCard(Zone.BATTLEFIELD,playerB,"FDN-Bear Cub@cub",1);
  if(spec.get("death").getAsString().equals("bite")){addCard(Zone.HAND,playerB,"FDN-Bite Down",1);addCard(Zone.BATTLEFIELD,playerB,"FDN-Forest",2);}
  setLife(playerA,spec.get("life").getAsInt());setLife(playerB,20);setStopAt(2,PhaseStep.PRECOMBAT_MAIN);execute();assertTrue("checkpoint not reached",captured);
  Files.write(Paths.get(System.getProperty("mtglab.output"),spec.get("id").getAsString()+".json"),new GsonBuilder().serializeNulls().setPrettyPrinting().create().toJson(points).getBytes(StandardCharsets.UTF_8));
 }
}
