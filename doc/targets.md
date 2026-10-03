# Targeted instants

`mtg_core::opening::targets` adds Giant Growth (G) and Bite Down (1G) to
`Game::cast_candidates`. These instants are available at either player's priority,
including with a nonempty stack, when their required targets and mana exist.
The creature casting API retains sorcery timing. No other spells are enabled.
All inspection, handles, `Debug`, `creature_state`, `stack_targets` and
`last_resolution` remain privileged; seat-filtered observations belong to GH-73.

1. Call `begin_targeted_cast(actor, decision_id, spell, capacity)` at priority.
   It returns a private `TargetDecision`: Growth chooses one creature; Bite
   chooses a creature you control, then a creature you don't control.
2. `choose_target(actor, target_decision_id, handle)` selects one offered handle
   and returns a fresh decision. `TargetKind::Complete` has no more choices.
   Each generation and live target is validated. No targets are inferred.
3. `finish_targets(actor, id)` requires the complete, still-legal selection and
   opens the existing [payment continuation](casting.md). Stage basic-land mana
   abilities and choose each payment unit, then `finish_cast` commits targets,
   payment, taps and the spell's new stack identity together.
4. The caster retains priority. Two consecutive passes resolve only the top
   spell, then the active player gets priority. Casting or a mana ability resets
   previous passes; target/payment micro-choices introduce no response window.

`cancel_targets` or, after target completion, `cancel_payment` abandons the
uncommitted cast. Invalid commands retain the full state and continuation.
The opponent receives neither target nor payment choices during the cast.
`begin_cast` cannot bypass required targets; use it for the two vanilla creatures.
Reset clears targets, pending casts, stack effects, temporary modifiers and damage.

At resolution, exact generation-tagged battlefield objects and controller
restrictions are checked again (CR 400.7, 608.2b). Growth grants +3/+3 until end
of turn and multiple Growth effects add. Bite uses its legal source's current
power (CR 608.2h) to damage the legal destination. If either target is illegal,
Bite deals no damage: an illegal targeted source supplies no last-known power.
With at least one legal target it resolves as far as possible; with no legal
targets it does not resolve. Both outcomes move the spell to its owner's
graveyard. `last_resolution` records this distinction for the most recent targeted instant
for privileged diagnostics.
There is no retargeting decision, damage to a replacement identity, or redirection.

Lethal marked damage moves a creature to its owner's graveyard before the next
priority decision (CR 704.5g). Its new identity has no inherited damage or boost.
Cleanup first performs any discard, then removes all marked damage and expires
all until-end-of-turn bonuses simultaneously (CR 514.2). There is no intermediate
lethal-damage check between those two changes. Cleanup triggers and repeated
cleanup processing for future abilities remain outside this card slice.

## Capacity and boundaries

Targets are factored into at most two decisions: n own and m opposing creatures
represent all n×m Bite pairs using lists of n and m, without a flat tuple limit.
`capacity` limits each complete candidate list, including the later destination
list. An insufficient limit returns `CapacityExceeded { needed, capacity }`
before any state changes, never a partial successful decision. Treat this as a
failed sample pending an adequate encoding, not a legal action to silently omit.
For the M1 no-token slice, at most 80 card objects exist in a normal two-deck
reset, so capacity 80 is a conservative upper bound for every target role;
actual Cub/Goblin counts are smaller. This is not the future M2 token/tensor bound.
Vectors also handle declared synthetic positions above normal deck counts.
Arithmetic exhaustion reports `TurnError::EffectOverflow` before resolution
mutation. No performance or allocation-free claim is made.

Supported cast creatures are Bear Cub and Swab Goblin. A synthetic Sentry's
frozen 4/4 characteristics are recognized solely to exercise the assigned
no-source-LKI catalog case; its casting, Reach and other M2 mechanics are not
implemented. Unsupported creature definitions are not target candidates.
Vanilla [combat](combat.md) and [terminal outcomes](terminal.md) are implemented.
The [scalar integration audit](evidence/scalar-integration/README.md) covers native
normal-reset games and matched synthetic Growth/Bite scenarios; full-pool and
full-game reference qualification remain outstanding.

Run `cargo test -p mtg-core targets` and `./scripts/torture.sh` in the managed
container. [Acceptance evidence](evidence/targets/README.md) maps the exact
catalog cases and separates normal reset scripts from synthetic positions.

## Thornweald deathtouch

[GH-199 acceptance](evidence/deathtouch/README.md) adds pinned 1G 2/1 Thornweald Archer with reach and deathtouch. Any positive damage it deals to a creature is lethal (CR 702.2/704.5h), including Bite damage. Zero damage does not kill. Combat remains simultaneous; for trample, one damage is lethal for a deathtouch source, but deathtouch alone never permits player damage through a blocker. Existing scalar work, factored choices, semantic actions and typed capture carry this behavior.
