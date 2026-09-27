# Safe combat and cleanup policy decisions (GH-112)

Partial R0002-B016/B025/B033 delivery, integration owner #19. No full-game
policy, collector, replay extension or M1 completion claim; #22 and M2–M5
obligations remain. The core combat/cleanup rules are unchanged.

## Independent expectations and executable coverage

Original tests use pinned CR 302.6, 508, 509, modern 510, 514.1/2 and 704.5g;
frozen Bear Cub/Swab Goblin are 2/2. RFC B016/B033 supplies privacy, factored
representation, explicit capacity and unchanged-state rejection requirements.
All tests are in normal Cargo discovery in
[policy_combat_tests.rs](../../../crates/mtg-core/src/policy_combat_tests.rs),
loaded by the existing policy tests module.

- `policy_combat_normal_reset_cast_attack_block_damage_cleanup_both_seats`:
  valid ordered 40-card green mirrors, seven-card keeps, two land plays and two
  individually paid creature casts per seat, summoning sickness removal, one
  attacker against two blockers, explicit 1+1 damage, priority and cleanup.
  Every player action uses the policy interface. Both starting seats execute the
  script. Only the attacker dies; life stays 20/20; cleanup clears marks and combat.
- `policy_cleanup_normal_reset_both_seats_and_distinct_sorted_rows`: no land
  plays, nonstarting player's eighth-card draw reaches mandatory one-card discard,
  with seven remaining and one graveyard card. Both seat assignments covered.
- `policy_combat_independent_subsets_maps_and_allocations`: explicit synthetic
  two-attacker/two-blocker board; independently loops 2^2 attacker subsets,
  3^2 blocker maps (none/A/B independently for each blocker), and all three
  compositions of power 2 over two blockers. Expected committed relationships,
  tapping, life and death counts derive from the chosen input and CR 508–510.
- `policy_cleanup_independent_pairs_sorted_identity_and_hidden_twins`:
  synthetic nine-card End-step hand; all C(9,2)=36 pairs leave seven cards and
  move exactly the two selected identities. Hand includes different cards so
  sorted policy order differs from privileged zone order. Duplicate/accepted
  repeated commands do not discard extra cards.
- `policy_combat_twins_private_backtracking_errors_and_capacity` and
  `policy_combat_every_boundary_hidden_twins_masks_and_atomic_errors`:
  both-seat hidden hand/library twins, byte-equivalent authorized ordering/masks,
  actor-only provisional state and replacement/backtracking; exact/one-below
  capacity boundaries, wrong actor, stale, wrong-kind and wrong-zone rows.
  Full Game debug state (including RNG/history) is equal after every rejection.
  Synthetic generation exhaustion explicitly returns capacity failure.
- `policy_combat_invalid_block_damage_and_finish_preserve_state`: duplicate
  attackers/blockers/recipients, illegal domains, wrong sums, unsigned maximum
  amounts, missing required allocations and invalid finish leave state unchanged.
- `policy_combat_storage_twins_and_departed_blockers_keep_public_identity`:
  synthetic private allocation/identity permutations preserve public row choices;
  departed/reentered blockers do not rebind, and remembered blocked status prevents
  damage to the player. Object moves are explicitly synthetic test setup.

The capacity bound covers all factored domain/provisional entries and flat rows;
it does not assert a total observation memory budget. No combinations are
silently clipped. [API details](../../policy-decisions.md).

## Red/green and requirement correction

[Compiled behavioral red](red.txt) records four intended failures: three
`UnsupportedCombat` and one `UnsupportedDecision`, before implementation.
An earlier exploratory run reached wire deserialization first; it is not counted
as the behavioral red. Later normal-reset script setup was corrected to wait for
its second legal land play on the next turn, without changing expected outcomes.
No compile failure is counted as behavioral evidence.

[Two compiled interface mutants](mutations.txt) were detected: ignoring factored
capacity and erasing public blocked status. Original code was restored before
final full verification.

Two old assertions represented the former unsupported combat/discard limitation:
one expected combat observation failure and one advertised those families as
unsupported. They now assert a supported attacker decision and an empty family
list, with the stronger eight-test module above covering the delivered behavior.
Legacy fieldless combat/spell requests still fail explicitly. Independent review
must explicitly check this requirement correction and replacement coverage; no
other test expectation, skip, gate or workflow policy is changed.

Run `cargo test -p mtg-core policy` and full `./scripts/torture.sh` in the managed
Linux container. [Full verification receipt](verification.json): 134 Python tests, 242 Rust tests
and one doctest in each debug/release profile, plus docs/program/catalog/fmt/Clippy,
passed after current-main integration and mutation restoration. Review/commit/CI
evidence is recorded in the issue workpad and delivery PR.

## Reference scope

[Fresh cached XMage combat receipt](xmage-combat.json) executes five matched
original synthetic scenarios and 20 checkpoints: unblocked Cub/Swab, simultaneous
trade, modern 1+1 and 0+2 multiple-blocker damage. The unchanged shared fixture
also runs through the native normal-discovery combat tests. Command:

```sh
python3 scripts/combat_reference.py --cache /tmp/mtg-xmage --output /tmp/combat-reference.json
```

This is pinned, offline, headless, closed-stdin, bounded reference execution.
It verifies existing combat semantics, not this custom policy schema, private
observations, exhaustive choices, full games, keywords or Forge. No production
rules were changed. The managed image's existing writable reference cache was
used; no host installation or Docker invocation occurred.

README updates the supported policy subset and limitations. No usable quickstart
command changed; the full suite checks existing CLI commands. Stage table stays
unchanged. Original integration/catalog acceptance stays with #19.
