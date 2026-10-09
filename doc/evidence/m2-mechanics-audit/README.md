# Frozen-pool mechanics integration audit — GH-23

Audit base: `17aeefe91c9a23bd91854a8e8c3d7960a06b6be4`.
**PASS for the scoped mechanics integration audit, conditional on protected
delivery and exact-main CI. This is not the M2 gate.** This bounded audit
owns the M2 aspects of R0002-B010, B011 and B029. It retains every original
clause, twenty card definitions and the Goblin token. The current
[engine-validation plan](../../programs/engine-validation.md) governs scope.
The [workpad](https://github.com/pabloxrl/mtg-lab/issues/23#issuecomment-6079276519)
records candidate review, protected delivery and exact-main CI separately.

## Change and independent basis

The existing normal-game matrices omitted the explicit green/red seat assignment.
They now execute RG, GR, RR and GG with each starting seat and both native policies.
All prior assertions remain. The [CLI test](../../../crates/mtg-cli/tests/native_simulate.rs)
checks repeated unattended completions and zero failures/truncations. The
[recorded-game test](../../../crates/mtg-recorder/tests/full_pool_policy.rs) checks
normal reset, actual rules endings, capture on/off, quantum 1/MAX, semantic replay,
typed JSONL persistence/reload and invalid/stale/wrong-seat nonmutation. It also
requires all eighteen delivered decision kinds across the matrix and validates
both policies at each encountered kind. Native policy games cannot concede;
life/failed-draw reasons are checked against the terminal observations.

This is a missing coverage row, not a newly demonstrated rules defect. No
production rules, expectations, fixture inputs, source pins, compatibility
versions, policy, CI or workflow change is made. There is no invented behavioral
red for the matrix expansion. Original rules-derived reds remain in the child
reports; the [earlier audit](historical-audit.md), [red log](torture-red.log),
[patch](activation-mana-red.patch), [historical handoff](historical-handoff.json)
and [original source receipt](audit.json)
preserve this audit's actual three compiled activation failures.

Those failures were repaired by [GH-254 / PR #260](https://github.com/pabloxrl/mtg-lab/pull/260),
merged `9a4e8812ecf9fc019cb2c76812058840da89a543`, with
[exact-main CI](https://github.com/pabloxrl/mtg-lab/actions/runs/37956881953).
All three original functions are byte-identical in current normal discovery.
[Repair evidence](../activation-mana/README.md) adds private source reservations,
target-before-mana ordering, cancellation/commit/surplus, rejection, snapshots,
policy versions, typed recorder and normal-reset Shivan/Invoker replay. CR
602.2b/601.2g/605.3a supplies the timing requirement independently of engine output.
The original failed audit is historical, not an unresolved current defect.

## Executable and reference crosswalk

[The executable index](executable-index.json) pins source bytes and exact test
entry points by the groups used below. [The card index](cards.json) enumerates
all twenty cards plus the token. [The reference index](reference-index.json)
pins nineteen concrete inherited receipt files and the commits that introduced
their current bytes. It reports current-source mismatches explicitly. These
indices are traceability records, not generated expectations or test results.
Every listed Rust test uses ordinary test discovery; no ignored replacement suite
is introduced. Independent card/rules sources are [pinned here](../../card-manifests.md).

| Card/token | Native group and independently specified behavior | Executed reference receipt key |
| --- | --- | --- |
| Mountain, Forest | `mana_cast`: land limits, colored mana, explicit payment, immediate mana/no stack | `costs` |
| Swab Goblin, Bear Cub | `mana_cast`, `combat`: 2/2 casts, sickness, legal timing, simultaneous trade | `costs`, `shivan` |
| Axgard Cavalry | `activations`: tap cost, creature target, haste through cleanup, sick activation rejection | `haste` |
| Llanowar Elves, Druid of the Cowl | `mana_cast`: G immediately, sick tap restriction, cast-payment sources; Druid survives two damage | `creature_mana`, `costs` |
| Magnigoth Sentry | `combat`: 4/4 reach for 3G; blocks flying, reach does not make it evasive | `flying_reach`, `shivan`, `instant` |
| Tajuru Pathwarden | `combat`: 5/4 vigilance/trample; blocker lethal threshold and excess assignment | `trample` |
| Thornweald Archer | `combat`, `targets`: 2/1 reach/deathtouch; positive damage lethal, zero not lethal | `deathtouch` |
| Shivan Dragon | `activations`, `combat`: 5/5 flying; each R gives +1/+0, source incarnation and cleanup | `shivan` |
| Wildheart Invoker | `activations`: eight generic, explicit target, +5/+5 and trample, dead target/source and expiry | `invoker` |
| Dragon Fodder; Goblin token | `identity_tokens`: two distinct red 1/1 tokens on resolution, sickness, death/cessation, stale targets | `tokens`, `costs` |
| Goblin Surprise | `mana_cast`: exactly one mode, two tokens or +2/+0 to resolution-time controlled set; expiry | `surprise` |
| Thrill of Possibility | `mana_cast`: additional discard commits with mana, two ordered draws, failed second draw and private cancel | `thrill`, `costs` |
| Giant Growth | `targets`: +3/+3, target revalidation, simultaneous cleanup/damage removal | `instant` |
| Bite Down | `targets`: current source power, opponent destination, not fight, independent target revalidation | `instant` |
| Firebrand Archer | `triggers`: noncreature cast deals one to opponent before spell, even after source departure | `cast`, `trigger_composition` |
| Crackling Cyclops | `triggers`: noncreature cast gives +3/+0 through turn; dead/reentered source does not inherit boost | `cast`, `trigger_composition` |
| Viashino Pyromancer | `triggers`: actual ETB, explicit player target, two damage, source-independent ability | `etb` |

| Original requirement | Concrete acceptance and evidence |
| --- | --- |
| B010 frozen decks, mirrors/both starters, twenty names/token, printing/Oracle/rules pins, no artwork/runtime API | `tests/test_card_manifest.py`, `card_definition_tests.rs`, committed data manifest and card index; expanded `native_cli` and `played_recording` matrix. Manifest remains two 40-card decks with 16 lands/24 spells each; token is not a castable deck card. |
| B011 two players, 20 life, seven cards, London mulligan, skipped first draw, best of one/no sideboards | `opening` group and [M1 gate](../m1-gate/README.md): transactional reset, keep/bottom choices and both starters. `terminal_empty_draw_versus_empty_library_and_first_draw_skip` retains the first-draw boundary. |
| Full turn/phase/step, priority/consecutive passes, stack/resolution/SBAs | `opening`, `mana_cast`, `combat`, `triggers`, `terminal_cleanup`; [turn settlement](../turn-settlement/README.md), [spell settlement](../spell-settlement/README.md), [combat settlement](../combat-settlement/README.md). Real Driver tests preserve mandatory internal work before a decision. |
| Land plays, colored/generic explicit payments, creature mana timing | `casting_literal_payment_stack_resolution_and_priority`, `casting_generic_choices_and_atomic_rejected_final_command`, `creature_mana_priority_is_immediate_and_rejects_illegal_sources`; `costs`/`creature_mana`. Activation-payment sources are covered by the retained `activation_mana_*` functions and newer `shivan`/`invoker` receipts. |
| Additional discard, modes, targets/revalidation | `thrill_discard_atomic_ordered_draw_and_failed_second_draw`, `thrill_private_pending_snapshots_cancel_stale_and_quantum`, `surprise_modes_literal_affected_set_and_rejection`, `cast_boundary_restore_finish_or_cancel_at_every_target_and_payment_stage`; `thrill`/`surprise`/`instant`. |
| Creature casting, sickness/tapping, activated abilities, attacks/blocks/damage/marked damage | `mana_cast`, `activations`, `combat`; paid activation stack/priority, rejection and source-incarnation tests; `combat_declarations_and_simultaneous_trade_cub_and_swab`, `combat_sickness_taps_and_rejections_are_atomic`. All keyword/activation receipts supplement native tests. |
| Flying/reach/haste/vigilance/trample/deathtouch, multiple blockers, simultaneous damage | `flying_reach_pair_matrix_policy_actions_snapshot_and_quantum`, `tajuru_vigilance_trample_and_unordered_blocker_splits`, `deathtouch_zero_split_and_no_trample_rejection`, `deathtouch_trample_synthetic_seven_six_both_seats_quantum_and_rejection`; `combat` and keyword receipts. |
| Foundations allocation migration | `combat_modern_split_does_not_require_lethal_or_blocker_order`, `combat_quantum_all_modern_allocations_simultaneous_lethal_both_seats`: every legal split, no obsolete blocker ordering/response window; trample/deathtouch requirements remain. |
| Cast/ETB timing, controller ordering, APNAP | `mandatory_trigger_order_precedes_priority`, `trigger_apnap_dead_source_and_every_placement_snapshot`, incomplete/duplicate ordering rejection, cast/ETB tests and `cast`/`etb`/`trigger_composition`. Failed casts do not trigger. APNAP injections are explicitly synthetic. |
| Tokens disappear, zone-change new identity, temporary P/T and ordering | `tokens_lethal_bite_ceases_and_pending_growth_cannot_recreate_dead_target`, `shivan_stacked_source_return_cannot_receive_boost`, object generation tests, target-source current power and Surprise affected-set tests. `tokens`/`instant`/`shivan`/`invoker`/`surprise` observe their declared fields. |
| Life loss, empty draw, concession, simultaneous loss/draw, empty library alone | `terminal` tests, Thrill draw tests and `terminal` receipt. Mixed life/failed-draw losses settle together; concession invalidates continuations; empty library without attempted draw is ongoing. |
| Maximum hand size, discard, repeated cleanup, temporary-effect expiry | `cleanup_pending_trigger_discards_expires_resolves_and_repeats`, `cleanup_ordinary_has_no_priority_and_terminal_stops_before_untap`, `cleanup_driver_private_choices_typed_capture_and_snapshot_replay`, normal-reset recorder cleanup and `cleanup` receipt. Expiry/damage removal occurs before SBA, ordinary cleanup has no priority, exceptional cleanup repeats. |
| General mechanics for the pool; relevant continuous ordering | Shared mana, targeting, priority, combat keyword queries, identity and modification settlement operate on current game objects, not fixture IDs/deck sequences. Different creature stats/keywords, Growth plus activated boosts, source death, resolution-time affected sets and cleanup are tested above. Named ability variants remain disclosed in the current plan; this audit does not certify RFC 0003's future runtime catalog or arbitrary layers. |
| B029 test first; positive/negative/interaction/regression; independent expectations and impacted references | Every child report preserves its original reds and CR/card-derived expected checkpoints. The original audit red and #254 repair remain linked above. Current change only expands the existing independently justified gameplay matrix; no rule changes or reference adapter changes require new impacted reference runs. Full torture includes verify, debug and release. |
| Real scalar/policy/action/recorder/replay, private pending choices | `played_recording` index covers normal-reset per-mechanic Driver/Run tests and the expanded full-pool matrix. `cast_boundary_*`, activation private/cancel/rejection, snapshot and semantic tests cover continuations. APNAP/exceptional cleanup supplement these with explicitly synthetic snapshot replay, not fabricated normal-reset reachability. |

## Reference provenance and limits

The eight latest combat child receipts match every recorded native source hash at
this audit base. Shivan includes actual payment-window mana choices and the
Cub/Sentry repair. Older cost/trigger receipts retain their original exact source
versions; they are not relabeled as current-head executions. Their fourteen
source differences include the independently delivered activation repair,
instrumentation/Driver work and added tests. The unchanged cast/ETB/cleanup,
Thrill/token/mana mechanics retain their executed child evidence and run again
natively in full torture. The newer instant receipt differs at `game.rs`, and
the terminal receipt additionally at the instant adapter/test additions. The
`game.rs` difference only registers the new test module; the terminal runner
does not execute the added instant-suite controls. Exact
paths are in the index. A zero mismatch count on an aggregate receipt with no
source map does not claim it validates all source files.

The previous Cub/Sentry and mixed-loss gaps have concrete repaired receipts:
[combat](../m2-reference-combat/README.md),
[Sentry/Bite](../sentry-instant/README.md) and
[mixed terminal](../mixed-terminal/README.md). Their observation boundaries
remain intact. No new reference engine is executed by this documentation and
normal-game matrix change; no missing reference is counted as agreement.

All these reference positions are synthetic selected-checkpoint comparisons.
Mana staging normalizes pending native reservations explicitly; it is not public
state equality before commit. APNAP/repeat-cleanup injections and mixed terminal
SBA positions are not naturally reached full games. Invalid raw native commands
and constrained reference callbacks have the distinctions recorded by each pack.
The audit does not claim full-state equivalence, Forge breadth, AI-generated
reference games, new official-text retrieval, arbitrary card support, runtime
catalog admission or general layer semantics.

#24 retains the complete reference union, all 175 M2 catalog assignments and
at least 100 distinct reviewed scenarios; invocation counts do not establish
that floor. #25 owns scalar measurements; #26 retains the complete M2 gate.
RFC 0003 migration and reference-AI corpus stages remain separately registered
work. This component report cannot waive those obligations.

## Reproduction and delivery

Inside the managed toolchain, run:

```sh
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- ./scripts/torture.sh
```

This includes `./scripts/verify.sh`. No Docker invocation or host installation is
needed from a worker. The full suite remains mandatory; the new rows do not
replace any prior test. The [verification receipt](verification.json) and [complete compressed log](torture.log.gz)
record exit 0: **244 Python tests and 1,394 Rust test executions** across debug
and release, zero failed/ignored Rust tests. Both expanded matchup tests passed
in both profiles; every one of the 263 indexed native test names appears in the
passing log. Formatting, lint, documentation, program and catalog checks passed.
The run acquired the shared lock after approximately one hour queued. Fresh main
remained the audited base after execution. Runtime and test sources are unchanged
from `a0306530998d9bfddb1c71d0ecfb75a9e70e7084`; only final evidence/documentation
is added afterward. Full-log SHA-256:
`1dc568572dd2433617be69f3a49b874035ce5a40f6c9b41d62063c04d8a8b517`.

The preliminary prescribed read-only review passed with no findings on that
source candidate, checking retained assertions, README scope and all indexed
test names/hashes. Final clean-candidate review, PR and exact-main CI receipts
are recorded in the linked workpad; they are not inferred from a local test pass.

README impact: add this durable component evidence link and its limits. Setup,
commands and supported behavior do not change. M2 remains incomplete until #26;
independent review must check the README against this bounded verdict. Protected
merge, exact-main CI and parent evidence/handoff remain required for completion.
