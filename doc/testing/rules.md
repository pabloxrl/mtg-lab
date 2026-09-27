# Independent rules/card test design

Status: **design only**, authored independently for the frozen `foundations_micro_v1` pool. [capability-test-plan.json](capability-test-plan.json) contains 320 planned records: one positive, negative, interaction and regression design for each of the actual 80 registry IDs. No game, fixture, reference engine, host build or behavioral test has been run. The generator only checks record completeness and uniqueness. These prose designs are not admitted neutral fixtures and must not be inserted into passing evidence arrays.

## Scope and ownership

The source inventory is `data/capabilities-v1.json`, both RFCs, `doc/neutral-scenarios.md`, `doc/card-manifests.md`, `doc/rules-source.md`, and the frozen card/rules manifests. The 20 named cards and red 1/1 Goblin token are the only card definitions used. Non-basic card behavior comes from pinned Oracle content, not assumptions from card names. The designs use synthetic positions unless explicitly called normal reset; a synthetic mix of red/green cards does not imply reachability from either fixed deck. No unsupported planeswalker, control-change, resurrection, counters, or old blocker-order mechanic is introduced.

Owner assignments name implementation/evidence tasks, not their later gates: GH-17 reset/decision contracts, GH-18 M1 scalar slice, GH-19 private views/snapshots, GH-20 rewards/trajectory integration, GH-23 complete M2 mechanics, GH-24 full-pool coverage of earlier interfaces, GH-27 batch consistency. An M1 interface exercised using M2 cards is deliberately staged at M2, rather than implying full-pool support in M1. M0 owns the reviewable design mapping; later owners still need behavioral red/green, admitted fixture provenance and reference executions. Issue dependencies and authorization must be checked at dispatch time; this plan creates no dispatch authorization.

## Source verification and expectation derivation

On 2026-09-27 a fresh official CR retrieval matched committed SHA-256 `8d860e451f20f38865b725b42d82feb714c725373dd8f3b32b8652b3eeb070ca`, source `https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt`. The downloaded source was inspected for the cited priority, casting/payment, damage, zone identity, cleanup, APNAP, target revalidation, mana ability, haste/vigilance/flying/reach/deathtouch/trample and terminal rules. Text remains outside the repository under `/tmp/mtg-rules-design/cr.txt`; it is not an artifact to distribute.

All 21 exact Scryfall printing endpoints from `data/cards/foundations_micro_v1.json` were fetched with curl and their `oracle_text` UTF-8 hashes matched the per-card manifest pins. The relevant text was read for derivation, not copied into a fixture corpus. Card-specific basis fields record exact endpoint and Oracle hash. These checks verify pinned-source consistency, not an independent official Oracle publication signature or engine agreement. An initial urllib Scryfall request returned HTTP 400; the subsequent curl requests all succeeded against the exact selected endpoints, without changing source pins.

Concrete traps reflected in expectations:

- CR 103.5 bottoms after every mulligan before the next keep/mulligan declaration; this source says so explicitly. Do not import a different procedural ordering from memory.
- CR 103.8a skips the starting player's first **draw step**, not both players' first draws or every first-player draw.
- CR 510.1c allows any distribution among multiple blockers. No blocker ordering or lethal-before-next-blocker constraint survives in this revision. Flying attackers use reach/flying blockers in positive scenarios.
- CR 608.2b denies information about an illegal target. Bite Down whose targeted source dies cannot deal cached last-known-power damage. This differs from an already-created Pyromancer trigger whose dead source still deals damage under CR 113.7a.
- CR 514.2 removes marked damage and expires boosts simultaneously. The grown, damaged Cub survives cleanup without an intermediate lethal check.
- Goblin Surprise chooses exactly one mode. Boost mode affects the set of creatures present at resolution; later tokens do not inherit it (CR 611.2c).
- Thrill spends itself and a discard card, then draws two: net hand change is zero. The repeat-cleanup regression deliberately tests **no redundant discard** after a seven-card hand casts/resolves Thrill.

RFC requirements, rather than Magic CR citations, govern determinism, private observations, stale actions, overflow, snapshots, batching and rewards. Registry family references are broad pointers; they are not automatically suitable numbered authority for each record. The per-case basis explains the actual governing rule/requirement.

