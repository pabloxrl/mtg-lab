# M2 scenario and reference coverage audit — GH-24

Audit base: `05920066391976cdfda6c856dd03186ba961246f` (the delivered initial
terminal-game admission). **Scoped technical audit: PASS, conditional on the
independent review and protected delivery recorded in the linked workpad.
This is not the M2 gate.** This bounded integration audit
owns the M2 aspects of R0002-B010, B011, B026, B028 and B030 under the current
[engine-validation scope](../../programs/engine-validation.md).

The [previous unsuccessful audit](historical-audit.md), its original inventory,
source hashes and dependencies remain historical evidence. Its missing opening
and continuous replay contracts were subsequently delivered through the reviewed
[#268 repair graph](../../programs/m2-audit-repair-prerequisites.md), ending in
[#277's sixteen-game admission](../admission/README.md). No rule, fixture
expectation, source pin, catalog owner, test or delivery gate changes here.
No artificial behavioral red is claimed for this documentation-only audit.

## Admission method and distinct scenario floor

[The per-scenario review](scenarios.md) admits **123 selected semantic scenarios**
across all twelve required families, independently of the catalog-row and game
counts. Each entry identifies its distinct rule situation, expected behavior,
ordinary-discovery Rust test, input/expectation hashes and exact inherited
reference receipt in [the machine-readable ledger](reviewed-scenarios.json).
Separate candidate review must independently assess these individual admissions;
its exact-head result is preserved in the delivery workpad and PR.

This is a conservative selected floor, not the total number of passing tests.
A test's seat/repeat/quantum loops contribute no additional scenarios. The six
symmetric damage splits, repeated catalog mappings, duplicate Pyromancer inputs,
and ordinary-cleanup aliases do not inflate this floor. Neither comparator
mutations nor the sixteen complete games add to it. All excluded repetitions
and aliases remain mandatory executable regressions. Native privacy, capacity,
and snapshot contracts count as semantic scenarios but not as XMage agreements.

All expectations predate this audit and derive from pinned card/rule sources or
explicit native API contracts. They are not regenerated from engine output.
For example, opposing Cavalry targets are legal and haste expires before their
next turn; Invoker accepts opposing creatures and mixed generic payment;
Thornweald's zero damage is not lethal; Thrill's failed second draw is a loss.
These are per-case semantic distinctions, not inferences from passing totals.

The [175-row historical catalog inventory](catalog-inventory.json) remains
byte-for-byte unchanged and retains every original setup, action, expected
result, basis and owner. [Capability accounting](capability-accounting.json)
joins those rows into 58 M2 capabilities: 175 executed native mappings and 173
bounded XMage mappings, with two explicitly native-only capacity mappings.
All four packs have also [passed fresh current-source execution](fresh-reference.json);
each capability links its fresh pack receipt beside the historical evidence.
These counts are **catalog mappings**, not 175 distinct scenarios or full-state
agreements. The same table also lists deduplicated suite/case keys and their
planned/executed/agreed/disputed/unsupported counts; key uniqueness alone does
not establish semantic independence. Every row retains its observation/API boundary. The complete registry
also includes earlier scalar capabilities and deferred batch contracts; these
are accounted for in [the registry supplement](registry-supplement.md).

## Original acceptance and RFC clauses

| Obligation | Executable evidence and conclusion |
| --- | --- |
| B010: frozen two 40-card decks, 16 lands/24 spells, twenty definitions and Goblin token, mirrors/both starters | [Mechanics/card index](../m2-mechanics-audit/cards.json), manifest tests, and sixteen admitted normal-reset games. All twenty nontoken cards have consumed plays and observed resolved witnesses in each admission receipt. Tokens are created during play, never smuggled into the deck. Printing/Oracle/rules pins and runtime independence remain unchanged. |
| B011: two players, 20 life, seven cards, London mulligans, skipped first draw, no sideboards | Reset, London and played priority families use exact frozen multisets and physical occurrences. Four separately reviewed opening schedules cover keep, one, unequal and repeated mulligans; all 32 London cases and both starters remain required. Admission games all keep seven and do not replace London coverage. |
| B011: turns/priority/stack/SBAs, mana/costs/targets/modes, casting/tapping/activations | Original mechanic and pack tests plus continuous priority/spell/activation families. Explicit passes, target-before-payment, source reservations, discard commitment, mode choice and selected-action rejection are retained. No response window is inserted within costs. |
| B011: combat and six keywords; simultaneous damage/multiple blockers | Synthetic packs plus nineteen played combat cases, pre-SBA damage marks, modern unrestricted nontrample splits, trample/deathtouch lethal constraints and explicit simultaneous graveyard owner orders. Patched-reference and raw-constraint exceptions remain visible below. |
| B011: cast/ETB/controller/APNAP triggers | Cast and ETB reference packs plus the played trigger family. Source/event/incarnation identities and surviving dead-source abilities are checked. APNAP cross-controller injections remain explicitly synthetic, not naturally reached pool games. |
| B011: tokens/zone identity/temporary effects and cleanup | Fodder/Surprise/token packs, occurrence-preserving spell/trigger histories and played cleanup. Boost expiry and damage removal are simultaneous. Extra cleanup is tested with explicitly injected supported trigger work; it is not a fabricated naturally reachable trigger. |
| B011: life loss, attempted empty draw, concession, simultaneous draw | Terminal-v2 and Thrill fixtures, native once-only terminal accounting, played cleanup, and sixteen nonconcession rules-terminal games. Empty library alone remains ongoing. Mixed simultaneous-loss setups remain synthetic; concession fills no normal-game quota. |
| B011: general mechanics and modern combat baseline | Shared rules remain the engine; no fixture recognizers, production reference delegation, invented card or new layer support. Existing named ability variants remain disclosed. RFC 0003 migration is later work, not certified here. |
| B026: all twelve families, normal/adversarial combinations | The 123 individually inspected entries span every family; the complete catalog/child tests remain required in ordinary discovery. Family-specific minimums and native/reference limits are below. Batch/worker additions are explicitly deferred by the current scope, not skipped advertised tests. |
| B028: pinned out-of-process XMage; source/card/JDK/dependency/bridge/patch provenance | [Reference index](reference-index.json) retains exact receipts and current-source differences. [Admission archive verification](admission-archive-verification.json) checks both actual-run archives and all 168 recorded source hashes against this audit tree. No archive check is relabeled as fresh reference execution. Existing MIT notices, source consultation/adaptation records and test-only process separation remain. |
| B028: same neutral inputs; no AI/default/shuffle fallback; physical/chance identity | Reset/London/continuous families consume exact physical occurrence permutations and semantic choices, rejecting missing/extra/wrong input. Both complete-game replay runs use frozen inputs without invoking a policy; eight heuristic origins are provenance, not runtime reference AI. |
| B028: intermediate fields, legal rejection, all twenty cards/every scoped capability | Normal-reset families and admission observe opening, committed actions, resolution, combat damage/SBAs, cleanup and termination. They retain life, ordered zones, stack, targets, mana, characteristics/effects, actors and identity histories. Selected-action acceptance, native raw rejection, reference constraint domains and full legal-set enumeration are distinct. Full cross-engine legal-set equality is not claimed where unobservable. |
| B028: scripted complete games in every row | Sixteen distinct native/XMage normal-reset games, two per RG/GR/RR/GG × starter: heuristic-origin and independently authored directed path. Each is replayed twice per engine in both inherited admission invocations. Repetitions do not raise sixteen. No concession or safety-limit ending fills the quota. |
| B028: dual reference, independent critical suite and reference corpus | Forge/at least twenty dual-reference critical scenarios and later reference-AI corpus qualification retain the #241 transition ownership. The reviewed repair plan explicitly assigns this audit the initial two-engine subset. No Forge or later release qualification is inferred or waived. |
| B028: planned/executed/agreed/disputed/unsupported and first divergences | Per-capability bounded catalog accounting, registry supplement, exact source/receipt inventory and the explicit two-item upstream exception register below. Unsupported native APIs and deferred work are separate from executed game-rule agreement. No unavailable reference is a pass. |
| B030: complete discovery, no expected failures for advertised behavior | Required full torture includes verify, all ordinary Python discovery, Rust debug/release, formatting and Clippy. No test removed, skipped, weakened or replaced. The old design catalog is not itself executable proof. |
| B030: independent expectations/holdouts and anti-overfitting | All four independently authored pack holdouts remain frozen. Three appear in the selected floor; the same-name Sentry/Growth/Bite identity holdout remains supplemental. Named mutants below test the verifier, without inflating scenario counts. Original behavioral reds and minimized failures remain in child receipts. |

## Twelve-family minimums and boundaries

| Family | Selected scenarios | Minimum behavior and supporting evidence |
| --- | ---: | --- |
| Setup and randomness | 5 | Exact opening/reset and 32 London scripts, both first-draw seats, ordered Thrill draws, native library privacy twins. |
| Priority and stack | 4 | Empty/nonempty passing, post-cast/resolution priority, response chains and no internal cost window; original instant response and played priority/spell receipts. |
| Costs and mana | 26 | Wrong/insufficient and mixed generic mana, creature timing/sickness, explicit in-payment sources, Thrill discard and native atomic rejection. |
| Target legality | 9 | Own/opponent restrictions, required targets, departure/partial invalidation, resolution revalidation; original instant and spell families supplement selected cases. |
| Triggers | 20 | Cast versus resolution, failed casts, ETB targets, multiple source ordering and synthetic APNAP. Played identities supplement synthetic scheduler checks. |
| Combat | 24 | Attack/block legality, all six keywords, simultaneous damage, modern multiple-blocker allocations and blocked status. |
| Continuous changes | 13 | Stacked Growth/Surprise/activation effects, current resolution power, fixed affected sets and simultaneous expiry/damage cleanup. |
| Objects and zones | 10 | Token birth/death/cessation, incarnation isolation, dead-source stack abilities and damage bookkeeping; token/played spell receipts supplement the selected rows. |
| Terminal states | 5 | Life loss, simultaneous mixed loss, failed empty draw, concession, once-only rewards; complete-game and terminal-v2 receipts remain separate. |
| Decisions and encoding | 3 | All 64 subsets and all six splits are independently enumerated; 79/80/81 domains reject overflow explicitly, masked/stale choices preserve state. Native representation, not a claim about XMage's API. |
| Privacy and PvP | 2 | Hidden hand/library twins, masks/candidate order/errors, private pending choices. Full privileged reference traces never become policy input. |
| Replay and batching | 2 | Pending snapshot/restore, generation invalidation, scalar semantic replay, capture/quantum equality. Batch equality, worker-count independence and selective batch reset are deferred, not certified. |

## Upstream exceptions and provenance

These are **two distinct adjudicated upstream limitations**, not zero disputes:

1. Simultaneous graveyard arrivals: original unmodified XMage disagreed at
   `shivan-multiple`, `before/304`, `$.graveyard[1][0]`. CR404.3/704.3/101.4 requires
   explicit owner choice and simultaneous movement. The [minimized mismatch and
   original failing execution](../combat-prefix/README.md) remain. Current played
   evidence uses separately hashed `CombatSbaGame.java` (`patched-combat-sba-v1`),
   not unmodified-upstream agreement. Native owner-order regressions preserve both
   orders, both owners, physical-copy swaps and atomic invalid-choice rejection.
2. Trample constraint dialog: pinned XMage exposes a total minimum but allows
   `[1,3]` on two undamaged 2/2 blockers while one damage tramples. The [independent
   combat oracle](../../../fixtures/reference/full-pool-combat-oracle.md) records
   CR702.19b and retained raw constraints. The adapter rejects using witnessed
   per-blocker lethal values. This is adapter-enforced rejection, not an upstream
   raw rejection. Both constraints and negative controls remain inspectable.

Neither is silently reclassified as agreement. The 173 historical bounded
catalog mappings report the fields their original receipts actually compared;
they did not certify these subsequently observed normal-game boundaries.
The machine-readable capability table also joins each exception to its affected
capabilities with separate 1 planned / 1 executed / 0 raw-upstream agreed /
1 raw-upstream disputed / 0 unsupported boundary counts. Those two observations
are not additional catalog mappings or admitted scenarios. Current played
agreement carries its patched/adapter-qualified classification.
No unexplained divergence is accepted by this audit. The broader Forge and
reference-AI qualification remains pending under the current plan.

Pins and provenance are retained in `references/xmage/pins.json`,
`references/xmage/dependencies.json`, the family-specific provenance files,
`data/cards/foundations_micro_v1.json` and the rules manifest. Original scenarios
and consulted upstream APIs are distinguished from adapted tests; a source
engine's own test is not counted as independent corroboration by that engine.
No source vintage is freshly certified merely by this audit's hash comparison.

## Named mutants and holdouts

[The twelve-mutant matrix](../m2-mutations/matrix/receipt.json) retains each
unmodified passing baseline, successful mutant build, exact discovered test and
named assertion failure: omitted priority, stacked mana ability, sick tap,
missing cast trigger, retained boost, late cleanup damage removal, stale action,
private-card leak, reversed terminal reward, truncation as draw, clipped choices
and missing failed-game denominator. Build errors/timeouts/wrong assertions are
not detections. Normal torture retains the named unmodified tests.

The [fresh current-source matrix](mutations.json) passed all twelve unmodified
baselines and detected all twelve compiled mutants at those named assertions.
[Build, discovery and test logs](mutation-logs.tar.gz) preserve the actual runs;
audit-document hashes captured by the runner precede this final report, while
production, test, fixture and mutation inputs are unchanged.

The independent holdouts remain: two Archers while discarding Surprise; two
Invoker boosts plus Growth on Thornweald; departed Cyclops between two labeled
Archer triggers; and same-name Sentry/Growth/Bite response/identity. Each has
literal pre-execution expectations and separate native/XMage observations.
They are frozen supplemental checks, not freshly undisclosed holdouts or a
substitute for the later independently authored dual-reference critical suite.

## Reproduction, verification and delivery

Run inside the managed toolchain; do not invoke Docker from the worker:

```sh
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- ./scripts/torture.sh
```

This includes `./scripts/verify.sh`. Reference reproduction uses the exact pack
commands in the [cost](../m2-cost-reference/README.md),
[combat](../m2-reference-combat/README.md), [trigger](../m2-triggers/README.md),
[foundation](../m2-reference-foundation/README.md) and
[admission](../admission/README.md) reports, with this issue's isolated
`$MTG_REFERENCE_CACHE` and the same heavy lock. Cache contents are build inputs,
not acceptance evidence. Extract privileged game archives into private diagnostic
directories, never player-visible artifact locations.

[Fresh full torture](torture.json) passed 397 Python tests and 1,524 Rust
debug/release executions, with zero failed/ignored Rust tests. Every selected
scenario maps to a named test that passed in both modes. The compressed
[raw log](torture.log.gz) is retained with its uncompressed hash. Reference
pack reruns also passed, with [exact source hashes, commands and artifact
manifest](fresh-reference.json): 56 cost, 50 combat, 38 trigger and 31 foundation
mappings. Their [cost](fresh-cost.json), [combat](fresh-combat.json),
[trigger](fresh-trigger.json) and [foundation](fresh-foundation.json) receipts
retain observation boundaries, independently fixed expectations and all four
holdouts. The [reference archive](reference-runs.tar.gz) retains 1,377 files,
including actual native/XMage observations, negative controls, build/test logs
and child receipts. All 427 recorded source/input hashes match the audit tree.

Of the selected 123 scenarios, 108 link fresh synthetic pack receipts, seven
link the source-matched inherited admission compatibility receipts, and eight
are native-only contracts checked in full torture. The sixteen complete games
were not rerun for this audit: both actual-run archives and their current source
hashes were independently checked. Historical reference source differences
remain visible rather than being overwritten. The unchanged catalog and all
selected fixture/oracle/test hashes are checked in [audit inputs](audit-inputs.json).
Documentation links, manifest/source/ledger dependency gates and whitespace
checks pass; no rules, tests, expectations or policy changed.

Delivery additionally requires
clean-candidate separate Codex review, protected PR checks/merge, successful CI on
that exact main commit and the bounded program handoff. The [single workpad](https://github.com/pabloxrl/mtg-lab/issues/24#issuecomment-6091093721)
records those gates. The M2 milestone verdict remains owned by gate #26.

README impact: add this audit's durable coverage/evidence link, keeping the M2
gate incomplete. Commands, setup, supported rules and architecture do not change;
there is no new quickstart to execute. Independent review must check README
accuracy and preserve the distinction between this audit and gate #26.
