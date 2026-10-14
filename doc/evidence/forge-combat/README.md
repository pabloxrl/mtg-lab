# Full Forge combat replay (L0-010)

`--abilities --full-game` extends the existing strict Forge consumer through
combat declarations, exact damage allocation, simultaneous deaths, ordered
graveyards, cleanup and a rules-terminal outcome. Prefix modes retain their
existing boundaries. No deck, card definition, source pin or adjudication changes.
This bounded delivery is not pilot qualification or corpus admission and earns
zero AI quota. Protected delivery evidence belongs in the
[issue workpad](https://github.com/pabloxrl/mtg-lab/issues/328#issuecomment-6169649924).

## Reproduction

Use the existing [private-cache preparation](../forge-ai-capture/README.md), the
pinned XMage cache, and new output directories. In Symphony run heavy commands
under its shared lock:

```sh
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- \
  python3 scripts/forge_casting.py --cache /home/agent/.cache/forge-GH-328 \
  --trace doc/evidence/forge-ai-capture/acceptance/game/result.json.jsonl.gz \
  --abilities --full-game --output .agent-artifacts/forge-full-example

python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- \
  python3 scripts/forge_combat_reference.py \
  --fixture .agent-artifacts/forge-full-example/native-fixture.json \
  --cache "$MTG_REFERENCE_CACHE" --output .agent-artifacts/forge-full-xmage-example
```

The first command validates the complete immutable capture receipt and executes
Forge and native. The second executes native and the pinned, explicitly patched
XMage admission consumer. It is a bounded verifier, not a corpus-ingestion tool.
Missing callbacks, observations, terminal outcomes or unconsumed inputs fail;
full mode cannot select an earlier prefix endpoint. The four new live mutations
are `--negative combat-allocation`, `graveyard-order`, `expired-effect`, and
`terminal-reason` on the first command.

## Independent expectations

The existing red/green seed-zero capture starts green, includes two red
mulligans, and ends on turn 29 with red losing to nonpositive life. It contains
13,676 source records and 7,022 random results. Its translation supplies 818
actions, 610 priority checkpoints, 13 simultaneous-damage observations and 28
cleanup observations. These expectations come from the immutable source and the
frozen card text/CR, never from native output.

| Behavior | Independent basis and witness |
| --- | --- |
| Declarations and sickness | CR 302.6/508/509. Actual attackers/blockers retain physical identities, incarnation, controller and blocked status. Empty blocks require a witnessed source declaration event. |
| Exact assignment | CR 510.1. At source record 5535, printed five-power Pathwarden assigns four to Cyclops and one to a Goblin. Forced single-recipient nontrample assignments are validated without inventing a native choice. |
| Trample/deathtouch/vigilance | CR 702.19/702.2/702.20. Actual Pathwarden/Thornweald combats retain each allocation, simultaneous marks, deaths and tapping. Trample to a player requires lethal on each blocker. |
| Flying/reach/haste gaps | CR 702.9/702.17/702.10. An explicitly synthetic Forge case rejects Bear Cub blocking Shivan, permits each printed reach creature, rejects a newly entered Pathwarden attacking, then pays Cavalry's tap cost and resolves its printed targeted haste activation before accepting the attack. |
| Simultaneous SBA and tokens | CR 704/111.6/404.3. All observed simultaneous damage and ordered persistent deaths are compared. Tokens participate in the move event and disappear; they occupy no persistent graveyard position. |
| Cleanup | CR 514.2. All 28 source cleanups have zero marked damage and expired temporary boosts. A synthetic nine-card hand separately exercises CR 514.1/404.3: exactly two distinct current hand cards are discarded in the chosen order (reversing hand order), and seven remain next turn; duplicate, short and out-of-hand selections fail. |
| Terminal reason | CR 704.5a. Red's recorded life is nonpositive and its loss is life loss, not an empty draw, concession or timeout. |

The existing native/XMage modern multiple-blocker, haste, flying/reach, cleanup,
terminal and sixteen admission regressions remain in normal discovery. The new
source-derived full-game fixture also runs in ordinary native discovery, twice,
with changed allocation, graveyard order, expired effect and terminal reason
controls. All authored cases remain separate from actual-AI provenance.

## Source scheduling and representation

Pinned Forge stores attacking bands in an identity-hashed multimap. Its
`Combat.getAllBlockers()` iteration drives both event context and blocker damage
callback order. A narrow, hashed replay-only integration seam orders the
independently computed blocker membership by the original `BlockersDeclared`
schedule. It cannot introduce a blocker: duplicate schedules or unequal
membership fail. Every resulting callback and rules observation is still checked
in original order. This is a scheduling alignment, not an upstream rules
adjudication. Whole-source pin validation recognizes only the exact original or
this declared patch; capture pins and original traces remain unchanged.

Forge's damage result is a map. Its iteration order is not the modern CR 510
assignment order. Translation binds each amount to its physical recipient and
emits recipient slots in the recorded blocker-list order required by the existing
XMage interface. It never redistributes damage. Existing
[allocation/identity exceptions](../forge-abilities/README.md#identity-and-comparison-limits)
remain explicit. Native repeated-run comparison canonically names process-local
ObjectStore IDs while preserving their equality relationships, epochs, slots and
generations; original raw handles remain in execution outputs.

Forge checks every source event/boundary state, including intermediate zone
departures. Consequently reversing a graveyard-order choice first fails at the
changed battlefield departure before the final graveyard is populated. The first
semantic error is preserved across later upstream unwinding callbacks. Native
checks source-derived priority, damage, cleanup and terminal observations;
XMage agreement additionally requires exact consumption and aligned observations.
No observed state is installed into another engine to manufacture agreement.

The shared observation checker previously reapplied every London bottom choice
after the final shuffle. This game independently reproduced its false rejection:
red's first-round Swab was reshuffled, while only the second-round Mountain and
Shivan define the final bottom order. CR 103.5 requires that later shuffle to
replace the earlier ordering. The checker now applies only bottom choices from
the final round. A new behavioral regression first reproduced the rejection and
also requires a reversed final bottom order to fail. No existing observation or
regression was removed; this requirement correction is included in independent
review.
The concurrent protected pilot delivery in PR #350 supplied an equivalent
final-round correction. Integration retains its observer implementation unchanged
and keeps this Forge-source regression as additional independent coverage.

## Executed acceptance and retained attempts

The [final Forge/native receipt](execution/combat-live-integrated.json) reports
`full-game-agreed`, including the four retained APNAP cases and the new authored
combat and cleanup cases. The [final native/XMage receipt](execution/combat-cross-integrated.json)
reports `native-xmage-agreed` for both reference executions at all 610 priority,
13 damage and 28 cleanup checkpoints. Native executes the fixture twice and
rejects all four mutations in normal discovery. The
[execution summary](execution/summary.json) keeps positive and negative results
distinct; the passing live graveyard-order control is attempt 3, while its first
two diagnostic-expectation failures remain failed attempts.

The [post-integration receipt](execution/combat-cross-integrated.json) repeats
native/XMage execution after incorporating protected main `a21b248d6ee2a8be69026a84b0f15b3dbabf6ec6`.
Its updated native observer and trigger-provenance checks pass. The
[integrated Forge/native run](execution/combat-live-integrated.json) also checks
the stronger authored cleanup-order assertion while retaining exact source replay.
The six Forge tests and all 24 newly delivered pilot tests also pass, including
fresh twice-native replay of the ten provisional games. Their ownership and
provisional status remain with the pilot delivery.

Local validation includes all 334 core tests (including the sixteen admission
games and retained Forge prefixes), and 27 Python opening/casting/ability/combat
tests, plus all 27 existing admission checks (54 Python tests total). The
[validation record](validation.json) records focused checks without
treating shared-container timings as latency qualification. Complete mandatory PR
CI supplies final-candidate suite evidence under the approved #348 amendment;
independent exact-candidate review, protected merge and exact-main CI are recorded
in the delivery workpad. These execution receipts do not replace those gates.

The [attempt index](attempt-index.json) preserves every retained consumer attempt,
including original failures, in content-addressed gzip objects. Each entry records
the original relative path, byte count, uncompressed SHA-256 and compressed SHA-256.
To recover a file, find its entry and decompress its `storage` path; check the
result against its `sha256` and `bytes`. Identical original files share one object.
No private cache is needed to reconstruct this published evidence. Original
source, configuration and capture receipt remain in the unchanged
[capture bundle](../forge-ai-capture/README.md); no new AI games were captured.
Upstream installations and binaries stay outside this bundle under the existing
[provenance policy](../../provenance-policy.md). Corpus admission and AI quota are
both zero.
