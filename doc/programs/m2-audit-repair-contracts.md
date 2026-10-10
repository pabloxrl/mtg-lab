# M2 audit repair child acceptance contracts

Registered by #268; implementation remains pending. Each section is the complete
child issue contract. Family runner and focused test commands are required delivery
interfaces, not already implemented commands. Existing tests remain unchanged.

## #269 — Occurrence-preserving native/XMage normal reset

Parent audit: #24. Registration: #268. Milestone M2; proposed until registration delivery.

## Observable deliverable

Occurrence-preserving native/XMage normal reset.

| Contract | Acceptance |
| --- | --- |
| Owned interfaces | New versioned test-only occurrence/chance envelope and native reset consumer around opening::Config/DeckConfig, object insertion/semantic_identity; new XMage normal-game constructor/initial shuffle hook (OpeningCountsTest remains unchanged). |
| Runnable positive acceptance | All eight exact 40-card matchup/starter rows execute actual reset and seven-card draws, then stop at first declaration. Bind (seat, card key, copy index) at creation before permutation; compare literal ordered library and hand membership, occurrence map, life 20 and active/declaration seat. Swap same-name copies in valid initial permutations and observe the independently specified different occurrence positions. Repeat reset and compare stable canonical identities without reusing stale native handles. New consumer must use existing Game reset/object APIs and real pinned XMage opening execution in this PR, not merely parse a schema. |
| Runnable negative acceptance | Reject wrong 39/41-card multiset, token in deck, unknown pin, duplicate/missing occurrence, wrong actor/kind/sequence/pre-event multiset, missing/extra/reordered initial chance events and swapped occurrence checkpoint despite unchanged counts. Any unimplemented mulligan or ongoing callback fails explicitly; bounded stop at first declaration is declared prefix completion, never full-game completion. |
| Independent oracle | CR 103 initial setup/draw and frozen deck JSON multiplicities; independently authored occurrence lists and permutations. GR-001/011/012/020. |
| Prerequisites | #268, #64 |
| Independent siblings | #278, #279 (subject to their own prerequisites and three-worker cap) |
| Exclusions | Mulligan chronology, first-turn progression, ongoing choices, Forge and complete-game admission. |

Focused acceptance: `python3 -m unittest discover -s tests -p 'test_m2_repair_reset.py'` plus full native normal discovery. The Python module must exercise the delivered real adapter/client, validate nonempty observations and run its negative controls; a parser-only pass cannot satisfy a played-prefix claim. The pinned reference runner family is `reset`, executed twice by the command below.

Partial additive owners: R0002-B010, R0002-B011, R0002-B026, R0002-B028, R0002-B030. All original owners and original acceptance remain.

## Mandatory delivery and authorization

One observable deliverable, one reviewed PR. Write behavioral/adapter tests FIRST and record an intended assertion failure (compile/import errors are not red evidence). Derive expected results independently from the pinned rules/card/requirements above; never bless actual engine output. Preserve every existing test and catalog case/owner/expectation; keep minimized seeds/tapes, new positive/negative controls and defect reproductions in normal discovery. Synthetic adapter probes supplement the required real normal-reset/played prefixes; no mock alternative rules engine or unimplemented sibling may be the only meaningful consumer.

Run `python3 scripts/run_tests.py` and `cargo test --workspace --locked` with the new cases discovered (record nonzero new test count). Add the focused test module named below and include it in normal discovery; commands below are delivery interfaces to implement, not claims they already exist. Each reference child extends the same versioned test runner, preserving prior families. At dispatch every prerequisite must have completed acceptance, review, protected merge and successful exact-main CI receipts.

For reference adapter changes run the actual pinned native/XMage family twice through `python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- python3 scripts/full_pool_reference.py --family reset --cache "$MTG_REFERENCE_CACHE" --output .agent-artifacts/reset-run-N.json` (N=1,2). Preserve inputs, actual consumed choices/chance, checkpoints, independent oracle, source/toolchain/dependency/bridge hashes, first-divergence negative controls and privileged artifact separation. Unsupported callbacks/observations fail explicitly; do not guess identities, inject derived state, default choices or report missing fields as agreement. All requested fields must have witnessed engine provenance; raw legal rejection, selected-action acceptance and complete legal-set comparison are distinct claims. Existing source/card/rules pins and compatibility tests remain unchanged; version new envelope semantics explicitly.

For instrumentation/profile/measurement changes, actual native runs prove four-mode semantic/privacy/capture equality; reuse pinned reference receipts for unchanged rules, run applicable affected real native/XMage comparisons twice with the same evidence contract if rules/semantic boundaries change. Do not manufacture a reference run for clock arithmetic or count unavailable hardware as zero.

After integrating current main run the full `python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- ./scripts/torture.sh`; all Cargo/Maven/reference/heavy commands use that existing shared lock. Commit clean and run the prescribed separate `python3 scripts/symphony/review.py origin/main` exact-candidate Codex review. Resolve blockers and retain review JSON, protected PR verify/merge and successful CI on the exact main SHA. Assess README impact, update supported commands/claims when affected, verify affected quickstart commands and require the independent reviewer to check README accuracy. No weakening/skips/fallbacks. No human routine approval.

Remain UNREADY until #268 reviewed registration merges and its exact-main CI passes. #268 is an explicit prerequisite, never self-satisfying by this text. Symphony implementation only, at most three workers, no off-dashboard implementation agent. Preserve holds/withdrawals and dependency controls. No task registration, RFC 0003, RL, source repins, workflow/CI/settings changes, new CLI/product surface or M2 completion claim. #24/#25 keep full original audits; #26 retains all ancestors; #241 owns later architecture/reference-AI corpus after verified #26. Reuse original audit workspaces/evidence; no deletion.

## #270 — Strict native/XMage London chance and choice chronology

Parent audit: #24. Registration: #268. Milestone M2; proposed until registration delivery.

## Observable deliverable

Strict native/XMage London chance and choice chronology.

| Contract | Acceptance |
| --- | --- |
| Owned interfaces | Occurrence envelope opening extension; Game::apply_with_order/bottom_cards and XMage shuffleLibrary/chooseMulligan/chooseTarget; actual consumed chance/choice ledger. |
| Runnable positive acceptance | Real reset for both starters and RG/GR/RR/GG with 0/1/multiple mulligans, both players mulliganing and distinct duplicate copies. Compare each declaration, redraw, cumulative ordered bottom and next declaration against hand-written CR chronology. Initial hands seven; after nth mulligan redraw seven then bottom n before the next declaration round. Exact ordered bottoms place chosen occurrences at library bottom. All chance/choice sequences consumed once; stop at first upkeep. Preserve old count-only tests. |
| Runnable negative acceptance | Wrong seat/kind/source, stale occurrence, wrong bottom cardinality, duplicate bottom, reordered declaration/bottom, missing/extra/reordered shuffle or choice, incomplete ledger and unexpected callback reject without default choice. Native rejection leaves state/RNG unchanged; reference rejection and unobservable RNG fields are reported distinctly. |
| Independent oracle | Pinned CR 103.5 and existing opening-counts independent ledger; GR-010/011/012/020/021. Explicitly review upstream chronology at the pin, do not mimic implementation output. |
| Prerequisites | #268, #269, #65 |
| Independent siblings | #278, #279, #280 (subject to their own prerequisites and three-worker cap) |
| Exclusions | First draw and ongoing priority, game admission, synthetic injected hands. |

Focused acceptance: `python3 -m unittest discover -s tests -p 'test_m2_repair_mulligan.py'` plus full native normal discovery. The Python module must exercise the delivered real adapter/client, validate nonempty observations and run its negative controls; a parser-only pass cannot satisfy a played-prefix claim. The pinned reference runner family is `mulligan`, executed twice by the command below.

Partial additive owners: R0002-B010, R0002-B011, R0002-B026, R0002-B028, R0002-B030. All original owners and original acceptance remain.

## Mandatory delivery and authorization

One observable deliverable, one reviewed PR. Write behavioral/adapter tests FIRST and record an intended assertion failure (compile/import errors are not red evidence). Derive expected results independently from the pinned rules/card/requirements above; never bless actual engine output. Preserve every existing test and catalog case/owner/expectation; keep minimized seeds/tapes, new positive/negative controls and defect reproductions in normal discovery. Synthetic adapter probes supplement the required real normal-reset/played prefixes; no mock alternative rules engine or unimplemented sibling may be the only meaningful consumer.

Run `python3 scripts/run_tests.py` and `cargo test --workspace --locked` with the new cases discovered (record nonzero new test count). Add the focused test module named below and include it in normal discovery; commands below are delivery interfaces to implement, not claims they already exist. Each reference child extends the same versioned test runner, preserving prior families. At dispatch every prerequisite must have completed acceptance, review, protected merge and successful exact-main CI receipts.