## Fixture completion contract

A future author must turn each design into strict neutral fixture data, independently review its literal expectations and retain the source pins. Use stable unique object identities, explicit owners/controllers/control timestamps, life totals (20 unless specified), mana/source choices, ordered libraries, phase/step and priority. Unless a case tests the contrary, creatures described as attackers or tap-ability sources have been controlled since turn start; all stated blockers are untapped; stated nonlethal starting positions have no omitted damage. Unless a cost/timing failure is the point, provide exact sufficient mana derived from pinned card cost and legal priority. Constructor assumptions must enumerate these defaults; they are not an executable runner fallback.

Write every pass, target, payment, mode, discard, attacker/blocker assignment, trigger order and combat allocation explicitly. “Resolve” in this design means the author must insert the necessary legal consecutive passes and assert after each relevant resolution; it does not authorize auto-choice behavior. Tests must capture the stack and priority before and after responses, plus state-based processing and terminal transitions. Negative probes assert unchanged game state, RNG state, decision state and private information even where the compact record states only the concrete affected fields.

All regression records are **seeded defect risks**, not invented historical regressions. Their expected result indicates the mutation to guard against (cached power, skipped window, wrong ordering, ghost token, retained effect, duplicate reward, etc.). On discovery of an actual defect, add failing revision, minimal trace and independent red/green evidence; do not manufacture those provenance fields for this plan.

## Synthetic boundaries and unresolved admission details

- APNAP records inject already-pending, independently identified triggers for each player immediately before placement. The pool's ordinary cast/ETB triggers do not establish natural simultaneous cross-controller triggering. Each trigger's controller/source/event provenance must be explicitly assumed; these are scheduler unit fixtures, not reachable game evidence.
- Simultaneous-loss records inject a before-SBA state with both loss conditions present. Simultaneous player damage is not supplied by an invented card. A failed-draw marker is assumed only in that named synthetic test. Constructor/bridge support for this boundary must be explicit; otherwise report it unsupported and extend the test-only boundary transparently.
- Repeat-cleanup tests inject pending supported Pyromancer work at the cleanup boundary, explicitly synthetic. The current pool does not establish a normal cleanup-trigger source or a natural cleanup hand-growth effect. These tests prove scheduling semantics, not naturally reachable cleanup triggers. The first cleanup discards eight to seven where stated; a subsequent Thrill changes seven to six to five to seven, then the repeated cleanup discards nothing.
- Zone-identity regression uses a separately constructed new-generation same-card object only to test handle isolation; it does not add a reanimation/return spell. The dead-source ability tests use actual pool damage and activations.
- Neutral schema v1 may need an explicit test-only boundary or helper representation for pending trigger batches, failed-draw flags, partial casting continuations and before-SBA loss states. A schema acceptance test is not behavioral proof. Do not silently approximate these cases or certify them as reachable.
- Candidate enumeration expectations concern semantic combinations, not a required internal Cartesian array. The six power-five/two-blocker splits are `(0,5)` through `(5,0)`; token attack choices use independently calculated powers of two. Complete factorization or a diagnosed capacity error is acceptable under the declared runtime capacity contract; silently truncated successful output is not.
- A few shorthand negative commands (e.g. choosing a non-top draw or resolving a lower stack object) may be protocol-level invalid probes if the final action API intentionally never exposes such a choice. Preserve the behavioral prohibition and no-mutation result; do not invent a public command solely to satisfy the sketch.

## Review and execution obligations

Review the synthetic assumptions and expected arithmetic before admission. Freeze candidate source/card/bridge/engine revisions. GH-24 must achieve the full category matrix and at least 100 independently reviewed cases; 320 design slots are not a scenario-execution count. GH-38 retains the independent dual-reference critical suite and scripted full games across all eight fixed matchup/start-seat configurations. Compare normalized intermediate fields and observable semantic decisions; report unavailable/unobservable/disputed separately. Neither reference majority nor our engine output defines expectations. Preserve differences and adjudicate against these sources.

Manual design arithmetic review corrected flying-blocker eligibility in the multiple-blocker cases and the Thrill cleanup hand count. These corrections are planning review, not evidence of a production bug or passing test.
