# GH-23 mechanics integration audit — incomplete

Audited base: `7b7affc387d0464995661d15a39f732e7937e543`.
This is a **partial, unsuccessful audit**, not a delivered capability, passing
component, whole-block verdict or M2 gate. No production implementation changes
have been made. The [source inventory](audit.json) pins the inspected files.
The [proposed regression patch](activation-mana-red.patch) contains independently
specified tests. The [full-command log](torture-red.log) records compiled failures
at all three intended assertions: missing Shivan and Invoker activation candidates,
and no legal mana-source action inside activation payment. The core unit binary
reports 272 passed, three failed, zero ignored. `./scripts/torture.sh` exits 101
there; the release stage was not reached. This is behavioral red, not a complete
regression-suite pass. No production fix, green or final delivery is claimed.

## Finding: mana abilities during nonmana activation payment

RFC 0002 §3 explicitly includes creature mana abilities, activated abilities,
explicit payments and exact mana-ability timing. CR 602.2b applies the casting
procedure to activation; 601.2g and 605.3a permit mana abilities during payment.
These expectations follow the rules, not either engine's output. This audit
reviewed committed requirements and the pinned rules-source metadata; it did not
perform a fresh official rules-text retrieval.

The current engine supports floating mana at priority before announcing Shivan
or Invoker. It does not support producing mana during their activation payment:

- `activation.rs::usable_activation_source` requires already floating red or
  eight mana. An untapped Mountain with an empty pool cannot fund announcement
  of Shivan's ability; eight untapped Forests cannot fund Invoker announcement.
- `PendingActivation` stores reserved spending but no staged mana sources/pool.
  `begin_activation` clears the ordinary priority decision, so `tap_mana` cannot
  be used as a substitute inside the continuation.
- `policy.rs` offers pay/finish/cancel during `activation_payment`, never
  `TapMana`. Its dispatch only handles casting or ordinary priority mana taps.
- `mtg-policy/src/lib.rs` and `mtg-recorder/src/structured.rs` also reject
  `TapMana` in this decision kind. Merely enabling a candidate would not repair
  played policy/recording/replay paths.
- The Shivan and Invoker reference bridges float their mana at priority. Their
  existing receipts therefore do not establish this missing payment window.

Pre-floating these simple sources often produces the same completed state, but
does not represent every legal sequence or faithfully replay reference choices.
The child [Shivan](../shivan/README.md) and [Invoker](../invoker/README.md) reports
explicitly document the float-first boundary. Their narrower receipts remain
valid; they cannot establish the original component's full timing acceptance.

A separate read-only diagnostic review independently confirmed the finding and
the recorder restriction. It noted that one bounded repair is plausible and
that touching multiple files alone does not justify calling work broad. This is
not the prescribed final clean-candidate review and supplies no integration pass.

## Proposed coordinator split

Register one independently testable **activation-payment mana** capability
delivery before resuming this audit. It should own the complete continuation
contract, rather than treating a policy-mask change as sufficient:

1. Allow affordable announcements using legal land/creature sources, then stage
   explicit mana abilities at the correct payment boundary without priority or
   an extra stack object. Both Shivan's colored and Invoker's generic cost apply.
2. Cover wrong-seat/stale/tapped/sick/duplicate sources, haste permission,
   target-before-payment, source/target revalidation, overflow, cancellation,
   atomic commit, surplus mana and private provisional choices.
3. Preserve snapshots at every continuation, semantic actions, both policies,
   typed recorder/JSONL and full normal-reset replay. Explicitly assess version
   changes: new legal choices alter seeded policy histories, while new staged
   state changes snapshot semantics. Keep historical evidence reproducible.
4. Execute strict native and pinned XMage scripts which actually invoke mana
   abilities *inside* nonmana activation payment, with literal intermediate
   expectations and negative controls. Float-first scenarios are regressions,
   not substitutes. Keep all cases in ordinary test discovery, run full torture,
   independent review, protected delivery and exact-main CI.

The proposed boundary is new missing timing/protocol behavior with independently
testable acceptance, not a runtime-catalog migration or a wholesale activation
rewrite. No new task or dependency is registered by this feature worker. The
coordinator decides whether to register that delivery or explicitly return this
bounded repair to #23. Existing #210/#212 reference-contract splits below should
be reconciled separately, without duplicating their workers' drafts.

## Original acceptance inventory (not a pass checklist)

The following identifies relevant delivered evidence inspected during the audit.
Each report preserves its own compiled red/green, independent expectations,
normal-discovery tests and exact reference receipts. Historical candidate
statements are not current whole-component acceptance.

