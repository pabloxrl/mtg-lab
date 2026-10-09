# Atomic MVP delivery

Operations #61 replaces broad implementation dispatch with one tested deliverable per PR. This changes ownership and sequencing, not the pinned RFC, test expectations or milestone acceptance. Existing #17–#21 remain integration/evidence deliveries and retain all original requirements. They implement only integration gaps after their children, not entire components. M1 is not complete. #17's failed attempt remains recorded. After this amendment
is delivered, the coordinator clears its obsolete block and records an explicit
`agent-resume-authorized` grant; handoff consumes it only when its new children
are complete. It is not dispatched ahead of them.

## Integration prerequisite correction

The original table below records the first component deliveries. The [M1 integration prerequisite plan](m1-integration-prerequisites.md) adds the missing played-game contracts found by #18–#21. Its issue-level scopes and the current manifest supersede the earlier assumption that those integration gaps were small. Original acceptance and catalog ownership remain intact; #18–#21 audit the completed prerequisites.

The [lossless trajectory prerequisite plan](trajectory-prerequisites.md) further
splits the contracts exposed by #117 into #152–#154. #117 retains aggregate
acceptance, and #20/#120/#21/#22 retain their integration and gate obligations.

Operations #158 explicitly supersedes the oversized C (#154) implementation
boundary in that historical plan. The [collector prerequisite plan and complete
acceptance crosswalk](collector-prerequisites.md) register exactly #159–#164:
owned execution, canonical capture, budgets, replay access, persistence and
publication. #154 becomes the final collector integration audit after #164;
#117 repeats its original aggregate acceptance afterward. Neither the driver
nor another prerequisite inherits the full collector obligation. All original
owners, catalog expectations and later/reference gates remain unchanged.

## M1 work packages

