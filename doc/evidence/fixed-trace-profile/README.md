# Fixed-trace profile acceptance (GH-280)

This is a bounded native diagnostic client, not a throughput campaign or an M2
completion claim. The tests invoke the same Driver, semantic action executor,
observations and bounded work executor used by scalar games. No rules change or
reference-engine agreement is claimed by clock arithmetic.

## Independent inputs

`fixtures/profile/combat.json` retains every action of the reviewed GH-178
rules-terminal green trace. Its checkpoint life ledger is 20 until the first
unblocked Growth-enhanced Cub attack, then 15; eight later attacks deal two each
and end at -1. CR 103, 305, 601, 608, 508–510 and 514 justify opening, lands,
payment, Growth, combat and cleanup. See the existing
[script oracle and independent correction review](../script-cli/README.md).
The no-effect control retains the first five actions (two keeps, two passes,
one Forest) and an explicit concession: life stays 20/20, hands are 6/7 and
libraries 33/33. It is a complete diagnostic trace, not natural-game throughput.

The authored red trace keeps ordered full frozen decks, plays a Mountain each
turn, casts Firebrand Archer on turn 3, Dragon Fodder on turn 5 and Goblin Surprise
(mode 0) on turn 7. Each noncreature spell triggers one damage before its own
resolution: opponent life 20 → 19 → 18. Fodder creates two 1/1 tokens; Surprise
adds +2/+0 to all three creatures until cleanup. All combat choices explicitly
decline attacks. Cleanup expires all three boosts. After seven turns P0 has
7 + 3 draws − 4 lands − 3 spells = 3 cards; P1 has 7 + 3 draws − 3 lands = 7.
Both libraries contain 30 cards. P1 then concedes, completing the trace. These
expectations were written before executing the native adapter.

## Measurement boundaries

Correctness checks every declared checkpoint and complete input consumption
before reading the performance clock. A second execution measures reset,
semantic application/replay, legal observation generation, encoding and
finalization at disjoint boundaries. Replay application includes the existing
transactional semantic decoder's validation and scratch execution; that shared
work is explicitly unattributable within application, not mislabeled pure rules
cost. Residual client/checkpoint/bookkeeping work is reported separately.

Effect attribution is a separate isolated experiment. It uses existing bounded
priority-pass execution and witnesses pending `Work::Modify` operations, timing
one `Game::resume(1)` at a time. The first operation consumed while submitting
the pass and preparation/restoration costs are excluded and disclosed. Remaining
Surprise modifications provide repeated effect dispatches; the control has none.
These observations are not subtracted from transition totals or added twice.
No clock enters the rules core. Unavailable hardware counters/call-stack sampling
remain explicit limitations; isolated wall-clock boundary timing is not CPU
cycle attribution.

Final verification and delivery receipts are recorded in the
[issue workpad](https://github.com/pabloxrl/mtg-lab/issues/280#issuecomment-6095243229). Existing tests,
reference pins, rules/card data and capability claims remain authoritative.

First implementation execution: six of seven focused Rust tests passed. The
new red input was rejected (`script_rejected`) because the authored tape omitted
the required explicit single-trigger ordering selections after the two
noncreature casts. The delivered semantic protocol requires a complete
controller ordering under CR 603.3b even when the only permutation has one
member. Added `trigger_order: [0]` after each cast, preserving every original
record and every expected game result. The red tape now has 156 records; no
rules change or expectation weakening was used to repair the input.

## Run the diagnostic adapter

```sh
python3 -m unittest discover -s tests -p 'test_m2_repair_profile.py'
MTG_PROFILE_ARTIFACT_DIR=.agent-artifacts/profile-run-1 python3 -m unittest discover -s tests -p 'test_m2_repair_profile.py'
cargo test --locked -p mtg-cli profile::
```

Symphony must wrap these native build/test invocations in its existing shared
heavy lock, as required by WORKFLOW.md. The focused Python suite builds and invokes
the native test executable's `profile::profile_export` adapter. This avoids a new
product CLI/configuration surface. `MTG_PROFILE_INPUT` and `MTG_PROFILE_OUTPUT`
select explicit privileged input/output files when invoking that one native test.
The callable client is `profile::run` in the native test target; later measurement
work can reuse it. Standard discovery executes all seven Rust and four Python
cases, without a skipped/probe-only alternative.

Reports include exact trace, native source, executable, toolchain, dependency,
card and rules hashes. The retained input JSON is immutable during execution.
The fixed adapter uses work quantum 64 and a 100,000-call settlement bound;
unsupported deadlines/persistence and incompatible inputs reject explicitly.
Capture is in-memory and independently selectable. Full-replay mode creates and
verifies the actual complete replay during finalization. No policy runs on a
fixed semantic tape; policy time is explicitly zero. The existing
`PolicyTiming.encode` observation encoder is used for actual encoded bytes.

Allocation traffic is measured by a diagnostic-only thread-local counter around
`System` allocations and reallocations. It records successful allocation requests
and requested bytes, not retained heap size or RSS; deallocations are not negative
allocations. It neither replaces the production allocator nor counts other test
threads. Overflow invalidates the report. Main elapsed time and allocation totals
include all measured categories plus residual client/checkpoint/report work.
The correctness-gate run, input parsing, final artifact serialization, binary
hashing and separate effect experiment are outside the main measurement window.

No production rules or reference adapter changed. Reuse the pinned
[cast/ETB reference evidence](../full-pool-policy/README.md),
[Surprise evidence](../surprise/README.md) and existing combat/cleanup regressions
for unchanged semantics. This does not claim a fresh reference-family execution,
full-game reference agreement, sampled CPU stacks or hardware-counter data.


Focused acceptance PASS: seven new Rust tests and four new Python tests; full
format and Clippy checks pass. The [raw diagnostic archive](profile-run-1.tar.gz)
and [per-file hash index](profile-run-1-index.json) preserve 12 successful actual
native reports (1,850 observations), their exact inputs, and seven rejected input
controls. Eight reports cover all four modes × capture off/on. Semantic history,
player-input hashes and final views match. The red tape witnesses exactly two
isolated pending modifier dispatches after the first modifier consumed by pass
submission; the no-effect control witnesses none. Measured boundary nanoseconds
are diagnostic observations only, not a performance target or CPU-cycle estimate.
The final integrated regression run, independent review and protected delivery
receipts are recorded in the issue workpad/PR; this archive alone is not a
milestone or delivery verdict.


The initial complete torture execution passed 272 Python tests and 1,454 Rust
debug/release executions. A subsequent verification improvement added explicit
canonical capture-content equality (155 captured decisions in the red trace),
not just matching gameplay, observations and capture counts. Its intended
[assertion failure](capture-red.txt) precedes the added capture digest. Full
integrated verification is repeated for that final correction; the earlier
full run is historical evidence, not a substitute for final validation.