| Original clause | Existing evidence / unresolved boundary |
| --- | --- |
| Exact two 40-card decks, 20 identities and Goblin token, source pins, no runtime API | [Manifest](../../card-manifests.md), `data/cards/foundations_micro_v1.json`, card-manifest Python tests; [typed definitions](../card-definitions/README.md). Runtime catalog expansion remains separately registered E1a work. |
| Mountain, Forest | [Mana](../mana/README.md), [cost reference pack](../m2-cost-reference/README.md). |
| Swab Goblin, Bear Cub | [Casting](../casting/README.md), [combat](../combat/README.md), [M1 gate](../m1-gate/README.md). Exact additional Cub reference cases remain reported by #210. |
| Giant Growth, Bite Down | [Targets](../targets/README.md), [instant reference](../instant-reference/README.md). Exact Cub/Sentry cases remain reported by #212. |
| Dragon Fodder and Goblin token | [Tokens](../tokens/README.md), cost pack: distinct identity, sickness, blocking, death/cessation and stale references. |
| Llanowar Elves, Druid of the Cowl | [Creature mana](../creature-mana/README.md), cost pack: immediate mana, sickness, cast-payment use; activation-payment gap above remains. |
| Axgard Cavalry | [Haste](../haste/README.md), cost pack: real haste resolution and creature-mana interaction. |
| Magnigoth Sentry | [Flying/reach](../flying-reach/README.md); exact grown-Sentry/Shivan and Cub/Sentry reference gaps remain with #210/#212. |
| Tajuru Pathwarden | [Trample](../trample/README.md): vigilance, lethal thresholds, all six five-power blocker-only splits and departed blockers. |
| Thornweald Archer | [Deathtouch](../deathtouch/README.md): positive versus zero damage, simultaneous death, Bite, trample interactions. |
| Shivan Dragon, Wildheart Invoker | [Shivan](../shivan/README.md), [Invoker](../invoker/README.md): costs, stack, dead sources, current power, target incarnation, expiry; missing payment-window mana above. |
| Thrill of Possibility | [Thrill](../thrill/README.md): additional discard, ordered two draws, private continuation and empty draw. |
| Goblin Surprise | [Surprise](../surprise/README.md): exactly one mode, resolution-time recipient set, tokens and expiry. |
| Firebrand Archer, Crackling Cyclops | [Cast triggers](../cast-triggers/README.md), [trigger reference pack](../m2-triggers/README.md): committed cast versus failed cast, ordering, source incarnation, Archer before Fodder and Cyclops before Bite. |
| Viashino Pyromancer | [ETB triggers](../etb-triggers/README.md), trigger pack: actual entry, player targeting, source departure and self-target lethal. |
| Two players, 20 life, seven cards, London mulligan, first draw skipped, best-of-one | [Opening](../opening/README.md), [mulligan](../mulligan/README.md), [terminal](../terminal/README.md); real reset tests retained. |
| Complete pool turn/priority/stack/SBAs | [Turns](../turns/README.md), [turn settlement](../turn-settlement/README.md), [spell settlement](../spell-settlement/README.md), [combat settlement](../combat-settlement/README.md). |
| Land, colored/generic payments, additional costs/modes, targets/revalidation | Mana, targets, creature-mana, Thrill, Surprise and cost pack above; activation-payment exception prevents full pass. |
| Casting, sickness/tapping, combat, marked damage, end-of-turn effects | Casting, haste, combat and [cleanup](../cleanup/README.md); normal-reset recorder tests supplement synthetic fixtures. |
| Flying/reach/haste/vigilance/trample/deathtouch, multiple blockers and simultaneous damage | Flying/reach, haste, trample, deathtouch, Shivan and Invoker above. Current Foundations unrestricted allocations retained; no obsolete ordered-blocker response window. |
| Cast/ETB/controller/APNAP order | [Trigger ordering](../trigger-ordering/README.md), cast/ETB and trigger pack; APNAP injections explicitly synthetic, not claimed naturally reachable full-pool events. |
| Tokens, new zone identity, temporary modifications/ordering | Tokens, [objects](../objects/README.md), targets, Shivan/Invoker/Surprise and cleanup; no arbitrary layer engine claimed. |
| Life loss, empty draw, concession, simultaneous loss/draw | Terminal and Thrill above; #212 reports missing mixed-life/failed-draw reference observations. Empty library alone is not loss. |
| Maximum hand size and repeat cleanup | Cleanup and trigger pack: private discard, ordinary no-priority cleanup, exceptional priority and second cleanup; snapshots/quantum/recorder coverage retained. |
| General pool mechanics, no fixture/deck recognition | Shared rules kernel and typed definitions; existing named ability dispatch is explicitly disclosed in the current engine-validation plan, with data-only architecture migration separately deferred to E1a. No new architectural acceptance claimed here. |
| Test-first, independent expectations, positive/negative/interaction/regression and impacted reference runs | Child reports retain this evidence; present audit adds a rules-derived regression, not an implementation-generated golden. Full acceptance remains unresolved. |
| Real scalar/policy/action/recorder/replay and private pending paths | [Full-pool policy](../full-pool-policy/README.md), per-mechanic recorder tests, [private replay](../private-replay-integration/README.md); all delivered decision kinds are tested, but that does not prove absent legal sequences. |

## Reference gaps already reported by their owners

[GH-210 workpad](https://github.com/pabloxrl/mtg-lab/issues/210#issuecomment-6076401142)
identifies missing exact native/XMage Cub casting/sickness, off-turn rejection,
and real Shivan against a single Growth-boosted Sentry.
[GH-212 workpad](https://github.com/pabloxrl/mtg-lab/issues/212#issuecomment-6078764991)
identifies six Cub-to-Sentry Bite/bookkeeping/current-power cases and mixed
simultaneous loss (P0 life zero plus P1 failed draw in one SBA batch).
These are reported reference-contract gaps, not newly demonstrated engine defects.
They do not become dependencies of #23 merely by appearing here; #24/#26 retain
their full reference acceptance. No absent execution is counted as agreement.

README assessment: the root currently says M2 is incomplete and links precise
child boundaries. No usable command, setup, implemented architecture or verified
milestone is delivered by this unsuccessful audit, so no root README change is
needed for this preserved draft. A future successful repair must update the
activation/payment capability and policy/version instructions and have independent
README review. Review/CI/full-suite passes are not inferred from this document.
