# Deterministic M1 heuristic

`mtg_policy::Heuristic::new(HEURISTIC_VERSION, seat)` creates a stateless native
opponent. `HEURISTIC_VERSION = heuristic-m1-v1` freezes the rules below. Its
`choose(&policy::Observation)` returns a semantic submission for
`Game::apply_policy`. Only the authorized observation is accepted; no game,
private replay, environment seed, RNG or clock is available. Repeated inputs
produce identical outputs, independent of call order. The caller owns routing,
reset/start_turns, limits and terminal accounting. Errors must propagate.

## Fixed decision rules

Only unmasked candidates participate. Ties retain input table/domain order;
there is no random tie breaking. Scores are lexicographic, so priority classes
always dominate card values. These are strategic preferences, not rules claims.

- Keep seven (never voluntarily mulligan). Bottom/discard lowest retention value
  first, without replacement: unsupported M1 cards 0, lands 1, Growth 2, Bite 3,
  Bear Cub/Swab Goblin 4. This intentionally simple policy may discard needed lands.
- Priority: play a land, then cast a creature, then Bite if own and enemy creatures
  exist, then Growth if an own creature exists and either combat or a committed
  stack is nonempty, then pass. Do not float mana outside a payment. Ineligible
  tactical spells and priority mana actions rank below pass.
- Targets: Growth and Bite source prefer own creatures of greatest power, then
  greatest remaining toughness (toughness minus marked damage, floored at zero).
  Bite destination prefers an enemy killable by the provisional source's current
  power, then greatest power and remaining toughness. Cancel instead of choosing
  a wrong-controller target. Finish complete targets before canceling.
- Payment: finish when legal, otherwise pay before activating a source. Payment
  color ties use candidate order. Source activation prefers a color still owed
  (Forest=green, Mountain=red), then candidate order; cancel only when no legal
  progress exists.
- Attack with every legal attacker. Replace a different provisional selection,
  then explicitly finish once it equals the desired selection.
- Block greedily in blocker domain order. Each blocker chooses the unassigned
  attacker with greatest power, then remaining toughness, then domain order.
  Assign at most one blocker per attacker; omit surplus blockers. Replace a
  different provisional mapping, then explicitly finish.
- Allocate the first unassigned attacker's damage in blocker domain order:
  give each except the last up to its remaining toughness, then give the last
  all remaining power. Include zero amounts. Finish after all allocations.
  This is modern vanilla allocation, with no obsolete blocker-order choice.

## Supported behavior and limits

Handles every M1 family: opening/bottom, priority, lands, targets, payment,
attackers, blockers, damage allocation and cleanup discard. Casts are restricted
to Bear Cub, Swab Goblin, Giant Growth and Bite Down; mana/lands to Forest and
Mountain. Unknown enabled commands, decision kinds or newly enabled card content
fail explicitly even when pass is legal. Schema/seat/shape errors are explicit.
The engine remains responsible for legality and stale submission rejection.
Other fixed-deck cards can be held, bottomed and discarded but remain uncastable.
This does not support the complete twenty-card pool, keywords or triggers.

The policy does not predict opponent hands, search future states or optimize
winning chances. It may waste spells or make poor attacks. Strength is not a
rules oracle. No CLI selection, collector, capture, training integration or
policy probabilities are supplied; CLI remains pass-v1. External limits are
truncations, never terminal draws; engine errors are failures.

Run `cargo test --locked -p mtg-policy`. Normal discovery and full torture cover
both native policies. Original integration acceptance remains with #21, M1
completion with #22, and later milestone obligations are unchanged.
