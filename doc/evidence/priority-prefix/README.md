# Played priority prefix evidence (GH-271)

The current-main candidate passes the full locked torture suite: **309 Python
tests and 1490 Rust debug/release test executions**, zero failed/ignored
or skipped. The [complete log](torture.log.gz) includes normal
`python3 scripts/run_tests.py`, `cargo test --workspace --locked`, release tests,
formatting, Clippy, documentation, program and catalog checks. New normal
coverage is **six Python tests and one Rust test**, executing six real prefixes,
760 checkpoints, repeated runs, same-game stop/continuation and twenty rejected
tapes. Base: `c779b190ed28ff845741c4ce1149f71e55ec675b`. This report records local
acceptance; independent review, protected merge and exact-main CI receipts belong
in the PR and [issue workpad](https://github.com/pabloxrl/mtg-lab/issues/271#issuecomment-6096816121).
Delivery completion remains conditional on those receipts.
The [final focused run](focused-final.log) discovers all six new Python tests.
Earlier [pre-integration](pre-integration-torture.log.gz) and
[intermediate integrated](intermediate-integrated-torture.log.gz) full runs are
retained separately; the latter preceded the source-inventory regression.

Both required actual pinned native/XMage priority executions agree:
[run 1](priority-run-1.json), [run 2](priority-run-2.json). Each runs six cases
three ways per engine (ordinary, repeated and bounded/resumed): 18 prefixes per
engine, not 18 distinct cases. Both include 26 first-difference comparator
controls, twenty invalid tapes and three unsupported-callback probes. Because the
shared bridge gained continuation hooks, the unchanged London family also ran
twice: [compatibility 1](priority-compat-mulligan-1.json),
[compatibility 2](priority-compat-mulligan-2.json), each with its existing 32 cases
and all original controls. All source and artifact hashes were verified against
the final files. The original reset/London tests, catalog cases, owners and
source/card/rules pins remain unchanged.

Separate privileged archives retain actual inputs, consumed chance/choices,
checkpoints, raw occurrence/stack UUID bindings, witnessed zone transitions and
raw counters, selected spell candidate provenance, native semantic action records
and scalar recorder captures, independent oracles, rejection/comparator controls,
source/dependency/toolchain hashes and logs:

- [Priority run 1](priority-run-1.privileged.tar.gz)
- [Priority run 2](priority-run-2.privileged.tar.gz)
- [London compatibility 1](priority-compat-mulligan-1.privileged.tar.gz)
- [London compatibility 2](priority-compat-mulligan-2.privileged.tar.gz)
- [Earlier unsuccessful or superseded attempts](prior-attempts.privileged.tar.gz)

Extract only into an isolated diagnostic directory. Archive top-level names match
`privileged_directory`; verify file contents using `privileged_artifacts` in each
receipt. These include both hands/library orders and are not player observations
or policy features. Earlier compile/lifecycle failures and the superseded broad
negative-control receipt are retained, not counted as final acceptance.

The archived [baseline test](priority-baseline-test.py) executed the delivered
real native London client and produced the intended [assertion failure](priority-red.log):
`first_upkeep != second_creature_resolved`. This was not a compile/import error.
Authored-before-execution hash records preserve independently specified inputs
and expectations. The [first native pass](native-first-pass.log) and
[first focused pass](focused-first-pass.log) preceded the complete suite above.
[Fixture-authoring arithmetic](../../../fixtures/reference/author_priority.py)
reads frozen card data and authored tapes, never actual engine output; both
clients execute real engines and never call it as an alternative rules consumer.

## Independent rules and representation review

The [oracle derivation](../../../fixtures/reference/full-pool-priority-oracle.md)
uses CR 103.8a, 106.4, 117.3/4, 302.6, 305.1/2, 400.7, 601.2, 608.3a and 508.1/2/8,
plus frozen Swab Goblin/Bear Cub costs and stats. The fetched official rules text
matched the unchanged source pin. Native expectations, final ledgers, all inputs
and all negative tapes remained byte-identical to their pre-execution hashes.

The first reference execution exposed two source-defined representation details.
The [initial reference oracle](initial-xmage-oracle.json.gz) is retained for review:

1. `Mulligan.drawHand` invokes `MulliganDefaultHandSorter`, arranging lands before
   creatures while preserving equal-card order. The corrected raw-order oracle
   derives that ordering from pinned source; subsequent draws append. CR 402.3
   permits arranging a hand in any convenient fashion. Each engine's raw hand
   order is independently asserted, while cross-engine hands compare membership.
   Ordered libraries, battlefield and stack remain exact. This adds raw-order
   coverage to the existing opening family's unordered-hand comparison.
2. `LondonMulligan.mulligan` returns the actual hand with `Library.addAll` and
   `hand.clear`, without incrementing its raw zone counter. The version-3 `incarnations` field starts at zero in the initial library and
   counts actual zone transitions under CR 400.7. The bridge retains every
   actual before/after zone and raw
   counter. A zero raw increment is permitted only for a hand-to-library return
   witnessed at the mulligan shuffle. Other transitions must increment once;
   unseen round trips or unsupported transitions fail. No state is injected
   into either engine. Consulted source hashes are in
   [priority provenance](../../../references/xmage/priority-provenance.json).

Raw in-flight casting also has distinct literal expectations: native holds a
private transaction until payment commits; XMage puts the announced card on the
stack before paying. Both raw states are retained and asserted; neither is
rewritten to pretend they are identical. Their payment ledgers and committed
fields agree, subject to the explicit hand-presentation distinction above.

After three reference repair/diagnostic cycles, a broad rejection check passed,
but a diagnostic audit found off-turn and unaffordable casts failing later with
actor mismatches. The stronger [intended-boundary assertion failed](negative-boundary-red.log)
against actual reference observations before the repair. Pinned `PlayerImpl.cast`
is a low-level method assuming caller validation. The bridge now queries the
engine's actual per-card playable actions, requires exactly one selected spell
candidate, retains its source/action/UUID, and invokes the real cast. The final
controls reject off-turn casting at tape entry 46 and insufficient mana at entry
3 through that engine query. This is selected-action legality evidence, not a
claim that the low-level cast call validates timing or that both engines' full
legal sets were compared. Wrong-color payment rejection separately uses the
witnessed pool and unpaid cost; it is an adapter guard, not raw payment-call
rejection. Native illegal records and stale policy candidates reject with full
caller state/RNG unchanged.

The independent reviewer must explicitly assess both representation mappings,
the new reference-oracle correction, unchanged requirements and stronger
raw-order/incarnation/negative-boundary coverage, plus README accuracy. These
corrections are not accepted merely because an engine produced them. No original
test or expectation was removed or weakened.

A final source-inventory audit found that the root-only Rust glob omitted the
nested recorder implementation actually consumed by this family. The new
[normal-discovery regression failed first](source-inventory-red.log), then passed
after recursive source coverage included `crates/mtg-core/src/trajectory/v2.rs`.
Both final priority receipts were rerun with that complete inventory. All earlier
receipts remain in the superseded-attempt archive; no gameplay expectation or
input changed. Full torture was repeated on the final integrated tree with all
six new Python tests discovered.

## Scope

The [protocol and runnable commands](../../full-pool-reference.md#version-3-played-priority-prefixes)
use an additive version 3 test envelope. Both starting seats, keep/one-mulligan
openings, floating/payment-stage basic mana, colored/generic payments, explicit
passes and empty declarations are covered. Two same-name creatures per seat stay
distinct through hand, stack and battlefield. A named first-cast stop resumes the
same engine; resolution occurs only through explicit passes. The scalar recorder
is an open played prefix, not a complete episode.

Vanilla creature timing admits one spell at a time. The stack-order comparator
control is explicitly synthetic, using two separately witnessed stack entries;
it is not a simultaneous-stack played claim. No reference rollback/internal-RNG
or complete legal-set equality, noncreature/trigger/activated-choice support,
nonempty combat, cleanup discards, complete games, RFC 0003/RL work or M2 completion
is claimed. Original owners and the #24/#25/#26 audits retain full acceptance.
No production rule, workflow, CI, dependency pin or repository setting changed.
