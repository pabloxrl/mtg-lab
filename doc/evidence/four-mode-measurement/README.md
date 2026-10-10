# Four-mode scalar measurement (GH-281)

The delivered four-mode runners have been measured on a correctness-gated release
binary, with both native policies and both encoding settings across all eight
matchup/starting-seat rows. This is scoped measurement evidence, **not M2 or
designated-host qualification**. No production runner or engine feature changed.
Partial additive evidence covers R0002-B008, B014, B019, B020 and B021;
all original requirement owners and aggregate acceptance remain.

## Campaign and accounting

Collection: 2026-10-10T11:00:55Z through 2026-10-10T11:55:37Z. The frozen plan produced
18 retained runs and 100 measured windows, including both legal-random/full-replay
extensions. Every configuration starts with 10s warmup and five ≥30s windows;
initial population CV >10% triggers exactly one additional 10s + ten ≥30s run.
Repeated seeds are repeated executions, not independent unique-game samples.

Measured totals: **7,204 natural completions**, 7,204 attempts,
3,091.847304s denominator; seat wins [3511, 3693], draws 0,
errors 0, truncations 0, unfinished 0,
concessions 0, pre-reset/not-started 0.
The windows processed 7,181,389 decisions; their full per-kind and logical-action
counters are retained where instrumentation is enabled. Separate warmups retain
504 attempts and 190.091372s.
All failed/unfinished work would remain in elapsed time and outside the completed
numerator; literal injected-clock and tampered-artifact controls enforce this.

The [raw throughput archive](throughput.tar.gz) and [file index](throughput-index.json)
retain every configuration, window, collector receipt and original manifest.
The [recomputed summary](summary.json) contains every throughput/overhead sample,
direct boundary costs, outcome totals, process receipts, latency buckets and profile metrics.
The separately labeled [privileged diagnostics](privileged-diagnostics.tar.gz)
and [index](diagnostics-index.json) retain trace inputs, checkpoints and raw reports;
these are administrative evidence, never player observations.

**Original validation failure retained:** collection used the pre-correction validator
and ended with `status: failed`: `all eight rows must be observed per interval`. It incorrectly required
eight warmup games rather than ten warmup seconds. All native invocations succeeded.
The separately corrected analysis validates all raw reports against the actual
contract; no run was retried, removed or selected for speed. The original validator
and intended positive assertion failure are preserved in [validation history](validation-notes.md).

## Comparable throughput

H = heuristic, R = legal-random; native = unencoded, encoded = Observation JSON.
Rates below use total count / total elapsed time. Overhead is decision-throughput
loss versus off in the same policy/track. The p50–p95 columns summarize **all
cross-product window comparisons**, which are correlated descriptive samples,
not paired episodes, confidence intervals or causal estimates. Off/off spread
is a variability reference. Each configuration cycles all eight rows; the runner
has no per-row elapsed denominator, so no per-row throughput is inferred.

