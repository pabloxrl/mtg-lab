# Four-mode scalar contract — acceptance design

GH-279, partial RFC 0002 B008/B019/B020/B021. Implementation candidate; protected delivery and full verification pending.

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
- Zero/oversized replay capacities, oversized diagnostic capacity, incompatible
  trace settings, unknown persistence, and old-version trace modes reject.
  Zero/tiny diagnostic buffers preserve gameplay with exact drops. Tiny replay
  bounds cause visible failure, never faster successful throughput; writer
  faults also fail. Successful capture must publish and reload through the
  existing canonical validator, including instrumentation off.
- New-version tests retain duration minima, unsupported-policy rejection, all
  raw windows, warmup separation, zero-completion failure and signal exit codes.
  Replay verification retains all existing pins/checkpoints/choice checks.
  Report replay serialization/verification and canonical publication costs
  separately, as subsets of the outer denominator.
- Injected clocks change only measurement, never the game engine. Literal
  pre-reset/reset/failure/finalization/unfinished/overshoot costs remain counted.
- Python normal discovery consumes real Rust-exported artifacts, validates rows,
  mode/version and arithmetic, and rejects missing/tampered denominators and
  replay/trace completeness. Synthetic arithmetic probes are supplementary.

No rules/reference semantic boundary changes are planned; reuse #208/#215
reference receipts. No campaign, histogram implementation, optimization, CLI
command family, RL or M2 completion claim.

Pre-implementation independent review: diagnostic capacity zero is valid under
#215/B021 and must report drops, not fail. This correction is incorporated above;
no existing expectation is changed. Complete implementation/review remains required.

The [independent expectation review](oracle-review.txt) is preserved verbatim.
Its diagnostic-capacity correction and required capture/qualification checks are
incorporated in the test plan above. The initial whole-candidate review correctly
reported the test-only commit as undeliverable; it is not an integration approval.


Behavioral red: [the independent schema-2 window-ledger rejection](red.txt) is an
assertion failure, not a compile/import error. It was recorded before production
changes. The first Rust build later found a test-only reference-type error; that
compile failure is not red evidence. The corrected first native run passed seven
new tests, including 64 actual normal-reset mode/capture/row combinations. A
subsequent finite-script test also retains the existing independent Growth/Bear
Cub five-damage ledger and concession exclusion. Full normal discovery and final
review remain required before delivery.
