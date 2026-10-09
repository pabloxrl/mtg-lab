# Native legal random policy

`mtg-policy::LegalRandom` is a native Rust client of the existing
[seat-authorized policy interface](policy-decisions.md). It receives only an
owned `policy::Observation` and returns a semantic `Submission`. It has no game,
object-store, environment RNG, replay or privileged-state handle. The trusted
runner obtains the current actor's observation and applies the submission with
`Game::apply_policy(actor, &submission, capacity)`.

Construct one policy **per episode and persistent seat**, using
`LegalRandom::new(VERSION, RNG_VERSION, policy_seed, episode_id, seat)`.
`VERSION = legal-random-activation-mana-v1` pins decision sampling and supported content;
`RNG_VERSION = legal-random-rng-v1` separately pins the underlying
`splitmix64-v1` seed derivation, policy-seat domains and bounded sampling. Both
versions must match exactly. Changing either contract requires a version change.
Stable episode IDs and separate owned seat streams make evaluation order between
seats/episodes irrelevant. The policy seed may differ from the environment seed;
even identical seeds use distinct RNG domains and independent consumption.
No policy seeds or raw RNG state are placed in observations.

```rust
use mtg_policy::{LegalRandom, VERSION, RNG_VERSION};
// Trusted runner initializes these once for episode 7:
let mut policies = [0, 1].map(|seat|
    LegalRandom::new(VERSION, RNG_VERSION, 42, 7, seat).unwrap());
// At each current decision:
// let observation = game.policy_observe(actor, capacity)?;
// let action = policies[observation.view.seat as usize].choose(&observation)?;
// game.apply_policy(actor, &action, capacity)?;
```

The caller owns opening-complete `start_turns`, terminal handling, resource
capacity, external decision/work limits, episode accounting, retries and resets.
Every evaluation consumes policy randomness, even when re-evaluating the same
observation. A repeated evaluation is a new sample. The policy cannot determine
whether an otherwise well-formed observation is still current: only the engine
can reject stale revisions/generations and wrong-seat/illegal submissions.
Never substitute pass or a human choice after an error.

## Exact sampling contract

All bounded draws use rejection-modulo: for positive `n`, discard words below
`2^64 mod n`, then return `word mod n`. Each residue has exactly equal mass;
modulo without rejection is not this algorithm. Even a bound of one consumes a
word. Candidate and domain order are the observation's order.

| Decision | Distribution and consumption |
| --- | --- |
| Keep/mulligan, priority, targets, payment | Uniform among unmasked rows, one draw. Pass, cancellation, finish and each individual card row remain eligible. Identical cards in different rows retain separate mass. |
| Ordered bottoming, cleanup discard | Repeated uniform draw among remaining unmasked rows; remove the selected row preserving order. Exactly `count` draws without replacement. Bottom permutations are uniform; discard sets are uniform although the returned order is sampled too. |
| Attackers | Fair coin: 1 finishes the current provisional selection; 0 replaces it, then one fair independent inclusion coin per legal attacker. Every subset has probability `1/2^a` conditional on replacement. Empty is explicit. |
| Blockers | Fair coin: 1 finishes; 0 replaces, then one independent uniform draw per blocker over unassigned (0) and the ordered `a` attackers (1 through `a`). Every mapping has probability `1/(a+1)^b` conditional on replacement. |
| Combat damage | First unassigned attacker in domain order; for each blocker except the last draw uniformly from zero through remaining power, then give the last the remainder. Emit every recipient, including zero. After all attackers have allocations, finish explicitly. |

This is a **factored command distribution**, not uniform over whole turns,
spells, final combat declarations or every integer composition. For example,
the initial empty attacker selection can be finished immediately; conditional
replacement is uniform, but final empty declarations have greater mass. For
power two over three blockers, `(2,0,0)` has probability 1/3 while `(0,0,2)` has
probability 1/9. All legal nonnegative divisions remain reachable. No exponential
subset/map/composition table is created; damage sampling is linear in recipients,
not power. The policy has no tactical scoring. Cancellation, backtracking and
passes may delay progress; the caller must impose and account for external limits.

## Supported content and boundaries

All delivered decision families are handled: opening, ordered bottoming, priority,
lands/mana, staged target/payment/finish/cancel choices, modal selection, casting
and cleanup discard, nonmana activations, trigger ordering/player targets, and
factored combat. All twenty frozen cards and Goblin token combat are supported.
A newly enabled cast/land/mana/activation candidate outside the explicit supported
content list fails with `UnsupportedContent`, even when pass is legal. Unknown
kinds or enabled unsupported commands fail explicitly. Malformed masks/cardinality
produce `InvalidObservation`; missing decision, wrong seat and incompatible version
have separate errors. Trigger orders must select every legal row exactly once.
These checks do not replace the engine's authoritative legality validation.

Trigger ordering samples a uniform permutation without replacement using exactly
`count` bounded draws. Player targets sample uniformly among legal rows, including
the controller: this policy can damage itself. Existing SplitMix64 seed domains,
RNG version, bounded sampling and all other decision distributions are unchanged.
The new policy ID rejects the previous `legal-random-surprise-v1` ID explicitly.

The core resets the frozen 40-card red/green decks; arbitrary decks/cards remain
unsupported. The existing Driver, native CLI and canonical capture consume these
choices; no other game loop or collector is introduced. See [simulation](simulate.md)
and [full-pool acceptance](evidence/full-pool-policy/README.md). Passive `pass-v1`
remains a separate baseline. No playing-strength or M2 gate claim is implied.

Caller-limited unfinished games are **truncated**, never terminal/drawn games.
Engine/capacity errors are failures, not successful truncations. The test harness
accounts for these boundaries separately and does not add a second rules engine.

Run `cargo test --locked -p mtg-policy` in the managed toolchain container; all
regressions are also discovered by `./scripts/torture.sh` in debug and release.
[Acceptance evidence](evidence/random-policy/README.md) separates synthetic
selection vectors, reachable forced scripts and uninterrupted seeded games.
The historical M1 audit is linked from the root README. Original component and
M2–M5 acceptance remain with their owners.

The token-capable version supersedes `legal-random-m1-v1`; the old ID rejects
explicitly. Sampling rules are unchanged, but the legal content domain expanded.

GH-195 adds Llanowar Elves and Druid of the Cowl casts and tap-for-G sources to
the accepted domain. Sampling and RNG rules remain unchanged.
`legal-random-mana-v1` supersedes `legal-random-tokens-v1`; the old policy ID
rejects explicitly. Full-pool qualification remains #208.

GH-196: `legal-random-reach-v1` supersedes `legal-random-mana-v1` (rejected).
Each blocker samples uniformly among no block and its legal attacker rows,
excluding `forbidden_blocks`. Sentry casts join the supported content domain.

GH-197 added Cavalry casting and staged targeted tap-cost activation. Random sampling includes all legal activation targets, finish and cancellation. Full-pool policy qualification remains #208.

GH-198 adds Tajuru Pathwarden (4G 5/4 vigilance/trample); current `legal-random-activation-mana-v1` rejects prior policy IDs. For trample with enough power for all lethal requirements, flip a fair coin: either use the existing blocker-only distribution, or reserve lethal for every blocker and sample each blocker’s additional amount uniformly from zero through remaining excess, leaving the rest for the defender. Every legal split remains reachable; this is not a uniform distribution over splits. Insufficient power uses blocker-only allocation.

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
