# Creature casting

`mtg_core::opening::Game` casts Bear Cub (1G) and Swab Goblin (1R) after opening
and `start_turns`. Both are vanilla creatures with ordinary sorcery timing.
Other spells, creature abilities, combat and terminal outcomes remain separate
work. This is a core API, not a playable CLI or complete game engine.

1. At a current priority decision, `cast_candidates(actor)` lists payable supported
   hand objects. The actor must be active in a main phase with an empty stack.
   Available resources include the pool and controlled untapped basic lands.
2. `begin_cast(actor, decision_id, card_handle)` revalidates the gate and opens a
   payer-only `PaymentDecision`. It fixes the card's cost; callers cannot supply it.
3. `cast_mana_sources(actor)` and `cast_tap_mana(actor, payment_id, land)` stage
   explicit Forest/Mountain activations. Use `choose_payment` for each unit,
   exact color before generic. Each action returns a fresh decision ID. Land
   activation is allowed during payment, including for freshly played lands.
   Empty payment choices can mean mana is still needed: it is not a completion
   signal by itself. `finish_cast` checks that the full cost has been paid.
4. `finish_cast(actor, payment_id)` commits selected land taps, the resulting
   pool and exactly one spell on the stack with a new object identity. The
   caster retains priority. `finish_payment` rejects a pending cast.
5. Two consecutive `apply_turn` passes resolve only the top spell. It enters
   untapped under its spell controller with another new identity; the active
   player receives priority. The step and mana pools do not change on resolution.

The existing `cancel_payment` cancels a pending cast, preserving its card and
unstaged public state. Payment-time land activations and spending are provisional
until the logical cast commits; the opponent has neither a response window nor
access to provisional choices. Pre-cast mana abilities remain committed when a
cast is cancelled. Ordinary turn/land/payment-start commands reject while a cast
is pending. Invalid commands preserve the entire state, RNG and current decision.
A rejected final command leaves the valid continuation available to correct or
cancel. There are no automatic resource substitutions.

`summoning_sick(handle)` reports the entry restriction tracked for these creatures.
It clears at the beginning of the controller's next turn, not an opponent's turn.
Attack and creature tap-ability legality will consume this state in their own
registered tasks. Control-changing effects are not supported. Reset clears all
pending cast, stack and sickness state and invalidates prior decisions/handles.

Stack order is explicit and resolves LIFO. These creatures cannot legally respond
to one another. Tests use declared synthetic multi-spell stacks for LIFO, plus
real mana-ability responses (which reset consecutive passes without using the
stack). Instant response chains belong to GH-70/GH-18. Arbitrary raw `ObjectStore`
stack placements remain unsupported unless registered by the rules layer.

All object inspection and `Game` Debug remain privileged. Seat arguments rely on
caller authorization; complete policy views are GH-73. No tensor/action-capacity
or performance claim is made. Payment choices have at most six colors; land
sources and creature candidates are dynamic vectors, never silently truncated.

Run `cargo test -p mtg-core casting` and `./scripts/torture.sh` inside the managed
container. [Acceptance evidence](evidence/casting/README.md) distinguishes normal
reset sequences, synthetic cases, red/green results and reference limitations.
