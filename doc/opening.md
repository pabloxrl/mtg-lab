# Opening reset

`mtg_core::opening::{Game, Config, DeckConfig}` implements the reset prefix of
SYS-CORE-001 and configuration rejection of SYS-CORE-009 (R0002-B015/B016).
Create a game with `Game::new()`, then call `reset(&config, master_seed, episode_id)`.
`Config::default()` selects red versus green, starting seat 0, game one, no
sideboards, `foundations_micro_v1`, `splitmix64-v1` and
`fisher-yates-rejection-v1`. Both starting seats and all red/green matchups work.

```rust
use mtg_core::opening::{Config, Game, OpeningChoice};
use mtg_core::objects::Seat;
let mut game = Game::new().unwrap();
let decision = game.reset(&Config::default(), 42, 9).unwrap();
assert_eq!(game.life(), [20, 20]);
assert_eq!(decision.actor, Seat::P0);
assert_eq!(decision.candidates, [OpeningChoice::Keep, OpeningChoice::Mulligan]);
```

Each player receives seven cards and retains 33 in their library (CR 103.3,
103.5). The starting player is the first keep/mulligan decision actor; reset
stops there. `decision()` returns that same boundary, with a checked generation
that advances on each successful reset. No keep is implicitly selected. Applying
these choices, London bottoming, the first-turn draw rule and gameplay are future
capabilities (#65/#67). This does not certify complete opening catalog cases.

Deck names are exactly `red` or `green`; card counts are the frozen manifest
projection. `DeckConfig::order = Some(keys)` specifies an exact **post-shuffle**
40-card order, top first, for deterministic setup/reference matching. It must
be a permutation of that deck's complete multiset; unknown cards, tokens,
wrong counts and other decks are rejected. `None` shuffles the manifest's listed
card/copy order. Explicit orders consume no randomness; other seats still shuffle
in seat order. A mixed explicit/shuffled configuration consequently consumes
only the shuffled seat's words.

## Shuffle contract

For every reset, create a fresh environment stream from the [RNG specification](rng.md)
using the supplied master seed and stable episode ID, independently of previous
consumption. Process seat 0 then seat 1. For each shuffled 40-card deck, iterate
`size = 40, 39, ..., 2`. Let `limit = floor(2^64 / size) * size`. Draw a u64 word
until it is strictly less than `limit`; swap indices `size-1` and `word % size`.
This is descending Fisher–Yates with rejection of the incomplete high tail to
avoid modulo bias. Index zero is the library top. Deal the first seven in order;
retain the rest in order. RNG and shuffle identifiers are both strict/versioned.
This algorithm is deterministic and noncryptographic, not a secrecy guarantee.

## Validation, storage and boundaries

Unsupported format, seat count, match/sideboard mode, starting seat, deck,
RNG/shuffle version or explicit order returns a distinct `ResetError`. All
configuration validates before changing any previous state, handle, capacity,
decision counter or RNG. The exact negative catalog case is
`rules-setup-two-player-opening-negative`: third seat or sideboard game two is
rejected, including before the first successful initialization.

Game state and environment RNG are owned per game. Reset reuses storage vectors,
invalidates old object handles through the storage epoch, clears all zones,
restores life, and sets a fresh decision. Scratch decks use stack arrays.
Storage capacity is reserved before clearing live state; storage/identity and
decision exhaustion report errors without discarding the previous game.
A storage allocation failure may grow reserved capacities before returning an
error, but cannot alter the semantic game or RNG. Invalid configuration does
not even reserve. No allocation-free gameplay claim is made.

`objects()`, `life()` and Rust `Debug` are privileged inspection, **not** player
observations. They expose both hands, libraries and (Debug) private RNG state;
do not send these to a policy/client or public log. No mutable storage/RNG is
exposed by this API. Handles and decision generations are process-local, not
snapshot/replay formats. Complete card effects, gameplay, observation privacy,
snapshots and replay remain subsequent work. Frozen card identities here do not
advertise their unimplemented effects.

Run `cargo test -p mtg-core opening` and `./scripts/torture.sh` inside the managed
container. [Acceptance evidence](evidence/opening/README.md) records the independent
expectations, retained regressions and red/green results.
