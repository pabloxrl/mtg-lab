# GH-24 coverage audit: missing normal-reset reference contract

**Unsuccessful integration audit; M2 remains incomplete.** Audited main
`1df9b65e95b2976ef9eeb3e46b66877580e2e20c`. This report preserves a missing
prerequisite, not a delivered rules change, fresh reference run, scenario-floor
admission or milestone verdict. No existing regression or expectation changes.

## Established evidence and remaining acceptance

All six direct prerequisites are closed completed. Their
[completion workpads, protected merges and successful exact-main CI](dependencies.json)
were re-fetched during this audit. Independent reviews and executable acceptance
are linked in those workpads. Current-main program/source/ledger/gates validate.

The [catalog inventory](catalog-inventory.json) joins all **175 unchanged M2
catalog rows**, with original expectations, owner, exact child mapping, observation
boundary and hashed receipt: 56 costs, 50 combat, 38 triggers and 31 foundation.
This is an allocation/receipt inventory, not 175 distinct reviewed scenarios.
Repeated invocations, seats and mappings must not be counted as new scenarios
without individually checking their semantic distinction and review evidence.
The complete 100-scenario admission audit is still outstanding; this report does
not establish that fewer than 100 suitable cases exist.

| Original requirement / clause | Evidence and audit outcome |
| --- | --- |
| R0002-B010: exact two 40-card decks, twenty definitions and token | [Mechanics audit](../m2-mechanics-audit/README.md) and its card index retain native full-pool coverage. Synthetic XMage cases do not establish a normal-reset game using those full multisets. |
| R0002-B011: pinned rules and scoped mechanics | Child maps retain literal rules/card-derived expectations and source limits. This audit makes no new upstream-vintage or whole-rules-baseline certification. |
| R0002-B026: all twelve scenario families, normal/adversarial combinations | All 175 M2 allocations are preserved. Setup/randomness has the concrete reference gap below. Priority/stack, costs/mana, targets, triggers, combat, continuous changes, objects/zones and terminal states have bounded child reference evidence. Decisions, privacy and pending replay have native assertions with explicit reference limits. Complete family-by-family distinct-scenario admission remains outstanding; batch/worker extensions remain deferred by the current scope amendment. |
| R0002-B028: real out-of-process pinned reference execution and provenance | The four child receipts record actual bounded native/XMage execution, source/build pins and limitations. Their exact old runs are inherited evidence, not freshly run on this audit head. No unavailable or unexecuted reference is counted as agreement. |
| R0002-B028: same neutral inputs, strict choices, chance and identities | **Incomplete:** no occurrence-preserving full-deck normal-reset chance/mulligan path is supplied by the opening bridge. GR-010/011/012 and GR-020/021/022 remain unfulfilled for that path. |
| R0002-B028: intermediate semantic normalization, legal rejection, every capability/card | Child receipts compare named fields and rejection/domain boundaries; [mechanics reference index](../m2-mechanics-audit/reference-index.json) and card index retain coverage for twenty cards. No full-state or every-capability verdict is possible while required opening identity/order fields are unobservable. |
| R0002-B028: independent dual suite and complete games | Forge qualification and later corpus work retain their existing owners/current-plan registration; this report does not demand implementing Forge or the new AI corpus inside GH-24. GH-24's explicit initial XMage full-pool path remains missing independently of those later quotas. Synthetic child positions and native terminal games cannot discharge that initial reference obligation. |
| R0002-B028: planned/executed/agreed/disputed/unsupported, first divergence and adjudication | Child receipts retain bounded counts and comparator controls. This inventory preserves boundaries rather than promoting them to aggregate agreement. The opening gap is unsupported coverage, not an observed rules disagreement. A final per-capability aggregate remains outstanding. |
| R0002-B030: supported cases pass without xfails/skips, independent expectations, holdouts and retained failures | Existing ordinary discovery and child evidence remain authoritative; no tests are removed or weakened. Four pack holdouts retain their original provenance. Full-scope admission is not complete. New validation results belong in the issue workpad; a suite pass cannot implement missing reference coverage. |
| Selected mutants caught by named tests | [Twelve compiled mutation detections](../m2-mutations/README.md) retain baseline, named assertion, minimized input and exact-source receipts. This audit does not claim a new mutation execution. |

