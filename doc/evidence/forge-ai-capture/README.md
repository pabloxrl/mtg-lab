# Forge real-AI raw capture — L0-006

This command runs pinned `PlayerControllerAi`/`AiController` for both seats,
using the unchanged forty-card red and green decks, twenty life, seven-card
opening hands and ordinary London mulligans. The source toss and AI choose the
starter. `Default` AI profile and the ordinary nonsimulation option are fixed.
There is no scripted replacement for the AI, test-mode mulligan suppression,
normalization, Forge replay, pilot qualification or corpus admission here.

## Run

Use the managed Linux ARM64 Java/Maven image. Keep this Forge cache private to
one worker and separate from its XMage cache. Preparation and the unchanged M0
smoke build install the pinned dependencies; capture then installs its explicit
observational patches in that cache. All reference commands run under the shared
heavy-work lock:

```sh
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- \
  python3 scripts/forge.py prepare --cache /tmp/my-forge-cache
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- \
  python3 scripts/forge.py build --cache /tmp/my-forge-cache
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- \
  python3 scripts/forge_ai_capture.py --cache /tmp/my-forge-cache \
  --output .agent-artifacts/my-forge-game --seed 0
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- \
  python3 scripts/forge_ai_capture.py --cache /tmp/my-forge-cache \
  --output .agent-artifacts/my-forge-acceptance --seed 0 --acceptance
```

Prefer a disk-backed external cache with several GiB free; `/tmp` may be a small
RAM filesystem. An existing output directory is rejected. The command creates a
private started receipt before source validation/build, then records status,
source/AI/toolchain/deck/rules/patch pins, exact command, elapsed time and artifact
hashes. Finalized files are read-only. A killed process leaves a started receipt
and any partial trace. Every attempted game remains recorded; errors and safety
limits never become wins or draws. Defaults are seed 324, 10,000 controller calls
and a 300-second process timeout. Seed 0 exercises natural mulligans in both seats.

`--acceptance` runs recorded and original-source uninstrumented games with the
same setup/seed, compares their complete event-state audit hashes and final states,
then runs actual unsupported-callback and call-budget controls. The controls must
fail for their specified reason and retain nonterminal traces. Each child has its
own receipt; failure stops acceptance. `--uninstrumented` is an audit control,
not a complete raw capture. `--reject-callback` is explicit fault injection only.

## Observations and source boundary

`result.json.jsonl.gz` contains the original ordered JSONL bytes. Each record
has an index and kind. Source callbacks have signature IDs, actors, entry arguments,
return values and after-call arguments; nested calls and returns retain their
chronology. AI cost visitors also retain the ability being considered. Candidate
choices inside AI evaluation are distinguishable from the subsequent committed
cast/zone/payment events; callback counts are not counts of committed actions.

The trace retains actual RNG `next(bits)` outputs, before/after ordered shuffles,
keep/bottom decisions, physical IDs and Java object-instance bindings, timestamps,
tokens, targets and divided allocations, mana lots and source abilities, paid cost
parts, combat declarations, and source events. Observations include ordered zones,
life, turn/phase/active/priority actors, stack, effective characteristics,
PT/keyword changes and their source table keys, pending-trigger context, before/after
SBA states, cleanup progression and terminal loss reasons. Library/private state
is privileged recording data and never fed back into the source AI.

The pinned source is `95dc682bf92460f49cebd7a9578f06ccf60d5569`. Source method bodies
are preserved verbatim in private helpers, each called exactly once by its
observational wrapper. Direct event hooks run before EventBus dispatch so recorder
errors cannot silently disappear inside subscriber handling. The audit subscriber
also retains any exception and checks it after play. Both paths reject unknown
payload representations. The default AI also evaluates candidate spells on a
worker with a five-second source timeout. Its swallowed timeout/worker exceptions
are detected in the retained runtime log and make the attempt truncated/failed,
even if upstream subsequently continues to a rules ending. Explicit callback and source-signature inventories,
sequence/return pairing, full opening multisets, identity bindings, record/chance
counts and required played witnesses are checked by the Python validator.

