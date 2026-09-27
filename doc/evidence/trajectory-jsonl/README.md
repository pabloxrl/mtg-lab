# GH-77 JSONL acceptance

Scope: R0002-B036/B037 atomic scalar persistence in `mtg-recorder`. No full game
catalog cases are assigned to #77. #20 retains full M1 recorder/replay acceptance;
#79 owns CLI exposure and M3 owns sharding, Parquet, batch interleaving and Python.
No whole RFC block or milestone completion is claimed.

Independent expectations: RFC 0002 B036 specifies sparse +1/-1/0 rewards,
external truncation distinct from terminal outcomes and failure quarantine.
B037 requires exact records, explicit identities, counts/checksums, no silent
loss, bounded buffering and capture-neutral gameplay. The original synthetic
[episode fixture](../../../crates/mtg-recorder/tests/episode.json) describes P0
passing followed by P1 conceding: P0 receives +1 and P1 -1 at the boundary, with
one recorded action. It is deliberately not a claim of a reachable position.
The [canonical JSONL bytes](../../../crates/mtg-recorder/tests/episode.jsonl) were
authored from that ledger, with sorted compact JSON and a seal independently
calculated by Python hashlib, not captured from the Rust implementation.

## Red/green

The named `jsonl_contract` module is part of ordinary Cargo discovery. The
[initial red run](initial-red.txt) compiled and failed all five behavioral tests
against inert API methods (no compile/import failure counted). It exposed empty
roundtrip output, accepted empty/truncated input, accepted malformed schema,
accepted duplicate episode and ignored byte limits. The [publication red](publication-red.txt)
compiled and failed because a successful no-op did not create the required file.
The [metrics red](metrics-red.txt) compiled and failed because default-zero metrics
omitted the actual final batch/seal bytes. [Green run](green.txt) retains all these
assertions without changing their expectations.

Additional executable checks cover real reset/two keeps/four priority passes/
concession, capture-off equality, buffer ownership after reset, all limit reasons,
zero decisions and empty datasets, same-seat/opponent interleavings, logical
continuations, terminal-action versus boundary reward, draw and nonempty game
truncation, invalid versions/IDs/indices/masks/actions/rewards/counts/seats,
unknown/duplicate/missing fields, whole-record loss, altered payload, byte limits,
duplicate/mixed runs, short writes, disk/read/final-flush failures, sticky errors,
blocking and fail-on-overflow modes, interrupted unsealed writes and atomic
publication preserving existing files and failed fragments. Channel-controlled
sink backpressure proves the second append waits for the sink to be released;
reported wait and final byte metrics include finalization.

The real episode comparison checks every canonical serialized field against the
native in-memory episode, then checks loaded data after game reset. Captured and
uncaptured games use seed 773/ordinal 21 and identical commands. Their full
snapshot payloads (including RNG) match after normalizing only independent
allocation capability namespace fields (`scope`, `store`, `objects.id`) as in
existing snapshot/trajectory tests. No rule outcome is inferred from this
implementation comparison: the +1/-1 concession expectation is independently
asserted from B036. Native core and policy APIs are unchanged.

## Verification and limitations

`cargo test -p mtg-recorder jsonl_contract` passes all 15 named tests. The [full torture run](torture.txt) passed: documentation links, current-main
program pins/DAG, all 320 unchanged catalog designs, 132 Python tests, fmt/Clippy,
and 178 Rust tests plus the existing doctest in each debug/release profile.
The freshly fetched main remained `986d8c70fa820a025e6d0db517539db236cace63`,
already the candidate ancestor; no integration conflict or new change existed.
This ran inside the managed Docker worker without invoking Docker.

Three temporary production mutations compiled and failed existing behavioral
assertions: [ignore SHA-256](mutation-checksum.txt), [accept duplicate ordinals](mutation-duplicate.txt),
and [ignore doubled reward accounting](mutation-reward.txt). The original source
was restored byte-for-byte and all 15 named tests rerun successfully. No mutated
code, existing-test deletion, skip or expectation weakening is delivered.

The prescribed separate read-only Codex review and exact-head CI receipts are
retained in the delivery PR and issue workpad. They are required before delivery;
this committed local test report alone is not an integration verdict.

README impact: added the implemented JSONL surface, limits and evidence link;
removed the now-stale statement that trajectories have no persistence. Updated
the workspace architecture listing. Existing quickstart commands and milestone
stage verdicts remain unchanged. See [API/format/limits](../../trajectory-jsonl.md).
The format is one sealed scalar file, with one complete episode per JSONL row;
no game rule changes, full-game collector, linked-replay reconstruction,
privileged-state export, full tensor sequence loader or mature-engine agreement
is claimed. Unknown fields and noncanonical JSON are rejected intentionally.
