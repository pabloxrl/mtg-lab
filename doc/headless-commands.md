# Headless verification and scalar smoke commands

All commands emit schema-version 1 JSON/JSONL on stdout; failures emit one JSON
error on stderr. No stdin, terminal or display is required. Use the built `mtg`
binary or `cargo run --quiet --locked -p mtg-cli -- COMMAND` inside the toolchain
container. The CLI calls the existing core/recorder/comparator implementations.

```text
mtg replay verify opening.json
mtg replay verify played.json
mtg replay inspect opening.json --seat 0 --format jsonl
mtg trajectories validate episodes.jsonl
mtg trajectories validate episodes.jsonl --format structured-jsonl-v2
mtg trajectories validate episodes.jsonl --manifest manifest.json
mtg trajectories validate episodes.jsonl --manifest manifest.json --diagnostic --max-bytes 16777216
mtg conformance --suite checkpoints-v1 --fixture fixture.json --actual checkpoints.json --artifacts new-directory
mtg bench --workload scalar-pass-v1 --config fixtures/simulate/pass-v1.json
```

Each command accepts an optional **trailing** `--output NEW_FILE`; then stdout is
empty and the result goes to a newly created file. Existing paths are never
replaced. Diagnostics remain on stderr. Filesystem failures propagate; a failed
write can leave an incomplete output file and must not be accepted as a result.

| Result | Exit code |
| --- | --- |
| Valid verification/comparison or completed smoke work budget | 0 |
| Conformance checkpoint/script mismatch (JSON result with diff/artifact paths) | 1 |
| Invalid arguments, malformed/incompatible input, missing input or unsupported suite/reference | 2 |
| Missing Python/repository tooling, engine/tool execution or output failure | 3 |
| Conformance interruption/10-second timeout or benchmark wall deadline | 4 |
| Benchmark SIGINT / SIGTERM, with accounted summary when output succeeds | 130 / 143 |

Replay verification accepts the current [opening and played replay formats](replay.md).
The required top-level `format` and integer `version` select exactly one core
verifier: `mtg-core-opening-replay`/1 or `mtg-core-played-replay`/1. Missing or
duplicate discriminators are malformed; unsupported pairs fail explicitly.
There is no parser fallback or migration. The core reconstructs normal reset,
consumes every semantic choice and checks every recorded checkpoint. Played
histories must be terminal, with no missing or extra choices.

The caller supplies an existing library-produced file (these commands do not
capture or create replays). Selecting its local path deliberately reads a
privileged artifact containing both seats' secrets. This does not resolve opaque
replay IDs, persist a grant, or authenticate provenance. Successful opening
verification retains its `scope: "opening-v1"` summary. Played verification emits:

```json
{"schema_version":1,"type":"replay_verification","status":"verified","scope":"played-v1","checkpoint":"terminal","life":[20,20],"outcome":{"winner":"P0","losses":[null,"Concession"]}}
```

This example describes a P1 concession with unchanged life; life/outcome reflect
the verified public terminal state. Verification never dumps hands, seeds, paths
or semantic payloads. Both verify and inspect report only error categories:
`Malformed`, `Incompatible`, `InvalidConfig`, `Divergence`, `MissingChoice`,
`UnconsumedChoice`, `InvalidChoice`, `SemanticChoice`, `Storage`, or `Turn`, in
`replay CATEGORY; privileged detail withheld`. Input failures use a fixed
readable-regular-file/byte-limit message. All these failures exit 2 with the
existing schema-version 1 error object. Detailed private divergence remains
available only through the deliberately privileged core API.

Inspection remains **opening-v1 only** and rejects played artifacts explicitly.
It verifies the entire opening artifact before emitting only its final observation
for the explicit seat through `Game::observe`. Verification does not enable
privileged inspection, and no `--privileged` flag is supported. Seat filtering is
an output contract, not host authentication. [CLI acceptance](evidence/played-replay-cli/README.md)
covers normal-reset land/concession play, legacy opening, routing and redaction.

Trajectory validation uses the [canonical strict readers](trajectory-jsonl.md).
Select exactly one input contract:

- `trajectories validate DATA` retains legacy v1 JSONL behavior and summary.
  `--format scalar-jsonl-v1` selects the same reader explicitly. As before, valid
  v1 stored truncations are accepted; use a manifest to enforce run completion.
- `trajectories validate DATA --format structured-jsonl-v2` selects v2, including
  exact structured observations and submissions. Completed episodes are required
  by default; `--diagnostic` admits valid sealed truncated episodes.
- `trajectories validate DATA --manifest MANIFEST` loads explicit dataset schema
  1 or 2 through `Manifest::load`/`load_v2`. The caller selects **both paths**;
  DATA's basename must match the manifest inventory. `--format` cannot be combined
  with `--manifest`. Completed runs are required unless `--diagnostic` is present.

