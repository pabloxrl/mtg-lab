# Archer and Cyclops cast-trigger acceptance

GH-205 implements Firebrand Archer (1R, 2/1) and Crackling Cyclops (2R, 0/4)
from the frozen card manifest. Only their controller's **committed noncreature
cast** detects these abilities (CR 601.2i, 603.2). Pending source-specific
abilities use the existing controller/APNAP placement, policy `OrderTrigger`,
semantic `order_trigger`, owned work, snapshot and typed recording paths.
Placement changes neither life nor power. Archer resolves for one damage to the
opponent; Cyclops resolves for +3/+0 on its original incarnation until cleanup.
CR 113.7a preserves stacked abilities after source death; CR 400.7 prevents an
old Cyclops ability from boosting a returned incarnation.

The compiled [behavioral red](red.log) predates implementation: Archer had zero
pending triggers after committed Growth (expected one), and Cyclops had no
creature state (expected pinned 0/4). Neither failure was a compile/import error.
Independent expectations come from the pinned card records and CR
601.2i/603/113.7a/400.7/611.2a/514, not generated engine results.

Delivery requires the candidate-bound separate Codex review, full torture,
protected merge and successful CI on the exact merge commit, recorded in the
[issue workpad](https://github.com/pabloxrl/mtg-lab/issues/205). This report alone
is not a completed delivery or M2 verdict. Partial R0002-B010/B011/B029 only;
original component owners and #26 retain full acceptance.

## Ordinary executable coverage

`cargo test -p mtg-core --locked --lib cast_trigger_tests` runs:

- `archer_committed_cast_waits_for_resolution_and_survives_source_death`:
  20/20 until trigger resolution, then 20/19 before Growth; source death does
  not cancel the stacked ability.
- `cyclops_trigger_then_growth_literal_stat_ledger`: 0/4, 3/4 after trigger,
  6/7 after Growth, then 0/4 at cleanup.
- `cast_trigger_all_six_orders_and_quantum_snapshots`: two Archers and Cyclops,
  all six bottom-to-top orders, one/unbounded work quantum and restore at every
  internal placement/resolution yield; no tokens until Fodder resolves.
  Incomplete/duplicate/foreign permutations preserve the snapshot.
- `cast_trigger_negative_land_mana_creature_failed_and_cancelled`: land play,
  mana ability and creature cast produce no trigger; no-green cast, unpaid
  finish and cancelled paid cast preserve life/power and create no triggers.
- `cast_trigger_opponent_cast_and_departed_cyclops_incarnation`: opponent
  sources do not trigger; a returned Cyclops gets neither the old trigger boost
  nor the old Growth targeting its prior incarnation.
- `cast_trigger_two_archers_growth_and_invalidated_spell`: two ordered damage
  resolutions happen even when Growth's target departs and the spell fails.
- `cast_trigger_archer_lethal_stops_before_growth_and_apnap_fixture`: lethal
  Archer damage settles terminal state before Growth; an explicitly synthetic
  two-controller Archer queue resolves nonactive before active.
- `cast_trigger_reference_literal_checkpoints`: executes the shared neutral
  reference cases and checks every independently specified intermediate ledger.

`cargo test -p mtg-recorder --locked --test cast_triggers` runs
`cast_triggers_normal_reset_typed_capture_replay_quantum`: real ordered frozen
red decks, opening keeps, land plays, paid Archer/Cyclops casts and Fodder,
controller order, 20/20 → 20/19 → 20/18, token resolution and cleanup. Existing
Driver/Run owns the game. Capture on/off and quantum one/unbounded yield identical
semantic histories; owned typed observations/submissions survive JSONL round-trip
and semantic played replay. Wrong-seat, incomplete and stale orders leave state,
RNG/history and capture unchanged. No synthetic object injection in this test.

The existing card-definition test now independently checks both new costs and
printed stats against the frozen manifest; the unsupported set still rejects
Pyromancer. This is the intentional requirement correction to the earlier
unsupported-card list, with equivalent rejection coverage for remaining
unsupported content and stronger positive coverage for the newly delivered
cards. Independent review must check this correction and README accuracy.
Existing synthetic ordering tests remain, including their original expected
orders; their tag extraction now explicitly rejects a non-synthetic kind.

## Pinned reference execution and catalog allocation

```sh
python3 scripts/cast_trigger_reference.py --cache /home/agent/.cache/xmage --output /tmp/cast-trigger-reference
python3 scripts/instant_reference.py --cache /home/agent/.cache/xmage --output /tmp/cast-trigger-instants
```

The shared [input](../../../fixtures/reference/cast-triggers.json) and
[hand-authored ledger](../../../fixtures/reference/cast-triggers-expectations.json)
run 22 cases twice in native Rust and pinned XMage
`000d8a7abc0ac31cc24af08691423e0c24dc59e7`. Compared points contain both life
totals, Cyclops power/toughness, stack count and Goblin count before and after
each resolution, plus cleanup where specified. The bridge explicitly selects
source identities in each trigger permutation; no AI choice fallback. Case
`apnap` explicitly injects a synthetic pending ability per controller in both
engines; the pool cannot naturally produce that same cast event for both seats.

Native transaction tests cover complete rejection nonmutation. XMage no-mana
checks its playable set; failed Thrill calls the actual cast three times (its
preliminary playable set can still contain Thrill); cancellation rejects target
selection before commitment. These are explicit translation boundaries, not
claims that the two engines expose identical cast microsteps. Dead-source and
dead-target scenarios use declared departure hooks after placement. No Forge,
reference full-game, full observation/mask or complete hidden-state agreement is
claimed. The Python ordinary-discovery comparator test mutates every ledger
field/checkpoint and missing case; live runner controls also prove divergence.

| Unchanged catalog cases | Native/XMage execution |
| --- | --- |
| `rules-triggers-cast-resolution-positive/negative/interaction/regression` | `archer_fodder`, `creature`, `mixed_growth`, `dead_target` |
| `rules-triggers-failed-cast-positive/negative/interaction/regression` | `mixed_growth` plus native exact Cub-target count test, `no_mana`, `bite_reject`, `thrill_fail` |
| `rules-triggers-ordering-positive/interaction/regression` | `mixed_growth`, six `order_*` cases; native restore at every placement/resolution yield retains source order |
| `rules-triggers-ordering-negative` | Native incomplete/duplicate/foreign permutation nonmutation, retained #204 coverage |
| `rules-triggers-apnap-positive` | Explicit synthetic `apnap` in both engines; both life totals checked |
| `rules-triggers-apnap-negative/interaction/regression` | Retained #204 controller rejection/both-active-seat/dead-source tests; source damage now also covered above |
| `rules-foundations_micro_v1-firebrand-archer-positive/negative/interaction/regression` | `archer_fodder`, `land`, `two_growth`, `dead_source` |
| `rules-foundations_micro_v1-crackling-cyclops-positive/negative/interaction/regression` | `cyclops_fodder`, `creature`, `cyclops_growth`, `thrill_fail` |

Fodder and failed-Thrill cases use already-delivered mechanics, so these basic
catalog executions are included here. Broader composed token/Thrill and ETB
interactions remain #209/#211; no ETB detection is added. The crosswalk's original
owners/expectations remain unchanged. Existing instant/departed-target/cleanup
reference checks are also retained as regressions for the shared stack boundary.

## Usage and limitations

Use the existing explicit script/semantic submission interfaces. A committed
cast may return a mandatory `TriggerOrder` decision before priority; submit its
complete controller-local bottom-to-top permutation. Ability objects expose no
creature characteristics. Quantum yields are internal work, never policy actions.
Snapshots/replays use the existing source-fingerprint compatibility policy and
reject incompatible older artifacts.

The current `legal-random-surprise-v1` and `heuristic-surprise-v1` policies retain
their declared seventeen-card subset and explicitly reject new Archer/Cyclops
casts/trigger decisions. Full-pool native policy delivery remains #208. Scripted
core support for these two cards is complete; nineteen cards are supported, and Pyromancer remains unsupported. No workflow, CI, scope registration,
card/rules pins or policy authorization changes are included.

The [executed reference receipt](reference.json) pins the exact fixture,
expectations, bridge, runner, upstream and native source hashes; all 22 cases
agreed twice and four deliberately wrong checkpoint fields were detected.
[Native points](native.json) and [XMage points](xmage.json) retain the observed
first repetition. These are observations compared against the independently
written fixture, not replacement expectations. The unchanged green-mirror
README native quickstart also completed both requested games successfully.
Full torture/review/merge receipts remain in the issue/PR until final delivery.