For reference adapter changes run the actual pinned native/XMage family twice through `python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- python3 scripts/full_pool_reference.py --family mulligan --cache "$MTG_REFERENCE_CACHE" --output .agent-artifacts/mulligan-run-N.json` (N=1,2). Preserve inputs, actual consumed choices/chance, checkpoints, independent oracle, source/toolchain/dependency/bridge hashes, first-divergence negative controls and privileged artifact separation. Unsupported callbacks/observations fail explicitly; do not guess identities, inject derived state, default choices or report missing fields as agreement. All requested fields must have witnessed engine provenance; raw legal rejection, selected-action acceptance and complete legal-set comparison are distinct claims. Existing source/card/rules pins and compatibility tests remain unchanged; version new envelope semantics explicitly.

For instrumentation/profile/measurement changes, actual native runs prove four-mode semantic/privacy/capture equality; reuse pinned reference receipts for unchanged rules, run applicable affected real native/XMage comparisons twice with the same evidence contract if rules/semantic boundaries change. Do not manufacture a reference run for clock arithmetic or count unavailable hardware as zero.

After integrating current main run the full `python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- ./scripts/torture.sh`; all Cargo/Maven/reference/heavy commands use that existing shared lock. Commit clean and run the prescribed separate `python3 scripts/symphony/review.py origin/main` exact-candidate Codex review. Resolve blockers and retain review JSON, protected PR verify/merge and successful CI on the exact main SHA. Assess README impact, update supported commands/claims when affected, verify affected quickstart commands and require the independent reviewer to check README accuracy. No weakening/skips/fallbacks. No human routine approval.

Remain UNREADY until #268 reviewed registration merges and its exact-main CI passes. #268 is an explicit prerequisite, never self-satisfying by this text. Symphony implementation only, at most three workers, no off-dashboard implementation agent. Preserve holds/withdrawals and dependency controls. No task registration, RFC 0003, RL, source repins, workflow/CI/settings changes, new CLI/product surface or M2 completion claim. #24/#25 keep full original audits; #26 retains all ancestors; #241 owns later architecture/reference-AI corpus after verified #26. Reuse original audit workspaces/evidence; no deletion.

## #271 — Strict played priority, land and creature-cast transcript prefixes

Parent audit: #24. Registration: #268. Milestone M2; proposed until registration delivery.

## Observable deliverable

Strict played priority, land and creature-cast transcript prefixes.

| Contract | Acceptance |
| --- | --- |
| Owned interfaces | Continuous prefix dispatcher for XMage priority/playLand/cast/playMana with basic-land mana and explicit passes; native actions::Record and delivered scalar policy/recording interfaces; shared occurrence-to-incarnation/stack-action mapping. |
| Runnable positive acceptance | Start from real full decks with explicit keep/mulligan prefix. Play lands and nontriggering creatures using floating and payment-stage basic mana, explicit generic/color payments and consecutive passes. Advance both starters through first/second draw (starter skips first); compare every priority handoff, announcement/payment/commit/resolution, land-play count, six mana values, tapping/sickness, ordered zones, incarnation and stack source/action identity. Every pass including empty combat declarations is an explicit tape entry. Two identical creatures must remain distinct through hand->stack->battlefield. Support bounded stop at named prefix checkpoint and strict continuation, not direct stack.resolve shortcuts. |
| Runnable negative acceptance | Missing/extra/reordered pass, wrong actor, off-turn creature cast, second land, insufficient/wrong-color payment, stale source/candidate, unscripted mana or any unsupported callback fails; attempted native commands preserve state on rejection. Alter first draw/priority/stack order/incarnation checkpoint and require first-field diff. |
| Independent oracle | CR 117, 305, 302.6, 601/608 and starting-player draw rule; frozen costs/stats; GR-022 and basic GR-030. Existing #208 semantic actions/recording is delivered, no mock sibling. |
| Prerequisites | #268, #270, #208 |
| Independent siblings | #278, #279, #280 (subject to their own prerequisites and three-worker cap) |
| Exclusions | Noncreature spells, triggered/activated choices, nonempty combat, cleanup discards and complete games; unsupported content fails at callback, never auto-passes. |

Focused acceptance: `python3 -m unittest discover -s tests -p 'test_m2_repair_priority.py'` plus full native normal discovery. The Python module must exercise the delivered real adapter/client, validate nonempty observations and run its negative controls; a parser-only pass cannot satisfy a played-prefix claim. The pinned reference runner family is `priority`, executed twice by the command below.

Partial additive owners: R0002-B010, R0002-B011, R0002-B026, R0002-B028, R0002-B030. All original owners and original acceptance remain.

## Mandatory delivery and authorization

One observable deliverable, one reviewed PR. Write behavioral/adapter tests FIRST and record an intended assertion failure (compile/import errors are not red evidence). Derive expected results independently from the pinned rules/card/requirements above; never bless actual engine output. Preserve every existing test and catalog case/owner/expectation; keep minimized seeds/tapes, new positive/negative controls and defect reproductions in normal discovery. Synthetic adapter probes supplement the required real normal-reset/played prefixes; no mock alternative rules engine or unimplemented sibling may be the only meaningful consumer.

Run `python3 scripts/run_tests.py` and `cargo test --workspace --locked` with the new cases discovered (record nonzero new test count). Add the focused test module named below and include it in normal discovery; commands below are delivery interfaces to implement, not claims they already exist. Each reference child extends the same versioned test runner, preserving prior families. At dispatch every prerequisite must have completed acceptance, review, protected merge and successful exact-main CI receipts.

For reference adapter changes run the actual pinned native/XMage family twice through `python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- python3 scripts/full_pool_reference.py --family priority --cache "$MTG_REFERENCE_CACHE" --output .agent-artifacts/priority-run-N.json` (N=1,2). Preserve inputs, actual consumed choices/chance, checkpoints, independent oracle, source/toolchain/dependency/bridge hashes, first-divergence negative controls and privileged artifact separation. Unsupported callbacks/observations fail explicitly; do not guess identities, inject derived state, default choices or report missing fields as agreement. All requested fields must have witnessed engine provenance; raw legal rejection, selected-action acceptance and complete legal-set comparison are distinct claims. Existing source/card/rules pins and compatibility tests remain unchanged; version new envelope semantics explicitly.

For instrumentation/profile/measurement changes, actual native runs prove four-mode semantic/privacy/capture equality; reuse pinned reference receipts for unchanged rules, run applicable affected real native/XMage comparisons twice with the same evidence contract if rules/semantic boundaries change. Do not manufacture a reference run for clock arithmetic or count unavailable hardware as zero.

After integrating current main run the full `python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- ./scripts/torture.sh`; all Cargo/Maven/reference/heavy commands use that existing shared lock. Commit clean and run the prescribed separate `python3 scripts/symphony/review.py origin/main` exact-candidate Codex review. Resolve blockers and retain review JSON, protected PR verify/merge and successful CI on the exact main SHA. Assess README impact, update supported commands/claims when affected, verify affected quickstart commands and require the independent reviewer to check README accuracy. No weakening/skips/fallbacks. No human routine approval.

Remain UNREADY until #268 reviewed registration merges and its exact-main CI passes. #268 is an explicit prerequisite, never self-satisfying by this text. Symphony implementation only, at most three workers, no off-dashboard implementation agent. Preserve holds/withdrawals and dependency controls. No task registration, RFC 0003, RL, source repins, workflow/CI/settings changes, new CLI/product surface or M2 completion claim. #24/#25 keep full original audits; #26 retains all ancestors; #241 owns later architecture/reference-AI corpus after verified #26. Reuse original audit workspaces/evidence; no deletion.

## #272 — Strict played spell modes, targets, discard costs and token identities

Parent audit: #24. Registration: #268. Milestone M2; proposed until registration delivery.

## Observable deliverable

Strict played spell modes, targets, discard costs and token identities.

| Contract | Acceptance |
| --- | --- |
| Owned interfaces | XMage choose/chooseTarget/chooseMode/playMana spell continuations; native semantic target/payment/discard/mode actions; token creation-event ordinal and departing-incarnation checkpoints. |
| Runnable positive acceptance | Normal-reset reachable prefixes cast Growth/Bite response chain, Thrill with an explicitly selected physical discard and ordered two-card draw, Fodder with two distinct creation-event token IDs, and both Surprise modes. Compare target roles and revalidation, cost-before-resolution, mana/discard commit, stack order, temporary effects/durations, actual tokens and token disappearance after a legally played Bite. Select among duplicate copies/tokens explicitly. Literal Growth Cub 5/5, Fodder two 1/1 red Goblins, Thrill discard one then draw two, and Oracle-derived Surprise modes; prefixes may stop before cleanup handled separately. |
| Runnable negative acceptance | Wrong target role/controller, departed incarnation, missing/disordered target, duplicate token ID, illegal mode, missing discard, cancel/insufficient resources, extra callback and truncated/extra tape fail. Mutate target/effect/token identity/order checkpoints. Incomplete spell must never partially commit a rejected native action. |
| Independent oracle | Pinned card definitions and CR 601.2, 608.2, 400.7, 111, 704; inherited #209/#212 exact cost/target receipts remain limited synthetic evidence. GR-030 and token part GR-012. |
| Prerequisites | #268, #271, #209, #212 |
| Independent siblings | #273, #275, #278, #279, #280 (subject to their own prerequisites and three-worker cap) |
| Exclusions | Triggered/activated choice dispatch, combat declarations, cleanup expiry and full-game admission. |

