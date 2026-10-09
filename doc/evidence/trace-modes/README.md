# Bounded diagnostic modes acceptance

Issue #215 owns the bounded mode/buffer/capture-independence portion of RFC B021
and B020. Original #25/#26 acceptance remains authoritative. No game rule,
recorder format, reference bridge, RL path or performance qualification changes.

## Independent oracle

RFC B021 permits diagnostic drops only with an explicit counter, forbids silent
loss of promised replay data, isolates private replay fields, and makes trajectory
capture independent of instrumentation. The existing played-replay format requires
complete semantic choices, pinned versions/seeds and checked checkpoints. Existing
canonical publication requires explicit failure and no committed manifest after
writer failure.

The minimized normal-reset trace uses seed 215: two keeps, three alternating
priority passes, then P1 concedes. There are five accepted decisions and six
semantic records (concession is out of band). Sampling every second accepted
decision selects 2 and 4; capacity one retains 2 and drops exactly one. These
counts follow the script and sampling contract, not generated engine goldens.

Normal-discovery tests in `crates/mtg-core/tests/trace_modes.rs` cover sampling,
mode/capture equality, existing-format replay verification, explicit replay size
failure, record-bound failure without rules mutation, and immutable bounded
configuration. Existing `metrics.rs` equivalence/clock tests extend to four modes
without removing prior assertions. CLI canonical capture tests cover full-replay
write failure and trajectory persistence with instrumentation off.

The API-only scaffold compiled and all five focused tests failed on the intended
missing behavior; [baseline receipt](red.txt). After implementation all five pass,
as do all ten existing metrics tests extended across four modes. Focused CLI
public-output and canonical writer-failure/off-persistence checks pass.

Full torture, independent exact-candidate review and protected delivery are
required before completion. Their final receipts are kept in the
[delivery workpad](https://github.com/pabloxrl/mtg-lab/issues/215#issuecomment-6079735470)
and linked PR; this document is the oracle/coverage record, not a milestone verdict.
The implementation base is `1077a249cbe1546bdb5250f928a2c4931a3cb081`.
No new rules require a new reference scenario; preserved rules/reference checks
remain in the full suite, and this issue makes no additional XMage/Forge claim.
