# Matched instant reference prerequisites

Operations [#135](https://github.com/pabloxrl/mtg-lab/issues/135) implements the smaller split proposed by the unsuccessful #115 audit. Existing native spell tests use declared synthetic positions; existing combat adapters start at different steps and cannot be represented as an identical complete response script. This plan delivers the missing shared execution infrastructure and reference cases without weakening that obligation.

The pinned RFC, all 44 verbatim requirement blocks, all previous owners and all 320 catalog designs remain unchanged. New owners implement subsets of B017/B026/B029; #115 retains every original matched-reference acceptance clause and #18 retains the full scalar-slice audit. M1 remains incomplete until #22 passes.

| Issue | Deliverable | Direct prerequisites |
| --- | --- | --- |
| [#136](https://github.com/pabloxrl/mtg-lab/issues/136) | Strict native and XMage instant-response executor | #135, #70 |
| [#137](https://github.com/pabloxrl/mtg-lab/issues/137) | Departed-target identities in matched instant scenarios | #136 |
| [#138](https://github.com/pabloxrl/mtg-lab/issues/138) | Bite-killed blocker through a shared reference script | #136, #71 |
| [#139](https://github.com/pabloxrl/mtg-lab/issues/139) | Shared cleanup and discard reference continuations | #136 |

The three scenario extensions branch independently from #136. The blocker scenario additionally requires delivered #71. #115 waits for #137–#139, which transitively include #136 and reviewed registration #135. No fake executor or sibling implementation is permitted to satisfy a task.

## #136: Strict native and XMage instant-response executor

One original shared ordered script format with native and pinned-XMage adapters for explicit setup, mana/payment/target/cast/pass choices. Deliver only Growth responding to Bite on the destination and source-power-at-resolution variants. Align the initial boundary and exact consumed semantic script in both engines; do not reuse different starting steps as identical execution. No departed targets, combat or cleanup extension.

Execute both real engines twice. Literal independent CR/Oracle expectations: 2/2 destination becomes 5/5 and survives two damage; source boosted before Bite deals five. Compare declared priority/stack/targets/stats/damage/zones at common checkpoints. Omitted, extra, reordered or wrong-actor choices fail; mutated target/power/stack must be detected. No AI fallback or caller-supplied success checkpoints.

## #137: Departed-target identities in matched instant scenarios

Extend the delivered executor with stable semantic target identities after real responding spells change zones. Cover Growth sole-target departure and both Bite target roles separately; distinguish partial from all-invalid resolution, no source-LKI damage, no replacement-object retargeting. No combat/cleanup.

Run repeated native/XMage sequences with exact scripts and independent expected zones/stack targets/resolution outcomes. A target leaving and a same-name new object must never be substituted. Negative controls detect retargeting, damage from illegal source, missing identity or silently unconsumed choice. Preserve first divergent checkpoint and minimized script.

## #138: Bite-killed blocker through a shared reference script

Extend the executor only for a vanilla declaration/response sequence: attack, block, cast Bite to kill blocker, resolve and reach combat damage. Preserve exact starting state, declaration/response/pass order, semantic references and observed blocked status. No cleanup or general combat harness rewrite.

Repeated native/XMage runs agree at declared intermediate stack/target/death/combat checkpoints. Independently expect that a blocked attacker remains blocked after its blocker dies and causes no player damage. A forgotten-blocked-status mutation is detected; illegal declarations and omitted/reordered responses fail rather than auto-select.

## #139: Shared cleanup and discard reference continuations

Extend executor only through cleanup and next turn for explicit seven/eight-card cases and boosted creatures with two/four marked damage. Supply exact discard choices before simultaneous expiration/damage removal. No new production rules, general card pool or combat expansion.

Repeated native/XMage runs compare hand/zone order, exact discarded identity, power/toughness/damage, priority and next-turn checkpoints against pinned independent rule expectations. Negative controls detect premature expiration/removal, missing/extra discard or invented priority. Seven-card no-discard and eight-card discard paths both execute strictly.

## Shared evidence requirements

Each task uses original independently justified CR/Oracle expectations, identical initial conditions and exact consumed semantic choices in native and real pinned XMage execution. Repeat runs, preserve source/card/toolchain/bridge pins, provenance and licenses, declare unobservable fields, and retain first-divergence artifacts and comparator mutations. An unavailable reference, wrong scenario, skipped execution, supplied success checkpoint or AI fallback cannot pass. Strict translator/comparator regressions remain in ordinary discovery; real external-engine receipts are separately identified. Full Docker torture, independent review, protected merge and exact-main CI remain mandatory.

#115 audits destination/source response variants, all target-departure cases, the killed blocker and both cleanup/discard paths after these children. Its original body and failed audit remain historical evidence; passing children do not waive any aggregate requirement. #18 remains dependent on #115. Forge, full-pool capability, dual-reference critical-suite and full-game qualification requirements retain their existing owners and gates.

## Recovery

After #135 protected delivery and exact-main CI, the coordinator updates #115 native dependencies and body, removes its resolved sizing block and grants deferred agent-resume-authorized. It remains unready until its new prerequisites have complete acceptance/review/merge/main-CI evidence. The bounded handoff consumes that grant only when normal controls pass. Archive its old attempt counter once while the controller is stopped; preserve its audit and checkout.

Preserve and explicitly resume #116 saved manifest work after its coordinator-approved one-time counter archive. This prioritizes finishing an existing candidate without altering its prerequisites or gates. Queue only #116; afterward normal manifest-order handoff can select #136. No other issue becomes ready simultaneously. If #116 blocks again, its ordinary failed-task handoff can select an independent new reference task. No policy, retry ceiling, concurrency or automatic retry-reset rule changes.