Focused acceptance: `python3 -m unittest discover -s tests -p 'test_m2_repair_spells.py'` plus full native normal discovery. The Python module must exercise the delivered real adapter/client, validate nonempty observations and run its negative controls; a parser-only pass cannot satisfy a played-prefix claim. The pinned reference runner family is `spells`, executed twice by the command below.

Partial additive owners: R0002-B010, R0002-B011, R0002-B026, R0002-B028, R0002-B030. All original owners and original acceptance remain.

## Mandatory delivery and authorization

One observable deliverable, one reviewed PR. Write behavioral/adapter tests FIRST and record an intended assertion failure (compile/import errors are not red evidence). Derive expected results independently from the pinned rules/card/requirements above; never bless actual engine output. Preserve every existing test and catalog case/owner/expectation; keep minimized seeds/tapes, new positive/negative controls and defect reproductions in normal discovery. Synthetic adapter probes supplement the required real normal-reset/played prefixes; no mock alternative rules engine or unimplemented sibling may be the only meaningful consumer.

Run `python3 scripts/run_tests.py` and `cargo test --workspace --locked` with the new cases discovered (record nonzero new test count). Add the focused test module named below and include it in normal discovery; commands below are delivery interfaces to implement, not claims they already exist. Each reference child extends the same versioned test runner, preserving prior families. At dispatch every prerequisite must have completed acceptance, review, protected merge and successful exact-main CI receipts.

For reference adapter changes run the actual pinned native/XMage family twice through `python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- python3 scripts/full_pool_reference.py --family spells --cache "$MTG_REFERENCE_CACHE" --output .agent-artifacts/spells-run-N.json` (N=1,2). Preserve inputs, actual consumed choices/chance, checkpoints, independent oracle, source/toolchain/dependency/bridge hashes, first-divergence negative controls and privileged artifact separation. Unsupported callbacks/observations fail explicitly; do not guess identities, inject derived state, default choices or report missing fields as agreement. All requested fields must have witnessed engine provenance; raw legal rejection, selected-action acceptance and complete legal-set comparison are distinct claims. Existing source/card/rules pins and compatibility tests remain unchanged; version new envelope semantics explicitly.

For instrumentation/profile/measurement changes, actual native runs prove four-mode semantic/privacy/capture equality; reuse pinned reference receipts for unchanged rules, run applicable affected real native/XMage comparisons twice with the same evidence contract if rules/semantic boundaries change. Do not manufacture a reference run for clock arithmetic or count unavailable hardware as zero.

After integrating current main run the full `python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- ./scripts/torture.sh`; all Cargo/Maven/reference/heavy commands use that existing shared lock. Commit clean and run the prescribed separate `python3 scripts/symphony/review.py origin/main` exact-candidate Codex review. Resolve blockers and retain review JSON, protected PR verify/merge and successful CI on the exact main SHA. Assess README impact, update supported commands/claims when affected, verify affected quickstart commands and require the independent reviewer to check README accuracy. No weakening/skips/fallbacks. No human routine approval.

Remain UNREADY until #268 reviewed registration merges and its exact-main CI passes. #268 is an explicit prerequisite, never self-satisfying by this text. Symphony implementation only, at most three workers, no off-dashboard implementation agent. Preserve holds/withdrawals and dependency controls. No task registration, RFC 0003, RL, source repins, workflow/CI/settings changes, new CLI/product surface or M2 completion claim. #24/#25 keep full original audits; #26 retains all ancestors; #241 owns later architecture/reference-AI corpus after verified #26. Reuse original audit workspaces/evidence; no deletion.

## #273 — Strict played creature mana and activated-ability transcript prefixes

Parent audit: #24. Registration: #268. Milestone M2; proposed until registration delivery.

## Observable deliverable

Strict played creature mana and activated-ability transcript prefixes.

| Contract | Acceptance |
| --- | --- |
| Owned interfaces | XMage activateAbility/playMana/chooseTarget for Elf/Druid/Cavalry/Shivan/Invoker; native delivered activation-payment semantic actions; source/ability/stack identities. |
| Runnable positive acceptance | Reach creatures/lands by normal play. Explicitly choose land/creature mana sources during Shivan/Invoker payment from empty pools as well as floating-first; immediate mana ability adds no stack/priority. Cavalry grants haste to sick Elf/Druid so tap mana becomes legal; Shivan resolves one R boost to 6/5, repeated boosts distinguish stack action IDs; Invoker selects Cub then pays eight and resolves to 7/7 trample. Observe targets, taps, reserved/private payment, commit/cancel, surplus mana, source incarnations and expiry metadata; stop before cleanup. |
| Runnable negative acceptance | Sick/tapped/enemy/departed mana source, wrong actor/target/color, insufficient eight mana, missing or reordered source/payment/target, accidental extra priority or default mana selection fails. Cancel at each stage preserves atomic semantics; swapped same-name sources and identical activation stack IDs are detected. |
| Independent oracle | #254 delivered rules/payment contract, CR 602/605, 302.6, 702.10 and exact pinned Oracle; GR-030. No sibling spell bridge needed: use nontargeted creature casting and selected activations only. |
| Prerequisites | #268, #271, #254 |
| Independent siblings | #272, #275, #278, #279, #280 (subject to their own prerequisites and three-worker cap) |
| Exclusions | Noncreature spell callbacks, trigger/combat choices, general ability language and complete-game admission. |

Focused acceptance: `python3 -m unittest discover -s tests -p 'test_m2_repair_activations.py'` plus full native normal discovery. The Python module must exercise the delivered real adapter/client, validate nonempty observations and run its negative controls; a parser-only pass cannot satisfy a played-prefix claim. The pinned reference runner family is `activations`, executed twice by the command below.

Partial additive owners: R0002-B010, R0002-B011, R0002-B026, R0002-B028, R0002-B030. All original owners and original acceptance remain.

## Mandatory delivery and authorization

One observable deliverable, one reviewed PR. Write behavioral/adapter tests FIRST and record an intended assertion failure (compile/import errors are not red evidence). Derive expected results independently from the pinned rules/card/requirements above; never bless actual engine output. Preserve every existing test and catalog case/owner/expectation; keep minimized seeds/tapes, new positive/negative controls and defect reproductions in normal discovery. Synthetic adapter probes supplement the required real normal-reset/played prefixes; no mock alternative rules engine or unimplemented sibling may be the only meaningful consumer.

Run `python3 scripts/run_tests.py` and `cargo test --workspace --locked` with the new cases discovered (record nonzero new test count). Add the focused test module named below and include it in normal discovery; commands below are delivery interfaces to implement, not claims they already exist. Each reference child extends the same versioned test runner, preserving prior families. At dispatch every prerequisite must have completed acceptance, review, protected merge and successful exact-main CI receipts.

For reference adapter changes run the actual pinned native/XMage family twice through `python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- python3 scripts/full_pool_reference.py --family activations --cache "$MTG_REFERENCE_CACHE" --output .agent-artifacts/activations-run-N.json` (N=1,2). Preserve inputs, actual consumed choices/chance, checkpoints, independent oracle, source/toolchain/dependency/bridge hashes, first-divergence negative controls and privileged artifact separation. Unsupported callbacks/observations fail explicitly; do not guess identities, inject derived state, default choices or report missing fields as agreement. All requested fields must have witnessed engine provenance; raw legal rejection, selected-action acceptance and complete legal-set comparison are distinct claims. Existing source/card/rules pins and compatibility tests remain unchanged; version new envelope semantics explicitly.

For instrumentation/profile/measurement changes, actual native runs prove four-mode semantic/privacy/capture equality; reuse pinned reference receipts for unchanged rules, run applicable affected real native/XMage comparisons twice with the same evidence contract if rules/semantic boundaries change. Do not manufacture a reference run for clock arithmetic or count unavailable hardware as zero.

After integrating current main run the full `python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- ./scripts/torture.sh`; all Cargo/Maven/reference/heavy commands use that existing shared lock. Commit clean and run the prescribed separate `python3 scripts/symphony/review.py origin/main` exact-candidate Codex review. Resolve blockers and retain review JSON, protected PR verify/merge and successful CI on the exact main SHA. Assess README impact, update supported commands/claims when affected, verify affected quickstart commands and require the independent reviewer to check README accuracy. No weakening/skips/fallbacks. No human routine approval.

Remain UNREADY until #268 reviewed registration merges and its exact-main CI passes. #268 is an explicit prerequisite, never self-satisfying by this text. Symphony implementation only, at most three workers, no off-dashboard implementation agent. Preserve holds/withdrawals and dependency controls. No task registration, RFC 0003, RL, source repins, workflow/CI/settings changes, new CLI/product surface or M2 completion claim. #24/#25 keep full original audits; #26 retains all ancestors; #241 owns later architecture/reference-AI corpus after verified #26. Reuse original audit workspaces/evidence; no deletion.

