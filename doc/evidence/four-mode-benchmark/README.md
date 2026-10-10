# Four-mode scalar execution contract

GH-279, partial RFC 0002 B008/B019/B020/B021. The
[delivery workpad](https://github.com/pabloxrl/mtg-lab/issues/279#issuecomment-6093346745)
records exact-candidate checks, independent review, protected merge and main CI.
This contract does not complete M2 or qualify performance.

Independent requirements: RFC B020 requires 10s warmup and at least five 30s
windows, normal reset and policy costs, raw distributions, failed work retained
in denominators, and rules-terminal-only completed throughput. B021 requires
four modes, bounded diagnostics with explicit drops, complete replay or explicit
failure, privacy and independently configurable canonical capture. The delivered
#215 contract samples accepted decisions at multiples of `every`, retaining the
first `capacity` checkpoints; dropped = floor(decisions/every) minus retained.
#216 supplies the hand-counted window ledger and real native client.

The additive `scalar-four-modes-v1` schema 2 retains the existing eight rows,
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

Normal-discovery acceptance, independent of rates or engine-generated goldens:

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
no existing expectation is changed. That review evaluates the oracle, not delivery completion.

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
review are mandatory in the linked delivery receipt.

The separate candidate review found capture publication overriding signal exit
codes and owner outcomes. [Compiled signal red](signal-red.txt) demonstrates
`3 != 130` before reset with capture enabled. The normal-discovery regression
covers SIGINT and SIGTERM before reset, after reset and before publication,
including absent committed manifests. The correction preserves 130/143 and the
owner's actual completed/unfinished/not-started classification while retaining
publication-failure evidence. Ordinary writer failures still fail with code 3.

Self-review also found that an intermediate generic JSON value accepted duplicate
fields that typed parsing had historically rejected. [Compiled parsing red](parse-red.txt)
reproduces this regression; direct typed deserialization restores strictness for
both versions. Both assertion failures were recorded before the corrections.
The queued job applied the corrections before the regression commit completed;
`28c9930` therefore contains the fixes as well as the new tests, not a red-only tree.

Diagnosis/revised approach: the complete real-game matrix passed, but its success
paths did not cover the interaction between signals and failed publication or
ambiguous parser input. The bounded repair adds those cross-path regressions;
no rule, capacity guarantee, existing expectation or delivery gate is weakened.


Focused Python acceptance passed four tests against 64 real native executions:
eight rows × four modes × capture off/on. The exporter witnessed 37,624 acting
observations (9,406 per mode), verified all 16 full-mode replays, and reloaded all
32 requested canonical captures. Sampled mode retained 16 records and explicitly
dropped 4,682 at interval two/capacity one. These are observed execution receipts,
not generated rules expectations. An injected seven-nanosecond phase clock checks
accounting; none of these finite-test durations is a performance measurement.
Missing rows/modes/observations, altered history, invalid denominators, missing
replays and incorrect drops are rejected by the focused Python negative controls.

The unchanged rules/reference basis remains the delivered
[#208 policy/reference receipt](https://github.com/pabloxrl/mtg-lab/issues/208#issuecomment-6074112807)
and [#215 trace/replay receipt](https://github.com/pabloxrl/mtg-lab/issues/215#issuecomment-6079735470).
No new reference-adapter execution or rules agreement is claimed for clock,
serialization retention or artifact arithmetic.
