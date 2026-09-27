# Lands and mana payments

`mtg_core::opening::Game` supports Forest/Mountain land plays and mana abilities
through `opening::mana`, after opening choices and `start_turns`.

- `land_candidates(actor)` lists legal basic lands from the priority actor's hand.
  `play_land(actor, decision_id, handle)` rechecks the same gate: active player,
  main phase, empty stack and unused land play this turn. It changes zone/identity,
  uses the land allowance and retains priority. A forged second-land command fails.
- `mana_sources(actor)` lists controlled untapped Forests/Mountains during priority.
  `tap_mana` taps the source and immediately adds G/R, without using the stack.
  It retains priority and resets consecutive passes. Lands can tap the turn they
  enter, including on an opponent's turn when their controller has priority.
- `mana()` returns public committed pools, indexed W/U/B/R/G/C. Both pools clear
  at step/phase boundaries without life loss. The ordinary turn untap still applies.

Rules code calls `begin_payment(actor, decision_id, ManaCost)` with a validated
cost and already floated mana. The exact-symbol array includes colorless C;
`generic` accepts any color or C. This is a rules integration primitive, not an
interface allowing a policy to set a spell's price. Spell casting is GH-69.

`payment_decision(actor)` returns only that payer's current ID and choices.
`choose_payment(actor, id, color)` consumes one provisional mana unit and returns
a fresh scoped decision. Exact symbols are selected in W/U/B/R/G/C order before
generic mana. All distinct legal remaining pools are expressible; interchangeable
units and equivalent orderings need not be separate strategic actions. Candidate
count is at most six, independent of pool size. Amounts use checked u32 storage
and affordability sums use u64; overflow fails explicitly. No tensor schema or
whole-engine capacity claim is made here.

During payment `turn_decision()` returns None, turn/land/tap commands reject, and
both seats' ordinary pools and objects remain unchanged. The opponent receives
no payment choices. When choices are empty, `finish_payment` commits once and
retains the payer's priority. The casting layer must join this preflighted commit
with its spell commit before returning any decision to a policy. `cancel_payment`
discards provisional spending and invalidates prior decisions. Mana abilities
activated before payment remain committed; cancellation does not untap them.

This prefix does not yet activate mana sources *inside* a casting continuation;
callers can float land mana first. Casting integration owns that lifecycle and
must preserve the same atomicity. Creature mana abilities, spell costs, targets,
stack resolution, complete player observations, snapshots and replay remain
registered sibling/integration work. Object inspection and `Game` Debug are
privileged; never use them as policy observations. Seat arguments rely on the
caller's seat authorization, as with opening/turn APIs.

Run `cargo test -p mtg-core mana` in the managed container. The normal full suite
also discovers these tests. [Acceptance and independent expectations](evidence/mana/README.md)
cover normal reset-to-play sequences and explicitly synthetic pool/storage cases.
