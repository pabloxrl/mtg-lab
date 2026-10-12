# XMage real-AI raw capture — L0-001

This bounded command runs the pinned `mage.player.ai.ComputerPlayer7` for both
seats with the unchanged red and green 40-card decks, London mulligans, twenty
starting life and seven-card opening hands. It records live source choices and
observations from normal reset through a rules ending. It never invokes
`TestComputerPlayer`, substitutes a scripted policy, or enables test-mode mulligan
suppression. A safety limit produces a failed/truncated attempt, never a win or draw.

Raw capture is **not replay admission**. No native or Forge replay, normalization,
pilot quota, all-card exercised coverage, large corpus or speedup is claimed.
The reference here is unmodified upstream rules with observation subclasses;
the existing `patched-combat-sba-v1` admission consumer, graveyard-order/trample
limitations and every previous regression remain separate and unchanged.

## Run

Use the pinned managed Linux ARM64 Java/Maven image and an issue-specific external
XMage cache. Prepare/build it once using the existing pinned runner. Under Symphony,
all of these reference commands require the shared heavy-work lock:

```sh
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- \
  python3 scripts/xmage.py prepare --cache "$MTG_REFERENCE_CACHE"
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- \
  python3 scripts/xmage.py build --cache "$MTG_REFERENCE_CACHE"
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- \
  python3 scripts/xmage_ai_capture.py --cache "$MTG_REFERENCE_CACHE" \
  --output .agent-artifacts/my-raw-game --seed 319
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- \
  python3 scripts/xmage_ai_capture.py --cache "$MTG_REFERENCE_CACHE" \
  --output .agent-artifacts/my-capture-acceptance --seed 319 --acceptance
```

Each output directory must be new. `--decks red green` and `--starter 0` are the
defaults; both deck identifiers and either starter are accepted. The bounded
acceptance command runs a real terminal game plus actual unsupported-callback
and record-limit failures, retaining every seed, result and artifact hash.
It checks the specific intended failure reasons, not just a nonzero exit status.
`--timeout` defaults to 1,800 seconds and `--max-records` to 100,000.

`attempt.json` is written before validation/build, then finalized with status,
configuration, source/toolchain/bridge/deck/rules pins, failure reason, elapsed
time and artifact hashes. A killed process leaves an explicit started/incomplete
attempt. Output is privileged and its directory is private. Existing output is
never overwritten. Missing source, unsupported observations and recording errors
fail visibly. Raw logs may contain upstream text and remain local under the
[provenance policy](../../provenance-policy.md).

## Trace contract

`trace.jsonl.xz` preserves the exact original JSONL bytes. The writer streams gzip
during play; finalization losslessly transcodes to XZ and verifies the restored
byte hash before removing the temporary codec. `trace_content` pins the
uncompressed bytes. Incomplete compression retains its original fragment.

Records have monotonic sequence, callback/event kind, actor, source-native detail,
an explicit simulation flag and a semantic observation. Calls and returns retain
nested chronology, arguments and selected results. Source player methods delegate
to the pinned AI, including actual searched actions, passes, targets, modes,
mana/payment, triggers, attacks, blocks and damage allocations. Unsupported
argument representations fail instead of silently disappearing. Human prompt and
effect-text labels are hashed; they are not used as rules or choice identifiers.
No opaque Java serialization or full Oracle/rules document is published.

Observations retain turn/step/active/priority/choosing actors; life, ordered hand,
library and graveyard IDs; mana and source lots; physical card/owner bindings,
source zone-change counters, effective characteristics and abilities; token
identities; stack abilities/targets/modes with source mode-to-effect bindings; pending triggers; combat groups; exile;
and source-native active-effect fields, including numeric parameters, target
pointers, timestamps and affected-object bindings. Target records retain the
source's remembered incarnation counters. Zone events retain source ability and
from/to context. Observations surround actual events, priority callbacks and
resolutions, with settled SBA and terminal boundaries. Source errors and
unsupported terminal reasons cannot fill the normal-game requirement.

Only the live **object instance** can emit records: copied games retain UUIDs,
so UUID equality alone is insufficient. A direct same-UUID copy probe checks this.
The recording equivalence control repeats a genuine first-main-phase AI search
from the same source state/IDs/RNG seed with recording off and on. Upstream
`ComputerPlayer6.copy` shares its action-history cache and omits some search
configuration; the isolated control copies explicitly detach that cache and
restore the declared skill-1 limits (depth 4, 5,000 nodes, three-second think cap).
The production source game's copy behavior is unchanged. Both control return
values and full semantic states must match, and off emits no records. A disabled-observer
probe also verifies that recording off does not inspect callback payloads.
This is controlled decision equivalence, not a claim that unrestricted timed AI
search or random UUID generation reproduces complete games from a seed alone.

## Evidence and limits

The [final live acceptance](acceptance-005/acceptance.json) passed all three real
source invocations. Its [game receipt](acceptance-005/game/attempt.json) records
3,923 source records, 15 played spells, six attacking combats, three shuffle
callbacks and three mulligan decisions, ending in life loss. The source's actual
opening includes a mulligan and reshuffle. Recording off/on produced equal
controlled return values and semantic states; recording off emitted zero records.
The source-game and control classes/configuration are pinned in that receipt.

[All 23 development/acceptance attempts](attempt-index.json), including failed
bridges and both negative controls, retain their seeds, outcomes, errors and
original artifact hashes. None is admitted or counted toward a quota. Earlier
control failures identified the shared source action-history cache; those failures
are retained instead of blessing the unequal results. The earlier cold baseline
build was killed after compilation; its unchanged retry passed before live work.
Unqualified development payloads and unchecked logs remain private diagnostics,
not acceptance authority. The [supporting prior source trace](acceptance-002/trace.jsonl.xz)
remains historical evidence. Ordinary corruption tests use a retained real source
trace, not a parser-authored game.

The [integrity manifest](integrity.json) pins published artifacts, including the
[original final trace](acceptance-005/game/trace.jsonl.xz), negative traces,
control states and execution-source snapshots. Verify bytes and source accounting:

```sh
python3 doc/evidence/xmage-ai-capture/check_evidence.py --full
# During delivery, additionally bind the receipt to current execution sources:
python3 doc/evidence/xmage-ai-capture/check_evidence.py --full --current
python3 -m unittest discover -s tests -p test_xmage_ai_capture.py
```

Original reviewed
source-native acceptance traces are retained with hashes; observed outcomes are
candidate source evidence, never independently established rules goldens.
CR 103 supplies the opening/deck/mulligan requirements; CR 117, 601, 603, 508–510,
514 and 704 supply the required choice and semantic boundaries; CR 104.3 supplies
the life/empty-draw termination distinction. Expected corruption/failure behavior
comes from the registered L0 capture contract and existing recording contract.

Ordinary discovery covers persistence, overwrite refusal, completeness and
corruption rejection. The first [independent review](review-001.json) reproduced
acceptance of erased callback payloads. The repair validates every supported
callback signature, including arguments, result types and required target/payment
fields. Retained corruption tests erase payloads, decisions, targets and payments;
the [behavioral red](callback-red.log) precedes that repair. The explicit acceptance command executes the actual source
game and negative callbacks. Full locked torture and separate exact-candidate
review remain required; this document alone establishes neither delivery nor
milestone completion. Public evidence excludes unchecked upstream log/text payloads.
