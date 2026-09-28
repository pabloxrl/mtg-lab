# Scalar run manifest acceptance (#116)

Scope: R0002-B036/B037 manifest component only. RFC 0001 trajectories/data
integrity and RFC 0002 §8 require explicit provenance, conventions, capture and
completion, reject incomplete data by default and keep privileged replays out of
policy inputs. The original #20 integration and #22 milestone gate remain open.

Original test inputs: the existing independently handwritten one-decision
synthetic concession episode (`crates/mtg-recorder/tests/episode.json[l]`) and new
handwritten `manifest.json`. Expected run UUID, ordinal 7, seat policies/decks,
versions, decision count 1, returns `[1,-1]` are specified directly. The whole-file
length 2318 and SHA-256 `778584a206189a8ad3352b8bd26cb6fbca143b44c21459f8a7f3854a1dff8bb6`
were calculated with Python hashlib over the committed fixture, independently of
the manifest implementation. Test-generated alternate JSONL uses the already
accepted writer only for encoding, never to derive expected status/reward facts.

Tests were written before validation. Minimal callable typed API scaffolding
parsed declarations and delegated directly to existing JSONL; it intentionally
had no new compatibility, inventory, completion or replay-ID validation.
[Compiled red](red.log): 3 passed and 5 behavioral failures (not import errors).
After implementing validation, unchanged initial assertions pass, with four
additional edge-case regressions: [focused green](green.log), 12 tests.

`cargo test -p mtg-recorder --test manifest_contract` is in normal Cargo discovery:

- Handwritten pair exact metadata and episode roundtrips, reward independently fixed.
- Every top-level and nested version/limit/inventory field required; unknown and
  duplicate fields rejected; explicit byte budgets and unsupported conventions.
- Consumer compatibility and every header provenance field bound to the file.
- File name/encoding/bytes/checksum/counts and selected-episode inventory bound;
  duplicate/incorrect ordinals and mismatched completion rejected.
- Failed/interrupted run declarations survive diagnostic roundtrip/loading;
  default completed data rejects them, as well as completely recorded truncations.
- Corrupt/unsealed prefixes rejected even with a recomputed outer checksum.
- Empty sealed runs and I/O failures; deterministic subset declaration preserved
  with bounds checks (selection itself remains collector-owned).
- Replay UUID excluded from policy rows; alternate-seat and invalid-seat access,
  path/URL/embedded-seed replay-reference rejection.

No existing tests were removed, skipped or weakened. No rules behavior changes;
no new cached reference-engine agreement is claimed or needed for this metadata
component. README gains the API, bounds and limitations; its milestone verdict
is unchanged. Full torture and separate review evidence follow below.

[Full post-integration torture receipt](verification.json): 134 Python tests and 270 Rust tests including doctests in each debug/release build passed in the managed Linux container. Documentation, program, catalog, formatting and Clippy passed. The documented manifest command also passed all 12 tests. No existing quickstart command changed; the full suite exercises its CLI checks. Independent review and protected CI receipts are recorded in the delivery PR/workpad.
