# Semantic opening replay v1

`mtg_core::opening::replay` records and verifies complete normal-reset opening
scripts. This is the GH-75 core portion of R0002-B016/B025 and SYS-REPLAY.
Spells and cross-feature acceptance remain with #19; the unattended replay CLI
remains #79. There is no full-game replay command yet.

```rust
use mtg_core::objects::Seat;
use mtg_core::opening::{Config, replay::{self, Action, Choice}};
let choices = [
    Choice { actor: Seat::P0, action: Action::Keep {} },
    Choice { actor: Seat::P1, action: Action::Keep {} },
];
let bytes = replay::record(&Config::default(), 42, 9, &choices).unwrap();
let mut game = replay::verify(&bytes).unwrap();
assert_eq!(game.life(), [20, 20]);
game.start_turns().unwrap();
```

`record(config, master_seed, stable_episode_id, choices)` validates and executes
all choices through `Game::reset`/`Game::apply`. It returns owned JSON bytes only
when opening is complete. `verify(bytes)` creates a fresh game, validates the
pins, repeats the same core path, compares the initial and every post-action
checkpoint, and returns the game at `OpeningComplete`. It never mutates a
caller-owned game. Internal work is drained by the scalar core API.

Choices name the actor and `Keep`, `Mulligan`, or `Bottom { cards }`. Each bottom
card is `{ "card": "mountain", "occurrence": 0 }`: frozen card identity plus
zero-based occurrence **among that identity in the current ordered hand**.
Resolve all references against the pre-action hand. The list's order is the
selected top-to-bottom order appended to the library. Same-identity copies in
opening have no distinct rules properties; occurrence distinguishes which
position is removed without recording a candidate index or process-local
handle. Missing identities, duplicate selections, wrong cardinality/kind/actor
are errors. Initial exact library orders are supported through `Config`;
subsequent mulligan shuffles use the pinned environment RNG. Explicit
per-mulligan chance injection is outside this format.

The envelope includes format/schema version, conservative engine fingerprint,
SHA-256 pins of the complete rules and card manifests, canonical typed config
hash and full config, RNG/derivation and shuffle versions, master seed and episode
ID, and semantic choices with checkpoints. Engine compatibility uses the same
source/data fingerprint as snapshots, now including replay source. Any source
change may reject old artifacts; no implicit migration is provided. The wire
schema rejects missing required fields, unknown fields and raw-index-only actions (including
extra index fields on otherwise semantic actions).

Checkpoints contain life, complete ordered hands/libraries by card identity,
starting seat, kept flags, private declarations, cumulative mulligan counts,
bottom obligations, actor/kind of the next opening decision and exact RNG state.
This is deliberately an opening-state schema, not a snapshot or full-game
fixture schema. Names and Oracle hashes are pinned through the card manifest.

`ReplayError` distinguishes malformed data, incompatible pins/config hash,
invalid core configuration, invalid semantic choice, missing choice, unconsumed
choice and checkpoint divergence. Choice indices are zero-based. Checkpoint 0
is after reset; checkpoint N is after choice N-1. Divergence includes a stable
JSON-pointer path and expected/actual JSON values; fields are compared in sorted
key order and arrays in order. Execution stops at the first mismatch. Structural
parsing and compatibility checks precede execution; a malformed later record
is a format error, not a simulated checkpoint divergence.

These are **privileged artifacts**, exposing both hands, libraries, declarations
and seeds. Keep bytes and divergence diagnostics out of policy observations and
public logs. Pins/checkpoints detect corruption and execution differences, not
author-authenticated intent: rewriting actions and all corresponding checkpoints
can describe a different valid game. Authenticate provenance externally when
required. Input size/allocation limits belong to the storage/CLI boundary.

Run `cargo test -p mtg-core --test replay` inside the managed toolchain, then
`./scripts/torture.sh`. [Acceptance evidence](evidence/replay/README.md) distinguishes
independent expected results from self-round-trip consistency.
