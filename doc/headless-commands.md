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

Trajectory validation uses the [canonical JSONL reader](trajectory-jsonl.md):
strict records, episode accounting, seal counts and SHA-256 integrity. Success
reports episode and decision counts, never private observations. Missing seals,
partial files, invalid records and corruption fail. This accepts one scalar file,
not a dataset directory or Parquet. It does not reconstruct linked replays or
certify a trusted collector's declared provenance. Replay and trajectory input
files must be regular files of at most 16 MiB. The tools buffer their bounded
inputs; callers needing larger datasets should use the recorder API with explicit
budgets.

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