The unchanged child reports distinguish actual engine output from comparator
mutations, synthetic fixtures from normal-reset reachability, and native raw
rejection from reference legal-domain observations. Those distinctions remain
mandatory in any successful aggregate report.

## Concrete missing contract

The [opening bridge](../../../references/xmage/OpeningCountsTest.java) is explicitly
an eight-case **count** ledger. `openingCounts` inserts seven Forests/Mountains
directly into each hand and 33 copies into each library because the upstream test
constructor skips initial draws. `skipInitShuffling=true` disables initial
shuffling; `shuffleLibrary` is a no-op; `chooseTarget` selects the first hand UUID
without consuming an occurrence-aware bottom tape. `priority` observes only
declaration/hand/library/bottom counts and stops at the first upkeep.
The [runner](../../../scripts/opening_reference.py) accurately declares no initial
shuffle, card identity/order or full-game comparison. These are deliberate limits
of an earlier bounded delivery, not newly alleged defects in that delivery.

The [neutral smoke](../../../references/xmage/NeutralSmokeTest.java) rejects
mulligans and accepts only the explicit one-pass script. The
[instant bridge](../../../references/xmage/InstantResponseTest.java) also rejects
unexpected mulligans and explicitly uses synthetic starting positions. The four
M2 pack reports all disclaim normal-reset reference-game coverage. Their focused
rules observations cannot fill the missing chronology/chance/identity contract.
The [source hashes](source-hashes.json) pin this inspection; this is source-level
evidence of an absent bridge, not a compiled behavioral-red test or a failed
live XMage execution.

The required counterexample is a red/green 40-card opening in which two physical
copies of the same card occupy different positions, a player mulligans, a recorded
permutation redraws them, and an explicit ordered bottom choice selects one copy.
Swapping just those occurrence identities, omitting a shuffle event or adding an
unused chance event must fail, even when all hand/library counts remain equal.
The existing count-only bridge has no input or observation field to run that
counterexample. No fabricated result or count-only substitute is admitted.

## Proposed coordinator split (not registered or authorized here)

1. **Occurrence-preserving XMage opening transcript.** Version the shared neutral
   occurrence/chance contract as needed; bind both frozen 40-card multisets at
   creation, implement real initial draws and London declaration/redraw/ordered
   bottom chronology, consume every recorded permutation exactly once, and export
   opening/next-declaration/first-draw checkpoints. Test both starters, 0/1/multiple
   mulligans and both players mulliganing. Retain independently derived expected
   orders; wrong identity/actor/order, missing/extra randomness and callbacks must
   fail. Preserve the old count tests. This is one independently testable opening
   path, not a whole-game implementation.
2. **Continuous full-pool XMage transcript and checkpoint integration.** After the
   opening contract, join existing focused mechanics into one strict normal-game
   controller for passes, payments, targets/modes/discards, triggers, attacks,
   blocks, damage and cleanup. Preserve occurrence/incarnation/token/stack identity
   through zones; export required intermediate fields and actual consumed choices.
   Missing fields/default callbacks reject visibly. Split further by concrete
   bridge interfaces if source inspection shows this is not one bounded delivery.
3. **Initial normal-reset native/XMage game admission.** Depends on the continuous
   controller. Retain reviewed rules-terminal games across the required matchup
   rows with exact tapes/checkpoints, deterministic repeats and negative controls;
   keep later dual-reference and amended AI-corpus obligations distinct. Reconcile
   the final quota/ownership against the unchanged current plan during registration,
   rather than treating this proposal as authority to move or waive requirements.

Resume GH-24 only after the necessary contracts have reviewed executable delivery
and exact-main CI. Then complete the distinct-scenario/family admission and aggregate
capability evidence; do not waive them because the catalog has 175 rows. The
coordinator decides the actual task registration/dependencies. This worker adds
no task, changes no manifest, and does not expand into RFC 0003 or new RL work.

README assessment: no new usable command, setup, supported behavior, architecture
or verified milestone status is delivered by this failed audit. The root already
keeps M2 incomplete, so no root README change is needed. Independent diagnostic
review should check that assessment and the evidence/scope distinction. No PR or
successful delivery is implied by preserving this report locally.
