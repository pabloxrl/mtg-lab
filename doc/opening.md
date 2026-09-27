# Opening reset and London mulligans

`mtg_core::opening::{Game, Config, DeckConfig}` implements the reset prefix of
SYS-CORE-001, real opening-choice rejection for SYS-CORE-003, and configuration
rejection of SYS-CORE-009 (R0002-B015/B016).
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
that advances on each successful reset and application. No keep is implicitly
selected except the rules-mandated keep after seven mulligans. Completed openings can enter the [turn prefix](turns.md), including first-turn
draw timing; casting, combat resolution and full gameplay remain planned.

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
restores life, and sets a fresh decision. Pending reset owns boxed fixed-size deck arrays.
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

## Applying opening choices

`OpeningDecision` carries a game-scoped `id`, actor and `kind`. At
`KeepOrMulligan`, candidate index 0 is Keep and 1 is Mulligan. Construct
`OpeningAction { decision: d.id, selection: Selection::Choose(d.candidate(0)) }`
and call `game.apply(d.actor, &action)`. The returned `Option<OpeningDecision>`
is the next opening boundary; `None` means opening choices finished, **not** a
terminal game. Call `start_turns()` to enter the [separate turn interface](turns.md).

Declarations are collected in starting-player order. Only after all remaining
players declare are mulligan hands shuffled back and seven cards drawn. Each
mulligan taker then receives `OpeningKind::Bottom { count }`, in the same seat
order. `bottom_cards()` returns the seven current candidate handles in index
order. Submit exactly `count` distinct indices using
`Selection::Bottom(vec![d.candidate(i), ...])`. Selection order is top-to-bottom
within the appended bottom block: selecting A then B makes A drawn before B.
The cards leave the hand immediately, **before** the next declaration round
(CR 103.5 in the pinned 2026-09-25 source). Counts accumulate independently per
seat; a kept seat receives no further opening choices. At seven mulligans all
seven cards are bottomed and that seat must keep its empty hand.

For ordinary seeded replacement shuffles, the pre-shuffle sequence is the
current hand followed by the current top-first library. Apply the same
Fisher–Yates specification above using the retained environment stream, in
starting-player order among that round's mulligan takers. Reset still shuffles
in fixed seat 0/1 order. This order is part of the shuffle version contract.

`apply_with_order(actor, &action, &handles)` is a **privileged reference/test
chance injection**, not a policy action: it replaces that mulligan's shuffle
with an exact post-shuffle order. Supply all 40 distinct current handles from
that seat's hand/library. Order is validated and retained at the declaration;
no cards move before every declaration arrives. It consumes zero RNG words;
other seats still consume their normal randomness. An order on Keep or Bottom,
foreign/stale handles, duplicates and incomplete permutations are rejected.

Every action validates actor, game/decision identity, selection kind, candidate
generation/range and cardinality/uniqueness before semantic mutation or RNG
consumption. `ApplyError` distinguishes those rejection classes. Candidate IDs
are scoped to the decision that generated them, even if a reset or new decision
reuses the same numeric index. Rejection also preserves the next valid result.
Finite decision-counter exhaustion fails before mutation. Reset reserves 40
slots in each hand/library; the seven-mulligan bound keeps subsequent opening
moves within reserved storage and object generation bounds.

`draw_top(seat)` is a rules primitive for a caller that has established a legal
draw event. It moves only the top card, creates its new zone identity, preserves
remaining order, and consumes no randomness. It accepts no selected card. It
rejects uninitialized games and pending opening choices. An attempted empty draw
now records a [rules loss](terminal.md) and returns `EmptyLibrary`; later draws
return `AlreadyEnded`. It is a standalone draw boundary, not a multi-draw effect.
This primitive is not a policy action or a turn scheduler.

Run `cargo test -p mtg-core mulligan` for the normally discovered catalog and
SYS-CORE-003 opening regressions. [Mulligan evidence](evidence/mulligan/README.md)
maps every assigned case and states reference coverage limits. All inspection,
decision/chance plumbing and Debug in this module remain privileged; seat-safe
policy views are #73's delivery.

## Bounded work

`reset_quantum(&config, master, episode, NonZeroUsize)` and
`apply_quantum(actor, &action, optional_order, NonZeroUsize)` return `Progress`.
The optional order has the same privileged meaning as `apply_with_order`.
On `InternalYield`, call `resume(NonZeroUsize)` until `Decision(d)` or
`OpeningComplete`. `NotStarted` distinguishes resume before reset.
`OpeningComplete` is not game termination. Quanta may change between resumes.
The scalar `reset`, `apply` and `apply_with_order` drain this same continuation.

Each work unit performs one shuffle RNG trial (including a rejected trial),
one card allocation/move, or bounded phase bookkeeping. Fixed-size operations
include validating the two frozen 40-card decks, reserving/resetting storage,
copying a 40-card permutation, installing its order, and scanning two seats.
These fixed-size setup/validation costs are not individually charged as card
units. There is no wall-clock deadline or allocation-free claim. The rejection
sampler itself yields after each trial, with its shuffle index retained.

Pending work owns all remaining cards, permutations and cursors. During a yield,
`decision()` and `bottom_cards()` return `None`; privileged inspection may see
partial work. Apply/reset return `WorkPending` without mutation, and `draw_top`
returns `OpeningPending`. Resume at a decision or completed opening is read-only.
Accepted actions advance the decision generation once; resumes never advance it,
create a policy action, or emit a reward/terminal result. No scheduler, batch
fairness, effect continuation or trajectory reward ledger is implemented here.

Run `cargo test -p mtg-core quantum` and the full torture suite. The
[quantum evidence](evidence/quantum/README.md) covers SYS-CORE-006's core prefix;
#18 owns actual effects, #19 snapshots and #27 batch fairness.
