# Resumable core work quantum — issue #66

Scope: R0002-B015/B016 and SYS-CORE-006 core prefix. No full game-catalog cases
are assigned. Effects, snapshots, batch fairness, rewards and full component
acceptance remain registered sibling/integration work; M1 is not complete.

Independent requirements: RFC 0002 §4 says internal yields are not actions,
rules events, terminal states or RL transitions and must resume deterministically.
Pinned CR 103.5 requires seven-card redraws, cumulative ordered bottoms after
each mulligan, kept-seat independence and forced keep at zero. The existing
opening/mulligan suite retains independently computed Python shuffle vectors.

Before implementation, synchronous API adapters compiled and failed three
behavioral assertions: reset and mulligan failed to yield for quantum one,
and unstarted resume was incorrectly reported as completed opening.
[Red output](red.txt) preserves those failures; the same assertions remain in
[quantum.rs](../../../crates/mtg-core/tests/quantum.rs). No existing test or
expectation was removed, skipped or weakened.

[Named green output](green.txt) covers seven tests in normal Cargo discovery:

- Quantum-one reset/mulligan yields, no exposed decision or premature draw,
  pending-action rejection preserving the complete debug state, resume exactly
  once and repeated settled resume without mutation.
- Independent explicit card ledger from the frozen manifest: two mulligans,
  reversed chance permutations, ordered bottom indices 5 then 6/0, a kept
  opponent, and changing resume budgets 1/7/40/2.
- Fifteen quanta (1, 2, 7, 38, 39, 40, 41, 78, 79, 80, 94, 173, 174, 175,
  10000), both starters and seeded/explicit chance through all seven mulligans.
  Every boundary compares full ordered card objects, RNG, decisions, counters
  and opening flags against scalar execution. Independently required final
  counts are zero hands, 40 libraries, 29 generations. No yield increments
  generation or changes life, and there is no reward/terminal variant on yield.
- Rejected shuffle-tail trials retain cards/cursor, then accept and advance
  exactly once; literal boundary words derive from `2^64 = 40*q + 16`.
- One-unit progression consumes at most one RNG word or allocation, with an
  independently initialized environment stream confirming exactly 78 accepted
  shuffle words for seed 42/episode 9.

[Full torture receipt](torture.txt) records the mandatory suite in the managed
Linux Docker worker after main integration. It includes formatting, Clippy,
documentation/program/catalog checks, all Python tests and all Rust tests in
debug/release. README and API documentation describe the new usable behavior;
no milestone status changes. Current reference bridges only execute priority
smoke, so there is no impacted cached quantum scenario or claimed reference
agreement. Review and exact-main CI links are recorded in the issue workpad/PR.