## #274 — Strict played cast and ETB trigger ordering and source identities

Parent audit: #24. Registration: #268. Milestone M2; proposed until registration delivery.

## Observable deliverable

Strict played cast and ETB trigger ordering and source identities.

| Contract | Acceptance |
| --- | --- |
| Owned interfaces | XMage chooseTriggeredAbility and ETB chooseTarget; pending/stack trigger canonical keys (source incarnation, ability, event ordinal); trigger-pending versus settled checkpoints. |
| Runnable positive acceptance | Normal-reset prefixes cast Archer/Cyclops and a noncreature spell: observe cast trigger before resolution, explicit ordering of two same-controller triggers, separate source/event IDs for identical Archers and pending versus placed states. Cast Pyromancer: no cast target, ETB player target chosen after creature resolution, damage two on trigger resolution. Respond with legally played Bite to kill source while ability remains on stack; retain source incarnation/LKI and resolve correctly. Native APNAP integration and applicable real reference APNAP scenario retained with explicit synthetic provenance if unreachable in frozen pool; never inject triggers into a claimed played prefix. |
| Runnable negative acceptance | Failed cast produces no cast trigger; creature/land/mana does not spuriously trigger Archer. Missing/reordered/extra trigger choice, wrong source/event, ETB target at casting time, creature as Pyromancer target and source identity rebound after death fail. Unsupported callback fails closed; mutate pending/settled alignment and trigger stack order. |
| Independent oracle | CR 603.2/603.3, 101.4, 113.7a, 601.2i and pinned Archer/Cyclops/Pyromancer; #211 retains existing APNAP and cleanup scenarios. GR-031. |
| Prerequisites | #268, #272, #211 |
| Independent siblings | #273, #275, #276, #278, #279, #280 (subject to their own prerequisites and three-worker cap) |
| Exclusions | Inventing reachable simultaneous events for an impossible pool combination, combat, terminal-game admission or general trigger language. |

Focused acceptance: `python3 -m unittest discover -s tests -p 'test_m2_repair_triggers.py'` plus full native normal discovery. The Python module must exercise the delivered real adapter/client, validate nonempty observations and run its negative controls; a parser-only pass cannot satisfy a played-prefix claim. The pinned reference runner family is `triggers`, executed twice by the command below.

Partial additive owners: R0002-B010, R0002-B011, R0002-B026, R0002-B028, R0002-B030. All original owners and original acceptance remain.

## Mandatory delivery and authorization

One observable deliverable, one reviewed PR. Write behavioral/adapter tests FIRST and record an intended assertion failure (compile/import errors are not red evidence). Derive expected results independently from the pinned rules/card/requirements above; never bless actual engine output. Preserve every existing test and catalog case/owner/expectation; keep minimized seeds/tapes, new positive/negative controls and defect reproductions in normal discovery. Synthetic adapter probes supplement the required real normal-reset/played prefixes; no mock alternative rules engine or unimplemented sibling may be the only meaningful consumer.

Run `python3 scripts/run_tests.py` and `cargo test --workspace --locked` with the new cases discovered (record nonzero new test count). Add the focused test module named below and include it in normal discovery; commands below are delivery interfaces to implement, not claims they already exist. Each reference child extends the same versioned test runner, preserving prior families. At dispatch every prerequisite must have completed acceptance, review, protected merge and successful exact-main CI receipts.

For reference adapter changes run the actual pinned native/XMage family twice through `python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- python3 scripts/full_pool_reference.py --family triggers --cache "$MTG_REFERENCE_CACHE" --output .agent-artifacts/triggers-run-N.json` (N=1,2). Preserve inputs, actual consumed choices/chance, checkpoints, independent oracle, source/toolchain/dependency/bridge hashes, first-divergence negative controls and privileged artifact separation. Unsupported callbacks/observations fail explicitly; do not guess identities, inject derived state, default choices or report missing fields as agreement. All requested fields must have witnessed engine provenance; raw legal rejection, selected-action acceptance and complete legal-set comparison are distinct claims. Existing source/card/rules pins and compatibility tests remain unchanged; version new envelope semantics explicitly.

For instrumentation/profile/measurement changes, actual native runs prove four-mode semantic/privacy/capture equality; reuse pinned reference receipts for unchanged rules, run applicable affected real native/XMage comparisons twice with the same evidence contract if rules/semantic boundaries change. Do not manufacture a reference run for clock arithmetic or count unavailable hardware as zero.

After integrating current main run the full `python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- ./scripts/torture.sh`; all Cargo/Maven/reference/heavy commands use that existing shared lock. Commit clean and run the prescribed separate `python3 scripts/symphony/review.py origin/main` exact-candidate Codex review. Resolve blockers and retain review JSON, protected PR verify/merge and successful CI on the exact main SHA. Assess README impact, update supported commands/claims when affected, verify affected quickstart commands and require the independent reviewer to check README accuracy. No weakening/skips/fallbacks. No human routine approval.

Remain UNREADY until #268 reviewed registration merges and its exact-main CI passes. #268 is an explicit prerequisite, never self-satisfying by this text. Symphony implementation only, at most three workers, no off-dashboard implementation agent. Preserve holds/withdrawals and dependency controls. No task registration, RFC 0003, RL, source repins, workflow/CI/settings changes, new CLI/product surface or M2 completion claim. #24/#25 keep full original audits; #26 retains all ancestors; #241 owns later architecture/reference-AI corpus after verified #26. Reuse original audit workspaces/evidence; no deletion.

## #275 — Strict played attackers, blockers and modern damage-choice transcripts

Parent audit: #24. Registration: #268. Milestone M2; proposed until registration delivery.

## Observable deliverable

Strict played attackers, blockers and modern damage-choice transcripts.

| Contract | Acceptance |
| --- | --- |
| Owned interfaces | XMage selectAttackers/selectBlockers/getMultiAmountWithIndividualConstraints, native combat semantic actions; attack/block/damage checkpoint extension with occurrence and incarnation IDs. |
| Runnable positive acceptance | Play frozen creatures from full-deck reset then script empty/nonempty attacks, blockers, multiple blockers and explicit damage allocations. Exercise Shivan flying versus Cub/Sentry reach; Pathwarden haste/vigilance/trample and Thornweald deathtouch/reach; no ability activation needed. Compare legal declarations, tapped/sickness, attacker/blocker links, each chosen allocation, simultaneous marked damage then SBA zone changes and source identity. Use literal independently justified power/toughness/lethal allocation ledgers. Every callback consumes explicit data; no obsolete damage assignment order or response window. |
| Runnable negative acceptance | Wrong actor, sick attacker, tapped/duplicate/enemy blocker, nonreach block of flyer, excess/negative/nonlethal-before-trample damage, wrong total, stale identity, reordered/missing/extra declaration/allocation or legacy blocker-order callback rejected. Mutated unblocked/blocked status, life, simultaneous damage or library/zone identities detected. |
| Independent oracle | Pinned modern CR 508-510, 702.2/9/10/17/19/20 and #210 rules-vintage reviewed cases. GR-032. Old upstream combat behavior must be adjudicated, not adopted as oracle. |
| Prerequisites | #268, #271, #210 |
| Independent siblings | #272, #273, #278, #279, #280 (subject to their own prerequisites and three-worker cap) |
| Exclusions | Activated buffs, spell response implementation, trigger ordering, cleanup and complete games; uses existing nontriggering creature-cast path only. |

Focused acceptance: `python3 -m unittest discover -s tests -p 'test_m2_repair_combat.py'` plus full native normal discovery. The Python module must exercise the delivered real adapter/client, validate nonempty observations and run its negative controls; a parser-only pass cannot satisfy a played-prefix claim. The pinned reference runner family is `combat`, executed twice by the command below.

Partial additive owners: R0002-B010, R0002-B011, R0002-B026, R0002-B028, R0002-B030. All original owners and original acceptance remain.

## Mandatory delivery and authorization

One observable deliverable, one reviewed PR. Write behavioral/adapter tests FIRST and record an intended assertion failure (compile/import errors are not red evidence). Derive expected results independently from the pinned rules/card/requirements above; never bless actual engine output. Preserve every existing test and catalog case/owner/expectation; keep minimized seeds/tapes, new positive/negative controls and defect reproductions in normal discovery. Synthetic adapter probes supplement the required real normal-reset/played prefixes; no mock alternative rules engine or unimplemented sibling may be the only meaningful consumer.

Run `python3 scripts/run_tests.py` and `cargo test --workspace --locked` with the new cases discovered (record nonzero new test count). Add the focused test module named below and include it in normal discovery; commands below are delivery interfaces to implement, not claims they already exist. Each reference child extends the same versioned test runner, preserving prior families. At dispatch every prerequisite must have completed acceptance, review, protected merge and successful exact-main CI receipts.

