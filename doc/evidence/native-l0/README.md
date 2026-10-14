# Native offline L0 replay

L0-004 packages the ten **provisional** XMage ComputerPlayer7 games qualified by
[the pilot](../xmage-ai-pilot/README.md). This is not final three-engine admission,
a 1,000-game corpus, an enforced latency budget, or completed performance monitoring.
The existing upstream exceptions and all original source artifacts remain unchanged.

Build and package once, using the shared heavy lock in Symphony:

```sh
cargo build --locked --release -p mtg-cli
python3 scripts/package_l0.py --output .agent-artifacts/l0-corpus
```

Use a new package directory. Packaging validates every original pilot artifact and
both independently checked source/native pairs, including negative controls, before
materializing local files. It does not run native code to author expected output.
The committed [manifest](../../../fixtures/l0/xmage-pilot-v1.json) freezes every
output checksum. A changed source, missing observation, or changed projection fails
packaging; there is no baseline-update option. A failed build must stop preparation;
never run an old binary as evidence for a failed candidate build.

Ordinary L0 needs only the prebuilt binary, packaged directory and Python standard
library for timing the process boundary. It performs no build, network retrieval,
Java execution, AI search or model invocation:

```sh
python3 scripts/run_l0.py --runner target/release/mtg \
  --manifest .agent-artifacts/l0-corpus/manifest.json \
  --manifest-sha256 "$(sha256sum fixtures/l0/xmage-pilot-v1.json | cut -d ' ' -f 1)" \
  --receipt .agent-artifacts/l0-run-1.json
```

Receipt paths must be new. The launcher enforces a 600-second deadline covering
both passes; `--timeout-seconds` sets a finite positive alternative. It drains
stdout and stderr concurrently and kills/reaps the owned process group on expiry,
retaining a failed receipt. Diagnostic stderr is capped at 1 MiB with explicit byte
counts/truncation; oversized stdout receipts fail instead of certifying a run.
This runs every game once cold and again warm in the
**same native process**. The cold clock starts before launching the binary and
ends upon receiving its first complete corpus receipt, including executable hash,
manifest/artifact loading, integrity, native execution and exact comparison. The
second full pass separately measures warm process throughput including loading,
checksums and comparisons. No games are discarded as warmup. Compilation and
packaging are separate preparation costs, never folded into replay timing. Record
build elapsed externally when benchmarking cold preparation; ordinary receipts mark
compilation as not performed. Peak child RSS and hardware/toolchain metadata accompany
counts, hashes and per-game latencies. Shared-container timings are observations,
not designated-host performance qualification.

The raw command `target/release/mtg l0 MANIFEST SHA256` prints one native JSON receipt;
its in-process clock excludes process launch and therefore **cannot** claim a full
comparable run. `--warm` emits a cold-completion JSON line followed by the final
receipt, which the launcher uses to measure both boundaries. Failures remain nonzero
and cannot become success on a later diagnostic replay. Missing timing, invalid
JSON or an unavailable binary produces a failed launcher receipt.

## Semantic contract and independent expectations

Both scalar regression and release execution use the same strict occurrence-aware
consumer and rules engine. The release path applies semantic actions through the
normal legality boundary. It retains ordered chance and opening choices, every
played choice, physical occurrence/birth/incarnation, ordered zones, life, mana,
stack/targets, effects/grants, trigger provenance, combat, outcome and required
opening/priority/resolution/SBA/cleanup/terminal checkpoints. Actual quantum probes
observe pre-SBA damage and cleanup; no state is reconstructed from a later boundary.

The expected report is a deterministic projection of the **already qualified**
scalar reports, whose source comparisons, rules-based exceptions and independent
review are retained in the pilot. Packaging reruns those independent comparisons;
it requires both retained scalar repeats to agree under this projection. All
semantic fields are deeply compared, including extra or missing fields and ordered
arrays. No semantic digests or diagnostic reruns replace assertions.

The projection excludes the recorder's `policy_capture` (including its historical
build hash), stale/continuation rejection counters and diagnostic `stops`. These
are scalar harness diagnostics, not game state. The release consumer omits only
those recorder/invalid-action probes; the original ordinary-discovery scalar tests
still execute all of them. Process-local ObjectStore arena nonces in
`raw_source_handle.store` normalize to first-seen labels (zero for these
single-arena games), preserving equality relationships; slot/generation and stable
physical identity bindings remain exact. The ten retained pairs differ only in those arena
nonces before projection. No source or gameplay expectation was regenerated.

All declared local files are checksummed, including raw traces, converted source
observations, retained reference results and qualification inventory. The expected
manifest SHA is required independently of the package; deleting a game or changing
a file cannot silently redefine the requested corpus. Failure receipts identify
replay seed, trace, action/source cursor where available and first differing field.
Manifest/integrity failures occur before gameplay and identify the artifact path.
Full counts distinguish successful, failed and unattempted entries. Repair and rerun
with a new receipt; retain the original failure. For deeper source diagnosis use
the pilot's scalar and patched-reference commands; their result cannot erase L0's
failed verdict.

