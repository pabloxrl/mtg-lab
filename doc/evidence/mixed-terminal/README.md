# Per-seat mixed terminal reference contract

Acceptance evidence for GH-257. Native and pinned-XMage execution passed on
integrated main `fb8689267ee5d80750998c55e9580074ac12dc8d`. Full shared-lock torture
also passed; review and protected delivery receipts are maintained in the issue
workpad linked below.
This is a synthetic boundary adapter repair, not new production terminal rules,
normal-game reachability evidence, GH-212 aggregate acceptance or an M2 verdict.

## Independent expectations

The unchanged pinned rules govern the authored `terminal-v2.json` expectations:
CR 704.5a loses a player with nonpositive life; CR 121 and 704.5b lose a player
who attempted an empty draw, not one whose library merely became empty. CR 704.3
collects applicable state-based actions together, and CR 104.4a makes simultaneous
losses by all remaining players a draw. Under CR 104.3, a single losing player
leaves the other player the winner. Concession retains its CR 104.3a control.

The literal outcomes were written before extending either adapter. Both mixed
seat arrangements, both draw/life injection orders and both life-seat iteration
orders must draw. Single life/failed-draw losses cover each seat. Empty libraries
without attempted draws and successful last-card draws stay ongoing. The seven
v1 cases and all their existing input/expected fields remain byte-for-byte in the
original fixture; v2 repeats their expectations with additive observations.
No catalog setups/actions/expectations/bases/owners or old receipts are modified.

## Contract and observation boundary

Version 1 keeps explicit legacy interpretation (draw P0, concede P1, existing
four observation fields); version 2 requires the seat for every draw/concession.
Unknown fields, malformed seats, unsupported versions and conflicting actions
must fail. Version 1 output is never interpreted as a v2 result.

Version 2 records the actual consumed life setters/draw/concession and final
settlement. It snapshots the pending boundary before concession/settlement and
the settled boundary, including life, lost flags, library/hand counts, canonical
outcome, winner seat and explicit draw flag. Intermediate life and draw operations
must not award a winner. The adapter injects life and attempts at most one draw;
this bounded operation is explicit and does not model a reachable spell script.

The native fixture uses the existing object draw operation and terminal SBA
collector. XMage uses real Player.drawCards, checkStateAndTriggered and game
completion APIs. Expectations do not supply winner/lost/draw output. XMage's
winner is read after its game loop completes, since its winnerId is finalized
there; pending observations never invoke a game-over check to manufacture a win.

## Regressions and limits

Python behavioral red demonstrates ignored unknown fields and bool/integer
comparison aliasing (`python-red.log`). `native-red.patch` preserves the original
helper extraction and two unchanged semantic assertions before adapter repair. The compiled native red has two intended assertion failures, with 22 other selected
tests passing (`native-red.log`). The first repaired build had a macro syntax error
(`native-build-failure.log`); a queued retry overlapped temporary stash integration
and failed before test execution (`integration-rerun-failure.log`). Neither is an
agreement. The restored integrated run passed all 27 selected native tests
(`native-green.log`), and full Python discovery passed 232 tests
(`python-workspace.log`). The additional duplicate-key behavioral red and ten
focused Python tests are retained in their named logs. Normal test discovery must retain all
legacy and new tests. Fault controls must name the first divergent field and
preserve minimal wrong-seat and premature-settlement inputs.

No Forge, private view, general card interaction, full-game or reachability claim.
The empty-library synthetic start deliberately bypasses normal deck setup. The
pending checkpoint and consumed choices are privileged test observations.

## Reproduction

The managed container supplies the pinned Java/Maven/Rust tools. Prepare an
isolated XMage cache from verified build inputs, then execute:

```sh
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- \
  python3 scripts/terminal_reference.py --cache "$MTG_REFERENCE_CACHE" \
  --output /tmp/terminal-reference.json
```

The receipt is JSON; `/tmp/terminal-reference/` retains both repetitions, exact
fixtures, native/XMage logs and observations, all comparator leaf controls, the
minimal mixed-loss fault input and its actual-engine outputs. Native fixture
setup uses the existing reset seed 42 / episode 9 before replacing zones with
the explicitly synthetic empty-library boundary. XMage skips initial shuffling;
no random library choice participates in these cases. Life setters and draw
operations are recorded in order. No AI/controller fallback is admitted.

## Retained catalog boundary

| Original catalog case | Bounded evidence in this repair |
| --- | --- |
| `rules-terminal-simultaneous-loss-positive` | Both zero-life losses, including reversed life-seat injection order. |
| `rules-terminal-simultaneous-loss-negative` | Exact [0,1] life setup yields P1 winner, plus its seat-swapped and life-order controls. |
| `rules-terminal-simultaneous-loss-interaction` | P0 zero life plus P1 failed draw in one SBA batch, the seat-swapped case, both draw/life orders and both life-seat orders. |
| `rules-terminal-simultaneous-loss-regression` | Both-zero life state and reverse life-seat iteration both draw with neither seat winning. Additional strict-input, legacy, observation/choice and actual-fault controls supplement this unchanged expectation. |

This adds bounded evidence for partial R0002-B011/B026/B028/B030. GH-212 still
owns its full assigned catalog audit, composition and aggregate acceptance.
The catalog's original setups, actions, expectations, bases and owners remain
unchanged; this table is an execution crosswalk, not a catalog replacement.

## Full regression result

The complete `./scripts/torture.sh` passed after current-main integration, under
the managed container’s shared heavy lock. [Full log](torture.log): 233 Python
tests, formatting/Clippy and documentation/program/catalog checks, plus 695 Rust
test executions in each of debug and release (including subprocess executions
and doctests), with no failures or ignored tests. Log SHA-256:
`d0bcb9f4e8efd2e188e8e2ee17ad1ec85c878ba965ceb55ae030f2a7f86e0394`.

## Actual reference results

[Final versioned receipt](reference-final.json): all seven legacy and 31 v2 cases agreed
twice in actual native Rust and pinned XMage. All 75 input/source hashes and 129
raw artifact hashes were verified after execution. The receipt records the exact
Rust/Java/Maven toolchains, pinned upstream commit, rules/cards and bridge bytes.
The adjacent `reference-final/` directory contains exact inputs, consumed operations,
pending/final observations and logs from each repetition.

All 27 malformed inputs were rejected by the actual XMage parser and normal
native/Python discovery. All 980 observation/choice leaf corruptions produced
named first divergences. Both real adapters were also executed with two deliberate
faults against the retained `minimal-mixed.json` input: wrong draw seat first
diverges at `settled.lost[1]`; premature settlement at `pending.lost[0]`. Both
faults also preserve their incorrect final outcomes for inspection. Missing
observations or a build failure cannot produce this receipt.

The earlier `reference.json` receipt and `reference/` artifacts remain unchanged.
`runner-before-guard.py` and `tests-before-guard.py` preserve their exact source
versions. A subsequent behavioral test (`fault-guard-red.log` and its reproduction
patch) exposed a fault-control guard that accepted missing observations. The final
runner requires complete observations and the exact independently justified
first divergence; the final receipt repeats both real engines with this guard.

README documents the supported test contract, explicit legacy compatibility and
scoped evidence; the existing quickstart is unchanged. The affected reference
command is the actual repeated execution above. Independent review and protected
merge/exact-main CI receipts are recorded in the
[issue workpad](https://github.com/pabloxrl/mtg-lab/issues/257#issuecomment-6084882789).