For reference adapter changes run the actual pinned native/XMage family twice through `python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- python3 scripts/full_pool_reference.py --family combat --cache "$MTG_REFERENCE_CACHE" --output .agent-artifacts/combat-run-N.json` (N=1,2). Preserve inputs, actual consumed choices/chance, checkpoints, independent oracle, source/toolchain/dependency/bridge hashes, first-divergence negative controls and privileged artifact separation. Unsupported callbacks/observations fail explicitly; do not guess identities, inject derived state, default choices or report missing fields as agreement. All requested fields must have witnessed engine provenance; raw legal rejection, selected-action acceptance and complete legal-set comparison are distinct claims. Existing source/card/rules pins and compatibility tests remain unchanged; version new envelope semantics explicitly.

For instrumentation/profile/measurement changes, actual native runs prove four-mode semantic/privacy/capture equality; reuse pinned reference receipts for unchanged rules, run applicable affected real native/XMage comparisons twice with the same evidence contract if rules/semantic boundaries change. Do not manufacture a reference run for clock arithmetic or count unavailable hardware as zero.

After integrating current main run the full `python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- ./scripts/torture.sh`; all Cargo/Maven/reference/heavy commands use that existing shared lock. Commit clean and run the prescribed separate `python3 scripts/symphony/review.py origin/main` exact-candidate Codex review. Resolve blockers and retain review JSON, protected PR verify/merge and successful CI on the exact main SHA. Assess README impact, update supported commands/claims when affected, verify affected quickstart commands and require the independent reviewer to check README accuracy. No weakening/skips/fallbacks. No human routine approval.

Remain UNREADY until #268 reviewed registration merges and its exact-main CI passes. #268 is an explicit prerequisite, never self-satisfying by this text. Symphony implementation only, at most three workers, no off-dashboard implementation agent. Preserve holds/withdrawals and dependency controls. No task registration, RFC 0003, RL, source repins, workflow/CI/settings changes, new CLI/product surface or M2 completion claim. #24/#25 keep full original audits; #26 retains all ancestors; #241 owns later architecture/reference-AI corpus after verified #26. Reuse original audit workspaces/evidence; no deletion.

## #276 — Strict played cleanup, discard and rules-terminal checkpoints

Parent audit: #24. Registration: #268. Milestone M2; proposed until registration delivery.

## Observable deliverable

Strict played cleanup, discard and rules-terminal checkpoints.

| Contract | Acceptance |
| --- | --- |
| Owned interfaces | XMage cleanup choose/chooseTarget and settled-step/end-game observations; native cleanup/outcome replay checkpoints; terminal accounting and explicit end-of-tape enforcement. |
| Runnable positive acceptance | Normal-reset prefixes reach hand-size discard with explicit physical choices and ordering, expiration of stacked Growth/Surprise with damage removed atomically, and additional cleanup processing where applicable. Play an empty-library loss from real 40-card reset with explicit draws/passes/discards, both starters; compare unsuccessful draw versus merely empty library and actual winner/reason. Read end-game result even when no next priority callback occurs. Retain #257 real synthetic simultaneous-loss evidence without claiming that setup is a reachable full game. Finish requires every event/choice consumed; concession test separately labeled. |
| Runnable negative acceptance | Missing/extra/reordered discard, wrong cardinality/actor/incarnation, premature terminal, omitted final checkpoint, unused tape suffix, limit/concession mislabeled natural completion, intermediate lethal check between simultaneous cleanup operations and missing additional cleanup observation fail. |
| Independent oracle | CR 514, 704, 104/121 and independent card duration/damage ledgers; #257 simultaneous-loss contract. GR-033; final reward mapping remains native API with independent outcome-derived expectations. |
| Prerequisites | #268, #272, #257 |
| Independent siblings | #274, #273, #275, #278, #279, #280 (subject to their own prerequisites and three-worker cap) |
| Exclusions | New combat/trigger callback implementation, broad campaign or dual-reference admission; missing required cleanup support blocks this delivery. |

Focused acceptance: `python3 -m unittest discover -s tests -p 'test_m2_repair_cleanup.py'` plus full native normal discovery. The Python module must exercise the delivered real adapter/client, validate nonempty observations and run its negative controls; a parser-only pass cannot satisfy a played-prefix claim. The pinned reference runner family is `cleanup`, executed twice by the command below.

Partial additive owners: R0002-B010, R0002-B011, R0002-B026, R0002-B028, R0002-B030. All original owners and original acceptance remain.

## Mandatory delivery and authorization

One observable deliverable, one reviewed PR. Write behavioral/adapter tests FIRST and record an intended assertion failure (compile/import errors are not red evidence). Derive expected results independently from the pinned rules/card/requirements above; never bless actual engine output. Preserve every existing test and catalog case/owner/expectation; keep minimized seeds/tapes, new positive/negative controls and defect reproductions in normal discovery. Synthetic adapter probes supplement the required real normal-reset/played prefixes; no mock alternative rules engine or unimplemented sibling may be the only meaningful consumer.

Run `python3 scripts/run_tests.py` and `cargo test --workspace --locked` with the new cases discovered (record nonzero new test count). Add the focused test module named below and include it in normal discovery; commands below are delivery interfaces to implement, not claims they already exist. Each reference child extends the same versioned test runner, preserving prior families. At dispatch every prerequisite must have completed acceptance, review, protected merge and successful exact-main CI receipts.

For reference adapter changes run the actual pinned native/XMage family twice through `python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- python3 scripts/full_pool_reference.py --family cleanup --cache "$MTG_REFERENCE_CACHE" --output .agent-artifacts/cleanup-run-N.json` (N=1,2). Preserve inputs, actual consumed choices/chance, checkpoints, independent oracle, source/toolchain/dependency/bridge hashes, first-divergence negative controls and privileged artifact separation. Unsupported callbacks/observations fail explicitly; do not guess identities, inject derived state, default choices or report missing fields as agreement. All requested fields must have witnessed engine provenance; raw legal rejection, selected-action acceptance and complete legal-set comparison are distinct claims. Existing source/card/rules pins and compatibility tests remain unchanged; version new envelope semantics explicitly.

For instrumentation/profile/measurement changes, actual native runs prove four-mode semantic/privacy/capture equality; reuse pinned reference receipts for unchanged rules, run applicable affected real native/XMage comparisons twice with the same evidence contract if rules/semantic boundaries change. Do not manufacture a reference run for clock arithmetic or count unavailable hardware as zero.

After integrating current main run the full `python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- ./scripts/torture.sh`; all Cargo/Maven/reference/heavy commands use that existing shared lock. Commit clean and run the prescribed separate `python3 scripts/symphony/review.py origin/main` exact-candidate Codex review. Resolve blockers and retain review JSON, protected PR verify/merge and successful CI on the exact main SHA. Assess README impact, update supported commands/claims when affected, verify affected quickstart commands and require the independent reviewer to check README accuracy. No weakening/skips/fallbacks. No human routine approval.

Remain UNREADY until #268 reviewed registration merges and its exact-main CI passes. #268 is an explicit prerequisite, never self-satisfying by this text. Symphony implementation only, at most three workers, no off-dashboard implementation agent. Preserve holds/withdrawals and dependency controls. No task registration, RFC 0003, RL, source repins, workflow/CI/settings changes, new CLI/product surface or M2 completion claim. #24/#25 keep full original audits; #26 retains all ancestors; #241 owns later architecture/reference-AI corpus after verified #26. Reuse original audit workspaces/evidence; no deletion.

## #277 — Admit the initial M2 native/XMage normal-reset terminal-game set

Parent audit: #24. Registration: #268. Milestone M2; proposed until registration delivery.

## Observable deliverable

Admit the initial M2 native/XMage normal-reset terminal-game set.

| Contract | Acceptance |
| --- | --- |
| Owned interfaces | Frozen normal-game tapes, admission manifest, per-game first-divergence/consumption/checkpoint verifier and ordinary discovery regression runner using delivered adapters. |
| Runnable positive acceptance | Admit at least two distinct nonconcession rules-terminal native/XMage games per RG/GR/RR/GG starting-seat row (16 total): one deterministic heuristic-origin and one independently authored coverage-directed path per row, recording provenance rather than running AI during replay. Run all admitted tapes twice in both actual engines. Full 40-card multisets, strict initial/mulligan permutations, choices, copy/incarnation/token/stack IDs and every required intermediate field are mandatory. All twenty card behaviors across the set plus reviewed prefixes, both Surprise modes, payment/trigger/combat/cleanup interactions and both starters; count played behavior, not cards in library. Freeze inputs and rules-derived checkpoints before candidate comparison. Preserve every attempted/duplicate/failed/truncated/unsupported/disputed game and first diff; no outcome-only comparison. |
| Runnable negative acceptance | Normal discovery must reject altered duplicate identity, missing/extra/reordered action/chance/checkpoint, wrong target/payment/damage, mismatched pin, forged outcome, concession/timeout as quota filler and truncated replay. Missing required field blocks admission even if winner agrees. Report selected-action legality versus full legal-set observability separately; native privacy/capture/snapshot tests stay mandatory. |
| Independent oracle | GR-040 initial two-engine subset of retained 16-game technical floor; frozen rules/card-derived authored expectations and independent reviewer, never captured engine output promoted to golden. Later all-three-engine qualification belongs to #241 transition, with historical #38/#40 obligations retained. #24 still owns >=100 distinct scenarios, twelve families and final per-capability aggregate. |
| Prerequisites | #268, #273, #274, #275, #276 |
| Independent siblings | #281 (subject to their own prerequisites and three-worker cap) |
| Exclusions | Forge/dual qualification, existing-reference-AI generation, later 10-game pilots/100-attempt campaigns/1000-game corpus, M2 verdict or scenario audit completion. |