There is no format guessing, fallback parser, migration, directory discovery,
Parquet support, or replay resolution. Missing/duplicate/unsupported versions,
mixed records, missing/duplicate declared episodes, metadata/provenance mismatch,
checksum/count mismatch and unsealed/corrupt files fail even in diagnostics.
Diagnostics never salvage fragments or filter a run into a completed prefix.
Failed/incomplete declared episodes have no stored episode; their counts remain
in the summary. A diagnostic success validates the data/metadata contract, **not
completion of the run**. Diagnostic reasons, paths, IDs, hands, seeds, policy
names and replay references are never emitted.

Inputs must be explicit regular files; final-component symlinks are rejected.
The manifest inventory name is validated and compared, never opened implicitly.
On Unix, no-follow/nonblocking open flags also reject symlink/FIFO replacement
between the path check and open. Ancestor directories remain caller-selected;
this is not a filesystem authorization boundary. Neither reading a dataset nor
an opaque replay ID grants replay access. Both-seat datasets themselves remain
privileged offline inputs; the validator emits only counts/status.

`--max-bytes N` accepts 1 through 16777216, defaults to 16 MiB, and bounds **combined
manifest plus data bytes** (or just data without a manifest). Inputs are buffered
within that byte budget; decoded objects add memory overhead. Duplicate/unknown
options reject. Recorder APIs remain available for larger explicit budgets.
Replay input separately retains its 16 MiB bound. Use an external job timeout for
synchronous filesystem stalls; this command introduces no collector or publisher.

Legacy success is unchanged:

```json
{"schema_version":1,"type":"trajectory_validation","status":"valid","format":"scalar-jsonl-v1","episodes":1,"decisions":1}
```

V2 adds stored completed/truncated counts. For the two land/concession games in
[real played acceptance](evidence/trajectory-cli/README.md):

```json
{"schema_version":1,"type":"trajectory_validation","status":"valid","format":"structured-jsonl-v2","episodes":2,"decisions":10,"completed":2,"truncated":0}
```

Manifest success reports `format: "run-manifest-v1"` or `"run-manifest-v2"`;
`episodes`/`decisions` count stored data, `declared_episodes` counts all declarations,
and `completed`, `truncated`, `failed`, `incomplete` partition those declarations.
`run_end` is `completed`, `truncated`, or `failed`; `recording_complete` retains
its distinct manifest meaning (a fully recorded truncated run can be true).
Noncompleted diagnostics use `status: "valid_noncompleted"`, including empty
sealed data for an entirely failed run. V2 JSONL diagnostics use that same status
when any episode is truncated. Successful validation, including diagnostic mode,
exits 0; rejection exits 2 without a success prefix. Errors are fixed redacted
messages, not underlying parser/I/O details. Output failures exit 3.

Manifest provenance/version strings are declarations cross-checked against every
stored episode, not authentication or proof that the current engine can replay
a historical dataset. Supported storage/observation schema versions are checked
by the delivered readers. Authenticity, policy identity and gameplay truth remain
producer/caller responsibilities; replay verification is a separate explicit action.

`checkpoints-v1` invokes the existing `scripts/checkpoints.py` strict neutral
comparator with Python 3. It requires this source checkout at the build-time path
(including schemas/data), `python3` on PATH and two regular JSON inputs of at most
1 MiB each. Both the fixture and the observed checkpoint file are explicit;
expected results are never calculated by the command. It checks full script
consumption, named assertions and invalid-action invariants and preserves the
fixture, actual checkpoints and first difference in a **new** artifact directory.
These artifacts are privileged. A pass means **supplied-checkpoints-only**, not
engine execution or reference agreement. Missing choices yield a script mismatch.
Malformed input or unavailable tooling fails; `all`, `fast` and `--references`
are not yet supported by this initial entrypoint and fail explicitly. Existing
XMage/Forge commands remain documented separately in the reference runbooks.
The subprocess has closed stdin, removed display variables, a 10-second deadline,
and a 1 MiB cap on each captured output stream. Timeout/interruption can leave
incomplete diagnostic artifacts and always fails.

`scalar-pass-v1` times the existing production scalar simulation with the
[simulation configuration](simulate.md). Both policies must explicitly be
`pass-v1`. The work includes reset/shuffle, policy selection, core actions and
summary JSON encoding into memory; terminal/file rendering occurs afterward.
The result embeds resolved run metadata (configuration, seed, rules/cards/engine
hashes), aggregate outcome counts and raw elapsed nanoseconds. No per-action
subprocess or terminal output is used. Caps are 100 episodes, 10,000 decisions per
episode and a cooperative 10-second wall deadline (or a shorter configured one).
Decision-limited episodes remain truncated; only rules outcomes count as
completed. Errors and interrupted/unfinished work retain the simulation accounting.

This is an honest command smoke, **not M2 performance qualification**. It does not
implement the future `foundations_micro_v1` performance workload, warmups/repeated
windows, hardware qualification, native random/heuristic policies, worker sweeps,
profiling or throughput targets. Timing includes summary encoding and is diagnostic
only. Ordinary file I/O is synchronous; deadlines do not preempt a stalled kernel
filesystem operation. Use an external job timeout as with existing simulation.

Acceptance: `cargo test -p mtg-cli --test headless_commands`, all CLI tests via
`cargo test -p mtg-cli`, and the full `./scripts/torture.sh`.
[Delivery evidence](evidence/headless-cli/README.md).
