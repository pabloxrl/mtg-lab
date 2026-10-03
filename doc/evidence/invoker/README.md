# Wildheart Invoker acceptance

GH-201 enables pinned 2GG 4/3 Wildheart Invoker and its eight-generic targeted
+5/+5 and trample until end of turn. Independent expectations use pinned Oracle
and CR 602, 608.2b, 113.7a, 400.7, 611.2, 514.2, 702.19 and 702.2.
[Compiled behavioral red](red.log) precedes rules implementation. Delivery is
conditional on full torture, separate candidate-bound review, protected merge
and exact-main CI; the issue workpad/PR records those receipts. This is partial
R0002-B010/B011/B029 coverage, not a whole requirement or M2 verdict.

Normal discovery includes core `invoker_tests`, recorder
`invoker_normal_reset_paid_activation_capture_replay`, policy
`invoker_native_policy_cast_activate_pay_finish_and_version_contract`, and Python
`test_invoker_reference.py` comparator mutation tests.

- Eight generic mana permits fresh or tapped Invoker to activate. Own/opposing
  Cubs become 7/7 with trample. Target selection precedes explicit reservation
  of eight individual mana units; mixed colors and colorless work. Finish commits
  all payment atomically. Source death does not cancel the stack ability.
- Seven mana, missing/land target, payment before target, duplicate/stale payment,
  unavailable reserved mana, departed target before commitment, overflow and
  post-concession input reject without partial mutation. Cancellation spends
  nothing. A departed target's returned incarnation receives no boost.
- Pending payment and every resolution work quantum survive snapshots. Semantic
  actions use the existing activation commands and generation checks. Cleanup
  removes the boost, damage and temporary trample without a policy/SBA boundary.
- Normal-reset ordered green decks play twelve Forests, cast Invoker and use its
  ability on its entry turn. Driver/Run proves 9/8 then next-turn 4/3, quantum
  1/unbounded and capture on/off equivalence, semantic replay, typed JSONL
  round trips and once-only concession rewards. Both seats see effective trample
  before combat, and typed records preserve the grant and its cleanup expiration.
  [Observation regression red](view-red.log) reproduces the first independent
  review finding; the existing played test now checks the field explicitly. Rejected target changes during
  payment preserve the complete Driver state, semantic history and capture.
  These played tests are distinct from focused synthetic positions.

## Reference checks

```sh
./scripts/torture.sh
python3 scripts/invoker_reference.py --cache /home/agent/.cache/xmage --output /tmp/invoker-reference
python3 scripts/instant_reference.py --cache /home/agent/.cache/xmage --output /tmp/invoker-instants
```

The shared sixteen-case native/XMage fixture checks own/opposing/mixed-mana
boosts, tapped/sick sources, seven mana, required/illegal target constraints,
source death, target incarnation change, cleanup, repeated Thornweald boosts,
combined Shivan/Thornweald cleanup, Thornweald trample combat and 2GG casting
with shortage. Literal expectations precede engine comparison. The Thornweald
combat case assigns one each to two Sentries and five to P1; all three die.
Existing deathtouch is consumed, not reimplemented. Duplicate trample gives no
additional damage. Both engines compare source/target/Dragon stats, trample,
life, mana, stack and legality; comparator tests mutate every field and omit
every case.

Missing/illegal target reference checks inspect the pinned ability's actual
required cardinality and target filter; native tests submit rejected commands
and compare complete snapshots. Control age and departed/returned positions
are explicit setup hooks; real card costs, abilities, combat and cleanup execute
in each engine. No Forge/full-game agreement is claimed. Upstream license and
provenance remain in [the reference directory](../../../references/xmage/UPSTREAM-LICENSE.txt).

## Catalog, compatibility and support boundaries

[Catalog mapping](catalog.json) preserves every original owner and assertion.
The Invoker positive/negative/interaction/regression cases, stacked Thornweald
boost regression, combined cleanup interaction and Thornweald trample regression
execute here because their prerequisites are now delivered. Future Surprise and
trigger compositions and original #23/#24/#26 audits retain full acceptance.

Native policy IDs become `heuristic-invoker-v1` and `legal-random-invoker-v1`;
old IDs reject. Float mana at priority before activation, choose the target, pay
eight units, then finish. Existing typed target/payment decisions carry the
continuation without new wire commands. Conservative fingerprints reject old
snapshots/replays; no migration is claimed.

Existing unsupported-Invoker tests now use still-unsupported Viashino Pyromancer,
retaining their rejection/nonmutation assertions. Pinned-definition tests add
literal 2GG/4/3 expectations. The passive collector's independent ledger adds
4/3 and an unpaid/masked cast, retaining every field comparison. The independent
review must assess these requirement corrections and README accuracy. No test
is removed, skipped or weakened; no CI/workflow/auth/budget policy changes.
