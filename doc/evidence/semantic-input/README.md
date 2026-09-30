# Owned semantic input acceptance

Related to #176; partial R0002-B020/B039/B041 only. #120 retains every composed
CLI clause, #21 retains integration/catalog acceptance, and #22 remains the M1
gate. No rules, action format, CLI, recorder or execution-loop replacement.

Normal discovery: `cargo test -p mtg-core --test semantic_input --locked`.
The five tests in [semantic_input.rs](../../../crates/mtg-core/tests/semantic_input.rs)
exercise the delivered codec, Driver, budgets and canonical capture. The first
four tests compiled against the API-only rejecting stub and failed on valid
opening input; [behavioral-red.log](behavioral-red.log) preserves that evidence.
Compile errors during test authoring are not behavioral evidence.

## Independent expectations and scope

The original GH-113 Cub/Growth case supplies the rule-derived script, now
submitted through the owner in all four capture/budget mode combinations.
Normal reset uses complete frozen green decks with explicit order, master 176,
ordinal 0. Opening hands are Cub, Cub, Growth, Forest x4. CR 103 gives seven
cards per seat and 33 remaining, with P0's first draw skipped. CR 305 permits
one land per turn. CR 601 requires two mana for Cub and one green for Growth;
CR 608 gives the chosen Cub +3/+3. On turn 7 its 5/5 attack is blocked by two
2/2 Cubs: the explicitly assigned 3/2 damage kills both, simultaneous damage
marks four on the surviving 5/5, and blocked combat leaves life [20,20]
(CR 508–510). Cleanup leaves an undamaged 2/2 (CR 514). P0's concession gives
returns [-1,1] (CR 104.3a and RFC sparse seat reward contract).

Handwritten semantic records, stable birth/incarnation references, literal
checkpoints, and a record ledger are independent of encoder output. Each
accepted input is compared with the existing ordinary `submit` path, including
exact history bytes, action-time observations/submissions, absent policy
statistics and complete trajectory. That equality is supplementary, not the
rules oracle. Every transition compares full snapshot/RNG between owners;
only process-local store/scope identity is normalized. Final snapshots/history
also agree across all four modes. An earlier finished episode deliberately makes
owner revision differ from the core's revision. Concession adds one history
record and no fabricated policy decision; footer, per-seat rewards and outcome
accounting are checked exactly once. Both seats can concede before acting,
including the seat without priority, with unassigned rewards and zero decisions.

## Rejection and recovery

Full owner debug equality checks private state/RNG, pending work, history,
trajectory, status/accounting and live tokens around each ordinary rejection.
Synthetic input corruptions cover malformed/empty/missing/version/kind/choices,
wrong seat, birth/card/owner/zone/incarnation, extra fields, old decision/reset
context, wrong context kind and foreign/stale concession episode. Reusing a
land's departed hand reference and playing a second land are illegal. Internal
reset and opening-to-turn inputs cannot advance or poison capture. Valid input
continues afterward, including a real shuffled mulligan; its bottom choice uses
the existing policy API and is not represented as a handwritten shuffle oracle.
Terminal/finalized input cannot add records, rewards or outcomes.

Decision limit one and record ceiling one demonstrate the existing owner
contract: the former explicitly finalizes truncation; the latter quarantines
recording when the next valid input needs a slot. Neither accepts the attempted
choice or mutates game/RNG/history. Later input preserves the entire stopped
owner. These intentional finalization effects are distinguished from invalid
input mutation; prior budget regressions remain unchanged.

## Delivery validation

Full managed-container `./scripts/torture.sh`, fresh-main integration/rerun,
clean-candidate prescribed separate review (including README), protected merge
and exact merged-main CI are required. Their exact candidate/run/merge receipts
are maintained in the [Agent workpad](https://github.com/pabloxrl/mtg-lab/issues/176#issuecomment-5913846408)
and the delivery PR. This document is scoped executable acceptance, not a claim
that a pending gate passed. Existing tests and all RFC/catalog owners remain
unchanged. README now describes only the new privileged library input; existing
quickstarts/CLI commands retain their prior scope and are exercised by the full
suite. No throughput, reference-engine agreement or whole-M1 claim is made.
