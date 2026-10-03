# Goblin Surprise choose-one acceptance

GH-203 adds pinned Goblin Surprise ({2}{R}, instant), with explicit mode 0
(creatures controlled at resolution get +2/+0 until end of turn) or mode 1
(create two red 1/1 Goblins). Independent expectations derive from pinned
Oracle and CR 700.2, 601.2b, 611.2c, 302.6, 510 and 514.
[Compiled behavioral red](red.log) precedes implementation; the
[policy version red](policy-red.log) precedes migration of the supported domain.
Delivery is conditional on full torture, separate candidate-bound review,
protected merge and exact-main CI, recorded in the
[issue workpad](https://github.com/pabloxrl/mtg-lab/issues/203#issuecomment-5969646495).
Partial R0002-B010/B011/B029 only; M2 and original component gates remain open.

## Executable coverage

Normal discovery runs core casting `surprise_*`, recorder
`surprise_normal_reset_modes_capture_replay`, policy
`surprise_policy_modes_and_previous_version_rejection`, and Python
`test_surprise_reference.py`.

- Literal controlled Cub/Goblin powers become 4/2 and 3/1, including a token
  entering between cast and resolution. Opposing creatures stay unchanged;
  creatures entering afterward stay unboosted. Cleanup expires the effect.
- Choose-one rejects zero, both, unknown, repeated, stale and wrong-seat modes
  without mutation or spending. Mode precedes payment. Both modes combine with
  every two-color generic payment and every three-of-four physical mana-source
  selection. Cancellation refunds provisional mana and taps and clears choices.
- Actor-only pending mode and public stack mode are explicit. Snapshots restore
  before/after mode selection, throughout payment and at every bounded resolution
  yield, including the immediate first yield. Full/quantum-one results agree.
- Real ordered red-deck reset plays lands, resolves Fodder, then casts either
  Surprise mode. Existing Driver/Run proves capture on/off and quantum equality,
  rejected-command state/history/capture nonmutation, semantic replay, typed
  JSONL round trips and once-only terminal rewards. No synthetic hook replaces
  this normal-reset evidence.

`begin_cast` starts the existing private cast continuation;
`choose_cast_mode(actor, decision, &[mode])` selects exactly one mode. Policy,
semantic action and typed trajectory use `Mode { mode }` at `cast_mode`.
`CancelPayment` cancels and `FinishPayment` commits the chosen mode with mana.
Both players see `StackSpell.mode` after commit; only the caster sees
`PendingSpell.mode` before commit. Resolution reuses existing modification,
token, cleanup and bounded-work paths. Source fingerprints reject incompatible
snapshots/replays. Native policy IDs now end in `surprise-v1`; prior IDs reject.
The heuristic prefers tokens; random retains both modes and cancellation.

## Independent reference execution

```sh
./scripts/torture.sh
python3 scripts/surprise_reference.py --cache /home/agent/.cache/xmage --output /tmp/surprise-reference
python3 scripts/instant_reference.py --cache /home/agent/.cache/xmage --output /tmp/surprise-instants
```

Seven native/XMage scenarios compare independently specified exact mode,
creature controller/name/power/toughness/damage and life ledgers. Checkpoints
cover resolution, subsequent Fodder tokens, combat and cleanup. Cases: Cub and
two Goblins boosted; token mode with existing unboosted creatures; later Fodder;
Growth plus Shivan activation plus Surprise producing 11/8; instant token
blocking; opponent-end-step tokens attacking next turn; boosted Swab trading
with Sentry. Both engines execute each case twice; comparator mutations must
fail. Synthetic setup is explicit; actual pinned XMage spell/continuous/token/
combat/cleanup code executes. Negative mode counts and cancellation are native
API contracts, not claims about an identical XMage input protocol. No Forge or
full-game reference agreement claimed.

[Catalog allocation](catalog.json) retains every original owner and expectation.
All cases using this mechanic and delivered prerequisites execute here; later
composition packs and #23/#24/#26 retain aggregate acceptance. No trigger
framework, new token rule, CI/workflow/auth change or task registration.

The former unsupported-card enumeration now recognizes Surprise with a literal
pinned {2}{R} expectation; remaining unsupported cards retain rejection tests.
Separate review must assess this requirement correction and stronger modal
coverage, README accuracy and policy version migration. No tests are removed,
skipped or weakened. Earlier bridge API/fixture sequencing errors were repaired
without altering the independent expected outcomes.

[Seven-case executed receipt](reference.json) records two runs per engine and
negative comparator controls. The [updated native quickstart](quickstart.json)
completed two games with zero failed, truncated or incomplete episodes. The
first retained instant regression run encountered an H2 database lock while
another XMage suite used the same cache; it is not a pass. The retry runs
sequentially without changing any test or expected outcome.
