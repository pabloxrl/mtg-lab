# Land plays and mana payments — GH-68

Atomic scope: R0002-B017/B029's land/payment portion. Casting belongs to GH-69;
shared-block and cross-feature acceptance remains GH-18. M1 is not complete.
[API and limitations](../../mana.md).

Original expectations derive from pinned CR 305.1–3/305.6 (land timing, one land
per turn and basic mana abilities), 605.3 (immediate mana abilities), 117.3c
(retained priority after action), 106.4 (pool emptying, no mana burn), 107.4 and
601.2h (exact symbols and generic spending), and the RFC's atomic continuation
and rejection boundary. Frozen card identities are unchanged. No implementation
output was used to generate an expected result.

[Land red](red.txt) preserves four compiled missing-land assertion failures.
[Payment red](payment-red.txt) independently shows a payable 1G cost rejected
from a synthetic RGG pool. Both preceded production implementation, using
nonfunctional API stubs. An initial test helper used the wrong manifest count
key; that setup failure was fixed and is not counted as behavioral red.
All assertions remain in normal Cargo discovery.

| Assigned catalog case | Executable test |
| --- | --- |
| rules-decisions-masked-rejection-regression | mana_masked_second_land_and_timing_rejected_transactionally |
| rules-foundations_micro_v1-forest-positive | mana_forest_mountain_positive_negative_regression |
| rules-foundations_micro_v1-forest-negative | mana_forest_mountain_positive_negative_regression |
| rules-foundations_micro_v1-forest-regression | mana_forest_mountain_positive_negative_regression |
| rules-foundations_micro_v1-mountain-positive | mana_forest_mountain_positive_negative_regression |

[Public tests](../../../crates/mtg-core/tests/mana.rs) start with the frozen deck,
explicit land-first orders and actual reset/keep/turn/land/tap transitions. They
check new object identity, no stack item, retained priority, no repeated tap,
no second land even in postcombat main, next-turn allowance, mana emptying and
life preservation. Additional cases exercise nonactive-seat activations and
pass reset; stale/foreign/wrong-seat handles and IDs; payment lock, cancellation,
reset, incomplete/duplicate commit, invalid color and provisional privacy.
Invalid inputs compare full privileged state for exact preservation.

[Synthetic unit tests](../../../crates/mtg-core/src/mana.rs) seed pools and
objects explicitly. They are not claims of reachable W/U/B/C production from
these two basics. The independent Cartesian enumerator considers every pool
with 0–2 of each of six colors and at most four total mana, every exact-symbol
cost with 0–2 per color and at most four symbols, and generic costs 0–4.
It enumerates spent-count vectors satisfying exact-symbol lower bounds and the
required total, then compares the set of resulting pools against every path
through the production continuation. Impossible costs must reject unchanged.
Repeated symbols, zero costs, colorless symbols, mixed generic choices and
missing colors are included. No random seed or expected-output regeneration.
The enumerator shares no engine legality helper. Synthetic tests also cover
wrong controller/nonland/hand sources, nonempty stack, and numeric exhaustion.

Mutation validation deliberately allows second lands, accepts stale payment IDs,
and exposes provisional spending. Each compiled fault must fail a named test.
An initial stale-ID mutant survived a test that rejected only because no units
remained; an unfinished two-unit replay assertion was added, and that mutant
was rerun. Expectations were strengthened, never weakened. The mutation source
is restored before final checks. [Mutation log](mutations.txt) preserves the final three killed mutants.

[Named mana acceptance](green.txt) PASS: nine new mana tests plus the existing
mana-boundary test. The independent enumerator checks 141,120 pool/cost pairs.
[Full torture](torture.txt) PASS after integrating current main: 120 Python tests
and all 70 Rust tests in each debug/release profile. The managed Docker worker
executes named mana acceptance and full torture,
including formatting/Clippy, Python validators and all Rust debug/release tests.
No Docker socket or nested Docker invocation is used. README gains a capability
row and API/evidence links; milestone status is unchanged. No workflow/CI/catalog
policy is modified. Separate exact-candidate review and PR/main CI are recorded
in the GH-68 workpad and PR.

Reference scope: the existing neutral priority-pass smoke is the only implemented
cached bridge scenario relevant to turn priority. The Rust decoder/regression
continues running in the full suite. [Fresh XMage execution](xmage.json) of that same fixture ran twice and agreed
using `python3 scripts/xmage.py run --cache /home/agent/.cache/xmage`.
Forge was not freshly run: its documented cache is absent in this worker. It verifies priority regression only, not land/payment
semantics; those bridge actions are not yet supported. Every-card and expanded
dual-reference coverage remains assigned integration/release work, and this is
not release qualification. No unavailable reference is counted as agreement.
