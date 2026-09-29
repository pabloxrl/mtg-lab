# Core snapshots

`Game::snapshot() -> Vec<u8>` and `Game::restore(&[u8]) -> Result<(), RestoreError>`
are privileged full-state operations in `mtg_core::opening`. The error type and
`SNAPSHOT_VERSION` are in `mtg_core::opening::snapshot`.

```rust
use mtg_core::opening::{Config, Game};
let mut game = Game::new().unwrap();
game.reset(&Config::default(), 42, 9).unwrap();
let bytes = game.snapshot();
let mut resumed = Game::new().unwrap();
resumed.restore(&bytes).unwrap();
let decision = resumed.decision().unwrap(); // reacquire scoped IDs after restore
assert_eq!(decision.generation, game.decision().unwrap().generation);
```

The version-1 JSON envelope contains `format: "mtg-core-snapshot"`, a numeric
`version`, an `engine` fingerprint, a hexadecimal `sha256`, and a JSON string
`payload`. The checksum covers the exact UTF-8 payload bytes. The conservative
engine fingerprint hashes the core source files and frozen card/rules manifests;
source changes (even formatting) reject old saves. Unsupported versions and
incompatible fingerprints are explicit errors. There are no migrations and no
cross-engine compatibility promise. Internal payload field layout is not a
policy or replay schema. See [semantic replay](replay.md) for opening and played-game formats.

State includes the RNG word, object slots/free list/ordered zones, object and
reset generations, knowledge history, life, episode identity/outcome, opening
declarations/keeps/mulligan counts/explicit chance orders, queued work and every
work cursor, decisions, turn/priority/mana state, private targets/payments/casts,
stack/effects/modifications, summoning sickness and combat state. Immutable card
identities are resolved through the fingerprinted card table. Vector allocation
capacity and process-global identity counters are not game state.

Restore verifies the envelope, versions, checksum, typed payload, storage
membership and opening/spell-work bounds in temporary owned state. Only the final
assignment replaces the destination; any returned error leaves all destination
state, RNG and pending work unchanged. Loading does not advance rules or consume
RNG. To resume an internal yield, call `resume` with a positive work quantum.

Every successful restore allocates a fresh process-local capability scope, even
when restoring into the original `Game`. Object/reset/decision generations and
semantic state are preserved; scope numbers are rebased throughout handles and
decision/episode tokens. Reacquire handles and decisions from the restored game.
Source-game and pre-restore destination handles/actions remain invalid there.
Snapshots can be loaded in a fresh process without identity collisions. This
also means serialized bytes differ after restore only in scope-bearing fields
(and their checksum); compare semantic state, not scope numbers.

These artifacts expose both hands, library order, private continuations and RNG.
Keep them restricted; never substitute them for `observe(seat)`. Load saves
produced by a trusted compatible engine: SHA-256 detects accidental corruption
but is not authentication or a proof that a deliberately rewritten state is
reachable/legal. Restore is not a general synthetic-state constructor, and does
not claim complete semantic validation of attacker-authored payloads. Ordinary
allocation exhaustion follows Rust allocation behavior; no bounded-memory or
allocation-free guarantee is made.

Run `cargo test -p mtg-core snapshot` in the managed toolchain, then
`./scripts/torture.sh`. [Acceptance evidence](evidence/snapshot/README.md) covers
opening decisions/yields, corruption, RNG, identity isolation, new-process load,
turn priority, historical knowledge and terminal/reset boundaries. The tests now include
complete cross-feature pending-choice/response-chain snapshot acceptance in the
[integration report](evidence/private-replay-integration/README.md).
[Spell settlement acceptance](evidence/spell-settlement/README.md) additionally
checks restore at every internal spell-work phase using the actual Growth/Bite
and creature APIs. No reference-engine agreement is implied by snapshot equality.


[Combat settlement acceptance](evidence/combat-settlement/README.md) checks restored
life/mark/death/bookkeeping/publication work at every quantum-1 yield, both seats,
and terminal suffixes. These are trusted compatible-engine snapshots; old engine
fingerprints still reject without migration.

[Turn settlement acceptance](evidence/turn-settlement/README.md) checks restore
after every yielded untap, step, draw, discard and combined modifier expiration.

The additive [policy interface](policy-decisions.md) also binds submissions to a
destination-local revision. Successful restore increments it; it is excluded
from snapshot payloads and never rewinds with a save. Refresh the policy decision
after restore. Counter exhaustion rejects the restore transactionally. Existing
privileged capability scopes, semantic generations and opening-view API contracts
are unchanged; the conservative engine fingerprint now includes policy source.

The nullable top-level RNG, decision, episode and outcome fields must be present
explicitly. Missing fields are corruption, not implicit `null`; rejection leaves
the live destination unchanged. Valid unstarted and terminal snapshots remain
supported. This decoder change updates the conservative engine fingerprint;
older artifacts reject explicitly, with no migration.
