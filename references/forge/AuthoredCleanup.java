package forge.gamesimulationtests.mtglab;

import forge.game.Game;
import forge.game.MtgCastingReplay;
import forge.game.card.Card;
import forge.game.player.Player;
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

/** Explicit synthetic setup, never AI provenance. CR 514.1 requires exactly
 * nine minus seven distinct hand cards, then CR 514.2 completes cleanup. */
public final class AuthoredCleanup {
    private static final class Done extends RuntimeException { }

    public static void verify() {
        GameStateSpecificationBuilder setup = new GameStateSpecificationBuilder();
        for (int i=0; i<9; i++)
            setup.addCard(new CardSpecificationBuilder("Forest").owner(PlayerSpecification.PLAYER_1).hand());
        for (PlayerSpecification seat : new PlayerSpecification[] {
                PlayerSpecification.PLAYER_1, PlayerSpecification.PLAYER_2})
            for (int i=0; i<2; i++)
                setup.addCard(new CardSpecificationBuilder("Mountain").owner(seat).library());
        final List<Card> discarded = new ArrayList<>();
        try (MockedConstruction<PlayerControllerForTests> controllers = Mockito.mockConstruction(
                PlayerControllerForTests.class, context -> Mockito.withSettings().defaultAnswer(call -> {
                    Player player = (Player) context.arguments().get(1);
                    Game game = player.getGame();
                    switch (call.getMethod().getName()) {
                        case "getPlayer": return player;
                        case "getGame": return game;
                        case "getLobbyPlayer": return context.arguments().get(2);
                        case "isAI": case "isOrderedZone": return false;
                        case "setPlayerActions":
                        case "resetAtEndOfTurn": case "autoPassCancel": return null;
                        case "chooseSpellAbilityToPlay":
                            if (!discarded.isEmpty()) {
                                Player first = game.getPlayers().get(0);
                                assertSame(game.getPhaseHandler().getPlayerTurn(), game.getPlayers().get(1));
                                assertEquals(first.getCardsIn(ZoneType.Hand).size(), 7);
                                assertEquals(first.getCardsIn(ZoneType.Graveyard).size(), 2);
                                for (Card card : discarded)
                                    assertTrue(first.getCardsIn(ZoneType.Graveyard).contains(card));
                                throw new Done();
                            }
                            return null;
                        case "chooseCardsToDiscardToMaximumHandSize": {
                            assertSame(player, game.getPlayers().get(0));
                            int count = call.getArgument(0);
                            assertEquals(count, 2);
                            assertTrue(discarded.isEmpty());
                            Card one=player.getCardsIn(ZoneType.Hand).get(0);
                            Card two=player.getCardsIn(ZoneType.Hand).get(1);
                            expectThrows(IllegalArgumentException.class,
                                () -> MtgCastingReplay.cleanupDiscards(player,count,Arrays.asList(one,one)));
                            expectThrows(IllegalArgumentException.class,
                                () -> MtgCastingReplay.cleanupDiscards(player,count,Arrays.asList(one)));
                            expectThrows(IllegalArgumentException.class,
                                () -> MtgCastingReplay.cleanupDiscards(player,count,
                                    Arrays.asList(one,player.getCardsIn(ZoneType.Library).get(0))));
                            discarded.add(one); discarded.add(two);
                            return MtgCastingReplay.cleanupDiscards(player,count,discarded);
                        }
                        default: throw new AssertionError("unscripted cleanup choice: " + call.getMethod().getName());
                    }
                }))) {
            try { new GameWrapper(setup.build(), null).runGame(); fail("missing cleanup endpoint"); }
            catch (Done expected) { assertEquals(discarded.size(), 2); }
            assertEquals(controllers.constructed().size(), 2);
        }
    }
}
