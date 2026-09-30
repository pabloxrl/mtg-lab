# Owned scalar result persistence

`mtg_recorder::collector::Run` binds the actual owned `EpisodeResult` boundary to
the existing v2 `Writer` and `Manifest::load_v2`. Related to #163; partial
R0002-B036/B037 only. Publication belongs to #164; #154/#117 retain the full
collector/integration acceptance. This component makes no durable-publication claim.

Construct a `Run` with a UUID, Config, explicit trusted policy identities, limits,
first ordinal and number of **started** episodes. Call `Run::header(ordinal)` before
`Driver::reset_captured`. Execute the real bounded driver, finish each started
episode once, retain every result, then call `Run::persist(&results, storage)`.
Results must be in consecutive ordinal order. The independent started count is a
caller lifecycle declaration: the adapter rejects missing/duplicate/reordered
results relative to that declaration; it cannot discover episodes never reported
by its caller. Full capture is the only selection mode.

The adapter checks actual immutable reset Config, ordinal, capture mode and budget
limits against the run. It rejects incompatible capture headers, including
unrelated versions/config/decks/policy identities. Unbounded drivers have no limits;
a header cannot claim limits which were never enforced. Policy identities are
explicit caller assertions; optional checkpoint/log probability/value/exploration
fields are preserved from actual capture and are not inferred or fabricated.

Provenance conventions:

- Engine: the core's existing conservative snapshot/replay compatibility digest.
- Rules/cards: SHA-256 of the exact compiled pinned metadata JSON bytes.
- Action: `policy-v` plus the actual policy schema version; observation: policy schema version.
- Deck: SHA-256 of compact key-sorted JSON for the selected frozen deck entry.
  This identifies composition; rules/cards pins identify content. Starting seat
  and stable seats `[0, 1]` are explicit.
- Config: SHA-256 of the existing Config serde JSON encoding, matching the replay
  convention, including seat order, optional exact library order, format,
  game number, sideboards, RNG and shuffle versions. Seeds remain outside default
  manifest and dataset; the master seed is not an input to this hash.
- Limits: actual driver decision/completed-turn/wall-time limits. Work quantum
  and record ceilings are execution resource controls retained in the original
  result, not extra fields invented in the existing manifest schema.

Every started ordinal is declared Completed, Truncated, Failed(reason) or
Incomplete. Only completed and validly sealed truncated trajectories are appended.
Failed or incomplete diagnostic prefixes never become training rows. A budget stop
during internal work has no final policy observations: the manifest declares that
recording Incomplete, with the actual truncation reason in RunEnd. The original
result retains its Truncated status; no frame or row is invented. Run failure
outranks truncation; incomplete runs use the manifest's truncated-run semantics.
`recording_complete` describes recording, not rules completion: a fully recorded
truncated run is still rejected by `LoadMode::CompletedOnly`. Diagnostic mode loads
only valid stored episodes and still rejects corruption and unsealed bytes.

`Storage` requires a queue/episode byte capacity, a total sealed-file byte limit,
and existing Block or Fail backpressure. Oversized rows, overflow, writes, drain,
seal and final flush errors propagate. The original results are borrowed and
survive success/failure. Serialization and validation need temporary owned copies;
these budgets do not promise a total RSS bound. Sink deadlines remain external.

`persist` uses owned memory. `persist_with_sink` accepts empty caller-owned scratch
implementing `Write + AsRef<[u8]>`, for instrumented I/O or a separate publisher's
staging buffer. Its byte view must report the actual written bytes; it must not
advertise the scratch as a dataset. Failures may leave caller-managed scratch
fragments; no successful bundle is returned. After seal and flush, the adapter
hashes the actual bytes, derives inventory counts and validates the complete
manifest against those bytes through `Manifest::load_v2` in diagnostic mode.
Checksums detect corruption, not malicious authenticated substitution.

The returned `Bundle` has immutable metadata/bytes/metrics accessors and an
`into_parts` ownership transfer for a separate publisher. Replay references are
explicitly absent at this boundary. The original privileged results remain
separate for the authorized replay component. Both-seat datasets require offline
authorization; they are not live player exports. No filesystem advertising,
sampling, automatic reset/retry, CLI, alternate recorder or new rules are added.

[Executable acceptance and independent expectations](evidence/collector-persistence/README.md).
