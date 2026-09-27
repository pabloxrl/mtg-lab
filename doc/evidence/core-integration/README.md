# Core integration audit — GH-17

Audited baseline: `5d13536b488ed2252313ef393e3c2106c8770d5b`.
Scope is the #62–#66 core prefix under [atomic delivery](../../programs/atomic-delivery.md),
not all of RFC §4 or the M1 gate. This delivery adds an integration ledger,
a bounded XMage count comparison and the acceptance crosswalk below; it changes
no production game rules. Completion is conditional on separate review,
protected merge and successful CI on that exact main commit, recorded in the
[workpad](https://github.com/pabloxrl/mtg-lab/issues/17#issuecomment-5854949076).

## Original acceptance and retained obligations

| Requirement / original clause | Executed core evidence / remaining owner |
| --- | --- |
| B013: authoritative Rust rules/state/decisions/views/RNG; external policies and tooling | `mtg-core` owns state and typed actions; `mtg-cli` calls it, `mtg-recorder` owns persistence, reference Python/Java remains test-only. Core dependencies are serde/JSON/SHA; no socket, HTTP, inference, telemetry exporter or clock API in core. Shared frozen identities are currently a private static module, not a separate `mtg-cards` crate. The RFC table is proposed package boundaries. Batch/Python/benchmark package separation and complete cards remain #23/#25/#27–#32, not certified here. |
| B014: language rationale, native ownership, no per-card Python callback | Native core paths and external test arithmetic are separate; no second Python rules engine. No Rust/Python speed comparison or throughput claim. Correctness-gated allocation/boundary profiles remain #25; GIL/binding work #27. |
| B015: compact numeric IDs, shared immutable cards, per-game mutable state and allocation reuse | `objects_frozen_identities_are_shared_and_match_manifest`, `objects_allocation_and_zone_change_literal_ledgers`, `objects_reuse_and_reset_never_revive_retained_handles`, `opening_reset_restarts_rng_reuses_allocations_and_isolates_games`. Complete zone ledger, foreign stores, generation exhaustion and retained capacities are checked. Card-driven return/token effects remain #18/#23. |
| B015: worker ownership, no per-game thread, batch observation allocation | Scalar APIs require exclusive mutable game access; no per-transition tasks/locks or thread per game. Actual shards/scheduler/batch buffers/fairness remain #27. This is an architecture audit, not a concurrency benchmark. |
| B015: explicit continuation; yield is not action/event/terminal/RL transition | `quantum_all_opening_boundaries_match_scalar_rng_and_decisions`, `quantum_explicit_ledger_and_changing_budgets`, `quantum_mulligan_really_yields_and_resume_is_not_an_action`, internal rejection-trial/one-unit tests. All seven rounds, both starters, explicit/seeded chance, 15 budgets and complete boundary comparisons. Effect suffix #18, snapshot suffix #19, scheduler fairness #27 and actual recorder boundaries #20 remain. |
| B015: versioned stable episode RNG, no clock/policy/scheduling dependence | `rng_episode_known_answers`, `rng_creation_order_and_policy_draws_do_not_change_environment`, `rng_unknown_versions_rejected`; independent Python integer vectors in [RNG evidence](../rng-v1/README.md), plus reset/replacement shuffle vectors below. No assumption that independent engines share PRNG algorithms. |
| B016: reset/decision/apply validate before mutation; candidate generation | [Opening](../opening/README.md) and [mulligan](../mulligan/README.md) tests compare complete private Debug state, RNG, counters, pending choices, ordered objects and capacities; control-game suffix checks remain. Wrong actor, stale/foreign/reset decisions, prior-generation candidates, wrong kinds, padding, missing/duplicate/wrong-count bottoms and invalid permutations are rejected. |
| Deterministic reset, London mulligans and ordered libraries | `opening_ordered_both_starters_and_mirrors`, `opening_shuffle_matches_independent_vectors`, `mulligan_random_replacements_match_independent_full_order_vectors`, every assigned catalog row below, and new `core_integration_reference_count_ledger_and_stale_reset`. Expected orders originate from pinned decks + independent arithmetic, not native snapshots. |
| First-player draw exception | Explicitly transferred to #67/#18 by #61; `turns_first_draw_positive_negative_interaction_regression` and [turn evidence](../turns/README.md). #18 audits full-turn interactions. This audit does not certify its reference coverage. |
| Full-game reset, damage/terminal outcomes | Explicitly transferred to #72/#18; [terminal evidence](../terminal/README.md). No early core pass substitutes for that integration audit. |
| B016: observe, snapshot/restore, versioned semantic replay and pending continuations | Atomic #73–#75 are delivered, but the full acceptance including pending spell/target suffix remains #19. These conceptual operations are not a single promised Rust ABI. |
| Contract default: limit in continuation | Core quantum stops without inventing a decision; collecting truncation reason/final seat views at safe boundaries remains #20/#29. No wall-clock stop-latency measurement or complete collector claim here. |

## System and catalog crosswalk

SYS-CORE-001 reset/opening prefix: opening/mulligan reset tests; complete played
scripts/mana/damage remain #72/#18. SYS-CORE-002: all RNG vectors/isolation tests.
SYS-CORE-003: `mulligan_sys_core_003_rejections_between_every_real_stage` and the
new integration ledger; actual spell/payment/target/combat suffix #18.
SYS-CORE-004: all four public object tests plus exhaustion/compact-slot unit tests;
card-driven identity #18/#23. SYS-CORE-006: all seven quantum tests, plus the new
ledger's three budgets; snapshot/effect/scheduler suffix #19/#18/#27.
SYS-CORE-009: `opening_invalid_config_preserves_complete_state_and_rng`, unknown
card/RNG/config rejection and checked identity exhaustion; CLI integration #21.
SYS-CORE-005/007 and 008 were explicitly transferred: factored actions/capacity
#18/#27, rules outcomes #72/#18 and reward ledger #76/#20. None is silently passed.

The current-main catalog assigns zero direct cases to #17 and 13 to its children:
`rules-setup-two-player-opening-negative` to #64; these 12 to #65:

- `rules-setup-mulligan-bottom-{positive,negative,interaction,regression}`
- `rules-setup-deterministic-reset-{positive,interaction,regression}`
- `rules-setup-ordered-draw-{positive,negative,regression}`
- `rules-decisions-stale-candidates-regression`
- `rules-setup-two-player-opening-positive`

The exact executable names are `opening_invalid_config_preserves_complete_state_and_rng`
and `mulligan_` + each full case ID with hyphens replaced by underscores.
[GH-65's exact checkpoint mapping](../mulligan/README.md#exact-catalog-mapping)
retains their setups and independent rules/RFC basis. All execute in normal test
discovery. First-draw/full-turn, reset-after-damage and spell/target/payment cases
remain in their transferred owners; no catalog text or ownership was changed.

## New cross-component and reference evidence

[Count ledger](../../../fixtures/reference/opening-counts.json) is original test
material derived from CR 103.5: initial hands seven, no free mulligans, cumulative
bottoms after each redraw, independent keep, forced keep at zero. Cases use both
starters and 0/1/2/7 mulligans. At each declaration after round `r`, the mulligan
seat has `7-r` cards and `33+r` library cards. Total single-card bottom choices
are `r*(r+1)/2`. These expected values were authored before either run.

[Native integration test](../../../crates/mtg-core/tests/core_integration.rs)
executes real frozen-deck seeded resets/shuffles/mulligans at quantum 1/7/10000,
reuses the same game across 24 executions, rejects retained pre-reset actions
and object handles, and probes wrong-actor nonmutation at every decision.
It compares the shared literal ledger, not XMage output. Existing full-order
vectors and object ledgers supply the separate identity/order/RNG assertions.

[Original XMage bridge](../../../references/xmage/OpeningCountsTest.java) invokes
the actual pinned engine's London phase. Its test constructor injects seven-card
hands and 33-card libraries of identical FDN basics; those are explicit synthetic
prefixes, not normal-reset proof. A test-only shuffle hook retains the supplied
all-basic permutation, avoiding independent reference randomness. The bridge scripts every declaration and bottom
selection, stops at first upkeep priority and exports every declaration's actor,
hand/library counts, final counts and number of bottom selections. It asserts
20 life and starting-seat priority. There is no AI or human fallback.

Only those common count/actor semantics are compared. Reference card identities,
initial shuffle, library order, per-episode PRNG, negative-action API, full-game
outcomes and Forge agreement are **not** established. Native independent-order
and rejection tests remain necessary. Broader normal-reset/capability/full-game
reference coverage remains #24/#38 and the M1 transferred integration owners;
this receipt is not full-game or dual-reference qualification. No upstream
unavailability is counted as agreement.

The bridge/fixture are authored here, not imported/adapted upstream tests. The
[consulted-source hashes](../../../references/xmage/opening-provenance.json),
existing pinned engine/dependency inventory and retained MIT notice establish
provenance. The runner verifies the consulted source bytes and exact toolchain
and dependency pins. Only metadata, original code and semantic counts are
committed; external engine/dependencies/card payloads remain in the cache.
The CR 103.5 count procedure is aligned; no combat-vintage rule is imported.

Reproduce inside the managed Linux Docker image (do not invoke Docker from a worker):

```sh
cargo test -p mtg-core --test core_integration
python3 -m unittest discover -s tests -p test_opening_reference.py
python3 scripts/opening_reference.py --cache /home/agent/.cache/xmage --output /tmp/opening-reference.json
./scripts/torture.sh
```

Use the [XMage setup instructions](../../../references/xmage/README.md) for a new
external cache. Missing source/toolchain/dependencies fails explicitly. Maven
runs offline, stdin closed, display unset and with a 300-second deadline.

## Validation and delivery

The original children preserve compiled behavioral red/green receipts:
[RNG](../rng-v1/README.md), [objects](../objects/README.md),
[reset](../opening/README.md), [mulligans](../mulligan/README.md),
[quantum](../quantum/README.md). This audit does not manufacture new rule failures
for unchanged production rules. The new comparator has its own genuine
[red](comparator-red.txt)/[green](comparator-green.txt): Python boolean `False`
was wrongly equal to count zero; typed JSON comparison now rejects it. Missing,
extra and changed checkpoint fields are also negative controls. Two initial
Java target-signature compilation errors were repaired and are not behavioral
red evidence.

README now links the audit and states the exact opening reference boundary;
no milestone stage verdict or usable simulation command changes. Its affected
native check path is exercised by torture. Separate review must assess README
accuracy along with this scope. No workflow, CI, test ownership or gate changes.

[Final torture receipt](torture.txt) passes after fetching unchanged current main:
134 Python tests, 201 workspace Rust tests/doctest each debug/release,
formatting/Clippy, documentation/program/catalog checks. No skips or weakened
expectations. [Native ledger output](native-green.txt), [exact-version XMage
receipt and all eight observed ledgers](xmage.json), and [independently checked
child exact-main CI receipts](dependencies.json) preserve execution evidence.
The RNG/reset/mulligan Python arithmetic oracles were rerun and each reproduced
its committed JSON byte-for-byte. Mature-engine agreement is 8 executed / 8
agreed / 0 disputed at the count boundary described above, not eight complete
game or catalog cases. Broader unsupported fields remain explicit.
