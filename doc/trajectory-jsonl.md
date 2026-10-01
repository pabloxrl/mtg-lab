# Scalar trajectory JSONL

`mtg-recorder` adds persistence outside the rules core. It consumes the
[canonical in-memory contract](trajectories.md), preserving every serialized
field in owned storage types (`schema::Episode`, including owned observation
strings). [Acceptance evidence](evidence/trajectory-jsonl/README.md) covers the
atomic #77 portion of R0002-B036/B037. This is small scalar dataset persistence;
#20 retains recorder/replay integration, #79 CLI validation, and M3 retains
sharded Parquet, batch interleaving, Python readers and trainer exports.

A separate [scalar run manifest](run-manifest.md) binds this file to run-level
provenance, capture declarations and completion, with a completed-run default.
Bare JSONL validation retains the episode-level behavior below.

## Structured v2 compatibility

The same writer, bounded queue, strict reader, inventory and file publisher now
support `mtg_core::trajectory::v2`. Use `from_core_v2`, `Writer::new_v2`,
`writer.append_v2`, `read_v2` and `write_file_v2` with owned
`structured::Episode`. V2 stores the full policy-v1 observation, pending spell
and payment inputs, stack targets, combat relationships, flat candidates/masks,
factored domains/provisional selections and complete ordered Submission.
There is no fabricated selected-candidate index or lossy v1 conversion on disk.
`structured::validate` checks references within the authorized view, full
submission membership/cardinality, revision/generation, logical/microchoice
commit/cancellation timing, footer accounting and actor-only pending fields.
This remains a producer-trust contract, not proof of engine execution or of the
completeness/authenticity of a collector's domain.

| Contract | Legacy API | Structured API |
| --- | --- | --- |
| Episode schema / JSONL seal format | 1 / 1 | 2 / 2 |
| Policy observation/submission schema | Legacy view 1 | Structured policy 1, nested view 1 |
| Dataset manifest schema | 1 | 2 |
| Missing policy statistics | Explicit null (unchanged) | Keys omitted; an unsupplied policy is `{}` |

V2 nullable action-time fields (such as pending state, factored domains and
remaining cost) are still required and explicitly null when absent. Only the
optional policy statistic keys may be omitted. Unknown/mixed versions, duplicate,
extra or missing required fields, noncanonical bytes and invalid structured
records reject. Readers never try another version after a failure; even an empty
file must carry the explicitly selected seal version. Legacy APIs, fixtures and
bytes are unchanged. The [CLI validator](headless-commands.md) supports explicit v1/v2 JSONL and run manifests; v1 remains its legacy default.

`structured::Episode::seat(0 or 1)` validates and returns owned same-seat
transitions/final observations, accumulated intervening rewards, decision/logical
intervals and cancellation counts. Terminal credit is assigned once, including
seats with no decisions; opponent observations and restricted replay references
are excluded. The caller authorizes the seat and access to the full dataset.

[Independent literal fixtures, compiled red and verification evidence](evidence/structured-jsonl/README.md)
cover pending payment/targeting, ordered bottom/discard, attacker/blocker/damage
payloads, privacy, buffers/reset, integrity and errors. These are durable contract
fixtures, not #154/#117 played-game collector acceptance. No game driver, CLI,
Parquet, batching, trainer or new rules are added.

```sh
cargo test -p mtg-recorder --test structured_contract
```

## Legacy v1 use

Convert a completed or externally truncated native episode with
`mtg_recorder::from_core(recorder.episode())`. This copies and validates the
in-memory record. Failed or unfinished episodes return `Error::Incomplete`.
The native `episode()` remains available explicitly for diagnostics.

Create `Writer::new(sink, capacity_bytes, Backpressure::default())`, append owned
episodes in increasing ordinal order, and call `finish()` to obtain the sink.
`finish_with_metrics()` also returns total bytes, episode-batch write count,
occupied-buffer high-water bytes and cumulative time waiting for writes/flush.
`flush()` drains the episode buffer; it does **not** finalize a dataset.

For files, `write_file(path, episodes, capacity_bytes, mode)` creates a sibling
`<path>.partial` with exclusive creation. After the JSONL seal, flush and file
sync succeed, a hard link atomically publishes the final path without replacing
any existing file. The partial name is then removed and the parent directory
synced. On failure the fragment remains for diagnosis; never discover or consume
`.partial` files as published datasets. A stale fragment or existing final name
fails explicitly. Hard-link support and directory sync on the target filesystem
are required; there is no non-atomic fallback. A post-publication cleanup/sync
failure is reported even though the complete final file may already exist.

