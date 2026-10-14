# XMage 100-attempt campaign and complete native L0 measurement

L0-005 measures actual ComputerPlayer7 generation with the unchanged frozen decks,
raw capture bridge and original recording-equivalence checks. Source completion,
conversion, diagnostic native agreement, provisional qualification and final
three-engine admission are separate results. This campaign does not itself admit
new games or change timing policy. Protected delivery receipts are recorded in the [issue workpad](https://github.com/pabloxrl/mtg-lab/issues/323#issuecomment-6175111180).

The declaration fixes seeds 323000–323099, cycling RG/0, RG/1, GR/0, GR/1, RR/0,
RR/1, GG/0 and GG/1. The first four rows receive 13 attempts and the others 12.
Sixteen single-worker attempts precede 84 attempts with at most two JVMs. Each
uses `-Xmx2g`; the two-worker probe starts only after source completion and a
single-worker peak below 2,500,000 KiB. This is an execution safety condition,
not a latency gate. Every attempt is retained; none is rerolled or replaced.
The two phases use different balanced seed samples, so their rates are not a
matched-workload causal scaling estimate. Timing-based upstream searches mean
seeds alone cannot reproduce a game; retained choices/chance define the replay.

## Measurement boundaries

Reference source/dependency preparation and compilation are separate from the
campaign. A small timing shell calls the unchanged `RawAiCaptureTest.capture()`;
it does not implement an AI or modify rules. `javac` prepares this shell and the
existing bridge once. The campaign pins every class/resource/JAR on its actual
classpath. Workers read those shared build inputs; they never run Maven or write
the shared source tree. Each attempt has its own output directory and JVM.

The Java management clock measures JVM startup through timing-shell entry.
`capture_and_equivalence_seconds` includes game setup, real AI play, full raw
recording and the original recording-on/off equivalence control. It is not pure
AI-search time or steady-state same-JVM generation. The outer Java wall clock
also includes process launch and exit. Raw trace validation, lossless archival
and normalization have separate elapsed fields. End-to-end phase wall time
includes all attempts, Python orchestration, validation, archival and conversion;
failures and duplicates stay in the projection denominator.

The ordinary native measurement uses the existing release L0 command against
**all ten provisional pilot entries**, zero final admissions. Campaign candidates
are excluded until separately qualified. Every native cold invocation includes
process start, binary/manifest/artifact hashes, loading, validation, full replay
and exact semantic comparison. Its second complete pass runs in the same process;
no games are discarded as warmup. Compilation and packaging stay outside this
clock. Loaded replay/comparison is available as a component, not a substitute for
cold full-path elapsed. “Cold” means a new process, not flushed operating-system
file caches. Full assertions, source exceptions and quotas remain unchanged.


## Measured result (2026-10-14)

All 100 source attempts completed; source failed/truncated/unsupported classes
were zero. Conversion produced 95 known-distinct canonical games, zero duplicates
within the campaign or against the ten-game pilot, and five unknown identities.
Diagnostic native replay ran all 95 converted candidates twice: 93 agreed, two
retained the differences below. The 93 agreeing games cover 51,083 actions and
36,474 source checkpoints per repeat; all source-specific negative tapes remain
checked. These source-observation checkpoints differ from the every-action L0
checkpoints below. The five unconvertible attempts remain in the
100-attempt denominator. **Zero new provisional or final admissions** result.
The separate real unsupported-callback and record-truncation controls both exited
nonzero with the expected reason; neither counts as a campaign attempt or game.

| Generation phase | Attempts / known distinct | Wall seconds | Per-attempt p50 / p95 seconds | 1,000 known-distinct projection |
| --- | --- | --- | --- | --- |
| One worker baseline | 16 / 16 | 995.597 | 59.749 / 90.988 | 17.285 hours |
| Two workers, subsequent balanced seeds | 84 / 79 | 3,139.925 | 73.899 / 100.195 | 11.041 hours |
| Declared combined campaign | 100 / 95 | 4,135.522 (68.925 minutes) | 72.557 / 98.008 | 12.092 hours |

Projection is observed phase wall cost × 1,000 / known-distinct convertible
completions, including failed conversion costs. At the 93/100 diagnostic-agreement
yield, the same generation-only projection is 12.352 hours. Neither estimate
includes patched-XMage/Forge qualification, retries, storage publication or later
corpus diversity effects. Cost for 1,000 **admitted** games remains unknown, as does
cost for the required mixed-source corpus. This is not a generation/replay speedup
ratio. Different seeds and shared-container load prevent a causal scaling claim.

The source campaign recorded 478,779 raw callback/event records and 52,335 normalized
semantic actions across 95 conversions. Raw records are not replay checkpoints.
It produced 732,699,861 bytes of attempt artifacts including private source logs
at each measurement boundary. Maximum Java-child RSS was 978,812 KiB; this does
not measure aggregate Python+Java pipeline RSS or simultaneous container high water.
Native resources and the full L0 checkpoint counts are reported separately below.

| Source component, across all 100 attempts | p50 seconds | p95 seconds |
| --- | --- | --- |
| JVM startup to timing-shell entry | 0.034 | 0.055 |
| Capture, setup and recording-equivalence checks | 26.761 | 38.084 |
| Outer Java process, including exit | 26.888 | 38.216 |
| Raw validation | 11.195 | 14.883 |
| Lossless archival | 11.223 | 15.137 |
| Conversion (including failed conversions) | 23.710 | 32.442 |

The checksum-bound attempt receipts additionally retain outer Java-process,
trace-validation and archival timings; `python3 scripts/check_xmage_campaign.py`
recomputes their distributions. Component sums across concurrent attempts are
work-duration sums, not elapsed campaign wall time.

Reference build/dependency resolution/smoke took **221.556 seconds** (peak child
RSS 4,587,548 KiB). Capture-shell preparation took **9.351 seconds**, including
classpath validation/pinning and javac; it is not a pure compilation microbenchmark.
Native release compilation took **25.407 seconds** (649,264 KiB peak child RSS).
Immutable ten-game L0 packaging took **15.340 seconds**. These costs are excluded
from both native cold replay and the measured generation phases.

## Full native L0 timing

The corpus pin is `bd67e4e952d8660d83377d36da931fa933b30edf32d0ed3635c9e90bc38a032d`;
the release runner pin is `e3b229104209d56eea438f4261c486c4d574aef3e1325b73dccadc0a56ba66aa`.
Every pass covers **10 distinct provisional games, 4,886 actions, 4,886 checkpoints
and 111,210,783 artifact bytes**. Five serial invocations precede two concurrent
invocations. Each has one cold and one warm full pass: 140 game executions, still
only ten distinct games. All pass, with zero failed or unattempted entries.

| Serial measurement | p50 seconds | p95 seconds | Range seconds |
| --- | --- | --- | --- |
| Cold process-start/load/checksum/validation/replay/comparison, full corpus | 3.1981 | 3.3186 | 3.1750–3.3186 |
| Warm second full load/checksum/validation/replay/comparison | 3.1819 | 3.1886 | 3.1147–3.1886 |
| Cold per-game loaded replay/comparison (50 game samples) | 0.2600 | 0.4828 | 0.1197–0.5196 |
| Warm per-game loaded replay/comparison (50 game samples) | 0.2614 | 0.4773 | 0.1262–0.4788 |

Percentiles use nearest rank (five-run p95 is the maximum). Median warm full-path
throughput is 3.143 games/s; loaded replay/comparison alone is a different boundary.
Cold loaded replay/comparison totals have median 2.5697 seconds; roughly 0.63 seconds
of the full cold path is loading, validation, integrity checks and launch overhead.
The current runner does not separately instrument each of those overhead components.
Serial peak child RSS is **197,808 KiB**. All five cold-plus-warm invocations finish
in 6.358–6.531 seconds each, excluding the outer Python measurement harness startup.

The subsequent two-runner probe finishes 40 repeated game executions in **6.5899
seconds** including both launcher lifetimes (6.070 executions/s). Individual cold
full paths are 3.2139 and 3.2396 seconds; this is concurrency across complete corpus
invocations, not internal parallel replay of one corpus. Raw receipts retain each
process's high water (197,804 KiB each); summing process peaks would not establish simultaneous RSS.

The 4.52% serial cold max/min spread is observed variability, not a confidence
interval or a host guarantee. With ten games, linear scaling of median full cold
elapsed suggests **319.8 seconds for 1,000 games**. That is an unvalidated sizing
estimate, not measured full-sized acceptance: about 139.8 seconds above a proposed
180-second interpretation of “low minutes,” and 5.33× a 60-second goal. Actual
1,000-game memory/storage, mix, caching and variance still require measurement.

## Budget proposal for operations review only

For this exact ten-game corpus, release build, instrumentation and hardware class,
propose a 4.0-second cold full-path investigation budget (25% over the observed
3.198-second median). Propose investigating a sustained >15% median increase over
a reviewed matched baseline, requiring two independent five-run batches; 15%
exceeds the observed 4.52% span but needs more samples before enforcement. Preserve
raw timings and failure counts and requalify changed corpus/runner/hardware versions;
never automatically bless a slower baseline. These are proposed investigation
thresholds for #332, **no installed timing gate or correctness waiver**.

Keep an aspirational 180-second full 1,000-game budget for policy review; the
small-corpus extrapolation misses it. Optimize or measure the preserved full path,
then test the actual full-sized corpus. Do not reduce quota, omit checks or use
loaded-only throughput to claim the target. Native timing and source generation
budgets are separate: provision approximately 12–18 hours for 1,000 convertible
XMage candidates under these observed one/two-worker conditions, plus separately
measured qualification cost. No host qualification or dollar-cost claim follows.

## Hardware, pins and retained evidence

Managed Linux aarch64 container, Linux 6.8.0-64-generic, four-CPU cgroup quota
(`cpu.max=400000 100000`), 7 GiB memory limit (`7516192768` bytes). Raw CPU and
memory descriptions are in `declaration.json`; limits, not host total RAM/CPU
counts, govern this run. The container is shared with the controller/other issue
workers; the common heavy-work lock serializes heavy commands. Java/javac are
Temurin 21.0.9+10, Maven 3.9.11, Rust 1.98.1 (`48a229cea`), Python 3.12.3.
JVM heap limit is 2 GiB per attempt, AI skill 1, search depth 4, nodes 5,000,
think allowance 3 seconds. Exact upstream commits, dependency hashes, decks, rules,
bridge, source and classpath hashes are in the receipts and `prepared.json.xz`.
The initial native baseline uses `fb493a1a112840ed8dd6df9be23d290ec299d096` engine
sources; subsequent integration validation is recorded in the delivery workpad.

The checksum-indexed evidence payload is **48,191,721 bytes** across 309 files
(excluding the index and this report).

`index.json` hashes the immutable source archives, diagnostic native executions,
L0 receipts, controls and measured source snapshots. `summary.json` is original
source accounting; `native/summary.json` preserves the deliberately nonzero partial
comparison outcome. Normal-discovery tests verify inventory and corruption
rejection, reproduce all three distinct conversion-failure boundaries, and reject
the retained native stack-target mismatch. They do not replace actual engine runs.

## Reproduction

Use the managed pinned Rust/Java/Maven/Python tools. Run reference commands,
Cargo and replay through Symphony's existing shared heavy-work lock. The issue's
reference cache must be writable and isolated from other issues.

```sh
heavy() { python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- "$@"; }
heavy python3 scripts/xmage.py prepare --cache "$MTG_REFERENCE_CACHE"
heavy python3 scripts/xmage.py build --cache "$MTG_REFERENCE_CACHE"
heavy python3 scripts/xmage_campaign.py prepare --cache "$MTG_REFERENCE_CACHE" \
  --output .agent-artifacts/campaign-prepared
heavy python3 scripts/xmage_campaign.py run \
  --prepared .agent-artifacts/campaign-prepared/prepared.json \
  --output .agent-artifacts/campaign-run
heavy python3 scripts/replay_xmage_campaign.py --source .agent-artifacts/campaign-run \
  --output .agent-artifacts/campaign-native
```

The diagnostic native command executes every successfully converted candidate
twice, preserves original/source-specific invalid-tape controls and checks source
observations independently. It records every outcome before exiting nonzero if
any declared attempt remains unconvertible or fails comparison; a partial campaign
cannot become a fully passing verification. It is not the fast L0 benchmark and does not claim
patched-XMage or all-three qualification. Failed traces retain the first difference.
For complete production L0, use the build/package/run commands in the
[native L0 report](../native-l0/README.md).

Source logs remain private under the [provenance policy](../../provenance-policy.md).
Immutable attempt receipts retain their hashes. Deployable attempt archives retain
raw compressed semantic tapes, configs, timing receipts, canonical content and
conversion output; no upstream installation or source log is redistributed.
The existing [XMage notice](../../../references/xmage/UPSTREAM-LICENSE.txt), source
pins and reviewed pilot exceptions remain applicable. These are privileged
verification artifacts, separate from player-visible inputs.

## Retained unresolved captures

These observed source endings are **not** admitted games. Normalization stops
rather than discarding an unexplained effect or manufacturing a binding. The
measurements retain the full attempts and costs. No upstream or native rules
fault is adjudicated here; repairing a consumer or adjudicating a source result
requires independent rules evidence and review.

| Attempt / seed | Row | First normalization difference |
| --- | --- | --- |
| 037 / 323037 | RR/1 | `/source/4352/effect lifetime` |
| 039 / 323039 | GG/1 | `/source/4775/departed effect target` |
| 068 / 323068 | RR/0 | `/source/5168/committed ability` |
| 082 / 323082 | GR/0 | `/source/2769/effect lifetime` |
| 087 / 323087 | GG/1 | `/source/6098/departed effect target` |

Without a complete canonical transcript, these attempts have **unknown** canonical
duplicate status. They are excluded from the known-distinct numerator; the report
must not call them distinct simply because their seeds or source UUIDs differ.
Normal-discovery regression tests retain these fail-closed reproductions. They test
rejection of unresolved input, not an assertion that either engine is rules-correct.


Two converted candidates also retain native/source stack-target mismatches:
seed 323006 (GG/0), source cursor 4695 / action 542, and seed 323079 (GG/1),
source cursor 1301 / action 152. Both comparisons require the recorded
`0/thornweald-archer/0` incarnation 3 target while native reports no target object.
This is an unresolved comparison, not a rules adjudication or permission to omit
that field. Original converted inputs, both native executions and first differences
are retained; neither candidate is provisionally or finally admitted here.

## Candidate validation

Eight new ordinary-discovery tests passed across focused invocations: four
accounting/statistics checks, archive corruption rejection, the three distinct
source-conversion failure reproductions, complete retained accounting, and the
actual native target-mismatch rejection. The final three logs are retained here;
no existing regression was removed or weakened. Fresh main
`44d66863d0e5f3ac01c4a2ba0069aba9add55866` is integrated. The subsequent
[`integrated-l0.json`](integrated-l0.json) again passes all ten provisional games
cold and warm, with the same immutable corpus and runner pins. It is integration
validation, separate from the five-run timing baseline. Mandatory complete PR CI,
separate exact-candidate review (including README accuracy), protected merge and
exact-main CI receipts are recorded in the delivery workpad and PR.
