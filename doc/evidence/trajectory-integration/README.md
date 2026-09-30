# M1 trajectory integration acceptance — GH-20

This is the scalar in-memory/JSONL acceptance audit for RFC 0002 B036/B037,
after the real collector prerequisites. Base:
`af5c71e53e544d03a0bb6f4e94790ecc40baee3a`. Child closure is not acceptance:
the executable composed path and all existing regressions run again here.
Delivery is conditional on independent candidate review, protected merge and
successful CI on that exact main commit. Those receipts are retained in the
[workpad](https://github.com/pabloxrl/mtg-lab/issues/20#issuecomment-5858777985)
and linked PR. Local execution and source/log hashes accompany this report.

The verdict covers M1 trajectory integration, not the M1 milestone. CLI capture
and active policy wiring remain #120/#21; #22 owns the M1 gate. M3 batch,
Python, Parquet, recurrent/trainer and seed-subset obligations and M5 reference
qualification remain mandatory. No program scope, test owner, policy or gate
is changed.

## Independent inputs and added regression coverage

Run `cargo test -p mtg-recorder --test collector_audit`. The real composition is
Driver → canonical v2 capture → Run → bounded JSONL/manifest → local publication
→ strict reload → separately authorized semantic replay. No sibling component
is replaced by a mock. Existing v1 and v2 tests remain in normal discovery.

The existing seed-154/ordinal-0 full-game ledger fixes the entire ordered green
deck, both starting seats, every keep/pass/cleanup submission, all authorized
observation fields, candidate order/masks, time, reward and ending independently
of engine output. CR 103.5/103.8a/104.3c/117/508.8/514.1 imply 1,140 decisions,
68 turns and the nonstarter's empty-draw loss. Only opaque revision/generation
tokens are taken from the live endpoint. The seed-160 27-turn played script
adds land/creature/Growth/Bite targeting, payment, cancellation/retry and factored
combat, with literal semantic choices and intermediate checkpoints. Its complete
input equality uses actual pre-action endpoints and is not a second independent
all-field oracle. Seed 117 adds two ordered mulligan bottoms [5,1], six decisions,
both starters and rejected missing selections. These original scripts and
assertions are preserved; see the [aggregate crosswalk](../scalar-collector-audit/README.md).

New ordinary-discovery coverage:

- `restored_terminal_polling_preserves_once_only_persisted_rewards` implements
  the sole catalog case directly owned by #20,
  `rules-terminal-rewards-once-regression`. Both normal-reset wins are restored
  from real completed snapshots. Outcome/view queries and core resume at quantum
  1 and maximum retain the exact terminal state; a second concession rejects.
  The collector rejects advancement with `Ended`, retains state/history/records,
  and rejects a second finish. Repeated durable loads and same-seat reads keep
  1,140 decisions, one +1/-1 credit per seat and zero boundary reward. Injecting
  another terminal boundary credit fails semantic validation.
- `canonical_trajectory_and_authorized_replay_reload_in_fresh_process` reopens
  the actual manifest/JSONL and separately granted replay in a child process,
  with stdin closed, DISPLAY absent and a 30-second child deadline. The child
  also checks the independently handwritten structured fixture. It returns
  every canonical header/decision/footer field and normalized reconstructed
  final state/RNG for exact comparison to owned records. Allocation capability
  namespaces alone are normalized, as in the existing ledger; RNG is retained.
  Default trajectory loading never resolves a replay reference. Parent replay
  access uses the existing authorization/binding API, including its deny test.
- `episode::budget_tests::disabled_capture_never_materializes_trajectory_frames`
  instruments the real Driver frame constructor only under `cfg(test)`. Normal
  reset, keeps, passes, concession, finish and reset build zero trajectory frames
  with capture disabled. Capture enabled is a positive control. The counter is
  thread-local, so parallel tests cannot mask an unwanted frame construction.

These are acceptance additions, not a rules or runtime behavior change. The
only production-file edit is test-only instrumentation. Initial new-test errors
were an optional-footer compile error and expecting Driver advancement to return
Terminal instead of documented `Ended` (see `doc/episode-driver.md`). Both were
corrected without changing existing assertions or production behavior. Neither
is claimed as production behavioral-red evidence. The component red/green and
mutation receipts remain in the linked prerequisite reports. No new reference
engine agreement or full-pool qualification is claimed.

## Complete original acceptance and RFC crosswalk

| Assigned clause | Executable evidence and result being checked |
| --- | --- |
| B036 sparse terminal rewards, stable seats, draws, concession, no shaping | Full independent game and new restore regression: zero interim reward, +1/-1 exactly once by persistent seat. `trajectory_rules_terminal_rewards_once_positive/negative/interaction` retain independent synthetic lethal/nonlethal/simultaneous-draw cases. `zero_decision_and_nonacting_rewards_survive_publication_for_both_losers` covers boundary credit with no invented actions. |
| B036 external limits versus failed samples | `mtg-core/tests/budgets.rs` checks real decision/turn/injected-clock limits, terminal precedence, record/work/capacity failures and pending-payment final views. `collector::actual_decision_truncation_is_diagnostic_and_never_a_rules_completion`, `mixed_actual_failures_and_incomplete_ordinals_never_become_training_rows` and publication diagnostics preserve distinctions. `trajectory_truncated_transition_bootstrap_and_failed_completed_recording` retains the independent gamma=1 target 0.75 and quarantines failed records. |
| B036 explicit reset and true final views | Full scripts retain owned records before resetting; final views contain the old terminal hand/zones/outcome and match the independent ledger. Structured conversion/reset and queue ownership tests preserve old data. There is no scalar autoreset wrapper; wrapper/slot cases remain M3. |
| B036 logical decisions, time and discount | Synthetic `[0,0,1,1,0,1]` ledger has explicit same-seat links and half-open elapsed intervals. V2 records distinguish continuing/committed/cancelled actions; real casts, cancellation/retry and combat retain microchoice IDs. Full game checks every interval. Manifests/records declare undiscounted episodic returns; unsupported conventions reject, no nonunit discount mode is advertised. |
| B036 fixed policies, seats, seed/version/limit provenance | `collector::provenance_matches_independent_pins_and_actual_config_not_caller_labels` plus manifest tests bind real config/decks/versions/limits and fixed per-seat policy IDs. Private seed/config/action history is bound in the restricted replay. Policy identities/statistics are trusted caller inputs, not secretly inferred. Native deterministic random/heuristic policies are delivered separately; CLI wiring remains #120/#21. |
| B037 distinct artifacts | Manifest + canonical observations/submissions/rewards are trajectories. Opaque IDs link separate privileged replay bytes through explicit authorization. Sampled diagnostics cannot be loaded as sealed training episodes; failed/incomplete data require explicit diagnostic handling. No sampled trace is used as trajectory or replay evidence. |
| B037 independently versioned manifest/header/decision/footer | `manifest_contract::{handwritten_pair_roundtrips,every_required_field_and_unknown_fields_rejected,incompatible_versions_and_provenance_rejected,nested_required_fields_duplicates_and_invalid_declarations_rejected}` and `structured_contract` check independent fixtures, required fields, pins, counts, checksums, version rejection, seat assignments and limits. Schema v1/v2 and JSONL encoding versions are explicit; legacy v1 remains supported. Every composed result reloads through that existing contract. |
| B037 exact action-time fields and identities | Full independent ledger compares all actor/opponent views, actual flat/factored domains, ordered full submissions, masks, semantic choices, global/per-seat indices, logical/micro IDs, rewards, next actor and flags. Played spell/combat and ordered-mulligan scripts supply continuation coverage; no fabricated selected flat row replaces a multi-selection. Visible semantic references stay in policy inputs; privileged birth/incarnation records stay in replay. |
| B037 optional statistics and action-time ownership | `structured_buffer_ownership_and_optional_statistics`, `jsonl_policy_statistics_preserve_every_float_bit`, v2/capture policy tests and the played publication script preserve supplied fields and exact float values. Unsupplied durable fields stay absent; probability-requiring consumers reject them. Reset/input mutation and delayed drain retain owned observations, candidates, masks and metadata. |
| B037 global stream, same-seat links, intervening time/reward, nonacting/zero-decision seats | Synthetic ledger plus `structured_same_seat_readers_preserve_rewards_time_and_privacy`, full independent game and capture `check_seats` check next same-seat decision/final view, not opponent next-global view. Once-only terminal/boundary reward reconciles with aggregate returns, which are not extra reward events. New restored-terminal regression covers the exact catalog scenario. |
| B037 genuine final views, truncated/failed and completeness | Budget, collector, structured and publication tests cover all endings. Completed game and complete recording are distinct. Failed/incomplete records remain excluded by default; diagnostic loading never silently repairs corrupt seals. Scalar explicit reset preserves IDs and owned final observations; shard/fragment/recurrent/autoreset clauses remain M3, not claimed here. |
| B037 collection/loading privacy | Hidden hand/library twins and pending-choice tests preserve authorized input/order/masks. Seat readers exclude opponent observations/private finals and restricted seed metadata. Strict structured validators reject unauthorized pending/hand shapes. Run/replay binding rejects wrong ID, config, history, authorization and corruption. Two-seat datasets remain privileged offline artifacts relative to either live seat. |
| B037 JSONL, recorder separation, optional persistence | mtg-core stays independent of storage encoding; recorder uses bounded queues and scalar file/record limits. Exact owned in-memory/JSONL equality plus authorized replay runs in-process and, newly, after process restart. Existing full played capture on/off comparisons preserve complete state/RNG after each accepted submission and settlement. |
| B037 capture selection and disabled overhead | Scalar capture is all decisions or off; the manifest records this. The new test-only frame counter checks no extra frame work when off. Every accepted selected decision is counted and missing/duplicate decisions reject. Deterministic seed-subset execution and telemetry/batch mode matrices remain M3. No outcome-based or sampled-decision dataset is advertised. |
| B037 bounded backpressure, failure propagation and publication | `structured_blocking_is_measured_and_does_not_drop_rows` measures queue wait; collector append/drain/seal/flush failure tests and publication write/sync/link/directory/corruption failpoints return explicit errors, preserve diagnostic fragments and never advertise missing data as complete. Prior runs cannot be clobbered; uncertain publication is explicit. Scalar native errors propagate; CLI failure exit integration remains #120/#21, sharded kill/recovery tests M3. |

All six B037 acceptance bullets are accounted for: (1) native roundtrip and
linked replay above; (2) batch ordering M3; (3) same-seat/interleaving/endings
above; (4) scalar ownership/reset/queue/failure/publication above, wrapper
autoreset/shard reassembly/process-kill M3; (5) scalar privacy/capture equality
above; (6) real trainer export/reload and comparative throughput/memory
benchmarks M3. Neither later work nor CLI work is silently counted as passed.

## Stable systems and data-test ownership

| Cases | M1 evidence and retained stage boundary |
| --- | --- |
| SYS-CORE-008; catalog terminal/rewards-once positive/negative/interaction/regression | Independent reward/SBA tests, actual empty-draw wins, concession/zero-action seats, limits/failure and restored-terminal regression above. Synthetic draw tests are labeled synthetic, not full matches. |
| DRL-001/004; SYS-DATA-002 | Handwritten actor/time ledger, reversed winning seats/draw, actual continuing/cancelled choices, same-seat readers and literal full-game final ledger. Gamma=1 only; trainer discounting remains its owner. |
| DRL-002 | Real bounded Drivers for all limit reasons and failures; native recorder bootstrap/quarantine checks, persistence diagnostic/default rejection and terminal precedence. |
| DRL-005; SYS-DATA-001 | Independent hand-authored v1/v2 records, both real played scripts and ordered mulligans; exact full-field reload and linked replay, now also fresh-process. Native scalar JSONL delivered; CLI, Parquet, Python/numeric batches remain their assigned integrations. |
| DRL-006; SYS-DATA-004 | Owned input mutations/reset, retained immutable results, blocked/drained writer tests; full state/RNG capture pairs. Neighbor-slot/batch/worker/Python lifetime cases M3. |
| DRL-007; SYS-DATA-010 | Required-field/version/provenance rejection, absent/supplied statistics, probability preconditions, exact floating-point preservation. Real trainer mask-softmax recomputation remains M3. |
| DRL-008 | Full both-seat ledger, hidden twins, private pending fields, seat-only readers, strict schema and separately denied/granted replay. Model-forward/padding/critic tests remain M3. |
| SYS-DATA-005 | Full played all/off equality and new instrumented disabled-frame check. Seed-subset/telemetry/worker matrix remains M3 as DRL-013/014. |
| SYS-DATA-007 | Actual native collector/write/publication errors and filesystem inspection; incomplete data never silently advertised. CLI exits remain #120/#21; later sharded recovery remains #28. |
| DRL-003/009–024; SYS-DATA-003/006/008/009 | Existing scalar tests provide relevant component evidence only. Wrapper/batch resets, recurrent/fragment loaders, Parquet, subprocess shard kills, deterministic subsets, actual trainers and capture benchmarks remain M3/M5 and are not waived. |

## Use, limits and README impact

Use the existing [publication API](../../collector-publication.md): derive
headers from Run, capture actual Driver results, register completed replay IDs,
publish to separate dataset/replay roots, load Manifest/JSONL and grant replay
access separately. Policy consumers use same-seat readers. Root README links
this scoped acceptance and retains v1-only CLI validation, pending CLI capture,
Parquet/Python limitations and incomplete M1. Setup, quickstart commands and
milestone table do not change; their existing checks run in full torture.

Access is trusted local-owner authorization, not hostile-host isolation.
Checksums are integrity checks, not authentication. Limits bound record/file
sizes, not total RSS or I/O deadlines. Unfinished internal-work stops may lack
final views and are explicitly incomplete. Reserved publication IDs have no
automatic retry. No power-loss durability, performance, full-pool rules,
playing-strength or new reference agreement is claimed.
