package org.mage.test.mtglab;

import com.google.gson.*;
import mage.abilities.*;
import mage.abilities.keyword.HasteAbility;
import mage.abilities.costs.mana.ManaCost;
import mage.cards.Card;
import mage.constants.*;
import mage.game.Game;
import mage.game.permanent.Permanent;
import mage.players.Player;
import mage.target.Target;
import mage.target.common.TargetCardInHand;
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

/** Bounded composition of the existing mana, haste, discard and trigger bridge
 * operations. Synthetic initial positions; all resulting rules execute in XMage.
 * No expected-checkpoint file is read by this bridge. */
@RunWith(Parameterized.class)
public class M2CostCompositionTest extends CardTestPlayerBase {
 private final JsonObject spec;
 private final JsonArray points = new JsonArray();
 private boolean captured, paymentMana;
 private int calls, manaCalls, discards, targets;
 @Parameterized.Parameters(name="{0}") public static Collection<Object[]> cases() throws Exception {
  JsonObject f=JsonParser.parseString(new String(Files.readAllBytes(Paths.get(System.getProperty("mtglab.fixture"))),StandardCharsets.UTF_8)).getAsJsonObject();
  assertEquals(1,f.get("version").getAsInt());
  List<Object[]> out=new ArrayList<>();for(JsonElement c:f.getAsJsonArray("cases"))out.add(new Object[]{c.getAsJsonObject()});return out;
 }
 public M2CostCompositionTest(JsonObject s){spec=s;}
 private String name(String key){switch(key){
  case "llanowar-elves":return "Llanowar Elves";case "druid-of-the-cowl":return "Druid of the Cowl";
  case "thrill-of-possibility":return "Thrill of Possibility";case "goblin-surprise":return "Goblin Surprise";
  case "forest":return "Forest";case "mountain":return "Mountain";case "bear-cub":return "Bear Cub";
  case "giant-growth":return "Giant Growth";case "bite-down":return "Bite Down";
  default:throw new AssertionError("unsupported card "+key);
 }}
 private Permanent find(Game g,String n){return g.getBattlefield().getAllActivePermanents().stream().filter(p->p.getName().equals(n)).findFirst().orElse(null);}
 private Card hand(Game g,String n){return g.getPlayer(playerA.getId()).getHand().getCards(g).stream().filter(c->c.getName().equals(n)).findFirst().get();}
 private JsonArray cards(Iterable<UUID> ids,Game g){JsonArray a=new JsonArray();for(UUID id:ids)a.add(g.getCard(id).getName().toLowerCase(Locale.ROOT).replace(' ','-'));return a;}
 private void point(Game g,String label){
  JsonObject p=new JsonObject();p.addProperty("checkpoint",label);Player player=g.getPlayer(playerA.getId());p.addProperty("stack",g.getStack().size());
  if(spec.get("kind").getAsString().equals("land")){
   JsonArray battlefield=new JsonArray();for(Permanent land:g.getBattlefield().getAllActivePermanents())battlefield.add(land.getName().toLowerCase(Locale.ROOT));
   p.add("battlefield",battlefield);p.add("hand",cards(player.getHand(),g));p.addProperty("red",player.getManaPool().getRed());p.addProperty("green",player.getManaPool().getGreen());p.addProperty("tapped",find(g,name(spec.get("first").getAsString())).isTapped());
  }else if(spec.get("kind").getAsString().equals("archer-thrill")){
   JsonArray life=new JsonArray();life.add(player.getLife());life.add(g.getPlayer(playerB.getId()).getLife());p.add("life",life);
   p.add("hand",cards(player.getHand(),g));p.add("graveyard",cards(player.getGraveyard(),g));p.add("library",cards(player.getLibrary().getCardList(),g));
   p.addProperty("mana",player.getManaPool().getRed());p.addProperty("tapped_mountains",g.getBattlefield().getAllActivePermanents().stream().filter(x->x.getName().equals("Mountain")&&x.isTapped()).count());
  }else{
   Permanent creature=find(g,name(spec.get("card").getAsString())),cavalry=find(g,"Axgard Cavalry"),enemy=find(g,"Bear Cub");
   p.addProperty("green",player.getManaPool().getGreen());p.addProperty("cavalry_tapped",cavalry.isTapped());p.addProperty("creature_tapped",creature.isTapped());
   p.addProperty("haste",creature.getAbilities().containsKey(HasteAbility.getInstance().getId()));p.addProperty("enemy_damage",enemy==null?0:enemy.getDamage());
  }points.add(p);
 }
 @Override protected TestPlayer createPlayer(String n,RangeOfInfluence r){return new TestPlayer(new TestComputerPlayer(n,r)){
  @Override public boolean priority(Game g){
   assertTrue("bounded priority script",++calls<20);if(captured)return false;
   if(g.getTurnNum()!=1||g.getTurnStepType()!=PhaseStep.PRECOMBAT_MAIN||!getId().equals(playerA.getId())){pass(g);return false;}
   getManaPool().setAutoPayment(false);
   if(spec.get("kind").getAsString().equals("land")){
    assertTrue(playLand(hand(g,name(spec.get("first").getAsString())),g,false));Permanent land=find(g,name(spec.get("first").getAsString()));
    assertTrue(activateAbility(land.getAbilities().getActivatedManaAbilities(Zone.BATTLEFIELD).get(0),g));point(g,"first-land");
    assertFalse(playLand(hand(g,"Mountain"),g,false));point(g,"second-rejected");
   }else if(spec.get("kind").getAsString().equals("archer-thrill")){
    for(Permanent p:g.getBattlefield().getAllActivePermanents())if(p.isLand(g))assertTrue(activateAbility(p.getAbilities().getActivatedManaAbilities(Zone.BATTLEFIELD).get(0),g));
    assertEquals(spec.getAsJsonArray("library"),cards(getLibrary().getCardList(),g));
    assertTrue(cast(hand(g,"Thrill of Possibility").getSpellAbility(),g,false,null));assertEquals(1,discards);
    g.checkStateAndTriggered();point(g,"committed");
    for(int i=0;i<spec.get("archers").getAsInt();i++){g.getStack().resolve(g);g.checkStateAndTriggered();point(g,"trigger-"+(i+1));}
    g.getStack().resolve(g);g.checkStateAndTriggered();point(g,"draws");
   }else{
    String creatureName=name(spec.get("card").getAsString());
    assertTrue(cast(hand(g,creatureName).getSpellAbility(),g,false,null));g.getStack().resolve(g);g.checkStateAndTriggered();
    Permanent creature=find(g,creatureName),cavalry=find(g,"Axgard Cavalry");point(g,"fresh");
    assertFalse(creature.getAbilities().getActivatedManaAbilities(Zone.BATTLEFIELD).get(0).canActivate(getId(),g).canActivate());
    addTarget(creatureName);ActivatedAbility haste=cavalry.getAbilities().getActivatedAbilities(Zone.BATTLEFIELD).stream().filter(a->a.getRule().contains("haste")).findFirst().get();
    assertTrue(activateAbility(haste,g));point(g,"haste-pending");g.getStack().resolve(g);g.checkStateAndTriggered();point(g,"haste-resolved");
    if(spec.get("payment").getAsBoolean()){
     Permanent forest=g.getBattlefield().getAllActivePermanents().stream().filter(p->p.isLand(g)&&!p.isTapped()).findFirst().get();
     assertTrue(activateAbility(forest.getAbilities().getActivatedManaAbilities(Zone.BATTLEFIELD).get(0),g));
     paymentMana=true;assertTrue(cast(hand(g,"Bite Down").getSpellAbility(),g,false,null));assertFalse(paymentMana);
     point(g,"bite-paid");g.getStack().resolve(g);g.checkStateAndTriggered();point(g,"bite-resolved");
    }else{assertTrue(activateAbility(creature.getAbilities().getActivatedManaAbilities(Zone.BATTLEFIELD).get(0),g));point(g,"mana-produced");}
   }
   assertTrue(g.getStack().isEmpty());captured=true;g.pause();return false;
  }
  @Override public boolean playMana(Ability a,ManaCost unpaid,String prompt,Game g){
   assertTrue("bounded mana script",++manaCalls<20);
   if(spec.get("kind").getAsString().equals("haste-mana")){
    if(paymentMana){paymentMana=false;int stackBefore=g.getStack().size();Permanent p=find(g,name(spec.get("card").getAsString()));assertTrue(activateAbility(p.getAbilities().getActivatedManaAbilities(Zone.BATTLEFIELD).get(0),g));assertEquals(stackBefore,g.getStack().size());return true;}
    if(getManaPool().getGreen()>0){getManaPool().unlockManaType(ManaType.GREEN);return true;}
    Permanent land=g.getBattlefield().getAllActivePermanents().stream().filter(p->p.isLand(g)&&!p.isTapped()).findFirst().get();assertTrue(activateAbility(land.getAbilities().getActivatedManaAbilities(Zone.BATTLEFIELD).get(0),g));return true;
   }
   assertTrue(getManaPool().getRed()>0);getManaPool().unlockManaType(ManaType.RED);return true;
  }
  @Override public boolean choose(Outcome o,Target t,Ability a,Game g,Map<String,Serializable> options){
   assertTrue(t instanceof TargetCardInHand);assertEquals(Outcome.Discard,o);assertEquals(1,++discards);assertEquals(1,t.getMinNumberOfTargets());assertEquals(1,t.getMaxNumberOfTargets());
   UUID id=hand(g,name(spec.get("discard").getAsString())).getId();assertTrue(t.canTarget(id,a,g));t.addTarget(id,a,g);return true;
  }
  @Override public boolean chooseTarget(Outcome o,Target t,Ability a,Game g){
   if(a.getSourceId().equals(find(g,"Axgard Cavalry").getId()))return super.chooseTarget(o,t,a,g);
   assertTrue(++targets<=2);Permanent p=find(g,targets==1?name(spec.get("card").getAsString()):"Bear Cub");assertTrue(t.canTarget(p.getId(),a,g));t.addTarget(p.getId(),a,g);return true;
  }
  @Override public TriggeredAbility chooseTriggeredAbility(List<TriggeredAbility> abilities,Game g){
   for(int i=1;i<=spec.get("archers").getAsInt();i++){UUID id=playerA.getAliasByName("archer"+i);for(TriggeredAbility a:abilities)if(a.getSourceId().equals(id))return a;}throw new AssertionError("unscripted trigger");
  }
 };}
 @Test public void executeCase() throws Exception {
  setStrictChooseMode(true);currentGame.setStartingPlayerId(playerA.getId());gameOptions.skipInitShuffling=true;
  removeAllCardsFromHand(playerA);removeAllCardsFromHand(playerB);removeAllCardsFromLibrary(playerA);removeAllCardsFromLibrary(playerB);
  String kind=spec.get("kind").getAsString();
  if(kind.equals("land")){
   addCard(Zone.HAND,playerA,"FDN-"+name(spec.get("first").getAsString()),1);addCard(Zone.HAND,playerA,"FDN-Mountain",1);
  }else if(kind.equals("archer-thrill")){
   setLife(playerB,spec.get("opponent_life").getAsInt());
   for(int i=1;i<=spec.get("archers").getAsInt();i++)addCard(Zone.BATTLEFIELD,playerA,"FDN-Firebrand Archer@archer"+i,1);
   addCard(Zone.HAND,playerA,"FDN-Thrill of Possibility",1);addCard(Zone.HAND,playerA,"FDN-"+name(spec.get("discard").getAsString()),1);
   addCard(Zone.BATTLEFIELD,playerA,"FDN-Mountain",2);JsonArray library=spec.getAsJsonArray("library");for(int i=library.size()-1;i>=0;i--)addCard(Zone.LIBRARY,playerA,"FDN-"+name(library.get(i).getAsString()),1);
  }else if(kind.equals("haste-mana")){
   addCard(Zone.BATTLEFIELD,playerA,"FDN-Axgard Cavalry",1);addCard(Zone.HAND,playerA,"FDN-"+name(spec.get("card").getAsString()),1);
   boolean payment=spec.get("payment").getAsBoolean();addCard(Zone.BATTLEFIELD,playerA,"FDN-Forest",spec.get("card").getAsString().equals("llanowar-elves")?1:payment?3:2);
   if(payment){addCard(Zone.HAND,playerA,"FDN-Bite Down",1);addCard(Zone.BATTLEFIELD,playerB,"FDN-Bear Cub",1);}
  }else throw new AssertionError("unsupported composition "+kind);
  setStopAt(1,PhaseStep.POSTCOMBAT_MAIN);execute();assertTrue("required checkpoint not reached",captured);
  Files.write(Paths.get(System.getProperty("mtglab.output"),spec.get("id").getAsString()+".json"),new GsonBuilder().setPrettyPrinting().create().toJson(points).getBytes(StandardCharsets.UTF_8));
 }
}
