# Played activation oracle (version 5)

These are independently authored fixed schedules, not sampled policy output or
an alternative rules engine. `author_activations.py` reads the unchanged frozen
40-card deck multisets; it never reads native or XMage results. Both consumers
start through the delivered normal-reset/London interfaces, keep seven, use the
specified complete shuffle permutations and play every land and creature.

Rules authority: unchanged `data/rules/cr-2026-09-25.json`, exact frozen card
content hashes in `data/cards/foundations_micro_v1.json`, GR-030 and the delivered
[#254 payment contract](https://github.com/pabloxrl/mtg-lab/issues/254).
CR 602.2b applies announcement, target selection and cost payment; CR 605 makes
mana abilities immediate, without a stack object or response window. CR 302.6
forbids a sick creature's tap activation; CR 702.10 permits it with haste.
CR 400.7 separates zone incarnations; CR 113.7a gives each activation its own
stack object. CR 611.2 governs the stated until-end-of-turn effects.

Literal Oracle expectations:

- Llanowar Elves costs G, is 1/1, and taps for G. Druid costs 1G, is 1/3,
  and taps for G. Neither newly cast creature can tap before haste.
- Cavalry costs 1R, is 2/2 and taps to give a target creature haste until end
  of turn. Its target may be an opponent's creature. Two normally played
  Cavalries grant haste to green's newly cast Elf and Druid; both then tap
  immediately, leaving GG and no new stack object or priority handoff.
- Shivan costs 4RR, is 5/5, and each R activation gives +1/+0 until end of
  turn. One resolves to 6/5. Two independent stacked activations resolve to
  7/5 and must have distinct action and raw stack identities. Floating RR
  before one activation leaves R; empty-pool payment explicitly taps Mountain.
- Invoker costs 2GG, is 4/3; its eight-mana activation first targets Cub,
  then pays using six Forests, Elf and Druid. Cub resolves from 2/2 to 7/7
  with trample. Empty and floating-first schedules use the same eight sources.
  Five Forests plus the two creatures provide seven and cannot activate it.

Every pass and empty attack declaration is specified. The initial player skips
its whole first draw step; each other turn draws exactly once. Final zone
ledgers count those draws, land plays and creature casts independently. No
noncreature casting, triggered choices, nonempty combat or final cleanup is used.

## Raw representations and cancellation

Native #254 intentionally reserves target, source taps and mana privately;
public state changes atomically at `FinishActivation`. The checker asserts each
stage's target, source incarnations, reserved colors and remaining private pool,
and unchanged public state. Every native continuation boundary cancels and
retries the same original activation, including fully paid but uncommitted.
The actual scalar semantic records and player recorder capture are retained.

Pinned XMage `PlayerImpl.playAbility` pushes a fresh `StackAbility` before calling
`Ability.activate`, then invokes real target/payment callbacks. It restores its
bookmark on cancellation. Its raw announcement stack, live mana/taps, unpaid
cost, targets and callback source identities are retained and checked separately.
Each actual target/payment callback is canceled in a separate played run. There
is no invented XMage callback for native's paid-but-uncommitted continuation;
those additional native cancellation stages are separately identified.

Committed states compare full ordered libraries, stacks and selected targets,
life, mana, actor/turn/step, incarnations and permanent characteristics. Hands and
battlefields compare membership as in the delivered priority/spell families.
Native ability-object birth IDs and reference raw UUIDs remain privileged and
must map one-to-one to activation action IDs, including repeated identical
activations. Physical cards retain the witnessed opening/zone-transition mapping.

Expiry evidence remains before cleanup: native's actual turn-scoped modification,
haste and trample storage is checked against literal effects; reference actual
continuous effects expose `EndOfTurn`, their raw IDs and source ability links.
This does not claim execution of the later cleanup-expiry boundary.

Invalid source/controller/sickness/incarnation, target, actor, color, payment
order, missing mana, unpaid commitment and insufficient-eight controls must fail
at the specified input cursor and category. Missing fields/observations, swapped
same-name sources, duplicate stack actions, incorrect stats/surplus/keywords
and target mutations cannot count as agreement. Reference selected playable
activation membership and actual callback acceptance are distinct from complete
legal-set equality; the latter is not claimed.
