# GH-212 prerequisite contract audit — incomplete

Audited current main `253a9521077ccea8dd09f9d75ec0f1f3d696438d` on 2026-10-09.
This is a source/fixture inventory, not an executed reference receipt, behavioral
red, engine defect diagnosis or completed acceptance. The adjacent
`prerequisite-audit.json` retains all 31 assigned catalog cases verbatim and hashes
the inspected contracts. No catalog expectation or production rule was changed.

## Missing Sentry/Bite reference contract

The existing strict `InstantResponseTest.java:282` fixture loader accepts only
Bear Cub, Forest, Mountain, Bite Down and Giant Growth. Line 284 rejects any other
card key before execution. It cannot construct Magnigoth Sentry. Its earlier
eleven-case M1 receipt remains valid for its original scope.

`FlyingReachTest.java` supports six fixed scenarios: reach, cub-illegal, tapped,
ground, growth and cast. Its Growth case is combat against an explicitly synthetic
flying Cub, not Bite Down. `DeathtouchTest.java` supplies Thornweald as the Bite
source (which kills Sentry through deathtouch), not the required ordinary Cub
source that leaves Sentry alive with two marked damage. `ShivanTest.java` supplies
Shivan as its boosted Bite source. `M2TriggerCompositionTest.java` supplies Cyclops.
These delivered scripts cannot be re-executed to establish the following exact
assigned cases:

- `rules-foundations_micro_v1-bite-down-positive`: Cub deals two to opposing
  Sentry; Sentry stays 4/4 with two marked damage, Cub takes zero.
- `rules-foundations_micro_v1-bite-down-negative`: reject own Sentry as Bite's
  destination, without mutation.
- `rules-foundations_micro_v1-bite-down-regression`: source Cub killed above
  Bite, opposing Sentry remains unmarked.
- `rules-objects-bookkeeping-positive`: inspect Sentry's separate 4/4 stats and
  two marked damage after Bite.
- `rules-objects-bookkeeping-interaction`: damaged Sentry becomes 7/7 with two
  marked damage after Growth, then 4/4 with zero at cleanup.
- `rules-continuous-resolution-power-positive`: Growth on source Cub above Bite
  yields five damage and kills Sentry.

Native tests and the GH-196 catalog inventory are related evidence, but the
inventory's inclusion of these IDs is not matched XMage execution. The GH-196
README explicitly retains the old instant suite; that bridge rejects Sentry.

## Missing mixed simultaneous-loss reference operation

`rules-terminal-simultaneous-loss-interaction` requires a synthetic before-SBA
state with P0 at zero life and P1 having failed an empty-library draw. Both lose
in the same batch and the outcome is a draw (CR 104.4a/704).

`TerminalTest.java:38` always performs `a.drawCards`, where `a` is P0. Its neutral
schema has no draw actor or failed-draw flags. `terminal_same_neutral_boundaries_as_xmage`
likewise calls `draw_top(Seat::P0)`. The seven existing fixture cases cover
simultaneous life loss and a separate P0 empty draw; neither executes mixed loss
reasons across the two seats. Existing exported loss flags are useful, but do not
include an explicit winner/draw field. No existing alternate terminal bridge
expresses this exact assigned input and result.

## Proposed coordinator split

Register a bounded prerequisite for strict native/XMage Sentry/Bite scenarios,
including the six unchanged cases above, literal intermediate observations,
friendly-target rejection and corrupt-checkpoint controls. Extend the existing
adapter rather than create another rules engine. Keep original M1 fixtures and
receipts intact and retain new cases in normal test discovery.

Register a separate bounded terminal bridge prerequisite for explicit per-seat
failed-draw setup and simultaneous loss settlement, with literal mixed-reason
draw, single-loss and seat-swap controls and observed outcome. Synthetic setup is
explicitly permitted by these catalog cases; it is not normal-reset game evidence.

Both prerequisites require real pinned reference execution, full torture,
independent review, protected delivery and exact-main CI. Then explicitly
reactivate GH-212 to finish the original composition, decision/privacy/replay/
capacity assertions, independent holdout and exact assigned-case receipt. The
coordinator should reconcile any overlap with GH-210's separately recorded gaps.
This report neither registers tasks nor changes dependencies or authorization.

## Validation and remaining work

Fresh-main manifest/ledger/schema/gates validate (124 tasks); GH-80 and GH-208
have completed workpads with acceptance/review/protected merge/exact-main CI.
Both exact-main CI runs were re-fetched and confirmed successful. Parent is active.
No Rust/Java build, reference run, torture run or separate review was performed:
source inspection established the missing input contracts before implementation.
The remaining assigned cases are not certified by this partial audit.

No README change is needed for this unsuccessful delivery: no usable command,
supported behavior, architecture or verified milestone status changed. No PR,
commit, push, merge, feature implementation or completion is claimed. Partial
R0002-B010/B011/B026/B028/B030 and the original #24/#26 acceptance remain unresolved.