| Policy/track | Mode | Windows | Games/s | Decisions/s | Aggregate overhead % | Overhead p50 / p95 % | Initial / pooled CV % |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| H/encoded | counters | 5 | 5.796 | 3507.91 | 0.00 | -0.10 / 1.45 | 1.56 / 1.56 |
| H/encoded | full_replay | 5 | 1.243 | 735.88 | 79.02 | 79.08 / 79.40 | 4.38 / 4.38 |
| H/encoded | off | 5 | 5.797 | 3508.04 | 0.00 | 0.00 / 1.44 | 1.48 / 1.48 |
| H/encoded | sampled_trace | 5 | 5.806 | 3514.08 | -0.17 | -0.05 / 1.25 | 1.46 / 1.46 |
| H/native | counters | 5 | 5.866 | 3548.09 | 0.76 | 0.96 / 2.19 | 1.32 / 1.32 |
| H/native | full_replay | 5 | 1.242 | 734.86 | 79.45 | 79.46 / 79.86 | 3.80 / 3.80 |
| H/native | off | 5 | 5.910 | 3575.36 | 0.00 | 0.00 / 2.28 | 1.57 / 1.57 |
| H/native | sampled_trace | 5 | 5.940 | 3591.12 | -0.44 | -0.27 / 1.53 | 1.78 / 1.78 |
| R/encoded | counters | 5 | 1.355 | 3327.92 | -0.67 | -0.43 / 1.42 | 5.47 / 5.47 |
| R/encoded | full_replay | 15 | 0.300 | 713.75 | 78.41 | 78.47 / 79.14 | 13.89 / 12.40 |
| R/encoded | off | 5 | 1.347 | 3305.85 | 0.00 | 0.00 / 2.99 | 5.71 / 5.71 |
| R/encoded | sampled_trace | 5 | 1.349 | 3311.95 | -0.18 | -0.10 / 2.25 | 5.50 / 5.50 |
| R/native | counters | 5 | 1.383 | 3399.29 | -0.51 | -0.46 / 2.43 | 5.02 / 5.02 |
| R/native | full_replay | 15 | 0.301 | 714.13 | 78.88 | 78.82 / 79.52 | 13.70 / 11.47 |
| R/native | off | 5 | 1.376 | 3381.99 | 0.00 | 0.00 / 2.88 | 5.64 / 5.64 |
| R/native | sampled_trace | 5 | 1.380 | 3391.30 | -0.28 | -0.53 / 2.87 | 5.72 / 5.72 |

## Direct cost and memory evidence

| Policy/track/mode | Reset s | Finalization s | Policy s | Replay s | Encoding s / bytes | Peak process RSS MiB |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| H/encoded/counters | 0.003102 | 0.083091 | 0.134411 | 0.000000 | 3.586238 / 3572530323 | 23.75 |
| H/encoded/full_replay | 0.001044 | 0.017950 | 0.028955 | 121.074103 | 0.753955 / 747031572 | 313.75 |
| H/encoded/off | 0.003406 | 0.082771 | 0.133061 | 0.000000 | 3.572306 / 3572530323 | 23.75 |
| H/encoded/sampled_trace | 0.003166 | 0.083367 | 0.133058 | 0.000000 | 3.616619 / 3581507603 | 23.88 |
| H/native/counters | 0.003315 | 0.084358 | 0.131689 | 0.000000 | unavailable | 23.25 |
| H/native/full_replay | 0.001044 | 0.017904 | 0.029064 | 121.142556 | unavailable | 313.76 |
| H/native/off | 0.003806 | 0.084993 | 0.138612 | 0.000000 | unavailable | 23.12 |
| H/native/sampled_trace | 0.003177 | 0.085516 | 0.132741 | 0.000000 | unavailable | 23.38 |
| R/encoded/counters | 0.000911 | 0.029938 | 0.123122 | 0.000000 | 4.141121 / 4969865010 | 24.00 |
| R/encoded/full_replay | 0.000926 | 0.020522 | 0.082889 | 382.700373 | 2.780357 / 3319668361 | 863.34 |
| R/encoded/off | 0.000895 | 0.029865 | 0.122187 | 0.000000 | 4.149867 / 4942845617 | 24.00 |
| R/encoded/sampled_trace | 0.000881 | 0.029427 | 0.120902 | 0.000000 | 4.141297 / 4942845617 | 24.00 |
| R/native/counters | 0.000921 | 0.030181 | 0.123238 | 0.000000 | unavailable | 23.50 |
| R/native/full_replay | 0.000918 | 0.020190 | 0.084363 | 381.606702 | unavailable | 863.28 |
| R/native/off | 0.001009 | 0.030765 | 0.127042 | 0.000000 | unavailable | 23.50 |
| R/native/sampled_trace | 0.001073 | 0.029958 | 0.127892 | 0.000000 | unavailable | 23.62 |

The JSON additionally preserves transition, legality/view and publication boundaries,
encoded-vs-native overhead distributions and every process wall/CPU receipt. Reset
and finalization remain inside elapsed time. Process RSS includes the whole client;
it is not marginal core-state memory.
The 64 MiB replay byte limit is not a process-RSS ceiling: legal-random full
replay reached about 863 MiB peak RSS including verification and client work.

