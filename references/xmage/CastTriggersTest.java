package org.mage.test.mtglab;
import com.google.gson.*;
import mage.abilities.*;
import mage.abilities.costs.mana.ManaCost;
import mage.cards.Card;
import mage.constants.*;
import mage.game.Game;
import mage.game.permanent.Permanent;
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
/** Original CR 601.2i/603/113.7a ledgers. Synthetic setup; real cast,
 * trigger selection, resolution and cleanup. No native calls or expected outputs. */
@RunWith(Parameterized.class)
public class CastTriggersTest extends CardTestPlayerBase {
 private final JsonObject spec;private final String id;private JsonArray points=new JsonArray();private boolean started,captured;private int calls,manaCalls,targetCalls;
 @Parameterized.Parameters(name="{0}") public static Collection<Object[]> cases() throws Exception {
  JsonObject f=JsonParser.parseString(new String(Files.readAllBytes(Paths.get(System.getProperty("mtglab.fixture"))),StandardCharsets.UTF_8)).getAsJsonObject();assertEquals(1,f.get("version").getAsInt());List<Object[]> out=new ArrayList<>();for(JsonElement c:f.getAsJsonArray("cases"))out.add(new Object[]{c.getAsJsonObject()});return out;
 }
 public CastTriggersTest(JsonObject s){spec=s;id=s.get("id").getAsString();}
 private String name(String key){switch(key){case "giant-growth":return "Giant Growth";case "dragon-fodder":return "Dragon Fodder";case "swab-goblin":return "Swab Goblin";case "mountain":return "Mountain";case "bite-down":return "Bite Down";case "thrill-of-possibility":return "Thrill of Possibility";default:throw new AssertionError(key);}}
 private Permanent cyclops(Game g){return g.getBattlefield().getAllActivePermanents().stream().filter(p->p.getName().equals("Crackling Cyclops")).findFirst().orElse(null);}
 private UUID target(Game g){return spec.get("cyclops").getAsBoolean()?playerA.getAliasByName("cyclops"):playerA.getAliasByName("target");}
 private UUID source(int row){return row<spec.get("archers").getAsInt()?playerA.getAliasByName("archer"+(row+1)):playerA.getAliasByName("cyclops");}
 private JsonArray point(Game g){JsonArray p=new JsonArray();JsonArray life=new JsonArray();life.add(g.getPlayer(playerA.getId()).getLife());life.add(g.getPlayer(playerB.getId()).getLife());p.add(life);Permanent c=cyclops(g);if(c==null)p.add(JsonNull.INSTANCE);else {JsonArray stats=new JsonArray();stats.add(c.getPower().getValue());stats.add(c.getToughness().getValue());p.add(stats);}p.add(g.getStack().size());p.add(g.getBattlefield().getAllActivePermanents().stream().filter(x->x.isToken()&&x.getName().equals("Goblin Token")).count());return p;}
 @Override protected TestPlayer createPlayer(String n,RangeOfInfluence r){return new TestPlayer(new TestComputerPlayer(n,r)){
  @Override public boolean priority(Game g){
   assertTrue("bounded explicit priority",++calls<150);
   if(captured)return false;
   if(started){if(spec.has("cleanup")&&g.getTurnNum()==2&&g.getTurnStepType()==PhaseStep.UPKEEP){points.add(point(g));captured=true;g.pause();return false;}pass(g);return false;}
   if(g.getTurnNum()!=1||g.getTurnStepType()!=PhaseStep.PRECOMBAT_MAIN||!getId().equals(playerA.getId())){pass(g);return false;}
   Permanent land=g.getBattlefield().getAllActivePermanents().stream().filter(p->p.isLand(g)&&!p.isTapped()).findFirst().orElse(null);
   if(land!=null){assertTrue(activateAbility(land.getAbilities().getActivatedManaAbilities(Zone.BATTLEFIELD).get(0),g));return true;}
   started=true;
   if(id.equals("apnap")){for(Permanent permanent:g.getBattlefield().getAllActivePermanents())if(permanent.getName().equals("Firebrand Archer")){TriggeredAbility a=permanent.getAbilities().getTriggeredAbilities(Zone.BATTLEFIELD).get(0).copy();g.getState().addTriggeredAbility(a);}}
   else if(id.equals("land")){Card c=getHand().getCards(g).iterator().next();assertTrue(playLand(c,g,false));}
   else if(!id.equals("mana")){
    Card c=getHand().getCards(g).stream().filter(x->x.getName().equals(name(spec.get("spell").getAsString()))).findFirst().get();
    if(id.equals("no_mana")){for(int i=0;i<3;i++)assertFalse(getPlayable(g,true).stream().anyMatch(a->a.getSourceId().equals(c.getId())));}
    else if(id.equals("thrill_fail")){for(int i=0;i<3;i++)assertFalse("no other card to discard",cast(c.getSpellAbility(),g,false,null));}
    else {boolean ok=cast(c.getSpellAbility(),g,false,null);assertEquals(!id.equals("cancel"),ok);}
   }
   g.checkStateAndTriggered();
   if(spec.has("death")){UUID dead=spec.get("death").getAsString().equals("archer")?source(0):target(g);assertTrue(g.getPermanent(dead).destroy(null,g));g.checkStateAndTriggered();}
   points.add(point(g));int limit=0;
   while(!g.getStack().isEmpty()){assertTrue(++limit<8);g.getStack().resolve(g);g.checkStateAndTriggered();points.add(point(g));}
   if(!spec.has("cleanup")){captured=true;g.pause();}else pass(g);
   return false;
  }
  @Override public TriggeredAbility chooseTriggeredAbility(List<TriggeredAbility> abilities,Game g){
   assertEquals(playerA.getId(),getId());for(JsonElement row:spec.getAsJsonArray("order")){UUID h=source(row.getAsInt());for(TriggeredAbility a:abilities)if(a.getSourceId().equals(h))return a;}throw new AssertionError("unscripted trigger choice");
  }
  @Override public boolean chooseTarget(Outcome outcome,Target t,Ability a,Game g){
   assertTrue(++targetCalls<4);if(id.equals("cancel"))return false;
   UUID selected=target(g);
   if(id.equals("bite_reject")){if(targetCalls==1)selected=source(0);else{assertFalse("Bite must reject friendly destination",t.canTarget(target(g),a,g));selected=playerB.getAliasByName("enemy");}}
   assertTrue(t.canTarget(selected,a,g));t.addTarget(selected,a,g);return true;
  }
  @Override public boolean playMana(Ability a,ManaCost unpaid,String prompt,Game g){assertTrue(++manaCalls<8);assertTrue(getManaPool().getRed()>0||getManaPool().getGreen()>0);if(getManaPool().getRed()>0)getManaPool().unlockManaType(ManaType.RED);if(getManaPool().getGreen()>0)getManaPool().unlockManaType(ManaType.GREEN);return true;}
 };}
 @Test public void executeCase() throws Exception {
  setStrictChooseMode(true);currentGame.setStartingPlayerId(playerA.getId());gameOptions.skipInitShuffling=true;
  removeAllCardsFromHand(playerA);removeAllCardsFromHand(playerB);removeAllCardsFromLibrary(playerA);removeAllCardsFromLibrary(playerB);addCard(Zone.LIBRARY,playerA,"FDN-Forest",3);addCard(Zone.LIBRARY,playerB,"FDN-Forest",3);
  for(int i=1;i<=spec.get("archers").getAsInt();i++)addCard(Zone.BATTLEFIELD,playerA,"FDN-Firebrand Archer@archer"+i,1);
  if(spec.get("cyclops").getAsBoolean())addCard(Zone.BATTLEFIELD,playerA,"FDN-Crackling Cyclops@cyclops",1);else addCard(Zone.BATTLEFIELD,playerA,"FDN-Bear Cub@target",1);
  if(id.equals("apnap"))addCard(Zone.BATTLEFIELD,playerB,"FDN-Firebrand Archer@other",1);
  if(id.equals("bite_reject"))addCard(Zone.BATTLEFIELD,playerB,"FDN-Bear Cub@enemy",1);
  String spell=spec.get("spell").getAsString();if(!id.equals("mana"))addCard(Zone.HAND,playerA,"FDN-"+name(spell),1);
  if(!id.equals("no_mana")&&!id.equals("land"))addCard(Zone.BATTLEFIELD,playerA,spell.equals("giant-growth")||spell.equals("bite-down")?"FDN-Forest":"FDN-Mountain",spell.equals("giant-growth")||id.equals("mana")?1:2);
  setStopAt(2,PhaseStep.PRECOMBAT_MAIN);execute();assertTrue("checkpoint not reached",captured);
  Files.write(Paths.get(System.getProperty("mtglab.output"),id+".json"),new GsonBuilder().serializeNulls().setPrettyPrinting().create().toJson(points).getBytes(StandardCharsets.UTF_8));
 }
}
