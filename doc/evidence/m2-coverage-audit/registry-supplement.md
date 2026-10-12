# Complete capability registry accounting

The [58-capability M2 mapping table](capability-accounting.json) preserves all
175 current M2 catalog rows. This supplement accounts for the other **22 of 80
registry IDs**. Counts below use **one named witness bundle per row**, not catalog
rows or distinct scenarios, and must not be added to the 175 or 123 totals.
`P/E/A/D/U` means planned/executed/agreed/disputed/unsupported for that bounded
XMage witness, at the exact inherited receipt source. Native API contracts have
passing ordinary-discovery evidence but are unsupported as *XMage API parity*.
Deferred batch contracts have no executed witness and make no capability claim.
The historical registry's `planned` fields are design metadata; this report does
not silently rewrite them or treat their empty evidence arrays as execution.

[The M1 gate's stable-ID index](../m1-gate/catalog.json),
[mechanics executable index](../m2-mechanics-audit/executable-index.json), and
[foundation supplement](../m2-reference-foundation/final-integration/reference-receipt.json)
retain the named native tests and exact earlier receipts. The complete current
normal-reset witness is [admission run 1](../admission/run-1.json), including its
hashed reset/London/priority/spells/activations/triggers/cleanup/combat compatibility
receipts. [Both archive checks](admission-archive-verification.json) bind the
recorded sources to this checkout; this is inherited execution, not a rerun.

| Registry ID | Witness / native contract | XMage P/E/A/D/U |
| --- | --- | --- |
| `setup/mulligan-bottom` | London `red-green-0-unequal`: exact occurrence-aware ordered bottoms, declarations and chance; both starters in retained full family. | 1/1/1/0/0 |
| `setup/first-draw` | Played priority `red-green-0-keep` and seat-swapped counterpart: starter skips first draw, opponent draws exact next occurrence. | 1/1/1/0/0 |
| `setup/deterministic-reset` | Reset family exact full multisets/permutations, physical duplicate swap controls and repeat equality. | 1/1/1/0/0 |
| `setup/library-privacy` | `views_only_unseen_library_permutation_and_hidden_allocation_order_are_invariant`, policy hidden twins. Reference full-state observations are deliberately privileged. | 1/0/0/0/1 |
| `priority/passing` | Played priority explicit empty-stack and nonempty-stack consecutive passes; no inferred defaults. | 1/1/1/0/0 |
| `priority/post-cast-resolution` | Played priority named cast/resolution checkpoints and actor/stack ledgers. | 1/1/1/0/0 |
| `priority/response-chain` | Played spells `growth-bite`; synthetic independent same-name Sentry holdout retains intermediate response checkpoints. | 1/1/1/0/0 |
| `costs/generic-payment` | Played activation family mixed explicit sources and Invoker payment; no invented response inside cost payment. | 1/1/1/0/0 |
| `targets/controller-restrictions` | Played spell/activation negative tapes plus synthetic instant target-role tests; Bite source own/destination opposing, Growth/Invoker/Cavalry allow opposing creatures. | 1/1/1/0/0 |
| `targets/partial-invalidation` | Played spells `departed-target`, plus `bite-source-gone` and `bite-destination-gone` in retained instant receipt. | 1/1/1/0/0 |
| `combat/attack-block` | Played combat eligible attacker and blocker-pair sets, strict declarations and raw illegal tapes. Full subset enumeration is a separate native test. | 1/1/1/0/0 |
| `objects/zone-identity` | Played spells Fodder tokens and departed target; full-game physical/incarnation/token/stack histories. | 1/1/1/0/0 |
| `terminal/concede` | Played cleanup `concession-only`; separate from all sixteen nonconcession admitted games. | 1/1/1/0/0 |
| `terminal/rewards-once` | `m2_terminal_reward` and retained recorder/collector once-only ledgers. XMage computes game outcomes, not mtg-lab reward records. | 1/0/0/0/1 |
| `decisions/stale-candidates` | `policy_restore_invalidates_pre_restore_submissions_without_state_change`, semantic/raw stale rejection and snapshot equality. Reference strict consumed-tape checks do not imply native candidate-generation parity. | 1/0/0/0/1 |
| `privacy/side-channels` | Policy hand/library twins through views, candidate order, masks, pending costs and errors. No private reference export is passed to a player policy. | 1/0/0/0/1 |
| `replay/scalar-batch` | Deferred by engine-validation scope; native scalar capture/quantum/replay equality does not stand in for a batch implementation. | 1/0/0/0/1 |
| `replay/worker-independence` | Deferred; no worker-count qualification inferred from scalar deterministic replay. | 1/0/0/0/1 |
| `replay/selective-reset` | Deferred; no selective batch-reset claim. | 1/0/0/0/1 |
| `setup/two-player-opening` | Reset/London: 20/20 life, two actual 40-card libraries, seven-card draws, declared keeps/mulligans, no sideboards. | 1/1/1/0/0 |
| `foundations_micro_v1/forest` | Played priority Forest play/tap plus explicit spell/activation payments; resolved witness in admission card set. | 1/1/1/0/0 |
| `foundations_micro_v1/giant-growth` | Played `growth-bite`, stacked-expiry cleanup and resolved witness in admission; literal +3/+3 and expiry. | 1/1/1/0/0 |

This supplement has **15 executed bounded reference witness bundles** and **7
unsupported reference-parity/deferred bundles**. The latter are four native-only
API concepts and three explicitly deferred batch capabilities. They are not
missing scoped game-rule bridges and never count as reference agreement.

The two independently adjudicated upstream exceptions are reported separately in
[the audit](README.md#upstream-exceptions-and-provenance), so zero disputes in
these bounded witness columns does not claim zero upstream discrepancies.
Patched combat and adapter-enforced trample rejection remain explicitly qualified.

All twenty nontoken card keys appear in both admission `played_cards` receipts;
[the card crosswalk](../m2-mechanics-audit/cards.json) independently maps their
costs, rules and original native/reference evidence. The Goblin token is covered
by the Fodder/Surprise/token reference packs and current played spell identities.
No card is admitted merely by appearing in a deck list or opening hand.
