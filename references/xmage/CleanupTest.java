package org.mage.test.mtglab;

import com.google.gson.*;
import mage.abilities.*;
import mage.abilities.costs.mana.ManaCost;
import mage.abilities.effects.common.continuous.BoostTargetEffect;
import mage.cards.Card;
import mage.constants.*;
import mage.game.Game;
import mage.game.events.GameEvent;
import mage.game.permanent.Permanent;
import mage.target.Target;
import mage.target.common.TargetDiscard;
import mage.target.common.TargetCardInHand;
import mage.target.targetpointer.FixedTarget;
import mage.watchers.Watcher;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.junit.runners.Parameterized;
import org.mage.test.player.TestComputerPlayer;
import org.mage.test.player.TestPlayer;
import org.mage.test.serverside.base.CardTestPlayerBase;
import java.nio.file.*;
import java.nio.charset.StandardCharsets;
import java.io.Serializable;
import java.util.*;
import static org.junit.Assert.*;

/** Original CR 514/704 tests. Synthetic hand, boost/damage and pending ETB
 * at first cleanup entry; real XMage EndPhase drives cleanup and repetition. */
@RunWith(Parameterized.class)
public class CleanupTest extends CardTestPlayerBase {
 private final JsonObject spec;
 private final JsonArray points = new JsonArray();
 private final List<List<Card>> hands = new ArrayList<>();
 private boolean installed, finished;
 private int cleanups, calls, discards, targets, stage;
 @Parameterized.Parameters(name="{0}") public static Collection<Object[]> cases() throws Exception {
  JsonObject f=JsonParser.parseString(new String(Files.readAllBytes(Paths.get(System.getProperty("mtglab.fixture"))),StandardCharsets.UTF_8)).getAsJsonObject();
  assertEquals(1,f.get("version").getAsInt());List<Object[]> out=new ArrayList<>();for(JsonElement c:f.getAsJsonArray("cases"))out.add(new Object[]{c.getAsJsonObject()});return out;
 }
 public CleanupTest(JsonObject spec) { this.spec=spec; }
 private Permanent pyro(Game g) {return g.getBattlefield().getAllActivePermanents().stream().filter(p->p.getName().equals("Viashino Pyromancer")).findFirst().get();}
 private JsonArray point(Game g) {
  JsonArray p=new JsonArray();Permanent c=pyro(g);
  p.add(g.getPlayer(playerA.getId()).getHand().size());p.add(g.getPlayer(playerB.getId()).getHand().size());
  p.add(g.getPlayer(playerA.getId()).getGraveyard().size());p.add(g.getPlayer(playerB.getId()).getLife());p.add(g.getStack().size());
  p.add(c.getPower().getValue());p.add(c.getToughness().getValue());p.add(c.getDamage());return p;
 }
 private class CleanupObserver extends Watcher {
  CleanupObserver(){super(WatcherScope.GAME);}
  CleanupObserver(CleanupObserver other){super(other);}
  @Override public Watcher copy(){return new CleanupObserver(this);}
  @Override public void watch(GameEvent event,Game g){
   if(event.getType()==GameEvent.EventType.CLEANUP_STEP_PRE){
    assertTrue(++cleanups<=2);
    if(cleanups==1){
     points.add(point(g));
     if(spec.get("trigger").getAsBoolean())g.getState().addTriggeredAbility(pyro(g).getAbilities().getTriggeredAbilities(Zone.BATTLEFIELD).get(0).copy());
    }
   }
  }
 }
 @Override protected TestPlayer createPlayer(String name,RangeOfInfluence range){return new TestPlayer(new TestComputerPlayer(name,range)){
  @Override public boolean priority(Game g){
   assertTrue("bounded explicit priority script",++calls<150);
   if(g.getTurnNum()==2){
    assertEquals(PhaseStep.UPKEEP,g.getTurnStepType());assertEquals(playerB.getId(),getId());
    assertEquals(spec.get("trigger").getAsBoolean()?2:1,cleanups);
    points.add(point(g));finished=true;g.pause();return false;
   }
   if(g.getTurnStepType()==PhaseStep.END_TURN&&!installed){
    assertEquals(playerA.getId(),getId());
    for(int i=0;i<2;i++)g.cheat(i==0?playerA.getId():playerB.getId(),Collections.emptyList(),hands.get(i),Collections.emptyList(),Collections.emptyList(),Collections.emptyList(),Collections.emptyList());
    Permanent c=pyro(g);BoostTargetEffect effect=new BoostTargetEffect(3,3,Duration.EndOfTurn);effect.setTargetPointer(new FixedTarget(c.getId(),g));
    g.addEffect(effect,c.getAbilities().getTriggeredAbilities(Zone.BATTLEFIELD).get(0));g.applyEffects();c.damage(3,c.getId(),c.getAbilities().getTriggeredAbilities(Zone.BATTLEFIELD).get(0),g,false,false);
    g.getState().addWatcher(new CleanupObserver());installed=true;
   }
   if(g.getTurnStepType()==PhaseStep.CLEANUP){
    assertTrue("ordinary cleanup cannot grant priority",spec.get("trigger").getAsBoolean());assertEquals(1,cleanups);
    if(getId().equals(playerA.getId())){
     if(stage==0){assertEquals(1,g.getStack().size());points.add(point(g));stage=1;}
     else if(stage==1&&g.getStack().isEmpty()){
      points.add(point(g));stage=2;
      if(spec.get("thrill").getAsBoolean()){
       for(Permanent land:g.getBattlefield().getAllActivePermanents())if(land.isLand(g)&&land.getControllerId().equals(getId()))assertTrue(activateAbility(land.getAbilities().getActivatedManaAbilities(Zone.BATTLEFIELD).get(0),g));
       Card c=getHand().getCards(g).stream().filter(x->x.getName().equals("Thrill of Possibility")).findFirst().get();assertTrue(cast(c.getSpellAbility(),g,false,null));points.add(point(g));return true;
      }
     }else if(stage==2&&spec.get("thrill").getAsBoolean()&&g.getStack().isEmpty()){points.add(point(g));stage=3;}
    }
   }
   pass(g);return false;
  }
  @Override public boolean chooseTarget(Outcome outcome,Target target,Ability source,Game g){
   assertEquals(1,++targets);assertEquals(PhaseStep.CLEANUP,g.getTurnStepType());assertTrue(target.canTarget(playerB.getId(),source,g));target.addTarget(playerB.getId(),source,g);return true;
  }
  @Override public boolean choose(Outcome outcome,Target target,Ability source,Game g,Map<String,Serializable> options){
   assertTrue("explicit discard choice: "+target.getClass(),target instanceof TargetDiscard||target instanceof TargetCardInHand);assertEquals(Outcome.Discard,outcome);assertEquals(playerA.getId(),getId());assertEquals(PhaseStep.CLEANUP,g.getTurnStepType());
   int count=discards++==0?spec.get("hand").getAsInt()-7:1;
   assertEquals(count,target.getMinNumberOfTargets());assertEquals(count,target.getMaxNumberOfTargets());
   for(Card c:getHand().getCards(g))if(c.getName().equals("Mountain")&&target.getTargets().size()<count)target.addTarget(c.getId(),source,g);
   assertEquals(count,target.getTargets().size());return true;
  }
  @Override public boolean playMana(Ability source,ManaCost unpaid,String prompt,Game g){assertTrue(getManaPool().getRed()>0);getManaPool().unlockManaType(ManaType.RED);return true;}
  @Override public void selectAttackers(Game g,UUID id) { assertEquals(getId(),id); }
 };}
 @Test public void executeCase() throws Exception {
  setStrictChooseMode(true);currentGame.setStartingPlayerId(playerA.getId());gameOptions.skipInitShuffling=true;
  removeAllCardsFromHand(playerA);removeAllCardsFromHand(playerB);removeAllCardsFromLibrary(playerA);removeAllCardsFromLibrary(playerB);
  addCard(Zone.LIBRARY,playerA,"FDN-Forest",4);addCard(Zone.LIBRARY,playerB,"FDN-Forest",4);
  addCard(Zone.BATTLEFIELD,playerA,"FDN-Viashino Pyromancer",1);addCard(Zone.BATTLEFIELD,playerA,"FDN-Mountain",2);
  addCard(Zone.HAND,playerA,"FDN-Mountain",spec.get("hand").getAsInt()-(spec.get("thrill").getAsBoolean()?1:0));
  if(spec.get("thrill").getAsBoolean())addCard(Zone.HAND,playerA,"FDN-Thrill of Possibility",1);
  addCard(Zone.HAND,playerB,"FDN-Mountain",8);
  for(TestPlayer p:Arrays.asList(playerA,playerB)){hands.add(new ArrayList<>(getHandCards(p)));getHandCards(p).clear();}
  setStopAt(2,PhaseStep.PRECOMBAT_MAIN);execute();assertTrue("next upkeep reached",finished);
  assertEquals(spec.get("thrill").getAsBoolean()?2:1,discards);
  Files.write(Paths.get(System.getProperty("mtglab.output"),spec.get("id").getAsString()+".json"),new GsonBuilder().setPrettyPrinting().create().toJson(points).getBytes(StandardCharsets.UTF_8));
 }
}
