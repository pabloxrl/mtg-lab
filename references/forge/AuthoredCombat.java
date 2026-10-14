package forge.gamesimulationtests.mtglab;

import forge.game.Game;
import forge.game.ability.AbilityKey;
import forge.game.ability.AbilityUtils;
import forge.game.ability.ApiType;
import forge.game.card.Card;
import forge.game.combat.CombatUtil;
import forge.game.cost.CostPart;
import forge.game.cost.CostTap;
import forge.game.cost.PaymentDecision;
import forge.game.player.Player;
import forge.game.spellability.SpellAbility;
import forge.game.zone.ZoneType;
import forge.gamesimulationtests.util.GameWrapper;
import forge.gamesimulationtests.util.PlayerControllerForTests;
import forge.gamesimulationtests.util.card.CardSpecificationBuilder;
import forge.gamesimulationtests.util.gamestate.GameStateSpecificationBuilder;
import forge.gamesimulationtests.util.player.PlayerSpecification;
import org.mockito.MockedConstruction;
import org.mockito.Mockito;
import static org.testng.Assert.*;

/** Synthetic rare legal-declaration checks, not AI games. The unchanged card
 * text and CR 302.6/702.9/702.10/702.17 provide expectations independently. */
public final class AuthoredCombat {
    private static final class Done extends RuntimeException { }
    private static Card named(Player p,String name) {
        for (Card c:p.getCardsIn(ZoneType.Battlefield)) if(c.getName().equals(name)) return c;
        throw new AssertionError("missing authored card "+name);
    }
    public static void verify() {
        GameStateSpecificationBuilder setup=new GameStateSpecificationBuilder();
        for(String name:new String[]{"Axgard Cavalry","Shivan Dragon"})
            setup.addCard(new CardSpecificationBuilder(name).owner(PlayerSpecification.PLAYER_1).battlefield());
        setup.addCard(new CardSpecificationBuilder("Tajuru Pathwarden").owner(PlayerSpecification.PLAYER_1).hand());
        for(String name:new String[]{"Bear Cub","Thornweald Archer","Magnigoth Sentry"})
            setup.addCard(new CardSpecificationBuilder(name).owner(PlayerSpecification.PLAYER_2).battlefield());
        for(PlayerSpecification seat:new PlayerSpecification[]{PlayerSpecification.PLAYER_1,PlayerSpecification.PLAYER_2})
            setup.addCard(new CardSpecificationBuilder("Forest").owner(seat).library());
        try(MockedConstruction<PlayerControllerForTests> controllers=Mockito.mockConstruction(
                PlayerControllerForTests.class,context->Mockito.withSettings().defaultAnswer(call->{
                    Player player=(Player)context.arguments().get(1);
                    Game game=player.getGame();
                    switch(call.getMethod().getName()) {
                        case "getPlayer": return player;
                        case "getGame": return game;
                        case "getLobbyPlayer": return context.arguments().get(2);
                        case "isAI": case "isOrderedZone": return false;
                        case "setPlayerActions": return null;
                        case "playSpellAbilityNoStack":
                            AbilityUtils.resolve(call.getArgument(0)); return null;
                        case "chooseSpellAbilityToPlay": {
                            Player first=game.getPlayers().get(0), second=game.getPlayers().get(1);
                            Card dragon=named(first,"Shivan Dragon");
                            assertFalse(CombatUtil.canBlock(dragon,named(second,"Bear Cub")),"flying excludes nonreach ground blocker");
                            assertTrue(CombatUtil.canBlock(dragon,named(second,"Thornweald Archer")),"printed reach");
                            assertTrue(CombatUtil.canBlock(dragon,named(second,"Magnigoth Sentry")),"printed reach");
                            // Explicit synthetic entry; the engine creates sickness. Haste
                            // itself is obtained only by resolving the printed activation.
                            Card ward=game.getAction().moveToPlay(first.getCardsIn(ZoneType.Hand).get(0),first,null,AbilityKey.newMap());
                            assertFalse(CombatUtil.canAttack(ward,second),"new Pathwarden has no haste");
                            Card cavalry=named(first,"Axgard Cavalry");
                            SpellAbility haste=null;
                            for(SpellAbility sa:cavalry.getSpellAbilities()) if(sa.getApi()==ApiType.Pump) haste=sa;
                            assertNotNull(haste);
                            haste.setActivatingPlayer(first);
                            assertTrue(haste.canTarget(ward)); haste.getTargets().add(ward);
                            assertEquals(haste.getPayCosts().getCostParts().size(),1);
                            CostPart cost=haste.getPayCosts().getCostParts().get(0);
                            assertTrue(cost instanceof CostTap);
                            assertTrue(cost.canPay(haste,first,false));
                            assertTrue(cost.payAsDecided(first,PaymentDecision.number(0),haste,false));
                            assertTrue(cavalry.isTapped());
                            game.getStack().add(haste); game.getStack().resolveStack();
                            assertTrue(CombatUtil.canAttack(ward,second),"resolved printed haste permits same-turn attack");
                            throw new Done();
                        }
                        default: throw new AssertionError("unscripted combat choice: "+call.getMethod().getName());
                    }
                }))) {
            try { new GameWrapper(setup.build(),null).runGame(); fail("missing combat endpoint"); }
            catch(Done expected) { assertEquals(controllers.constructed().size(),2); }
        }
    }
}
