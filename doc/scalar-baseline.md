# Full-pool scalar measurements

The scalar collectors measure the existing native client and Driver. They add no
rules, policy optimization, batching or training path. The measurement contract
is [scalar-windows-v1](scalar-benchmark.md); `scalar-full-pool-v1` rotates normal
resets through RG, GR, RR and GG, each starting seat, by episode ordinal modulo
eight. Its report retains attempts and natural completions for every row.
The 20,000-decision, 100,000-work-call and 20,000-record horizons are unchanged.
The historical green/green `scalar-windows-v1` command remains available.

The [experiment plan](evidence/full-pool-baseline/plan.md) freezes order, seeds,
policy versions, minima and extension criteria. The [recorded container
artifact](evidence/full-pool-baseline/README.md) includes raw results and limits.
Run from the repository root
inside the managed Linux toolchain container. The Python collectors require the
existing shared heavy-work lock; they do not build, install tools or change the
lock directory. Use new output directories for every attempt:

```sh
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- \
  cargo build --release --locked -p mtg-cli
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- \
  python3 scripts/collect_resident_baseline.py --output /tmp/mtg-resident-new
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- \
  python3 scripts/collect_scalar_baseline.py --output /tmp/mtg-scalar-new
```

The throughput collector runs both frozen policies, each with `off` and
`counters`. Each run takes at least 160 seconds: ten-second warmup and five
thirty-second windows. A window-rate coefficient of variation above 10% triggers
an additional ten-window run. Both runs remain in the aggregate; nothing is
replaced with the faster result. Every failed or incomplete run retains its
available report, process receipt and stdout/stderr. A failed run prevents a
successful aggregate artifact. Collection is followed by correctness review; the
collector itself never certifies the rules or a milestone.

Each scalar report has separate raw output and a copy with a collector receipt.
The receipt uses Linux `wait4` for actual per-process CPU time, high-water RSS,
faults and context switches. It records cgroup quota/memory limits and CPU
affinity before and after. One allowed logical CPU is pinned for the campaign;
this is not a physical-core reservation. External host contention remains a
shared-container limitation. Profiler/loader injection is rejected; changed
binary identity, affinity or resource limits mark a run contaminated. The
independent validator rejects that run, omitted/short intervals, changed game
horizons, bad outcome/row/rate arithmetic and incomparable policy/mode pairs.

Validate individual receipt-bearing files with:

```sh
python3 scripts/scalar_artifact.py /tmp/mtg-scalar-new/heuristic-off.json
```

`collection.json` retains all runs, OS receipts, variability and the aggregate
comparison. Counters overhead is `100 * (1 - counters_decisions/s / off_decisions/s)`
over all retained valid windows. A deadline can expire between scheduling an attempt and its reset;
`attempts - started` retains that unstarted boundary attempt without inventing a
game outcome. Raw completion, decision, logical-action and
boundary-time distributions remain available. The provisional targets are
hypotheses; a miss does not justify lowering a horizon or changing rules.

## Resident states and stress

`scalar-resident-v1` has two strict diagnostic requests: `{"operation":"capture"}`
and `{"operation":"sweep","specimen_path":"PATH"}`. The resident collector
orchestrates both through `mtg bench --workload scalar-resident-v1 --config FILE`.
Capture plays 64 normal-reset games, cycling all eight rows with both existing
policies. It retains every attempt and selects the first turn-five position and
the largest observed token, target-choice and stack/trigger positions. Required
stress floors are four Goblins, six legal object-target choices and three stacked
or pending effects. Missing stress or failed games make capture unsuccessful;
there is no synthetic substitute or lowered floor.

Samples include configuration, ordinal, decision index, actual observed shape,
privileged snapshot, its checksum and complete semantic action prefix. Each
sweep replays that prefix through the authoritative Driver and checks its shape
and public state against the saved snapshot. The replayed normal-reset state
supplies the resident copies. These are repeated copies of a declared reached
position, not 10,000 distinct played games or a concurrent progression claim.

Each class runs in a fresh process. Three allocation/release cycles retain
0, 1, 32, 128, 512, 1,000, 5,000 and 10,000 core `Game` states. Linux process
status and `smaps_rollup` supply RSS, high-water, PSS and shared/private pages.
The report distinguishes snapshot bytes, inline type size and vector capacity
from resident memory. The zero-state base includes process/shared tables, one
snapshot input and retained prefix-replay/parser allocations. Resident policy,
ready, inference, history and trajectory buffers are absent. First-cycle RSS
increase divided by 10,000 is the declared marginal estimate; retain the whole
curve to see allocator/page effects. Each cycle then resets all 10,000 core states to the ordinary red/green opening
(seed 42, ordinal equal to slot), records reset-churn RSS, and releases them.
Released RSS exposes allocator retention;
three cycles do not establish general leak freedom. No allocation-free claim
is made.

Snapshots and action prefixes contain both seats' private game information.
They are privileged generated experiment artifacts, never live player exports.
Memory restoration/parser costs are not rules-transition throughput. Independent
heap/PC profiles are separate diagnostic runs and cannot enter off/counters
speed comparisons. Unsupported hardware counters, missing tools and deferred
Python/batch/inference/recording/designated-host tracks must remain explicit in
the published evidence. A container result does not qualify the designated
Linux x86-64 performance host or complete M2 acceptance.