Focused acceptance: `python3 -m unittest discover -s tests -p 'test_m2_repair_admission.py'` plus full native normal discovery. The Python module must exercise the delivered real adapter/client, validate nonempty observations and run its negative controls; a parser-only pass cannot satisfy a played-prefix claim. The pinned reference runner family is `admission`, executed twice by the command below.

Partial additive owners: R0002-B010, R0002-B011, R0002-B026, R0002-B028, R0002-B030. All original owners and original acceptance remain.

## Mandatory delivery and authorization

One observable deliverable, one reviewed PR. Write behavioral/adapter tests FIRST and record an intended assertion failure (compile/import errors are not red evidence). Derive expected results independently from the pinned rules/card/requirements above; never bless actual engine output. Preserve every existing test and catalog case/owner/expectation; keep minimized seeds/tapes, new positive/negative controls and defect reproductions in normal discovery. Synthetic adapter probes supplement the required real normal-reset/played prefixes; no mock alternative rules engine or unimplemented sibling may be the only meaningful consumer.

Run `python3 scripts/run_tests.py` and `cargo test --workspace --locked` with the new cases discovered (record nonzero new test count). Add the focused test module named below and include it in normal discovery; commands below are delivery interfaces to implement, not claims they already exist. Each reference child extends the same versioned test runner, preserving prior families. At dispatch every prerequisite must have completed acceptance, review, protected merge and successful exact-main CI receipts.

For reference adapter changes run the actual pinned native/XMage family twice through `python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- python3 scripts/full_pool_reference.py --family admission --cache "$MTG_REFERENCE_CACHE" --output .agent-artifacts/admission-run-N.json` (N=1,2). Preserve inputs, actual consumed choices/chance, checkpoints, independent oracle, source/toolchain/dependency/bridge hashes, first-divergence negative controls and privileged artifact separation. Unsupported callbacks/observations fail explicitly; do not guess identities, inject derived state, default choices or report missing fields as agreement. All requested fields must have witnessed engine provenance; raw legal rejection, selected-action acceptance and complete legal-set comparison are distinct claims. Existing source/card/rules pins and compatibility tests remain unchanged; version new envelope semantics explicitly.

For instrumentation/profile/measurement changes, actual native runs prove four-mode semantic/privacy/capture equality; reuse pinned reference receipts for unchanged rules, run applicable affected real native/XMage comparisons twice with the same evidence contract if rules/semantic boundaries change. Do not manufacture a reference run for clock arithmetic or count unavailable hardware as zero.

After integrating current main run the full `python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- ./scripts/torture.sh`; all Cargo/Maven/reference/heavy commands use that existing shared lock. Commit clean and run the prescribed separate `python3 scripts/symphony/review.py origin/main` exact-candidate Codex review. Resolve blockers and retain review JSON, protected PR verify/merge and successful CI on the exact main SHA. Assess README impact, update supported commands/claims when affected, verify affected quickstart commands and require the independent reviewer to check README accuracy. No weakening/skips/fallbacks. No human routine approval.

Remain UNREADY until #268 reviewed registration merges and its exact-main CI passes. #268 is an explicit prerequisite, never self-satisfying by this text. Symphony implementation only, at most three workers, no off-dashboard implementation agent. Preserve holds/withdrawals and dependency controls. No task registration, RFC 0003, RL, source repins, workflow/CI/settings changes, new CLI/product surface or M2 completion claim. #24/#25 keep full original audits; #26 retains all ancestors; #241 owns later architecture/reference-AI corpus after verified #26. Reuse original audit workspaces/evidence; no deletion.

## #278 — Bounded scalar sampled latency collection and summaries

Parent audit: #25. Registration: #268. Milestone M2; proposed until registration delivery.

## Observable deliverable

Bounded scalar sampled latency collection and summaries.

| Contract | Acceptance |
| --- | --- |
| Owned interfaces | Boundary-owned sampler/histogram in mtg-cli native::run_instrumented and collector summaries; pure fixed counters/buckets as needed in metrics.rs; existing PolicyTiming injected clock is the real consumer. |
| Runnable positive acceptance | Tests first inject exact clock sequence through actual scalar/collector execution: fixed deterministic sampling schedule, fixed labels/buckets, bucket-edge membership, count/sum and p50/p95/p99 bucket bounds, reset and local merge, saturation/overflow/error flags. Sample reset, transition/application, legality/view, policy, encoding and finalization separately; distinguish unavailable/not_measured/not_applicable from zero and preserve skipped-sample denominator. Off creates no optional summaries or extra performance-clock calls; counters and trace/replay get declared bounded timings. Normal full-deck executions compare semantic state/RNG/actions/records and capture on/off across all four modes with enabled/disabled encoding. |
| Runnable negative acceptance | Reject invalid sample interval/buckets/config, nonmonotonic clock and malformed merges, report overflow and error attempts without wrapped/negative durations. Hidden sentinels never appear in public labels; no per-game/seed/card label. Sample policy and encoding with different literal durations and prove they cannot be folded into rules time; no extra game RNG draws or clock inside rules. |
| Independent oracle | RFC B008 decision latency and B021 sampled histograms; hand-authored fake clock arithmetic, fixed bucket tables and unchanged semantic transcripts. Real native execution is the consumer; no schema-only deliverable. |
| Prerequisites | #268, #214, #215, #216 |
| Independent siblings | #269, #279, #280 (subject to their own prerequisites and three-worker cap) |
| Exclusions | New benchmark workload version, measurement campaign, per-small-rule-operation clocks, worker/batch/inference/Python/CLI scope or exporter service. |

Focused acceptance: `python3 -m unittest discover -s tests -p 'test_m2_repair_timing.py'` plus full native normal discovery. The Python module must exercise the delivered real adapter/client, validate nonempty observations and run its negative controls; a parser-only pass cannot satisfy a played-prefix claim. Add injected-clock/native execution cases in the existing Rust test suites as well; the Python focused module validates real exported artifacts and rejects tampered accounting.

Partial additive owners: R0002-B008, R0002-B020, R0002-B021. All original owners and original acceptance remain.

## Mandatory delivery and authorization

One observable deliverable, one reviewed PR. Write behavioral/adapter tests FIRST and record an intended assertion failure (compile/import errors are not red evidence). Derive expected results independently from the pinned rules/card/requirements above; never bless actual engine output. Preserve every existing test and catalog case/owner/expectation; keep minimized seeds/tapes, new positive/negative controls and defect reproductions in normal discovery. Synthetic adapter probes supplement the required real normal-reset/played prefixes; no mock alternative rules engine or unimplemented sibling may be the only meaningful consumer.

Run `python3 scripts/run_tests.py` and `cargo test --workspace --locked` with the new cases discovered (record nonzero new test count). Add the focused test module named below and include it in normal discovery; commands below are delivery interfaces to implement, not claims they already exist. Each reference child extends the same versioned test runner, preserving prior families. At dispatch every prerequisite must have completed acceptance, review, protected merge and successful exact-main CI receipts.

For reference adapter changes run the actual pinned native/XMage family twice through `python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- python3 scripts/full_pool_reference.py --family timing --cache "$MTG_REFERENCE_CACHE" --output .agent-artifacts/timing-run-N.json` (N=1,2). Preserve inputs, actual consumed choices/chance, checkpoints, independent oracle, source/toolchain/dependency/bridge hashes, first-divergence negative controls and privileged artifact separation. Unsupported callbacks/observations fail explicitly; do not guess identities, inject derived state, default choices or report missing fields as agreement. All requested fields must have witnessed engine provenance; raw legal rejection, selected-action acceptance and complete legal-set comparison are distinct claims. Existing source/card/rules pins and compatibility tests remain unchanged; version new envelope semantics explicitly.

For instrumentation/profile/measurement changes, actual native runs prove four-mode semantic/privacy/capture equality; reuse pinned reference receipts for unchanged rules, run applicable affected real native/XMage comparisons twice with the same evidence contract if rules/semantic boundaries change. Do not manufacture a reference run for clock arithmetic or count unavailable hardware as zero.

