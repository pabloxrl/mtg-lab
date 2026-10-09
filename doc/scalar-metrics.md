# Scalar counters

The owned Driver supports `metrics::Mode::Off` (default), `Counters`, `SampledTrace` and
`FullReplay` through
`Driver::instrumented` and `Driver::bounded_instrumented`. The bounded constructor
retains its existing injected deadline clock and budget contract. Counters do not
read a performance clock or change budget checks. `metrics()` returns a cumulative
snapshot across resets, or `None` in off mode. No mutable game access is introduced.

Native and semantic-script simulation accept `native.instrumentation: "counters"`.
The default is `"off"`; unknown modes are rejected. The final JSONL summary includes
`metrics` only when requested. Its `schema_version: 1` covers a fixed set of numeric
fields and availability tags. The run's existing provenance and episode rows are
separate artifacts, not privacy-safe metric labels. Do not publish those artifacts
as player telemetry merely because the aggregate metrics are safe to export.

Counters describe actual boundaries:

- `resets` and `started`: successful normal-reset commits, including a subsequently
  failed reset-time capture or capacity check. Rejected resets have their own count.
- `completed`: completed owner outcomes, including concessions. `rules_completed`
  excludes concessions and is the natural-ending numerator for later benchmarks.
  Truncated, failed and incomplete outcomes are separate and finalized once.
- `decisions`: accepted policy submissions, including continuations, counted once
  for direct and semantic-record input. The distribution groups the fixed decision
  kinds; unknown future kinds use a fixed `other` bucket, never a dynamic label.
- `logical_actions`: begun action groups, matching the existing trajectory grouping.
  `committed_actions` counts completed actions for throughput; `cancelled_actions`
  counts cancellations. Unfinished groups do not count as committed. Concessions
  are separate out-of-band commands, not policy decisions or logical actions.
- `rules_work_units`: executed work-queue units, including reset, not resume calls
  or time. Equivalent work quanta produce equal counts. The tally is optional and
  excluded from snapshots and restored state.
- `boundary_errors`: returned errors at public owner boundaries. Invalid/stale
  action requests, rejected resets, capacity, clock and recording failures are
  separately identified. Rejected input does not add an accepted decision.

All additions saturate at `u64::MAX` and set `overflowed`. Treat an overflowed
report as inexact; saturation never wraps or changes game rules. Aggregation is
local (`Counters::merge`), with no shared atomics, dynamic labels or unbounded
buffers. Counts contain no card names, seeds, game identifiers, observations,
private choices, arbitrary error text or timings inferred from those values.

Native summaries aggregate at the run boundary. A caller/script failure marks the
attempt failed in these counters even if its underlying owner completed. Existing
`owner_status` remains diagnostic evidence of that distinction.

Capture is independently requested. Off mode does not discard semantic history
or selected trajectory capture; those are existing owned-execution contracts.
No mode materializes additional trajectory frames for instrumentation.

Worker/queue/batch/inference and diagnostic-buffer fields in the counters report
are explicitly `not_applicable` to that path; sampled diagnostics have a separate
versioned per-episode report. Encoding/policy timing, sampled
histograms and memory high-water are `not_measured`, not measured zeros. Existing
benchmark policy timing remains separate. Benchmark workloads, overhead qualification and memory measurements remain the
registered #216–#217 deliveries; no five-percent overhead or full RFC B021 claim is made here.

The literal traces and injected-clock checks are in
[the normal regression suite](../crates/mtg-core/tests/metrics.rs), with
[native output checks](../crates/mtg-cli/tests/native_simulate.rs).
[Delivery evidence](evidence/scalar-metrics/README.md) records validation status.

## Bounded trace modes

`sampled_trace` adds decision checkpoints to the same scalar counters. Selection
is deterministic: every Nth accepted decision, numbered from one at each reset.
It never consumes game or policy RNG. A checkpoint contains only the decision
count and cumulative rules-work count. These are public numeric diagnostics;
there are no hands, library order, action payloads, seeds or state hashes. It is
not a player observation or a replacement for a replay.

`TraceConfig` declares a nonzero interval and a capacity from zero through 65,536
records (defaults: 64 and 256). A full buffer retains its earlier checkpoints and
increments `dropped` for each selected checkpoint it cannot retain. A zero-capacity
buffer counts every selected checkpoint as dropped. Counts saturate with a visible
`overflowed` flag. Configuration is immutable after the first reset; the buffer and
its drop counter restart at each successful reset. Rejected input adds no sample.
The trace has its own schema version and is separate from aggregate counter labels.

`full_replay` enables the same counters and explicit privileged export through
`EpisodeResult::privileged_replay(max_bytes)`. Export uses the existing played
replay format: complete semantic history, version pins, seeds, initial state and
per-action checkpoints, verified against the owned final state. It is independent
of trajectory capture. Incomplete, truncated or failed episodes cannot yield a
complete replay; size overflow returns an error. This byte limit bounds the returned artifact,
not peak memory during serialization. Export is an explicit recording operation;
a recording error does not rewrite the immutable underlying rules result. Existing Driver record bounds
fail the episode before accepting an unrecordable action. No replay data is
silently replaced by sampled diagnostics.

Native/script CLI modes use the same owner. `native.trace` optionally configures
sampled mode. Episode rows expose only numeric `diagnostics` or a `replay_status`;
private replay bytes never enter stdout. Without requested durable capture,
complete replay generation is an in-memory operation. Request the existing
canonical capture configuration to persist replay artifacts through its restricted
replay directory and atomic manifest publication. Storage/queue failures remain
explicit publication failures; they do not change the underlying rules outcome.
Trajectory capture with instrumentation off retains the same persistence contract.

These modes add no timing histograms, worker implementation or performance claim.
Overhead qualification remains the separately registered benchmark work. See
[acceptance evidence](evidence/trace-modes/README.md) for current validation status.
