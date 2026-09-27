# Bounded combat damage settlement — GH-108

Partial R0002-B016/B017/B026/B029 component delivery. #18 retains the original
integration acceptance and catalog expectations; #22 alone gates M1 completion.
No turn/cleanup refactor, new keyword, policy/collector or full-game claim.

## Independent acceptance and red/green

Original tests in `crates/mtg-core/src/combat_tests.rs` use synthetic old vanilla
creatures, then real declaration, allocation and priority APIs. Expected results
come from frozen Bear Cub/Swab Goblin 2/2 definitions and CR 510.1c/510.2,
704.5a/g and 117.5. Scalar equality supplements the literal assertions; it does
not supply their expectations.

- `combat_quantum_all_modern_allocations_simultaneous_lethal_both_seats`: all
  0+2, 1+1, 2+0 allocations. The attacker takes four and dies. Exactly a blocker
  assigned two dies; otherwise both retain one damage. Life stays 20/20, survivors
  remain untapped 2/2s, active player gets priority in CombatDamage, stack is empty.
- `combat_quantum_removed_blockers_remember_blocked_status`: one or both blockers
  leave; one reenters with new identity. One remaining blocker trades with the
  attacker; zero leaves a blocked, undamaged attacker and no player damage.
- `combat_quantum_terminal_after_all_damage_and_deaths`: a blocked trade plus two
  unblocked 2/2s takes defender from 1 to -3. Both creatures in the trade die before
  publication; active player wins, with defender's literal Life loss. Both seats.
- `combat_quantum_preflight_rejections_preserve_exact_snapshot`: wrong seat,
  missing/duplicate allocations, stale completion, insufficient inspection
  capacity, numeric overflow and exhausted generation reject without mutation.
- Existing `combat_shared_xmage_reference_checkpoints` retains its scalar checks
  and additionally runs each shared fixture at budgets 1/2/3/5/64 against its
  independent expected damage checkpoint. No assertion removed or weakened.

Budgets 1/2/3/4/5/6/8/64 (relevant subsets per scenario) compare complete normalized
state, including RNG and identity generations. Only fresh store capability scopes
are normalized after restore. Every internal yield checks both seat views,
turn/opening decisions and outcome are unavailable, commands reject without
changing snapshot bytes, and restored work resumes identically. Generation
advances exactly once; settled duplicate resumes and completion retries cannot
repeat damage. Tests remain in normal discovery. Existing normal-reset combat
scripts still execute the scalar API through the same continuation.

[Compiled behavioral red](red.txt): a compatibility-only quantum entry point
called the existing scalar implementation; three tests failed because quantum 1
returned settled priority/terminal instead of InternalYield. The negative test
passed. A preceding test compilation correction (RNG has no Clone) is not counted
as behavioral failure. [Focused green](green.txt) records the implemented tests.

## Reference and reproduction

The unchanged [shared fixtures](../../../fixtures/reference/vanilla-combat.json)
and [original pinned XMage bridge](../../../references/xmage/VanillaCombatTest.java)
cover five cases and 20 checkpoints: unblocked Cub/Swab, simultaneous trade, and
modern 1+1/0+2 allocations. The native test checks these same expected checkpoints
through bounded and scalar paths. The [combat receipt](xmage-combat.json) records
all 20 agreed checkpoints. The [terminal receipt](xmage-terminal.json) separately
records seven agreed cached terminal boundaries, not terminal-combat reference
agreement. Final full-suite/review/CI status is recorded with this delivery;
a missing or failed reference run is not agreement.

```sh
cargo test -p mtg-core combat
./scripts/torture.sh
python3 scripts/combat_reference.py --cache /tmp/mtg-xmage --output /tmp/combat-reference.json
```

Use the managed image's writable cached reference copy. The bridge runs offline,
headless, stdin closed, with pinned source/card/dependency checks and a 300-second
timeout. Observations cover life, battlefield/graveyard, power/toughness/damage,
tapping and blocker membership. It does not verify internal scheduling, private
views, removed/reentered blockers, all allocations, terminal combat, full games or
Forge; these are native checks or future reference scope, never implied agreement.

README now documents the bounded combat API, remaining scalar declarations and
turn/cleanup work, and snapshot support. Quickstart commands are unchanged;
full torture covers their inner CLI/comparator commands. Milestone table unchanged.
The workpad/PR records exact candidate review and exact merged-main CI.

[Full-suite receipt](verification.json): complete torture passes before and after
fresh-main integration, including 134 Python tests, 209 Rust tests and one doctest
in each debug/release profile, documentation/program/design checks, fmt and Clippy.
No tests skipped, removed or weakened. Separate review and protected delivery
remain recorded against the final candidate in the issue workpad/PR.
