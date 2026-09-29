# Owned episode budget acceptance

Related to #161; partial R0002-B036/B037. #154/#117 retain full collector and
aggregate audits; all later owners and gates remain unchanged. No M1 verdict.

Normal-discovery tests: [budgets.rs](../../../crates/mtg-core/tests/budgets.rs) and
[injected failures](../../../crates/mtg-core/src/episode_budget_tests.rs).
[API and precise boundary contract](../../episode-budgets.md).

Independent expectations come from RFC 0002 episode/reward/time and trajectory
contracts, the reviewed collector prerequisite plan, CR 103.5/103.8a (opening and
skipped first draw), 104.3c/704.5b (attempted empty draw), 305.2 (land per turn),
117 (priority passes), and 514 (cleanup). Inputs are literal normal-reset ordered
green mirrors with seed 161 and ordinal 0, not a fake collector or synthetic game.
The configured order contains exactly the frozen green inventory. Both-seat
observations and full state/RNG comparisons are independent of capture output.

- One accepted Keep exhausts one decision; another input cannot mutate state.
  Two mulligan rounds stop with the real two-card bottom choice still pending.
  Two legal land plays on turns 1 and 3 reach a pending Cub payment at decision
  40: truncation neither commits nor cancels it. Captured rows and final inputs
  retain that real continuation and zero rewards.
- Empty turn 1 has seven priority windows (draw step skipped), two passes each,
  plus two opening Keeps: 16 decisions. A one-turn budget stops at turn 2 upkeep.
- Controlled clocks expire before input, in a partial reset, in opening-to-turn
  internal work, and at a ready reset boundary. No sleeps or wall-clock assertions.
  Decision beats coincident turn/time, and turn beats coincident time; real concession beats time at its atomic
  boundary, with correct zero/nonacting-seat rewards. Backwards time fails.
- Actual empty-library loss: 40 minus seven opening cards leaves 33 draws.
  P1 draws on even turns 2..66, then loses at the attempted draw on turn 68.
  Two Keeps + 14 turn-1 passes + 66*(16 passes + one discard) + two final upkeep
  passes = 1140 decisions. Exact-limit 1140 and generous-limit 1200 executions
  both reach the independently specified P0 win / P1 EmptyDraw loss and [1,-1].
  Every operation matches the same unbounded Game script in normalized full
  state/RNG; normalization removes only process-local object-store identities.
  Same-seat transitions sum to the expected returns once.
- Requested unbounded reset/advance quanta are clamped to one core work unit,
  compared after each step with the existing core quantum API. No artificial
  policy decision or terminal row is added.
- Invalid/zero limits, mismatched enforced header, repeated finish/reset,
  immutable retained results, input-domain overflow and record exhaustion remain
  explicit. Previously captured rows remain quarantined, not evicted. Every
  started episode in each mixed completion/reset sequence is accounted once.
- Injected early closure of the real recorder exercises append, concession-finish
  and settlement errors, separately declared component faults. Accepted semantic
  operations remain in privileged history. Failure outranks an underlying
  terminal outcome, prevents further input, and permits one failed result;
  training readers reject its retained prefix. These faults are not substitutes
  for the normal-reset played terminal game.

Compiled intended failures are preserved in [red.log](red.log): API-only stubs
accepted zero limits and a second decision at budget one. Follow-up regressions
retain [lost diagnostic prefix](record-red.log) and [reset-time missing footer](reset-red.log)
failures. The [unrestricted capture-failure regression](unbounded-red.log) also demonstrates
that failure finalization must not depend on choosing the bounded constructor.
All were compiled behavior failures and are fixed without changing existing tests. During new-script construction, rules/contract inspection corrected
an attempted repeated P0 mulligan without P1's declaration/bottoming, an incorrect
first-turn draw window and unnecessary empty-board combat choice, and an illegal
Cub cast with only one Forest. The final literal scripts preserve legal choices;
none rebaseline an existing expectation or infer a winner from engine output.

Validation commands: `cargo test -p mtg-core --test budgets --locked`,
`cargo test -p mtg-core --lib episode::budget_tests --locked`, and mandatory
`./scripts/torture.sh`. Delivery workpad/PR preserve full-run and independent
candidate review results, exact merge and main-CI evidence. A focused pass alone
is not delivery or a milestone claim.

README's owned-episode row now describes this bounded in-memory scope and its
limitations. Quickstart commands/setup are unchanged; torture checks their
existing command paths. Independent review must assess README accuracy. No tests
were removed/weakened/skipped; no rules, CLI, CI or workflow policy changed.
