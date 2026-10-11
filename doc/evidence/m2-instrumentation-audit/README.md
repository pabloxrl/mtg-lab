# M2 scalar instrumentation integration audit (GH-25)

Audit base: `c779b190ed28ff845741c4ce1149f71e55ec675b` (2026-10-10).
Verdict: **applicable scalar requirements satisfied, conditional on this report's
full validation, independent review, protected merge and exact-main CI**.
This is the bounded #25 integration audit, not M2 gate #26, a new measurement
campaign, or designated-host performance qualification. The
[current scope](../../programs/engine-validation.md) and
[atomic ownership](../../programs/m2-atomic-delivery.md#original-integration-ownership-and-complete-crosswalk)
retain later obligations and the unfinished reference acceptance.

The [earlier unsuccessful audit](https://github.com/pabloxrl/mtg-lab/issues/25#issuecomment-6090375546)
remains preserved. Its missing sampled histograms, four-mode overhead, encoded
track and fixed-trace cost evidence are now supplied by #278–#281. No original
requirement, regression, source pin or failed measurement was removed. This
delivery changes documentation/evidence only; it needs no new behavioral-red
test and does not relabel child red/green receipts as fresh execution.

## Reviewed prerequisite receipts

Both direct dependencies are closed completed, with acceptance and separate
review in their workpads. Their exact-main CI success was re-fetched for this audit.
Child implementation reviews are additional evidence, not this audit's review.

| Delivery | Protected PR / exact merge | Exact-main CI | Acceptance and independent oracle |
| --- | --- | --- | --- |
| #214 counters | [#248](https://github.com/pabloxrl/mtg-lab/pull/248), `1077a249cbe1546bdb5250f928a2c4931a3cb081` | [37920865357](https://github.com/pabloxrl/mtg-lab/actions/runs/37920865357) | [Literal counts/privacy/overflow](../scalar-metrics/README.md) |
| #215 modes | [#250](https://github.com/pabloxrl/mtg-lab/pull/250), `29ad7f00e4df2bc54d56d8e7a3079be094996ed9` | [37928470735](https://github.com/pabloxrl/mtg-lab/actions/runs/37928470735) | [Sampling, complete replay, capture independence](../trace-modes/README.md) |
| #216 workload | [#251](https://github.com/pabloxrl/mtg-lab/pull/251), `522b7da8b86cd412c03c709bf88d8540c0385e16` | [Completion receipt](https://github.com/pabloxrl/mtg-lab/issues/216#issuecomment-6080297251) | [Frozen contract and arithmetic](../scalar-benchmark/README.md) |
| #217 baseline (direct) | [#267](https://github.com/pabloxrl/mtg-lab/pull/267), `71776215b7c2372757b37c4113ac1c0a28bb1362` | [37996690481](https://github.com/pabloxrl/mtg-lab/actions/runs/37996690481) | [Raw windows, OS memory and software profiles](../full-pool-baseline/README.md), [completion/review](https://github.com/pabloxrl/mtg-lab/issues/217#issuecomment-6082444182) |
| #278 sampled latency | [#283](https://github.com/pabloxrl/mtg-lab/pull/283), `4068c855d1bcfbe9f1c63dfd5c98a44752e8c95c` | [38034577373](https://github.com/pabloxrl/mtg-lab/actions/runs/38034577373) | [Injected-clock oracle and real native summaries](../sampled-latency/README.md) |
| #279 four-mode contract | [#284](https://github.com/pabloxrl/mtg-lab/pull/284), `3744ffab7ac2571fcb59e4aae2ff744810eee5b5` | [38038753842](https://github.com/pabloxrl/mtg-lab/actions/runs/38038753842) | [New version, preserved v1 rejection and complete-or-fail accounting](../four-mode-benchmark/README.md) |
| #280 fixed profiles | [#286](https://github.com/pabloxrl/mtg-lab/pull/286), `e1ee7bfc2715a04e0f7e3a29a827466d342565d3` | [38043420315](https://github.com/pabloxrl/mtg-lab/actions/runs/38043420315) | [Independent trace checkpoints and direct dispatch measurement](../fixed-trace-profile/README.md) |
| #281 campaign (direct) | [#288](https://github.com/pabloxrl/mtg-lab/pull/288), `c779b190ed28ff845741c4ce1149f71e55ec675b` | [38052644546](https://github.com/pabloxrl/mtg-lab/actions/runs/38052644546) | [Raw artifacts and reproduction](../four-mode-measurement/README.md), [completion/review](https://github.com/pabloxrl/mtg-lab/issues/281#issuecomment-6096505485) |

## Original acceptance and every assigned clause

The pinned [RFC](../../rfcs/0002-first-mvp.md) and lossless requirement ledger
remain authoritative. The following accounts for all five assigned blocks,
including shared clauses whose later ownership is deferred, not passed here.

| Clause | Scalar disposition and executable/evidence basis |
| --- | --- |
| B008: valid games/s/core and /host, decisions by type, completed logical actions | Raw #281 windows retain natural completions, denominator, decisions and fixed per-kind counters. Physical-core allocation is unavailable: report games/s on one logical CPU, never physical-core or whole-host efficiency. Completed logical-action arithmetic is explicitly corrected below; cancelled groups do not count. |
| B008: resident capacity versus throughput; latency p50/p95/p99; no fastest-engine claim | #217 retains four resident curves separately from throughput. #278/#281 publish bounded sampled native phase quantiles; they are bucket bounds, not exact decision percentiles or queue time. Learner throughput and queue scheduling are deferred. No cross-engine speed comparison. |
| B014: correctness-gated M2 baseline; allocations/boundaries; language choice | #217 and #281 retain full correctness-gate receipts and measured executable hashes. #280 directly profiles shared semantic application, legal/view, encoding and allocations. No Rust/Python performance assertion, GIL assumption, second engine, unsafe optimization or language rewrite. Python/batching architecture remains later scope. |
| B019: declared hardware and provisional scalar/counters/state hypotheses | Shared ARM64, four-CPU quota, 7 GiB and affinity are recorded. Scalar rates miss 100k decisions/s. Historical heuristic counters miss 5%; new observed comparisons are distributions, not a universal ≤5% guarantee. Four 10,000-core-state curves meet the first-cycle ≤64 KiB estimator, with allocator retention. No physical-eight-core/x86 qualification. |
| B019: scaling, Python batch, interactive hypotheses; actual games/s and misses | These later targets remain deferred under the scope amendment. #281 reports actual games/s and completed decisions/game, never the RFC's illustrative 200 games/s. Misses lead to published dominant costs and an explicit unqualified budget, not shorter games or weaker rules. |
| B020 track 1: fixed validated transition/application and legality | #280 combat/control/effects tapes retain independently authored checkpoints for priority/casting/triggers/combat/cleanup. Correctness and full input consumption gate a second measured execution. Application includes semantic validation/scratch execution; legal/view is a separate boundary. Twelve successful profiles and seven invalid inputs remain in #281's privileged archive. |
| B020 track 2: native complete rollout, reset/shuffle/policy | Both frozen native policies execute all eight normal-reset rows. Reset, policy, finalization and failures remain in elapsed time; policy time is separate. #281 records 7,204 natural completions and all 7,204 attempts across 100 windows. Seeds repeated between configurations are repeated executions, not distinct independent games. |
| B020 track 3: encoded collection | Same two-policy/eight-row workload measured with actual authorized Observation JSON serialization enabled/disabled, with encoded bytes and direct time. No tensor, binding or Python equivalence claim; those paths are deferred. |
| B020 track 4 and track 6: inference and CLI | Fixed-model/device/transfer/queue and command-to-view qualification remain later owners; no new RL or interactive work is authorized. Tiny effect samples do not qualify CLI latency. |
| B020 track 5: capacity and stress | #217 normal-play prefixes are replay-validated before copying states: typical, nine-token, fifteen-target and ten-effect-stack samples; 10,000 states and three reset/release cycles each. Whole-game allocation profiles retain unequal game lengths. Policy/history/replay/client buffers are excluded from marginal core-state capacity and not silently counted as supported concurrent clients. |
| B020 pins, resources, worker/batch sweep | Frozen workload/schema/deck/action/policy/seed and binary/source/toolchain/flags/OS/affinity pins survive. Single-worker native path is measured. Worker and batch-size sweeps, Python package pins and designated-host scaling are deferred; no fabricated zero or acceptance. |
| B020 warmup, windows, variance, denominators | Every configuration has ≥10s warmup and ≥5×30s windows, two declared full-replay extensions retained. Pooling leaves 11.47%/12.40% CV; uncertainty stays explicit. Original warmup-validator failure is archived; reviewed correction removes an invented warmup game-count minimum, not any time/window/row requirement. No native failure was hidden or rerun for selection. Wins/draws/truncations/errors/concessions/unfinished/pre-reset attempts and reset/finalization costs remain explicit. |
| B020 allocation/legal/encoding/effect/cache/synchronization and dominant costs | #217 software profiles rank SHA-256, JSON and allocation with tool discrepancies disclosed; #280/#281 directly measure boundary/allocation and witnessed modifier dispatch. Requested allocation traffic is not live RSS. Hardware/cache events and synchronization/scaling are unavailable/unmeasured, not zero. No allocation-free or leak-free claim. These limits preclude corresponding qualification, not the scoped available-container baseline. |
| B021 four explicit modes, local aggregation, boundary export | Driver counts use fixed fields/local saturating merges; native sampler has six fixed phase labels/eight buckets. Off ordinary simulation omits the sampler and optional clock. Enabled modes collect sampled timings; sampled_trace adds bounded numeric checkpoints; full_replay exports complete verified privileged history. JSONL/native and benchmark JSON export remain outside rules; no shared per-rule atomic or clock. No Prometheus/OpenTelemetry dependency. |
| B021 required outcomes, decisions, logical actions, work, resets | `metrics.rs` literal tests cover successful/rejected reset, natural completion versus concession, truncation/failure/incomplete, accepted decisions by kind, begun/committed/cancelled groups and rules work units. Counters survive run aggregation; quantum changes do not redefine work as resume calls. |
| B021 encoding, policy/inference, memory, worker/queues/batch | Native sampled phases separate policy/encoding/transition; benchmark direct totals separate them again. Linux wait4/proc provides peak memory in campaign receipts. Core-only timing/memory is honestly not_measured. Scalar worker utilization, ready queues, batch fill and inference have no implemented path and remain not_applicable/deferred. |
| B021 overflow/invalid/stale/dropped and every exceptional path | Existing tests retain nonmutation and literal rejection counts, clock/recording/capacity errors, saturation flags, interrupted/pre-reset/failed-work denominators and explicit trace drops. Failed or incomplete promised replay is not admitted as successful benchmark output. |
| B021 cardinality/privacy | Fixed counter fields and phase labels contain no IDs/seeds/card names/raw errors/hands/library/RNG. Public sampled checkpoints contain only accepted-decision/work counts. Private replay, profile inputs and two-seat archives remain privileged diagnostic artifacts, never policy features. Sentinel tests and player-input hash comparisons enforce the boundary. |
| B021 bounded trace and complete replay/backpressure | Exact interval/capacity/zero-capacity tests count drops; bounded records reject unrecordable decisions and replay-size overflow fails export. Canonical publication/write/signal failure remains explicit. Replay byte bound constrains the artifact, not peak serialization RSS (observed roughly 863 MiB); rules outcomes are separate from recording status. |
| B021 versioning, equivalence and measured overhead | Historical benchmark v1 still rejects trace/replay; additive scalar-four-modes-v1 fixes capacity/sampling/in-memory persistence. #281 measures 16 configurations and retains all 18 invocations/100 windows. Separately, 128 fixed policy/encoding/mode/row episodes preserve observation/history/final hashes; 64 capture/mode/row runs check capture reload and replay. Timed-window totals are not an equivalence oracle. |
| B021 independent trajectories and separate storage budget | Off does not discard requested canonical capture. Existing publication/failure and capture-content tests remain mandatory. Fixed profiles include in-memory capture comparisons; timed windows declare capture off and replay verification/serialization inside elapsed time. Full compression/queue/durable-storage qualification remains SYS-PERF-004's deferred #28/#39 scope, never included in counters-only acceptance. |

Stable catalog disposition: **SYS-METRIC-001/002 and SYS-PERF-001/002/003 satisfy
their applicable M2 scalar composition**, with the hardware, later-path and
provisional-budget limits above. This does not claim all stages of those shared
IDs passed. SYS-PERF-004 and full GR-070/071 worker/batch/corpus obligations remain
outside this audit. #24/#26 retain rules/reference scenario admission; no unit-test
count, generated game count or native replay is substituted for independent
reference agreement.

## Completed logical actions: distinguish cancellation

The historical benchmark JSON field `logical_actions_per_second` divides **begun**
groups (`counters.logical_actions`) by elapsed time. It must not be described as
B008's completed logical-action throughput. `committed_actions` and
`cancelled_actions` are already retained independently in every enabled-mode
window. This audit publishes the required completed rates from those untouched
raw counters; it changes no historical artifact/schema or test expectation.
Off mode has no optional logical counters and is unavailable, not zero.
The [machine-readable totals](committed-actions.json) retain exact denominators,
unrounded rates, source archive hash and all four unavailable off-mode entries.

| Policy / track | Mode | Begun | Committed | Cancelled | Committed/s |
| --- | --- | ---: | ---: | ---: | ---: |
| Heuristic / native | counters | 403766 | 403766 | 0 | 2682.529 |
| Heuristic / native | sampled_trace | 409146 | 409146 | 0 | 2715.292 |
| Heuristic / native | full_replay | 84619 | 84619 | 0 | 555.889 |
| Heuristic / encoded | counters | 399023 | 399023 | 0 | 2652.391 |
| Heuristic / encoded | sampled_trace | 399960 | 399960 | 0 | 2656.947 |
| Heuristic / encoded | full_replay | 85086 | 85086 | 0 | 556.473 |
| Legal-random / native | counters | 371431 | 330985 | 40446 | 2179.784 |
| Legal-random / native | sampled_trace | 371431 | 330985 | 40446 | 2174.657 |
| Legal-random / native | full_replay | 248951 | 222199 | 26752 | 460.724 |
| Legal-random / encoded | counters | 366024 | 326306 | 39718 | 2136.161 |
| Legal-random / encoded | sampled_trace | 363920 | 324443 | 39477 | 2125.327 |
| Legal-random / encoded | full_replay | 250877 | 223871 | 27006 | 460.260 |

Reproduce from the repository root, verifying the archive before reading it:

```sh
python3 - <<'PYCODE'
import json, runpy, tempfile
from pathlib import Path
source = Path('doc/evidence/four-mode-measurement')
unpack = runpy.run_path(str(source / 'archive.py'))['unpack_verified']
groups = {}
with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    unpack(source / 'throughput.tar.gz', source / 'throughput-index.json', root)
    for run in json.loads((root / 'campaign.json').read_text())['runs']:
        report = json.loads((root / run['name'] / 'report.json').read_text())
        config = report['pins']['config']
        if config['instrumentation'] == 'off':
            continue
        key = (config['policies'][0], config['encoding'], config['instrumentation'])
        totals = groups.setdefault(key, [0, 0, 0, 0])
        for window in report['windows']:
            totals[0] += window['elapsed_ns']
            for i, name in enumerate(('logical_actions', 'committed_actions', 'cancelled_actions'), 1):
                totals[i] += window['counters'][name]
for key, (ns, begun, committed, cancelled) in sorted(groups.items()):
    print(key, begun, committed, cancelled, round(committed * 1e9 / ns, 3))
PYCODE
```

## Integration checks and evidence boundaries

Normal discovery retains the following concrete executable checks; their literal
inputs and child red receipts supply independent expectations:

- [Core metrics tests](../../../crates/mtg-core/tests/metrics.rs): literal counts,
  natural empty-draw numerator, invalid/stale nonmutation, privacy, quantum/capture
  equality, rejected resets, clock/record failure and saturating merges.
- [Trace tests](../../../crates/mtg-core/tests/trace_modes.rs): decisions 2/4 with
  capacity one yield one drop, zero-capacity/reset behavior, four-mode capture
  equality and promised-replay overflow.
- [Latency arithmetic](../../../crates/mtg-cli/src/latency_tests.rs) and
  [native tests](../../../crates/mtg-cli/src/native.rs): exact bucket edges/ranks,
  sample/reset/merge, policy 3+7 ns versus transition 30 ns and encoding 100 ns,
  off zero reads, backwards-clock errors and real client summaries.
- [Four-mode tests](../../../crates/mtg-cli/src/benchmark/modes_tests.rs) and
  [measurement tests](../../../crates/mtg-cli/src/benchmark/measurement_tests.rs):
  real mode/capture/row executions, strict versions, failures/signals/persistence,
  literal failed-work denominator 17+29=46 and fixed-episode semantic hashes.
- Normal Python discovery: `test_m2_repair_timing.py`,
  `test_m2_repair_modes.py`, `test_m2_repair_profile.py`,
  `test_m2_repair_measurement.py` and `test_scalar_artifact.py` validate actual
  native exports and tampered artifacts, not parser-only mock substitutes.

Source inspection confirms `native::run` creates a sampler for every enabled
mode even with the benchmark's supplied `PolicyTiming`. Histogram updates thus
occur inside the measured execution. That benchmark requests every-boundary
timing in every mode, including off, and reuses those clock reads for samples;
the observed overhead is conditional on this common profiling contract, not
the overhead of ordinary clock-free off simulation. Its output does not export
histograms. The #281 fixed-episode sampler uses a separate pinned test binary
and affinity; those bucket summaries cannot be attributed to benchmark windows.

The original #217 executable `e64842e` and #281 executable `2ec601c` remain their
measurement pins. No new performance campaign or reference engine run is claimed
by this documentation audit. Unchanged rules/reference receipts are linked by
the child reports; reference execution cannot be an oracle for clock arithmetic.
No source fingerprint or independently authored trace is rewritten for this report.

Fresh archive integrity and full summary recomputation passed on the audit base.
The [archive-check receipt](archive-check.json) also records a direct sum of all
100 raw windows: 7,204 completions/attempts, 7,181,389 decisions and
3,091,847,303,512 ns, with every warmup ≥10s and every window ≥30s. Per-window
row completions and seat wins reconcile with the natural-completion numerator.
The fresh shared-lock full `./scripts/torture.sh` (including `./scripts/verify.sh`)
passed: **303 Python tests, 744 debug + 744 release Rust tests (1,488 executions),
zero failed/ignored Rust tests**, formatting and strict Clippy. The
[validation receipt](validation.json) pins the unchanged freshly fetched base and
[complete log](torture.log.gz). No new test was added or existing assertion changed;
this delivery audits the normal discovered suite. The documented reproduction
command, all 12 committed-action table rows and local documentation links passed.
Prescribed separate review, protected PR and exact-main CI are recorded in the
[single issue workpad](https://github.com/pabloxrl/mtg-lab/issues/25#issuecomment-6090375546).

README impact: link this aggregate evidence and keep the stage table/M2 verdict
unchanged. No new setup or usable command is introduced. Benchmark documentation
clarifies begun versus committed rates and common profiling overhead; existing
runtime behavior remains unchanged. Review must check those distinctions.
