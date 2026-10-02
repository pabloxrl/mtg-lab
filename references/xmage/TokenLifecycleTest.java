package org.mage.test.mtglab;

import com.google.gson.*;
import mage.constants.*;
import mage.abilities.Ability;
import mage.game.Game;
import mage.game.permanent.Permanent;
import mage.players.Player;
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

/** Original pinned-Oracle/CR 111, 302.6, 307, 601, 704 scenarios.
 * Synthetic setup; scheduled casts and combat use XMage's real rules paths. */
@RunWith(Parameterized.class)
public class TokenLifecycleTest extends CardTestPlayerBase {
    private final String mode;
    private boolean sawPending, sawCreated, sawTimingRejection, sawGrowthDamage;
    private final Set<UUID> births=new HashSet<>();
    @Parameterized.Parameters(name="{0}") public static Collection<Object[]> cases() {
        return Arrays.asList(new Object[][]{{"repeat"},{"lethal"},{"stale-growth"},{"combat"},{"growth-block"}});
    }
    public TokenLifecycleTest(String mode){this.mode=mode;}
    private List<Permanent> tokens(Game game){
        List<Permanent> out=new ArrayList<>();
        for(Permanent p:game.getBattlefield().getAllActivePermanents())if(p.isToken())out.add(p);
        return out;
    }
    @Override protected TestPlayer createPlayer(String name,RangeOfInfluence range){
        return new TestPlayer(new TestComputerPlayer(name,range)){
            @Override public void selectAttackers(Game game,UUID active){
                assertEquals(getId(),active);
                if(mode.equals("combat") && game.getTurnNum()==3) {
                    assertEquals(playerA.getId(),getId());
                    for(String alias:Arrays.asList("@token1","@token2"))declareAttacker(getAliasByName(alias),playerB.getId(),game,false);
                } else if(mode.equals("growth-block") && game.getTurnNum()==2) {
                    assertEquals(playerB.getId(),getId());
                    Permanent cub=game.getBattlefield().getAllActivePermanents().stream().filter(p->p.getName().equals("Bear Cub")).findFirst().get();
                    declareAttacker(cub.getId(),playerA.getId(),game,false);
                }
            }
            @Override public void selectBlockers(Ability source,Game game,UUID defending){
                assertEquals(getId(),defending);
                if(mode.equals("combat") && game.getTurnNum()==3) {
                    Permanent cub=game.getBattlefield().getAllActivePermanents().stream().filter(p->p.getName().equals("Bear Cub")).findFirst().get();
                    declareBlocker(getId(),cub.getId(),getAliasByName("@token1"),game);
                } else if(mode.equals("growth-block") && game.getTurnNum()==2) {
                    Permanent cub=game.getBattlefield().getAllActivePermanents().stream().filter(p->p.getName().equals("Bear Cub")).findFirst().get();
                    declareBlocker(getId(),getAliasByName("@token1"),cub.getId(),game);
                }
            }
            @Override public boolean priority(Game game){
                List<Permanent> ts=tokens(game);
                if(mode.equals("growth-block") && game.getTurnNum()==2 && game.getTurnStepType()==PhaseStep.COMBAT_DAMAGE){
                    Permanent blocker=game.getPermanent(playerA.getAliasByName("@token1"));
                    assertNotNull(blocker);assertEquals(4,blocker.getPower().getValue());assertEquals(4,blocker.getToughness().getValue());assertEquals(2,blocker.getDamage());
                    assertFalse(game.getBattlefield().getAllActivePermanents().stream().anyMatch(p->p.getName().equals("Bear Cub")));
                    sawGrowthDamage=true;
                }
                if(getId().equals(playerB.getId()) && game.getTurnNum()==1 && game.getTurnStepType()==PhaseStep.BEGIN_COMBAT){
                    for(UUID id:getHand())if(game.getCard(id).getName().equals("Dragon Fodder")){
                        assertFalse(game.getCard(id).getSpellAbility().canActivate(getId(),game).canActivate());sawTimingRejection=true;
                    }
                }
                if(mode.equals("repeat") && !game.getStack().isEmpty()) {
                    for(UUID id:playerA.getHand())if(game.getCard(id).getName().equals("Dragon Fodder"))
                        assertFalse(game.getCard(id).getSpellAbility().canActivate(playerA.getId(),game).canActivate());
                }
                if(!game.getStack().isEmpty() && game.getStack().peek().getName().equals("Dragon Fodder") && births.isEmpty()){
                    assertEquals(0,ts.size());sawPending=true;
                }
                for(Permanent p:ts){
                    if(births.add(p.getId())) { String alias="token"+births.size();playerA.addAlias(alias,p.getId());playerB.addAlias(alias,p.getId()); }assertEquals(playerA.getId(),p.getOwnerId());assertEquals(playerA.getId(),p.getControllerId());
                    assertTrue(p.getColor(game).isRed());assertTrue(p.hasSubtype(SubType.GOBLIN,game));
                    if(game.getTurnNum()==1){assertFalse(p.isTapped());assertFalse(p.canAttack(playerB.getId(),game));}
                }
                if(ts.size()==2 && game.getTurnNum()==1 && game.getStack().isEmpty())sawCreated=true;
                return super.priority(game);
            }
        };
    }
    @Test public void lifecycle() throws Exception {
        setStrictChooseMode(true);currentGame.setStartingPlayerId(playerA.getId());gameOptions.skipInitShuffling=true;
        removeAllCardsFromHand(playerA);removeAllCardsFromHand(playerB);
        removeAllCardsFromLibrary(playerA);removeAllCardsFromLibrary(playerB);
        addCard(Zone.LIBRARY,playerA,"FDN-Mountain",8);addCard(Zone.LIBRARY,playerB,"FDN-Forest",8);
        addCard(Zone.BATTLEFIELD,playerA,"FDN-Mountain",4);addCard(Zone.BATTLEFIELD,playerA,"FDN-Forest",2);
        addCard(Zone.BATTLEFIELD,playerB,"FDN-Forest",2);
        addCard(Zone.HAND,playerA,"FDN-Dragon Fodder",mode.equals("repeat")?2:1);
        if(!mode.equals("repeat"))addCard(Zone.BATTLEFIELD,playerB,"FDN-Bear Cub",1);
        addCard(Zone.HAND,playerB,"FDN-Dragon Fodder",1);
        castSpell(1,PhaseStep.PRECOMBAT_MAIN,playerA,"Dragon Fodder");
        if(mode.equals("repeat"))castSpell(1,PhaseStep.POSTCOMBAT_MAIN,playerA,"Dragon Fodder");
        if(mode.equals("lethal")||mode.equals("stale-growth")){
            addCard(Zone.HAND,playerB,"FDN-Bite Down",1);
            if(mode.equals("stale-growth")){
                addCard(Zone.HAND,playerA,"FDN-Giant Growth",1);
                castSpell(1,PhaseStep.POSTCOMBAT_MAIN,playerA,"Giant Growth","@token1");
                castSpell(1,PhaseStep.POSTCOMBAT_MAIN,playerB,"Bite Down","Bear Cub^@token1","Giant Growth");
            }else castSpell(1,PhaseStep.POSTCOMBAT_MAIN,playerB,"Bite Down","Bear Cub^@token1");
        }
        if(mode.equals("growth-block")){
            addCard(Zone.HAND,playerA,"FDN-Giant Growth",1);
            castSpell(2,PhaseStep.DECLARE_BLOCKERS,playerA,"Giant Growth","@token1");
        }
        setStopAt(mode.equals("combat")?3:mode.equals("growth-block")?3:1,
            mode.equals("growth-block")?PhaseStep.UPKEEP:PhaseStep.END_TURN);
        execute(); assertTrue("must observe spell pending",sawPending);assertTrue("must observe two tokens",sawCreated);assertTrue("off-turn combat sorcery rejected",sawTimingRejection);
        if(mode.equals("growth-block"))assertTrue("must observe boosted blocker damage and dead Cub",sawGrowthDamage);
        int count=mode.equals("repeat")?4:mode.equals("growth-block")?2:1;
        List<Permanent> ts=tokens(currentGame);assertEquals(count,ts.size());
        assertEquals(mode.equals("repeat")?4:2,births.size());
        assertLife(playerA,20);assertLife(playerB,mode.equals("combat")?19:20);
        for(Permanent p:ts){assertEquals(1,p.getPower().getValue());assertEquals(1,p.getToughness().getValue());assertEquals(0,p.getDamage());}
        JsonObject result=new JsonObject();result.addProperty("tokens",ts.size());result.addProperty("unique_created",births.size());
        JsonArray life=new JsonArray();life.add(playerA.getLife());life.add(playerB.getLife());result.add("life",life);
        result.addProperty("pending_zero",sawPending);result.addProperty("one_one",true);
        int graveTokens=0;for(Player p:Arrays.asList(playerA,playerB))for(UUID id:p.getGraveyard())if(currentGame.getCard(id).getName().equals("Goblin Token"))graveTokens++;
        assertEquals(0,graveTokens);result.addProperty("grave_tokens",graveTokens);
        Files.write(Paths.get(System.getProperty("mtglab.output"),mode+".json"),new GsonBuilder().setPrettyPrinting().create().toJson(result).getBytes(StandardCharsets.UTF_8));
    }
}
