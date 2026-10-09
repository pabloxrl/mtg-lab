# Available-container full-pool scalar baseline

Measured implementation: `e64842ed6c8529d70997ba1d508607e5c19c66cd`, based on
main `362aad0eeb5c7ab38d4b710de3f63c266ec48a52`, on 2026-10-09.
This is the bounded #217 scalar artifact, not whole-RFC certification, M2 gate
completion, or designated-host qualification. Independent review and protected
delivery receipts belong in the issue/PR workpad.

The release executable SHA-256 is
`db8a96e08e14be53347bee590cbe24a581b6e0e2ed878cf1e1290522dba6aeac`;
the embedded source hash is
`052f657b724a6ab05287cd7ff94c6e4dbc733cfa95849c5ac300382094d796b5`.
Rust 1.98.1, default release optimization level 3, no extra Rust flags.
The Linux ARM64 container has four logical CPUs/quota and a 7-GiB memory limit.
Throughput was pinned to logical CPU 0 under the shared heavy lock. CPU model and
physical-core allocation are unavailable; host contention and temporal drift
cannot be excluded. This is not the required designated x86-64 host.

The [frozen plan](plan.md), [commands and definitions](../../scalar-baseline.md),
[environment](environment.json), and per-run receipts describe reproduction.
Wording correction: the plan's “twenty-card decks” means the **20-card pool**;
the executed frozen red and green decks each contain **40 cards**, as defined in
`crates/mtg-core/src/opening.rs`. No deck, seed, horizon or selection rule changed.
The original plan bytes remain available for its recorded checksum.

## Timed windows

[Collection and aggregate comparison](throughput/collection.json) retain all
five runs: four initial runs plus the required heuristic/counters extension.
Each has a ten-second warmup; initial runs have five 30-second windows and the
extension has ten. Initial heuristic/counters variation was 10.74%, triggering
the frozen greater-than-10% rule. Its extension was slower and remains included.
No omitted, contaminated, profiled, failed or shortened interval was accepted.
Warmup outcomes are retained separately in each raw report.

| Policy / mode | Windows | Completed | Unfinished | Not started | Decisions/s | Games/s | OS peak RSS (MiB) |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Heuristic / off | 5 | 785 | 5 | 0 | 3172.34 | 5.2332 | 23.13 |
| Legal-random / counters | 5 | 181 | 4 | 1 | 2989.78 | 1.2066 | 23.24 |
| Legal-random / off | 5 | 184 | 5 | 0 | 3036.12 | 1.2266 | 23.37 |
| Heuristic / counters, initial | 5 | 729 | 5 | 0 | 2942.99 | 4.8599 | 23.37 |
| Heuristic / counters, extension | 10 | 1247 | 10 | 0 | 2541.59 | 4.1565 | 23.37 |

All measured windows report zero failures, truncations, concessions, draws and
counter/capacity overflows. Their 3,126 completions comprise 1,514 seat-0 wins
and 1,612 seat-1 wins. Thirty boundary attempts remain explicit: 29 unfinished
games and one attempt stopped before reset. Natural completions alone contribute
to games/s. All eight matchup/starting-seat rows have separate attempt and
completion counts in the raw JSON. Every distribution sample, nearest-rank
percentile, phase duration and logical-action count remains in those files.

Pooling all retained windows gives heuristic/counters **2675.39 decisions/s**,
an observed **15.66%** reduction against off; legal-random's reduction is **1.53%**.
Heuristic misses the provisional 5% overhead target; legal-random meets it in
this campaign. Both miss 100,000 decisions/s by a wide margin. The fixed run order
and shared host limit causal interpretation of overhead, especially across the
slower extension. These are observed comparisons, not confidence bounds or a
claim that counters alone caused the difference.

## Resident memory

[Capture](resident/capture.json) completed all 64 normal-reset games. Each chosen
semantic prefix was replayed through Driver before restoring copies of its
reached position. Stress scores were nine Goblins, fifteen legal object targets,
and ten stacked/pending effects; the typical state was the first turn-five
position. [Summary](resident/summary.json) and the four class reports retain
all 24 resident points per class, reset-churn readings, release readings, vector
capacities, OS RSS/PSS/private/shared pages and independent process receipts.

| Class | First-cycle marginal KiB/state | First-cycle total RSS at 10,000 (MiB) | Third release RSS (MiB) |
| --- | ---: | ---: | ---: |
| Typical | 4.79 | 51.88 | 63.59 |
| Token-heavy | 5.54 | 65.73 | 72.36 |
| Target-rich | 5.09 | 60.81 | 69.77 |
| Stack-heavy | 7.09 | 76.32 | 76.73 |