The setup uses the pinned card loader and its existing asset/preference/localization
test stubs, with the production type/keyword initialization added before loading
cards. These stubs do not choose actions. Real Match startup deals, shuffles and
mulligans; the real rules engine executes the game. Full-simulation mode is disabled
as in a normal default LobbyPlayerAi. Only the selected live Game object may emit
records; copied game instances cannot contribute callbacks/events. The capture
calls `Match.startGame` directly: Forge's convenience `SimulateMatch` forces a Draw
in `finally`, so it is deliberately not used as terminal evidence.

The only external build dependency patch exposes the **already pinned Gson 2.13.2**
artifact to the game module. Original and patched source hashes, both new Java
files, callback signatures and the unchanged dependency inventory are bound in
receipts. No upstream rules repair, AI tactic change or source repin is implied.
The original M0 Forge smoke and all XMage pins/exceptions/regressions remain intact.
Use a fresh/restored cache when rerunning the original smoke's unmodified-source
check; capture caches contain explicitly patched source.

## Evidence and limits

The [four-run source acceptance](acceptance/acceptance.json) passed. Its
[recorded game receipt](acceptance/game/receipt.json) binds 13,676 original records,
7,022 RNG draws, 1,254 nested callback entries/returns, four source shuffles,
four keep decisions, two actual bottoming choices and 644 paired SBA boundaries.
The normal game ends on turn 29 by life loss, with 31 casts and resolutions,
13 blocker declarations and token creation. These witnesses are observed coverage,
not independently certified Forge rules correctness or all-card coverage.

The [original-source audit](acceptance/uninstrumented/receipt.json) has exactly the
same full event-state digest and final state as recorded play:
`c728e2e6a3021edf33cddf3f27687623afc53766839220da52a9bdfb47dc8a4a`.
The [unsupported callback](acceptance/unsupported/receipt.json) and
[budget truncation](acceptance/truncated/receipt.json) fail at the intended seams,
retain their raw traces, and leave the game nonterminal. TestNG must execute exactly
one named, unskipped source test; stale reports are removed before each invocation.

The ordinary suite has nineteen raw-evidence checks, three source-diagnostic checks
and six source-patch guards.
The complete real [game trace](acceptance/game/result.json.jsonl.gz) is checked once
positively. Corruption cases keep the unchanged prefix through their defect and
require its specific diagnostic, so a generic missing-footer failure cannot pass.
They cover unknown schema/callbacks, return pairing, lost chance and record counts,
wrong physical copies/decks, missing zones/effects/payments/trigger queues/events,
SBA boundaries and nonnormal/incomplete endings. Parser tests alone do not certify
source execution; the actual JVM receipts above supply that evidence.

Full integrated-main torture and separate review results are added to the delivery
workpad/PR. Completion remains conditional on protected merge and exact-main CI.
No pilot/corpus quota is claimed.
This is a raw source observation contract, not proof that Forge's rules/card vintage
matches current mtg-lab or that a future replay can consume every callback. Later
registered consumers own normalization, strict replay and adjudication. Unsupported
callbacks/types are visible failures; no silent generic string fallback is used.
Only the red/green seat order is exposed here; this is not an eight-row pilot.
Natural rules draws are permitted by the program, but this bounded validator
currently accepts the demonstrated life-loss/deck-out wins and rejects other
terminal results explicitly rather than counting them as complete evidence.

## Provenance and distribution

The adapter and tests are original project code under the existing unselected
repository-license status. Forge and its linked runtime remain separately installed
GPLv3 material; the complete source, card scripts, compiled bridge and dependencies
are not redistributed. Event prose that may reproduce card text is hashed at
capture time; semantic card identities, source IDs, costs, choices and observations
remain available. Runtime logs and pre-redaction exploratory originals are retained
privately outside workspace cleanup in the issue-specific managed cache, indexed in
[attempt-index.json](attempt-index.json), and are not release artifacts. The index
is a pre-review snapshot; its torture log was still running when copied. Final
verification results and log hashes belong to the delivery workpad/PR. Published raw semantic captures
are privileged evidence, not player-observation datasets. Existing
[provenance restrictions](../../provenance-policy.md) and release licensing gates
remain unchanged; no artwork or full rules/Oracle documents are published.
