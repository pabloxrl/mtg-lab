# Structured in-memory trajectory acceptance (GH-152)

Bounded partial R0002-B036/B037 delivery on baseline
`a2a681b8561e1aab73bee041f16c24209c997b18`. This is the canonical structured v2
record/reader contract, not a game-driving collector, durable sink or milestone
audit. Existing v1 records/tests and legacy unavailable-pending-view regression
remain unchanged. No dependency edge correction was necessary: #151/#76/#111/#112
supply exactly the inspected contracts. No rules or policy behavior changed.

## Independent expectations and normal discovery

`crates/mtg-core/src/trajectory_v2_tests.rs` runs in ordinary core tests, debug
and release, via `cargo test -p mtg-core trajectory` and full torture.

| Test suffix (`trajectory_v2_`) | Independent basis and checks |
| --- | --- |
| `ordered_bottom_owned_and_atomic_rejections` | CR 103.5 and existing opening contract: two mulligans produce a two-card ordered bottom submission, rows 5 then 1. Retain both semantic choices and all seven candidates, never fake a single selected row. Unknown schema, stale generation, wrong cardinality/duplicate rows, discontinuity, reset/foreign final frames reject transactionally. Header and returned data ownership. |
| `same_seat_interleavings_rewards_and_zero_decision_seats` | RFC 0002 §8 literal actors 0,1,0,1: links P0 [2,final], P1 [3,final], elapsed decisions [2,2]/[2,1], terminal concession returns +1/-1 once. Both zero-decision seats receive final credit with no invented transitions. |
| `played_pending_privacy_cancellation_and_full_inputs` | Normal valid ordered 40-card green decks, seed 42/ordinal 0, real keeps/lands/Cub cast through turn five. Real Growth targeting/payment/tap/pay/cancel retains every input and complete Submission. CR 601.2, frozen Growth G cost; pending target is Cub battlefield row 2, source Forest row 0. Hidden opponent hand/library twins have identical P0 sequences; rejected commands and provisional actions preserve opponent views, and legacy capture still explicitly fails. Six same-seat microchoices are one cancelled attempt; then two passes produce literal durations [1,1,1,1,1,1,2] and logical durations [1,1,1,1,1,1,2], one cancellation at microchoice 5. |
| `combat_domains_subsets_maps_allocations_and_backtracking` | Explicit synthetic two-Cub-per-seat setup, real commands thereafter. CR 508/509/510: independently enumerate 2² attacker subsets, 3² blocker maps, three integer compositions of power 2 over two blockers, plus omitted zero recipient representation. Real flat table contains only FinishCombat; actual factored domains and every provisional replacement/allocation remain separate from that table. |
| `cleanup_multiple_discards_draw_failure_and_nonacting_reward` | Explicit synthetic nine-card hand at real cleanup: CR 514 requires two discards, rows 8 then 0 retained in full. Concession gives acting loser -1/nonacting winner +1. Explicit simultaneous life-zero SBA setup gives draw 0; false terminal/blank failure reject; failed record quarantines. |
| `played_commit_retains_payment_targets_and_owned_policy_data` | Same normal-reset prefix; real Growth commitment and resolution yields literal 5/5 Cub from frozen +3/+3. Complete targets, provisional pool/sources/cost, resulting stack and full action history retained; absent stats remain absent, supplied checkpoint/log probability/value copy by value; buffer/seat-read mutation and reset leave stored episode unchanged. Seat JSON excludes replay reference and seeds. |
| `terminal_action_rewards_once_and_truncation_pending` | Explicit synthetic Cub attacks into life 2, real policy combat settlement: one decision delta [+1,-1], boundary [0,0], nonacting loser -1, repeat finish rejects. All three truncation reasons retain genuine actor-only pending observations and zero reward. |
| `invalid_maps_allocations_and_target_cancellation` | CR 509/510 and the actual domains reject duplicate/wrong-seat/wrong-zone blockers, unattacking targets, unknown allocation attackers/recipients, wrong totals, duplicate recipients, overflow and masked FinishCombat without mutating stored data. Repeated cancel-targets ends exactly two attempts, and retry starts a new logical ID. |
| `rejection_matrix_is_transactional` | RFC integrity: unknown/wrong-domain/duplicate/empty selections, stale revision, nonfinite statistics, invalid status/timing, unchanged/foreign frames, discontinuous finalization, unsupported observation version and capacity errors leave records unchanged. Continuing declaration cannot silently start another logical action. |

Exact Observation equality against the separately obtained action-time policy
input checks losslessness, not rules correctness; the literal and independently
enumerated expectations above establish the required semantics. Fixtures drive
existing game APIs only and are not a substitute collection driver. Synthetic
edge setups do not claim full played-game or reference-engine acceptance.

## Test-first and correction evidence

[Initial red](red.log): five tests compiled and failed at the intended missing
structured capture (`Unavailable`) against nonimplementing v2 API scaffolding,
before recorder implementation. The first mulligan setup attempt incorrectly
assumed bottoming happened only after keep; corrected to the existing API's
bottom-after-each-redraw contract before recording these reds. That setup error
is not counted as intended behavioral evidence.

[Sparse-damage red](sparse-damage-red.log): independently extending the allocation
case to the existing `assign_combat_damage` documented omitted-zero semantics
exposed an overstrict recorder cardinality check. The engine accepted the legal
2-to-one-blocker submission, but recorder append returned `InvalidChoice`.
Removed that extra cardinality restriction; retain uniqueness, membership and
exact power-total validation. No engine expectation changed.

The hidden-hand twin extension initially caused the fixture's first-row cleanup
strategy to discard different *public* cards. The script now explicitly discards
Forests in both games (a legal CR 514 choice); the original full-sequence equality
assertion is unchanged. No existing test was edited, removed, weakened or skipped.
The reviewer must check these fixture corrections and root README accuracy.

## Verification and limits

[Focused green](green.log): nine v2 tests pass. [Initial full torture](torture-initial.log)
and [final full torture after fresh-main integration](torture-integrated.log) both
passed in the managed Linux container. The final run executes 143 Python tests
and 319 Rust checks (including two doctests) per debug/release profile, plus
documentation/program/catalog validation, formatting and Clippy. The initial run
began before the final added assertions; the integrated run covers the complete
candidate. [Source hashes and command receipt](verification.json) pin that result. Separate read-only review and protected merge/exact-main CI receipts
belong in the issue workpad and PR; this document alone is not a merge verdict.
No new reference agreement is claimed because no rules changed.

Use `trajectory::v2::Frame::capture`, `Recorder::new/append/finish`, and
`Episode::seat`; [contract, timing, version and producer responsibilities](../../trajectories.md).
The producer must report every accepted decision and its true post-action frame;
a domain-valid Submission is not an execution receipt. V2 persistence remains
#153, owned lifecycle #154, aggregate #117, full trajectory integration #20,
CLI #120/#21, and M1 gate #22. All original catalog and later-stage obligations
remain. Root README now distinguishes implemented v2 memory support from planned
v2 JSONL/collector support; quickstart commands are unchanged.
