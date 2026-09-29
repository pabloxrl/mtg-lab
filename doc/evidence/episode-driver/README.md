# Scalar episode ownership acceptance (GH-159)

Scope: partial R0002-B036/B037, one driver around delivered Game/policy/actions.
No rules changes, full collector or milestone completion claim. The executable
regressions live in [`crates/mtg-core/tests/episode.rs`](../../../crates/mtg-core/tests/episode.rs)
and normal Cargo discovery. Run `cargo test -p mtg-core --test episode`, plus the
mandatory `./scripts/torture.sh`. [API and limitations](../../episode-driver.md).

## Independent expectations

Original hand-authored normal-reset script, master 159/ordinal 0, complete frozen
40-card green mirror decks with exact valid library orders. This is played-game
acceptance, not synthetic state or a substitute sibling. CR 103 supplies seven
opening cards and 33 library cards; CR 305 permits the scripted land each turn;
CR 601/608 and frozen Bear Cub text give a paid 2/2 creature. Two Forests are
tapped after its turn-three payment; the creature is summoning sick and undamaged.
CR 508–510/704 give ten unblocked attacks on odd turns 5 through 23: literal life
checkpoints `[20, 18]` through `[20, 0]`, then P0 wins with P1 loss reason Life.
There are exactly 23 land plays and ten attacker declarations. Both seats keep;
every intervening priority pass and empty combat declaration is explicit.

Semantic object references are independently authored from the fixed order:
P0 Cub birth 0, Forests 1–12; P1 Forests 41–51; hand incarnation 1, land battlefield
incarnation 2, resolved Cub incarnation 3. Each operation's captured bytes are
parsed and compared to the literal semantic record. The existing action decoder
only resolves those references for submitting the same choices through the driver.
Every accepted operation adds exactly one record; every boundary also compares
full normalized snapshots against direct existing-action execution. A fresh Game
reapplies the completed captured sequence and matches the final full state/RNG.
Only process-local object store/scope IDs are normalized, not seeds, generations,
ordered zones, continuations, life or RNG. Equality is supplemental to the literal
rules checkpoints and semantic identities, not the expected-outcome oracle.

Wrong actor, revision, generation, schema and empty illegal selections are injected
before each valid action, including payment and combat continuations. Every attempt
must preserve exact snapshot bytes and history, then the legal suffix must finish.
Terminal advance, submission and concession also preserve both. Capacity failures
and invalid resets are nonmutating. Seat observations expose only own hand and
actor choices, with no privileged input/history fields.

CR 104.3a independently justifies nonacting P1 concession before any decision,
with one literal semantic concession record and P0 winner. Finished results retain
actual inputs and bytes through a different starting seat/seed/ordinal reset;
stale submission and concession tokens fail. Unfinished reset is explicitly
Incomplete, can be finalized and replaced, and cannot fabricate a terminal result.
Seed 917/ordinal 3 exercises actual randomized reset against the existing core.
An external compile-fail doctest additionally checks that Game access is private;
this is an ownership regression, not behavioral-red evidence.

## Test-first receipt

Before driver implementation, the two main behavioral tests compiled and both
failed at reset: `left: Err(NotStarted)`, `right: Ok(InternalYield)` (0 passed,
2 failed). The initial public-contract scaffold returned NotStarted for mutation;
no compile/import error is counted. This established the missing reset/ownership
behavior before implementing it. Preserved compiler/test output: [red.log](red.log).

The first implementation run found that core OpeningComplete also occurs during
payment; driver progress now distinguishes internal work from all ready policy
choices. A subsequent script diagnostic found the explicit combat-damage finish
choice missing from its idle helper; it was added without changing any literal
checkpoint or weakening an assertion. All four behavior tests then passed.
Existing tests and expectations are unchanged. No new reference-engine claim is
made: this wraps established rules and introduces no new game mechanics.

Full-suite, exact-candidate separate review, protected PR and exact-main CI receipts
are recorded in the [issue workpad](https://github.com/pabloxrl/mtg-lab/issues/159#issuecomment-5893322183)
and delivery PR. Those gates must pass before delivery is complete.

## Independent review repair

The first prescribed review of `4ec29c933b509308f87fbc26527cc8c3a6fbbe03`
found one blocking replay-integrity defect: concession during an unfinished reset
could return Completed with a partial deck/RNG state that normal reset plus history
cannot reconstruct. [Original review](review-round1.json). The added normal-discovery
`concession_during_yielding_reset_is_rejected_then_settled_history_replays` test first
compiled and failed with `Ok(())` versus `Err(Concede(SettlementPending))`;
[preserved behavioral failure](concession-red.log). The driver now rejects
concession during internal work without state/history changes. Both seats are
checked; advancing to the opening boundary then conceding remains legal and matches
a fresh normal-reset action replay including full RNG. Core rules are unchanged.
The complete suite and separate candidate review are repeated after this repair.