## Executable acceptance

`python3 -m unittest discover -s tests -p 'test_l0_batch.py'` builds the release
runner and executes all ten actual pilot games twice, comparing the qualified
scalar semantics. It mutates checkpoint life, action suffix, terminal completeness,
pins, manifest inventory and availability; each must fail. Native unit coverage
also mutates ordered identities, incarnation and missing effect fields. The original
pilot/source comparator and scalar replay regressions remain normal discovery.

Local acceptance on 2026-10-14: all seven batch/receipt tests and 52 existing XMage
capture/normalization/pilot tests passed, including fresh twice-scalar execution of
the complete pilot. Strict workspace Clippy, formatting and documentation checks
passed. No existing tests were removed or weakened.

The [standalone receipt](run-v1.json) records **10/10 cold and 10/10 warm, zero
failures**, 4,886 actions per pass, 4.862 seconds cold full path and 4.807 seconds
warm full pass; peak child RSS 197,916 KiB. This is one shared-container observation
on the ten-game provisional workload, not a 1,000-game projection or a speed ratio.
Corpus SHA-256: `bd67e4e952d8660d83377d36da931fa933b30edf32d0ed3635c9e90bc38a032d`.
The receipt retains the exact release binary/source hashes, hardware, compiler,
individual game times, counts and explicit timing boundaries. Preparation/build
costs are outside these replay measurements.

Separate exact-candidate review, mandatory PR CI, protected merge and exact-main
CI are delivery gates. Their immutable receipts and final verdict are recorded in
[the issue workpad](https://github.com/pabloxrl/mtg-lab/issues/322#issuecomment-6172013179)
and its linked PR. Local acceptance alone does not claim protected delivery or
complete the broader L0 milestone.


The [initial independent review](review-initial.json) found an unbounded launcher
wait and stderr-pipe deadlock. [Behavioral reproductions](review-red.json) ran the
original launcher against explicitly synthetic stalled/stderr-flood transport
probes: both required the outer harness to kill the process group and left empty
receipts. The deadline/concurrent-drain repair has ordinary-discovery regressions;
these transport probes never count as gameplay acceptance. The original native
receipt remains historical evidence; repaired-launcher evidence is recorded below.

The [repaired-launcher receipt](run-v2.json) again passes 10/10 cold and 10/10
warm, zero failures: 6.085 seconds cold, 4.721 seconds warm, 197,912 KiB peak RSS.
It additionally pins the launcher SHA-256 and records the 600-second deadline.
All seven batch/receipt tests pass after repair. The 52 prior source/scalar checks
remain applicable because this repair changes only the launcher and its tests.
Both observations remain in history; no speed/regression inference is made from
these shared-container samples.

After integrating main `cd07def7bb7d42e0c664845f16fc32905cf4c8f1`, all six
affected Forge tests and all seven batch/receipt tests pass, with strict workspace
Clippy. The [first integrated receipt](run-integrated-initial.json) and
[final integrated receipt](run-integrated.json) preserve both change runs. The final
run passes 10/10 cold and 10/10 warm, zero failures, with
5.424s cold, 4.573s warm and
197,788 KiB peak RSS. An additional real opening-library-order mutation
checks failure localization: opening checkpoint indices are labeled explicitly,
without incorrectly assigning a later action's source cursor.

The [final diagnostic-check receipt](run-final.json) passes all ten games cold and
warm after preserving consumed-action cursors for extra-suffix and missing-terminal
errors. All seven tests pass with exact cursor assertions; strict workspace Clippy
also passes. This run records 4.083s cold and
4.677s warm. The prior
[integrated independent review](review-integrated.json) passed; the final candidate
requires its own review, retained in the PR/workpad rather than inferred from it.

Mandatory PR CI exposed stale compatibility metadata in the three fixed-profile
fixtures: exposing the existing consumer changes `game.rs` and `opening.rs`, both
included in the conservative snapshot engine hash. The existing combat profile
test reproduced the `unvalidated trace or incompatible pin` failure locally.
Only each fixture's `engine` pin is updated from
`18e216f6f23d6acb68254cb11d0bfd6028761b30d636f83f478cc907ff494078` to
`3a9440c1370ff21a1fe3da1fb9e2ef4bb9f80a0e28eba36e60c72047bfa46ba4`.
All actions, checkpoints and outcomes remain byte-for-byte unchanged, with their
[independent rules basis](../fixed-trace-profile/README.md). Strict pin rejection
and all existing negative controls remain enabled. This correction needs fresh
profile validation, exact-candidate review and complete PR CI before delivery.

After repair, all seven Rust profile tests, four profile integration tests and
seven batch tests pass. The [new complete receipt](run-profile-repair.json)
passes 10/10 cold and 10/10 warm with zero failures: 9.235s cold
and 7.498s warm. Batch tests now resolve the
actual executable through Cargo JSON, respecting configured target directories.
The [preceding candidate review](review-before-ci.json) is retained as history;
the corrected candidate requires fresh review and full CI.