After integrating current main run the full `python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- ./scripts/torture.sh`; all Cargo/Maven/reference/heavy commands use that existing shared lock. Commit clean and run the prescribed separate `python3 scripts/symphony/review.py origin/main` exact-candidate Codex review. Resolve blockers and retain review JSON, protected PR verify/merge and successful CI on the exact main SHA. Assess README impact, update supported commands/claims when affected, verify affected quickstart commands and require the independent reviewer to check README accuracy. No weakening/skips/fallbacks. No human routine approval.

Remain UNREADY until #268 reviewed registration merges and its exact-main CI passes. #268 is an explicit prerequisite, never self-satisfying by this text. Symphony implementation only, at most three workers, no off-dashboard implementation agent. Preserve holds/withdrawals and dependency controls. No task registration, RFC 0003, RL, source repins, workflow/CI/settings changes, new CLI/product surface or M2 completion claim. #24/#25 keep full original audits; #26 retains all ancestors; #241 owns later architecture/reference-AI corpus after verified #26. Reuse original audit workspaces/evidence; no deletion.

## #279 — Versioned four-mode scalar benchmark execution contract

Parent audit: #25. Registration: #268. Milestone M2; proposed until registration delivery.

## Observable deliverable

Versioned four-mode scalar benchmark execution contract.

| Contract | Acceptance |
| --- | --- |
| Owned interfaces | New workload/schema path in benchmark.rs plus collector/artifact validation; existing native Config.trace, Driver diagnostics and replay retrieval. Historical scalar-windows-v1/scalar-full-pool-v1 behavior stays byte-contract compatible. |
| Runnable positive acceptance | Freeze new named workload version accepting off/counters/sampled_trace/full_replay and run actual native normal-reset eight-row workloads using existing modes. Freeze equal seeds/policies/limits, trace sample interval/capacity, drops, replay byte/record capacity, completeness and persistence semantics before measurements. Default comparable throughput contract retains complete replay bytes and serializes/finalizes within timer, explicitly names in-memory versus durable persistence; requested canonical capture remains independently configurable, bounded and validated. Compare same finite semantic scripts/seeded episodes across four modes and capture settings; report actual trace selection/drop and complete replay verification. Inject clocks only for denominator accounting, not substitute game engine; every reset/failure/finalization/overshoot/unfinished/pre-reset attempt remains in denominator. |
| Runnable negative acceptance | Old workload versions still reject sampled_trace/full_replay with unchanged tests. New version rejects incompatible trace config, invalid capacities, incomplete replay, unaccounted failed/unfinished attempts and unsupported persistence configuration; forced capacity/write failure is visible, never fast success. Artifact validator detects missing rows/mode/version/denominators. Independently review new expectations against B020/B021 before implementation. |
| Independent oracle | RFC B020 windows/accounting and B021 four modes; delivered #215 trace/replay contracts and #216 injected-clock hand-counted window ledger. Independent of sampled histograms: timing availability honestly not_measured until that child delivers; campaign depends on both. |
| Prerequisites | #268, #215, #216, #208 |
| Independent siblings | #269, #278, #280 (subject to their own prerequisites and three-worker cap) |
| Exclusions | Sampled histogram implementation, campaign, profiling/optimization, changes to frozen v1 rejections, batch/Python/RL or new CLI surface. |

Focused acceptance: `python3 -m unittest discover -s tests -p 'test_m2_repair_modes.py'` plus full native normal discovery. The Python module must exercise the delivered real adapter/client, validate nonempty observations and run its negative controls; a parser-only pass cannot satisfy a played-prefix claim. Add injected-clock/native execution cases in the existing Rust test suites as well; the Python focused module validates real exported artifacts and rejects tampered accounting.

Partial additive owners: R0002-B008, R0002-B019, R0002-B020, R0002-B021. All original owners and original acceptance remain.

## Mandatory delivery and authorization

One observable deliverable, one reviewed PR. Write behavioral/adapter tests FIRST and record an intended assertion failure (compile/import errors are not red evidence). Derive expected results independently from the pinned rules/card/requirements above; never bless actual engine output. Preserve every existing test and catalog case/owner/expectation; keep minimized seeds/tapes, new positive/negative controls and defect reproductions in normal discovery. Synthetic adapter probes supplement the required real normal-reset/played prefixes; no mock alternative rules engine or unimplemented sibling may be the only meaningful consumer.

Run `python3 scripts/run_tests.py` and `cargo test --workspace --locked` with the new cases discovered (record nonzero new test count). Add the focused test module named below and include it in normal discovery; commands below are delivery interfaces to implement, not claims they already exist. Each reference child extends the same versioned test runner, preserving prior families. At dispatch every prerequisite must have completed acceptance, review, protected merge and successful exact-main CI receipts.

For reference adapter changes run the actual pinned native/XMage family twice through `python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- python3 scripts/full_pool_reference.py --family modes --cache "$MTG_REFERENCE_CACHE" --output .agent-artifacts/modes-run-N.json` (N=1,2). Preserve inputs, actual consumed choices/chance, checkpoints, independent oracle, source/toolchain/dependency/bridge hashes, first-divergence negative controls and privileged artifact separation. Unsupported callbacks/observations fail explicitly; do not guess identities, inject derived state, default choices or report missing fields as agreement. All requested fields must have witnessed engine provenance; raw legal rejection, selected-action acceptance and complete legal-set comparison are distinct claims. Existing source/card/rules pins and compatibility tests remain unchanged; version new envelope semantics explicitly.

For instrumentation/profile/measurement changes, actual native runs prove four-mode semantic/privacy/capture equality; reuse pinned reference receipts for unchanged rules, run applicable affected real native/XMage comparisons twice with the same evidence contract if rules/semantic boundaries change. Do not manufacture a reference run for clock arithmetic or count unavailable hardware as zero.

After integrating current main run the full `python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- ./scripts/torture.sh`; all Cargo/Maven/reference/heavy commands use that existing shared lock. Commit clean and run the prescribed separate `python3 scripts/symphony/review.py origin/main` exact-candidate Codex review. Resolve blockers and retain review JSON, protected PR verify/merge and successful CI on the exact main SHA. Assess README impact, update supported commands/claims when affected, verify affected quickstart commands and require the independent reviewer to check README accuracy. No weakening/skips/fallbacks. No human routine approval.

Remain UNREADY until #268 reviewed registration merges and its exact-main CI passes. #268 is an explicit prerequisite, never self-satisfying by this text. Symphony implementation only, at most three workers, no off-dashboard implementation agent. Preserve holds/withdrawals and dependency controls. No task registration, RFC 0003, RL, source repins, workflow/CI/settings changes, new CLI/product surface or M2 completion claim. #24/#25 keep full original audits; #26 retains all ancestors; #241 owns later architecture/reference-AI corpus after verified #26. Reuse original audit workspaces/evidence; no deletion.

## #280 — Validated fixed-trace scalar application, legality and effect-dispatch profiling

Parent audit: #25. Registration: #268. Milestone M2; proposed until registration delivery.

## Observable deliverable

Validated fixed-trace scalar application, legality and effect-dispatch profiling.

| Contract | Acceptance |
| --- | --- |
| Owned interfaces | Native profiling runner around existing semantic actions, legal observation generation and bounded rules work; immutable trace inputs and allocation/CPU profile attribution. Clock ownership stays in scalar/profile client. |
| Runnable positive acceptance | Replay fixed independently validated priority/cast/trigger/combat/cleanup traces against actual engine; compare every expected checkpoint before measuring. Separately time application/replay and legal generation/view at declared nonoverlapping boundaries; explicitly profile effect-dispatch symbols/call stacks with counts/time or isolated boundary measurement, not infer effect cost from transition total. Include valid typed trace with repeated effects and no-effect control, plus encoded feature construction using existing PolicyTiming.encode; report shared/unattributable work separately. Inject clock arithmetic to test exact accounting and exclude policy/encoding from application; preserve trace, binary/source/toolchain hashes and allocation counts/bytes. |
| Runnable negative acceptance | Reject wrong action/target/checkpoint/pin, missing or extra trace suffix, unvalidated/unfinished trace and overlapping/missing cost categories. Mutation of one known checkpoint must fail correctness gate before timing. Missing profiler facility is explicit limitation; effect-dispatch measured evidence is still required via available in-process profiling or boundary attribution without clocks in rules. |
| Independent oracle | RFC B020 separate transition and legal-action tracks plus effect/allocation/encoding profile; independent rules-derived frozen trace expectations and exact injected durations. Existing native semantic executor and benchmark boundary hooks suffice without new modes/timing children. |
| Prerequisites | #268, #208, #216 |
| Independent siblings | #269, #278, #279 (subject to their own prerequisites and three-worker cap) |
| Exclusions | Broad benchmark campaign, rules optimization, batching/Python/RL/CLI, host/cache/synchronization qualification; no second rules engine or guessed effect cost. |

Focused acceptance: `python3 -m unittest discover -s tests -p 'test_m2_repair_profile.py'` plus full native normal discovery. The Python module must exercise the delivered real adapter/client, validate nonempty observations and run its negative controls; a parser-only pass cannot satisfy a played-prefix claim. Add injected-clock/native execution cases in the existing Rust test suites as well; the Python focused module validates real exported artifacts and rejects tampered accounting.