All classes reached 10,000 states and met the provisional 64-KiB marginal
core-state hypothesis under the declared first-cycle estimator. Inline `Game`
size is 1,632 bytes; serialized snapshot bytes are reported separately. Neither
number substitutes for RSS. The zero-state base includes process/shared data,
snapshot input and retained replay/parser allocations. Resident policy, history,
trajectory, ready and inference buffers are absent. Three complete reset/release
cycles expose substantial allocator retention; they do not establish leak freedom
or a general concurrent-game capacity guarantee.

## Allocation, boundary and dominant costs

[Profiler receipts](profiles/receipt.json) retain every command and exit code.
All four heap/PC collections and both CPU-only collections, including their
metric/function/heap/statistics/overview exports, returned zero. The original
[campaign script](profile-candidate.py) reproduces the diagnostic commands from
the repository root with an existing resident capture, verified release binary,
the shared heavy lock, and fresh `profiles` output directories. Exported heap
reports are losslessly gzip-compressed here. Raw experiment-file hashes are
retained for heap collections; the multi-gigabyte temporary experiments are not
committed. Full exported call-stack allocation reports remain available.

| Played specimen game | Decisions | Allocations reported | Bytes allocated (cumulative) | Tool-reported outstanding bytes |
| --- | ---: | ---: | ---: | ---: |
| [Typical](profiles/typical.heap.txt.gz) | 593 | 1,370,253 | 331,527,660 | 80,758,100 |
| [Token-heavy](profiles/token-heavy.heap.txt.gz) | 3006 | 7,724,876 | 1,959,151,563 | 496,703,505 |
| [Target-rich](profiles/target-rich.heap.txt.gz) | 2818 | 7,064,744 | 1,779,462,111 | 417,351,721 |
| [Stack-heavy](profiles/stack-heavy.heap.txt.gz) | 799 | 2,017,846 | 490,083,886 | 101,123,675 |

These are whole normal games containing each selected position, not isolated
single-position operations. All four completed naturally. Call stacks include
snapshot restore/serialization and semantic action validation/encoding.
Cumulative allocation is not live resident size. gprofng labels outstanding
allocations “Bytes Leaked”; preserve that output without treating it as an
independently established engine leak. Process shutdown, tool interception and
allocator retention need further diagnosis for any leak claim. No allocation-free
or leak-free claim is supported by this artifact.

Disjoint benchmark boundaries assign **98.44–98.93%** of each run's measured
elapsed time to the native transition call. Legality/view accounts for
0.48–0.87%, policy selection 0.07–0.11%, and finalization 0.02–0.06%; raw nanoseconds
also retain reset and outer-loop remainder. Extra benchmark encoding was off;
this does not remove the native semantic/snapshot work inside transitions.

Separate CPU-only profiles retain a complete ten-second warmup and five
thirty-second windows per policy. They are excluded from the rate comparison.
The [heuristic](profiles/cpu-heuristic.functions.txt) and
[legal-random](profiles/cpu-legal-random.functions.txt) reports assign respectively
**26.44% / 24.25%** of exclusive sampled CPU to SHA-256 compression;
JSON string formatting is **5.94% / 6.50%**, JSON string scanning
**4.88% / 5.12%**, and inclusive `malloc` **11.81% / 12.19%**.
Inclusive costs overlap and must not be summed. Unresolved libc symbols remain
unresolved rather than being assigned an invented operation.

These rank dominant sampled costs, not calibrated absolute CPU seconds:
gprofng reports 16.011 sampled CPU seconds for each roughly 160-second experiment,
while its process statistics report roughly 160 CPU seconds. That discrepancy
is retained as a profiler limitation. Heap tracing additionally spends substantial
time in its own collector. No cache-miss, hardware-counter, synchronization or
multicore-efficiency claim follows from these software samples.

## Correctness and delivery limits

The [independent oracle and behavioral-red receipts](oracle.md) explain literal
accounting expectations, negative cases and preserved regression inputs. Complete
passing [focused](focused-e64842e.log.gz) and [torture](torture-e64842e.log.gz) logs
cover the measured implementation after current-main integration, including
debug/release full-pool normal play, replay, capture, work-quantum equivalence and
rejection nonmutation. No existing test was removed, skipped or weakened.

This change adds measurement adapters, not mechanics or a reference bridge.
The prerequisite component/reference receipts remain authoritative; #23/#24/#25
and #26 retain their integration acceptance. In particular, the delivered
[activation-payment repair](https://github.com/pabloxrl/mtg-lab/pull/260) and
its [exact-main CI](https://github.com/pabloxrl/mtg-lab/actions/runs/37956881953)
precede this campaign. Local agreement and benchmark arithmetic are not
independent Magic rules oracles.

No Python/batch/inference throughput, trajectory-storage qualification, language
comparison, optimization, new RL work or M5 host qualification is claimed.
Hardware performance counters/cache events are unsupported here; perf,
Valgrind, heaptrack and GNU time are absent. Linux `wait4` and proc memory data
provide independent OS accounting; unavailable facilities are not counted as a
passing qualification.
