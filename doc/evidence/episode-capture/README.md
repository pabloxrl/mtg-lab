# GH-160 canonical capture acceptance

Related to #160; partial R0002-B036/B037. No milestone/full-collector claim.

Normal discovery: `cargo test -p mtg-core --test capture` and
`cargo test -p mtg-core owned_capture_preserves --lib`. Full delivery uses
`./scripts/torture.sh`, fresh-main integration and separate candidate review.

## Independent contract and rules basis

The new `tests/capture.rs` script starts the delivered driver through normal
reset (seed 160, ordinal 0, explicit valid green mirrors). Each submitted command
forms an independently authored ledger row with actor, kind, complete choices and
expected logical/microchoice status; the recorder is never used to generate that
ledger. Exact observations/domains/masks actually delivered to the caller are
retained before submission and compared to every canonical row. Named boundaries
add independent literal rules expectations (CR 103.5, 104.3a, 117, 601, 608,
508–510, 514, 704). Semantic history is only a second completeness/reapplication
check, not the ledger's oracle.

The 27-turn played game keeps both sevens (33 cards remain in each library),
plays lands, pays for a Cub on turn 3 and defending Cubs on turns 4/6, cancels
Growth targets, cancels its payment, then retries and commits on turn 7. Growth
makes the attacker 5/5. All two-subset attacker choices, four blocker subsets
for two blockers/one attacker, and six nonnegative sum-5 damage allocations are
independently enumerated, including replacement/backtracking. Final allocation
[2,3] kills both 2/2 blockers simultaneously; the 5/5 survives four damage.
Cleanup removes the boost and damage. Ten later unblocked 2/2 attacks reduce
20 life to zero in literal two-point steps. Captured observations, full choices,
indices/status, supplied/absent statistics, final views, and both seat sequences
are checked. Terminal action returns are exactly [+1,-1] once; no concession row
is fabricated in this game.

Every submission is paired with capture-disabled execution and exact normalized
full snapshot/RNG comparison (only process-local owner capability scopes are
normalized). Rejected stale/wrong-actor submissions are injected before every
valid row and preserve full snapshot, history and capture. Actual semantic
history is reapplied from normal reset to the identical full final state/RNG.

Other normal-reset cases cover ordered two-bottom lists (and rejected missing
second choice), consecutive/interleaved/both/nonacting seats, both concession
winners, zero-decision seats, supplied policy values versus absent statistics,
real shuffled reset, invalid header/stats, stale/duplicate/direct submissions,
next-domain capacity overflow, unfinished settlement, result ownership across
reset and post-finalization writes. Per-seat elapsed decisions/logical actions/
cancellations are independently counted over the submitted ledger. Hidden-hand
and library twins keep P0's full seat sequence equal while P1's hand differs;
provisional targets/payment are visible only to their actor.

`src/episode_capture_tests.rs` is explicitly a **synthetic component edge**:
nine cards are installed at cleanup before handing the real Game to the real
Driver/Recorder through a test-only constructor. It checks all nine discard
candidates, ordered [8,0], missing-second rejection, seven remaining cards and
nonacting-seat rewards. This fixture is not normal-reset played acceptance and
cannot be constructed through the production API. The full game above supplies
that acceptance; no fake driver/recorder or future sibling is substituted.

## Red evidence and fixture development

[red.log](red.log) preserves the compiled intended failure of three initial
regressions against API scaffolding that delegated execution but retained no
capture. The failures are missing accepted rows/owned canonical episode, not
compile/import errors. All remain in normal discovery.

New-script corrections before acceptance were checked against existing contracts:
private payment taps do not change public tapped flags before commit, so the
script explicitly supplies two distinct lands; documented target kinds are
`growth_target` then `targets_complete`; one blocker needs no damage-allocation
choice, so the nontrivial script supplies two blockers; each blocker has one
recipient and deals its damage automatically; two defending Cub casts require
two opening Cubs; hidden twins must exchange distinct identities, not two Cubs.
No preexisting tests or expected rules outcomes were changed or weakened.

README scope and quickstart impact: only the in-memory capture/API disclosure
changes. Existing CLI commands remain unchanged; CLI capture and the full
collector remain planned. Full torture exercises existing CLI/workspace checks.
Separate review and exact delivery/CI receipts are recorded in the issue workpad
and PR; completion is conditional on those gates.
