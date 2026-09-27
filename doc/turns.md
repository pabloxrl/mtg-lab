# Empty-stack turns and priority

`mtg_core::opening::Game` now continues a completed opening through empty-stack
turns. Types are in `mtg_core::opening::turns`. All state inspection, `Debug`,
object handles and the earlier `draw_top` primitive remain privileged. This is a
rules component, not a playable CLI, full engine, or player observation API.

Complete reset and explicit opening choices as described in [opening.md](opening.md).
Then call `game.start_turns()?` once. It untaps the starting player's permanents
and returns their upkeep priority decision. Calling it before opening completes
or twice returns a structured error unchanged. Existing `decision`, `apply`,
`resume` and `Progress::OpeningComplete` describe the opening subsystem only;
use `turn_decision` / `apply_turn` for play after this explicit boundary.

For a `TurnKind::Priority` decision `d`, submit:

```rust
let action = TurnAction {
    decision: d.id,
    selection: TurnSelection::Pass(d.candidate(0)),
};
let next = game.apply_turn(d.actor, &action)?;
```

The pass candidate remains at index zero. [Land and mana actions](mana.md)
use the same priority decision ID through dedicated validated methods. One pass gives the
opponent priority without drawing, clearing mana or advancing the step. A second
consecutive pass advances the empty-stack step and gives the active player
priority. Both the action and every candidate carry the existing game-scoped,
monotonic decision identity. Wrong actor, stale/reset/foreign decision or
candidate, invalid index, wrong kind and malformed discard sets are rejected
before mutation. Finite generation/turn counters fail explicitly on exhaustion.

`turn_position()` returns `(turn_number, active_seat, step)`; `mana()` returns
both six-element pools in W/U/B/R/G/C order. Turn one belongs to the configured
starter, including P1. Turn/step semantics use pinned CR 103.8a, 106.4, 117 and
500–514:

- Untap happens before upkeep; it untaps only the active player's controlled
  permanents. No untap priority decision is invented.
- The starting player's first **draw step** is skipped entirely. Other draw
  steps draw the ordered top card exactly once, before active-player priority.
- Empty combat visits beginning combat, declare attackers and end combat.
  With no attackers, blocker and damage steps are skipped (CR 508.8).
- Precombat main, postcombat main and end step each offer both priority windows.
- Both mana pools empty at every step/phase boundary, never on the first pass.
- Cleanup offers no ordinary priority. Hands above seven receive a typed
  `TurnKind::Discard { count }` choice. `discard_cards()` supplies current hand
  handles in candidate-index order; submit exactly `count` distinct candidates
  via `TurnSelection::Discard`. The selected order is their graveyard order.
  Otherwise cleanup proceeds directly through next turn's untap to upkeep.

The cleanup discard prefix is necessary for pass-only opening-to-second-turn
acceptance: the nonstarter has eight cards at their first cleanup. [Targeted instants](targets.md) now add simultaneous damage removal and Growth
expiration after discard. Cleanup triggers remain with later abilities and
integration acceptance.

Storage records now retain controller and tapped status. New identities from
allocation/zone changes default to owner control and untapped; same-zone moves
preserve status. Game inspection is immutable; [land plays and mana production](mana.md) now
use validated transitions. [Creature casting](casting.md) joins payments and stack resolution. Synthetic unit tests seed battlefield status and
mana directly and are explicitly distinguished from normal opening tests.

A stack registered by [creature casting](casting.md) or [targeted instants](targets.md) resolves one top spell on
two consecutive passes, then gives the active player priority without advancing
the step or emptying mana. Unregistered raw storage stack objects still return
`UnsupportedStack`. [Vanilla combat](combat.md) now adds explicit attacker/blocker
and damage choices. Other combat permanents still return `UnsupportedCombat`. Attempting a
required draw from an empty library returns `TurnError::Draw(EmptyLibrary)` and
preserves the current decision/state; it is not a terminal result. Terminal
outcomes remain #72. These limitations prevent claiming complete games.

During a target or mana-payment continuation, `turn_decision()` returns None and turn
commands reject until payment finishes or is cancelled. Reset clears turn
state/mana/payment and invalidates old decisions. Turn actions operate
on the fixed 80-card two-deck state: a pass draws at most one card or untaps at
most 80 objects; a cleanup choice discards at most 33 cards. This bounded scalar
turn prefix adds no internal unbounded resolution loop. The existing resumable
opening work contract remains intact; future effects use their own continuations.

Run `cargo test -p mtg-core turns` and `./scripts/torture.sh` inside the managed
container. [Acceptance evidence](evidence/turns/README.md) maps all eight assigned
catalog cases, retained red/green assertions, mutations and the applicable
cached XMage priority smoke. Broader shared RFC blocks and M1 remain incomplete.
