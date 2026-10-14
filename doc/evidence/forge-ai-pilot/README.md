# Forge AI pilot (L0-012)

This report accompanies the ten-game pilot manifest. It is not final corpus
admission, a 100-attempt throughput campaign, a 500-source quota, or completion
of L0. The frozen forty-card decks, twenty nontoken definitions and Goblin token
are unchanged. Existing M3 holds and source-specific exceptions remain in force.

## Reproduction

Verify retained source-bound inputs and actual execution receipts:

```sh
python3 scripts/check_forge_pilot.py
```

Replay every selected game twice in native, including every required source
checkpoint, through the ordinary regression consumer. Fresh native observations
are also compared against the retained independent reference observations:

```sh
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- \
  python3 scripts/check_forge_pilot.py --native
```

For fresh three-engine qualification of an original trace, prepare private
pinned Forge and XMage caches using their existing documented commands, then:

```sh
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- \
  python3 scripts/forge_ai_pilot.py \
  --source doc/evidence/forge-ai-capture/acceptance/game/result.json.jsonl.gz \
  --forge-cache /home/agent/.cache/forge-GH-330 \
  --xmage-cache "$MTG_REFERENCE_CACHE" \
  --output .agent-artifacts/forge-pilot-example
```

The output directory must be new. This command qualifies one candidate, not an
entire pilot or a corpus admission. Each invocation executes two Forge replays,
three native invocations (each internally replays twice), and two patched-XMage replays.
All eight opening checkpoints and gameplay, simultaneous-damage, cleanup and
terminal observations remain required. Neither wrapper nor packager installs
checkpoint state into an engine. Expectations come from immutable source
observations and the existing independently reviewed rules contracts.

## Identity and evidence

`pilot.json` lists ten canonical content identities and all eight matchup/starting
seat rows. Physical copies, incarnations, chronological choices and chance are
significant. Seed, source path, runtime allocation IDs and diagnostic labels
cannot create a new game. `prior_all_three_overlap` explicitly identifies reuse
of the earlier one-game Forge normalization witness. Prior XMage pilot identities
are checked for overlap; no identity can fill both source quotas. Replays,
authored controls and unsuccessful attempts contribute zero new games.

Original capture traces and receipts retain their exact bytes. Existing captures
are referenced at their original repository paths; new captures are under
`sources/`. Compressed semantic outputs decompress to the exact live-output
checksums. `attempt-index.json` records original attempt paths, raw byte counts
and SHA-256, and content-addressed gzip storage with its own SHA-256. Equal bytes
share storage only. Decompress and verify both hashes to recover any original.
Later candidate seeds target missing starting-seat rows using the unchanged
Java Random opening sequence, checked against thirty actual captured prefixes.
This is seed selection only: no AI choice, shuffle or game outcome is overridden,
and the recorded starter remains authoritative. The pilot is an adaptive
qualification sample, not a throughput campaign or an unbiased statistical sample.

Failed captures, initial normalization failures, rejected native/reference runs,
and successful executions superseded by a consumer repair remain visible.

The manifest pins the current replay and validation consumers. Each original
capture receipt separately pins its source AI, capture runner and instrumentation;
child replay receipts retain toolchain/dependency, bridge/patch, native-source and
card/rules metadata. Historical capture pins are not replaced by current validator
hashes.
A changed consumer requires fresh qualification evidence. The full pilot native
replay is part of normal Python test discovery. Complete mandatory PR CI and
exact-main CI remain separate delivery gates, linked from the issue workpad.

## Bounded repairs and independent basis

CR602.2b invokes activation payment under CR601.2h. A Shivan Dragon activation
costs one red mana each time. The pinned Forge `SpellAbility` retains historical
paying-ability entries across reuse while `clearManaPaid` clears spent mana lots;
`ManaPool` appends the new ability. In the unchanged RR seed 2 trace, activation
13726 taps physical Mountain 56 at event 13741 and spends one red lot from ability
163. Ability 160 / Mountain 49 belongs to the prior activation. The translator
now verifies the old identity prefix and consumes only the newly appended
sources. The regression rejects altered history and missing fresh payment;
unsupported floating or multi-unit mana still fails. Original red/green logs
and pinned-source inspection hashes remain in the attempt archive.

The full-game Forge bridge now enables the already guarded ability-allocation
binding used by ability prefixes. Runtime ability numbers differ across fresh
processes; physical source, controller, API and bijection must still match.
The original GR seed 0 failure at `/forge/5357/state/stack/0/id` and subsequent
live repeated success establish the regression. No source AI or game rule is
changed, and no card identity/incarnation check is relaxed.

Some source games stop on player loss before the settlement boundary required by
the native consumer: lethal-damaged permanents remain in the terminal snapshot,
or a required graveyard ordering choice is absent. These are rejected boundary
alignment attempts, not passing adjudication exceptions or claims of an upstream
rules defect. CR704.3/704.5g supply the settlement requirement; missing choices
are never inferred and expected terminal states are never rewritten. An existing
native identity lookup failure in retained RG seed 4 is likewise excluded.
RG seed 7 passes native and Forge but is excluded because the pinned patched
XMage consumer rejects `/play/359 /unsupported priority callback`; RG seed 31
reproduces it at `/play/555`. In both cases a five-power Pathwarden faces one
undamaged five-toughness Dragon: upstream automatically assigns all five damage
and supplies no selection callback. The current adapter has no automatic-assignment
witness. No callback is invented and no expected checkpoint is removed. None of
these failures is converted into agreement or an admitted exception.

The six existing live corruption controls (copy, mode, target incarnation,
chance, payment and checkpoint) remain mandatory, with their precise original
rejection reasons. Pinned Forge scheduling/allocation alignment and the explicit
XMage combat/SBA patch retain the scope and notices in the
[full-game report](../forge-combat/README.md) and
[normalization report](../forge-ai-normalize/README.md). No new adjudication
exception is introduced. Source-engine agreement does not replace independent
rules review or clear the repository's conservative public-release holds.

## Local validation

The selected ten identities cover all eight rows. The full ledger preserves 44
new capture attempts, 26 execution attempts and 14 normalization outcomes; these
84 phase records are not 84 distinct games. Thirteen distinct games passed all
three engines; ten are selected, three extra successes remain outside the pilot,
and two repeated successful executions remain visible separately.

[Validation receipt](validation/validation.json) pins the pilot manifest and
changed consumers. All 773 unique archived objects were decompressed and checked
against their original hashes (1,007 indexed files). Eight focused accounting,
payment and evidence-corruption checks passed, followed by the documented full
native pilot command and complete provisional L0 cold/warm runs (ten successes,
zero failures each). The native pilot check is also in normal test discovery.
Earlier affected Forge/native regressions and all six live source corruption
controls are retained in the attempt archive. Protected delivery remains
conditional on separate exact-candidate review, mandatory PR CI and exact-main CI.
