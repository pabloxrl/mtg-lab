# Shivan Dragon acceptance

GH-200 implements pinned 4RR 5/5 flying Shivan and its nontargeted R activation.
Independent expectations use pinned Oracle and CR 602, 302.6, 113.7a, 400.7,
611.2, 613, 509/510 and 702.2. [Compiled behavioral red](red.log) precedes rules
implementation. Delivery requires full torture, separate candidate-bound review,
protected merge and exact-main CI; the issue workpad/PR records those receipts.
No M2 gate or whole RFC requirement block completion is claimed.

Normal test discovery includes core `shivan_tests`, recorder
`shivan_normal_reset_paid_activation_capture_replay`, policy
`shivan_native_policy_cast_activate_pay_finish_and_version_contract`, and Python
`test_shivan_reference.py` comparator mutation tests.

- Fresh or tapped Shivan can activate; two explicit red payments produce two
  independent nontargeted stack abilities. Each resolution adds one power only;
  7/5 retains flying and cleanup restores 5/5. Growth composes to 9/8 after one
  activation; Bite reads seven after two responses.
- No red, wrong actor/color, missing/duplicate payment, stale decision/source,
  malformed foreign target, arithmetic exhaustion and semantic replacement-source
  submission reject without mutation. Cancellation spends nothing. Concession
  clears pending payment without paying or publishing an ability.
- Source death leaves the stack ability intact. It resolves without affecting a
  returned incarnation. Ability objects disappear rather than entering a graveyard.
  Pending payment and every resolution work quantum restore correctly.
- Normal-reset ordered red decks play eight Mountains, cast Shivan and activate
  twice on its entry turn. Real Driver/Run execution checks 7/5 and next-turn 5/5;
  quantum 1/unbounded, capture on/off, semantic history/replay, typed JSONL round
  trips and once-only concession rewards agree. Invalid target submissions preserve
  the complete Driver state, history and capture. Synthetic tests are separate.

## Reference checks

```sh
./scripts/torture.sh
python3 scripts/shivan_reference.py --cache /home/agent/.cache/xmage --output /tmp/shivan-reference
python3 scripts/instant_reference.py --cache /home/agent/.cache/xmage --output /tmp/shivan-instants
```

The shared 23-case native/XMage fixture uses real pinned cards for repeated boosts,
sickness/cleanup, no mana, foreign-target cardinality, departed source, 4RR casting
and shortage, current-power Bite, Cavalry haste, Sentry/Cub/Thornweald blocks,
Thornweald Bite, every nonnegative two-blocker split totaling five, rejected six
and Growth before unrestricted assignment. Every surviving opposing creature is
compared, as well as Dragon statistics, life, red mana, stack and legality.
Comparator tests mutate every field and omit every case.

Only sickness/departure setups are synthetic reference hooks; the engines execute
real abilities, mana costs, spells, combat and cleanup. Foreign-target rejection is
native executable submission plus the pinned XMage ability's empty target list.
Excess allocation is rejected against XMage's actual allocation bounds. No Forge
or full-game reference agreement is claimed. Upstream license/provenance remains
in [the XMage reference directory](../../../references/xmage/UPSTREAM-LICENSE.txt).
A failed build/run or unavailable reference is never agreement.

## Catalog and scope

The unchanged catalog's Shivan cases and currently executable Shivan compositions
are executed here: mana-shortage/summoning-sickness regressions; flying/reach
positive, negative and interaction; all multiple-blocker cases; resolution-power
interaction; all-combinations and masked-rejection interactions; creature-abilities
interaction; Bite/Thornweald/Sentry card interactions. Native tests additionally
cover new-incarnation and atomic input contracts. The exact retained catalog
inventory is [catalog.json](catalog.json). Surprise/Invoker/trigger compositions
still require their named future mechanics and original #210/#212 composition
owners. No original catalog expectation or ownership changes.

## Compatibility and reviewed boundary corrections

Existing unsupported-Shivan tests now use unsupported Wildheart Invoker, retaining
rejection and nonmutation assertions. Full frozen-definition tests add independent
4RR/5/5 expectations and retain all remaining unsupported cards. Existing modifiers
initialize the new power-only component to zero; no prior assertions are removed.
Independent review must check these requirement corrections and README accuracy.
Snapshots/replays reject old source fingerprints. Typed trajectories add the
`activation_payment` decision using existing pay/finish/cancel command types.
Policy IDs advance to `legal-random-shivan-v1` and `heuristic-shivan-v1`; previous
IDs reject. Red mana is floated at priority before starting the activation; finish
commits the reserved payment atomically. No Invoker, trigger or general layer engine.

## Executed reference and quickstart receipts

[Shivan receipt](reference.json): 23 shared cases agree twice, with four detected
checkpoint mutations. [Existing instant receipt](instant-reference.json): all
11 Growth/Bite/cleanup cases agree twice, with strict script and checkpoint
negative controls. Receipt hashes pin the actual executed bridge, corpus and
native sources. [Native README quickstart](quickstart.jsonl) completes two games
with zero failures, truncations or incomplete episodes.

Bridge setup failures were not counted as agreement: playable-ability queries,
explicit target/blocker aliases and explicit priority passing corrected adapter
assumptions. Bounded payment/priority callbacks diagnose malformed scripts.
Expectations remain rule-derived; the survivor comparison caught and corrected
an initially misidentified grown blocker rather than rebaselining its result.