Partial additive owners: R0002-B008, R0002-B014, R0002-B020. All original owners and original acceptance remain.

## Mandatory delivery and authorization

One observable deliverable, one reviewed PR. Write behavioral/adapter tests FIRST and record an intended assertion failure (compile/import errors are not red evidence). Derive expected results independently from the pinned rules/card/requirements above; never bless actual engine output. Preserve every existing test and catalog case/owner/expectation; keep minimized seeds/tapes, new positive/negative controls and defect reproductions in normal discovery. Synthetic adapter probes supplement the required real normal-reset/played prefixes; no mock alternative rules engine or unimplemented sibling may be the only meaningful consumer.

Run `python3 scripts/run_tests.py` and `cargo test --workspace --locked` with the new cases discovered (record nonzero new test count). Add the focused test module named below and include it in normal discovery; commands below are delivery interfaces to implement, not claims they already exist. Each reference child extends the same versioned test runner, preserving prior families. At dispatch every prerequisite must have completed acceptance, review, protected merge and successful exact-main CI receipts.

For reference adapter changes run the actual pinned native/XMage family twice through `python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- python3 scripts/full_pool_reference.py --family profile --cache "$MTG_REFERENCE_CACHE" --output .agent-artifacts/profile-run-N.json` (N=1,2). Preserve inputs, actual consumed choices/chance, checkpoints, independent oracle, source/toolchain/dependency/bridge hashes, first-divergence negative controls and privileged artifact separation. Unsupported callbacks/observations fail explicitly; do not guess identities, inject derived state, default choices or report missing fields as agreement. All requested fields must have witnessed engine provenance; raw legal rejection, selected-action acceptance and complete legal-set comparison are distinct claims. Existing source/card/rules pins and compatibility tests remain unchanged; version new envelope semantics explicitly.

For instrumentation/profile/measurement changes, actual native runs prove four-mode semantic/privacy/capture equality; reuse pinned reference receipts for unchanged rules, run applicable affected real native/XMage comparisons twice with the same evidence contract if rules/semantic boundaries change. Do not manufacture a reference run for clock arithmetic or count unavailable hardware as zero.

After integrating current main run the full `python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- ./scripts/torture.sh`; all Cargo/Maven/reference/heavy commands use that existing shared lock. Commit clean and run the prescribed separate `python3 scripts/symphony/review.py origin/main` exact-candidate Codex review. Resolve blockers and retain review JSON, protected PR verify/merge and successful CI on the exact main SHA. Assess README impact, update supported commands/claims when affected, verify affected quickstart commands and require the independent reviewer to check README accuracy. No weakening/skips/fallbacks. No human routine approval.

Remain UNREADY until #268 reviewed registration merges and its exact-main CI passes. #268 is an explicit prerequisite, never self-satisfying by this text. Symphony implementation only, at most three workers, no off-dashboard implementation agent. Preserve holds/withdrawals and dependency controls. No task registration, RFC 0003, RL, source repins, workflow/CI/settings changes, new CLI/product surface or M2 completion claim. #24/#25 keep full original audits; #26 retains all ancestors; #241 owns later architecture/reference-AI corpus after verified #26. Reuse original audit workspaces/evidence; no deletion.

## #281 — Measure the delivered four-mode scalar and profile contracts

Parent audit: #25. Registration: #268. Milestone M2; proposed until registration delivery.

## Observable deliverable

Measure the delivered four-mode scalar and profile contracts.

| Contract | Acceptance |
| --- | --- |
| Owned interfaces | Reproducible campaign configuration, raw artifact collection/validation and scoped report using delivered timing/workload/profile runners. |
| Runnable positive acceptance | On exact correctness-gated candidate run all four modes with heuristic and legal-random policies across all eight rows. At least 10s warmup and >=5x30s windows per configuration, declared variance-extension rule frozen before runs; retain all extensions and failed/unfinished/pre-reset attempts, reset/finalization time, wins/draws/errors/truncations/concessions and full denominator. Publish comparable throughput/overhead distributions, sampled native latency bucket summaries, encoded versus unencoded track, and fixed validated trace/application/legal-generation/effect-dispatch/allocation evidence from delivered profile runner. Preserve binary/source/flags/affinity/memory/container limits and replay/trace/capture semantics; reproduce semantic equivalence on fixed episodes rather than infer from window totals. Retain historical #217 resident/stress evidence and rerun affected capacity measures if new buffers change claims. |
| Runnable negative acceptance | Artifact validation rejects absent mode/policy/row/windows, shortened horizon, denominator omitting failed work, incomplete replay, missing sampling/capacity/persistence pin, invalid trace or inferred effect/encoding cost. Unit tests with literal small data demonstrate rejection before collection; campaign failures stay published and never fill completed-game numerator. |
| Independent oracle | Frozen reviewed child contracts and RFC B008/B014/B019/B020/B021 arithmetic; independent review of statistics/denominators and unmeasured limits. Target misses are results, not reason to weaken requirements. |
| Prerequisites | #268, #278, #279, #280, #217 |
| Independent siblings | reference prefix repairs, #277 (subject to their own prerequisites and three-worker cap) |
| Exclusions | Any new collector/benchmark/profile feature; if missing contract remains, report bounded prerequisite gap. No optimization, designated-host demand, Python/batch/RL/CLI or claimed M2 completion. |

Focused acceptance: `python3 -m unittest discover -s tests -p 'test_m2_repair_measurement.py'` plus full native normal discovery. The Python module must exercise the delivered real adapter/client, validate nonempty observations and run its negative controls; a parser-only pass cannot satisfy a played-prefix claim. Add injected-clock/native execution cases in the existing Rust test suites as well; the Python focused module validates real exported artifacts and rejects tampered accounting.

Partial additive owners: R0002-B008, R0002-B014, R0002-B019, R0002-B020, R0002-B021. All original owners and original acceptance remain.

## Mandatory delivery and authorization

One observable deliverable, one reviewed PR. Write behavioral/adapter tests FIRST and record an intended assertion failure (compile/import errors are not red evidence). Derive expected results independently from the pinned rules/card/requirements above; never bless actual engine output. Preserve every existing test and catalog case/owner/expectation; keep minimized seeds/tapes, new positive/negative controls and defect reproductions in normal discovery. Synthetic adapter probes supplement the required real normal-reset/played prefixes; no mock alternative rules engine or unimplemented sibling may be the only meaningful consumer.

Run `python3 scripts/run_tests.py` and `cargo test --workspace --locked` with the new cases discovered (record nonzero new test count). Add the focused test module named below and include it in normal discovery; commands below are delivery interfaces to implement, not claims they already exist. Each reference child extends the same versioned test runner, preserving prior families. At dispatch every prerequisite must have completed acceptance, review, protected merge and successful exact-main CI receipts.

For reference adapter changes run the actual pinned native/XMage family twice through `python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- python3 scripts/full_pool_reference.py --family measurement --cache "$MTG_REFERENCE_CACHE" --output .agent-artifacts/measurement-run-N.json` (N=1,2). Preserve inputs, actual consumed choices/chance, checkpoints, independent oracle, source/toolchain/dependency/bridge hashes, first-divergence negative controls and privileged artifact separation. Unsupported callbacks/observations fail explicitly; do not guess identities, inject derived state, default choices or report missing fields as agreement. All requested fields must have witnessed engine provenance; raw legal rejection, selected-action acceptance and complete legal-set comparison are distinct claims. Existing source/card/rules pins and compatibility tests remain unchanged; version new envelope semantics explicitly.

For instrumentation/profile/measurement changes, actual native runs prove four-mode semantic/privacy/capture equality; reuse pinned reference receipts for unchanged rules, run applicable affected real native/XMage comparisons twice with the same evidence contract if rules/semantic boundaries change. Do not manufacture a reference run for clock arithmetic or count unavailable hardware as zero.

After integrating current main run the full `python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- ./scripts/torture.sh`; all Cargo/Maven/reference/heavy commands use that existing shared lock. Commit clean and run the prescribed separate `python3 scripts/symphony/review.py origin/main` exact-candidate Codex review. Resolve blockers and retain review JSON, protected PR verify/merge and successful CI on the exact main SHA. Assess README impact, update supported commands/claims when affected, verify affected quickstart commands and require the independent reviewer to check README accuracy. No weakening/skips/fallbacks. No human routine approval.

Remain UNREADY until #268 reviewed registration merges and its exact-main CI passes. #268 is an explicit prerequisite, never self-satisfying by this text. Symphony implementation only, at most three workers, no off-dashboard implementation agent. Preserve holds/withdrawals and dependency controls. No task registration, RFC 0003, RL, source repins, workflow/CI/settings changes, new CLI/product surface or M2 completion claim. #24/#25 keep full original audits; #26 retains all ancestors; #241 owns later architecture/reference-AI corpus after verified #26. Reuse original audit workspaces/evidence; no deletion.

