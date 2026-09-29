# Scalar dataset run manifest

`mtg_recorder::manifest` binds one run's declared provenance and completeness to
one existing [canonical JSONL file](trajectory-jsonl.md). The independently
[handwritten manifest](../crates/mtg-recorder/tests/manifest.json) and
[episode file](../crates/mtg-recorder/tests/episode.jsonl) are a small synthetic
pair, not a recorded played game. [Acceptance evidence](evidence/run-manifest/README.md)
covers the #116 portion of R0002-B036/B037. #117 owns collection and #20 retains
full recorder integration; M1 completion still requires #22.

## Explicit structured v2

`dataset_schema: 2` requires `versions.schema: 2`, `versions.observation: 1`
and `file.format: 2`. All other manifest declarations and integrity/completion
rules below apply unchanged. `Manifest::parse/encode/validate` supports either
explicit contract. Use `manifest.load_v2(...)` to obtain
`LoadedRun<structured::Episode>`; `load(...)` remains v1-only. Neither loader
silently migrates or falls back, and expected versions remain caller-supplied.
V2 `policy_decisions(seat)` excludes opponent inputs and replay references;
`episodes()[i].seat(seat)` additionally builds owned same-seat reward sequences.
The caller authorizes both forms of access. See [v2 durable evidence](evidence/structured-jsonl/README.md).

This extends the existing single-file manifest and publisher, with no new
coordinated multi-artifact publication API. Publish finalized JSONL before
advertising its manifest; a metadata encode result is not a published dataset.
The existing CLI trajectory validation command remains v1-only.

## Reading and writing metadata

1. `Manifest::parse(reader, max_manifest_bytes)` reads bounded JSON and validates
   declarations. All fields must be explicit, including nullable limit fields.
   Unknown, duplicate, omitted and incompatible schema fields fail.
2. `manifest.encode(max_manifest_bytes)` validates and serializes owned metadata
   as sorted compact JSON plus LF. It does not publish files or collect games.
3. `manifest.load(file_name, reader, &expected_versions, max_file_bytes,
   LoadMode::default())` checks the exact caller-supplied compatibility versions,
   inventory, SHA-256, canonical JSONL seal and each episode's provenance/status.
   The caller opens the source; the library never follows a manifest filename or
   replay reference. The filename must match a plain basename (letters, digits,
   `.`, `_`, `-`; neither `.`/`..` nor `.partial` names are accepted).
4. `LoadedRun::episodes()` is **authorized offline dataset access**, containing
   both seats' separate observations. `policy_decisions(seat)` returns only that
   seat's decision rows, without episode headers or replay references. It rejects
   seats other than 0 and 1. The caller must authorize the requested seat; this is
   not authentication, a tensor loader or a per-seat reward sequence builder.

Run the contract regression through normal test discovery:

```sh
cargo test -p mtg-recorder --test manifest_contract
```

No CLI manifest command is added. The existing `mtg trajectories validate`
command still validates JSONL alone, without a run manifest.

## Required declarations

- `dataset_schema: 1` versions this manifest, separately from `versions.schema: 1`
  (episode schema), `versions.observation: 1`, and `file.format: 1` (JSONL encoding).
  Engine/rules/cards/action identities must be nonempty and exactly match both
  the caller's `Versions` and every episode header. There is no implicit migration.
- Run UUID, two deck hashes, config hash, two policy IDs/versions (the other seat
  is the opponent), `seats: [0,1]`, starting seat and limits apply to every stored
  episode. Hashes are 64 hex digits. Changing seats' policies, decks, starting
  seat, limits or other provenance requires another run/file in this scalar v1.
- Reward is `SparseZeroSumTerminal`; discount is `UndiscountedEpisodic` (gamma 1).
  `time: DecisionAndLogicalAction` retains global policy-decision indices and
  logical/micro-choice indices from the episode format. Wall time is a limit,
  not a discount step. Both decision and logical-action counts remain in footers.
- `capture.AllEpisodes` declares a first ordinal and count. Every ordinal in
  that range must appear exactly once in the ordered episode declarations.
  `capture.DeterministicSeedSubset` records the collector's algorithm identity,
  selection config hash and population size; declared selected ordinals must be
  increasing and within `[0,population)`. This layer **does not implement or
  verify the seed selection algorithm**. The trusted collector chooses before
  outcomes and includes all decisions of each selected episode. Unselected
  episodes are outside this manifest's recording-completeness claim. No private
  seeds belong in the manifest; the config hash identifies separate configuration.
- Each selected episode declares its ordinal and `Completed`, `Truncated`,
  `Failed(reason)` or `Incomplete` status. Completed/truncated entries must match
  exactly the stored JSONL rows; failed/incomplete entries have no accepted row.
  They account for selected samples lost before a valid complete episode existed.
- The single `file` inventory supplies name, encoding, exact total bytes,
  SHA-256 of the **entire file including its seal**, stored episode count and
  decision count. The existing JSONL seal separately hashes episode lines.
  Empty sealed files are supported. Counts/checksums detect accidental changes,
  not malicious rewriting or authenticity.

## Completion and diagnostics

`recording_complete` is true exactly when every selected episode has a complete
stored row, including externally truncated games. Run `end.Completed` requires
all selected episodes to be rules-completed. `end.Truncated(reason)` requires at
least one truncated or incomplete episode and no failed entries.
`end.Failed(reason)` requires at least one failed entry. Reasons must be nonempty.
Thus recording completeness and game completion are separate facts.

Default loading rejects the entire noncompleted run with `Error::Incomplete`;
it does not silently filter out unfinished games and present a biased prefix as
completed data. Explicit `LoadMode::Diagnostic` can load the valid sealed rows of
a truncated/failed run while its manifest retains the missing/failed declarations.
It never relaxes checksums, provenance, canonical encoding or episode validation.
An interrupted/unsealed/corrupt file is rejected even diagnostically. Its manifest
can still be inspected with `parse`, and the raw fragment retained separately;
this API cannot salvage a prefix or treat a failed episode as a training sample.
Existing bare JSONL loading continues to accept completely recorded truncated
episodes; manifest loading adds the stricter completed-run default.

## Privacy, trust and limits

Replay references in stored episode headers must be opaque UUID-shaped IDs.
Paths, URLs and embedded JSON are rejected at this manifest boundary. No resolver
or privileged artifact reader exists here. References remain available only in
explicit full dataset access, never `policy_decisions`. Canonical JSONL alone
retains its prior opaque-string contract; this manifest API adds UUID validation.
Neither UUID shape nor a checksum authenticates a producer. Policy metadata and
candidate features must be supplied truthfully and without secrets by a trusted
collector. Do not publish both seats' dataset access during live play.

Both reads have explicit byte budgets; metadata and full decoded rows are owned
in memory with deserialization overhead. There is no streaming, atomic manifest
publication, collector, shard inventory/reassembly, Parquet, trainer, new reward
sequence algorithm, replay reconstruction or game/rules change in this delivery.
Filesystem durability and coordinated publication belong to the collector.
