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
