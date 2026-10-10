# London opening chronology oracle

GH-270 uses CR 103.5 at the unchanged 2026-09-25 pin. The official
source was fetched and its complete raw SHA-256 independently matched
`8d860e451f20f38865b725b42d82feb714c725373dd8f3b32b8652b3eeb070ca`.
CR 103.5 requires declarations in starting-player order, then simultaneous
mulligans: shuffle the previous hand back, redraw seven, bottom the cumulative
mulligan count in the chosen order, then repeat declarations. A kept player
never declares again. CR 101.4e treats the starter as active during setup.
No free mulligan applies to this two-player non-Brawl format.

The unchanged opening-counts independent ledger and GR-010/011/012/020/021
remain additional requirements. They do not prove occurrence identity.

The new oracle specifies physical copies in manifest creation order,
explicit permutations, complete declaration/bottom inputs, witnessed hand and
library boundaries and the first upkeep. Expected outcomes are authored from
these rules and inputs, never exported from either engine. Both starters and
RG/GR/RR/GG include no mulligans, one mulligan, unequal multiple mulligans
and repeated mulligans by both players. Ordered bottom choices include
distinct same-name physical copies.

## Explicit fixture arithmetic

For each of the eight deck/starter rows, the profiles specify starter/opponent
mulligan counts `(0,0)`, `(1,0)`, `(2,1)` and `(3,3)`. Every declaration is
written before the round's redraws. Kept seats are absent from later rounds.
For zero-based redraw round r and seat s, the supplied top-first permutation
is a rotation of manifest creation order by r+s. The new hand is exactly the
first seven occurrences. For cumulative count n, select those first n in
reverse order. The remaining library is positions 7..39 followed by that
chosen list. The retained hand is the other seven-minus-n occurrences.
These formulas use only authored chance/choice inputs and CR 103.5, not
native or reference observations. Initial permutations retain the independently
authored version-1 reset permutations.

Distinct basic-land copies make the reversed two/three-card suffix observable.
The next declaration hand sizes are 6, 5 and 4 after the first, second and third
mulligans. Every bottom boundary begins with seven cards. Library lengths at
those next declarations are 34, 35 and 36. The final first-upkeep checkpoint
includes both seats independently, with life 20 and no first-draw action.

## Upstream chronology review

At unchanged XMage `000d8a7abc0ac31cc24af08691423e0c24dc59e7`,
`Mulligan.executeMulliganPhase` asks all unkept players in starter order and then
calls `LondonMulligan.mulligan` for each mulliganing player. That method returns
the hand to the library, calls `shuffleLibrary(null, game)`, redraws seven and
executes one-card `chooseTarget`/bottom moves until the cumulative reduced
size is reached. Its old comment labels this 103.4, but the reviewed current
rule is 103.5. Source hashes match the retained original opening provenance:
`LondonMulligan.java` = `abea450526a3ebcfe6d29c9a52656e474f79f8b6e60d9bb74639eb6083c5b80b`,
`Mulligan.java` = `8753cf8165f4a71aedd2e1029c6783bc99e8ba8ef87997dd3ca081ac0c8d32db`.

Native and XMage serialize the simultaneous independent redraws differently:
native redraws both seats before bottoming, while XMage finishes each seat.
The version-2 contract therefore has separate exact chance and choice streams,
not a fabricated shared global callback order. Compare each actor's witnessed
hand/library at declarations and the redraw/bottom boundary, then both final
hands/libraries. Retain raw internal callback order separately. The independent
rules do not require the opponent's intermediate hidden hand to match while
simultaneous independent mulligans are serialized. No engine state is replaced
with oracle data, and no equality of these intermediate cross-player states is
claimed. The reviewer must check this explicit comparison boundary.

Each selected bottom list is checked against the current hand and round before
submission; its physical suffix is checked at the next observed boundary.
Reference internal RNG and rollback are unobservable and cannot be called
agreement. Native explicit chance consumption and transactional rejection are
checked against complete private state/RNG. Rejection controls validate the
adapter plus selected production actions; they do not claim exhaustive legal
set equivalence.

## Adapter repair diagnosis

The initial native private-helper call and XMage target overload were compiler
integration errors, not behavioral red evidence. The first compiled reference
run reached upkeep but `GameImpl.playPriority` caught the stop exception and
retried it. The adapter now uses the existing `game.pause()` API, without
submitting a pass; an explicit stop flag and exact upkeep check verify the stop.
The next run exercised all played prefixes and negative inputs but the extra
callback probe found null `choosingPlayerId` after opening; comparison now treats
that as an explicit unexpected-boundary rejection rather than dereferencing it.

After these repair cycles the revised approach is to check lifecycle boundaries
against the pinned API, keep all failure artifacts, require the exact callback
controls to complete, and rerun both actual reference executions plus normal
regressions. No rules expectation, pin, existing test or required gate changed.

A further repeat check correctly exposed raw unordered pre-shuffle data:
`red-green-0-keep.raw_callbacks[0].state[1].library[0]` varied between two
Forest copies before seat 1's initial shuffle. Both complete semantic point
lists and consumed ledgers were equal. Repeat equality applies to those
ordered/witnessed contracts; raw unshuffled set iteration is preserved in both
runs without claiming equality, as already specified by the reset protocol.
This comparison boundary is explicitly subject to independent review. New
missing-source and mulligan/bottom-source controls strengthen schema coverage.

The native test client uses master seed 270, episode 0 and the retained default
environment RNG version. Both initial and replacement permutations are explicit;
the test checks the complete RNG state remains unchanged by the opening choices.
XMage UUIDs and pre-shuffle set iteration are not seeded-equivalent claims.
