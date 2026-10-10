# Frozen GH-281 measurement plan

Frozen before collection. Requirements: RFC 0002 B008/B014/B019/B020/B021 and
reviewed GH-268 child contracts; measurement is not M2 completion or a speed
qualification. No production engine/collector/profile feature is introduced.

Run the delivered `scalar-four-modes-v1` benchmark and `scalar_modes.collect_report`
on a correctness-gated release binary, preserving its exact source/binary,
compiler/flags and lockfile pins. Run sequentially under the shared heavy lock
on the lowest available logical CPU; retain before/after affinity, quota and
memory limits. Physical-core reservation, hardware counters and designated-host
qualification are unavailable, not zero. Separate test-target diagnostic binary
hashes from the production release binary. Never pool them as throughput data.

Matrix/order: encoding false then true; heuristic then legal-random; off,
counters, sampled_trace, full_replay. Each configuration uses seeds 42/42 and
cycles RG0, RG1, GR0, GR1, RR0, RR1, GG0, GG1. Each has 10 seconds warmup and five
30-second windows. The delivered runner does not select a single matchup or
report per-row elapsed time: this is a mixed eight-row workload, not per-row
throughput. Retain per-row attempt/completion counts and reject missing rows.
Horizon stays 20,000 decisions / 100,000 work calls, quantum 64.

Variance extension frozen rule: population coefficient of variation of initial
five completed-games/second windows greater than 0.10 triggers exactly one
additional run with 10s warmup and ten 30s windows of the same configuration.
Keep the original and all extension windows, including slower results. An
extension restarting seeds is a repeated workload, not additional independent
games. No retry on failure; retain its raw output/process receipt and mark the
campaign unsuccessful. Continue the other planned configurations for diagnosis.
A high CV after extension stays visible; no iterative stopping or best-window
selection. Timeout per invocation is 3700 seconds, with failed attempt retained.

Trace sampling every 64 decisions, capacity 256. Replay capacity 20,000 records /
67,108,864 bytes, persistence `in_memory`. Complete replay serialization and
verification remain inside elapsed time; this is not durable replay throughput.
Canonical capture is off for comparable throughput. Validate its independent
on/off semantics through the delivered finite four-mode native/capture matrix
and fixed profile runner, whose raw receipts remain separate. Failed, truncated,
unfinished, pre-reset and concession work stays in the denominator and out of
the natural completed-game numerator. Reset/finalization, replay, encoding and
publication boundaries remain as reported; do not subtract them from elapsed.

Publish every raw window, aggregate count/elapsed rates, nearest-rank p50/p95,
min/max/mean, and all cross-product percent overhead samples against off for the
same policy and encoding track, using 100*(1-mode_decision_rate/off_decision_rate).
Cross-product samples are correlated descriptive comparisons, not matched games,
confidence intervals or causal estimates. Off-versus-off spread is a variability
reference, not instrument cost. Separately report total process CPU/wall/RSS.
Warmup and outer process overhead are separate from the benchmark window rates.

Sampled native latency comes from separate fixed natural episodes of both
policies, eight rows, four modes and both encoding settings using the delivered
native sampler. The benchmark explicitly marks sampled timing `not_measured`;
never invent window histograms. Preserve each fixed episode's bucket/count/sum
and p50/p95/p99 bounds, full sampling/skipped/error denominator. Off remains
unavailable. Compare actual observation, history and final hashes on the same
fixed episode across modes/encoding; equal window totals are no equality oracle.
Injected-clock accounting tests are never performance measurements.

Execute unchanged validated control/combat/effects fixtures through the delivered
profile adapter; retain inputs/rejections, trace hashes, boundary time/allocation
traffic and isolated witnessed modifier dispatch samples. No effect cost inferred
from application subtraction; no encoding cost inferred from a field-size guess.
Profile allocation traffic is not RSS. Reference rules/pins are unchanged; reuse
GH-208/GH-215 plus delivered reset, combat and spell reference receipts. No
manufactured XMage comparison for clock arithmetic.

Historical GH-217 resident sweeps/stress and allocation profiles remain immutable.
This task adds no production buffers or structures; do not extend their core-state
capacity claims to replay/capture buffers. Native process peak RSS is measured by
the existing collector; future concurrent buffer capacity is unmeasured.

Before collecting: focused literal positive/negative tests, real adapter checks,
new native/injected-clock tests and full normal torture on candidate source.
After main integration: repeat full torture and separate exact-candidate review.
No removal or weakening of existing tests. README and report require independent
accuracy review. Missing runner contracts require bounded prerequisite reporting.
