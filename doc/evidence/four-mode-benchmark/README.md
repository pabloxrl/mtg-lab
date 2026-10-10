# Four-mode scalar contract — acceptance design

GH-279, partial RFC 0002 B008/B019/B020/B021. Implementation pending.

Independent requirements: RFC B020 requires 10s warmup and at least five 30s
windows, normal reset and policy costs, raw distributions, failed work retained
in denominators, and rules-terminal-only completed throughput. B021 requires
four modes, bounded diagnostics with explicit drops, complete replay or explicit
failure, privacy and independently configurable canonical capture. The delivered
#215 contract samples accepted decisions at multiples of `every`, retaining the
first `capacity` checkpoints; dropped = floor(decisions/every) minus retained.
#216 supplies the hand-counted window ledger and real native client.

The additive `scalar-four-modes-v1` schema 2 will retain the existing eight rows,
seeds, policies and 20,000-decision / 100,000-work-call horizon. Old schema 1
workloads keep rejecting sampled_trace/full_replay. Default trace selection is
every 64 accepted decisions, capacity 256; explicit settings require sampled
mode. Replay record capacity defaults to 20,000, byte capacity to 67,108,864.
Full replay bytes are serialized, verified and retained until the episode attempt
finishes inside the outer timer. This is in-memory persistence, not durable
storage. Optional canonical capture uses the existing bounded durable publisher
independently of instrumentation; unsupported persistence strings reject.

A started episode finishes before checking the window deadline again: overshoot,
reset, capture publication, replay verification, summary parsing and destruction
remain measured. Signals/failures remain visible, including pre-reset attempts.
No incomplete replay is counted as a verified complete replay. Sampled latency
histograms remain `not_measured` in this contract pending their separate child.

Expected tests, independent of measured rates or engine-generated goldens:

- All four modes accept the new version and execute all eight normal-reset rows.
- Same seed/row and finite choices produce equal authorized observations,
  history and outcome across modes and capture settings; nonempty observations
  prove real execution. Full replay verification reconstructs the final state.
- Literal sampling formula above checks real decision counts, retained indices,
  and drops; public artifacts contain no replay bytes or private state.
- Zero/oversized capacities, incompatible trace settings, unknown persistence,
  and old-version trace modes reject. Tiny valid bounds cause visible runtime
  failure, never faster successful throughput; writer faults also fail.
- Injected clocks change only measurement, never the game engine. Literal
  pre-reset/reset/failure/finalization/unfinished/overshoot costs remain counted.
- Python normal discovery consumes real Rust-exported artifacts, validates rows,
  mode/version and arithmetic, and rejects missing/tampered denominators and
  replay/trace completeness. Synthetic arithmetic probes are supplementary.

No rules/reference semantic boundary changes are planned; reuse #208/#215
reference receipts. No campaign, histogram implementation, optimization, CLI
command family, RL or M2 completion claim.
