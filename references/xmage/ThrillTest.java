package org.mage.test.mtglab;
import com.google.gson.*;
import mage.abilities.*;
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
/** Original pinned Thrill / CR 601.2h, 121, 608, 704.5b ledgers.
 * Synthetic setup only; actual spell, cost, draw and SBA execute in XMage. */
@RunWith(Parameterized.class)
public class ThrillTest extends CardTestPlayerBase {
 private final JsonObject spec; private JsonObject result; private int calls,manaCalls,discards; private boolean captured;
 @Parameterized.Parameters(name="{0}") public static Collection<Object[]> cases() throws Exception {
  JsonObject f=JsonParser.parseString(new String(Files.readAllBytes(Paths.get(System.getProperty("mtglab.fixture"))),StandardCharsets.UTF_8)).getAsJsonObject();assertEquals(1,f.get("version").getAsInt());List<Object[]> out=new ArrayList<>();for(JsonElement c:f.getAsJsonArray("cases"))out.add(new Object[]{c.getAsJsonObject()});return out;
 }
 public ThrillTest(JsonObject s){spec=s;}
 private String key(String name){return name.toLowerCase(Locale.ROOT).replace(' ','-');}
 private JsonArray cards(Iterable<UUID> ids,Game g){JsonArray a=new JsonArray();for(UUID id:ids)a.add(key(g.getCard(id).getName()));return a;}
 private JsonObject point(Game g){Player p=g.getPlayer(playerA.getId());JsonObject o=new JsonObject();o.add("hand",cards(p.getHand(),g));o.add("graveyard",cards(p.getGraveyard(),g));o.add("library",cards(p.getLibrary().getCardList(),g));JsonArray stack=new JsonArray();g.getStack().forEach(s->stack.add(key(s.getName())));o.add("stack",stack);o.addProperty("mana",p.getManaPool().getRed()+p.getManaPool().getGreen());o.addProperty("lost",p.hasLost());return o;}
 @Override protected TestPlayer createPlayer(String n,RangeOfInfluence r){return new TestPlayer(new TestComputerPlayer(n,r)){
  @Override public boolean priority(Game g){
   assertTrue("bounded explicit script",++calls<12);assertEquals(playerA.getId(),getId());assertEquals(PhaseStep.UPKEEP,g.getTurnStepType());
   Permanent land=g.getBattlefield().getAllActivePermanents().stream().filter(p->p.isLand(g)&&!p.isTapped()).findFirst().orElse(null);
   if(land!=null){assertTrue(activateAbility(land.getAbilities().getActivatedManaAbilities(Zone.BATTLEFIELD).get(0),g));return true;}
   assertEquals("top-to-bottom setup translation",spec.getAsJsonArray("library"),cards(g.getPlayer(playerA.getId()).getLibrary().getCardList(),g));
   Card spell=getHand().getCards(g).stream().filter(c->c.getName().equals("Thrill of Possibility")).findFirst().get();
   boolean legal=getPlayable(g,true).stream().anyMatch(a->a.getSourceId().equals(spell.getId()));
   if(legal){legal=cast(spell.getSpellAbility(),g,false,null);if(legal)assertEquals(1,discards);}
   result=new JsonObject();result.addProperty("legal",legal);result.add("before",point(g));
   if(legal){assertEquals(1,g.getStack().size());g.getStack().resolve(g);}
   // Capture zones at completed resolution before losing-player cleanup, then
   // execute the real SBA and append the independently observed loss flag.
   JsonObject after=point(g);g.checkStateAndTriggered();after.addProperty("lost",g.getPlayer(playerA.getId()).hasLost());result.add("after",after);
   captured=true;g.pause();return false;
  }
  @Override public boolean playMana(Ability a,ManaCost unpaid,String prompt,Game g){assertTrue(++manaCalls<=4);assertTrue(getManaPool().getRed()>0);getManaPool().unlockManaType(ManaType.RED);return true;}
  @Override public boolean choose(Outcome outcome,Target target,Ability source,Game g,Map<String,Serializable> options){assertTrue(target instanceof TargetCardInHand);assertEquals(Outcome.Discard,outcome);assertEquals(1,target.getMinNumberOfTargets());assertEquals(1,target.getMaxNumberOfTargets());assertEquals(1,++discards);UUID id=getHand().getCards(g).stream().filter(c->c.getName().equals("Mountain")).findFirst().get().getId();target.addTarget(id,source,g);assertTrue(target.getTargets().contains(id));return true;}
 };}
 @Test public void executeCase() throws Exception {
  setStrictChooseMode(true);currentGame.setStartingPlayerId(playerA.getId());gameOptions.skipInitShuffling=true;
  removeAllCardsFromHand(playerA);removeAllCardsFromHand(playerB);removeAllCardsFromLibrary(playerA);removeAllCardsFromLibrary(playerB);
  addCard(Zone.HAND,playerA,"FDN-Thrill of Possibility",1);if(spec.get("discard").getAsBoolean())addCard(Zone.HAND,playerA,"FDN-Mountain",1);
  Map<String,String> names=new HashMap<>();names.put("forest","Forest");names.put("bear-cub","Bear Cub");names.put("giant-growth","Giant Growth");
  // XMage test setup puts each added card on top; neutral fixtures are top-to-bottom.
  JsonArray library=spec.getAsJsonArray("library");for(int i=library.size()-1;i>=0;i--)addCard(Zone.LIBRARY,playerA,"FDN-"+names.get(library.get(i).getAsString()),1);
  addCard(Zone.BATTLEFIELD,playerA,spec.get("mana").getAsString().equals("red")?"FDN-Mountain":"FDN-Forest",spec.get("amount").getAsInt());
  setStopAt(1,PhaseStep.PRECOMBAT_MAIN);execute();assertTrue("checkpoint not reached",captured);
  Files.write(Paths.get(System.getProperty("mtglab.output"),spec.get("id").getAsString()+".json"),new GsonBuilder().setPrettyPrinting().create().toJson(result).getBytes(StandardCharsets.UTF_8));
 }
}
