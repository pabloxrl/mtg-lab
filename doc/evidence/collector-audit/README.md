# Scalar collector integration audit — GH-154

Related to #154 and #117; partial R0002-B036/B037 only. This audit exercises the
real delivered Driver → canonical v2 Recorder → Run → JSONL/Manifest → Registry
→ local publication composition. It introduces no production behavior or second
recorder. Delivery is conditional on independent review, protected merge and
successful CI on that exact merged main commit; receipts belong in the issue
workpad/PR. #117 still owns its original aggregate audit, #20 full trajectory
integration, #120 CLI integration, #21 the CLI/baseline audit and #22 the M1 gate.
No milestone, full-pool, batch, sharding, Parquet or trainer completion is claimed.

## Independent oracle and retained inputs

Normal discovery: `cargo test -p mtg-recorder --test collector_audit`.
The new `both_starting_seats_full_independent_ledger_publish_reload_and_replay`
test uses master seed **154**, ordinal **0**, both starting seats, and an explicit
40-card green-mirror order retained in `collector_audit.rs::order`. There is no
synthetic state injection. Both players keep, pass every priority window and
discard sorted hand row zero in each required cleanup. No land/spell is played
in this script: it deliberately provides a small independent oracle for *every*
field, complemented by the real combat script below.

CR 103.5 gives seven opening cards and 33 remaining. CR 103.8a skips the starter's
first draw. The other player draws 33 times on turns 2 through 66, then loses on
the attempted empty draw at turn 68 (CR 104.3c/704.5b); an empty library alone
never loses. CR 117 and 508.8 give paired priority passes, with no blocker/damage
choices when no creatures attack. CR 514.1 gives one discard after each successful
draw: hand counts return to seven. Thus the independent action count is
**2 keeps + 14 first-turn passes + 66 × (16 passes + 1 discard) + 2 final upkeep
passes = 1,140**. Winner is the starting seat; life stays [20,20], terminal reward
is +1/-1, and only the final accepted pass carries global terminal reward.

The ledger independently constructs sorted own hands from the configured deck,
public counts, graveyard entry order, historical revelations, all public zones,
turn/active/acting seats, mana, terminal result, candidate rows/masks, and null
pending/stack/combat/factored fields. `doc/views.md` and
`doc/policy-decisions.md` specify field ordering and masks. Without lands or mana,
all spell choices are masked; Forest land choices are legal only during the
active player's main phase. Frozen Cub 2/2 and Sentry 4/4 characteristics are
literal constants; other unsupported characteristics remain absent under the
published M1 view contract. No production legality function computes expectations.
Only opaque current revision/generation tokens are bound from the endpoint, as
required to submit an action. Complete submissions and those tokens are then
compared unchanged after capture and persistence.

Every action-time actor **and opponent** observation is compared to this oracle.
Every recorded row, actor/seat index, logical/micro index, full submission,
next actor, reward and terminal/truncation flag is checked. Both final views,
footer counts and same-seat next/final inputs, elapsed decisions/actions and
once-only returns are checked. Paired capture-enabled/disabled Drivers compare
full snapshots including RNG after every action, normalizing only independently
allocated owner scopes/store identities. Results survive subsequent reset and
reject stale inputs. The actual files are reopened, hashes/byte and decision
counts checked, every reloaded observation compared to the independent ledger,
and authorized replay reconstructs the complete normalized actual final state/RNG.

The oracle must detect omitted/extra observations and deliberately wrong life or
mask checkpoints. The strict writer also rejects omitted/duplicate decisions
from these real episodes; the reader rejects incompatible versions and corrupted
published bytes. These negatives do not generate new expected outputs.

`played_target_payment_cancel_and_combat_survive_publication` re-executes the
existing seed-160, ordinal-0 **27-turn normal-reset** script using real Run-derived
provenance and publishes/reloads it with separately authorized replay. The
original core test still executes all its assertions. The additive shared helper
returns its owned result; no old assertion is removed or skipped. The recorder
also rediscovers the original capture module tests. The script checks Cub casts,
Growth target/payment cancellation and retry, both seats' choices, all two
attacker subsets, four blocker maps and six sum-five damage allocations, literal
[2,3] assignment, a 5/5 surviving four simultaneous damage while two 2/2 blockers
die, cleanup, and ten subsequent two-damage attacks to [20,0]. Full inputs in this
combat ledger are matched to the actual pre-action endpoint, with independently
specified choices/timing/domains and named rules checkpoints; they are **not**
claimed to be a second independent all-field observation oracle. The exhaustive
all-field ledger above supplies that distinct acceptance evidence.

`zero_decision_and_nonacting_rewards_survive_publication_for_both_losers` adds
normal-reset zero/one-decision concessions for both losers through the complete
publisher, loader and replay path. Literal boundary rewards and unassigned versus
last-transition returns are asserted exactly once. Knowing the replay UUID is
insufficient: deny-all reads fail in every publication helper.

## Every retained acceptance clause

All paths below are normal executable discovery, not proposed sibling work.
Full torture executes them together with the new composed-path tests above.

