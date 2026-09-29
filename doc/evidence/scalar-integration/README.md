# GH-18 scalar integration acceptance

This is the bounded integration audit after #109 and #115, on baseline
`26211632c2535c989f8e716bfedcc20ac9080137`. It accounts for the original land,
creature, Growth and Bite scope, all eight directly owned catalog cases and
applicable scalar system contracts. It does not declare M1 complete: #19–#22
retain their integration/gate obligations. Delivery is conditional on independent
candidate review, protected merge and successful exact-main CI recorded in the
[issue workpad](https://github.com/pabloxrl/mtg-lab/issues/18).

No production rule changes or fixture expectations are needed. Two additional
normal-discovery tests explicitly attempt Growth at every one-unit combat-damage
and untap yield, for both seats, checking exact snapshot nonmutation. They also
check the legal declaration response windows and literal final 2/2 trade/untap
results. Their first compiled execution passed against the delivered engine;
a private-import compile correction was test authoring, not behavioral red.
Original component red/green evidence remains linked below. No tests were removed,
replaced, skipped or weakened.

## Original acceptance

| Obligation | Executable evidence and independent expectation |
| --- | --- |
| Priority, passing, stack, SBA, responses | `casting_literal_payment_stack_resolution_and_priority`, `targets_response_chain_priority_current_power_and_lethal`, `targets_hold_priority_bite_above_growth_and_five_no_redirection`; CR 117/601/608/704 require caster priority, active-player priority after each resolution, LIFO and SBA before the next decision. [Casting](../casting/README.md), [targets](../targets/README.md), and [matched references](../matched-growth-bite/README.md). |
| Lands, mana, payment | `mana_forest_mountain_positive_negative_regression`, `mana_masked_second_land_and_timing_rejected_transactionally`, `mana_payment_private_atomic_rejection_and_duplicate_commit`, `casting_generic_choices_and_atomic_rejected_final_command`; CR 305/605/106/601 give one own-main land, immediate land mana, colored/generic choices and exact-once committed payment. [Mana evidence](../mana/README.md). |
| Casting and vanilla combat | Normal-reset `combat_normal_reset_cast_sickness_attack_cleanup_and_next_untap_both_seats`; synthetic `combat_declarations_and_simultaneous_trade_cub_and_swab`, independent subset/map enumeration and all 0+2/1+1/2+0 allocations. Pinned Cub/Goblin are 2/2s; CR 302.6/508–510 prohibit sick attacks and require simultaneous damage without old blocker ordering. [Combat evidence](../combat/README.md). |
| Targets, revalidation, temporary effects | All role pairs independently enumerated; wrong controller, missing/departed targets, partial/all-invalid and source-power-at-resolution tested. `targets_cleanup_damage_and_expiration_simultaneous` and bounded cleanup tests require a 5/5 with two/four damage to survive as an undamaged 2/2. CR 608.2b/613.4c/514.2; no lethal check between boost expiry and damage removal. Eleven live matched scenarios corroborate these checkpoints. |
| Private atomic choices | `policy_spells_twins_cancel_each_stage_and_reject_without_mutation`, `policy_combat_every_boundary_hidden_twins_masks_and_atomic_errors`, `policy_combat_twins_private_backtracking_errors_and_capacity`; opponent views omit proposed choices, no priority inside targeting/payment/declarations, explicit cancellation and stale/wrong-seat rejection. Full privacy audit remains #19. |
| Bounded automatic work | Spell, combat and turn quantum tests at one/boundary/large budgets retain literal states, RNG, identity, exact-once generation, pending rejection, snapshots and no provisional player view. [Spell](../spell-settlement/README.md), [combat](../combat-settlement/README.md), [turn](../turn-settlement/README.md) receipts retain original compiled red. Added `integration_growth_*` tests directly exercise illegal casting during internal yields. |
| Scripted bounded complete games | `played_replay_normal_reset_independent_checkpoints` starts with legal ordered frozen decks, keeps, plays lands, casts Cubs, responds to Bite with Growth, crosses cleanup and completes ten unblocked attacks: P1 life 18,16,…,0. No injected state or concession. `played_replay_fresh_process_and_quantum_suffixes` repeats full semantic state after every choice at budgets 1/3/17. [Independent script and checkpoints](../played-replay/README.md). `terminal_full_opening_turns_combat_best_of_one_and_explicit_reset` additionally runs both starting seats in red/green mirrors. |
| Supported slice only | Only Forest/Mountain, Bear Cub/Swab Goblin, Giant Growth/Bite Down and vanilla combat are advertised. Frozen decks can contain later cards, but those are uncastable; full-pool capability requests fail. `casting_fresh_forest_mountain_negative_and_unsupported_cards`, opening config validation and policy unsupported-content tests retain explicit rejection. No fixture-ID or deck-sequence dispatch added. |
| Rules-derived positive/negative/interaction/regression fixtures and references | Unchanged pinned CR/Oracle expectations in normal discovery plus fresh real XMage execution below. The generic production rules receive typed actions, not fixture IDs. Synthetic focused cases and normal-reset reachability are separately identified. Agreement is corroboration, not the source of expected output. |

## Eight owned catalog cases

The unchanged [catalog](../../testing/capability-test-plan.json) owns these exact
IDs. Each row maps its original setup/action/expectation, not merely a child issue.

| Case ID | Test / expected checkpoint |
| --- | --- |
| `rules-priority-response-window-regression` | `integration_growth_windows_end_before_atomic_combat_damage`: active priority and Growth candidate after each declaration; no player decision/cast during damage. Existing combat declaration tests also assert provisional selections remain uncommitted. |
| `rules-combat-simultaneous-damage-negative` | Same added test attempts targeted Growth during **every** quantum-one damage/SBA yield, both actors, and requires exact snapshot equality after rejection; both 2/2s die with life 20/20. |
| `rules-combat-blocked-status-negative` | Shared `bite-killed-blocker` script really kills the blocker with Bite, inspects remembered blocked status and requires zero player damage. `combat_quantum_removed_blockers_remember_blocked_status` covers both seats and every bounded phase; player damage is not an available allocation for a blocked vanilla attacker. |
| `rules-priority-turn-structure-negative` | `integration_growth_rejected_through_every_untap_yield`: Growth attempts reject unchanged at each internal untap pause, no player view or cast candidates, next exposed boundary is upkeep with untapped nonsick Cub. |
| `rules-priority-turn-structure-regression` | `turn_quantum_priority_in_cleanup_repeats_discard_before_next_turn`: synthetic cleanup-priority boundary repeats cleanup, including extra discard and fresh boost expiry, before next upkeep. Positive M1 boosts cannot themselves create CR 514.3a trigger/SBA priority; the declared synthetic premise tests the required transition without claiming an implemented trigger source. |
| `rules-foundations_micro_v1-bear-cub-positive` | Normal-reset combat test casts a 1G Cub, observes sickness, advances to next own turn, attacks unblocked and requires exactly two player damage, both seats. |
| `rules-foundations_micro_v1-bear-cub-interaction` | `combat_declarations_and_simultaneous_trade_cub_and_swab`, the new Growth-window test and live vanilla reference require Cub/Swab simultaneous deaths and unchanged life. |
| `rules-foundations_micro_v1-bear-cub-regression` | Normal-reset played replay, `targets_cleanup_damage_and_expiration_simultaneous`, bounded turn cleanup and four live cleanup cases require surviving undamaged 2/2 after Growth/Bite. |

## System contracts and retained later scope

- **SYS-CORE-003:** state-preserving stale/wrong-actor/duplicate/missing/illegal
  choices at target, payment, combat and cleanup stages, including invalid final
  commit, are in `policy_spell_tests.rs`, `policy_combat_tests.rs`, `targets_tests.rs`
  and `tests/mana.rs`. Snapshots include RNG and proposed choices.
- **SYS-CORE-005 / B017:** target pairs, generic payment colors, attacker subsets,
  blocker maps and integer damage splits are independently enumerated in small
  positions (`targets_independent_all_pairs_and_capacity_boundaries`,
  `policy_spells_all_generic_colors_preserved_and_finish_requires_paid_cost`,
  `policy_combat_independent_subsets_maps_and_allocations`). Choices are factored,
  not an exponential flat action table. Reversal/cancel and privacy are tested.
- **SYS-CORE-006:** atomic plan assigns #18 the real-effect scalar part. The
  normal-reset replay quantum test crosses Growth/Bite, combat, cleanup and game
  end with full scalar equality; focused tests observe every private yield.
  No scheduler fairness, batch equality or collector transition claim: #27 and
  #20 retain those responsibilities.
- **SYS-CORE-007:** normal no-token two-deck resets have at most 80 physical cards;
  therefore 80 bounds each target/attacker/blocker/recipient list. Core vectors
  preserve complete lists; smaller explicit capacities error without mutation,
  clipping or a terminal result. Synthetic above-deck stress is labelled.
  [Target bound](../../targets.md#capacity-and-boundaries), [combat bound](../../combat.md)
  and `policy_spells_capacity_masks_stale_and_wrong_seat_at_every_stage` /
  `combat_linear_capacity_and_all_remaining_blockers` exercise boundary and
  one-below required capacity. Fixed tensors, tokens and runner quarantine remain
  #27/later owners; a core error is never recorded here as a completed game.
- **B026:** M1 portions of setup/randomness, priority, costs, targeting, vanilla
  combat, Growth changes, object identity, terminal, decisions, privacy and
  replay are covered above and by #17. Cast/ETB triggers, creature mana, discard
  costs, keywords, tokens and other card effects remain M2; batch/worker/tensor
  and RL/data integration remain their registered owners. None is counted as
  an M1 supported pass or a skipped mandatory test.
- **B029:** independently authored component expectations and compiled red/green
  remain durable. This audit adds acceptance assertions without altering rules.
  Fresh references retain strict-choice and comparator negatives. No outputs
  were regenerated into golden expectations.

## Fresh execution and reproduction

Run inside the managed Linux toolchain container, using the prepared external
XMage cache; do not invoke Docker inside the worker:

```sh
cargo test -p mtg-core --locked --lib integration_growth
./scripts/torture.sh
python3 scripts/instant_reference.py --cache /tmp/mtg-xmage --output /tmp/gh18-reference
python3 scripts/combat_reference.py --cache /tmp/mtg-xmage --output /tmp/gh18-combat.json
python3 scripts/terminal_reference.py --cache /tmp/mtg-xmage --output /tmp/gh18-terminal.json
```

Run the reference commands sequentially because they share the Maven cache.
The [verification receipt](verification.json) records completed commands and
hashes, with full torture output and reference artifacts retained here. Instant
execution repeats all eleven original neutral scripts on each real engine,
comparing 103 checkpoints and 409 ordered entries per run, plus live negative
controls and first divergences. The [reference scope and observation audit](../matched-growth-bite/README.md)
applies unchanged: strict identical choices, original CR/Oracle expectations,
actual stack/targets/stats/damage/zones/priority, explicit unobservable fields,
synthetic upkeep setup and no full-game reference claim. [Combat](combat.json) and [terminal](terminal.json) receipts
cover their declared narrower fields; neither proves whole-game agreement.

The [instant receipt](instant/acceptance.json) pins source/card/rules/build dependencies, bridge/native
source hashes and toolchains. All inventoried files are retained. Failed runs
are not agreement: an initial overlapping build encountered a temporary test
compile error while these new tests were being authored; the complete reference
command was restarted only after the tests compiled. No result from that failed
attempt is counted. Forge/full-pool/dual-reference release acceptance remains
mandatory at its registered stages.

README now links this scoped integration evidence and keeps M1 unverified until
its gate. Stale target API prose about unavailable combat/outcomes is corrected.
Quickstart commands are unchanged; torture runs their CLI/comparator checks.
