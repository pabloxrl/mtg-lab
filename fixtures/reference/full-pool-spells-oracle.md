# Played spell prefixes: independent acceptance basis

Inputs in `author_spells.py` are authored from the frozen deck and pinned card
manifest before execution. No engine output is used to choose actions or results.
Both players keep; chance supplies a complete physical permutation of each deck.
Seat 0 starts; CR 103.8a skips its first draw step. Every pass and empty attack
choice is explicit. Tests stop in main before cleanup can expire an effect.

- Dragon Fodder costs {1}{R}; resolution creates two **distinct** 1/1 red Goblin
  creature tokens (CR 111, 608). Their canonical IDs include controller, creation
  event ordinal and within-event ordinal. The IDs require witnessed engine births.
- Thrill costs {1}{R} plus a discard. Select physical Mountain copy 2, preserving
  the other copies. The discard is committed before resolution, then precisely
  the next two library cards are drawn in order (CR 601.2h, 608.2).
- Goblin Surprise costs {2}{R}. Mode 0 grants creatures controlled at resolution
  +2/+0 until end of turn; the two existing Goblins therefore become 3/1.
  Mode 1 creates two new 1/1 red Goblins. A second creation event must not reuse
  either original token ID. The pinned XMage GoblinSurprise.java independently
  confirms both modes, matching the unchanged Oracle text hash.
- Bite Down costs {1}{G}, targets an own creature then an opposing creature.
  Giant Growth costs {G}, targets a creature and gives +3/+3 until end of turn.
  In the response chain Bite targets Cub and the second Goblin; responding
  Growth targets the Cub. Growth resolves first: Cub becomes **5/5**, Bite
  remains on the stack with its original ordered targets. Bite then deals five
  damage, the selected Goblin dies and ceases to exist (CR 608.2b, 704.5g/d).
  The other Goblin remains 1/1; Cub remains 5/5. Departed references cannot bind
  to another token or incarnation (CR 400.7).

Additional acceptance requires exact intended-boundary rejection, private native
transaction nonmutation, explicit callback provenance and effect durations.
Inputs/oracles alone are not executed acceptance or an M2 completion claim.

The opposite response order is also authored: Growth targets Goblin ordinal 1,
then Bite targets Cub and that Goblin. Bite resolves first; Growth keeps the
original target reference, finds it departed and does not resolve its boost.
Cub remains 2/2; Goblin ordinal 0 remains 1/1.

Native token births export the actual engine CardId key and immutable content
hash, binding the observed object to the frozen red Goblin definition. XMage
exports the observed token color bits/subtype and raw CREATED_TOKEN identity.
No native dynamic color field is fabricated. Native effects are witnessed in
`turns.modifications`, whose lifetime is the cleanup expiration queue in
`turns.rs`; XMage reports the actual continuous effect's `EndOfTurn` duration
and source. The family verifies current effects and source/recipient identities;
cleanup expiry remains the separate owner's acceptance.

Negative tapes retain only the reachable prefix through the illegal action.
Their explicit requirement, action sequence and diagnostic category are checked
in native discovery, actual XMage execution and the Python verifier. The
self-discard reference control uses the announced spell's current stack
incarnation, so it tests the discard rule rather than stale-reference rejection.
The original weaker control and its asserted failure are retained in evidence.
