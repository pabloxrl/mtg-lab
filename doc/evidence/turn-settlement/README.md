# Bounded turn and cleanup settlement — GH-109

Partial R0002-B016/B017/B026/B029 delivery. #18 retains all original integration
acceptance and catalog expectations; #22 gates M1. No scheduler, new card/trigger,
collector or full-game capability is claimed. M2–M5 obligations remain unchanged.

## Executable evidence

Seven normal-discovery tests in `crates/mtg-core/src/turn_quantum_tests.rs` use
explicit synthetic boundary positions after real reset/keep/start. Expectations
come from CR 103.8a, 502.3, 504.1, 514.1–3, 704.5b and frozen vanilla 2/2s;
normalized full-state scalar equality supplements literal checks. Existing real
turn, spell, combat, terminal and opening regressions remain intact.

- `turn_quantum_draw_skip_order_and_empty_terminal`: both seats, skipped first
  draw step, ordered subsequent draw, empty draw loss and no extra card.
- `turn_quantum_cleanup_discard_and_simultaneous_expiration`: both seats, zero
  or two discards, actual reversed chosen-card order in graveyard, two 5/5s with
  four marks survive as 2/2s with zero marks, next controller alone untaps and
  loses summoning sickness. Discard decision retains all pre-expiration effects.
- `turn_quantum_priority_in_cleanup_repeats_discard_before_next_turn`: explicitly
  synthetic CR 514.3a priority window with ordinary or enlarged hand; repeat
  cleanup expires fresh modifiers and asks for any additional discard before advancing. No implemented trigger is
  claimed to create this setup; M1 positive boosts cannot create that window.
- `turn_quantum_initial_untap_unit_bound_and_controller`: ownership differs from
  controller; literal four units for boundary, sickness removal, untap and
  publication; budgets 1/2/3/4/64 stop at the expected counts.
- `turn_quantum_literal_intermediate_cleanup_and_draw_units`: exact cleanup and
  draw intermediate states, including combined boost/damage removal, no SBA
  between creatures, draw before publication, and no untap policy decision.
- `turn_quantum_each_empty_combat_step_and_rejections`: every empty-combat turn
  boundary at budgets 1/2/3/64, wrong actor unchanged and first-pass behavior.
- `turn_quantum_shared_xmage_draw_checkpoints`: unchanged shared empty/last-card
  draw fixture expectations through the actual bounded turn path, budgets
  1/2/3/64. The existing primitive-based shared test is retained unchanged.

The main comparisons use budgets 1/2/3/4/5/8/64 as applicable. Every pending
boundary checks absent policy views/decisions/outcome, unchanged rejected draw
and concession, and exact restored continuation suffix (only capability scopes
normalize). Pending turn retries reject unchanged; generation increments once;
settled resumes are read-only. These core yields have no reward/action record;
collector integration remains its registered owner's acceptance.

[Compiled behavioral red](red.txt): three initial tests failed against the
unchanged implementation because a one-unit budget published settled priority
instead of yielding. No compiler error is counted as red. Supplemental tests
extend intermediate-state, initial-untap, boundary and shared-reference coverage.
[Focused green](green.txt) records the completed tests.

## References and reproduction

[Fresh cached XMage receipt](xmage-terminal.json): all seven existing terminal
scenarios agree; the two draw scenarios also run through bounded native turns.
Pinned source/card/dependency validation, offline/headless execution, closed
stdin and the existing 300-second timeout are retained. No upstream code patch,
host installation or worker Docker invocation. This reference verifies terminal
flags, life and zone counts; it does not verify scheduling, full turns, cleanup,
private views, full games or Forge. Cleanup has native rules-derived evidence;
there is no matching cached cleanup bridge. The [cached priority smoke](xmage-priority.json)
also agrees in two runs; it observes the existing first-pass boundary only.
Broader reference acceptance stays
with the registered integration/release owners.

```sh
cargo test -p mtg-core turn_quantum
./scripts/torture.sh
python3 scripts/terminal_reference.py --cache /tmp/mtg-xmage --output /tmp/turn-terminal.json
```

Run in the managed toolchain container using its writable reference cache.
README and turn/opening/spell/snapshot docs describe the new supported bounded
work, exact unit costs, synthetic repeated-cleanup limitation and deferred
integration. Quickstart commands are unchanged; full torture exercises their
inner CLI/comparator commands. No milestone status changes.

[Full torture receipt](verification.json): 134 Python tests, 216 Rust tests and
one doctest in each debug/release profile, plus docs/program/catalog/fmt/Clippy.
The complete suite passed before and after final refinements on current main.
Independent review and exact-main delivery evidence are recorded in
the issue workpad and PR against the final candidate. No tests are removed,
skipped or weakened. Old snapshot fingerprints reject without migration.
