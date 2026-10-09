# Deterministic native heuristic

`mtg_policy::Heuristic::new(HEURISTIC_VERSION, seat)` creates a stateless native
opponent. `HEURISTIC_VERSION = heuristic-activation-mana-v1` freezes the rules below. Its
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
  first, without replacement: all other cards 0, lands 1, Growth 2, Bite 3,
  Bear Cub/Swab Goblin 4. This intentionally simple policy may discard needed lands.
- Priority: play a land, then cast a creature or Dragon Fodder, then Bite if own and enemy creatures
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
  (Forest/Elf/Druid=green, Mountain=red), then candidate order; cancel only when no legal
  progress exists.
- Attack with every legal attacker. Replace a different provisional selection,
  then explicitly finish once it equals the desired selection.
- Block greedily in blocker domain order. Each blocker chooses the unassigned
  legal attacker with greatest power, then remaining toughness, then domain order.
  Assign at most one blocker per attacker; omit surplus blockers. Replace a
  different provisional mapping, then explicitly finish.
- Allocate the first unassigned attacker's damage in blocker domain order:
  give each except the last up to its remaining toughness, then give the last
  all remaining power. Include zero amounts. Finish after all allocations.
  For trample use the lethal-requirement rule below; no obsolete blocker-order choice is introduced.

## Supported behavior and limits

Handles every delivered continuation and all twenty frozen cards, with the same
strict content/shape/version boundary as [LegalRandom](random-policy.md).
Archer, Cyclops and Pyromancer use the existing creature cast score of 30.
Trigger ordering preserves the candidate table's bottom-to-top order, selecting
every legal row once. Pyromancer targets the opponent when legal, with candidate
order breaking ties. The previous `heuristic-surprise-v1` ID rejects explicitly.
These deterministic preferences are not a policy-strength claim.

[Full-pool acceptance](evidence/full-pool-policy/README.md) covers both starting
seats and red/green/mirror matchups through Driver, replay and canonical capture.
The engine remains responsible for legality and stale submission rejection.

The policy does not predict opponent hands, search future states or optimize
winning chances. It may waste spells or make poor attacks. Strength is not a
rules oracle. The existing owned runner, native CLI and capture use this policy;
training integration and policy probabilities remain separate obligations. External limits are
truncations, never terminal draws; engine errors are failures.

Run `cargo test --locked -p mtg-policy`. Normal discovery and full torture cover
both native policies. Historical M1 integration and gate evidence remain with #21/#22; later milestone
obligations are unchanged.

GH-194 extends the accepted content domain to Dragon Fodder at the existing
vanilla-development score (30). Its legal payment/priority choices use the same
rules, and token combat uses observed 1/1 characteristics. Existing six-card
input scores and tie rules are unchanged; the engine fingerprint identifies the
expanded legal domain. Full-pool strategy and support remain #208 acceptance.

`heuristic-mana-v1` supersedes `heuristic-tokens-v1` and `heuristic-m1-v1`; both old IDs reject explicitly. GH-195 adds Elf/Druid casts at the existing creature score (30) and tap-for-G payment sources. It changes no other score or tie rule. Full-pool qualification remains #208.

GH-196: `heuristic-reach-v1` supersedes `heuristic-mana-v1` (rejected). Sentry
casts score 30. Block selection excludes observed `forbidden_blocks` pairs before
choosing the best remaining attacker; reach itself gives attackers no evasion.

GH-197 added Cavalry casting and staged targeted tap-cost activation. Cavalry uses creature cast score 30; starting activation scores 5, finishing scores 30 and cancellation 0. Targets use the existing own-creature power/toughness ordering. Full-pool policy qualification remains #208.

GH-198 adds Tajuru Pathwarden (4G 5/4 vigilance/trample); current `heuristic-activation-mana-v1` rejects prior policy IDs. For trample, assign up to each blocker’s observed lethal requirement in domain order and send remaining power to the defender. Other allocation behavior is unchanged.

GH-199 adds pinned Thornweald Archer (1G 2/1 reach/deathtouch) to the supported casts. Policy IDs now end in `activation-mana-v1`; older IDs reject. Trample allocation consumes the same observed lethal domain, whose entries are one for deathtouch sources. The heuristic gives Thornweald the existing creature score; full-pool policy evidence is linked above; M2 qualification remains pending.

GH-200 adds Shivan Dragon casting and the nontargeted `activation_payment` continuation. Current IDs end in `activation-mana-v1`; the prior `deathtouch-v1` IDs reject. Explicit red payment and finish/cancel use the existing candidate sampling/scoring. Choose eligible mana sources during payment, or use already floated mana. Full-pool policy evidence is linked above; M2 qualification remains pending.

GH-201 enables Wildheart Invoker casting and targeted activation. Current IDs end
in `activation-mana-v1`; previous IDs reject. Select a creature target, then reserve eight
units through `activation_payment`, choosing land/Elf/Druid mana abilities there
or using floated mana. Finish commits all eight
atomically; cancellation spends nothing. Full-pool qualification remains #208.

GH-202 adds Thrill of Possibility and `cast_discard`: choose one other hand card
or cancel payment. Discard selection stays private until the cast commits;
then the existing explicit mana choices finish payment. Current IDs end in
`activation-mana-v1`; prior IDs reject. The heuristic scores Thrill as 30 and selects a
lowest-retention discard; random samples the legal candidates. Full-pool
qualification remains with #208.

GH-203 adds Goblin Surprise and the explicit `cast_mode` continuation. Mode 0
boosts creatures controlled at resolution; mode 1 creates two Goblins. Current
IDs end in `activation-mana-v1`; prior IDs reject. Legal-random samples both modes and
cancellation. The heuristic scores the cast as 30 and prefers token mode (2)
over boost mode (1), with cancellation retaining its existing lower score.
This small deterministic preference is not a strength or full-pool qualification.

GH-254 extends payment-stage TapMana choices using the existing sampling/scoring.
The heuristic pays available mana before generating more, prefers a required
color when choosing sources, and finishes paid activations before extra taps.
The random policy retains every eligible source choice, including surplus mana.
See the [payment repair and version inventory](evidence/activation-mana/README.md).
