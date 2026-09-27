# Turn steps and priority — GH-67

Scope: the atomic turn prefix assigned to GH-67 under R0002-B016/B017/B029.
No complete shared RFC block or M1 verdict is claimed. [API](../../turns.md)
describes the implementation and explicit limits.

Expectations were authored from pinned CR 103.8a (skip the starter's first draw
step), 106.4/500.4 (mana boundaries), 117.3/117.4 (priority), 502.3 (untap by
controller), 504.1 (draw before priority), 508.8 (skip blockers/damage without
attackers), and 514.1/514.3 (cleanup discard, no ordinary priority). RFC 0002
requires rejection before mutation and decision-scoped candidates. The frozen
card manifest supplies independently ordered input decks, with the eighth card
swapped with index 16 so drawing the wrong basic-land position is detectable.
No implementation output defines expected behavior.

[Red log](red.txt): seven compiled behavioral failures against `NotReady` API
stubs, showing that a completed opening could not enter turn play. An earlier
missing test-helper import was fixed before collecting red evidence; compile
errors are not the behavioral red. Those assertions all remain in normal Cargo
discovery. [Green log](green.txt) executes ten tests including the additional
boundary and neutral-reference checks. No old test was deleted, skipped or
weakened; the catalog and its expectations are unchanged.

| Assigned catalog case | Executable test in `cargo test -p mtg-core turns` |
| --- | --- |
| rules-setup-first-draw-positive | turns_first_draw_positive_negative_interaction_regression |
| rules-setup-first-draw-negative | turns_first_draw_positive_negative_interaction_regression |
| rules-setup-first-draw-interaction | turns_first_draw_positive_negative_interaction_regression |
| rules-setup-first-draw-regression | turns_first_draw_positive_negative_interaction_regression |
| rules-priority-passing-positive | turns_priority_passing_positive_and_stale_candidates_negative |
| rules-decisions-stale-candidates-negative | turns_priority_passing_positive_and_stale_candidates_negative |
| rules-setup-two-player-opening-interaction | turns_red_mirror_mulligan_opening_interaction |
| rules-priority-turn-structure-positive | turns_untap_controller_and_mana_every_boundary; turns_empty_combat_sequence_cleanup_and_reset_scope |

Public integration tests are in [tests/turns.rs](../../../crates/mtg-core/tests/turns.rs).
Synthetic tests in [src/turns.rs](../../../crates/mtg-core/src/turns.rs) explicitly
seed lands with different owners/controllers, all six mana colors for both
players, empty-library/unsupported-stack/creature boundaries, multiple-card
cleanup and finite counter exhaustion. These are not claims of implemented land
plays, mana production, casting, terminal outcomes or combat. The normal reset
flows exercise both starters, known top draws through turn three, seven-card
cleanup, full empty-combat priority sequence and the red/red P1-start/P0-mulligan
case. Invalid inputs compare complete private debug state, including RNG and
storage capacities; valid actions after rejection still reach independently
required checkpoints. Reset and cross-game stale candidates are covered.

[Mutation log](mutations.txt) records four temporary semantic faults: skip every
draw, retain mana across boundaries, omit untap, and accept stale pass candidates.
Each compiles and is caught by its named assertion. Source was restored before
the final green/full-suite run. These seeded risks are not historical bugs.

[Full torture receipt](torture.txt) is from the managed Linux Docker worker after
fetch/integration of current main: docs/program/catalog checks, 120 Python tests,
all Rust tests in debug/release, formatting and Clippy. Named acceptance is run
separately too. No Docker invocation or socket access occurs inside the worker.
README and opening/storage docs now describe the usable turn prefix; milestone
status remains unchanged. Exact candidate review and CI links live in the
[GH-67 workpad](https://github.com/pabloxrl/mtg-lab/issues/67#issuecomment-5855607690)
and its PR.

## Reference boundary

The existing [neutral priority-pass fixture](../../../fixtures/scenarios/xmage-priority-pass.json)
is decoded into a synthetic Rust position in
`turns_same_neutral_priority_fixture_as_reference_bridges`. The test compares
all smoke-observable fields and exact unchanged storage/RNG against the
rule-derived fixture, including its checkpoint assertions. It is a narrow
fixture decoder, not a general scenario runner. The cached real XMage bridge
executed the identical unchanged fixture twice, independently, with
`python3 scripts/xmage.py run --cache /home/agent/.cache/xmage`.
[Fresh receipt](xmage.json) preserves input/bridge/toolchain/checkpoint/log hashes,
closed stdin, absent display and Linux ARM64 execution. This is priority-handoff
agreement only; no full-turn/reference draw or cleanup claim is made. Reference
bridges do not yet support those cases.

Forge was not freshly executed: its documented `/tmp/mtg-forge` cache is absent
in this worker. Its historical smoke receipt remains in `references/forge` and
is not counted as fresh agreement. The impacted available cached scenario was
run in XMage. Broader dual-reference and every-capability execution remains
assigned release/integration work; this receipt is not release qualification.
