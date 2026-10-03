# Core player views, schema 1

`mtg_core::opening::Game::observe(Seat)` returns an owned, structured
`PlayerView`. It borrows no engine state and contains no `Handle`, process-local
scope, RNG, seed, deck configuration or library identities/order. The records
implement `serde::Serialize`; JSON consumers can use `serde_json::to_vec`.
`Debug` on these view records is also filtered. `Game`, `ObjectStore`, existing
inspection methods and their `Debug` remain **privileged**, not policy output.

The additive [structured policy interface](policy-decisions.md) supplies safe turn,
land and standalone payment candidates without changing this schema or opening API.

This is the atomic core observation contract from #73, not the complete #19
policy integration or M3 numeric tensor encoder. Read it at opening decisions,
completed opening, settled turn boundaries and terminal results. Before reset,
during internal opening work, or during private targeting/casting/payment,
`observe` explicitly returns `ViewError::Unavailable` for either seat. No partial
spell view is advertised; #19 owns that integration, committed target/combat
relationships and later action candidate encodings. A caller must not substitute
privileged inspection when a view is unavailable.

## Fields and ordering

| Field | Visibility and meaning |
| --- | --- |
| `schema_version`, `seat` | Version 1, requested persistent seat 0 or 1. |
| `life`, `hand_counts`, `library_counts`, `mana` | Public, always indexed by persistent seat; mana order W/U/B/R/G/C. Counts expose no card identities. |
| `hand` | Only the requested seat's hand, sorted by frozen card key, stable among indistinguishable duplicates. Hidden library or allocation order cannot reorder these records. |
| `public_zones` | Always graveyard 0, graveyard 1, battlefield, stack, exile, including empty lists. Members retain public zone-entry order; stack is bottom first. No global storage indices. All current public-zone objects are face up; face-down mechanics are unsupported. |
| visible card | Frozen card key, owner/controller seats, tapped, optional `[power,toughness,damage]` for implemented creature characteristics and summoning sickness on the battlefield. `None` does not assert an unsupported card is a noncreature. |
| `remembered` | Ordered historical revelation facts described below. |
| `starting_seat`, `turn`, `acting_seat` | Starting seat; optional `(turn number, active seat, step name)`; current opening/settled-turn actor, absent without a decision. Actor identity does not include the other seat's candidates. |
| `opening` | Present only for the opening decision's actor. Generation, kind, required count and ordered legal candidate rows. Keep/mulligan rows are `keep`, then `mulligan` (only `keep` at the limit). Bottom rows are card keys in the same sorted order as `hand`, including each duplicate. |
| `terminal` | Absent before ending; otherwise winner (null for draw) and per-seat loss reasons `life`, `empty_draw`, or `concession`. Final hands stay seat-filtered. |

Strings are schema tags or frozen card keys, not formatted private diagnostics.
All exposed vectors are dynamically sized; there is no padding, clipping or
fixed tensor-capacity claim. Public zones may legitimately be empty. Schema 1
is structured data, not a throughput measurement. Same information and history
produce equal serialized bytes, including field order, candidate order and
lengths, across independently allocated games. Future draws/outcomes need not
match. Returned records remain snapshots when the game advances.

## Remembered information

Storage records a fact `(card, owner, zone_at_reveal)` for **both** seats whenever
an object enters a public zone, including initial synthetic public placement.
A same-zone no-op records nothing. Rules/test code can explicitly reveal a live
object to one seat with `Game::privileged_reveal_to(seat, handle)`. This is a
privileged event hook, not a player action or a new reveal mechanic. Invalid or
foreign handles fail before mutation. The hook grants no transport permissions.

Facts contain no live handle, library position, current hidden zone, current
controller or hidden movement timestamp. Later hidden moves, shuffles, removal
and slot reuse neither refresh nor erase them. Repeated genuine revelations
append repeated facts; they are events, not a unique-card tracker. A previously
revealed Forest is a historical Forest, not an assertion that it is still in
that library or at any particular index. Unknown hidden draws append no fact.
Own hand history can be retained by a recurrent consumer; this log does not
invent additional revelations for private hand changes. Successful reset clears
both knowledge lists, while failed reset preserves them. Allocation reservation
precedes public event mutation; storage batch preflight also reserves journal
capacity. Events are never silently dropped.

## Opening action boundary and errors

`apply_opening_view(actor, generation, rows)` translates the **current view's**
rows to the existing engine opening choices. A keep/mulligan command has one row;
a bottom command has exactly `count` distinct rows in the desired bottom order.
Rows are not object IDs and cannot address an opponent's hand or either library.
Sorting is translated back internally before normal rules validation. No chance
permutation is accepted by this player-facing method.

The trusted caller must route the correct game and bind its authorized seat.
Visible generations are game-local, increase across decisions/resets, and are
not cross-game authentication tokens. Transport authentication and idempotency
belong to later interfaces. Errors serialize as the context-free literals
`unavailable`, `wrong_actor`, `stale_decision`, `invalid_selection`; they never
include submitted rows, card identities, handles or seeds. Invalid commands
leave the complete game/RNG/history unchanged. Post-opening candidates are not
implemented by this boundary; absence of `opening` is not an empty legal-action
set for a later turn.

Run `cargo test -p mtg-core views` inside the managed toolchain, then the complete
`./scripts/torture.sh`. [Acceptance and red/green evidence](evidence/views/README.md)
map the exact assigned cases and distinguish synthetic from normal-reset tests.

Visible cards expose effective `trample` (true for printed or currently granted
trample, omitted when false). Invoker grants it publicly to both seats on
resolution and cleanup removes the grant. The typed recorder preserves this
additive Boolean; absent legacy fields deserialize as false. New engine
fingerprints distinguish new captures/replays from earlier artifacts.
