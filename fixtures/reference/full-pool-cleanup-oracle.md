# Played cleanup independent oracle (envelope 7)

CR 514.1: the active player discards to seven. Every selection identifies a
physical occurrence and incarnation, in explicit order. In the no-cast tapes,
keep the original seven and discard each drawn card, so the graveyard equals
the original shuffle suffix in order. Missing, extra, wrong actor, foreign card,
wrong incarnation, reordered and wrong-cardinality choices reject at their first
applicable boundary. `finish` is an observation acknowledgement, not a game action;
its actor field is the transcript owner and does not claim terminal priority.

CR 103.8a/121.4/704.5b: skip the entire first draw step. Each 40-card deck has
33 cards after opening seven. The nonstarter draws their last card on turn 66;
the starter draws their last card on turn 67. Both states are ongoing. On turn
68 the nonstarter attempts draw 34, loses, and the starter wins. Twenty life
persists. Every pass and discard is supplied; no safety limit or concession
supplies these results. CR 104.3a concession has its own separately labeled
normal-reset case and is excluded from natural-completion counts.

CR 514.2 and the pinned Oracle definitions: two Giant Growths give a Goblin
+6/+6; Goblin Surprise's boost mode gives both Goblins +2/+0. Bear Cub's Bite
Down deals two damage to the enhanced token. Just before cleanup it is 9/7
with two damage; its fellow token is 3/1. Cleanup leaves both 1/1, undamaged,
with no temporary effects. Checking lethal damage after reducing toughness but
before removing damage would incorrectly kill the first token. Native quantum
observations are from a snapshot-restored real engine executing the same accepted
turn action; semantic state and RNG must equal scalar completion. No state is
injected to create these normal-reset positions.

The pinned reference's PlayerImpl removes a losing player's objects when leaving
the game. Native retains them for replay. Retain raw post-loop final state AND
the reference LOST-event state before departure for empty-draw games. Compare
zone/identity ledgers at that witnessed boundary, and finalized winner/reason
separately. Concession calls setConcedingPlayer before LOST; retain its actual
pre-concession checkpoint separately. Never fabricate cleared objects in the
reference, suppress the observed storage difference, or claim a priority callback
returned after terminal settlement.

The native existing terminal API supplies reward accounting: winner +1, loser
-1, draw zero, once per seat. Expectations derive from the independently stated
outcome, not a saved reward baseline. External limits remain unfinished games.

Additional cleanup requires a waiting trigger or state-based action (CR 514.3a).
The normal-reset frozen-card cases above create neither at cleanup. Preserve and
execute the original explicitly synthetic cleanup-start pending-Pyromancer cases
(CR 514.3a), including a real Thrill cast during cleanup and repeated cleanup.
Their synthetic setup is not claimed reachable by these normal-reset tapes.
Preserve #257's separately labeled real synthetic simultaneous-loss suite; it is
not evidence of a reachable full game. Neither supplements nor replaces the
required played cases, original audit acceptance, or later corpus admission.
