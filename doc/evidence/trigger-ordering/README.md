# Pending trigger placement acceptance

GH-204 adds a typed pending queue and owned APNAP placement to the existing
Game executor. CR 603.3b and 101.4 require each controller's complete local
bottom-to-top order, active player first. CR 117.5 requires mandatory settlement
before priority. A pending ability survives its source's departure.

All trigger declarations in this delivery are **test-only synthetic fixtures**.
No production trigger kind or card detection is enabled. The empty production
`TriggerKind` is intentional; real cast and ETB sources belong to #205/#206.
This report does not certify real card casts, whole RFC blocks, or M2 completion.
Delivery is conditional on separate review, protected merge and exact-main CI;
the issue workpad and PR carry these receipts.

## Executable checks

Normal discovery runs `game::trigger_tests`, recorder
`typed_trigger_order_preserves_identity_and_order`, and
`synthetic_trigger_order_roundtrip_and_invalid_permutations`.

- Two triggers per seat, both active seats, literal orders `[11,10,20,21]` and
  `[21,20,10,11]`; quantum one and unbounded, restore at pending decisions and
  every internal placement yield. Dead sources remain represented.
- Missing, duplicate, cross-controller, wrong-seat and stale orders reject
  without snapshot mutation. Policy capacity one rejects two candidates.
  Exhausting the second stack object's identity rejects the whole batch.
- Explicit synthetic source-death work runs before placement; a terminal loss
  settles before any trigger choice or priority. Synthetic resolution removes
  the ability rather than moving its source-card representation to battlefield.
  Public observations and captured records give abilities no source creature
  characteristics or printed trample; [review regression red](observation-red.log)
  and [keyword red](keyword-red.log) precede the fixes.
- Existing Driver captures both ordered submissions, rejects wrong-seat/stale
  submissions, retains semantic history, and delivers concession rewards once.
  Reapplying history from the declared initial synthetic snapshot preserves order.
  This is snapshot-based component replay, not normal-reset played evidence.
- Typed recorder conversion preserves trigger row identity and order; JSONL
  round trips and malformed permutations are checked against literal commands.
- Restore rejects duplicated placement work with a valid recomputed envelope
  checksum, without replacing the original state. Trigger-order decisions also
  require their pending controller and saved return boundary; malformed decisions
  reject before restore (see [behavioral red](decision-red.log)).

The resumed checkout's original two tests already passed. Added tests exposed
behavioral failures before repairs: synthetic resolution increased battlefield
count from four to five; duplicate placement work restored successfully instead
of returning `Corrupt`. These assertions use CR ability lifetime and the owned
continuation integrity contract, not engine-generated expectations. Initial
exhaustive-match compilation failures were not counted as behavioral red.

## Catalog and reference boundary

Unchanged `rules-triggers-ordering-negative`, `rules-triggers-apnap-negative`,
`rules-triggers-apnap-interaction` and `rules-triggers-apnap-regression` are
covered by the named omission/rejection, both-seat order and dead-source tests.
Archer/Cyclops detection and effect cases retain #205 and composition pack #211
prerequisites, including the literal Archer APNAP positive. Synthetic tags do
not replace those real ability assertions. Original owner #23 and gate #26
retain full acceptance. No basic real-card trigger execution is claimed or
silently deferred: production trigger detection is explicitly excluded here.

Existing instant/target/death/cleanup native/XMage comparisons are applicable
regressions for the modified stack and priority boundary. Their execution
receipt, full torture, and separate review are recorded in the PR/workpad.
No new native/XMage trigger agreement or Forge agreement is claimed.

## Usage and limits

At a `TriggerOrder` decision, submit the controller's complete permutation via
`order_triggers_quantum`, policy `OrderTrigger`, or semantic `order_trigger`.
Other controllers' rows are not candidates. Stack views mark placed objects as
abilities. Trigger injection is compiled only into unit tests. There is no new
CLI command or playable card. Passive `pass-v1` explicitly rejects unexpected
trigger choices. Real-source policy support remains #208's acceptance.

Snapshot fingerprints include the new implementation; older incompatible
snapshots/replays reject explicitly. Existing schemas gain the typed command;
no legacy reader compatibility or snapshot migration is promised. Existing
regressions remain intact; exhaustive test scripts add explicit rejection for
unexpected trigger decisions rather than silently passing.
