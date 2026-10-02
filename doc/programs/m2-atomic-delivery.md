# M2 atomic delivery amendment

Operations [#80](https://github.com/pabloxrl/mtg-lab/issues/80) registers the following
bounded deliveries after the verified M1 gate. This is a plan, not new card support,
executed test evidence, completion of #131–#133, or an M2 verdict. Original issues
#23/#24/#25 remain full acceptance audits; #26 remains the independent M2 gate.
All original RFC blocks/source bytes, catalog IDs/setup/actions/expectations/bases/
stages/owners and previous milestone gates remain unchanged. Requirement ownership
is additive only. Other stages, authorization, review, CI, concurrency and budgets
are unchanged. Explicit #80 edges prevent execution from an unmerged proposal.

## Established interfaces and dependency decisions

Inspected main `503ab665372122fd7a8c08b7ba8e6ba77ce525c2`.
[M1 gate receipt](https://github.com/pabloxrl/mtg-lab/issues/22#issuecomment-5943196087)
and [module extraction receipt](https://github.com/pabloxrl/mtg-lab/issues/130#issuecomment-5862456385)
include independent review, protected merges and successful exact-main CI.

| Current code/contract | Consequence for this split |
| --- | --- |
| `mtg-core/src/game.rs`, `opening.rs`, `work.rs`: private Game owner, compatibility re-export and owned quantum executor (#130) | No second executor. #131 narrows existing sibling access; no new framework. #130 remains standalone completed history, not a new program task. |
| `casting.rs::PendingCast`, `mana.rs::Payment`, `targets.rs`, `turns.rs` | #131 is one state-boundary refactor. Card mechanics use its atomic target/payment semantics. |
| `card_identities.rs`, duplicated `casting::cost`, combat/target card-key branches; frozen `data/cards/foundations_micro_v1.json` | #132 centralizes six supported definitions first; it must not silently enable the other fourteen cards. |
| `mtg-recorder/src/lib.rs::from_core_v2`, `collector.rs`, `structured.rs` | #133 replaces JSON conversion under existing handwritten wire/capture tests; schema/privacy remain owned by their existing modules. |
| `policy::{Choice,Observation,apply_policy}`, `actions::{encode,decode,apply}`, snapshot and played replay | Each mechanic owns its corresponding policy/semantic/replay representation alongside scalar rules, avoiding a later giant integration implementation. |
| `episode::Driver::{advance,observe,submit,submit_record,finish}`, `Run::persist`, publication | Real owner, recording and CLI paths already exist. #208 extends policies/support checks, not rules or collector ownership. |
| `mtg-policy::{LegalRandom,Heuristic}`; `mtg-cli` native loop and active benchmark | Metrics and benchmark contracts can be tested on the delivered M1 game, independent of pool implementation. Full-pool measurements join only at #217. |
| Existing pinned XMage bridges, neutral fixtures, strict choices and checkpoint comparisons | Every mechanic runs impacted real references in its own PR. #209–#212 deliver the exact composed catalog executions/bridge extensions below, not replacement mechanics. |

#131/#132/#133 can run independently after #80. They are all mandatory ancestors
of pool expansion by the explicit structural request. Further edges express actual
contracts: tokens before the modal token mode, flying/reach and trample before
combined deathtouch, ordering before real cast/ETB detection, and real Thrill/ETB
before repeat-cleanup composition. #208 joins mechanics to extend native policies.
#209–#211 use only their mechanic prerequisites; they need not wait for #23's audit.
#212 and mutations require full-pool policy/semantic input. Metrics begin from M1.
Redundant feature edges are removed; the explicit #80 edge on every child is the
requested dispatch guard, not an accidental serialization chain.

## Deliverables and direct prerequisites

| Issue | Deliverable | Direct prerequisites | Full acceptance owner |
| --- | --- | --- | --- |
| #131 | Refactor cast, target and payment state boundaries | #80 | #23 |
| #132 | Centralize supported card characteristics and costs | #80 | #23 |
| #133 | Replace JSON round-trip recorder conversion with typed conversion | #80 | #23 |
| #194 | Dragon Fodder and token lifecycle | #80, #131, #132, #133 | #23 |
| #195 | Llanowar Elves and Druid mana abilities | #80, #131, #132, #133 | #23 |
| #196 | Flying and reach combat legality | #80, #131, #132, #133 | #23 |
| #197 | Axgard Cavalry haste activation | #80, #131, #132, #133 | #23 |
| #198 | Tajuru vigilance and trample allocation | #80, #131, #132, #133 | #23 |
| #199 | Thornweald deathtouch damage | #80, #196, #198 | #23 |
| #200 | Shivan Dragon repeated power activation | #80, #196 | #23 |
| #201 | Wildheart Invoker targeted boost | #80, #198 | #23 |
| #202 | Thrill discard cost and ordered draw | #80, #131, #132, #133 | #23 |
| #203 | Goblin Surprise choose-one modes | #80, #194 | #23 |
| #204 | Pending trigger ordering and resumable placement | #80, #131, #132, #133 | #23 |
| #205 | Firebrand Archer and Cyclops cast triggers | #80, #204 | #23 |
| #206 | Viashino Pyromancer targeted ETB trigger | #80, #204 | #23 |
| #207 | Trigger and discard cleanup continuation | #80, #206, #202 | #23 |
| #208 | Native policies for every full-pool decision | #80, #195, #197, #199, #200, #201, #203, #205, #207 | #23 |
| #209 | XMage costs, tokens and modal coverage | #80, #195, #197, #203, #202, #205 | #24 |
| #210 | XMage keyword and activated-ability coverage | #80, #199, #200, #201, #197, #203 | #24 |
| #211 | XMage cast, ETB and cleanup coverage | #80, #205, #207, #194 | #24 |
| #212 | Full-pool legacy and decision coverage | #80, #208 | #24 |
| #213 | M2 named semantic mutation checks | #80, #208 | #24 |
| #214 | Scalar counters and bounded public metrics | #80 | #25 |
| #215 | Bounded sampled diagnostics and complete replay mode | #80, #214 | #25 |
| #216 | Versioned scalar benchmark workload and accounting | #80, #214 | #25 |
| #217 | Reproducible full-pool scalar performance and memory artifact | #80, #208, #216, #215 | #25 |
| #23 | Complete pool integration/evidence audit | #208 | #26 |
| #24 | Scenario/reference/mutation integration audit | #23, #209, #210, #211, #212, #213 | #26 |
| #25 | Instrumentation/baseline integration audit | #217 | #26 |
| #26 | Independent M2 exit gate | #23, #24, #25 (unchanged) | Program #7 |

## Test and delivery contract for every child

One independently testable reviewed PR per issue. New behavior requires a compiled
behavioral failure before implementation, independent Oracle/CR or interface-contract
expectations, positive/negative/interaction/regression tests in ordinary discovery,
retained minimized input/seed, full `./scripts/torture.sh` after fresh-main integration,
applicable real pinned reference checks, prescribed separate Codex review, protected
merge and exact-main CI. Refactors retain every existing expected outcome; comparisons
must account for the existing conservative source-fingerprint compatibility policy.
No test deletion, weakening, skipped advertised behavior, synthetic sibling, runtime
reference-engine delegation or human approval step is introduced.

Each new mechanic delivers its scalar/work path AND its existing policy Choice,
semantic action, pending snapshot/replay and typed trajectory representation. Verify
normal-reset played scripts with literal checkpoints, quantum 1/unbounded equality,
capture on/off equality and invalid-action state/RNG/history nonmutation. Synthetic
keyword/trigger states are declared test-only and do not advertise an incomplete card.
The existing recorder/API contracts are extended only where the mechanic needs them;
no new recorder or alternative rules loop. Original component audits verify composed
acceptance and report a concrete split for any broad missing implementation.

The mechanic owner must implement and execute the unchanged catalog cases that use
only its delivered prerequisites, including the applicable native/XMage translation;
it cannot leave basic card/reference execution to #209–#212. The exact crosswalk's
execution pack is the later composition/re-execution owner, while the unchanged
catalog owner remains the full acceptance owner. Thus reference packs reuse actual
mechanic receipts and strict bridge operations, add only bounded cross-mechanic
scripts/one independent holdout each, and publish the joined receipt. They are not
56/50/38/31-case implementations from scratch. If a prerequisite omitted a required
bridge operation, the pack records the missing contract and requests a coordinator
split instead of silently absorbing a new engine bridge or rules implementation.

The tests below are required future executable assertions, not claims they already
exist or passed. Use `cargo test --workspace --locked` for Rust normal discovery,
`python3 scripts/run_tests.py` for Python normal discovery, and full torture for both
profiles. Child reports must supply exact test names, red/green receipts and actual
reference commands/pins. Expectations must never be regenerated from the engine.

### #131 — Refactor cast, target and payment state boundaries

Narrow internal methods around the existing PendingCast/Payment state; preserve public APIs and all legal choices, cancellation and errors.

- Positive assertions: Run existing casting/targets/payment tests unchanged; snapshot a pending target and each payment stage, restore and finish/cancel to the same independently specified hand, mana, taps and stack. Compare normal-reset semantic history and capture before/after extraction.
- Negative assertions: Stale/wrong-seat payment, duplicate source and illegal target leave state/RNG/history unchanged; cancelling restores provisional choices without creating priority or leaking private choices.
- Independent oracle: Existing M1 compiled red/green and handwritten rejection fixtures; CR 601.2 and the established compatibility contract, not newly generated snapshots.
- Exclusions: No new cards, schema redesign, payment rules or general framework.
- Partial requirement owners: R0002-B011, R0002-B016, R0002-B029. All prior owners remain.

### #132 — Centralize supported card characteristics and costs

One typed source for currently supported characteristics, costs and behavior tags used by casting, targets and combat; preserve frozen identities/hashes.

- Positive assertions: Table-driven assertions for all six supported cards against the pinned card manifest; current Growth/Bite/combat results and public paths remain exact.
- Negative assertions: Other frozen identities remain unsupported for play; unknown/corrupt identities and source hashes reject explicitly. No newly enabled card via default characteristics.
- Independent oracle: data/cards/foundations_micro_v1.json and its frozen content hashes; independently enumerated six-card support list from M1 acceptance.
- Exclusions: No pool expansion, source repinning or card-text interpreter.
- Partial requirement owners: R0002-B010, R0002-B011, R0002-B029. All prior owners remain.

### #133 — Replace JSON round-trip recorder conversion with typed conversion

Explicit core-to-wire typed conversions at existing recorder boundaries, preserving canonical bytes and validation ownership.

- Positive assertions: Existing handwritten v1/v2 fixtures, real played capture and optional statistics preserve every field and canonical byte; float bit roundtrips and absent values remain exact.
- Negative assertions: Malformed records and interrupted/failed capture retain the same validation errors; no private field crosses the player boundary and no fabricated policy statistics appear.
- Independent oracle: Existing handwritten structured_contract/jsonl fixtures, v2 wire contract and original privacy/fault-injection tests. Do not regenerate golden output.
- Exclusions: No schema/version changes, new storage format, game changes or weakening tests.
- Partial requirement owners: R0002-B036, R0002-B037. All prior owners remain.

### #194 — Dragon Fodder and token lifecycle

Sorcery token creation through the authoritative cast/resolution path and generation-safe Goblin lifecycle.

- Positive assertions: Cast Fodder for 1R: before resolution zero tokens, afterward two distinct untapped red 1/1 Goblins; they can block immediately, cannot attack that turn, and a dead token ceases after SBAs. Repeat resolution yields four unique handles.
- Negative assertions: Off-turn/nonempty-stack sorcery cast, unpaid cast and exhausted object capacity reject or fail explicitly without partial token creation; old dead token targets cannot resolve onto another token.
- Independent oracle: Pinned Fodder/token definitions; CR 111, 302.6, 601, 704; literal object/life counts.
- Exclusions: No modal spell, cast triggers or haste; those composed cases remain later reference-pack acceptance.
- Partial requirement owners: R0002-B010, R0002-B011, R0002-B029. All prior owners remain.

### #195 — Llanowar Elves and Druid mana abilities

Creature tap-for-green mana sources in both priority and staged casting payments, including printed 1/1 and 1/3 bodies.

- Positive assertions: Old untapped Elf/Druid taps and adds exactly G immediately without a stack entry; Elf plus Forest pays Cub 1G with GG; Druid blocks Cub and survives with two marked damage.
- Negative assertions: New sick, already tapped, enemy or stale sources cannot pay; cancelled staged payment leaves source taps and pool unchanged; mana activation opens no response window.
- Independent oracle: Pinned Elf/Druid definitions; CR 302.6, 605 and 601; existing independent payment enumeration.
- Exclusions: No haste grant or nonmana activated ability; Cavalry composition is audited after both land.
- Partial requirement owners: R0002-B010, R0002-B011, R0002-B029. All prior owners remain.

### #196 — Flying and reach combat legality

General flying/reach blocker legality and Magnigoth Sentry support; use explicitly synthetic flying objects until Shivan is fully enabled.

- Positive assertions: Synthetic 5/5 flyer can be blocked by 4/4 Sentry with reach: Sentry dies, flyer has four damage; attacking Sentry remains blockable by ordinary Cub. Real Sentry casts for pinned cost and stats.
- Negative assertions: Cub cannot block flying and tapped Sentry cannot block; raw scalar submission rejects the same illegal pair with no mutation.
- Independent oracle: Pinned Sentry definition; CR 509, 702.9, 702.17; declare synthetic keyword fixtures, not full Shivan support.
- Exclusions: No Shivan activation, deathtouch, trample or inferred full-card support from synthetic fixtures.
- Partial requirement owners: R0002-B010, R0002-B011, R0002-B029. All prior owners remain.

### #197 — Axgard Cavalry haste activation

Tap-cost targeted activated ability with temporary haste and source-independent stack resolution; enable Cavalry.

- Positive assertions: Old Cavalry taps to give a fresh Cub haste; Cub can attack; opposing Cub is also a legal target. Kill source after activation: ability still resolves. Cleanup removes granted haste.
- Negative assertions: Fresh sick or tapped Cavalry cannot activate; stale/departed target produces no haste on a replacement object; malformed target/cost leaves state unchanged.
- Independent oracle: Pinned Cavalry definition; CR 602, 302.6, 702.10, 113.7a, 611.2; hand-authored source/target ledgers.
- Exclusions: No creature-mana implementation or general trigger framework; Elf interaction belongs to composed coverage.
- Partial requirement owners: R0002-B010, R0002-B011, R0002-B029. All prior owners remain.

### #198 — Tajuru vigilance and trample allocation

Vigilance and trample in current-rules factored combat allocation; enable Tajuru Pathwarden.

- Positive assertions: 5/4 Tajuru attacks untapped; two 2/2 Cubs require two damage each before one to defender; both Cubs and Tajuru die. Departed blockers allow all five to defender.
- Negative assertions: One to an undamaged Cub and four to player rejects; all nonnegative blocker-only splits remain expressible where current rules permit, without obsolete blocker order.
- Independent oracle: Pinned Tajuru definition; CR 510, 702.19, 702.20 and Foundations damage-assignment update.
- Exclusions: No deathtouch or flying implementation, no ordered-blocker response window.
- Partial requirement owners: R0002-B010, R0002-B011, R0002-B029. All prior owners remain.

### #199 — Thornweald deathtouch damage

Deathtouch for both combat and Bite damage, preserving simultaneous damage and trample lethal allocation; enable Thornweald Archer.

- Positive assertions: Thornweald blocks a synthetic 5/5 flyer and both die; its Bite kills Sentry without return damage. Synthetic trample/deathtouch 7/6 assigns one each to two Sentries and five to player, then dies to eight.
- Negative assertions: Zero damage is not lethal; deathtouch without trample cannot assign defender damage through a blocker; invalid allocations leave state unchanged.
- Independent oracle: Pinned Thornweald; CR 702.2, 702.19, 510, 704.5h; independently counted damage.
- Exclusions: No Invoker activation or full Shivan implementation; synthetic combinations remain explicitly labeled.
- Partial requirement owners: R0002-B010, R0002-B011, R0002-B029. All prior owners remain.

### #200 — Shivan Dragon repeated power activation

Nontargeted R activation through explicit payment and stack, additive +1/+0 until cleanup; enable Shivan.

- Positive assertions: New Dragon may activate despite sickness; two R paid and two resolutions make 7/5, retain flying, then cleanup restores 5/5. Bite uses current boosted power.
- Negative assertions: No R, stale source or supplied foreign target rejects atomically; an already stacked ability is not cancelled by source death and cannot boost a new incarnation.
- Independent oracle: Pinned Shivan; CR 602, 113.7a, 611.2, 613; literal base plus one per resolved activation.
- Exclusions: No generalized layer engine, Invoker or trigger implementation.
- Partial requirement owners: R0002-B010, R0002-B011, R0002-B029. All prior owners remain.

### #201 — Wildheart Invoker targeted boost

Eight-generic targeted activation granting +5/+5 and trample until end of turn; enable Wildheart Invoker.

- Positive assertions: New Invoker may activate without tap/sickness restriction; own or opposing Cub becomes 7/7 with trample; source death does not cancel ability and cleanup restores printed stats.
- Negative assertions: Seven mana, missing/illegal target and stale payment reject without partial spending; departed target causes no boost on its new incarnation.
- Independent oracle: Pinned Invoker; CR 602, 608.2b, 611.2, 702.19; literal cost/stat assertions.
- Exclusions: No deathtouch implementation or arbitrary permanent abilities; Thornweald composition later.
- Partial requirement owners: R0002-B010, R0002-B011, R0002-B029. All prior owners remain.

### #202 — Thrill discard cost and ordered draw

Additional discard-cost continuation committed atomically with payment, then two ordered draws on resolution; enable Thrill.

- Positive assertions: Discard another hand card into graveyard before responses; draw exactly the two known top cards after resolution. One-card library draws one then loses at SBA after failed second draw.
- Negative assertions: Thrill alone, bad discard cardinality, stale/wrong-seat choice or unpaid mana cannot cast and leave all state unchanged; pending snapshot/cancellation keeps provisional discard private.
- Independent oracle: Pinned Thrill; CR 601.2h, 608, 121, 704.5b; explicit ordered deck and hand ledger.
- Exclusions: No cast-trigger implementation or cleanup exception; do not refund a paid cost when effect fails.
- Partial requirement owners: R0002-B010, R0002-B011, R0002-B029. All prior owners remain.

### #203 — Goblin Surprise choose-one modes

Explicit choose-one modal casting with either two Goblins or +2/+0 to creatures controlled at resolution until cleanup.

- Positive assertions: Boost Cub and existing token to 4/2 and 3/1; tokens created afterward stay 1/1. Token mode creates two blockers at instant speed without boost.
- Negative assertions: Both/zero/unknown modes reject without spending; enumerate both modes across explicit payments and cancel without leaking provisional choices.
- Independent oracle: Pinned Surprise; CR 700.2, 601.2b, 611.2c; exact chosen-mode and affected-set ledger.
- Exclusions: No trigger framework or new token rules beyond the delivered token contract.
- Partial requirement owners: R0002-B010, R0002-B011, R0002-B029. All prior owners remain.

### #204 — Pending trigger ordering and resumable placement

Typed pending-trigger queue and controller/APNAP ordering integrated with owned work, priority and snapshot continuation; test-only injection of declared synthetic triggers.

- Positive assertions: Two pending triggers per seat: active seat chooses its order below nonactive seat order; dead-source pending abilities remain. Quantum 1 versus unbounded and pending snapshot restore produce the same order and checkpoints.
- Negative assertions: Cross-controller/APNAP-invalid order, duplicates, omissions, stale/wrong-seat submission and capacity exhaustion reject/fail explicitly; no priority is granted before mandatory SBA/placement completes.
- Independent oracle: CR 603.3, 101.4, 117.5; literal bottom-to-top synthetic stack, independent of production trigger detection.
- Exclusions: No new card trigger detection, made-up production trigger or claim synthetic placement verifies real card casts.
- Partial requirement owners: R0002-B011, R0002-B016, R0002-B026, R0002-B029. All prior owners remain.

### #205 — Firebrand Archer and Cyclops cast triggers

Detect committed noncreature casts and enqueue source-specific Archer damage/Cyclops boosts; enable both creatures.

- Positive assertions: Cast existing Growth: Archer deals one to opponent before spell, Cyclops gains +3/+0 before Growth makes it 6/7; two Archers allow controller order and deal two total. Source death does not delete a stacked Archer trigger.
- Negative assertions: Creature cast, land play, mana activation, rejected or cancelled noncreature cast creates no trigger; neither opponent damage nor Cyclops bonus occurs before trigger resolution.
- Independent oracle: Pinned Archer/Cyclops; CR 603, 601.2i, 113.7a; literal timing/life/stat ledger.
- Exclusions: No ETB trigger detection or token/Thrill interactions until composed coverage.
- Partial requirement owners: R0002-B010, R0002-B011, R0002-B029. All prior owners remain.

### #206 — Viashino Pyromancer targeted ETB trigger

ETB detection and required player target selection through the pending-trigger contract; enable Pyromancer.

- Positive assertions: Resolving Pyromancer puts 2/1 on battlefield then targeted ability on stack; either player is legal, target loses two on resolution; self-target at two life loses. Source death does not cancel ability.
- Negative assertions: Creature target, absent target and stale/wrong-seat choice reject without mutation; merely casting Pyromancer does not cause ETB damage.
- Independent oracle: Pinned Pyromancer; CR 603.2/603.3d, 115, 113.7a, 704; no planeswalker/battle candidates exist in scope.
- Exclusions: No cast-trigger implementation or new permanent types.
- Partial requirement owners: R0002-B010, R0002-B011, R0002-B029. All prior owners remain.

### #207 — Trigger and discard cleanup continuation

Complete repeat-cleanup processing with pending triggers and private discard continuations using existing turn/work owner.

- Positive assertions: Synthetic cleanup entry with eight-card active hand and pending Pyromancer trigger discards one, removes damage/bonuses, places/resolves trigger, then repeats cleanup; Thrill during exceptional cleanup priority returns hand to seven without redundant discard.
- Negative assertions: Wrong discard count/seat rejects atomically; ordinary cleanup grants no priority, opponent oversized hand is untouched, terminal outcome stops subsequent turn work.
- Independent oracle: CR 514 and 704; unchanged catalog hand-size-cleanup interaction/regression and explicitly synthetic trigger setup.
- Exclusions: No new trigger source, ordinary-turn redesign or arbitrary cleanup state injection in public APIs.
- Partial requirement owners: R0002-B010, R0002-B011, R0002-B029. All prior owners remain.

### #208 — Native policies for every full-pool decision

Extend the existing LegalRandom/Heuristic and supported-pool validation to all delivered decisions, through Driver and existing CLI; no alternative game loop.

- Positive assertions: Both policies consume every mode/target/discard/ability/trigger/combat continuation; fixed seeds in red/green and both mirrors with both starting seats produce reproducible actual games, semantic replay and captured trajectory roundtrips.
- Negative assertions: Unknown content, missing policy choice and illegal/stale actions fail without a pass/default fallback; hidden-hand/library swaps preserve permitted inputs and choice behavior; explicit limits remain truncations.
- Independent oracle: Frozen decks plus existing policy RNG/version/privacy contract; hand-authored short scripts with literal checkpoints supplement reproducibility, which is not a rules oracle.
- Exclusions: No policy-strength claim, training integration, new collector, benchmark qualification or monolithic rules fixes.
- Partial requirement owners: R0002-B010, R0002-B011, R0002-B020, R0002-B029, R0002-B039. All prior owners remain.

### #209 — XMage costs, tokens and modal coverage

Compose and rerun delivered native/XMage cost, mana, modal and token fixtures; add bounded interaction scripts and publish the exact assigned-case receipt. Basic bridge operations belong to the mechanic prerequisites.

- Positive assertions: Actual pinned XMage/native runs for Elf, Druid, Cavalry, Fodder, Surprise, Thrill and Goblin token; include haste-mana, Archer-before-Fodder/Thrill, fixed boost recipient set, ordered empty draw and token identity.
- Negative assertions: Illegal sickness/tap/mode/discard/timing choices reject; mutate one life/zone/mana/choice/checkpoint field and comparator must identify first divergence; missing build or unsupported script is failure, never agreement.
- Independent oracle: Unchanged catalog bases, pinned Oracle/CR and original neutral scripts; independent expected checkpoints, not either engine output.
- Exclusions: No rules implementation, Forge expansion or whole-pool verification claim.
- Partial requirement owners: R0002-B010, R0002-B011, R0002-B026, R0002-B028, R0002-B030. All prior owners remain.

### #210 — XMage keyword and activated-ability coverage

Compose and rerun delivered native/XMage keyword and activation fixtures; add bounded cross-mechanic combat scripts and publish the exact assigned-case receipt.

- Positive assertions: Execute Sentry, Tajuru, Thornweald, Shivan and Invoker cases including dead sources, boosted Bite and current-rules unrestricted blocker splits; compare intermediate damage, mana, stack and cleanup.
- Negative assertions: Reject flying-illegal blockers, insufficient trample lethal and wrong targets/payments; mutated damage split or retained temporary keyword must fail a named checkpoint.
- Independent oracle: Pinned card/CR catalog expectations including Foundations removal of damage assignment order; preserve upstream discrepancies with independent adjudication.
- Exclusions: No engine mechanics, invented legacy blocker order, Forge expansion or release waiver.
- Partial requirement owners: R0002-B010, R0002-B011, R0002-B026, R0002-B028, R0002-B030. All prior owners remain.

### #211 — XMage cast, ETB and cleanup coverage

Compose and rerun delivered native/XMage trigger and cleanup fixtures, add bounded ordering/cleanup interaction scripts and publish the exact assigned-case receipt.

- Positive assertions: Archer/Cyclops/Pyromancer per-card cases, two-source controller ordering, APNAP synthetic cases, Archer lethal before Fodder, Cyclops power read by Bite and cleanup-trigger repeat; every synthetic setup is labeled.
- Negative assertions: Failed cast creates no trigger; creature ETB target invalid; corrupt ordering or skipped cleanup checkpoint is detected with minimized artifact.
- Independent oracle: CR 603/101.4/514 and unchanged catalog; literal expected stack/life/stat order and independently authored cases.
- Exclusions: No new rules, Forge expansion or fabricated naturally reachable APNAP setup for synthetic cases.
- Partial requirement owners: R0002-B010, R0002-B011, R0002-B026, R0002-B028, R0002-B030. All prior owners remain.

### #212 — Full-pool legacy and decision coverage

Complete M2 native decision/privacy/replay/capacity cases and refresh XMage per-capability/legacy-card crosswalk; retain exact earlier M1 receipts where unchanged.

- Positive assertions: Enumerate six legal five-power/two-blocker splits and 64 six-token attacker subsets independently; pending Thrill snapshot reproduces private choice; execute all remaining M2 catalog assertions plus Forest/Mountain/Cub/Swab/Growth/Bite reference coverage.
- Negative assertions: Above declared capacity errors explicitly without clipped candidate; hidden sentinels never leak; stale/raw masked choices reject; corrupt fixture checkpoint fails comparison.
- Independent oracle: Independent combinatorics, unchanged M1/M2 catalog and CR; deterministic equivalence supplemented with literal expectations.
- Exclusions: No giant implementation repair; bounded defects use ordinary delivery rules and larger gaps require coordinator replanning. No batch/Python tests moved into M2.
- Partial requirement owners: R0002-B010, R0002-B011, R0002-B026, R0002-B028, R0002-B030. All prior owners remain.

### #213 — M2 named semantic mutation checks

Executable designated M2 mutation matrix with one named failing behavioral assertion per mutant, minimized inputs and ordinary test discovery.

- Positive assertions: Unmodified suite passes; individually detect omitted priority, stacked mana ability, allowed sick tap, missed cast trigger, retained boost, late cleanup damage removal, stale acceptance, private leak, reversed reward, truncation-as-draw, clipped choices and omitted failed denominator.
- Negative assertions: A surviving mutant, missing assertion, mutant build/import failure or mismatched scenario is a failing mutation run; never count a compiler error as behavioral detection.
- Independent oracle: RFC B029/SYS-MUT-001 and rule-derived existing assertions; tests independent of mutation switch, no generated expected output.
- Exclusions: No CI/enforcement changes, mutation of later batch/RL/IPC capabilities or weakened production tests; later #37 retains full suite.
- Partial requirement owners: R0002-B026, R0002-B029, R0002-B030. All prior owners remain.

### #214 — Scalar counters and bounded public metrics

Off/counters instrumentation at existing Driver/native boundaries with local aggregation and bounded privacy-safe metric names.

- Positive assertions: Hand-counted successful, truncated, failed, reset and invalid/stale paths yield exact counters by decision/logical action; instrumentation leaves state/RNG/replay and separately requested capture unchanged.
- Negative assertions: Hidden card/seed/game/error sentinels absent from labels and public metrics; overflow/error paths counted, never successful completion; off avoids optional materialization and rules never read performance clock.
- Independent oracle: RFC B008/B021 and SYS-METRIC-001/002; literal tiny traces and injected boundary clock, not elapsed-time goldens.
- Exclusions: No worker/batch/inference implementation: not-applicable metrics explicitly distinguished from measured zero; #27/#29/#39 retain future timing obligations.
- Partial requirement owners: R0002-B008, R0002-B020, R0002-B021. All prior owners remain.

### #215 — Bounded sampled diagnostics and complete replay mode

sampled_trace/full_replay modes using existing replay/capture ownership; explicit bounded diagnostic drop versus complete-replay failure semantics.

- Positive assertions: Same seeded scripts in four modes and independent capture on/off preserve decisions/outcomes; fixed sampling selection repeats, bounded trace saturation increments exact drop count.
- Negative assertions: Promised replay overflow/write failure marks incomplete or fails recording, never silently drops; private replay data cannot appear in public diagnostics; requested trajectories persist with instrumentation off.
- Independent oracle: RFC B021 and existing replay/recorder completeness contract; forced bounded buffers and literal counts.
- Exclusions: No new recorder format, storage backend, general logging platform or game semantic change.
- Partial requirement owners: R0002-B020, R0002-B021. All prior owners remain.

### #216 — Versioned scalar benchmark workload and accounting

Extend existing bench command with frozen scalar workload/version metadata, repeated windows and truthful separate transition/legality/rollout/encoding/policy timing.

- Positive assertions: Controlled clock tests count all elapsed time including reset/failures; only rules-terminal games enter completed rate. Store raw windows/distributions, hardware/toolchain/affinity/seeds/deck/action/policy/instrumentation pins.
- Negative assertions: Reject malformed/nonpositive windows, incompatible workload/version and missing policy; zero completions cannot imply successful throughput and unfinished episodes remain explicit.
- Independent oracle: RFC B008/B019/B020 and SYS-PERF-001/002; independent arithmetic over known counts/durations, existing real six-card workload for contract tests.
- Exclusions: No speed qualification, new policies, fabricated Python/inference/batch timings or reduced production warmup/window minimum; full-pool measurement comes later.
- Partial requirement owners: R0002-B008, R0002-B014, R0002-B019, R0002-B020, R0002-B021. All prior owners remain.

### #217 — Reproducible full-pool scalar performance and memory artifact

Publish correctness-gated scalar baseline using delivered workload and modes, allocation/boundary/dominant-cost profiles and resident-state memory curve.

- Positive assertions: Run 10s warmup and at least five 30s windows for frozen heuristic and legal-random full-pool workload; raw distributions, actual outcome counts, off/counters overhead, total RSS and marginal state memory through declared resident sweep, typical/token-heavy/target/stack stress.
- Negative assertions: Report misses of provisional targets, failures, overflow, unavailable hardware/perf facilities and unsupported tracks explicitly; contaminated/omitted windows fail artifact validation. No best-run selection or shorter horizon to improve rates.
- Independent oracle: Independent OS/process memory/allocation/profile tools plus benchmark arithmetic and correctness receipts; RFC SYS-PERF-001/002/003, SYS-METRIC-002.
- Exclusions: No optimization project, language rewrite, M3 batch/Python/inference or M5 designated-host qualification. Available-container baseline must name hardware limits; required unavailable access remains a blocker for any claimed qualification.
- Partial requirement owners: R0002-B008, R0002-B014, R0002-B019, R0002-B020, R0002-B021. All prior owners remain.

## Original integration ownership and complete crosswalk

#23 audits EVERY original §3 clause and all 20 cards plus Goblin token, including
costs/modes, keywords/multiple blockers, mana/activated/trigger timing, APNAP, targets,
identity/dead sources, continuous expiry, terminal/concession/empty draw and repeat
cleanup. Its child receipts must account for real scalar/policy/action/recorder/replay
paths, including private pending choices. A passing child is not full component
acceptance. The frozen decks and source pins remain unchanged. #23 may consume later
reference pack receipts when available; #24/#26 cannot pass without every pack.

#24 retains ≥100 individually reviewed semantic scenarios spanning all twelve RFC
families, every advertised case without skip/xfail, XMage per capability and all twenty
cards, negative commands and intermediate checkpoints, provenance and independently
authored holdouts. Count distinct reviewed scenarios, not parameter invocations or
catalog designs. #209–#212 must preserve original M1 executed receipts for unchanged
cases and add original independently authored interaction cases, with review/provenance
per scenario. Their delivery reports list planned/executed/agreed/disputed/unsupported
by capability and engine, first divergence and minimized reproductions. Native unit
success alone is not XMage execution. All native/M2 catalog cases below are executed
by their assigned pack; the aggregate `owner_issue` stays #23/#24 in the unchanged
catalog. Pack assignment is test execution work, not permission to implement missing
mechanics in a coverage task. #24 checks the union, the twelve-family distribution,
reference breadth, independent holdouts and #213's named mutant detections. The later
≥20 dual-reference critical suite/full-game release obligations remain #38/#40; no
M2 child waives or counts unavailable reference execution as agreement.

#25 retains full §6 instrumentation acceptance and scalar baseline/profile obligations.
SYS-METRIC-001: #214 counter paths and #215 modes/buffers/capture independence;
SYS-METRIC-002: #214 privacy/cardinality and #217 measured overhead. SYS-PERF-001/002:
#216 frozen workload/accounting and #217 actual scalar measurements/distributions;
SYS-PERF-003: #217 capacity/memory/stress. #25 audits their complete M2 composition.
The full six-track/worker/Python/inference/CLI and designated-host qualifications
remain original later owners #27–#39/#40; absence is reported, never fabricated zero
or completed M2 evidence for a later path. SYS-PERF-004 remains #28/#39 capture
measurement. SYS-MUT-001: #213 owns the applicable M2 semantic/denominator mutants;
#37 retains batch/RL/IPC and full qualification. Existing SYS IDs and expectations
are unchanged. A local scalar baseline may report target misses with evidence;
it cannot claim designated-host qualification without required access.

#26 independently verifies all M2 requirements/exit criteria on exact main, including
the complete pool, ≥100 reviewed scenarios, named mutants, every card/capability
XMage evidence and reproducible scalar performance/memory artifact. This amendment
delivers none of those exit criteria. Shared RFC blocks and original owners remain.

[Exact per-case execution crosswalk](m2-test-crosswalk.md) enumerates every M2 catalog
ID and its original aggregate owner, execution pack and unchanged oracle basis.
No M1/M3/M4/M5 case or catalog field changes. Remaining cross-feature interactions
are assigned to #212 after full-pool support, not assumed tested by synthetic unit cases.

## Native links and driver-facing handoff

New feature/reference/metric issues are native sub-issues of #23/#24/#25 respectively.
Move existing #131–#133 from planning #80 to component #23, retaining their original
bodies/history and the explicit #80 blocked-by link. Native blocked-by sets reconcile
to the table; #80 still depends on #22 and #26 on #23/#24/#25. Register all children
without ready labels. Component dependency replacement is applied only after reviewed
registration is merged and exact-main CI passes, preserving the old graph until then.
No old acceptance issue is closed or recreated. A native edge is not completion.

After protected merge/exact-main success, fetch the manifest afresh, recheck parent,
issue states/labels/events and prerequisite completion receipts, and hand off at most
one eligible child (manifest order starts with #131). Metrics remain independently
eligible if a structural task later blocks. Record actual live queue status in the
single #7 Program workpad, not this durable plan. No routine human plan/PR approval.

README assessment: this registration changes only future ownership/dependencies,
not usable commands, setup, supported engine behavior, delivered architecture or the
verified M1 verdict. No root README change is needed. The independently reviewed
README must keep its six-card limitation and M2 pending status; this plan is linked
from the atomic delivery guide. All new support claims belong to future deliveries.