## Fixed-profile observations

| Trace hash prefix / mode / capture | Choices | Elapsed ms | Application ms | Legal/view ms | Isolated dispatches / ns | Allocations / requested bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 0701f81aff / full_replay / False | 156 | 312.666 | 64.017 | 0.300 | 2 / 250 | 5163166 / 775583511 |
| 30ba9ab534 / counters / False | 156 | 68.103 | 64.982 | 0.245 | 2 / 334 | 551197 / 127521810 |
| 59fcdd8f50 / sampled_trace / True | 156 | 70.351 | 65.994 | 0.151 | 2 / 83 | 566322 / 133478285 |
| 7554ee70c9 / full_replay / True | 156 | 312.891 | 65.054 | 0.154 | 2 / 500 | 5178290 / 781539917 |
| 7957154bb7 / off / True | 156 | 69.269 | 65.018 | 0.150 | 2 / 125 | 566321 / 133478211 |
| 9ef3dda058 / counters / True | 156 | 70.439 | 66.058 | 0.155 | 2 / 292 | 566321 / 133478216 |
| a98f0595ee / off / False | 156 | 68.100 | 65.043 | 0.248 | 2 / 83 | 551197 / 127521805 |
| bfae162c00 / sampled_trace / False | 156 | 67.985 | 64.912 | 0.259 | 2 / 251 | 551198 / 127521879 |
| 03d54d9d6b / off / False | 6 | 2.464 | 2.222 | 0.004 | 0 / 0 | 19604 / 4651781 |
| 03d54d9d6b / off / False | 6 | 2.517 | 2.276 | 0.004 | 0 / 0 | 19604 / 4651781 |
| 3958cba4b7 / off / False | 434 | 194.999 | 185.839 | 0.500 | 0 / 0 | 1581800 / 367272383 |
| 4f400a387c / off / False | 156 | 66.954 | 63.951 | 0.247 | 2 / 126 | 551197 / 127521805 |

These finite traces show application dominates the ordinary profile runs; full
replay adds substantial shared replay/allocation work. The control and combat
fixtures have no isolated modifier dispatch; the effects fixture witnesses two.
The JSON retains direct encoding/reset/finalization and all isolated samples.

## Sampled native latency bounds

The transition bucket summaries below pool separate fixed episodes, not timed windows.
All other phase buckets, skipped/error denominators and quantile bounds remain in JSON.

| Policy/track/mode | Transition samples / attempts | p50 ns | p95 ns | p99 ns |
| --- | ---: | ---: | ---: | ---: |
| H/encoded/counters | 152 / 9414 | 100001–1000000 | 100001–1000000 | 100001–1000000 |
| H/encoded/full_replay | 152 / 9414 | 100001–1000000 | 100001–1000000 | 100001–1000000 |
| H/encoded/sampled_trace | 152 / 9414 | 100001–1000000 | 100001–1000000 | 100001–1000000 |
| H/native/counters | 152 / 9414 | 100001–1000000 | 100001–1000000 | 100001–1000000 |
| H/native/full_replay | 152 / 9414 | 100001–1000000 | 100001–1000000 | 100001–1000000 |
| H/native/sampled_trace | 152 / 9414 | 100001–1000000 | 100001–1000000 | 100001–1000000 |
| R/encoded/counters | 555 / 35216 | 100001–1000000 | 100001–1000000 | 100001–1000000 |
| R/encoded/full_replay | 555 / 35216 | 100001–1000000 | 100001–1000000 | 100001–1000000 |
| R/encoded/sampled_trace | 555 / 35216 | 100001–1000000 | 100001–1000000 | 100001–1000000 |
| R/native/counters | 555 / 35216 | 100001–1000000 | 100001–1000000 | 100001–1000000 |
| R/native/full_replay | 555 / 35216 | 100001–1000000 | 100001–1000000 | 100001–1000000 |
| R/native/sampled_trace | 555 / 35216 | 100001–1000000 | 100001–1000000 | 100001–1000000 |

