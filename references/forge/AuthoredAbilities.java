package forge.gamesimulationtests.mtglab;

import forge.game.Game;
import forge.game.ability.AbilityKey;
import forge.game.ability.AbilityUtils;
import forge.game.card.Card;
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
import java.util.*;
import static org.testng.Assert.*;

/** Rules-derived scheduler setup, never an original-AI game or quota entry.
 * CR 603.3b orders AP then NAP; CR 113.7a preserves a departed source.
 * The frozen Pyromancer text independently requires two damage per trigger.
 */
public final class AuthoredAbilities {
    private static final class Done extends RuntimeException { }

    @SuppressWarnings("unchecked")
    public static void verify() {
        for (int scenario : new int[] {0, 1, 2, 3}) {
            final List<Integer> placement = new ArrayList<>();
            final int active = scenario % 2;
            final boolean departBeforePlacement = scenario < 2;
            GameStateSpecificationBuilder setup = new GameStateSpecificationBuilder();
            for (PlayerSpecification seat : new PlayerSpecification[] {
                    PlayerSpecification.PLAYER_1, PlayerSpecification.PLAYER_2}) {
                setup.addCard(new CardSpecificationBuilder("Viashino Pyromancer").owner(seat).hand());
                setup.addCard(new CardSpecificationBuilder("Forest").owner(seat).library());
            }
            try (MockedConstruction<PlayerControllerForTests> controllers = Mockito.mockConstruction(
                    PlayerControllerForTests.class, context -> Mockito.withSettings().defaultAnswer(call -> {
                        Player player = (Player) context.arguments().get(1);
                        Game game = player.getGame();
                        int actor = game.getPlayers().indexOf(player);
                        switch (call.getMethod().getName()) {
                            case "getPlayer": return player;
                            case "getGame": return game;
                            case "getLobbyPlayer": return context.arguments().get(2);
                            case "isAI": return false;
                            case "isOrderedZone": return false; // inherited controller metadata
                            case "setPlayerActions": return null;
                            case "orderAndPlaySimultaneousSa": {
                                List<SpellAbility> choices = (List<SpellAbility>) call.getArgument(0);
                                assertEquals(choices.size(), 1, "one authored ETB per player");
                                SpellAbility ability = choices.get(0);
                                assertTrue(ability.isTrigger());
                                assertEquals(ability.getHostCard().getName(), "Viashino Pyromancer");
                                assertSame(ability.getActivatingPlayer(), player);
                                Player target = game.getPlayers().get(1 - actor);
                                assertTrue(ability.canTarget(target));
                                ability.getTargets().add(target);
                                placement.add(actor);
                                game.getStack().add(ability);
                                return null;
                            }
                            case "playSpellAbilityNoStack": {
                                SpellAbility ability = call.getArgument(0);
                                assertFalse((Boolean) call.getArgument(1));
                                AbilityUtils.resolve(ability);
                                return null;
                            }
                            case "chooseSpellAbilityToPlay": {
                                game.getPhaseHandler().setPlayerTurn(game.getPlayers().get(active));
                                game.getStack().freezeStack(null);
                                Card departed = null;
                                for (Player owner : game.getPlayers()) {
                                    Card source = owner.getCardsIn(ZoneType.Hand).get(0);
                                    Card entered = game.getAction().moveToPlay(source, owner, null, AbilityKey.newMap());
                                    if (owner == game.getPlayers().get(active)) departed = entered;
                                }
                                // Explicit synthetic scheduler setup. Finish recognizing both
                                // ETB events before changing zones, just as resolution's event
                                // boundary does; freezing alone defers that recognition in Forge.
                                assertNotNull(departed);
                                game.getTriggerHandler().resetActiveTriggers();
                                assertTrue(game.getTriggerHandler().runWaitingTriggers());
                                if (departBeforePlacement) game.getAction().moveToGraveyard(departed, null);
                                game.getStack().unfreezeStack();
                                assertTrue(game.getStack().addAllTriggeredAbilitiesToStack());
                                assertEquals(placement, Arrays.asList(active, 1 - active), "CR 603.3b APNAP");
                                assertEquals(game.getStack().size(), 2);
                                if (!departBeforePlacement) game.getAction().moveToGraveyard(departed, null);
                                assertSame(game.getStack().peekAbility().getActivatingPlayer(), game.getPlayers().get(1 - active));
                                game.getStack().resolveStack();
                                assertEquals(game.getPlayers().get(active).getLife(), 18);
                                assertEquals(game.getPlayers().get(1 - active).getLife(), 20);
                                assertEquals(game.getStack().size(), 1);
                                assertEquals(game.getPlayers().get(active).getCardsIn(ZoneType.Graveyard).size(), 1);
                                game.getStack().resolveStack();
                                assertEquals(game.getPlayers().get(1 - active).getLife(), 18,
                                             "CR 113.7a departed source still deals printed two damage");
                                assertTrue(game.getStack().isEmpty());
                                throw new Done();
                            }
                            default: throw new AssertionError("unscripted authored choice: " + call.getMethod().getName());
                        }
                    }))) {
                try {
                    new GameWrapper(setup.build(), null).runGame();
                    fail("missing authored endpoint");
                } catch (Done expected) {
                    assertEquals(placement.size(), 2);
                }
                assertEquals(controllers.constructed().size(), 2);
            }
        }
    }
}
