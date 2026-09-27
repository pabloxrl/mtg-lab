# GH-65 opening-choice acceptance

One atomic delivery: R0002-B015/B016 opening continuations, not complete RFC
blocks or M1. Prerequisite #64 supplied the actual reset hands. Expected
semantics are independently specified by pinned CR 103.5/121.1, RFC 0002 §4,
and the unchanged [catalog](../../testing/capability-test-plan.json).
[Rules-source review](../../testing/rules.md) explicitly establishes bottoming
after every mulligan. No game output was used to invent expected results.

## Red/green

[red.txt](red.txt): `cargo test -p mtg-core mulligan` compiled against new API
stubs that returned the unchanged decision / refused draws; 13 behavioral
assertions failed, one metamorphic test passed. The assertions remain in normal
test discovery. A first local run revealed an unbounded test-helper loop against
the no-op stub; the helper was bounded to the maximum two keep declarations
before capturing this complete red run. No compile failure is counted as red.

[green.txt](green.txt): same command after implementation, plus independent
shuffle-vector and invariant tests. [torture.txt](torture.txt): full
`./scripts/torture.sh` in managed Linux Docker after integrating current main,
including all debug/release Rust regressions, Python tests, fmt/clippy and
program/catalog/doc validation. The first full run caught Clippy's large enum
variant; bottom handles are now retrieved separately from the compact decision.
No tests were removed, skipped or weakened; no catalog expectations changed.

[vectors.py](vectors.py) uses the independent Python integer RNG oracle,
frozen deck manifest and specified shuffle order to derive [vectors.json](vectors.json).
The Rust test checks every hand/library card after two rounds, for both starting
seats. This is an independent arithmetic check, not mature-engine agreement.

## Exact catalog mapping

All tests below are in `crates/mtg-core/tests/mulligan.rs`, run by ordinary Cargo
test discovery. Test names are `mulligan_` followed by the catalog ID with hyphens
replaced by underscores.

| Catalog ID | Executed checkpoint |
| --- | --- |
| rules-setup-mulligan-bottom-positive | One mulligan, bottom Mountain, six cards, Mountain last, then keep |
| rules-setup-mulligan-bottom-negative | After second redraw, one-card selection fails and complete state/RNG are unchanged |
| rules-setup-mulligan-bottom-interaction | P0 bottoms two distinct cards in selected order; P1's kept hand remains unchanged |
| rules-setup-mulligan-bottom-regression | First bottom reduces to six before next declaration; second redraw seven then bottom two |
| rules-setup-deterministic-reset-positive | Independent same-seed games with both real keeps have identical ordered zones and semantic next decisions |
| rules-setup-deterministic-reset-interaction | Same-seed real mulligan, replacement shuffle and bottom match |
| rules-setup-deterministic-reset-regression | Consume two rounds of both-seat shuffles, reset; fresh game and subsequent mulligan match |
| rules-setup-ordered-draw-positive | Ordered green library Forest/Bear Cub/Giant Growth: Forest drawn, Bear Cub next, new zone handle |
| rules-setup-ordered-draw-negative | Card selection at declaration is wrong-kind; draw during opening rejected; library unchanged. Draw primitive has no card-selection parameter, per catalog's protocol-probe allowance in rules.md |
| rules-setup-ordered-draw-regression | Bottom Mountain then Swab Goblin; draw all 35; final two draws have that exact order |
| rules-decisions-stale-candidates-regression | Old reset decision and old candidate paired with current decision both rejected |
| rules-setup-two-player-opening-positive | Actual both-keep sequence, two starters, 20 life, seven-card hands, 33-card libraries |

`mulligan_sys_core_003_rejections_between_every_real_stage` submits wrong actor,
stale decision, foreign game token, out-of-range/padding index, wrong kind,
missing choice and duplicate application at reachable opening stages. Bottom
uniqueness, cardinality and candidate generation have additional focused tests.
Every rejection checks complete private Debug state (including RNG, counters,
pending choices and orders) plus retained capacities. A separate control game
checks subsequent valid results. This is the assigned **opening** portion;
payment/target/combat continuations remain with their registered siblings.

Additional normally discovered tests cover invalid replacement permutations,
both-seat red and green mirrors with both starters, unchanged hands until all
declarations arrive, seven-mulligan forced keep, stale cross-seat bottom
candidates, independent full-order vectors, zero RNG consumption for explicit
replacement orders, and decision exhaustion in both declaration/bottom kinds.
The ordered-draw primitive reports empty library without mutation; rules loss
and turn timing remain #72/#67 scope.

## Limits and references

The current pinned XMage/Forge bridges execute only a synthetic priority-pass
smoke. They do not accept opening/mulligan scenarios; there are no impacted
cached opening reference scenarios to execute. No upstream agreement or
reference-verified opening capability is claimed. Expanded neutral/reference
integration remains with #17/#24/#38 as registered; these native tests establish
the assigned scalar behavior only. Snapshot, observations, turns, terminal
rules and full-game reset acceptance are separate deliveries.

README now documents usable opening application and the draw primitive while
retaining explicit limits and unchanged milestone verdicts. Independent review,
PR and exact-main CI receipts are recorded in the GH-65 workpad and PR after
this candidate is committed.
