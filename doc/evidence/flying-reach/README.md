# Flying/reach and Magnigoth Sentry acceptance

GH-196 implements CR 509, 702.9b and 702.17 blocker restrictions through the
shared scalar and policy paths. Pinned Sentry is 3G, 4/4, reach. Shivan Dragon
remains unsupported; flying Cub fixtures are explicitly synthetic and test-only.
No deathtouch, trample or activation support is implied. M2 remains incomplete.

## Executable evidence

The retained [combat behavioral red](red.log) and [casting behavioral red](cast-red.log)
fail compiled assertions before implementation. Ordinary test discovery includes:

- `flying_reach_illegal_cub_block_is_atomic`: raw illegal block preserves snapshot.
- `flying_reach_sentry_blocks_five_five_and_dies`: flyer survives with four damage,
  Sentry dies, players retain 20 life.
- `flying_reach_sentry_attacking_is_blockable_by_cub`: reach is not evasion.
- `flying_reach_pair_matrix_policy_actions_snapshot_and_quantum`: both seats,
  flying/flying and flying/reach, tapped rejection, sickness permits blocking,
  mixed ground/flying attackers, semantic actions, snapshot and bounded work.
- `flying_reach_sentry_pinned_cost_casts_for_three_generic_one_green`: literal 3G.
- `flying_reach_normal_reset_sentry_cast_combat_capture_replay`: normal-reset
  ordered green decks, real land payments and Sentry combat against Cub,
  quantum 1/unbounded and capture on/off history equivalence, typed conversion,
  JSONL round trip, authorized replay and once-only concession rewards.
- Policy tests retain ordinary ground choices while excluding forbidden pairs.

Expected outcomes derive from the pinned card manifest and CR, not engine output.
The [catalog inventory](catalog.json) retains original requirements unchanged.
Real Shivan, Thornweald, Surprise and Invoker compositions remain assigned to
#210/#212 after their mechanics land; synthetic flyer checks do not certify those
full-card scenarios. Ground Sentry/Cub regression executes here. Existing Bite,
Growth, damage bookkeeping and cleanup cases are retained in the instant suite.

## Reproduction and scope

```sh
./scripts/torture.sh
python3 scripts/flying_reach_reference.py --cache /home/agent/.cache/xmage --output /tmp/flying-reach-reference
```

The shared six-case fixture covers reach combat, Cub rejection, tapped Sentry,
ground Sentry, Growth after blocking and real 3G Sentry casting. The original
XMage bridge uses the pinned harness and card definitions, with strict scripted
choices and a test-only 5/5 flying Cub (three +1/+1 counters in XMage; a
synthetic +3/+3 modifier in native). No counter gameplay capability is claimed. Comparisons cover power/toughness/damage,
creature presence, player life and block legality. They do not claim full-state,
Forge, full-game or full Shivan agreement. Native rejection additionally checks
nonmutation. Upstream harness provenance and MIT notice remain in
[references/xmage](../../../references/xmage/UPSTREAM-LICENSE.txt).

README and policy documentation now describe ten supported cards and new policy
IDs. No stage verdict changes. The [pinned reference receipt](reference.json) records six cases agreeing twice
and four detected checkpoint mutations. Earlier fixture-layer setup failures were
not counted as agreement; expected outcomes were unchanged. Full-suite and review
results are recorded in the issue workpad and PR before delivery.

## Existing boundary-test correction

The turn-boundary unsupported-combat regression previously used Sentry. With
GH-196 authorizing its complete reach behavior, that premise is obsolete; it
now uses still-unsupported Shivan and retains the exact error/nonmutation checks.
The former card-definition rejection is replaced by pinned-cost/stats, raw/policy
legality, real played casting/combat and reference coverage above. Independent
review must assess these requirement corrections and stronger replacement coverage.

The unchanged [eleven-case native/XMage instant receipt](instant-reference.json)
confirms Growth/Bite/cleanup and damage bookkeeping twice, with strict negative
controls. The native README quickstart completes two games, with zero failed,
truncated or incomplete episodes. Broader reference composition remains pending.
