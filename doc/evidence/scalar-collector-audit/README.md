# Aggregate scalar collector acceptance — GH-117

This audit repeats the original scalar collector acceptance against the real
Driver → canonical v2 recorder → Run → JSONL/manifest → authorized replay → local
publisher composition delivered by the prerequisites. It does not infer aggregate
success from closed children. Base: `b428a73be643db55105d3324d07605fa7fa1d9d3`.
Delivery requires separate candidate review, protected merge and passing CI on
that exact main commit; those receipts are retained in the
[issue workpad](https://github.com/pabloxrl/mtg-lab/issues/117#issuecomment-5882414737)
and linked PR. Local executable results and source/log hashes are in
`verification.txt` and `torture.log` beside this report.

Scope is partial R0002-B036/B037, without CLI wiring. #20 retains full trajectory
integration and every original catalog expectation; #120/#21 retain CLI
integration/audit. #22 alone decides M1 completion. Batch equality, sharded
Parquet, Python/trainer integration, capture benchmarks, full-pool/reference and
M2–M5 acceptance remain with their existing owners. No program metadata, tests,
workflow or enforcement is removed or weakened here.

## Executable aggregate evidence

Run `cargo test -p mtg-recorder --test collector_audit` for the composed tests;
`./scripts/torture.sh` remains mandatory and executes the component failure and
privacy regressions as well. The detailed independent oracle and all retained
inputs are in the [preceding composition report](../collector-audit/README.md)
and `crates/mtg-recorder/tests/collector_audit.rs`. The table below maps EVERY
original #117 acceptance clause to executable normal discovery, not future work.

| Original obligation | Executable evidence and independently specified result |
| --- | --- |
| Actual action-time structured observations, candidates, masks and full submissions | `both_starting_seats_full_independent_ledger_publish_reload_and_replay`: normal reset, seed 154/ordinal 0, both starting seats. The independently constructed actor/opponent views cover every field; the literal deck order and CR 103.5/103.8a/104.3c/117/508.8/514.1 yield exactly 1,140 decisions and empty-draw loss on turn 68. Only opaque endpoint revision/generation tokens are borrowed. |
| Record all microchoices and logical timing, including pending choices | `played_target_payment_cancel_and_combat_survive_publication` runs the real 27-turn seed-160 script through Run and publication. Explicit casts, targets, mana payment, cancellation/retry, factored attacker/blocker/damage submissions and literal checkpoints retain every original assertion. Full input equality here uses pre-action endpoint values; it is not claimed as a second independent all-field oracle. |
| Ordered multi-card submissions | New `ordered_mulligan_choices_survive_aggregate_publication_for_both_starting_seats`: seed 117/ordinal 0, both starting seats, two mulligans, explicit bottom rows [5,1], five-card final hand (CR 103.5). Six literal submissions and logical IDs 0–5 survive persistence/reload. A missing second selection fails before mutation and produces no record. Existing capture multi-bottom and real full-game cleanup remain; the two-discard synthetic component edge is not presented as played-game evidence. |
| Same-seat next/final views and logical elapsed time | Full independent game compares every per-seat link and final view, global/seat indices and half-open elapsed intervals. Original combat `check_seats` covers consecutive same-seat microchoices and opponent interleavings. |
| Once-only rewards, both/nonacting/zero-decision seats | Full game checks starting-seat +1, other-seat -1 only on the last accepted decision; composed zero/one-decision concessions cover both losers and boundary/unassigned credit. New mulligan test verifies zero decision rewards and only footer boundary reward on concession. Aggregate return is not another reward event. |
| Optional statistics only when supplied | Combat row-two supplied checkpoint/log probability/value survive exact publication roundtrip. Full-game defaults and new mulligan durable `{}` prove unsupplied fields stay absent, per JSONL v2 contract. Existing capture and structured-contract tests reject unavailable behavior probabilities and invalid supplied values. |
| Normal-reset played game versus independent ledger, JSONL reload and authorized linked replay | Both full scripts publish real captured results and reopen actual files. `publish_check` checks exact canonical rows, manifest byte/count/SHA-256 inventory, denies unauthorized replay reads and reconstructs actual full final state/RNG via the existing replay verifier. The new mulligan test uses that same real composition. No fabricated sibling collector, recorder or replay exists. |
| Same-seed capture on/off state/RNG equality | Paired full-game Drivers compare full normalized snapshots after every submission; original paired combat tests additionally compare after settlement. Only owner scopes/store identities are normalized, not RNG or game data. |
| Reset/owned-buffer lifetime | Both complete scripts retain results across reset and reject stale commands. New mulligan result is retained across reset before publication. `collector::writer_drain_seal_flush_and_late_failure_propagate_and_originals_survive` preserves original owned results across writes/failures. |
| Truncation/failure and pending choices | Normal-discovery `mtg-core/tests/budgets.rs` covers actual decision/turn/time/work/record limits, terminal precedence and pending-payment stop. `mtg-recorder/tests/collector.rs` and `publication_tests::diagnostics_preserve_real_truncation_failure_and_incomplete_accounting` retain final observations and zero terminal rewards for truncation, account for every ordinal, reject ordinary loading of unfinished runs and never fabricate completed replay for failed/incomplete results. |
| Privacy and opaque authorized replay linkage | Independent actor/opponent observations; original `capture::multi_bottom_and_hidden_hand_library_twins` and pending `growth` checks; replay registry wrong-ID/config/history/corruption/lifetime tests; publisher authorization and same-ID/different-result rejection. Both-seat datasets remain authorized offline artifacts, never a live policy feed. |
| Missing decisions fail visibly | Full-game writer rejects removed/duplicated real decisions; independent ledger catches missing/extra observations and wrong life/mask checkpoints. Driver prevents direct submission bypass, and Run rejects missing/duplicate/reordered/mismatched results. New incomplete bottom submission rejects without consuming a decision. |
| Backpressure and writer failures propagate | `collector::append_backpressure_and_drain_failures_are_not_silent_drops` checks fail-overflow and blocking drain with write failures and bounded high-water; `writer_drain_seal_flush_and_late_failure_propagate_and_originals_survive` checks drain/seal/flush failures. Publication fault tests cover write/sync/collision/corruption and uncertain commit, with no silently advertised partial success. |
| Actual run manifest provenance | Both full scripts and new mulligan script derive headers from real Run configuration. Collector provenance tests compare independent pins and actual config, reject wrong policy/config/header/limits and validate sealed bytes/counts/checksums. Caller policy identity/statistics remain explicitly trusted. |

## Audit changes and test-first accounting

This delivery adds acceptance coverage and evidence only; no rules or production
behavior changes. The new test was written before any implementation change and
requires none. Its initial assertions exposed two mistakes in the new test:
core default statistics serialize nulls whereas documented durable v2 omits them;
external concession closes the last global row while reward remains at the
footer boundary. Assertions were corrected to the existing storage/boundary
contracts, preserving all pre-existing tests. These are not claimed as production
behavioral-red evidence. No existing expected outcome was regenerated or changed.

The added script has literal choices/cardinality/timing/count/reward expectations,
not generated snapshots. It independently enumerates the seven legal bottom rows
and mask, rejects a short two-card selection, verifies retained result ownership,
then checks all six persisted submissions and absent statistics. It complements
the full all-field oracle rather than claiming random shuffled hand identities
have independently specified expected values. Existing normal-discovery suites
remain intact. Rules are unchanged, so no new mature-engine run/agreement is
claimed; the later mandatory reference gates remain unchanged.

## Supported use and limits

Use the existing [publication API](../../collector-publication.md): derive Run
headers, reset/capture actual Driver results, register completed results under
opaque replay IDs, publish into separately managed roots, then load manifest and
JSONL. Resolve replay separately with explicit authorization and the retained
owned result. Failed/incomplete runs are diagnostic accounting, not valid training
episodes; truncated rows require diagnostic mode. A caller must propagate errors.

This is trusted local access control, not hostile-host isolation. Checksums give
integrity, not authenticity. Record/file bounds do not bound total RSS or I/O
deadlines. Reserved publication IDs are not automatically retried. There is no
capture sampling, CLI wiring, new policy or whole-pool claim in this delivery.

README now links this precise aggregate audit while preserving full trajectory,
CLI and M1 limits. Stage table, setup and quickstart commands are unchanged;
their existing automated checks remain in full torture. Live queue status belongs
in the Program workpad, not the README.
