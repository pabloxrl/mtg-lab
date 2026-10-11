# GH-272 played spell prefixes

Candidate evidence is being finalized; no protected delivery or M2 completion
is claimed by this report. The focused module and the actual reference runner
are the executable acceptance interfaces described in
[the version-4 protocol](../../full-pool-reference.md#version-4-played-spell-prefixes).

## Scope and independent expectations

Six full-deck, normal-reset, keep-opening prefixes exercise Dragon Fodder,
Thrill of Possibility, both Goblin Surprise modes and both orders of a
Growth/Bite response chain. They stop in main, before cleanup. All casts,
passes, empty attacks, physical targets, modes, payments and discard choices
are explicit. There is no alternative rules engine or direct resolution hook.

The [authored schedule and final ledger](../../../fixtures/reference/author_spells.py)
and [rules/card basis](../../../fixtures/reference/full-pool-spells-oracle.md)
precede execution and use the unchanged source pins. Live acquisition of all 21
card sources and the CR text subsequently matched those exact pins; the
[hash-only source receipt](oracle-sources.json) records the verification. Raw
Oracle responses/text and the CR document remain outside the repository. CR 601.2, 608.2, 400.7,
111 and 704 justify the costs, ordered targets, stack order, temporary changes
and token disappearance. GR-030 and the token portion of GR-012 are covered by
these prefixes; original audit owners and acceptance remain unchanged.
Partial additive requirement owners: R0002-B010/B011/B026/B028/B030.

Literal checks include Cub 5/5 after responding Growth, surviving ordinal-0
Goblin 1/1 after Bite kills ordinal 1, Growth failing to boost its departed
Goblin target, Thrill discarding physical Mountain copy 2 before drawing
Mountain copies 6 then 7, two 1/1 Goblins from Fodder, two 3/1 recipients from
Surprise's boost mode and four distinct tokens after its creation mode.

## Test-first and diagnostic history

- [Original baseline test](baseline-test.rs) and [behavioral red](spells-red.log):
  a normal-reset legal Fodder cast was rejected by the prior adapter as an
  unsupported noncreature spell. The test compiled and failed its intended
  assertion; no compile/import error is counted as red evidence.
- [Self-discard control red](discard-negative-red.log): the first XMage control
  used the spell's former hand incarnation and therefore failed too early. The
  corrected reference input uses its witnessed announced-stack incarnation,
  requiring rejection at the discard rule. Native retains the hand incarnation
  because its whole cast remains private until commit. The original assertion
  is in [discard-negative-red.py](discard-negative-red.py).
- [Token provenance red](token-definition-red.log): the real native birth report
  initially omitted its immutable engine definition hash. Birth observations
  now include the actual CardId key/hash; XMage separately exports observed
  color bits and subtype from the actual created permanent.

Initial reference diagnostics also corrected source-evidenced adapter assumptions:
XMage's token display name is `Goblin Token`; its discard-cost callback follows
mana payment; its announced stack object has no selected mode before chooseMode;
and token references use permanent identity instead of card-only diagnostics.
No card expectation, source pin, catalog owner, original fixture or rule was
changed to match an engine result. Earlier raw diagnostics remain retained.

## Evidence boundaries

Native submission uses existing semantic decode/apply, factored choices and the
scalar recorder. Every stale submission and rejected action checks live
state/RNG nonmutation. Uncommitted cast checkpoints retain public mana, zones,
stack and characteristics; cancellation restores those fields. Failed complete
input envelopes also preserve the caller's game. These are separate claims.

XMage selected casts are checked against the engine's per-card playable-action
query before actual player casting. Actual chooseMode/chooseTarget/choose and
playMana callbacks consume their supplied choices. Resource guards read the
real pool and unpaid cost; this is not raw illegal low-level cast-call rejection
or complete legal-set equality. Unsupported callbacks fail explicitly.

Every positive and rejected prefix retains its actual consumed opening and
played choices, raw checkpoints and intended first-error category/sequence.
Reference callback probes also reject extra target/mode/mana calls and a
repeated token-creation event. Native and reference repeats retain separate
physical raw identities while canonical spell/token identities remain stable.

Native and XMage announcement/payment states differ and are retained separately.
Every comparable committed checkpoint checks life, mana, zones, ordered
library/stack/target references, characteristics and token creation order.
Cross-engine hands/battlefields compare membership; raw orders and ordered
Thrill draws are retained. Native temporary modification slots are witnessed
from `turns.modifications`; XMage exports actual effect source/duration objects.
Native token definition hashes bind to the frozen red Goblin identity; no
unimplemented dynamic native color observation is fabricated.

Privileged traces, ordered decks, private hands and scalar captures belong in
separate `.privileged.tar.gz` archives. Public receipts contain hashes, counts,
source/toolchain/dependency pins and explicit claims/limits. There is no
reference internal-RNG/rollback, cleanup-expiry, trigger/activated-choice,
nonempty combat, complete-game, complete legal-set or M2 completion claim.

## Delivery validation

The final source-bound actual comparisons passed twice per affected family:

| Family | Public receipts | Separate privileged traces |
| --- | --- | --- |
| Spells: six prefixes, 468 aligned committed checkpoints, 18 invalid tapes, four callback probes, 14 native and 13 reference comparator controls | [run 1](spells-final-1.json), [run 2](spells-final-2.json) | [run 1 archive](spells-final-1.privileged.tar.gz), [run 2 archive](spells-final-2.privileged.tar.gz) |
| Priority compatibility: six prefixes and original controls | [run 1](spells-final-priority-1.json), [run 2](spells-final-priority-2.json) | [run 1 archive](spells-final-priority-1.privileged.tar.gz), [run 2 archive](spells-final-priority-2.privileged.tar.gz) |
| London compatibility: 32 prefixes and original controls | [run 1](spells-final-mulligan-1.json), [run 2](spells-compat-mulligan-2.json) | [run 1 archive](spells-final-mulligan-1.privileged.tar.gz), [run 2 archive](spells-compat-mulligan-2.privileged.tar.gz) |

Each archive expands to the `privileged_directory` named in its public receipt.
Every receipt source hash and raw artifact hash was checked before packaging;
[archive hashes](artifact-hashes.json) bind the transport files. The spell receipts
include all workspace Rust sources/manifests as well as adapter, fixture,
Oracle, dependency and toolchain provenance. Prior families retain their versioned
receipt formats. These are separate real invocations, each also repeating its
positive prefixes internally.

The [focused log](spells-focused-final.log.gz) records ten passing Python tests;
all ten are newly discovered tests. One new native test exercises all six prefixes,
repeats and eighteen rejected tapes through actual semantic actions. The
[interim full torture log](spells-torture-1.log.gz) passed, but started before the
final hardening and does not replace the final run. Four unsuccessful initial
reference adapter attempts remain in the
[diagnostic archive](adapter-diagnostics.privileged.tar.gz).

Final full locked torture on the frozen sources passed: **321 Python tests and
1,492 Rust debug/release executions, zero failed or ignored**. This includes
`python3 scripts/run_tests.py`, `cargo test --workspace --locked`, release discovery,
formatting, Clippy, docs, catalog and program validation. The
[complete final log](spells-torture-final.log.gz) records discovery of the ten new
Python tests and the new native test in both profiles. Current main
`af642fe81596900994a68726520bd88062c600e3` was fetched and already integrated before
the final run. The affected focused command and both documented spell commands
were exercised against the real adapters; final output filenames distinguish the
source-frozen reruns from interim diagnostics.

Separate exact-candidate review JSON, protected PR verification and exact-main
CI receipts are recorded in the [issue workpad](https://github.com/pabloxrl/mtg-lab/issues/272#issuecomment-6099051987)
and linked PR after delivery; this report alone does not claim that merge. README adds only the
bounded test-only spell family and links its commands/limitations; the milestone
status remains unchanged. Independent review must check these claims and the
source-justified negative-control correction.
