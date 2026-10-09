package org.mage.test.mtglab;
import com.google.gson.*;
import mage.abilities.*;
import mage.abilities.costs.mana.ManaCost;
import mage.abilities.keyword.FlyingAbility;
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
/** Original pinned-card/CR 602, 113.7a, 611.2, 613, 509/510 cases.
 * Only control age/departure use setup hooks. Rules execute in pinned XMage. */
@RunWith(Parameterized.class)
public class ShivanTest extends CardTestPlayerBase {
 private final JsonObject spec;private final String mode;
 private boolean initialized,started,captured,legal=true;private int boosts;private int manaCalls;private int priorityCalls;private JsonObject result;
 private int activationPriority, sourceChoices; private final JsonArray payment=new JsonArray();
 private void paymentPoint(Game g, Ability ability, String stage, String source){
  assertEquals("no priority inside payment",activationPriority,priorityCalls);
  assertEquals("only the announced nonmana ability uses the stack",1,g.getStack().size());
  JsonObject point=new JsonObject();point.addProperty("stage",stage);
  if(source==null)point.add("source",JsonNull.INSTANCE);else point.addProperty("source",source);
  assertTrue(ability.getTargets().isEmpty());point.add("target",JsonNull.INSTANCE);
  point.addProperty("pool",g.getPlayer(playerA.getId()).getManaPool().getRed()+g.getPlayer(playerA.getId()).getManaPool().getGreen());
  int tapped=0;for(Permanent p:g.getBattlefield().getAllActivePermanents())if(p.isLand(g)&&p.isTapped())tapped++;
  point.addProperty("tapped",tapped);point.addProperty("announced_stack",g.getStack().size());point.addProperty("priority_calls",priorityCalls-activationPriority);payment.add(point);
 }
 @Parameterized.Parameters(name="{0}") public static Collection<Object[]> cases() throws Exception {
  JsonObject f=JsonParser.parseString(new String(Files.readAllBytes(Paths.get(System.getProperty("mtglab.fixture"))),StandardCharsets.UTF_8)).getAsJsonObject();assertEquals(1,f.get("version").getAsInt());List<Object[]> out=new ArrayList<>();for(JsonElement c:f.getAsJsonArray("cases"))out.add(new Object[]{c.getAsJsonObject()});return out;
 }
 public ShivanTest(JsonObject s){spec=s;mode=s.get("mode").getAsString();}
 private boolean combat(){return mode.startsWith("split_")||Arrays.asList("sentry","cub_illegal","thorn","excess","grown_split","haste").contains(mode);}
 private List<Permanent> creatures(Game g,int seat){UUID id=seat==0?playerA.getId():playerB.getId();List<Permanent> out=new ArrayList<>();for(Permanent p:g.getBattlefield().getAllActivePermanents())if(p.getControllerId().equals(id)&&p.isCreature(g))out.add(p);if(seat==1&&spec.has("amounts"))out.sort(Comparator.comparingInt(p->p.getId().equals(playerB.getAliasByName("blocker.1"))?0:1));return out;}
 private Permanent dragon(Game g){return creatures(g,0).stream().filter(p->p.getName().equals("Shivan Dragon")).findFirst().orElse(null);}
 private ActivatedAbility boost(Permanent p){return p.getAbilities().getActivatedAbilities(Zone.BATTLEFIELD).stream().filter(a->a.getRule().contains("+1/+0")).findFirst().get();}
 private JsonElement stats(Permanent p){if(p==null)return JsonNull.INSTANCE;JsonArray a=new JsonArray();a.add(p.getPower().getValue());a.add(p.getToughness().getValue());a.add(p.getDamage());return a;}
 private void sick(Permanent p){try{java.lang.reflect.Field f=PermanentImpl.class.getDeclaredField("controlledFromStartOfControllerTurn");f.setAccessible(true);f.setBoolean(p,false);}catch(Exception e){throw new AssertionError(e);}}
 private void capture(Game g){result=new JsonObject();Permanent d=dragon(g);if(d!=null)assertTrue(d.getAbilities().containsKey(FlyingAbility.getInstance().getId()));result.add("dragon",stats(d));JsonArray others=new JsonArray();for(Permanent p:creatures(g,1))others.add(stats(p));result.add("others",others);JsonArray life=new JsonArray();life.add(g.getPlayer(playerA.getId()).getLife());life.add(g.getPlayer(playerB.getId()).getLife());result.add("life",life);result.addProperty("mana",g.getPlayer(playerA.getId()).getManaPool().getRed());result.addProperty("stack",g.getStack().size());result.addProperty("legal",legal);if(mode.equals("payment_sources")){assertEquals(2,payment.size());result.add("payment",payment);}captured=true;g.pause();}
 @Override protected TestPlayer createPlayer(String n,RangeOfInfluence r){return new TestPlayer(new TestComputerPlayer(n,r)){
  @Override public boolean priority(Game g){
   assertTrue("bounded explicit priority script: "+mode,++priorityCalls<500);
   if(captured)return false;
   Permanent d=dragon(g);
   if(g.getTurnNum()==1&&g.getTurnStepType()==PhaseStep.PRECOMBAT_MAIN){
    // Float exact scripted lands before spells/targets. No AI mana decisions.
    Permanent land=g.getBattlefield().getAllActivePermanents().stream().filter(p->p.getControllerId().equals(getId())&&p.isLand(g)&&!p.isTapped()&&(!mode.equals("bite")||!p.getName().equals("Mountain")||!g.getStack().isEmpty()&&g.getStack().getFirst().getName().equals("Bite Down"))).findFirst().orElse(null);
    if(land!=null&&!mode.equals("grown_split")&&!mode.equals("payment_sources")){assertTrue(activateAbility(land.getAbilities().getActivatedManaAbilities(Zone.BATTLEFIELD).get(0),g));return true;}
    if(getId().equals(playerA.getId())){
     if(!initialized){initialized=true;if(Arrays.asList("sick","cleanup","haste").contains(mode))sick(d);}
     if(mode.equals("cast")){if(d!=null&&g.getStack().isEmpty()){capture(g);return false;}}
     else if(mode.equals("short_cast")){assertNull(d);assertEquals(5,getManaPool().getRed());assertFalse(getPlayable(g,true).stream().anyMatch(a->a instanceof SpellAbility));legal=false;capture(g);return false;}
     else if(mode.equals("no_red")){assertFalse(getPlayable(g,true).stream().anyMatch(a->a.getSourceId().equals(d.getId())));legal=false;capture(g);return false;}
     else if(mode.equals("foreign")){assertTrue(boost(d).getTargets().isEmpty());legal=false;capture(g);return false;}
     else if(mode.equals("dead_before")){ActivatedAbility a=boost(d).copy();assertTrue(d.destroy(null,g));assertFalse(getPlayable(g,true).stream().anyMatch(x->x.getSourceId().equals(a.getSourceId())));legal=false;capture(g);return false;}
     else if(!combat()||mode.equals("haste")){
      if(mode.equals("thorn_bite")){if(d==null&&g.getStack().isEmpty()){capture(g);return false;}}
      else {
       if(mode.equals("bite")&&boosts==0&&!g.getStack().isEmpty()&&g.getStack().getFirst().getName().equals("Giant Growth")){pass(g);return false;}
       boolean ready=!mode.equals("bite") || boosts>0 || g.getStack().size()>0 && g.getStack().getFirst().getName().equals("Bite Down");
       int count=mode.equals("double")||mode.equals("bite")?2:1;
       if(ready&&boosts<count){assertNotNull(d);assertEquals(5,d.getPower().getValue());assertTrue(boost(d).getTargets().isEmpty());activationPriority=priorityCalls;assertTrue(activateAbility(boost(d),g));if(mode.equals("payment_sources"))paymentPoint(g,g.getStack().getFirst().getStackAbility(),"committed",null);boosts++;if(mode.equals("dead_after")){assertTrue(d.destroy(null,g));assertEquals(1,g.getStack().size());}return true;}
       if(boosts==count&&g.getStack().isEmpty()){
        if(mode.equals("haste")&&!started){started=true;Permanent cav=creatures(g,0).stream().filter(p->p.getName().equals("Axgard Cavalry")).findFirst().get();ActivatedAbility a=cav.getAbilities().getActivatedAbilities(Zone.BATTLEFIELD).stream().filter(x->x.getRule().contains("haste")).findFirst().get();addTarget("Shivan Dragon");assertTrue(activateAbility(a,g));return true;}
        if(!mode.equals("cleanup")&&!mode.equals("haste")){capture(g);return false;}
       }
      }
     }
    }
   }
   if(mode.equals("cleanup")&&g.getTurnNum()==2&&g.getTurnStepType()==PhaseStep.UPKEEP){capture(g);return false;}
   if(mode.equals("grown_split")&&g.getTurnStepType()==PhaseStep.DECLARE_BLOCKERS&&getId().equals(playerB.getId())){Permanent land=g.getBattlefield().getAllActivePermanents().stream().filter(p->p.isLand(g)&&!p.isTapped()).findFirst().orElse(null);if(land!=null){assertTrue(activateAbility(land.getAbilities().getActivatedManaAbilities(Zone.BATTLEFIELD).get(0),g));return true;}}
   if(combat()&&g.getTurnStepType()==PhaseStep.COMBAT_DAMAGE){capture(g);return false;}
   return super.priority(g);
  }
  @Override public boolean playMana(Ability a,ManaCost unpaid,String prompt,Game g){
   if(mode.equals("payment_sources")){
    assertEquals(activationPriority,priorityCalls);assertTrue(sourceChoices<1);
    String alias="pay"+(++sourceChoices);Permanent land=g.getPermanent(playerA.getAliasByName(alias));
    assertNotNull(land);assertFalse(land.isTapped());
    assertTrue(activateAbility(land.getAbilities().getActivatedManaAbilities(Zone.BATTLEFIELD).get(0),g));
    paymentPoint(g,a,"mana",alias);
   }

   assertTrue("bounded explicit payment: "+unpaid+" red="+getManaPool().getRed()+" green="+getManaPool().getGreen(),++manaCalls<32);assertTrue(getManaPool().getRed()>0||getManaPool().getGreen()>0);if(getManaPool().getRed()>0)getManaPool().unlockManaType(ManaType.RED);if(getManaPool().getGreen()>0)getManaPool().unlockManaType(ManaType.GREEN);return true;
  }
  @Override public void selectAttackers(Game g,UUID active){assertTrue(combat());Permanent d=dragon(g);assertTrue(d.canAttack(playerB.getId(),g));declareAttacker(d.getId(),playerB.getId(),g,false);}
  @Override public void selectBlockers(Ability a,Game g,UUID defending){for(Permanent b:creatures(g,1)){legal=b.canBlock(dragon(g).getId(),g);assertEquals(!mode.equals("cub_illegal"),legal);if(legal)declareBlocker(getId(),b.getId(),dragon(g).getId(),g);}}
  @Override public List<Integer> getMultiAmountWithIndividualConstraints(Outcome o,List<MultiAmountMessage> messages,int min,int max,MultiAmountType type,Game g){
   assertEquals(playerA.getId(),getId());assertEquals(2,messages.size());assertEquals(5,min);assertEquals(5,max);List<Integer> ns=new ArrayList<>();for(JsonElement v:spec.getAsJsonArray("amounts"))ns.add(v.getAsInt());if(mode.equals("excess")){assertTrue(ns.stream().mapToInt(Integer::intValue).sum()>max);legal=false;capture(g);throw new RejectedAllocation();}for(int i=0;i<2;i++)assertTrue(ns.get(i)>=messages.get(i).min&&ns.get(i)<=messages.get(i).max);return ns;
  }
 };}
 private static class RejectedAllocation extends RuntimeException {}
 @Test public void executeCase() throws Exception {
  setStrictChooseMode(true);currentGame.setStartingPlayerId(playerA.getId());gameOptions.skipInitShuffling=true;
  removeAllCardsFromHand(playerA);removeAllCardsFromHand(playerB);removeAllCardsFromLibrary(playerA);removeAllCardsFromLibrary(playerB);addCard(Zone.LIBRARY,playerA,"FDN-Forest",2);addCard(Zone.LIBRARY,playerB,"FDN-Forest",2);
  boolean casting=mode.equals("cast")||mode.equals("short_cast");addCard(casting?Zone.HAND:Zone.BATTLEFIELD,playerA,"FDN-Shivan Dragon@own",1);
  int red=casting?(mode.equals("cast")?6:5):Arrays.asList("double","bite").contains(mode)?2:combat()&&!mode.equals("haste")||mode.equals("no_red")||mode.equals("thorn_bite")?0:1;
  if(mode.equals("payment_sources"))addCard(Zone.BATTLEFIELD,playerA,"FDN-Mountain@pay1",1);else if(red>0)addCard(Zone.BATTLEFIELD,playerA,"FDN-Mountain",red);
  if(mode.equals("cast"))castSpell(1,PhaseStep.PRECOMBAT_MAIN,playerA,"Shivan Dragon");
  if(mode.equals("bite")){addCard(Zone.BATTLEFIELD,playerB,"FDN-Shivan Dragon@enemy",1);addCard(Zone.BATTLEFIELD,playerA,"FDN-Forest",3);addCard(Zone.HAND,playerA,"FDN-Giant Growth",1);addCard(Zone.HAND,playerA,"FDN-Bite Down",1);castSpell(1,PhaseStep.PRECOMBAT_MAIN,playerA,"Giant Growth","@enemy");castSpell(1,PhaseStep.PRECOMBAT_MAIN,playerA,"Bite Down","@own^@enemy");}
  if(mode.equals("thorn_bite")){addCard(Zone.BATTLEFIELD,playerB,"FDN-Thornweald Archer",1);addCard(Zone.BATTLEFIELD,playerB,"FDN-Forest",2);addCard(Zone.HAND,playerB,"FDN-Bite Down",1);castSpell(1,PhaseStep.PRECOMBAT_MAIN,playerB,"Bite Down","Thornweald Archer^Shivan Dragon");}
  if(mode.equals("haste"))addCard(Zone.BATTLEFIELD,playerA,"FDN-Axgard Cavalry",1);
  if(combat()&&!mode.equals("haste")){String b=mode.equals("cub_illegal")?"Bear Cub":mode.equals("thorn")?"Thornweald Archer":"Magnigoth Sentry";addCard(Zone.BATTLEFIELD,playerB,"FDN-"+b+(spec.has("amounts")?"@blocker":""),spec.has("amounts")?2:1);}
  if(mode.equals("grown_split")){addCard(Zone.BATTLEFIELD,playerB,"FDN-Forest",1);addCard(Zone.HAND,playerB,"FDN-Giant Growth",1);castSpell(1,PhaseStep.DECLARE_BLOCKERS,playerB,"Giant Growth","@blocker.1");}
  setStopAt(2,PhaseStep.UPKEEP);try{execute();}catch(RejectedAllocation e){assertEquals("excess",mode);}assertTrue("checkpoint not reached",captured);
  Files.write(Paths.get(System.getProperty("mtglab.output"),mode+".json"),new GsonBuilder().serializeNulls().setPrettyPrinting().create().toJson(result).getBytes(StandardCharsets.UTF_8));
 }
}
