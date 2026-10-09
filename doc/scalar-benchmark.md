# Versioned scalar benchmark

`scalar-windows-v1` extends `mtg bench` with repeated scalar measurement windows.
It uses the existing native client and authoritative Driver. It does not qualify
engine speed, the full card pool, or the complete RFC benchmark tracks. The
[contract evidence](evidence/scalar-benchmark/README.md) distinguishes deterministic
accounting tests from performance measurements. Full-pool profiling remains #217;
aggregate acceptance remains #25 and #26.

Inside the toolchain container, build and run:

```sh
cargo build --release --locked -p mtg-cli
target/release/mtg bench --workload scalar-windows-v1 \
  --config fixtures/bench/scalar-windows-v1.json --output /tmp/scalar-benchmark.json
```

The output path must be new. The sample takes at least 160 seconds: ten seconds
of warmup followed by five thirty-second measurement windows. Longer intervals
and more windows are supported (up to 3,600 seconds per interval and 100 windows).
Shorter production minima, fractional/nonpositive intervals, incompatible
workload/schema versions, unknown fields and missing/unsupported policies are
errors (exit 2). Tests inject a clock internally; there is no short-window CLI
override. The old `scalar-pass-v1` and `native-rollout-v1` smoke commands retain
their original contracts and are not comparable to this workload.

## Frozen workload and reproducibility

[The dedicated episode fixture](../fixtures/bench/scalar-workload-v1.json) pins
normal-reset green/green games, starting seat 0, quantum 64, a 20,000-decision
horizon, 100,000 work calls and 20,000 history records per attempt. No synthetic
state, shortened production horizon or substitute rules implementation is used.
Current native policies support the frozen full pool; this is not the historical
six-card M1 limitation. A contract smoke game is not full-pool qualification.

The config must name both native policy versions. Use the same heuristic on both
seats for a heuristic baseline, and both legal-random identities for a separate
random baseline. The example selects heuristic on both seats. Seeds and
instrumentation (`off` or `counters`) are explicit. `encoding: true` additionally
serializes each acting player's actual authorized observation to temporary JSON
memory; the default native track does not. This is JSON encoding, not a numeric
tensor, Python, inference or batched collection measurement.

Artifacts include the resolved episode/config, master/policy seeds, RNG versions,
deck source, rules/card manifests, policy sources, action/observation schemas,
workload fixture/source digests, compiled source digest and binary hash. Episode
ordinals start at zero and continue through warmup and windows; each window
records its first ordinal and attempt count. A window boundary can leave the last
game unfinished; replay its ordinal without the measurement stop to investigate.
Do not compare different workload/policy/source pins as an engine speedup.

Hardware metadata records CPU model, RAM, OS/kernel, effective CPU affinity,
available logical CPUs, one worker, rustc/LLVM version, target, optimization level,
Rust flags, debug assertions and build commit. The source digest disambiguates
local edits from that commit. Missing host metadata is null, never invented.
The benchmark does not set affinity or claim an allocated physical-core count;
control affinity/load externally and retain the reported mask. No games per
CPU-second/core efficiency, memory capacity, allocation or reference-AI speed
claim is made.

## Accounting and timing

Each raw window retains all elapsed nanoseconds, starts, natural completions,
wins/draws, concessions, truncations, failures, unfinished episodes, decisions,
work calls and measured phase totals. Counters mode also retains the existing
fixed counters, including decision kinds, logical actions and rules work units.
Off mode reports unavailable counters as null. Work calls are not rules work units
or logical actions. Decisions/game uses only decisions from completed games.

Only natural rules-terminal outcomes enter completed games/s. Concessions,
external limits, failures and unfinished games do not. A zero-completion window
has a null successful completion rate and makes the run exit 3, with its complete
report retained. The distribution explicitly retains numeric zero samples for
such windows; it cannot turn the run's failed status into a throughput claim.
Raw samples stay in execution order; summaries use arithmetic means and
nearest-rank p50/p95, with min/max. Inspect variation and extend the run when
needed rather than selecting the best window. No speed threshold is asserted.

| Field | Measured work |
| --- | --- |
| `elapsed_ns` | Entire window: owner/policy setup, reset/shuffle, all game work, observations, policy attempts, history, finalization, JSONL summaries, aggregation and timer overhead. Includes failed/unfinished work and boundary overshoot. |
| `phases.reset_ns` | Driver reset and its initial bounded rules work, including rejected resets. |
| `phases.transition_ns` | Driver submission and advance attempts, including validation and semantic action encoding. Reset is separate. |
| `phases.legality_and_view_ns` | Seat probes through Driver observe, including authorized views and legal candidate construction. The API combines these costs; this is not pure legality time. |
| `phases.encoding_ns` | Extra actual observation JSON serialization when requested; null in the native track. |
| `policy_ns` | Policy initialization and all choose attempts, including failures. |
| `phases.finalization_ns` | Driver finish, including final observations, snapshot and history. |

These phase intervals are disjoint subsets of elapsed time, not standalone
fixed-trace microbenchmarks. Unattributed runner/report/clock overhead remains in
elapsed time. There are no per-rules-operation clocks and no core wall-clock
dependency. Benchmark timers are enabled independently of scalar counters and
their overhead stays in the denominator; neither mode is an untimed speed claim.

Configuration validation, hardware discovery and executable startup precede
warmup. Final report aggregation and artifact output follow the windows and are
explicitly excluded. JSONL episode/report preparation within each native attempt
is included. Capture is disabled for this version; recording overhead, Python,
inference and worker/batch tracks remain separate obligations.

Window deadlines are cooperative at native-client work boundaries. The report
uses actual elapsed time, including overshoot. Failures and signals stop the run
with partial raw evidence, never a fabricated full window. SIGINT/SIGTERM retain
exit 130/143. A fixed 100,000-attempt safety bound per window fails explicitly if
it prevents the requested duration; no short run is accepted as a full window.

Reports contain seeds and configuration for reproducibility and are experiment
artifacts, not live player-facing telemetry. They contain no observation payloads
or hidden card order from the temporary JSON encoding.