`read(reader, max_bytes)` returns a validated `Vec<schema::Episode>` or an error.
It requires an explicit whole-file byte budget; it never returns a successful
prefix. The result preserves global and per-seat indices, masks, observations,
rewards, optional statistics and final views. Full episode access is privileged
dataset access: both seats' separate observations are present. It is not a
policy-feature loader; do not concatenate opponent-private fields into inputs.
The in-memory API retains its seat-filtered sequence reader. Privileged seeds or
state are not added by persistence, and unknown observation fields are rejected.

## Canonical format and validation

Each episode occupies one LF-terminated UTF-8 JSON object:

```text
{"episode":{...all canonical episode fields...},"kind":"episode"}
```

The final line is the single-file integrity seal:

```text
{"decisions":N,"episodes":M,"format":1,"kind":"seal","sha256":"..."}
```

The SHA-256 covers the exact bytes of all preceding episode lines, including LF.
Objects use recursively sorted keys, compact serde_json encoding with
roundtrip-safe finite f64 parsing (including optional collector statistics), explicit nulls
for absent optional fields and no insignificant whitespace. Arrays retain order.
The committed [handwritten fixture](../crates/mtg-recorder/tests/episode.jsonl)
fixes the byte-level contract; its seal was calculated independently with Python
hashlib. Format 1 is this encoding, separately from canonical trajectory schema 1.
The reader accepts only this canonical representation, so duplicate keys, omitted
optional fields, extra fields, alternate formatting and trailing records fail.

A file has one run UUID and strictly increasing, unique episode ordinals (gaps
are allowed). This scalar ordering contract requires no growing identity set.
Each episode validates global/per-seat indices, matching episode identity,
actor/observation seat, next actor, logical/micro-choice ordering, selected legal
candidate/action equality, versioned conventions, finite optional statistics,
footer counts and authorized view shape. Terminal/truncation flags and sparse
zero-sum returns must agree; boundary rewards and decision rewards cannot be
credited twice. Failed, incomplete, corrupt and truncated *files* are rejected;
explicitly **game-truncated** complete episodes remain valid. Empty sealed files
and zero-decision episodes are supported. A missing seal, including abandonment
after `flush`, cannot become a successful dataset. Counts and SHA-256 detect
lost whole records and changed payloads.

Validation checks structure and accounting, not whether a collector's semantic
action was actually legal in a game. Truthful provenance, complete collection,
policy-private feature contents and globally unique run IDs remain the trusted
collector's responsibility. Checksums detect accidental corruption, not malicious
rewriting or authenticity. The validator does not reconstruct linked replays.

## Bounds and backpressure

The writer is synchronous and has no worker thread or unbounded queue. A pending
batch contains at most `capacity_bytes` encoded bytes. Each entire encoded episode
must also fit that budget; oversized episodes fail with `Error::Limit` instead of
truncating decisions. Native episode capture and `from_core` are caller-owned
in-memory data and require caller episode limits. Encoding scratch and sorted
JSON storage are bounded by the encoded episode budget with allocation overhead;
the buffer high-water metric reports occupied bytes, not RSS/allocation capacity.
The reader's input and decoding storage scale with its explicit whole-file budget.
This does not claim an allocation-free or streaming large-dataset reader.

Default `Block` mode drains a full batch synchronously before accepting the next
episode; sink wait is measured. `Fail` mode returns `Error::Overflow` if the next
episode cannot fit the pending buffer; callers can drain with `flush` beforehand.
Any append/drain error permanently poisons the writer: it cannot emit a successful
seal. I/O errors, including short-write failures and final flush errors, propagate
as `Error::Io`. The collector must fail the recording/run and quarantine affected
samples when an error occurs. The rules engine is never called by the sink and
is not silently advanced, reset, truncated or credited with a result.

`Write` implementations control I/O deadlines; a blocked OS/device can block a
synchronous write. This layer bounds queued data, not arbitrary sink wall time.
Callers needing a deadline must provide a sink with bounded I/O. There is no
background shutdown thread to hang or detach. Dropping a writer does not finalize
it. Seed-subset selection, multi-shard continuation/reassembly and diagnostic
loading of corrupt prefixes remain outside this atomic scalar format.

Run in the managed toolchain container:

```sh
cargo test -p mtg-recorder jsonl_contract
./scripts/torture.sh
```