## Provenance

Measured source commit: `2ec601cd778312871937ed242933052a365fad29`.
Production binary SHA-256: `591c16239509a7c126fb82026c761b134492ba0984eb732dfc04f35964e0c6b9`.
Native source fingerprint: `c67331f1be5327b6df094fcb081df31170c03caca558fe272e14e801a1d25e29`.
Diagnostic release test binary: `27381ddb99fdc2a5d6e5c9e4e4cf5d8135c59e84558032ca66cb64c451a0f402`.

Rust 1.98.1, `aarch64-unknown-linux-gnu`, release opt-level 3, empty rustflags.
Throughput and fixed profiles used logical CPU 0; the separate native latency
matrix retained affinity 0–3. The shared container quota was 4 CPU equivalents
and memory limit 7 GiB. No exclusive physical-core allocation or CPU-model
identity was available. All per-run before/after resource receipts are retained.
Trace sampling is every 64 decisions with capacity 256; complete replay capacity
is 20,000 records / 64 MiB with `in_memory` persistence. Canonical capture is off
for timed windows; verification/serialization stays inside their denominator.
Horizon is unchanged at 20,000 decisions / 100,000 work calls, quantum 64.

Main commit `94f0e0334f4037d815824bf8832c1d594e00755e` was integrated after
collection. Its Rust changes add London-mulligan tests inside the existing
`cfg(test)` reference module; production gameplay, benchmark and profile paths
are unchanged. The conservative native source fingerprint includes test files,
so a later rebuilt fingerprint is **not** relabeled as the measured binary.
The archived executable and original source identity remain the measurement pin.

## Interpretation and limits

The observed scalar decision rates fall far below RFC 0002's provisional
100,000 decisions/s hypothesis. This is an ARM64 shared-container observation,
not designated x86-64 host qualification or a physical-core efficiency claim.
Counters overhead is reported as a distribution; noisy negative overhead is not
an optimization or evidence that instrumentation accelerates the engine. There
is no invented games/s target. No horizon, rule, workload or outcome filter was
changed to improve a result.

Each window includes reset, policy, legality/view construction, transitions,
finalization and the requested replay/encoding work. The JSON publishes these
direct boundaries and all process wall/CPU/RSS receipts. Warmup and process
startup/reporting are retained separately. Phase boundaries describe the delivered
adapter, not pure isolated rules costs. The encoded track serializes the existing
authorized Observation JSON; it is not tensor encoding, Python batching or RL.

The separate fixed-episode diagnostic contains 128 natural normal-reset runs:
two policies × two encoding settings × four modes × eight rows. Observation,
history and final-state hashes agree for each fixed policy/row across all mode
and encoding variants. Its sampled native latency buckets include all attempts,
samples, skipped observations, errors and sums; pooled p50/p95/p99 are bucket
bounds, not exact quantiles. The benchmark runner itself honestly reports sampled
timing `not_measured`; no window histograms are fabricated. The native test target
has its own binary pin and observation-hash probe, so these timings are never
pooled with production benchmark windows. Off-mode latency is unavailable.

The delivered 64-run heuristic mode/capture matrix separately verifies canonical
capture reload, complete replay and player-view equality on all eight rows.
Its injected 7ns clock proves accounting only. Fixed-profile traces additionally
check all modes with capture on/off. Equality is established on fixed episodes
and traces, never inferred from totals of timed windows.

The privileged diagnostic archive contains 12 valid profile executions and seven
explicit invalid-trace controls, using the delivered control/combat/effects inputs.
The original checkpoints, trace hashes and rules/card/source/toolchain/dependency
pins are retained. Application includes semantic decode, validation, settlement
and internal legality; exposed legal/view generation is a separate direct boundary.
The witnessed effect probe isolates pending `Work::Modify` dispatches in a separate
experiment. It is not application-minus-encoding or inferred per-effect cost.
Allocation count/bytes measure requested successful allocation/reallocation traffic,
not live memory or RSS. Tiny isolated dispatch samples do not qualify CLI latency.
Hardware cache counters, sampled call stacks, multiworker scaling, queue latency
and concurrent client-buffer capacity are unmeasured, not zero.

