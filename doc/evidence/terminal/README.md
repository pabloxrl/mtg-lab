# GH-72 terminal outcomes and reset acceptance

Atomic portion of R0002-B016/B026/B029 and SYS-CORE-001/004/008.
The full shared blocks, reward ledger and cross-feature integration remain with
#18/#20/#76 and the M1 gate. M1 remains incomplete.

## Independent rules and red/green

CR 104.3a: either player may concede at any time and loses immediately.
CR 104.4a: simultaneous losses by both players draw. CR 704.5a: zero or negative
life loses. CR 704.5b: a failed draw causes loss, not library emptiness itself.
CR 103.8a skips the starting player's first draw. CR 704.5g creature death does
not imply player loss. CR 400.7 gives new zone identity; CR 514.2 clears damage
and expiring effects together. RFC 0002 §3 requires persistent seats and
best-of-one; §8 separates external truncations from rules results.

[red.txt](red.txt) retains four compiled behavioral failures before implementation:
lethal still granted priority; empty draw was an error instead of terminal;
nonacting concession and pending-payment concession could not complete. A minimal
API scaffold returned no outcome/NotStarted so assertions compiled; it implemented
no successful concession. The illegal-hand-blocker negative already passed.
[green.txt](green.txt) records the named regression run after implementation.
No engine output was used to author expected results.

The full suite exposed two earlier placeholder expectations that empty draw left
all state unchanged (`turns` and `mulligan` tests). These are replaced under
CR 704.5b with explicit seat loss, unchanged card/life checks, draw-step position,
and post-terminal nonmutation. Original ordered-bottom/draw assertions and
unsupported-combat rejection remain. Independent review must explicitly assess
this requirement correction and stronger replacement coverage before delivery.

The new stale-episode regression also failed behaviorally: the first draft's
store-only token survived reset. The corrected token includes the reset decision
generation. Its normal-discovery test retains foreign/reset rejection.

## Exact assigned catalog crosswalk

Every function below is discovered by `cargo test -p mtg-core terminal`.
Unit scenarios in `src/terminal_tests.rs` declare synthetic starting state;
integration scripts in `tests/terminal.rs` use real frozen deck reset, keeps,
lands, payments, spells, turns, combat, cleanup and outcomes.

| Catalog case | Executable evidence |
| --- | --- |
| rules-setup-deterministic-reset-negative | `terminal_reset_after_real_mana_growth_and_bite_damage`: nonzero floated mana and real Growth/Bite damage; malformed deck reset preserves full Debug state including RNG; valid random reset equals fresh semantic opening |
| rules-objects-bookkeeping-negative | `terminal_hand_cub_cannot_block`: hand handle rejected unchanged, no phantom permanent |
| rules-objects-bookkeeping-regression | `terminal_grown_dead_cub_then_real_cast_has_fresh_bookkeeping`: real Growth, three real Bite resolutions kill 5/5, a new same-name creature is cast as clean 2/2 |
| rules-terminal-lethal-positive | `terminal_lethal_zero_negative_and_creature_death_is_not_player_loss`: unblocked Cub at two life ends at zero before priority; one life also ends at −1 |
| rules-terminal-lethal-negative | Same function: three life becomes one, no result |
| rules-terminal-lethal-regression | Same function: last Cub dies in a trade at 20 life; game continues |
| rules-terminal-empty-draw-positive | `terminal_empty_draw_versus_empty_library_and_first_draw_skip`: normal draw step due; P0 loses at attempted draw |
| rules-terminal-empty-draw-negative | Same function: empty-library main-phase pass continues |
| rules-terminal-empty-draw-regression | Same function: skipped first draw does not lose |
| rules-terminal-concede-positive | `terminal_main_priority_concession_each_seat_and_stale_actions_after_lethal`: P0 and P1 each concede at their main-phase priority and lose immediately |
| rules-terminal-concede-negative | `terminal_concede_nonacting_seat_and_finality`: winner's later concession cannot rewrite result or mutate state |
| rules-terminal-concede-interaction | `terminal_concession_preserves_stack_and_invalidates_target_and_cast_choices`: P1 concedes while P0 retains priority with Growth unresolved; stack preserved |
| rules-terminal-concede-regression | Same function and payment test: paid cast/payment cannot commit or cancel into a live decision after concession |
| rules-setup-two-player-opening-regression | `terminal_full_opening_turns_combat_best_of_one_and_explicit_reset`: ten actual 2-damage attacks, stable result and no auto-reset; only explicit reset starts new episode |

Additional acceptance: `terminal_simultaneous_losses_and_external_work_limits`
checks both nonpositive seats draw, repeat settlement, one-unit yield without
outcome, concession during yielded reset, frozen resume, stale/foreign tokens.
Normal-reset red/green mirrors run both starting seats to combat lethal. A second
complete script runs both starting seats to turn 68's failed draw: the nonstarter
loses, both libraries empty, life unchanged. A bounded collector stop earlier in
that script has no rules result. No fake sibling implementation is used.

## Reference execution

[Seven matched XMage boundary cases](xmage-terminal.json) executed against pinned
commit `000d8a7abc0ac31cc24af08691423e0c24dc59e7`, offline/headless in the managed
Docker worker using writable `/tmp/mtg-xmage`. The same original
[neutral fixture](../../../fixtures/reference/terminal.json) is consumed by the
native `terminal_same_neutral_boundaries_as_xmage` test and
[TerminalTest.java](../../../references/xmage/TerminalTest.java).

Life is explicitly injected at a synthetic upkeep boundary before the real
reference SBA check. The real draw primitive/concession are invoked as scripted.
Observed life, seat loss flags, hand/library counts agree in all seven cases:
zero life, negative life, simultaneous zero life, empty without draw, empty draw,
last-card draw, and nonacting concession. Every observed field/seat plus missing
and extra cases is corrupted by normal Python comparator regression tests.
First Java compilation used the wrong `setLife` signature; corrected to the pinned
API. That failed build was not counted as agreement.

Reproduce with `python3 scripts/terminal_reference.py --cache /tmp/mtg-xmage --output /tmp/terminal-reference.json`.
Toolchain/source/dependency pins use the existing XMage runner; a 300-second
process-group timeout bounds execution. [API provenance](../../../references/xmage/terminal-provenance.json)
and [upstream MIT notice](../../../references/xmage/UPSTREAM-LICENSE.txt) apply.
No upstream scenario/code is copied and no upstream source patch is made.

These are synthetic settled-boundary comparisons, not reference full games,
combat-triggered endings, complete state, winner flags, private views or Forge
coverage. Native tests cover actual combat-triggered endings and full scripts.
Later integration/dual-reference obligations remain with their registered owners.

## Use and limitations

See [the terminal API](../../terminal.md). README supported-behavior/API/evidence
rows are updated; milestone table unchanged. Quickstart commands are unchanged;
inner managed-container validation is covered by the full torture command.
No policy views, recorder rewards, CLI matches, multi-draw spell effects, new
cards/keywords or program/workflow changes are included.

## Validation receipt

[Full torture receipt](verification.json): 132 Python tests and 116 Rust tests
in each debug/release profile, formatting, Clippy, docs, program and catalog all
pass after current-main integration. Named acceptance passes 11 unit tests
(including the updated turns regression) and three integration tests.
[Three compiled mutants](mutations.json) are killed: zero life not lethal,
payment surviving ending, and stale episode acceptance. Original code was
restored before the full suite. [Five existing matched XMage combat cases](xmage-combat.json)
also reran successfully. Final separate review and exact commit/CI evidence are
recorded in the issue/PR; this report alone does not claim merged delivery.
