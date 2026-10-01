# Unattended M1 integration and first active baseline — GH-21

This audit covers every original #21 clause after the composed CLI prerequisite
#120. Base: `762d331984c8f312b4a7c85fd1ed7cbdd1e6edc1`. The bounded addition is
`bench --workload native-rollout-v1`: the existing owned native simulation client,
with optional policy timing, unmodified episode/accounting rows and decision/work
counters. No rules, policy, recorder or supported card pool changes.

Delivery is conditional on full torture, separate clean-candidate review,
protected merge and successful CI on that exact main commit. Final receipts live
in the [single workpad](https://github.com/pabloxrl/mtg-lab/issues/21#issuecomment-5858829865)
and linked PR. This is M1 integration evidence, **not the M1 gate verdict**.
#22 retains that verdict; partial ownership of B020/B039/B041 does not satisfy
their later-stage obligations.

## Behavior and independent expectations

The [red log](red.log) records successful compilation followed by two intended
behavioral failures: the active benchmark command returned structured usage exit
2 instead of running the native workload. The negative configuration case already
passed. A redundant test-helper assignment warning was subsequently removed;
that warning was not the failing evidence. The [green log](green.log) records all
five new subprocess cases and the pending-signal unit case passing.

All six additions remain in normal test discovery:

- `native_benchmark_preserves_production_rows_and_all_failure_denominators`:
  both policy identities, three requested games, one-decision truncation and
  one-record exhaustion. CR 103 opening choices cannot end a normal game;
  the declared budgets independently require zero wins/draws, three truncations
  or one failure plus two not-started requests. Exact equality with ordinary
  simulation additionally checks instrumentation does not alter execution.
- `native_benchmark_complete_games_equal_untimed_simulation`: normal completed
  gameplay, exact episode/summary/decision/work equality with the production
  client. This is an instrumentation equivalence check, not a winner oracle.
- `native_benchmark_rejects_other_workloads_and_excessive_budgets`: passive,
  script, episode-count and decision-count violations fail explicitly.
- `native_benchmark_work_and_wall_stops_never_become_completed_games`: reset-only
  work and one-millisecond deadline preserve all-requested accounting, no rules
  wins/draws, episode-row count and reset-call counters.
- `native_benchmark_rejects_capture_before_creating_artifacts`: explicit
  no-capture workload rejects capture with no destination changes.
- `commands::benchmark_tests::native_benchmark_preserves_pending_signals_before_first_episode`:
  SIGINT/SIGTERM return 130/143, zero starts and both requests not started.

Subprocess helpers close stdin, remove DISPLAY/WAYLAND_DISPLAY, pipe stdout/stderr,
and enforce external kill/reap timeouts. Expectations derive from the RFC outcome
contract and explicit budget configuration; no generated benchmark winner is
installed as expected rules data. Existing tests/assertions remain unchanged.
No cached-reference rerun is claimed for this instrumentation-only change.

The first full suite caught a compatibility regression in the pre-existing
`input_limits_and_benchmark_configuration_fail_explicitly` test: the passive
budget diagnostic lost its documented `smoke limits` text. The original message
was restored; the unchanged regression passes. The first independent review
passed with one advisory: the measurement driver omitted a timed-out or malformed
attempt because Python raised before publication. `tests/test_unattended_measure.py`
now reproduces actual timeout/partial output and malformed/missing JSON, and
retains nonzero structured outcomes. [Behavioral red](measure-red.log) and
[green](measure-green.log) show that these become explicit failed samples, with
elapsed time and available output, rather than disappearing. The loop continues
to retain subsequent attempts. A launch error likewise becomes a failed sample.
Neither failed samples nor partial JSON become completed games. The final
baseline was rerun after the compatibility fix; the first review does not approve
the repaired candidate, which requires a new full suite and independent review.

## Complete original acceptance crosswalk

Each listed suite is executed again by the full torture command, including the
composed tests; this verdict is not inferred from closed child issues. Paths are
under `crates/mtg-cli` unless stated otherwise. The prerequisite's
[full crosswalk](../composed-cli/README.md) supplies finer-grained names and its
independently reviewed script ledger; those obligations are retained here.

| Original #21 clause | Executable acceptance and evidence |
| --- | --- |
| Noninteractive simulation | `tests/native_simulate.rs`, `tests/script_simulate.rs`, `tests/composed_cli.rs`: real native/script games, both starters, no stdin/display, complete script consumption |
| Fixture checks | `tests/headless_commands.rs`: actual checkpoint comparator, true mismatches with diff artifacts, missing dependencies/reference, malformed input and comparator timeout; supplied-checkpoint scope is explicit, not a claim of reference execution |
| Replay checks | `tests/played_replay_commands.rs`, `tests/headless_commands.rs`, composed producer-to-verifier: opening and terminal played formats, version/corruption/missing-choice rejection, private diagnostic redaction |
| Structured results | All CLI suites check versioned JSON/JSONL, exit status and error output; resolved config, seeds, engine/rules/cards/policy identity, limits, workers/capture/instrumentation are retained; no terminal progress/ANSI output |
| Deterministic random opponent | Native repeated games and direct-library equality; `mtg-policy` selection/games tests cover independent seat/episode RNG vectors, hidden twins and legal choices; native benchmark failure cases and 40 measured games |
| Deterministic heuristic opponent | Native repeated games; `mtg-policy` heuristic/heuristic-games tests cover literal fixed choices/ties, hidden twins and complete games; native benchmark cases and 40 measured games |
| Explicit episode/work budgets | Native/script/composed tests cover counts, invalid bounds/overflow, reset-only work, safe-boundary work stops, record exhaustion and not-started requests; benchmark preserves the same limits |
| Truncation limits | Native/script/captured/composed suites distinguish decision/turn/deadline limits from terminal outcomes; benchmark adds independent decision/work/deadline denominator checks |
| Interruption summaries | Native/captured/composed real SIGINT/SIGTERM and deadline subprocesses; injected owner/publication stops test exact boundaries; benchmark pending-signal and real deadline cases |
| Close stdin/unset display | Every automation command is exercised by the above headless subprocess helpers, including legacy simulate/bench, native/script simulate, conformance, opening verify/inspect, played verify, v1/v2/manifest trajectory validation |
| Malformed inputs fail explicitly | Headless/native/script/replay/trajectory suites: bad files/config/schema/bounds, absent policy/dependency/reference, corruption, invalid output and broken stdout; no prompt or fallback |
| Missing decisions fail explicitly | Script and composed missing/extra/reordered choices, illegal actions and unresolved choices produce caller failure; `src/script_tests.rs` verifies full owner/capture nonmutation and subsequent recovery |
| No required UI/IPC/trainer | Actual commands above run directly as native clients; Cargo dependency graph has core/policy/recorder plus CLI serialization/hash/signal dependencies; Python is only used by the checkpoint comparator, not simulation; no session server or trainer starts |
| Benchmark/counters before broadening pool | Passive command preserved; new native rollout uses the unchanged six-card M1 capability subset and reports accepted decisions/work calls plus total/policy time |
| Outcome/failure accounting | New truncation/capacity/work/deadline/signal tests retain failures and nonstarts; requested = started + not_started, started = completed + truncated + failed + incomplete; wins/draws only for rules outcomes |
| Initial measured baseline even if slow | Raw five-repeat native random AND heuristic measurements below; all attempts, versions/source hashes, hardware, limits and timing boundaries retained |
| Invalid actions unchanged | `src/script_tests.rs::routing_rejects_preserve_full_owner_and_capture_then_valid_script_recovers`, core `tests/semantic_input.rs`, composed wrong-incarnation checkpoint: complete state/RNG/history/capture/cursor comparison, no substitute engine |
| Trajectory boundaries | Composed simulate → canonical manifest/JSONL → public reader/replay equality; literal rewards/final states and incomplete/publication rejection; core `trajectory_v2_*` and recorder collector suites retain same-seat/opponent intervals, nonacting terminal rewards, pending truncation, zero-decision seats and once-only returns |

## Catalog and authoritative block audit

Current `capability-test-plan.json` has **zero cases directly owned by #21**.
That does not erase the integration families. `atomic-delivery.md` assigns:

- **SYS-CLI-001:** complete automation matrix and negative paths above; supported
  replay inspection is opening-only and other modes fail explicitly.
- **SYS-CLI-002:** native/script/captured/composed requested-episode equations,
  mixed outcomes, real signals, provenance and declared resource limits.
- **SYS-CORE-009 public configuration:** native bad version/policy/bounds tests,
  headless unknown suite/reference errors, script illegal/missing input and record
  capacity failure remain distinct from rules outcomes. Internal core fault and
  unsupported-card categories remain covered by the core integration suites;
  no CLI fault injection or approximate card substitution is introduced.
- **SYS-CLI-008 M1 portion:** native headless simulation needs no terminal, IPC
  server or trainer. Python collection and interactive accepted-command-to-view
  latency belong to M3/M4 and are not claimed by this audit.

The game-replay plan's M1 normal-reset script/replay and rejection/privacy portion
is exercised through composed/played replay tests and the retained core replay
suite. Its independent full-pool dual-reference game matrix remains a later
qualification obligation. Data/RL plan M1 portions run via canonical collector,
same-seat reward/boundary, corruption/privacy and CLI capture suites; batching,
autoreset adapters, sharded Parquet and actual trainer updates remain M3. Contract
defaults remain intact: game outcome differs from recording success, no invented
action for a nonacting seat, final reward credited once, private replay authority
separate from opaque links, diagnostic admission does not pardon corruption.

**B020:** native rollout now has actual initial measurements with reset/all-choice
cost and separately reported policy time. Core-transition, encoded/inference,
capacity/stress, CLI rendering latency, all-pool tracks, worker/batch sweeps,
warmup/long-window dedicated-host qualification and allocation/profiling reports
remain with the existing later owners, as the reviewed
[CLI prerequisite plan](../../programs/cli-prerequisites.md) explicitly assigns.
The short runs below cannot pass those qualification requirements.

**B039:** validated noninteractive configurations, both supplied seats, versioned
machine output, errors, explicit budgets/signals, deterministic native policies
and capture/replay verification are exercised above. No per-action process is
spawned by simulation. The illustrative full-pool/Parquet commands are not current
interfaces; Python batches/training and the persistent JSONL decision protocol
remain M3/M4, with no silent substitutes.

**B041:** supported commands/tests run unattended with bounded subprocesses and
actionable artifacts. Missing references fail rather than count as agreement.
This change adds no reference, fuzz or RL integration; their broader execution
matrix remains later work. Independent script/CR expectations are retained;
native policy outcomes are measurements and equality probes, never rules oracles.

## Initial measurement, not qualification

Reproduce in the managed toolchain container from repository root:

```sh
cargo build --locked --release -p mtg-cli
python3 doc/evidence/unattended-integration/measure.py /tmp/new-m1-baseline.json
```

`measure.py` refuses to overwrite an existing artifact. [baseline.json](baseline.json)
contains all 80 attempts: five repetitions of 16 cases, each one episode, both
policies separately self-playing red/green, green/red, red/red and green/green,
both starting seats. Master/policy seed 42, ordinal 0, no warmup, no best-run
selection. These are repeated identical traces, not 80 independent strength
samples. The production fixture fixes 20,000 decisions, 100,000 work calls,
20,000 records and work quantum 64. Each invocation has the command's cooperative
10-second cap and an external 30-second timeout. Capture is off; ordinary retained
semantic history and summary encoding remain inside the timing window.

Recorded 2026-10-01 on shared Linux aarch64, four visible Apple CPU cores,
8,113,364 KiB visible RAM; affinity allowed cores 0–3, **not pinned**; one worker.
Rust 1.98.1 release build, empty RUSTFLAGS, Python 3.12.3, complete compiler/OS/CPU
metadata and binary/source SHA-256s in the artifact. Source hashes identify the
measured source exactly against the artifact's base commit; the final reviewed commit
is linked from the workpad. Resolved configuration/version hashes occur in every
sample. CPU visibility is not dedicated-host reservation or an RSS measurement.

Per-repeat aggregate rates divide eight completed games or all accepted decisions
by the sum of that repeat's eight **whole command measurement windows**. No work
is subtracted for failures. All 40/40 requests completed for each policy; neither
had truncations, errors, incomplete or not-started episodes. Wins by seat were
heuristic `[30,10]`, random `[20,20]`, draws zero; these are descriptive, not strength
or correctness claims. Individual raw episode outcomes/counters remain available.

| Policy | Total seconds per repeat (all five) | Completed games/s median [min, max] | Accepted decisions/s median [min, max] |
| --- | --- | --- | --- |
| heuristic-m1-v1 | 0.850823, 0.832826, 0.832548, 0.828327, 0.903686 | 9.606 [8.853, 9.658] | 4507.5 [4154.1, 4532.0] |
| legal-random-m1-v1 | 3.676727, 3.650508, 3.655348, 3.655152, 3.646188 | 2.189 [2.176, 2.194] | 3886.6 [3863.8, 3896.1] |

Accepted decisions per eight-case repeat: 3,754 heuristic, 14,206 random. Policy
initialization/choice time occupies 0.0930–0.1017% and 0.0669–0.0686% respectively;
observation construction/submission is outside that subset but inside total time.
These clock-granularity-sensitive numbers are not evidence of a dominant cost.
The benchmark excludes executable startup/config parsing/final output; raw
`process_elapsed_ns` includes the external process interval separately. See the
[command contract](../../headless-commands.md#initial-active-policy-benchmark)
for exact timers, counters and cooperative interruption limits.

Short shared-container windows, partial card support, unpinned cores, no warmup,
no memory/allocation profile, no capture overhead comparison and no worker sweep
preclude RFC performance qualification. Exit zero alone does not prove completion;
work-limited incomplete episodes can exit zero. Dataset publication failure does
not rewrite a real game outcome. Storage stalls/crashes are not preempted by the
cooperative clock. These limits remain explicit in README and command docs.

## Verification and README impact

Run `./scripts/torture.sh` (includes `verify.sh`, full debug/release tests), not
only the new benchmark filter. The final workpad/PR records the unchanged
candidate's counts, log checksum, independent review and exact-main CI.
README now documents the active command, links this acceptance report and keeps
the M1 gate pending. The affected quickstart command must be exercised headlessly
before delivery; no milestone completion is inferred from this integration.
