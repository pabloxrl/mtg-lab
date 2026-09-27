# Vanilla combat

`mtg_core::opening::combat` supports Bear Cub and Swab Goblin, using the existing
opening, turn, casting and cleanup APIs. All inspection remains privileged;
seat-filtered observations are separate work. Other combat creatures and keywords
are explicitly unsupported. Game outcomes and complete-game acceptance remain
with GH-72/GH-18; negative life does not yet terminate a game.

`turn_decision()` now also returns `TurnKind::Combat(Attackers | Blockers | Damage)`.
Pass and spell/mana commands reject during these continuations. Each accepted
selection advances the decision ID. Wrong-seat, stale, foreign, duplicate and
illegal commands leave the full state unchanged. `Game::life()` now returns
`[i64; 2]` so simultaneous damage can take life below zero without clamping.

1. Pass priority through beginning combat. With vanilla creatures on the active
   battlefield, the next decision belongs to the attacking player. Inspect
   `combat_decision(actor, capacity)` for eligible untapped, nonsick attackers.
   `select_attackers(actor, id, &[handles])` stages a subset. Replace it to
   backtrack, then call `finish_combat(actor, id)` to commit and tap attackers.
   Finishing the empty selection explicitly declares no attackers. A battlefield
   with no active creatures automatically declares none, retaining the existing
   empty-combat turn progression.
2. Both players get priority after attackers are declared. If no attackers were
   declared, skip blockers/damage (CR 508.8). Otherwise the defender chooses
   blockers through `select_blockers(actor, id, &[(blocker, attacker)])` and
   `finish_combat`. Untapped creatures may block even when summoning sick.
   Each blocker can block one attacker; multiple blockers may block the same
   attacker. Blocking does not tap. Provisional subsets/maps are available only
   from the choosing actor's combat inspection; public `combat()` shows only
   committed declarations.
3. Both players get priority after blocks. The combat-damage continuation belongs
   to the active player. Its `damage` rows list attackers with multiple remaining
   blockers, their current power, and legal recipients. For each such attacker,
   call `assign_combat_damage(actor, id, attacker, &[(blocker, amount)])`.
   The amounts must total its power; omitted recipients receive zero. Assignments
   can be replaced. There is **no blocker order or lethal-first requirement**:
   a 2/2 can assign one damage to each of two 2/2 blockers. Missing assignments
   reject completion. Single-recipient assignments and each blocker's damage
   are forced by the rules and computed at completion.
4. `finish_combat` deals all damage simultaneously, moves all lethally damaged
   creatures to their owners' graveyards, then grants active-player priority.
   No priority occurs between allocation and damage/SBAs. A blocked vanilla
   attacker remains blocked if its blockers leave, and deals no player damage.
   Departed objects and new zone identities cannot deal or receive combat damage.
5. Passing through end combat clears combat membership. Existing cleanup removes
   marked damage and expires temporary boosts together. The next controller
   turn untaps creatures and clears summoning sickness normally.

Declarations use lists of objects, not a flat enumeration of subsets or maps;
allocation is factored per attacker. The encoder chooses its capacity explicitly.
`combat_decision` rejects an insufficient capacity without truncation. For the
M1 pool there are at most 80 cards across both decks and no created tokens:
80 bounds each role/recipient list, while declarations hold at most one entry per
creature and block maps at most one per blocker. Future token/keyword encoders
need their own bounds. No fixed tensor ABI is claimed here. Numeric overflow
rejects before life, damage, deaths or decision state changes.

Run `cargo test -p mtg-core combat` and `./scripts/torture.sh` inside the managed
container. [Acceptance and catalog mapping](evidence/combat/README.md) distinguish
synthetic positions, real reset-to-combat scripts, and reference coverage limits.
