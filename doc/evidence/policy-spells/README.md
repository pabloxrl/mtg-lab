# Safe spell decision acceptance — GH-111

Partial R0002-B016/B025/B033 component, not an M1 gate verdict. #19 keeps original
integration/catalog acceptance; #22 gates M1. M2–M5 obligations remain unchanged.
No combat, replay, CLI policy, collector or workflow implementation is included.

Independent basis: RFC 0002 §§4/7/8/9 requires real structured decisions, hidden
information invariance and transactional rejection. CR 601.2c/f/h/i requires
explicit targets and payment before a spell is committed; 117.3/4 retains caster
priority and resolves the top spell after consecutive passes. CR 107.4 permits G
plus any available color for 1G; 608.2b/h revalidates targets and uses current power.
Frozen Giant Growth gives +3/+3; Bite Down deals source power to its destination.

## Executable evidence

`cargo test -p mtg-core policy` discovers all old tests plus eight new tests.
[Initial compiled red](red.txt): payable creature choice absent; pending target
observation incorrectly unavailable. [Expanded red](red-acceptance.txt): three
response/payment tests fail on absent cast candidates. The cancellation test's
unknown wire variant is recorded but not counted as rules-behavior red evidence.
[Focused green](green.txt) records the final named run.

New tests cover:

- Real normal-reset green mirrors, explicit keeps, lands and paid Bear Cub casts
  through safe submissions. Two scenarios: Growth responses with literal 2→5→8
  stats, and Bite/cancellation with a literal lethal two damage result. Hidden
  library-tail twins have identical final policy records. No synthetic game
  state is used in these scripts and no combat choice is needed.
- Explicitly synthetic battlefield/pool setup followed exclusively by real safe
  submissions: Growth responds to Bite (defender becomes 5/5 with two damage);
  responding Bite kills the original source (one legal target, resolution with
  no source LKI damage). Public stack targets follow current battlefield rows;
  a departed source becomes null.
- Hidden hand/library twins: identical acting-seat tables and selected-state
  records at every cancellation stage; opponent view equals its pre-cast view
  throughout. Cancellation restores committed cards/taps/mana; generation advances.
- Source-before-destination masks, selected actor-only targets/pool/cost/sources,
  all six generic payment colors, no Finish when unfunded, explicit target finish.
- Every stage: exact capacity versus one below, every masked candidate, hidden-zone
  or guessed references, wrong seat, stale accepted requests, and invalid colors
  reject with exact full Game/RNG/history nonmutation.

The old pending-spell policy rejection expectation is replaced with stronger
positive/private-state checks and the above retained regressions. The independent
review must explicitly assess this requirement correction. The old schema-1
`observe` payment-unavailable test remains intact: that API has no continuations.
A literal old mask gains one false row for the newly represented, unaffordable
Swab Goblin cast (1R with only one Mountain). Its other assertions remain intact.
No tests are removed or skipped. An early green build's array-length compile
error is not behavioral evidence. The implementation's initial targeting
projection lost acting-seat information; unchanged privacy assertions caught it
and the shared projection now retains the actor during targeting.

## Validation and limits

Full `./scripts/torture.sh` and separate committed-head Codex review are mandatory;
full suite PASS after current-main integration: 134 Python tests, 234 Rust tests
and one doctest per debug/release profile, plus docs/graph/catalog/format/Clippy.
The first full attempt found a question-mark style lint, fixed without changing
behavior or expectations. Review and exact-main CI are recorded in the workpad/PR.
[Cached XMage priority receipt](xmage-priority.json) checks the existing pinned
priority fixture twice, offline/headless with closed stdin. The bridge does not
verify safe policy schemas, Growth/Bite effects or hidden-information twins.
Matched instant references remain #115; no Forge/effect agreement is claimed.

README and API docs describe exact supported choices. Existing quickstart commands
are unchanged and the full suite exercises their verification paths. Schema 1
has additive spell choice/projection fields; legacy opening APIs remain unchanged.
