# Bounded M2 repair registration

Operations #252 registers four repairs after independently inspecting main
`522b7da8b86cd412c03c709bf88d8540c0385e16`. This is planning only: no engine
implementation, executed new reference evidence, audit completion or M2 verdict.
The original requirements, pins, catalog cases/owners and aggregate gates remain.
The existing [M2 plan](m2-atomic-delivery.md) and [crosswalk](m2-test-crosswalk.md)
retain every acceptance clause. Only the additional prerequisites below apply.

## Independent source reconciliation

| Report | Fresh-main evidence | Registration decision |
| --- | --- | --- |
| [#23 compiled failures](https://github.com/pabloxrl/mtg-lab/issues/23#issuecomment-6079276519) | `activation.rs::usable_activation_source` requires floating R/eight mana; `PendingActivation` has no staged mana sources; `policy.rs` exposes only Pay during payment; `mtg-recorder/src/structured.rs::command_matches` excludes TapMana there. | Confirmed full continuation/protocol gap, not a mask-only defect. #254 owns it; preserve the three failing tests and float-first regressions. |
| [#210 inventory](https://github.com/pabloxrl/mtg-lab/issues/210#issuecomment-6076401142) | `FlyingReachTest.java` explicitly injects a synthetic flying Cub. `ShivanTest.java` grown_split has two blockers; its fixture assigns zero to the grown one. Existing Cub payment observations do not assert untapped/sickness; no strict opponent-turn non-flash Cub attempt exists. | #255 adds exactly the missing inputs/checkpoints. Related examples do not certify literal unchanged catalog slots. |
| [#212 inventory](https://github.com/pabloxrl/mtg-lab/issues/212#issuecomment-6078764991) | `InstantResponseTest.java` accepts Cub/Forest/Mountain/Bite/Growth only, rejecting Sentry. Existing deathtouch/Shivan scripts do not express ordinary Cub-to-Sentry damage. | #256 extends the existing instant adapters for six exact slots. |
| Same #212 inventory | `TerminalTest.java` and `terminal_tests.rs::terminal_same_neutral_boundaries_as_xmage` draw only for P0; `fixtures/reference/terminal.json` exports life/lost/library/hand but no explicit outcome. | #257 adds per-seat failed draw and observed outcome in the same SBA batch; separate single-reason cases cannot substitute. |

This is a source reconciliation, not a fresh behavioral/reference execution claim.
The diagnostic patch and inventories remain in the linked workpads and original
GH-23/GH-210/GH-212 workspaces; reuse those drafts on resumption. The #212 count
is **31** assigned cases, correcting its earlier 30-case transcription. No other
false diagnosis was found. Do not read support from a similar synthetic case.

## Contracts and dependencies

| Repair | Observable deliverable | Direct prerequisites | Additional audit edge |
| --- | --- | --- | --- |
| #254 | Payment-stage mana source choices across scalar, policy, semantic action, snapshots and typed recording/replay | #252, #80, #195, #197, #200, #201, #208 | #23 and #217 require #254 |
| #255 | Strict Cub casting/sickness/off-turn rejection and real single-Sentry/Shivan Growth checkpoints | #252, #80, #196, #200 | #210 requires #255 |
| #256 | Strict ordinary Sentry/Bite, power and cleanup inputs/checkpoints | #252, #80, #196, #137, #139 | #212 requires #256 |
| #257 | Per-seat failed draw and explicit mixed simultaneous-loss outcome | #252, #80, #72 | #212 requires #257 |

#252 depends on delivered #80, whose #22 ancestor preserves the verified M1 gate.
Each child explicitly depends on #252's reviewed registration/exact-main receipt
and #80. #195/#197 supply creature mana/haste, #200/#201 activations, #208 existing
native choices and inherited structural/recorder contracts. #196 supplies real
Sentry and #200 real Shivan. #137/#139 inherit the strict #136 instant executor
and add departed identity/cleanup. #72 supplies terminal settlement/reference.
All are existing delivered contracts; links to their completion receipts are in
[the registration evidence](../evidence/m2-repair-registration/README.md).

There are no cross-repair edges: shared source files are not dependencies. #255's
Dragon never activates; #256 uses no deathtouch; #257 uses synthetic SBA boundaries.
#23/#210/#212 gain only the edges listed, retaining all old prerequisites.
#24/#25/#26 remain unchanged; transitive M2 gate ancestry includes #252 and all
four new children as well as every original implementation/operations task.

### Correctness-gated benchmark prerequisite

[Coordinator steering](https://github.com/pabloxrl/mtg-lab/issues/252#issuecomment-6082492423)
also authorizes the real #217 → #254 edge confirmed by the
[benchmark worker's source/contract audit](https://github.com/pabloxrl/mtg-lab/issues/217#issuecomment-6082444182).
#217 promises a **correctness-gated full-pool baseline**. Its measurement cannot
qualify a known incomplete legal payment/choice protocol: repair changes the legal
decision space, seeded policy histories and decision/cost accounting being measured.
The existing #23 compiled-red patch applies unchanged; fresh source inspection
confirms activation/mana/policy-library/typed-recorder bytes match that audit.
Core `policy.rs` only adds `metric_completion`; activation choices/validation are
unchanged. This is preserved compiled evidence plus fresh source inspection,
not a newly executed failure or a claim that current throughput was measured.

Add only #254 to #217's existing #80/#208/#216/#215 prerequisites. Do not require
all of #23 or any independent reference repair. Preserve #217's measurement scope,
drafts and useful diagnostic preparation; it must rerun qualified measurements
against the delivered repair. #216's delivered benchmark contract is not revoked.
After verified registration, record #217's dependency-gated coordinator resume
grant, clearing only a documented activation-gap block if its worker has entered
that state; retain any other hold/withdrawal. Do not make it ready with #254 pending.

## Exact child acceptance

The following issue contracts are copied into the registration so review assesses
the full acceptance together with the executable graph. All ownership is partial
and additive; original aggregate owners remain.

### #254 — mana abilities during activation payment across scalar and replay contracts

Deliver explicit land/creature mana-source choices during Shivan/Invoker activation payment, through the existing scalar continuation, both native policies, semantic actions, snapshots and typed recorder/normal-reset replay. This is a complete payment protocol repair, not a mask-only change.

Prerequisites: #252 reviewed registration, #80, #195 creature mana, #197 haste, #200 Shivan, #201 Invoker, #208 native full-pool choices. #208 transitively includes recorder/structural contracts. No dependency on the three reference repairs.

Evidence: https://github.com/pabloxrl/mtg-lab/issues/23#issuecomment-6079276519 includes the exact three compiled failing regressions and patch hash; reuse them and the preserved GH-23 audit. Fresh registration inspection confirms activation.rs requires floating mana and policy/typed recording omit TapMana during activation_payment.

Acceptance:
- Positive: announce Shivan with empty pool plus untapped Mountain; choose source during payment, pay R and commit one ability; resolution yields 6/5 flying. Announce Invoker with eight untapped Forests, choose its creature target before payment, pay eight and resolve Cub to 7/7 trample. Preserve floating-first paths and repeated Shivan activations.
- Payment semantics: explicit eligible land/Elf/Druid choices; immediate mana ability with no extra stack object/priority window; reserve taps/mana privately, cancel atomically, commit exactly once, retain surplus mana. Revalidate source/target/payment; capacity/mana/decision overflow fails explicitly without partial commit.
- Negative: wrong actor, stale decision/handle, duplicate/tapped/enemy source, wrong color, insufficient mana, source in wrong zone and premature target/payment/finish reject without state/RNG/history mutation. Sick Elf/Druid cannot tap; haste enables it. Sick Shivan/Invoker may activate their nontap ability. No hidden hand/library information changes visible choices, errors or pending observations.
- Interaction/regression: target before mana, target/source departure and incarnation rules, mixed floating and generated mana, multiple sources/colors and cancellation at each pending stage. Independently enumerate bounded source/payment choices and compare quantum 1/unbounded and capture on/off. Snapshot/restore every continuation, explicit semantic encode/decode/apply, both LegalRandom and Heuristic choices, typed conversion/JSONL validation and normal-reset Driver/Run replay must represent payment-stage source actions without defaults.
- Inventory all affected policy/action/snapshot/record versions; explicitly preserve compatible artifacts or version and reject incompatible ones, with tests. Do not silently reinterpret old artifacts. Existing float-first regressions remain unchanged.
- Real pinned XMage Shivan/Invoker scripts must actually invoke mana abilities inside payment, export intermediate target/mana/tap/stack checks and prove no invented response window. Retain existing float-first reference cases.

Independent oracle: pinned Shivan/Invoker/Elf/Druid/Cavalry Oracle and CR 602.2b applying 601.2g/h, 605.3a, 302.6, 702.10, 113.7a and 611.2; literal costs and stats above. Tests declare synthetic setups separately from required normal-reset played recording evidence. Excludes general ability language/layers or unrelated mechanic implementation.

Delivery contract: one reviewed PR; write independently justified behavioral/adapter failure tests before implementation (compile errors are not behavioral red), preserve minimized scripts/seeds and every existing test in normal discovery (`cargo test --workspace --locked`, `python3 scripts/run_tests.py`). Use pinned rules/card sources and literal authored expectations, never engine-generated expected output. Run applicable real pinned native/XMage comparisons twice, preserving exact inputs, consumed choices, checkpoints, source/toolchain/bridge hashes, limitations and first-divergence negative controls. Missing observations/build failures cannot count as agreement. Run full `./scripts/torture.sh` in the managed container through the shared heavy lock after current-main integration. Commit clean and run the prescribed separate `python3 scripts/symphony/review.py origin/main`; resolve findings and preserve exact-candidate review. Protected PR verify/merge and successful CI on the exact main merge SHA precede completion. Assess/update README supported behavior, commands, compatibility and evidence; independent reviewer checks README accuracy. No test weakening, substitute sibling engine, silent defaults or skipped capability.

Authorization: proposed by operations #252; remain UNREADY until its reviewed registration merges and exact-main CI passes, and all listed prerequisites have completed acceptance/review/merge/exact-main receipts. Only normal dependency/control/label-history checks can dispatch this task. Preserve original catalog setups/actions/expectations/bases/owners and aggregate audits #23/#24/#26. No M2 completion claim, RFC 0003 migration, RL, source repin, policy/CI/settings change or new task registration. Reuse the linked diagnostics and original audit workspaces/drafts; do not replace or erase their evidence.

### #255 — exact Cub casting and single-Sentry Shivan reference checkpoints

Deliver strict native/pinned-XMage inputs and observable checkpoints for exactly the three missing unchanged #210 catalog slots, extending existing adapters. #210 retains all 50 assigned cases and composition/holdout/re-execution acceptance.

Prerequisites: #252, #80, #196 flying/reach/Sentry, #200 actual Shivan. M1 casting/Growth/priority contracts are inherited through #80/#22. No activation-payment repair prerequisite: these three cases do not activate Shivan.

Diagnostic: https://github.com/pabloxrl/mtg-lab/issues/210#issuecomment-6076401142; reuse preserved GH-210 draft and prerequisite-gap.md. Existing synthetic flying Cub/Growth and real Shivan grown_split (two blockers, zero to grown blocker) do not establish the required single-Sentry interaction.

Acceptance:
- rules-combat-creature-abilities-positive: P0 main, empty stack, Cub in hand and 1G; cast then resolve. Observe payment/stack/zone transition, 2/2 Cub untapped and unable to attack this turn. Presence alone is insufficient.
- rules-combat-creature-abilities-negative: P1 has priority during P0 turn, Cub in hand; explicitly attempt non-flash creature casting and observe rejection and unchanged zones/mana/stack/actor. No silent skip of the attempted illegal action.
- rules-foundations_micro_v1-magnigoth-sentry-interaction: real 5/5 Shivan attacks, one real 4/4 Sentry blocks then gets Growth; observe 7/7 before damage, five marked damage on surviving Sentry and Dragon killed by seven, unchanged player life. Do not use a synthetic flying Cub or multiple-blocker split.
- Interaction/regression controls: same-turn sickness versus untapped status, legal own-turn Cub cast, seat reversal, illegal Cub flying block, mutated tap/sickness/damage/zone checkpoints and omitted/extra/wrong-actor choices must fail at a named first divergence. Preserve all existing related bridge cases.

Independent oracle: exact catalog text and pinned Cub/Sentry/Shivan/Growth Oracle; CR 302.6/601/509/510/702.9/702.17. Expectations authored before adapter changes. Add ordinary-discovery strict parsing, actual derived observation and comparator tests. Excludes production rules changes, new combat engine, remaining #210 compositions and activation-payment support.

Delivery contract: one reviewed PR; write independently justified behavioral/adapter failure tests before implementation (compile errors are not behavioral red), preserve minimized scripts/seeds and every existing test in normal discovery (`cargo test --workspace --locked`, `python3 scripts/run_tests.py`). Use pinned rules/card sources and literal authored expectations, never engine-generated expected output. Run applicable real pinned native/XMage comparisons twice, preserving exact inputs, consumed choices, checkpoints, source/toolchain/bridge hashes, limitations and first-divergence negative controls. Missing observations/build failures cannot count as agreement. Run full `./scripts/torture.sh` in the managed container through the shared heavy lock after current-main integration. Commit clean and run the prescribed separate `python3 scripts/symphony/review.py origin/main`; resolve findings and preserve exact-candidate review. Protected PR verify/merge and successful CI on the exact main merge SHA precede completion. Assess/update README supported behavior, commands, compatibility and evidence; independent reviewer checks README accuracy. No test weakening, substitute sibling engine, silent defaults or skipped capability.

Authorization: proposed by operations #252; remain UNREADY until its reviewed registration merges and exact-main CI passes, and all listed prerequisites have completed acceptance/review/merge/exact-main receipts. Only normal dependency/control/label-history checks can dispatch this task. Preserve original catalog setups/actions/expectations/bases/owners and aggregate audits #23/#24/#26. No M2 completion claim, RFC 0003 migration, RL, source repin, policy/CI/settings change or new task registration. Reuse the linked diagnostics and original audit workspaces/drafts; do not replace or erase their evidence.

### #256 — strict Sentry Bite and Growth bookkeeping reference inputs

Extend the existing strict instant-response native/XMage adapters to express the six unchanged ordinary Cub/Sentry Bite cases assigned to #212. Preserve exact identity, script consumption and observations; #212 retains its 31-case aggregate acceptance.

Prerequisites: #252, #80, #196 Sentry characteristics, #137 departed-target shared scripts, #139 Growth/cleanup scripts (both inherit #136). No dependency on the keyword or activation or terminal repair solely because files overlap.

Diagnostic: https://github.com/pabloxrl/mtg-lab/issues/212#issuecomment-6078764991; reuse preserved GH-212 prerequisite-gap.md and prerequisite-audit.json (all 31 cases). InstantResponseTest currently rejects Sentry at its input name map. Thornweald/deathtouch, Shivan and Cyclops fixtures cannot substitute.

Acceptance, with exact catalog setups/actions and independently authored checkpoints:
- rules-foundations_micro_v1-bite-down-positive: own 2/2 Cub to opposing 4/4 Sentry; Sentry receives two marked damage, remains 4/4, Cub takes zero.
- rules-foundations_micro_v1-bite-down-negative: own Cub to own Sentry rejects friendly destination atomically; mana/targets/zones/history remain unchanged at the rejected boundary.
- rules-foundations_micro_v1-bite-down-regression: kill Cub source with a real response above Bite; opposing Sentry remains unmarked, no cached/LKI source damage.
- rules-objects-bookkeeping-positive: inspect Sentry after two Bite damage: 4/4 and damage two, not toughness two.
- rules-objects-bookkeeping-interaction: damaged Sentry receives Growth: 7/7 with two damage, cleanup leaves 4/4 with zero.
- rules-continuous-resolution-power-positive: Growth on source above Bite resolves first; Cub 5/5 deals five and Sentry dies.
- Strict negative/regression controls: omitted/extra/reordered/wrong-actor choices, friendly targets, source-incarnation retargeting and mutated power/damage/cleanup/stack checkpoints fail with preserved first divergence. Preserve all M1 shared scripts and receipts; do not broaden a whitelist without matching execution and observation coverage.

Independent oracle: pinned Bite/Growth/Cub/Sentry Oracle; CR 608.2b/h, 120.6, 400.7, 514.2 and exact unchanged catalog. Normal discovery must execute native inputs, strict translator and comparator controls. Excludes production rules implementation, deathtouch substitutes, keyword-combat or mixed-loss contracts and remaining #212 compositions.

Delivery contract: one reviewed PR; write independently justified behavioral/adapter failure tests before implementation (compile errors are not behavioral red), preserve minimized scripts/seeds and every existing test in normal discovery (`cargo test --workspace --locked`, `python3 scripts/run_tests.py`). Use pinned rules/card sources and literal authored expectations, never engine-generated expected output. Run applicable real pinned native/XMage comparisons twice, preserving exact inputs, consumed choices, checkpoints, source/toolchain/bridge hashes, limitations and first-divergence negative controls. Missing observations/build failures cannot count as agreement. Run full `./scripts/torture.sh` in the managed container through the shared heavy lock after current-main integration. Commit clean and run the prescribed separate `python3 scripts/symphony/review.py origin/main`; resolve findings and preserve exact-candidate review. Protected PR verify/merge and successful CI on the exact main merge SHA precede completion. Assess/update README supported behavior, commands, compatibility and evidence; independent reviewer checks README accuracy. No test weakening, substitute sibling engine, silent defaults or skipped capability.

Authorization: proposed by operations #252; remain UNREADY until its reviewed registration merges and exact-main CI passes, and all listed prerequisites have completed acceptance/review/merge/exact-main receipts. Only normal dependency/control/label-history checks can dispatch this task. Preserve original catalog setups/actions/expectations/bases/owners and aggregate audits #23/#24/#26. No M2 completion claim, RFC 0003 migration, RL, source repin, policy/CI/settings change or new task registration. Reuse the linked diagnostics and original audit workspaces/drafts; do not replace or erase their evidence.

### #257 — per-seat failed draw and explicit simultaneous-loss reference outcomes

Extend the existing shared terminal native/XMage fixture contract with per-seat failed draw and explicit winner/draw observations, so different loss reasons can be settled in one actual SBA batch. #212 retains aggregate acceptance; synthetic boundary setup is explicit and does not claim game reachability.

Prerequisites: #252 reviewed registration, #80 and #72 terminal outcomes/shared reference bridge. No dependency on the other repairs.

Diagnostic: https://github.com/pabloxrl/mtg-lab/issues/212#issuecomment-6078764991; preserved GH-212 inventory. Current shared native runner and TerminalTest always draw for P0; observations expose lost flags but no explicit outcome. Separate life-loss and empty-draw scripts do not test mixed simultaneous loss.

Acceptance:
- rules-terminal-simultaneous-loss-interaction: inject P0 zero life and cause P1 to fail an empty draw before the same SBA batch; after actual engine settlement both lose and explicit outcome is draw. Seat-swapped mixed reasons must also draw.
- Positive/single-loss controls: only zero life and only failed draw for each seat yield the other seat as winner; both life losses draw. Check explicit per-seat lost flags and canonical outcome/winner together.
- Negative: empty library without attempted draw causes no loss; drawing the last card is successful and nonterminal; invalid seat/unknown fields/malformed or conflicting action inputs fail visibly, without defaulting to P0. No intermediate SBA that awards a winner before processing the other pending reason.
- Interaction/regression: retained seven existing terminal fixtures preserve all prior expectations; test synthetic same-batch ordering independently of reason/seat; corrupt winner/draw/lost flags, wrong draw seat and premature settlement must be detected with minimized input and named divergence.

Independent oracle: pinned CR 104.3/104.4a, 121 and 704.5a/b and literal unchanged catalog. Author explicit expected outcome before extending adapters; observe actual native/XMage results rather than computing the answer from fixture expectations. Version the fixture/observation contract or provide explicit tested legacy compatibility; no silent old-artifact reinterpretation. Normal discovery covers strict parsing, native terminal execution and all comparator fields. Excludes new production terminal rules, normal-game reachability claims, alternative engine and other #212 compositions.

Delivery contract: one reviewed PR; write independently justified behavioral/adapter failure tests before implementation (compile errors are not behavioral red), preserve minimized scripts/seeds and every existing test in normal discovery (`cargo test --workspace --locked`, `python3 scripts/run_tests.py`). Use pinned rules/card sources and literal authored expectations, never engine-generated expected output. Run applicable real pinned native/XMage comparisons twice, preserving exact inputs, consumed choices, checkpoints, source/toolchain/bridge hashes, limitations and first-divergence negative controls. Missing observations/build failures cannot count as agreement. Run full `./scripts/torture.sh` in the managed container through the shared heavy lock after current-main integration. Commit clean and run the prescribed separate `python3 scripts/symphony/review.py origin/main`; resolve findings and preserve exact-candidate review. Protected PR verify/merge and successful CI on the exact main merge SHA precede completion. Assess/update README supported behavior, commands, compatibility and evidence; independent reviewer checks README accuracy. No test weakening, substitute sibling engine, silent defaults or skipped capability.

Authorization: proposed by operations #252; remain UNREADY until its reviewed registration merges and exact-main CI passes, and all listed prerequisites have completed acceptance/review/merge/exact-main receipts. Only normal dependency/control/label-history checks can dispatch this task. Preserve original catalog setups/actions/expectations/bases/owners and aggregate audits #23/#24/#26. No M2 completion claim, RFC 0003 migration, RL, source repin, policy/CI/settings change or new task registration. Reuse the linked diagnostics and original audit workspaces/drafts; do not replace or erase their evidence.

## Controlled recovery

Keep all four children unready until protected registration merge and exact-main
CI pass. Then, under the shared handoff lock, re-fetch parent/current/task controls,
label histories and all other ready/running claims. Record coordinator reactivation
for #23/#210/#212 (and #217 as specified above), clearing only their documented prerequisite-gap blocks and adding
`agent-resume-authorized`; leave them unready while new dependencies are pending.
Other holds/withdrawals remain authoritative. Preserve original workspaces/drafts.
This is explicit #252 coordinator authority, not feature-worker permission.

Normal bounded handoff queues at most one eligible child. The root coordinator
may fill independent free Symphony slots only after the same controls/evidence
checks. Preserve #216's benchmark-only resumed handoff and any successor claim.
Update the compact #7 workpad under the handoff lock with short issue-keyed links,
retaining all six archive links. Do not close or claim completion of the audits.
#241 still owns E1a/reference-corpus registration only after #26.
