# GH-107 spell settlement acceptance

Requirements: partial R0002-B016/B017/B026/B029. Original integration acceptance
and all catalog expectations remain with #18 and later owners. M1 is incomplete.

## Independent behavior and red/green

The original tests cite CR 117.3b/117.4, 400.7, 608.2b/h, 613.4c, 704.5g and the
pinned Growth/Bite Oracle text. Seed 42 / episode 9; synthetic postcombat
battlefields and mana pools are explicitly declared. Target selection, payment,
casting and response/pass commands are real core APIs; no fake sibling engine.
This is not a normal-reset full played-game claim.

[Compiled behavioral red](behavioral-red.txt): an API adapter initially drained the
existing synchronous implementation. Two tests asserted `InternalYield` at
quantum 1 and received a player decision instead. Earlier test-import/borrow
compile fixes are not red evidence. The full-state resume test also caught the
old `OpeningComplete` response during turns.

Normal-discovery tests in `crates/mtg-core/src/targets_tests.rs`:

- `settlement_quantum_growth_responds_to_bite_literal_checkpoints`: P0 casts Bite;
  P1 responds with Growth on the destination. Literal priority P0/P0/P1/P0 and
  stack sizes 2/1/1/0; destination becomes 5/5 then has two damage and survives;
  source stays 2/2. Payment stays 28/29 green units from original 30 each.
  Old spell handles expire; RNG is unchanged. Budgets 1,2,3,4,5,6,64 compare full
  states against independently asserted scalar checkpoints (scope rebasing only).
- `settlement_quantum_bite_uses_grown_source_and_terminal_boundary`: Growth on
  source makes Bite deal five and kill a 2/2. Literal owner graveyard and identity
  checks; synthetic zero-life boundary separately verifies final P0 winner and loss reason.
  Budgets 1,4,5,64 match complete scalar state, identity, RNG and outcome.
- `settlement_quantum_response_death_revalidates_targets`: an actual responding
  Bite kills the lower spell's Growth target or Bite source. Lower Growth has no
  legal targets; lower Bite has one but deals no damage. Budgets 1,3,64.
- `settlement_quantum_creature_identity_and_sickness`: actual Cub/Goblin payment
  and casting; three work units, fresh battlefield identity, 2/2 and sickness,
  active-player priority, unchanged committed mana at budgets 1,3,64.
- `settlement_quantum_rejections_preserve_exact_state`: explicit target capacity,
  wrong seat, stale decision, and damage-overflow errors preserve snapshot bytes.

Shared drain assertions check every internal phase: both player views unavailable,
no decisions/outcome, rejected pending commands preserve exact bytes, snapshot
restore continues to identical full state/RNG, and repeated resumes never repeat
effects. Existing effect/identity/terminal/quantum/snapshot regressions remain
unchanged. No tests removed, skipped, weakened or replaced.

## Verification

Five focused tests pass. [Two cached XMage priority runs](xmage-priority.json)
and [seven cached XMage terminal cases](xmage-terminal.json) agreed, with pinned
source/dependency checks, closed stdin, no display, offline Maven and bounded
timeouts in the managed Linux ARM64 container. The image cache was copied to
writable `/tmp/mtg-xmage`; no host installation or Docker invocation occurred.
Reproduce using `python3 scripts/xmage.py run --cache /tmp/mtg-xmage` and
`python3 scripts/terminal_reference.py --cache /tmp/mtg-xmage --output /tmp/terminal-reference.json`.
No reference Growth/Bite agreement is claimed: matched effect bridges belong
to #115. These cached scenarios check their declared priority/terminal fields
only; they are not effect, full-game, Forge or quantum reference evidence.

[Full torture receipt](verification.json): 134 Python tests, 205 Rust tests plus
one doctest per debug/release profile; docs/program/catalog/fmt/Clippy passed.
The complete suite passed again after integrating freshly fetched current main.
Independent review and protected merge/exact-main CI are recorded in the delivery
workpad/PR. No failed reviewer execution counts as a pass.

README now documents the usable bounded spell API and its exact limitations.
Quickstart commands and milestone table are unchanged. Full integration, rewards,
combat/turn quantum work, scheduler/batch execution and later RFC obligations are
not certified by this component.
