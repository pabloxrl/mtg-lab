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

## Complete played-game replay v1

`opening::replay::played::{record, verify}` extends the core persistence API to
complete games using `opening::actions::Record` (action version 1). The opening
v1 API and CLI remain supported separately; the CLI does **not** accept this new
format yet. Example:

```rust
use mtg_core::{objects::Seat, opening::{Config, actions, replay::played}};
let choices = [actions::Record {
    version: actions::ACTION_VERSION,
    actor: Seat::P1,
    decision: "concession".into(),
    choices: vec![actions::Choice::Concede {}],
}];
let bytes = played::record(&Config::default(), 42, 9, &choices).unwrap();
assert_eq!(played::verify(&bytes).unwrap().outcome().unwrap().winner, Some(Seat::P0));
```

A recording starts with normal reset and accepts every implemented semantic
choice, including mulligan/bottom, lands, mana, creature/Growth/Bite casts,
ordered targets, payments/cancellation, combat, cleanup and concession. Completing
opening automatically enters first upkeep; this is engine work, not a fabricated
player choice. Every submitted choice is validated through the existing action
codec and core. A stream must reach a terminal outcome, with no extra choices.
An unfinished prefix fails rather than masquerading as a completed game.

The separate `mtg-core-played-replay` envelope pins format/version, action version,
engine source fingerprint, complete rules/card manifests, PRNG/shuffle versions,
canonical configuration hash, full configuration, master seed and episode ID.
Initial and every post-choice checkpoints include ordered zones with birth and
zone-incarnation identity, owner/controller/tapped state, creature power/toughness
and damage, life, opening declarations/bottom state, RNG, turn/step/active player,
priority/pass state, mana and land use, pending payment/cast/targets, stack and
ordered target relationships, temporary modifications, summoning status, pending
and committed combat selections/assignments, and terminal reason/winner.
Historical references retain their witnessed incarnation after a zone change.
Process-local handles and live decision IDs are removed. Internal work is drained
before each checkpoint; snapshots remain the API for saving internal yields.

Verification recreates the game from configuration and choices, compares the
initial checkpoint and each subsequent checkpoint in order, and reports the first
mismatch's checkpoint index and JSON field path with expected/actual values. Index
zero is reset; index `n + 1` follows choice `n`. Missing/extra choices and illegal
semantic references are explicit failures. Checkpoint values have exact field-set
comparison; absent or additional fields cannot silently disappear. This is an
in-memory diagnostic format, with full checkpoint cost per choice; it is not a
streaming dataset collector or throughput API. The factored M1 policy candidate
capacity is 256 (80 initial cards); any overflow remains an explicit error.

All bytes, seeds, checkpoints and detailed mismatch errors are **privileged**.
They must not be sent as a seat export or fed to a policy. Authorized observations
continue through the separate seat-filtered APIs. Artifact integrity is not
cryptographic authentication against an attacker rewriting both actions and
checkpoints. No migration or cross-engine shuffle injection is supplied: randomness
is pinned mtg-lab PRNG execution, with optional exact initial orders in `Config`.

[Played replay acceptance](evidence/played-replay/README.md) covers the normal-reset
response/cleanup/combat/life-loss script, fresh-process verification, bounded-work
suffix comparison and corruption controls. #19 retains all original integration
and GR-010/011/012/020–022 expectations, including cross-engine obligations with
#24/#38. This component does not complete those integrations or M1 (#22).
