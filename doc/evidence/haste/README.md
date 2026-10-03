# Axgard Cavalry haste acceptance

GH-197 implements the pinned 1R 2/2 Cavalry and its tap-cost targeted haste ability.
Independent expectations use pinned Oracle and CR 602, 302.6, 702.10, 113.7a,
608.2b, 611.2 and 514.2. [Compiled behavioral red](red.log) precedes implementation.
The full regression suite, separate review and exact-main delivery receipts belong
in the issue workpad and PR; this report does not certify the M2 gate.

Normal discovery includes `haste_tests` in mtg-core and
`haste_normal_reset_cavalry_activation_capture_replay` in mtg-recorder:

- Printed cost/body, fresh/tapped/wrong-controller/dead-source rejection, cancelling
  without tap, missing/stale/illegal targets, malformed semantic cost, exhausted
  object identity and policy capacity preserve state.
- Own/opposing fresh creatures gain haste only after resolution. Source death,
  including an actual Bite Down response, leaves the ability on stack; target
  departure or return never redirects the grant. Cleanup removes haste.
- Pending selection and every resolution work quantum restore; one-unit and
  unbounded execution agree. Ability objects are removed, never graveyard cards.
- Normal-reset ordered red decks cast Cavalry on turn 3 and Swab on turn 5 using
  actual Mountain payments. The freshly cast Swab attacks for two after the
  activation; turn-6 cleanup has removed haste. Driver/Run capture on/off and
  quantum 1/unbounded give equal semantic histories, replay, typed conversion,
  JSONL round trips, and once-only concession rewards. Invalid activation targets
  preserve Driver state, RNG, history and capture.

Synthetic positions are explicitly test-only. They do not substitute for that
normal-reset played evidence. No creature-mana implementation or general trigger
framework is added; Elf/Druid haste composition remains assigned to #209 per the
issue exclusion. Original #23/#24 and #26 retain complete acceptance.

## Reference reproduction

```sh
./scripts/torture.sh
python3 scripts/haste_reference.py --cache /home/agent/.cache/xmage --output /tmp/haste-reference
```

Ten original shared native/XMage cases cover Cub/Swab haste attacks, opposing Cub
through cleanup, own cleanup, fresh and tapped Cavalry, source/target death,
activation after source death, and actual 1R Cavalry casting. The bridge uses
synthetic control-age/departure setup hooks; activation, target selection, tap cost,
resolution, combat and cleanup run in the pinned engine. Checkpoints include life,
source presence/body/tap, target presence, haste and stack size. Comparator tests
mutate every field and omit every case. No Forge/full-game reference claim.
Upstream provenance and MIT notice remain in [references/xmage](../../../references/xmage/UPSTREAM-LICENSE.txt).

## Existing boundary-test correction

The former unsupported-Cavalry casting and heuristic assertions now use unsupported Shivan and
retains rejection/nonmutation. The all-frozen-card support test adds independently
pinned Cavalry cost/body checks; all remaining unsupported cards still reject.
The replacement coverage above includes activation, normal play and references;
`cavalry_policy_cast_activation_target_finish_and_version_contract` additionally
checks native policy choices and rejection of superseded policy IDs.
Independent review must explicitly assess this requirement correction and README
accuracy. Source-fingerprint compatibility rejects older snapshots/replays; policy
IDs advance to `legal-random-haste-v1` and `heuristic-haste-v1`. Existing wire fields
retain their meaning; new haste/ability flags default false when absent.

## Unchanged catalog crosswalk

[Original catalog entries](catalog.json) retain their owners and expectations.
The following case suffixes are executed here:

| Capability | Cases | Executable evidence |
| --- | --- | --- |
| `costs/summoning-sickness` | negative | fresh Cavalry raw/policy rejection; reference `sick` |
| `combat/haste-vigilance` | positive, regression | normal-reset Swab attack; reference `swab`/`cleanup` |
| `objects/dead-source` | negative, interaction | raw stale-source rejection, actual Bite response; reference `dead_before`/`source_dies` |
| `combat/creature-abilities` | regression | second activation rejection; reference `tapped` |
| `foundations_micro_v1/axgard-cavalry` | positive, negative, regression | normal-reset Swab and fresh source tests; reference `swab`/`sick`/`opposing` |

The summoning-sickness positive old Cub attack is retained by existing combat
regressions. Shivan non-tap/stack abilities and Tajuru vigilance cases require their
future mechanics (#198/#200/#210). Elf haste-mana and Druid payment composition
remain #209 as explicitly excluded here; #195 already owns their mana rules.
Source/target generic identity cases remain covered by existing target tests plus
this issue's departed/replacement-target tests. No catalog case is marked skipped
or changed; later composition packs retain their original crosswalk obligations.

[Final haste reference receipt](reference.json) records ten cases agreeing twice
with four detected checkpoint mutations. [Existing instant-response receipt](instant-reference.json)
records eleven Growth/Bite/cleanup cases agreeing twice and strict negative controls.
The updated native quickstart completed both games with zero failures, truncations
or incomplete episodes. All reference compilation/setup failures were repaired;
none counted as agreement.

## Independent review correction

Review of initial candidate `b73eaee06806ada71f95dce71e713410e010b3da`
found that concession left the new activation continuation alive. Both the
[scalar red](terminal-red.log) and [normal-reset Driver red](terminal-driver-red.log)
compile and fail terminal-observation assertions before the fix. Termination now
clears activation alongside casting continuations, and raw activation validation
rejects terminal state. Tests cover concession before/after target selection,
both conceding seats, unchanged unpaid tap cost, terminal observations, subsequent
command nonmutation, capture on/off, quantum equivalence and replay/JSONL.

The lifecycle audit distinguishes reset/termination (clear pending choice) from
budget truncation (retain the genuine resumable position). Full torture and
candidate-bound independent re-review are required after this correction; their
final results are recorded in the PR/workpad. The original review and resolution
remain preserved there rather than being overwritten by a later pass.
