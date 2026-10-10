# Scalar sampled latency evidence (GH-278)

Status: bounded latency implementation with executable regression evidence.
Consult the latest verification, protected-delivery and independent-review receipts in the [GH-278 workpad](https://github.com/pabloxrl/mtg-lab/issues/278#issuecomment-6093342402).
No measurement qualification or M2 completion is claimed.

Oracle: RFC 0002 B008 requires decision latency p50/p95/p99; B020/B021
require fixed-cardinality local sampled histograms and independent policy and
encoding time. Tests use literal arithmetic, not timings copied from the engine.

The version-1 boundary contract has six labels: reset, transition,
legality_and_view, policy, encoding, finalization. Each label samples attempt
ordinals 0, 64, 128, ... independently across the run, without game RNG draws.
Skipped attempts remain in the denominator. Policy initialization and choose
attempts are policy work. Submission and advance are transition work. Buckets
are disjoint inclusive upper bounds 0, 10, 100, 1000, 10000, 100000, 1000000 ns,
then infinity. Percentiles use nearest rank and return the containing bucket's
inclusive integer lower/upper bounds, never interpolated exact latency.

A hand-authored one-choice clock transcript assigns policy initialization 3 ns,
reset 10 ns, legality/view 20 ns, encoding 100 ns, policy selection 7 ns,
application 30 ns and finalization 40 ns. Every-boundary test configuration
therefore yields policy sum 10 ns and rules/application sum 30 ns; neither the
100 ns encoding nor 7 ns policy selection can be folded into application time.
The single opening keep choice cannot finish a rules game (CR 103), and the
existing decision limit truncates it. Longer normal-reset full-deck prefixes
check unchanged histories, actions, state/RNG checkpoints and captured records.

Unavailable clock observations, unmeasured encoding and scalar-inapplicable
queue/inference timings remain distinguishable from measured zero. Failed
operations retain attempt/error counts; backwards clocks fail explicitly.
Saturation sets overflow rather than wrapping. Local merges reject incompatible
configuration or malformed accounting before changing their destination.

No rules or reference bridge changes are made. Pinned reference receipts in
prior M2 deliveries remain evidence for their unchanged tested semantics;
reference games cannot be an oracle for fake-clock arithmetic. No new benchmark
workload version, measurement campaign or performance qualification is claimed.

## Existing equality assertion requiring review

`benchmark_real_native_attempt_and_clock_phases_preserve_semantics` currently
compares entire JSONL byte output from two counters-mode runs, one without
encoding/timers and one with encoding and a fake clock. B021's new sampled
summary necessarily differs between those configurations. The proposed
requirement correction compares all pre-existing JSONL fields exactly, excluding
only the newly introduced latency object, and independently checks that object
against its clock/availability/accounting contract. No gameplay, capture, counter
or legacy phase-total expectation is removed. The prescribed independent review
must explicitly assess this correction and its replacement timing coverage.

## Executable acceptance

Seven new native execution tests pass: literal clock separation, default real
summary, off zero clock reads/interval-two denominators, backwards clock rejection,
real reset/policy/application error attempts and multi-run collector merge,
large-duration saturation, and eight normal full-deck matchup/start rows across
all four modes, both capture choices and both encoding choices (128 runs).
Each matrix run executes 64 choices and compares normalized full state/RNG,
semantic history and captured decisions. Only fresh publication UUIDs and existing
process-local object scopes are normalized; all game content remains compared.
This is played-prefix equivalence, not a new complete-game reference claim.

Three pure histogram/config/merge tests and two real exported-artifact Python tests
also pass in ordinary discovery. Two additional Python regression cases cover
the CI-discovered controller-environment/target-directory defect in the test
harness. There are ten new Rust tests and four new Python tests. The first candidate `432473823196de739ed946f6e7bae82520d5e451` passed full
locked `./scripts/torture.sh` against main `a44d7bfde8ae78fc3da3a2caf0063b023b5bbaba`: 266 Python tests and
1,440 Rust debug/release executions, zero failed/ignored. This includes
`python3 scripts/run_tests.py` and `cargo test --workspace --locked`.
The focused Python command also passes with two nonempty real-client tests:

```sh
python3 -m unittest discover -s tests -p 'test_m2_repair_timing.py'
```

Full local validation log SHA-256:
`f7972230d685f1a6eae5f45e11c9f4c9f9edaed574f097914499953e8141aad4`.
The protected PR/CI and exact-candidate review receipts are linked from the
workpad above; delivery is conditional on those checks, not this local receipt.
README support wording and the existing counters example are included in review.
The [previous full-pool policy evidence](../full-pool-policy/README.md) and
[scoped M2 reference audit](../m2-mechanics-audit/README.md) remain applicable to
unchanged rules; no new reference bridge or clock-arithmetic reference claim.



The first PR checks job exposed a test-harness portability defect: the pinned
image sets a controller-root environment variable even in isolated verification,
and Cargo's target directory is outside the read-only source tree. New behavioral
red cases are retained in `red.txt`. The harness now leaves resource scheduling
to its caller and honors `CARGO_TARGET_DIR`; in Symphony, wrap focused and full
commands with the existing shared heavy lock. No alternate lock, skipped test,
CI policy change or fallback engine is introduced. The original two real native
artifact tests remain mandatory. Consult the workpad for the corrected candidate’s full rerun and exact-head review.
