# M1 integration prerequisites

Operations [#106](https://github.com/pabloxrl/mtg-lab/issues/106) repairs the missing implementation prerequisites found by the #18–#21 audits. All 18 initial component tasks were delivered, but their limited interfaces do not satisfy complete played-game integration. This plan adds implementation work; it does not declare M1 complete.

The pinned RFC, all original acceptance clauses, existing requirement owners and the entire 320-case catalog remain unchanged. Added requirement owners are partial implementation owners. The original issues retain final acceptance, including every directly owned catalog case. No future-stage scope is pulled forward or waived.

The [lossless trajectory prerequisite plan](trajectory-prerequisites.md) supersedes
the original #117 prerequisite row below with #154 (and transitive #152/#153).
The original scope remains aggregate acceptance after those children.

## Work and dependencies

| Issue | Deliverable | Direct prerequisites | Integration owner |
| --- | --- | --- | --- |
| [#107](https://github.com/pabloxrl/mtg-lab/issues/107) | Bounded stack and spell settlement | #106, #66, #70, #72 | #18 |
| [#108](https://github.com/pabloxrl/mtg-lab/issues/108) | Bounded combat damage settlement | #107, #71 | #18 |
| [#109](https://github.com/pabloxrl/mtg-lab/issues/109) | Bounded turn and cleanup settlement | #108, #67 | #18 |
| [#110](https://github.com/pabloxrl/mtg-lab/issues/110) | Safe structured turn and land decisions | #106, #73, #68 | #19 |
| [#111](https://github.com/pabloxrl/mtg-lab/issues/111) | Safe spell target and payment decisions | #110, #70 | #19 |
| [#112](https://github.com/pabloxrl/mtg-lab/issues/112) | Safe combat and cleanup decisions | #110, #71 | #19 |
| [#113](https://github.com/pabloxrl/mtg-lab/issues/113) | Semantic action references for played M1 games | #111, #112, #75 | #19 |
| [#114](https://github.com/pabloxrl/mtg-lab/issues/114) | Full M1 replay checkpoints and verification | #113, #109 | #19 |
| [#115](https://github.com/pabloxrl/mtg-lab/issues/115) | Matched Growth and Bite reference scenarios | #106, #70, #71 | #18 |
| [#116](https://github.com/pabloxrl/mtg-lab/issues/116) | Scalar dataset run manifest | #106, #77 | #20 |
| [#117](https://github.com/pabloxrl/mtg-lab/issues/117) | Complete scalar game trajectory collector | #114, #116, #76 | #20 |
| [#118](https://github.com/pabloxrl/mtg-lab/issues/118) | Versioned legal random M1 opponent | #111, #112, #78 | #21 |
| [#119](https://github.com/pabloxrl/mtg-lab/issues/119) | Versioned deterministic M1 heuristic opponent | #111, #112, #78 | #21 |
| [#120](https://github.com/pabloxrl/mtg-lab/issues/120) | CLI played-game policies and capture | #117, #118, #119, #79 | #21 |

The manifest orders dispatch, not architectural dependencies: player-interface work, reference scenarios and dataset metadata do not wait for unrelated quantum work. Random and heuristic policies are independent siblings. Semantic action resolution has its own executable contract before full replay; the collector uses the completed replay contract, and CLI capture uses the real collector. One worker remains configured.

## Acceptance per implementation task

### #107: Bounded stack and spell settlement

Extend owned core work to accepted priority passes, stack resolution and resulting state-based/terminal checks for M1 spells. Scalar execution drains the same continuation. Preserve exact-once payment, revalidation and effects. No player decisions, observations of provisional state or rewards at internal yields. Excludes combat/turn settlement, owned by sibling tasks.

Compiled behavioral red then green: actual responding Growth/Bite sequence at quantum 1, boundary and large budgets equals independently specified scalar checkpoints, identity, RNG and terminal result; replaying a resume never repeats effects; commands while pending, wrong seat, stale IDs and capacity errors are rejected without mutation. Snapshot owned work serialization must remain compatible or explicitly version-rejected.

Partial RFC ownership: R0002-B016, R0002-B017, R0002-B026, R0002-B029.

### #108: Bounded combat damage settlement

Move vanilla combat damage and resulting state-based/terminal settlement into the shared owned continuation. Preserve simultaneous damage, blocked status, all legal damage allocations and no blocker ordering. No turn cleanup refactor.

Quantum 1 versus larger budgets across multi-blocker simultaneous lethal damage, removed blockers and both seats yields identical literal checkpoints and RNG. No provisional damage observation or extra response window; pending/duplicate/stale actions cannot mutate state or duplicate damage.

Partial RFC ownership: R0002-B016, R0002-B017, R0002-B026, R0002-B029.

### #109: Bounded turn and cleanup settlement

Bound untap, draw, step transitions and cleanup/discard settlement through owned cursors. Preserve simultaneous damage removal and boost expiration, real discard choices and required repeated cleanup. Scalar path drains identical work. All M1 automatic work is then budgetable; no batching scheduler.

Quantum 1/boundary/large comparisons for boosted damaged creatures, cleanup with and without discard, untap, first draw and empty-library terminal. Literal rule-derived intermediate/final states, no extra decision/reward, pending rejection unchanged, bounded unit accounting and existing opening quantum regressions.

Partial RFC ownership: R0002-B016, R0002-B017, R0002-B026, R0002-B029.

### #110: Safe structured turn and land decisions

Define versioned seat-authorized structured candidates, legal masks and safe submissions for opening, priority/pass, land play and standalone mana choices. Use stable visible references and validated decision generations, never privileged handles in policy inputs. Existing opening API stays supported. Explicit unsupported errors for spell/combat families until their siblings deliver; do not silently pass.

Normal-reset both-seat opening/land/mana/pass script matches literal states. Hidden hand/library twins produce identical authorized views, row ordering, masks and errors. Guessed hidden references, stale generations, wrong actor and capacity failure preserve full state/RNG. No fixed tensor, Python or trainer scope.

Partial RFC ownership: R0002-B016, R0002-B025, R0002-B033.

### #111: Safe spell target and payment decisions

Extend the real structured policy interface to casting, source/destination targets, provisional mana/payment, finish and cancel. Acting seat sees its authorized pending choices; other seat sees only committed public information. Public stack/targets remain inspectable without revealing provisional selections. No combat or replay implementation.

Real Growth/Bite response and cancellation through safe submissions in hidden-information twins. Test source-before-destination, partial target invalidation, all payment choices, wrong seat/stale/hidden references and exact nonmutation. Replace the existing unavailable-spell-view expectation only with independently reviewed stronger positive and privacy rejection coverage.

Partial RFC ownership: R0002-B016, R0002-B025, R0002-B033.

### #112: Safe combat and cleanup decisions

Extend structured player decisions/submission to attacker subsets, blocker mappings, modern damage allocation and cleanup discards, with public combat relationships. Reuse factored core choices; do not enumerate an exponentially flat action table or change combat rules.

Both-seat reachable combat and discard scripts through policy interface; independently enumerate small subsets/mappings/allocations. Hidden twins preserve authorized ordering and masks; invalid/duplicate/wrong-seat/stale rows preserve state. Explicit capacity failures, no silent clipping. No policies or collector.

Partial RFC ownership: R0002-B016, R0002-B025, R0002-B033.

### #113: Semantic action references for played M1 games

Add versioned semantic action encoding/decoding for every implemented M1 decision, including land, mana, cast, targets, payment, combat and cleanup. Resolve semantic object identity plus zone incarnation against current state; never persist process-local handles or candidate row indices. Opening replay remains supported; full replay envelope/checkpoints are the next sibling.

Hand-authored action records for a normal-reset spell/combat sequence resolve to the intended legal choices in fresh processes. Wrong actor/kind/zone incarnation, duplicate identity confusion, missing required fields and incompatible action versions fail without mutation. This component must have runnable tests without the future full-game replay envelope.

Partial RFC ownership: R0002-B016, R0002-B025, R0002-B033.

### #114: Full M1 replay checkpoints and verification

Extend record/verify beyond opening using delivered semantic actions. Preserve version/config/rules/cards/PRNG pins and exact choice consumption. Semantic checkpoints include turn, priority, zones, stack, target/combat relationships, mana, damage, pending decisions and terminal state. Privileged artifacts stay distinct from seat exports. No dataset collector.

Normal-reset land/creature/Growth/Bite/response/combat/cleanup/terminal script reproduces independently derived intermediate states in fresh process. Missing/extra choices, chance/duplicate identity corruption, incompatible versions and first differing life/target/priority checkpoint fail even if winner matches. Compare scalar and quantum suffixes; original GR-010/011/012/020–022 obligations retained.

Partial RFC ownership: R0002-B016, R0002-B025, R0002-B033.

### #115: Matched Growth and Bite reference scenarios

Extend pinned XMage test bridge with original shared neutral scripts for Growth responding to Bite, source power at resolution, target departure/partial invalidation, blocker killed by Bite and cleanup with/without discard. Native execution uses real rules; bridge translates only APIs/IDs. No copied engine expectation or AI fallback.

Execute strict identical choices/order in native and real pinned XMage. Compare all declared priority/stack/target/stats/damage/zones checkpoints; mark unobservable fields. Pin source/card/bridge provenance and retain repeated receipts plus comparator mutation negatives. Failed/unavailable references never pass. Forge/full-pool/dual-reference release obligations remain unchanged.

Partial RFC ownership: R0002-B017, R0002-B026, R0002-B029.

### #116: Scalar dataset run manifest

Versioned scalar dataset manifest with engine/rules/cards/observation/action versions, deck/config/policy identities, seats, reward/time conventions, capture selection, single-file inventory/checksum and declared completeness. Reuse canonical JSONL without implementing collector, shards, Parquet or trainers.

Independently handwritten valid manifest/episode pair roundtrips; missing/incompatible fields, wrong inventory/checksum, inconsistent provenance or completion are rejected. Truncated/failed runs remain explicit and excluded from completed data by default. Opaque replay references cannot expose privileged artifacts to policy readers.

Partial RFC ownership: R0002-B036, R0002-B037.

### #117: Complete scalar game trajectory collector

Connect actual action-time structured views/candidates/masks/submissions to the canonical in-memory/JSONL recorder and run manifest. Record all microchoices, logical timing, same-seat next/final views, once-only terminal rewards including nonacting seats, optional policy stats only when supplied, and opaque authorized replay linkage. Propagate backpressure/writer failures. No CLI wiring.

Normal-reset scripted played game compared to independent decision ledger, then reload JSONL and authorized linked replay. Same-seed capture on/off state/RNG equality; both seats, zero-decision seat, truncation/failure, reset/owned-buffer lifetime, pending choices and privacy. Missing decisions and writer errors must fail visibly; no fabricated sibling implementations.

Partial RFC ownership: R0002-B036, R0002-B037.

### #118: Versioned legal random M1 opponent

Implement a native legal-random policy over only the real seat-authorized interface for every M1 choice. Policy RNG is independently versioned and separated by episode/seat from environment RNG. Declare supported content; no unsupported-choice fallback. Excludes heuristic, recording and CLI capture.

Seeded complete both-seat games reproducibly select semantic actions and use land/creature/spell/combat choices in forced reachable cases. Hand-specified candidate distribution/selection vectors, unchanged environment RNG on policy evaluation, hidden twins and wrong/stale candidate rejection. Account explicit truncation; do not relabel passive games as active policies.

Partial RFC ownership: R0002-B020, R0002-B039, R0002-B041.

### #119: Versioned deterministic M1 heuristic opponent

Implement one documented deterministic heuristic using only structured authorized inputs, fixed scoring/tie rules and all M1 choice families. Independent of random policy, collector and CLI capture. Unsupported choices fail explicitly, never fall back to pass.

Hand-authored choice examples force land, creature, response, payment, targeting, blocking, allocation and discard decisions. Hidden twins give identical actions. Normal-reset complete games from both seats with reproducible actions and genuine terminal accounting. Policy strength is not a rules oracle.

Partial RFC ownership: R0002-B020, R0002-B039, R0002-B041.

### #120: CLI played-game policies and capture

Wire native policies and explicit semantic scripts into existing simulate/verify commands with canonical capture and opaque replay links. Declare episode/work/capture budgets and preserve headless structured errors. No second engine, recorder or performance qualification.

Closed-stdin/no-display played games produce run manifest, JSONL and replay with exact reload equality. Scripted missing/illegal choices leave state unchanged; writer failure, SIGTERM, deadline and truncation preserve completion/failure boundaries and all requested-episode accounting. Compare capture on/off same-seed runs; existing commands remain supported.

Partial RFC ownership: R0002-B020, R0002-B039, R0002-B041.

## Retained integration and test ownership

- #18 waits for bounded turn/combat/effect settlement and matched instant references. It retains every original scalar-slice acceptance clause and all eight directly owned catalog cases, including real reset-to-terminal scripts, capacity/invalid-action nonmutation, target/cleanup interactions and SYS-CORE-006 real-effect quantum evidence.
- #19 waits for full played-game replay, which includes safe choice interfaces and bounded automatic work. It retains all eight owned catalog cases: four privacy side-channel cases, setup-library privacy regression and three pending-snapshot cases. Verify fresh-process restore at source-before-destination, payment/target stages, response stack, nondefault combat allocation, cleanup and internal yields; compare complete RNG/suffixes, reacquire references, reject stale/wrong-seat actions and corrupt/incompatible snapshots before replacing live state. Seat-only exports must not reveal privileged replay state.
- #20 waits for the real scalar collector, including manifest, safe choices, replay and existing JSONL prerequisites. It retains every B036/B037 M1 clause and its restored-terminal no-duplicate-reward regression: full run/header/decision/footer, all action-time fields, same-seat timing/rewards/final views, owned buffers, JSONL/replay equality, privacy, capture-off equivalence and storage failure propagation.
- #21 waits for CLI capture, including both native policies and collector/replay prerequisites. It retains the full command matrix, invalid-action nonmutation, explicit work/episode/capture budgets, no-display/closed-stdin operation, interruption accounting and honest initial active-policy rollout measurements. Random/heuristic opponents are M1 requirements; broader M2 workloads and performance qualification remain M2.
- #22 still audits the complete M1 requirements after #17–#21. Passing children never implies whole-block acceptance. M3 tensor/batch/subset/shard/trainer obligations and M5 full-pool XMage/dual-reference/full-game qualification remain with their existing owners. M1 matched scenarios do not count as those later gates.

Every new behavior enters ordinary executable test discovery with independent rule/contract expectations, positive and negative assertions, retained minimized regressions and full Docker torture validation. Rules changes execute relevant cached reference cases. Missing siblings, unavailable engines, synthetic-only evidence or compile failures cannot count as completed integration. Independent candidate review must explicitly assess any replacement of the unavailable-spell-view assertion and require stronger privacy coverage.

## Queue recovery

This registration operation #106 must pass separate review, protected merge and exact-main CI before dispatch. The coordinator then preserves old blocked reports, appends the dependency amendment to #18–#21, removes only their resolved scope-block labels and gives each an explicit deferred agent-resume-authorized grant. They remain unready until all newly registered prerequisites have delivered acceptance, review, merge and exact-main CI evidence. Normal handoff consumes the grant atomically when adding ready. No worker is authorized to clear a hold/block or bypass a dependency.

The coordinator records operation completion, closes it only after validation, and queues exactly one eligible child. New children have no previous dispatch counter. The four resumed integration workspaces receive a one-time archived-counter reset while the controller is stopped, preserving files and workpads. Future blocks still require diagnosis; this plan does not authorize automatic retry-limit resets.

For the driver: no issue unblocking or PR review is needed. Read the dashboard for current work and parent #7 for evidence. Agents advance through these prerequisites, then rerun the four integration checks and M1 gate. Essential product decisions or missing access remain reasons to ask for input.

## Matched-reference refinement

The [matched instant reference prerequisite plan](instant-reference-prerequisites.md) splits #115 into #136–#139 after its source audit identified missing executor and continuation contracts. It supersedes the assumption that all matched scenarios fit one implementation PR. #115 retains aggregate acceptance and waits for these children; original requirements and test expectations remain intact.

## Played CLI prerequisite refinement

Operations #175 registers exactly #176–#181 in the [played CLI prerequisite
plan and complete acceptance crosswalk](cli-prerequisites.md). It refines the
preserved unsuccessful #120 proposal into owned semantic input, native
simulation, script routing, canonical publication, played replay verification
and structured dataset validation. #120 audits all original composed acceptance
after #179/#180/#181; #21/#22 and later gates remain unchanged. Historical
prerequisite rows above are retained; the current manifest and this refinement
supersede the original #120 implementation boundary. Registration delivers no
new CLI capability and does not complete M1.