| Issue | Deliverable | Direct prerequisites | Integration owner |
| --- | --- | --- | --- |
| [#62](https://github.com/pabloxrl/mtg-lab/issues/62) | Versioned episode RNG | #61 | #17 |
| [#63](https://github.com/pabloxrl/mtg-lab/issues/63) | Generation-safe object storage | #61 | #17 |
| [#64](https://github.com/pabloxrl/mtg-lab/issues/64) | Transactional reset and opening hands | #62, #63 | #17 |
| [#65](https://github.com/pabloxrl/mtg-lab/issues/65) | London mulligans and validated choices | #64 | #17 |
| [#66](https://github.com/pabloxrl/mtg-lab/issues/66) | Resumable core work quantum | #65 | #17 |
| [#67](https://github.com/pabloxrl/mtg-lab/issues/67) | Turn steps and priority passing | #65 | #18 |
| [#68](https://github.com/pabloxrl/mtg-lab/issues/68) | Land plays and mana payments | #67 | #18 |
| [#69](https://github.com/pabloxrl/mtg-lab/issues/69) | Creature casting and stack resolution | #68 | #18 |
| [#70](https://github.com/pabloxrl/mtg-lab/issues/70) | Targeted Growth and Bite effects | #69 | #18 |
| [#71](https://github.com/pabloxrl/mtg-lab/issues/71) | Vanilla combat and simultaneous damage | #69 | #18 |
| [#72](https://github.com/pabloxrl/mtg-lab/issues/72) | Rules terminal outcomes and reset integration | #71, #70 | #18 |
| [#73](https://github.com/pabloxrl/mtg-lab/issues/73) | Seat-filtered core observations | #65 | #19 |
| [#74](https://github.com/pabloxrl/mtg-lab/issues/74) | Versioned snapshots and atomic restore | #66 | #19 |
| [#75](https://github.com/pabloxrl/mtg-lab/issues/75) | Semantic replay verification | #74 | #19 |
| [#76](https://github.com/pabloxrl/mtg-lab/issues/76) | Canonical in-memory trajectories | #72, #73 | #20 |
| [#77](https://github.com/pabloxrl/mtg-lab/issues/77) | JSONL trajectory writer and validator | #76 | #20 |
| [#78](https://github.com/pabloxrl/mtg-lab/issues/78) | Unattended scalar simulation command | #72, #73 | #21 |
| [#79](https://github.com/pabloxrl/mtg-lab/issues/79) | Headless replay and conformance commands | #75, #77, #78 | #21 |

RNG and object storage can start independently. Observations, snapshots and turns branch from established opening contracts. Vanilla combat does not wait for targeted spells. Integration checks join only the pieces they exercise. Previous milestone gates remain mandatory; review and CI remain required for every delivery. Direct graph edges avoid duplicate transitive prerequisites.

## Exact test ownership

The catalog remains 320 designs, not executable results. Only `owner_issue` changes; IDs, setups, actions, expected results, basis and milestone stay identical. The JSON catalog is the per-case implementation crosswalk. #17 audits core state/RNG/opening/identity/quantum contracts; first-draw/full-turn and terminal clauses originally assigned to #17 are delivered by #67/#72 and audited by #18 and M1 #22. This is an explicit transfer, not a waiver.

SYS-CORE-001: #64/#65 own reset/opening prefixes; #72 and #18 own reset after played games, mana/damage and full scripts. SYS-CORE-002: #62. SYS-CORE-003: #65 real opening choices; #68–#70 real payment/target/priority; #71 combat; #18 audits completeness. SYS-CORE-004: #63 storage operations; #70/#72 card-driven identity. SYS-CORE-005/007: #68–#71 factored actions; #18 integrates capacity and all relevant combinations. SYS-CORE-006: #66 core work; #18 extends to actual effects, #19 snapshots, #27 batch fairness. SYS-CORE-008: #72 rules outcomes; #76/#20 reward ledger. SYS-CORE-009: #64 core config; #21 public CLI. Stable SYS IDs and all original assertions remain required. Later M2 effects/trigger chains and M3 batching remain their original stage obligations.

SYS privacy/replay: #73/#74/#75 deliver core contracts; #19 checks real pending casts/targets, errors and snapshots after #70. SYS/DRL data: #76 in-memory and #77 JSONL; #20 audits the full M1 ledger including terminal nonacting seats. CLI: #78 simulation, #79 command suite, #21 integration. Each component audit must enumerate every original acceptance clause and planned family, link exact executable tests and evidence, and fix only bounded integration defects or report a concrete split. A passing child never implies a whole RFC block passed.

## Later stages

Coordinator operations #80 (M2), #81 (M3), #82 (M4), #83 (M5) are prerequisites of their stage work. Once the previous gate passes, each splits the existing backlog against the interfaces actually delivered. This avoids inventing detailed dependencies before APIs exist. Each is explicitly authorized to register bounded children within its own stage through reviewed changes, update lossless requirement owners and native GitHub links, and queue one eligible child only after exact-main CI. Feature agents cannot expand their own scope. Original component and stage acceptance stays intact.

## Atomic task contract

One task has one observable deliverable, a runnable positive and negative test, an independent expected result, explicit exclusions, and only the contracts/code it needs as prerequisites. Target one reviewable PR sized by independent testability and coherent scope. Elapsed time and dispatch count are not task-sizing or stopping criteria. Keep an independent ready task eligible when another blocks. Do not create a dependency merely because two tasks are in the same component. Do not call a task atomic if its only meaningful test requires an unimplemented sibling. Synthetic state tests are allowed when explicitly declared; they do not substitute for promised real game sequences.

## What the driver does

Describe the desired behavior and constraints in an issue. Agents break it into tested deliveries, implement and retain regressions, independently review, merge after CI and move the queue. You can watch #7 for completed work and blockers; no PR review or routine stage approval is needed. Product ambiguity or missing access is brought back to you. A failed test stops the affected delivery; it is never disabled to progress. Request pause/resume or change priorities through the coordinator. The growing executable baseline and reproducible command are in [torture-suite.md](../testing/torture-suite.md).

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

## M2 after the verified M1 gate

Coordinator operations #252 adds the [bounded M2 repair prerequisites](m2-repair-prerequisites.md)
to #80's registration authority solely for #23/#210/#212's documented gaps.
It may register #254–#257, additive owners and real native/executable dependencies,
then record dependency-gated reactivation after verified registration delivery.
Feature workers remain prohibited from registering tasks or widening scope.
No original case/requirement, stage gate or #241 transition ownership changes.

Operations #80 registers [27 bounded M2 deliveries](m2-atomic-delivery.md), including
the three existing structural prerequisites #131–#133. The [exact case crosswalk](m2-test-crosswalk.md)
preserves all catalog assertions and component acceptance owners. #23/#24/#25 become
bounded full-acceptance audits; #26 remains the M2 gate. Registration is planning,
not feature implementation or an M2 completion claim. The earlier M1-pending prose
above records historical planning; the [M1 gate report](../evidence/m1-gate/README.md)
and root README carry the reviewed delivered verdict. Live queue status stays in #7.
