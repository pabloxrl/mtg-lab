# Rules outcomes and reset

`mtg_core::opening::terminal` adds stable seat results for the supported scalar
rules slice. `Game::outcome()` is `None` until a rules ending, then an immutable
`Outcome { winner, losses }`. `losses[0]` is P0 and `losses[1]` is P1, independent
of active player or priority. `winner: None` with both losses present means a
simultaneous rules draw (CR 104.4a). Reasons distinguish life, attempted empty
draw, and concession. This is not a reward ledger; recording remains separate.

`apply_turn` and `finish_combat` now return `TurnProgress::Decision` or
`TurnProgress::Terminal`. Combat checks player life after simultaneous damage
and creature deaths, before granting priority. Nonpositive player life loses;
a dead creature alone does not. Normal draw steps distinguish drawing the last
card from attempting to draw from an empty library. The starting player's first
draw step remains skipped. The low-level `draw_top` primitive settles an empty
draw and returns `DrawError::EmptyLibrary`; subsequent calls return
`AlreadyEnded`. This primitive requires an already-established standalone draw
event; it is not a multi-draw spell implementation.

For concession, retain `game.episode_id()` and call
`game.concede(seat, episode_id)`. Either seat may concede without priority,
including during opening, a yielded reset, targeting, payment, or an unresolved
stack. The opaque episode token rejects commands from another game or a previous
reset. It is a freshness token, not authentication. A repeated concession returns
`AlreadyEnded` and cannot change the result. Transports must authorize seat identity.

Endings remove available decisions and cancel provisional continuations. They
preserve final life, mana, zones, and unresolved stack; no remaining spell is
resolved. Read access and repeated `resume` calls are nonmutating; `resume`
returns `Progress::Terminal`. Old actions cannot commit. Best-of-one endings do
not launch sideboarding, another game, or reset. An explicit valid `reset` starts
a new episode, clears results, mana, damage, effects, stack and continuations,
and invalidates old handles and commands. Malformed reset leaves the entire
previous state and RNG unchanged.

External decision/turn/wall-time budgets are collector truncations, never
invented rules draws. No such budget enters this rules-result API. Internal work
quantum exhaustion remains `InternalYield`; numeric/storage failures remain
errors. Callers must account for external truncations and failures separately.

Run `cargo test -p mtg-core terminal` and `./scripts/torture.sh` inside the managed
container. [Acceptance evidence](evidence/terminal/README.md) distinguishes
normal-reset complete scripts, synthetic edge positions, and matched XMage
boundary observations. This does not implement recorder rewards, policy views,
CLI matches, unsupported cards/keywords, multi-draw effects, or all M1 integration.


## Synthetic reference contract

`fixtures/reference/terminal.json` remains version 1: draws explicitly retain the
historical P0 interpretation, concessions P1, and observations retain life/lost/
library/hand only. `fixtures/reference/terminal-v2.json` requires `draw_seat` or
`concede_seat` for those actions and specifies life-seat and draw/life injection
order. Unknown fields, invalid seats and conflicting actions fail visibly.
Version 2 observations carry `version: 2`, `pending`, `settled` and `consumed`;
pending/settled contain the four legacy fields plus `outcome` (`ongoing`, `win`,
`draw`), nullable `winner` (0 or 1), and a Boolean `draw`. Old artifacts cannot be
read as this richer contract.

The pending checkpoint follows the synthetic setters and attempted draw but
precedes the single SBA settlement. Concessions use a separate immediate action
after that checkpoint. Native draws retain actual empty-library failure until
the existing terminal collector; XMage performs real drawCards and SBA processing.
The final XMage winner/draw is observed after its game loop finalizes the result.
Neither adapter derives outcomes from fixture expectations. These tests do not
claim that the injected states are reachable through ordinary games.

Inside the managed container, with the issue's prepared pinned cache:

```sh
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- \
  python3 scripts/terminal_reference.py --cache "$MTG_REFERENCE_CACHE" \
  --output /tmp/terminal-reference.json
```

The runner now writes a version-2 receipt with a hashed artifact directory, instead
of the old unversioned receipt's inline `checkpoints`. Historical receipts remain
unchanged. This runs all seven legacy and 31 version-2 cases twice in native Rust and XMage,
retaining raw inputs, choices, observations, hashes, strict rejection and actual
wrong-seat/premature-settlement controls in the receipt's sibling directory.
See [the scoped evidence](evidence/mixed-terminal/README.md). Production outcome,
snapshot and replay versions are unchanged by this test-only adapter contract.