| Retained clause | Executable evidence |
| --- | --- |
| Owned reset/advance/submission, semantic completeness, invalid versions/actors/stale/illegal inputs | `mtg-core/tests/episode.rs`; `capture::direct_submit_cannot_bypass_capture_and_invalid_stats_headers_are_atomic`; independent 1,140-row ledger |
| All action-time observations/candidates/masks/full submissions, both seats | Independent all-field ledger for both starts; exact combat publication roundtrip |
| Factored attack/block/damage domains, target/payment cancel/retry and logical timing | `capture::complete_played_ledger_targets_payment_cancel_retry_and_factored_combat`, re-executed by composed combat publication |
| Ordered bottom/discard, missing selections | `capture::multi_bottom_and_hidden_hand_library_twins`; real full-game cleanup ledger; `episode_capture_tests::owned_capture_preserves_two_ordered_discards_and_rejects_missing_second` explicitly isolates an otherwise unreachable two-discard component boundary, not a substitute played game |
| Consecutive/interleaved seats, same-seat next/final inputs, elapsed actions | Combat `check_seats`; independent full-game transitions and final views |
| Terminal/nonacting/zero-decision returns exactly once | New composed zero/one-decision concession test for both losers; real empty-draw and combat endings |
| Capture on/off identical state/RNG | Independent paired full-game drivers after every submission; original paired combat script after submission and settlement |
| Pending privacy/hidden twins, both endpoints and restricted replay | Full-game independent opponent view; original `growth` and hidden-hand/library twins; `episode::replay::tests` and publication authorization tests |
| Owned buffer/reset lifetime, stale/post-end input | Full-game retained result after reset; combat result after reset; collector persistence `writer_drain_seal_flush_and_late_failure_propagate_and_originals_survive` |
| Statistics present only when supplied | Combat row-two supplied log probability/value/checkpoint preserved exactly through publication; independent full-game `PolicyInfo::default`; capture missing-probability rejection; `structured_buffer_ownership_and_optional_statistics` |
| Decisions/turns/time/work/records limits, pending choice truncation, failure precedence | `mtg-core/tests/budgets.rs`, including real terminal at 1,140 decisions, controlled clocks, pending payment and record/domain capacity |
| Every started ordinal and completed/truncated/failed/incomplete accounting | `mtg-recorder/tests/collector.rs` mixed-result tests; `publication_tests::diagnostics_preserve_real_truncation_failure_and_incomplete_accounting` |
| Actual config/decks/versions/seat/limits and trusted supplied policy provenance | Real Run headers for both complete scripts; `provenance_matches_independent_pins_and_actual_config_not_caller_labels`; mismatched header/config/policy/limit negatives |
| JSONL counts/checksums/completeness, invalid/missing/extra fields | Both complete scripts published/reloaded; new real missing/duplicate decision rejection; collector corrupt-sink and `structured_contract` negatives |
| Backpressure, append/drain/seal/flush/write/sync/publication errors | Collector `append_backpressure_and_drain_failures_are_not_silent_drops`; `writer_drain_seal_flush_and_late_failure_propagate_and_originals_survive`; all eleven publication fault/corruption/collision tests |
| Opaque separately authorized replay reconstructs actual game | Both full-game scripts and zero/one-decision concessions; `episode::replay::tests` wrong-ID/config/history/corruption/lifetime failures; publication same-ID different actual-result rejection |
| Corrupt publication/interruptions/no-clobber/uncertain commit | `publication_tests`, including actual hard-link collisions and explicit sync/withdrawal uncertainty; full-game corrupt bytes negative |
| No fabricated replay for truncation/failure/incomplete | `episode::replay::tests`; real publication diagnostic mix; default loader rejection and explicit diagnostic mode |

The public trust and resource limits remain those in `doc/collector-publication.md`:
trusted local owner/grant callback, retained owned result required for disk replay
resolution, separate roots, checksums are integrity rather than authenticity,
record-count/file limits are not total RSS/I/O deadlines, and reserved run IDs
are not automatically retried. Both-seat datasets need authorized offline access.

## Verification and test-first accounting

This is an acceptance-only change: no rules or production implementation changed,
so no fabricated failing feature scaffold is used. Initial compilation mistakes
are not behavioral-red evidence. An early oracle assertion incorrectly omitted
Magnigoth Sentry's exposed 4/4 characteristics; the frozen card definition and
view contract justify the correction, not agreement with implementation output.
No existing expectation was changed. New corrupt/missing/extra-input negatives
remain in normal discovery. No cached reference execution is claimed or required
for this rules-unchanged audit; previous reference requirements remain intact.

Initial composed-path focused run: nine tests passed (including the seven
rediscovered original capture tests). The final zero/nonacting publication test
and explicit row/footer boundary assertions were then added. Final unchanged-test
full torture, source hashes and independent review evidence are recorded in
`verification.txt` and the linked PR/workpad; focused runs are not substitutes.

README impact: update the scalar component rows to link this composed acceptance,
while keeping #117/#20 aggregate and M1 gates explicitly pending. No quickstart
command, setup, production API, capability scope or milestone table changes.
The existing quickstart command checks remain in full torture.
