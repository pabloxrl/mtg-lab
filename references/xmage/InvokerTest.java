package org.mage.test.mtglab;
import com.google.gson.*;
import mage.abilities.*;
import mage.abilities.costs.mana.ManaCost;
import mage.abilities.keyword.TrampleAbility;
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
/** Original pinned Oracle / CR 602, 608.2b, 113.7a, 400.7, 611.2, 702.19.
 * Only control age and zone departure/return are synthetic setup hooks. */
@RunWith(Parameterized.class)
public class InvokerTest extends CardTestPlayerBase {
 private final String mode,targetName; private boolean initialized,captured,legal=true; private int boosts,manaCalls,priorityCalls;private boolean dragonBoost;private JsonObject result;
 private int activationPriority, sourceChoices; private final JsonArray payment=new JsonArray();
 private void paymentPoint(Game g, Ability ability, String stage, String source){
  assertEquals("no priority inside payment",activationPriority,priorityCalls);
  assertEquals("only the announced nonmana ability uses the stack",1,g.getStack().size());
  JsonObject point=new JsonObject();point.addProperty("stage",stage);
  if(source==null)point.add("source",JsonNull.INSTANCE);else point.addProperty("source",source);
  assertEquals(find(g,"Bear Cub").getId(),ability.getTargets().get(0).getFirstTarget());point.addProperty("target","bear-cub");
  point.addProperty("pool",g.getPlayer(playerA.getId()).getManaPool().getRed()+g.getPlayer(playerA.getId()).getManaPool().getGreen());
  int tapped=0;for(Permanent p:g.getBattlefield().getAllActivePermanents())if(p.isLand(g)&&p.isTapped())tapped++;
  point.addProperty("tapped",tapped);point.addProperty("announced_stack",g.getStack().size());point.addProperty("priority_calls",priorityCalls-activationPriority);payment.add(point);
 }
 @Parameterized.Parameters(name="{0}") public static Collection<Object[]> cases() throws Exception {
  JsonObject f=JsonParser.parseString(new String(Files.readAllBytes(Paths.get(System.getProperty("mtglab.fixture"))),StandardCharsets.UTF_8)).getAsJsonObject();assertEquals(1,f.get("version").getAsInt());List<Object[]> out=new ArrayList<>();for(JsonElement c:f.getAsJsonArray("cases"))out.add(new Object[]{c.getAsJsonObject()});return out;
 }
 public InvokerTest(JsonObject s){mode=s.get("id").getAsString();targetName=mode.startsWith("thorn")?"Thornweald Archer":"Bear Cub";}
 private Permanent find(Game g,String name){return g.getBattlefield().getAllActivePermanents().stream().filter(p->p.getName().equals(name)).findFirst().orElse(null);}
 private ActivatedAbility boost(Permanent p){return p.getAbilities().getActivatedAbilities(Zone.BATTLEFIELD).stream().filter(a->a.getRule().contains("+5/+5")).findFirst().get();}
 private JsonElement stats(Permanent p){if(p==null)return JsonNull.INSTANCE;JsonArray a=new JsonArray();a.add(p.getPower().getValue());a.add(p.getToughness().getValue());a.add(p.getDamage());return a;}
 private void sick(Permanent p){try{java.lang.reflect.Field f=PermanentImpl.class.getDeclaredField("controlledFromStartOfControllerTurn");f.setAccessible(true);f.setBoolean(p,false);}catch(Exception e){throw new AssertionError(e);}}
 private void capture(Game g){result=new JsonObject();Permanent t=find(g,targetName);result.add("dragon",stats(find(g,"Shivan Dragon")));result.add("source",stats(find(g,"Wildheart Invoker")));result.add("target",stats(t));result.addProperty("trample",t!=null&&t.getAbilities().containsKey(TrampleAbility.getInstance().getId()));JsonArray life=new JsonArray();life.add(g.getPlayer(playerA.getId()).getLife());life.add(g.getPlayer(playerB.getId()).getLife());result.add("life",life);result.addProperty("mana",g.getPlayer(playerA.getId()).getManaPool().getRed()+g.getPlayer(playerA.getId()).getManaPool().getGreen());result.addProperty("stack",g.getStack().size());result.addProperty("legal",legal);if(mode.equals("payment_sources")){assertEquals(9,payment.size());result.add("payment",payment);}captured=true;g.pause();}
 @Override protected TestPlayer createPlayer(String n,RangeOfInfluence r){return new TestPlayer(new TestComputerPlayer(n,r)){
  @Override public boolean priority(Game g){
   assertTrue("bounded explicit priority: "+mode,++priorityCalls<500);if(captured)return false;
   Permanent s=find(g,"Wildheart Invoker"),t=find(g,targetName);
   if(g.getTurnNum()==1&&g.getTurnStepType()==PhaseStep.PRECOMBAT_MAIN&&getId().equals(playerA.getId())){
    Permanent land=g.getBattlefield().getAllActivePermanents().stream().filter(p->p.getControllerId().equals(getId())&&p.isLand(g)&&!p.isTapped()).findFirst().orElse(null);
    if(land!=null&&!mode.equals("payment_sources")){assertTrue(activateAbility(land.getAbilities().getActivatedManaAbilities(Zone.BATTLEFIELD).get(0),g));return true;}
    if(mode.equals("thorn_cleanup")&&!dragonBoost){dragonBoost=true;Permanent d=find(g,"Shivan Dragon");ActivatedAbility a=d.getAbilities().getActivatedAbilities(Zone.BATTLEFIELD).stream().filter(x->x.getRule().contains("+1/+0")).findFirst().get();assertTrue(activateAbility(a,g));return true;}
    if(mode.equals("thorn_cleanup")&&boosts==0&&!g.getStack().isEmpty()){pass(g);return false;}
    if(mode.equals("thorn_cleanup")&&boosts==0)assertEquals(6,find(g,"Shivan Dragon").getPower().getValue());
    if(!initialized){initialized=true;if(mode.equals("opposing"))sick(s);if(mode.equals("tapped"))s.setTapped(true);}
    if(mode.equals("cast")){if(s!=null&&g.getStack().isEmpty()){capture(g);return false;}}
    else if(mode.equals("short_cast")){assertNull(s);assertEquals(3,getManaPool().getGreen());assertFalse(getPlayable(g,true).stream().anyMatch(a->a instanceof SpellAbility));legal=false;capture(g);return false;}
    else if(mode.equals("seven")){assertFalse(getPlayable(g,true).stream().anyMatch(a->a.getSourceId().equals(s.getId())));legal=false;capture(g);return false;}
    else if(mode.equals("missing")){assertEquals(1,boost(s).getTargets().get(0).getMinNumberOfTargets());legal=false;capture(g);return false;}
    else if(mode.equals("illegal")){Permanent forest=find(g,"Forest");assertFalse(boost(s).getTargets().get(0).canTarget(forest.getId(),boost(s),g));legal=false;capture(g);return false;}
    else {
     int count=mode.equals("thorn_double")?2:1;
     if(boosts<count){assertNotNull(s);addTarget(targetName);activationPriority=priorityCalls;assertTrue(activateAbility(boost(s),g));if(mode.equals("payment_sources"))paymentPoint(g,g.getStack().getFirst().getStackAbility(),"committed",null);boosts++;
      if(Arrays.asList("dead_source","thorn_dead","thorn_cleanup").contains(mode)){assertTrue(s.destroy(null,g));assertEquals(1,g.getStack().size());}
      if(mode.equals("departed")){UUID id=t.getId();assertTrue(t.destroy(null,g));assertTrue(g.getCard(id).putOntoBattlefield(g,Zone.GRAVEYARD,null,playerA.getId()));}
      return true;
     }
     if(g.getStack().isEmpty()&&!Arrays.asList("cleanup","thorn_cleanup","thorn_combat").contains(mode)){capture(g);return false;}
    }
   }
   if(Arrays.asList("cleanup","thorn_cleanup").contains(mode)&&g.getTurnNum()==2&&g.getTurnStepType()==PhaseStep.UPKEEP){capture(g);return false;}
   if(mode.equals("thorn_combat")&&g.getTurnStepType()==PhaseStep.COMBAT_DAMAGE){assertNull(find(g,"Magnigoth Sentry"));capture(g);return false;}
   return super.priority(g);
  }
  @Override public boolean playMana(Ability a,ManaCost unpaid,String prompt,Game g){
   if(mode.equals("payment_sources")){
    assertEquals(activationPriority,priorityCalls);assertTrue(sourceChoices<8);
    String alias="pay"+(++sourceChoices);Permanent land=g.getPermanent(playerA.getAliasByName(alias));
    assertNotNull(land);assertFalse(land.isTapped());
    assertTrue(activateAbility(land.getAbilities().getActivatedManaAbilities(Zone.BATTLEFIELD).get(0),g));
    paymentPoint(g,a,"mana",alias);
   }
assertTrue("bounded payment",++manaCalls<80);assertTrue(getManaPool().getRed()>0||getManaPool().getGreen()>0);if(getManaPool().getRed()>0)getManaPool().unlockManaType(ManaType.RED);if(getManaPool().getGreen()>0)getManaPool().unlockManaType(ManaType.GREEN);return true;}
  @Override public void selectAttackers(Game g,UUID active){if(mode.equals("thorn_combat")){Permanent t=find(g,targetName);assertTrue(t.canAttack(playerB.getId(),g));declareAttacker(t.getId(),playerB.getId(),g,false);}}
  @Override public void selectBlockers(Ability a,Game g,UUID defending){if(mode.equals("thorn_combat"))for(Permanent p:g.getBattlefield().getAllActivePermanents())if(p.getName().equals("Magnigoth Sentry"))declareBlocker(getId(),p.getId(),find(g,targetName).getId(),g);}
  @Override public List<Integer> getMultiAmountWithIndividualConstraints(Outcome o,List<MultiAmountMessage> messages,int min,int max,MultiAmountType type,Game g){assertEquals("thorn_combat",mode);assertEquals(2,messages.size());assertEquals(7,max);assertEquals(2,min);for(MultiAmountMessage m:messages)assertTrue(m.min<=1&&m.max>=1);return Arrays.asList(1,1);}
 };}
 @Test public void executeCase() throws Exception {
  setStrictChooseMode(true);currentGame.setStartingPlayerId(playerA.getId());gameOptions.skipInitShuffling=true;
  removeAllCardsFromHand(playerA);removeAllCardsFromHand(playerB);removeAllCardsFromLibrary(playerA);removeAllCardsFromLibrary(playerB);addCard(Zone.LIBRARY,playerA,"FDN-Forest",2);addCard(Zone.LIBRARY,playerB,"FDN-Forest",2);
  boolean casting=mode.equals("cast")||mode.equals("short_cast");addCard(casting?Zone.HAND:Zone.BATTLEFIELD,playerA,"FDN-Wildheart Invoker",1);
  addCard(Zone.BATTLEFIELD,mode.equals("opposing")?playerB:playerA,"FDN-"+targetName,1);
  int green=casting?(mode.equals("cast")?4:3):mode.equals("seven")?7:mode.equals("thorn_double")?16:mode.equals("mixed")?4:8;
  if(mode.equals("payment_sources")){for(int i=1;i<=8;i++)addCard(Zone.BATTLEFIELD,playerA,"FDN-Forest@pay"+i,1);}else addCard(Zone.BATTLEFIELD,playerA,"FDN-Forest",green);if(mode.equals("mixed"))addCard(Zone.BATTLEFIELD,playerA,"FDN-Mountain",4);
  if(mode.equals("thorn_cleanup")){addCard(Zone.BATTLEFIELD,playerA,"FDN-Shivan Dragon",1);addCard(Zone.BATTLEFIELD,playerA,"FDN-Mountain",1);}
  if(mode.equals("cast"))castSpell(1,PhaseStep.PRECOMBAT_MAIN,playerA,"Wildheart Invoker");
  if(mode.equals("thorn_combat"))addCard(Zone.BATTLEFIELD,playerB,"FDN-Magnigoth Sentry",2);
  setStopAt(2,PhaseStep.UPKEEP);execute();assertTrue("checkpoint not reached",captured);
  Files.write(Paths.get(System.getProperty("mtglab.output"),mode+".json"),new GsonBuilder().serializeNulls().setPrettyPrinting().create().toJson(result).getBytes(StandardCharsets.UTF_8));
 }
}
