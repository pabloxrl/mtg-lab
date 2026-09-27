# Bounded priority and spell settlement

`Game::apply_turn_quantum(actor, &TurnAction, NonZeroUsize)` accepts the same
commands as `apply_turn`. It returns `Progress::InternalYield`,
`Progress::TurnDecision`, or `Progress::Terminal`. On an internal yield, call
`Game::resume(quantum)` until a boundary is returned. Budgets may change between
resumes. `apply_turn` drains this same continuation with an unlimited budget.
Repeated resumes at a settled turn or terminal boundary are read-only.

The bounded portion covers first priority passes and consecutive passes resolving
a supported creature, Giant Growth or Bite Down. Validation, target revalidation,
numeric bounds and storage reservation occur before accepting the command.
Rejected commands preserve rules state, identities, decision generation and RNG.
Acceptance increments the decision generation once. Payment has already committed
at casting and is never replayed during resolution.

Each work unit performs one modification, one zone move, stack bookkeeping, or
terminal/priority publication. A creature takes three units; a legal Growth or
nonlethal Bite takes four; lethal Bite takes five; an effect without a change
takes three. A first pass takes one. Bookkeeping scans the supported M1 population
(two 40-card decks, no token creation). Validation/reservation and these fixed-pool
scans are not wall-clock or allocation-free guarantees. There is no scheduler or
batch runner in this component.

Targets and source power are revalidated at resolution (CR 608.2b/h). Growth adds
+3/+3; Bite uses current legal source power and produces no damage if either
required target is illegal. Zone moves invalidate old handles (CR 400.7). Lethal
damage moves the creature to its owner's graveyard before priority (CR 704.5g).
After the stack item finishes, simultaneous life-loss conditions are checked and
otherwise the active player receives priority (CR 117.3b). These M1 spells do not
themselves alter player life.

Internal work exposes neither an opening/turn decision nor a seat observation.
Turn, cast, mana, combat, opening, draw, reset and concession commands cannot
interrupt pending spell settlement. Concession returns
`ConcedeError::SettlementPending`; submit it after settlement. Existing concession
at opening work and private choice boundaries remains supported. Privileged raw
inspection/debug/snapshot access can see internal states and must not be used as
policy input. Internal yields carry no rewards or terminal results.

Compatible-engine snapshots serialize every pending work item and restore with
fresh capability scopes. Old engine fingerprints return
`RestoreError::IncompatibleEngine` before replacing the live game; no migration is
provided. [Snapshot integrity/trust boundaries](snapshot.md) remain applicable.

Second passes on an empty stack and cleanup discards still execute turn work
synchronously even through `apply_turn_quantum`; turn/cleanup settlement remains
with #109. [Combat damage settlement](combat.md#bounded-damage-settlement) now uses
the same owned work through `finish_combat_quantum`. Complete policy decisions, played-game replay, reference effect
bridges and integration acceptance remain with their respective owners. This is
not M1 completion; gate #22 is still required.

[Executable acceptance and limitations](evidence/spell-settlement/README.md).
