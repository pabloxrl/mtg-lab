# GH-19 private views, snapshots and semantic replay integration

This bounded integration audits R0002-B016/B025/B033 on baseline
`d5fe803e3c51fa32c3eddbabf6f067424cf8808c`, after completed prerequisite #114.
It covers the implemented M1 slice, not the entire shared RFC blocks or the M1
milestone. Delivery remains conditional on independent review, protected merge
and exact-main CI in the [workpad](https://github.com/pabloxrl/mtg-lab/issues/19).

## Defect and independent expectations

A compatible snapshot with its top-level RNG field deleted and its checksum
recomputed was accepted, replacing the live game. RFC B016 requires pending
continuations and RNG state; absence cannot silently mean the explicit `null`
of an unstarted game. The compiled [red](red.txt) asserts `Corrupt` and received
`Ok(())`. The fix requires explicit presence of nullable RNG, decision, episode
and outcome fields during decoding. [Green](green.txt) checks each omission and
exact destination-byte preservation, plus all existing snapshot tests. Valid
unstarted/terminal snapshots remain accepted. The conservative source fingerprint
changes, so earlier artifacts explicitly reject; no migration is advertised.

No rules, candidate ordering, expected game outcomes, existing assertions or
workflow gates were changed. New tests enter ordinary Cargo discovery. Initial
test-authoring corrections to the cleanup kind name and subprocess test path
were not behavioral red evidence; the missing-RNG rejection above is the real
reproduction. Checksums detect corruption, not malicious forgery; full semantic
validation of arbitrary rewritten state is not promised.

## Original acceptance and catalog accounting

| Original clause / exact catalog cases | Executable acceptance and independent basis |
| --- | --- |
| Seat-filtered observations; masks, order and errors. `rules-privacy-side-channels-positive`, `rules-privacy-side-channels-negative`, `rules-privacy-side-channels-interaction` | New `integration_privacy_growth_snapshot_diagnostics_and_seat_jsonl_twins` uses both seats, seven hidden Forests versus Mountains, reversed unknown library tails, identical public state/history and the same real Growth cast/target/payment actions. Complete serialized policy views (including masks, candidate order, IDs and lengths) match after diagnostic snapshot/restore. Guessed private/out-of-range references return exactly `InvalidSelection` without state/RNG mutation. Frozen Growth text requires the Cub to become 5/5 without damage. Synthetic added permanents/cards are declared; normal-reset scripts separately establish reachable play. |
| Privileged full state separately named and absent from default views. `rules-setup-library-privacy-regression`, `rules-privacy-side-channels-regression` | Same added test materializes privileged diagnostics and snapshots, restores, then serializes only typed seat policy views as JSONL; checks private-field absence and identical bytes in hidden twins. Existing CLI `replay_verify_and_seat_filtered_inspect` executes actual opening replay seat export and requires only the seat's seven cards; `replay_errors_are_explicit_and_inspection_diagnostics_are_redacted` checks sentinel private identities never reach stderr on failed inspection. `policy_observe`/`observe` are seat paths; `snapshot`, semantic records/replays and `privileged_reveal_to` are explicitly restricted APIs. Full played replay is not a policy export. Core has no telemetry sink; M4 wire endpoints and M3 loaders retain their owners. |
| Snapshots retain pending continuations and RNG. `rules-replay-pending-snapshot-positive` | New `snapshot_played_pending_choices_fresh_process` snapshots every decision along the existing normal-reset land/Cub/Bite/Growth/cleanup/terminal script. Every distinct kind/action/stack-depth combination also loads in a fresh, bounded, closed-stdin process and applies the complete semantic suffix with reacquired references. Includes Bite source-before-destination, selected targets, provisional taps/payments, two-spell response stack, combat and real cleanup discards. Every local restore rejects old policy revision and wrong actor with exact state preservation. Full state/ordered zones/RNG equal uninterrupted execution; independently specified CR/card checkpoints require surviving 5/5 with two damage, undamaged 2/2 after cleanup, and ten two-damage attacks ending [20,0], P0 winner. |
| Internal work and RNG-consuming continuations (SYS-CORE-006 / SYS-REPLAY-001) | Extended `played_replay_fresh_process_and_quantum_suffixes` roundtrips snapshots at every internal yield at budgets 1/3/17 and compares complete normalized state after every semantic action. First encountered work families additionally load in a fresh process and complete the suffix. Existing `snapshot_every_random_reset_redraw_bottom_yield_matches_independent_vectors` preserves every opening RNG work cursor and later redraw against independently authored Python vectors; `snapshot_pending_explicit_chance_order_survives_all_seven_rounds` preserves explicit orders. No yield becomes a player action. Trigger choices remain M2; scheduler fairness/batching remain M3. |
| Restore preserves alternative decisions. `rules-replay-pending-snapshot-regression` | New `snapshot_nondefault_combat_fresh_process`, both seats, loads synthetic two-blocker allocation in a fresh process, chooses 1+1, saves/loads the selected allocation and finishes. CR 510.1c allows this split without lethal-first ordering: attacker dies, both 2/2 blockers survive with one damage, life stays [20,20]. Opponent sees no provisional split. Existing policy tests independently enumerate all 0+2/1+1/2+0 divisions, subsets and blocker maps. This fulfills the catalog's combat alternative; ordered triggers are not claimed implemented. |
| Reject incompatible/corrupt artifacts atomically. `rules-replay-pending-snapshot-negative` / SYS-REPLAY-002 | New missing-field regression plus `snapshot_versions_corruption_and_truncation_preserve_destination`, `snapshot_structural_corruption_with_recomputed_digest_is_atomic`: version/fingerprint/checksum/truncation/structure errors leave live state unchanged. Rules/card definitions are bound by the snapshot engine fingerprint, rather than an independent rules field. Replay tests change explicit rules/cards/action/engine/PRNG/shuffle/config pins, mandatory checkpoints and chance/config data and require rejection. No implicit migration or fallback. |
| Replay configuration, versions, seeds and semantic actions / SYS-REPLAY-003 | `played_replay_normal_reset_independent_checkpoints`, `played_replay_strict_corruption`, `replay_*` and `actions_*` tests retain explicit config/pins/master seed/episode, semantic birth+zone incarnation and exact consumption. Wrong actor/kind/object incarnation, missing/extra choices and raw indices fail. Intermediate life/target/priority mutations report first divergence despite unchanged final winner. Snapshot equality is corroboration, never the source of literal rules expectations. |

All eight directly owned catalog IDs above retain their original requirements.
Public history follows the documented historical-revelation contract:
`views_revelations_are_historical_seat_scoped_and_reset` checks that subsequent
hidden moves do not refresh remembered locations. Fixed numeric tensors,
padding/reuse and batch policy encoding remain #27; dataset loaders #28/#29;
protocol/log endpoints #34/#35. This audit does not substitute typed structured
records for those later mandatory numeric and transport tests.

## Replay plan and source provenance

GR-010/011/012: existing opening and played replay suites reject wrong
actor/kind/action, missing/extra choices, stale/same-name object substitutions,
changed chance order/seeds and altered intermediate checkpoints. Explicit ordered
opening decks and all seven mulligan rounds are independently checked; raw row
indices cannot replace semantic actions. GR-020/021/022: normal-reset replay and
opening vector tests retain 40-card configuration, zero/one/multiple mulligans,
ordered redraw/bottom and first-draw coverage. Actual first-player draw behavior
also remains in the turn/core integration tests. The three-engine full-game
matrix and cross-engine chance bridge expansion remain #24/#38; no such agreement
is inferred from native replay equality.

Original author-written tests use pinned CR 103/117/305/601/608/508–510/514/704
and frozen card text. Existing fixtures and component provenance are unchanged:
[views](../views/README.md), [snapshots](../snapshot/README.md),
[actions](../actions/README.md), [played replay](../played-replay/README.md),
[opening replay](../replay/README.md), and
[matched Growth/Bite references](../matched-growth-bite/README.md).

## Reproduction and delivery receipts

Run inside the managed Linux image, without invoking Docker in the worker:

```sh
cargo test -p mtg-core --test snapshot
cargo test -p mtg-core --test played_replay
cargo test -p mtg-core --lib integration_privacy_growth_snapshot_diagnostics_and_seat_jsonl_twins
cargo test -p mtg-core --lib snapshot_nondefault_combat_fresh_process
./scripts/torture.sh
```

The fresh [reference receipt](instant/acceptance.json) records two executions of
all eleven matched Growth/Bite scenarios in the native engine and pinned XMage,
with independent checkpoints and live negative controls. The retained inventory
pins source, rules/cards, bridge/build dependencies and each output. This is
synthetic matched-scenario corroboration, not Forge or full-game agreement;
[reference observation limits](../matched-growth-bite/README.md) remain unchanged.
The snapshot decoding correction changes no Magic rule. Full `./scripts/torture.sh`
passed after integrating current main: see [verification receipt](verification.json)
and [complete debug/release log](torture.txt); independent review and exact
merged-main CI belong to the workpad/PR.
README now links the bounded integration and explicitly preserves the M1 gate,
full-pool, full-game reference, tensor and CLI limits. Quickstart commands are
unchanged; their existing CLI/comparator checks run in torture.
