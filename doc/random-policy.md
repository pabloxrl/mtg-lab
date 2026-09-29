# Native legal random M1 policy

`mtg-policy::LegalRandom` is a native Rust client of the existing
[seat-authorized policy interface](policy-decisions.md). It receives only an
owned `policy::Observation` and returns a semantic `Submission`. It has no game,
object-store, environment RNG, replay or privileged-state handle. The trusted
runner obtains the current actor's observation and applies the submission with
`Game::apply_policy(actor, &submission, capacity)`.

Construct one policy **per episode and persistent seat**, using
`LegalRandom::new(VERSION, RNG_VERSION, policy_seed, episode_id, seat)`.
`VERSION = legal-random-m1-v1` pins decision sampling and supported content;
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

All current M1 decision families are handled: keep/mulligan, ordered bottoming,
priority, lands and mana, staged target/payment/finish/cancel choices, attacker
subsets, blocker mappings, modern vanilla damage allocation, and cleanup discard.
Supported casts are Bear Cub, Swab Goblin, Giant Growth and Bite Down; land/mana
sources are Forest and Mountain. A newly enabled cast/land/mana candidate outside
that list fails with `UnsupportedContent`, including when pass is also legal.
Unknown decision kinds or enabled unsupported commands fail explicitly.
Malformed masks/cardinality/factored domains produce `InvalidObservation`;
missing decision, wrong seat and incompatible version have separate errors.
These shape checks do not replace the engine's authoritative legality validation.

The core still resets only its fixed 40-card red/green deck configurations. Other
cards remain physically present and can be drawn, bottomed or discarded, but
cannot be cast through the current M1 interface. This policy does **not** make
the full frozen card pool playable. No keywords, activated creature abilities,
triggers, heuristic, production episode runner, CLI selection, trajectory capture,
policy checkpoint serialization or training integration is added. The existing
CLI remains `pass-v1`; do not report passive CLI games as this random policy.

Caller-limited unfinished games are **truncated**, never terminal/drawn games.
Engine/capacity errors are failures, not successful truncations. The test harness
accounts for these boundaries separately and does not add a second rules engine.

Run `cargo test --locked -p mtg-policy` in the managed toolchain container; all
regressions are also discovered by `./scripts/torture.sh` in debug and release.
[Acceptance evidence](evidence/random-policy/README.md) separates synthetic
selection vectors, reachable forced scripts and uninterrupted seeded games.
Original #21 integration/catalog acceptance remains with its owner; M1 is not
complete until #22. M2–M5 requirements remain unchanged.
