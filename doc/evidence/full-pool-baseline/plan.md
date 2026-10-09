# Available-container scalar experiment plan

This plan precedes measurement. It is not a speed or M2 acceptance result.

Scope: current scalar Driver, frozen twenty-card decks; RG, GR, RR and GG, each
starting seat, repeated in that order by episode ordinal modulo eight. Seed 42
for both streams; named `heuristic-activation-mana-v1` and
`legal-random-activation-mana-v1` on both seats in separate runs. Preserve the
existing 20,000-decision, 100,000-work-call and 20,000-record limits. Throughput
uses normal reset, no synthetic positions, no capture and no extra JSON encoding.
Ten-second warmup then five thirty-second windows per policy and off/counters.
Every attempt and unfinished boundary game stays in the elapsed denominator.

Run order: heuristic/off, random/counters, random/off, heuristic/counters.
Hold the shared heavy-work lock through the entire campaign and pin one allowed
logical CPU. Record container CPU quota, memory limit, CPU/OS/compiler/source and
binary identity, affinity, OS process CPU/RSS, and all exceptional outcomes.
This cannot reserve a physical host core or eliminate outside-host contention;
shared-container results are diagnostic, never designated-host qualification.
Keep every run, including unsuccessful or contaminated ones. Profiled runs are
separate and cannot enter the off/counters comparison. If window rate CV exceeds
10%, retain the original and collect an additional ten-window run with the same
configuration; summarize both, never select the fastest.

Resident measurement: sample actual native games for typical, token-heavy,
target-rich and stack/trigger positions. Record exact observed sizes and seed,
ordinal, policy and decision index, plus privileged snapshot and action prefix.
No synthetic replacement if a class fails to appear. In fresh processes restore
those core states at resident counts 0, 1, 32, 128, 512, 1,000, 5,000 and 10,000,
for three allocation/release cycles, each including actual normal RG resets of
all 10,000 states (seed 42, ordinal equal to slot) before release. Total RSS and high-water are OS measures;
serialized snapshot length is not resident memory. Separate shared/process base,
core states, snapshot input and absent policy/inference/ready buffers. Report
allocator retention, slope, per-state cost and cap failures explicitly; three
cycles do not prove absence of all long-run leaks.

Use independent GNU gprofng heap/PC profiling and Linux process accounting where
available. Separate allocation and sampling probes from speed windows. Report
unavailable counters/cache/synchronization facilities and unsupported Python,
batch, inference, trajectory-storage and designated-host tracks explicitly.
Compare the provisional 100,000 decisions/s, 64 KiB/core state, 10,000 residents
and 5% counters hypotheses; misses are evidence, not permission to change rules.

Correctness evidence: normal discovered regressions, full torture after current
main integration, prerequisite review/exact-main receipts, existing pinned rules
and reference checks, and independent arithmetic validation of every interval.
No new game mechanic or reference bridge is introduced. Existing stage/reference
audits retain their original acceptance. The artifact validator must reject
omitted/shortened/reordered intervals, bad outcome/rate arithmetic, overflow and
contaminated measurement contexts. Red assertions are retained with the artifact.
