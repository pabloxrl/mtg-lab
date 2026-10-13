# Forge ordered opening replay (L0-007)

The headless consumer replays opening prefixes from immutable complete Forge
PlayerControllerAi games. It uses normal frozen red/green forty-card decks,
initial shuffles, recorded starting-player choice, actual London mulligans and
ordered bottom selections. The Forge engine computes every transition; the
native engine independently executes the translated opening through its existing
normal-reset London interface. No seven-card state is injected.

## Run

Prepare an isolated external Forge cache with the unchanged
[smoke commands](../forge-ai-capture/README.md). In the managed container, run:

```sh
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- \
  python3 scripts/forge_opening.py --cache /home/agent/.cache/forge-GH-325 \
  --trace doc/evidence/forge-ai-capture/acceptance/game/result.json.jsonl.gz \
  --output .agent-artifacts/forge-opening-example
```

Use a new output directory for each attempt. The second retained full source game
is `doc/evidence/forge-opening/source-seed-1/result.json.jsonl.gz`. The original
source game starts seat 1 and redraws seat 0 twice; the second starts seat 0 and
redraws it once. Both finish through actual AI play in their capture receipts.
Those terminal captures are inputs, not full-game replay acceptance.

The command verifies source completion, hashes, capture implementation, frozen
manifest/rules/source pins and the complete raw trace before extracting a prefix.
It runs an actual unskipped Forge test, checks its dependency inventory, then
runs `captured_forge_opening_strict_replay` in native. The normal regression suite
also discovers that native test and the Python opening/alignment tests.

Explicit `--negative` values `missing-chance`, `reordered-chance`,
`duplicate-occurrence`, `unconsumed-choice` and `checkpoint` corrupt the consumer
input **after** source validation. Each executes Forge and requires the intended
rejection. Its receipt is `negative-detected`, never `opening-agreed`.

## Semantics and source scheduling difference

CR 103.4–103.5 independently supplies twenty life, seven-card draws, cumulative
London bottom counts and preservation of each forty-card multiset. The source
supplies actual choices and shuffle orders, never native-generated goldens.
Bindings identify each physical copy by seat/card/occurrence; Forge choice
selection also checks its source zone-change timestamp. Native chance injection
uses the existing physical-birth/handle client and leaves native RNG unchanged.

The Forge replay consumes the exact chronological prefix, including source events,
SBA boundaries, shuffle callbacks and raw random bit draws. All 115 public AI
choice wrappers route to the strict controller; no AI helper is executed during
replay. Nested private AI deliberation is retained, paired and consumed without
rerunning search; it may contain only callbacks/random draws and must preserve
the current game state. Any rules event inside that subtree fails explicitly.

Every observed Forge state field is compared, including ordered zones, life,
actor/phase/turn, card characteristics, physical IDs and incarnation timestamps,
mana, stack, combat and pending effects/triggers. The capture serializer's
`instance` is a Java allocation-observation index, not a rules incarnation; it is
retained in the source binding ledger but excluded from equality against a fresh
observer. Diagnostics identify the first state field at the source cursor.

**Preserved source scheduling difference:** Forge's pinned `MulliganService`
redraws/bottoms a player immediately before asking the next player to declare.
Native collects all declarations in a round first (CR 103.5). The adapter retains
source chronology verbatim, and separately aligns each actor's own hand/library,
life and mulligan count by round, placing native declarations before bottoming.
The explicit permutation in the fixture accounts for every source choice exactly
once. Libraries and selected bottom order remain ordered; native opening hand
comparisons use physical-copy sets, as in the delivered London reference client.
This establishes those actor-local checkpoints and settled first-upkeep states,
not equality of every transient global state or rules correctness of Forge's
callback scheduling. No computed state is overwritten and no AI is reconsulted.

## Scope and evidence

The endpoint is the first `chooseSpellAbilityToPlay` callback at first upkeep.
It is not consumed as an opening choice. The original full tape is immutable;
its remainder is explicit, with first record, count and uncompressed-byte digest.
Every output says `complete_game: false`. No gameplay, casting/payment, combat,
terminal replay, pilot, corpus admission, quota or milestone completion follows
from this prefix. Frozen pool, source pin, capture implementation, smoke and
previous reference regressions are unchanged.

The fixture retains source observations for both starters and native alignment.
Execution receipts, live negative controls, source/tool hashes, full torture and
separate exact-candidate review are recorded during delivery. Failed and
superseded attempts remain separately indexed and are never passing evidence.

### Executed acceptance

[Nine-run summary](execution/summary.json) and [artifact index](execution/index.json):
four positive Forge/native executions (both starters twice) and five actual Forge
rejection controls passed. Repeated Forge and native outputs are identical within
each source game. Every Java report has one test, zero failures/errors/skips.
Each native invocation also repeats its opening and checks four invalid ledgers.
The eight Python tests independently check source provenance, all-choice alignment,
CR-derived hand/deck counts and malformed source prefixes in normal discovery.

Each published run contains the exact receipt, compressed replay input, resulting
Forge state, native results where applicable and a source-report summary. Original
JVM logs/XML may contain runtime prose and remain private; their exact hashes stay
in receipts. The old runner archive records a sole trailing-whitespace cleanup
made during the sweep; its normalized bytes equal the current runner. Java,
native and patch implementations are identical across all nine runs and the
candidate; both runner hashes remain explicit, not silently rewritten.

Three unsuccessful integration probes preceded the passing runs: upstream
unused-import validation, overloaded wrapper replacement and a temporary recorder
helper dependency that broke a capture compile. Their diagnosis was integration
of pinned test harnesses, not a rules defect; retaining unreachable original
bodies and observing the existing serializer resolved those seams. The original
logs and failed receipts are retained separately. Five initial behavioral
failures, subsequent passing regressions, all source attempts and superseded
replays remain in the attempt history.

Full locked torture, separate exact-candidate review (including the scheduling
limitation and README), protected PR and exact-main CI receipts are recorded in
the [delivery workpad](https://github.com/pabloxrl/mtg-lab/issues/325#issuecomment-6134585413).
A passing prefix never completes the larger L0 corpus requirement.
