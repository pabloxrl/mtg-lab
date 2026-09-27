# GH-113 semantic action acceptance

The normal-discovery [action tests](../../../crates/mtg-core/src/actions_tests.rs)
use original hand-authored JSON records, not implementation-produced expected
records. The reachable 40-card green-mirror script keeps, plays lands, pays for
three Cubs, resolves Growth on a specific Cub, attacks into two same-name blockers,
assigns 3+2 damage and observes two dead blockers, four damage on the surviving
5/5, unchanged life, and cleanup returning it to an undamaged 2/2. Expectations
follow CR 103/117/305/601/608/400.7/508–510/514 and the frozen card text. Independent
fresh subprocesses run that same literal script with closed stdin.

Additional tests cover a reachable cleanup with exactly eight cards, duplicate
rejection for a synthetic two-card discard, restore into a fresh capability scope,
real mulligan/bottom identity, target/payment cancellation, every generic-payment
color, ordered Bite targets and lethal result, wrong actor/kind/card/owner/zone/
incarnation/birth/version, missing and extra fields, stale encoding, and concession
by either seat without priority. Explicit synthetic edge positions exercise
same-name copy selection, zone reentry and slot reuse. Invalid operations compare
full Game debug state, including RNG/history. Successful encode/decode equality
is consistency evidence in addition to, not instead of, hand-written expectations.

[Initial compiled red](red.txt) records three failures at the deliberately
unimplemented codec boundary. [Concession red](concession-red.txt) records the
missing command before its implementation. [Strict fieldless-action regression](fieldless-red.txt) caught Serde accepting
an extra index on unit variants; explicit empty structs now reject it. The
first full-suite attempt also caught this same regression during release tests.
Compile errors are not behavioral red evidence. Test setup was corrected to put two blockers on the battlefield
before requesting a division (a single blocker has automatic damage), and to
bottom the redrawn mulligan hand before the next declaration. No expected rules
outcome or existing assertion was weakened. Existing opening replay tests remain.

No Magic rule changed. Birth metadata is maintained separately from compact
object/slot records and preserved by snapshots. The codec delegates legality to
the existing authoritative policy/core path on an isolated snapshot; it does not
introduce a fallback, new rule or future replay envelope. Original #19 catalog
acceptance and #22 milestone gate remain outstanding. M2–M5 are unaffected.

README documents the standalone API and limits; no quickstart command or stage
verdict changes. Full regression, reference, review and CI receipts are recorded
in the delivery workpad/PR; final local receipts accompany this report.

Two [compiled mutants](mutations.txt) ignoring birth or incarnation were caught
by the duplicate/reentry/slot-reuse test. Production source was restored.
[Cached XMage combat](xmage-combat.json) agreed on five existing scenarios and
20 checkpoints, offline/headless with bounded timeouts. This verifies existing
combat results, not the custom codec, privacy, full games or Forge.

[Final full verification](verification.json) passed after current-main integration:
134 Python tests and 252 Rust tests plus one doctest in each debug/release
profile, with docs/program/catalog/fmt/Clippy checks. The fresh-process test
requires exactly one successful child test and enforces a 30-second bound.
