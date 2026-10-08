# Pyromancer ETB trigger acceptance

GH-206 implements the frozen Viashino Pyromancer: 1R, 2/1, entering creates
an ability that deals two damage to a required player target. The creature spell
has no targets. The ability uses the existing pending controller/APNAP queue;
ordering precedes required target selection, and damage waits for a separate
stack resolution. Either player is legal; there are no planeswalker or battle
candidates in this pool. Source departure does not cancel the ability.

The independent basis is the pinned Pyromancer Oracle record and CR
603.2/603.3d, 115, 113.7a, 608 and 704. [Behavioral red](red.log) compiled and
failed because the cast was rejected. [Snapshot red](snapshot-red.log) compiled
and caught acceptance of a target continuation with its placement rows removed.
Both remain ordinary regressions; expectations were not generated from output.

Delivery requires candidate-bound independent review, full torture after fresh
main integration, protected merge and exact-main CI recorded in the
[issue workpad](https://github.com/pabloxrl/mtg-lab/issues/206). This report does
not itself certify delivery, whole RFC blocks or M2. Partial
R0002-B010/B011/B029 only; original components and #26 retain full acceptance.

## Executable coverage and use

```sh
cargo test -p mtg-core --locked --lib etb_trigger_tests
cargo test -p mtg-recorder --locked --test etb_triggers
python3 scripts/etb_trigger_reference.py --cache /home/agent/.cache/xmage --output /tmp/etb-reference
./scripts/torture.sh
```

Seven core tests cover:

- `pyromancer_etb_requires_player_target_then_deals_two`: no damage or target
  prompt on cast; entry is 2/1; both targets lose exactly two after resolution,
  including after source departure.
- `pyromancer_missing_creature_wrong_seat_stale_target_reject_without_mutation`:
  empty, pass, creature, wrong-seat and stale submissions preserve snapshots;
  self-target at two life loses.
- `pyromancer_corrupt_target_continuation_rejects_restore`: missing/duplicate
  rows, wrong actor and missing return state reject transactionally.
- `pyromancer_quantum_snapshot_target_and_resolution`: quantum one/unbounded,
  restoration at every internal yield and pending target, stale capabilities,
  public stacked player target and literal life checkpoints.
- `pyromancer_synthetic_multi_target_apnap_placement_and_end_step`: explicit
  synthetic multi-controller queue, both active seats, multiple target pauses,
  controller order/APNAP and all resolutions before advancing from end step.
- `pyromancer_concession_during_target_clears_choice_and_restores_terminal`:
  concession cancels the target-selection continuation; the terminal snapshot
  restores and neither seat observes an active choice. Its compiled
  [red receipt](concession-red.log) caught the stale continuation.
- `pyromancer_reference_literal_checkpoints`: shared neutral scenarios with
  independent intermediate ledgers, real Bite response killing the source,
  target rejection and self-lethal outcomes.

`pyromancer_normal_reset_typed_capture_replay_quantum` plays two real Pyromancers
from normally reset, ordered frozen red decks using the existing Driver/Run.
It selects self then opponent, yielding 20/20 → 18/20 → 18/18. Capture on/off and
quantum one/unbounded preserve semantic history; canonical typed observations,
selected actions, rewards and JSONL round-trip remain exact. Wrong-seat, missing
and stale choices do not change state, history or capture. Replay reconstructs
the game. Missing player targets in stored stack observations reject.

At `trigger_order`, submit the complete controller-local bottom-to-top
`OrderTrigger` permutation. At `trigger_target`, submit exactly one
`TargetPlayer { seat: 0 | 1 }`; semantic JSON uses `target_player`. There is no
pass/cancel/default target. `selecting_target` identifies the pending source;
stack source features retain `target_player` even after source death. The raw
scalar API is `target_trigger_quantum`. Snapshot/replay compatibility retains
the existing conservative source fingerprint rule.

## Native and pinned XMage reference boundary

The [neutral input](../../../fixtures/reference/etb-triggers.json) and
[hand-authored expectations](../../../fixtures/reference/etb-triggers-expectations.json)
execute 14 scenarios twice in native Rust and XMage
`000d8a7abc0ac31cc24af08691423e0c24dc59e7`. The [receipt](reference.json)
records pins/source hashes and comparator negative controls. [Native](native.json)
and [XMage](xmage.json) results compare both life totals, Pyromancer stats,
stack count, player targets and loss flags before entry, after target selection,
after source death where applicable, and after resolution. The Python regression
mutates every checkpoint field and removes every case to prove strict comparison.

Twelve scenario IDs are the unchanged catalog IDs: required-targets interaction,
resolution-revalidation regression, all four ETB-target cases, dead-source positive,
turn-structure interaction and all four Pyromancer card cases. The extra
`self_nonlethal` and `required_target` cases reinforce the target domain/cardinality.
Bite death is a real spell response in both engines. Direct source departure and
end-step queue injection are explicitly synthetic hooks. XMage validates player
and creature target legality and minimum target cardinality through its own
Target API; raw transaction rejection, end-step advancement, quantum/snapshot,
normal-reset replay and recording are native evidence, not claimed XMage fields.
The cleanup-start pending-trigger composition still needs #207 and remains #211
composition acceptance. No applicable basic Pyromancer case is deferred.

## Support and regression corrections

README now describes twenty scripted core cards. Existing native policies retain
their seventeen-card subset until #208 and explicitly reject unsupported trigger
choices. No new command/setup, cast-trigger behavior, permanent type, full-pool
policy, Forge or reference full-game claim is introduced.

The former unsupported-card assertions used Pyromancer as the last unsupported
frozen identity. Its newly required support is checked against the independent
manifest/cost/stat table and the tests above. The casting rejection fixture now
uses an uncastable Goblin token; unsupported-combat rejection now uses an explicitly
invalid sorcery on the battlefield. Unknown/corrupt identity rejection remains.
No rejection assertion is removed; new TurnKind arms in unrelated scripts panic
if unexpectedly reached. Independent review must explicitly assess these
requirement corrections, replacement coverage and README accuracy.

Full regression also exposed a native stop-accounting fixture using the default
red/green decks: Pyromancer becoming playable correctly triggered the unchanged
native policy's unsupported-content error before the scripted interrupt. That
single test now uses the policy-supported green mirror, preserving every stop,
deadline, decision/work count and nonterminal assertion. The failing full-run
receipt is [retained](native-policy-red.log); the focused unchanged assertions pass.
This correction does not expand the native policy or add a fallback and is also
subject to independent review.