Historical [GH-217 resident/stress evidence](../full-pool-baseline/README.md)
remains intact, including all four 10,000-state sweeps and original speed misses.
This change adds no production buffer or Game field. Those core-state capacity
claims exclude the newer optional client replay/capture buffers; the current
process RSS receipts do not extend them to concurrent buffered clients. No
capacity rerun is claimed or required for an unchanged core-state claim.

No rules or reference adapter changed. The fixed-profile engine pin and reference
fixtures remain unchanged; reuse the [policy reference basis](../full-pool-policy/README.md),
[GH-215 replay receipt](https://github.com/pabloxrl/mtg-lab/issues/215#issuecomment-6079735470)
and [delivered profile oracle](../fixed-trace-profile/README.md). No XMage comparison
was manufactured for clock arithmetic. Original owners and acceptance remain;
this evidence does not complete the M2 gate, Python/batch/RL or CLI contracts.

## Reproduction and validation

The [frozen plan](plan.md) and [campaign orchestration](campaign.py) consume the
delivered collector; they add no production benchmark feature or CLI surface.
The exact measured aarch64 production executable is retained in
[measured-mtg.gz](measured-mtg.gz), with [binary identity](binary.json). It is an
evidence artifact, not a portable installation package. Preserve the recorded
source revision when reproducing it; never relabel a later binary as measured.
For a fresh campaign, correctness-gate the selected source/binary first, then
call `run(binary_path, new_output_directory)` from `campaign.py` under the shared
heavy lock. The output directory must not exist. The function freezes the same
16 configurations, retains every invocation and applies the declared extension
rule; it returns the original collection status instead of deleting failed work.

To verify archive integrity and recompute every published statistic from the
repository root (no native execution):

```sh
python3 - <<'PY'
import json, runpy, tempfile
from pathlib import Path
p = Path('doc/evidence/four-mode-measurement')
unpack = runpy.run_path(str(p / 'archive.py'))['unpack_verified']
summarize = runpy.run_path(str(p / 'summarize.py'))['summarize']
with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    unpack(p / 'throughput.tar.gz', p / 'throughput-index.json', root / 'throughput')
    unpack(p / 'privileged-diagnostics.tar.gz', p / 'diagnostics-index.json', root / 'diagnostics')
    assert summarize(root / 'throughput', root / 'diagnostics') == json.loads((p / 'summary.json').read_text())
print('Archived measurement evidence validated')
PY
```

Normal discovery includes literal positive/negative accounting controls, the
archived-artifact regression and actual native adapter/client executions. The
focused interface is:

```sh
python3 -m unittest discover -s tests -p 'test_m2_repair_measurement.py'
```

In a Symphony worker, run it and all Cargo/full-suite commands through the
existing shared heavy lock. `python3 scripts/run_tests.py` and
`cargo test --workspace --locked` remain normal delivery interfaces; full
`./scripts/torture.sh` additionally checks release execution and strict linting.
See [preparation and correction history](validation-notes.md) for literal red
assertions, independent arithmetic and the review-confirmed stronger replay
expectation. No pre-existing regression or ownership was removed or weakened.

The [integrated validation receipt](validation.json) and
[full torture log](integrated-torture.log.gz) record 12 focused tests, 303 normal
Python tests, and 744 Rust tests in each of debug and release (1,488 executions),
with no failures or ignored tests. Formatting and strict Clippy passed. This adds
12 Python cases and two Rust cases to normal discovery. The documented archive
reproduction command also passed. The [precollection gate log](precollection-validation.log.gz)
preserves the original correctness gate and the later aggregate-validator failure.
Final independent review, protected merge and exact-main CI receipts are recorded
in the [issue workpad](https://github.com/pabloxrl/mtg-lab/issues/281#issuecomment-6096505485);
delivery remains conditional on those receipts.
