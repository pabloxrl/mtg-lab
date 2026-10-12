# Captured XMage game normalization — L0-002

This test-only converter consumes the immutable ComputerPlayer7 game published by
[L0-001](../xmage-ai-capture/README.md). It freezes a version-10 strict transcript
before running either engine. Native and **patched-combat-sba-v1 XMage** each
execute the complete game twice without AI search. This is one provisional
native/XMage game, not a ten-game pilot, Forge admission, an L0 batch runner,
1,000-game corpus, speed claim or milestone verdict.

```sh
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- \
  python3 scripts/xmage_ai_normalize.py --cache "$MTG_REFERENCE_CACHE" \
  --output .agent-artifacts/my-normalized-game
python3 -m unittest discover -s tests -p test_xmage_ai_normalize.py
```

Prepare/build the pinned issue-local XMage cache using the capture guide first.
The output directory must be new. It contains original-source cursors, normalized
inputs, source-derived checkpoints, two real runs per engine, rejected transcripts,
logs and a checksummed receipt. Failures retain their output and failed receipt.
The source tape is never overwritten. The reference bridge and execution sources
are hashed; the upstream source/dependency pins and combat patch remain explicit.
Run the full locked torture suite as well as this bounded acceptance command.

## Independent basis and semantic boundaries

The source game has 451 normalized actions, 15 casts, three actual shuffle outcomes,
four opening choice callbacks (including one bottom choice), 301 settled action
checkpoints, six pre-SBA damage checkpoints and thirteen cleanup checkpoints.
It ends on turn fourteen with seat zero losing to life loss. Repeated execution
of this same captured content is not another distinct game.

CR 103.5 supplies London chronology; CR 400.7 supplies physical/incarnation
separation; CR 117 supplies priority/pass boundaries; CR 601.2 supplies announced
targets and paid costs; CR 603 supplies trigger contexts; CR 508–510 supplies
combat choices/damage; CR 514 and 704 supply cleanup and state-based actions.
Frozen card definitions supply costs, targeting roles and characteristics.
The original source observations are candidate evidence, not golden rules output.
The existing independent admission cost/effect/identity ledgers validate both
actual consumers; source checkpoints and cross-engine comparisons add agreement.
No native output is copied into an expected checkpoint or game state.

The regression checks exact action/chance/opening consumption, life, actor/turn/
step, ordered library/graveyard, hand/battlefield membership, exile, every physical
incarnation, effective permanent characteristics and damage, stack identities and
targets, mana, effects/grants and combat groups. Hand and battlefield iteration
order is representational; semantic membership is compared, as in existing
admission. Cast/payment transaction snapshots are separately checked by the
rules/card-derived ledger because the two engines stage their transactions
differently. The recorded mana and discard order projection is the existing
explicit CR 601.2h permutation. Opening declarations/bottoms are checked against
the actual source, including the mulligan reshuffle.

Terminal comparison preserves source-specific losing-player removal. Pinned
XMage clears the loser's existing library/hand/graveyard at LOST before the same
SBA moves lethal creatures. The converter retains the original before-LOST zone
contents and final checkpoint. Comparison removes only those witnessed contents
from native's terminal observation, verifies the declared losing seat, and checks
the remaining full source state. The native/reference terminal comparison also
uses XMage's actual before-departure observation. No engine state is overwritten.

The raw game's single-blocker trample allocations bind the recorded attacker UUID
to its combat group and typed amounts. Token birth groups bind physical tokens
and their stable creation ordinals; source UUIDs remain in the original tape.
Source callback nesting and integrity are validated before conversion.

## Explicit unsupported seams

This bounded converter rejects source callbacks outside its declared set. It also
rejects pending/stack trigger normalization, modal Goblin Surprise, multi-blocker
allocation recipient ambiguity, owner-ordered simultaneous graveyard arrivals,
unsupported target roles/activation kinds/effect
classes, departed continuous-effect targets, failed cast/land transactions, overlapping stack entries from the same
physical source, standalone choices outside supported transactions and natural
draws. These are visible normalization failures, never default modes,
passes, allocations or missing observations. The selected complete real game
exercises none of these unsupported seams. Existing independently authored
trigger/order, both-mode, graveyard-order and other admission regressions remain
mandatory; their coverage is not attributed to this captured game. Later captures
requiring these seams cannot be admitted until explicit conversion is implemented
and verified. All twenty definitions, both 40-card decks and Goblin token are
unchanged; no new rules mechanic or M3 migration is introduced.

## Negative controls and observer repair

Actual consumers reject five mutated transcripts: a same-name physical Mountain
still in the library (action 2), a duplicate shuffle occurrence (opening), an
incorrect payment color (23), an illegal Bite target (137), and an inserted empty
trigger order where only priority is available (0). This game has no real trigger
ordering choice; the inserted-order control is not a claim of captured trigger
coverage. Existing admission's genuine trigger permutation controls are retained.
Ordinary discovery also mutates a source-linked life checkpoint and an opening
library order, and deletes a required field. Failures localize the first relevant
source cursor/field or executable action cursor.

Stronger ledger validation exposed a reference observer ordering defect: a
terminal snapshot could advance the semantic incarnation before the departure
watcher read it. The watcher now binds the departing incarnation using the
witnessed source/destination zone. This changes observation only, not rules.
The original failing output and behavioral red are retained. The shared library
ledger now follows the last recorded shuffle and London bottom order; earlier
all-keep admission expectations are unchanged.

Delivery remains conditional on full regression, separate exact-candidate review,
protected merge and exact-main CI receipts. Live delivery status belongs in the
issue workpad. No unsupported capture contributes to a corpus quota.

## Retained execution

[Final acceptance](acceptance.json) records **two successful executions per
engine**, all five executable negative controls, source hashes and canonical game
digest. The canonical version-1 bytes contain seat assignments, frozen deck
occurrence lists, physical copy/incarnation bindings, ordered chance and normalized choices; UUIDs, seeds,
engine names, timestamps and file paths are excluded. This digest is scoped to
the unchanged card/rules pins in this report, not cross-revision corpus admission.
The [run archive](runs.tar.xz)
retains all four development acceptance invocations: acceptance-1 through
acceptance-3 are superseded; only acceptance-4 uses the final complete verifier.
The original observer failure and the required-field/ledger behavioral reds are
retained alongside the runs. [Execution sources](execution-sources.tar.xz) pin
all core Rust sources and replay/comparison dependencies. The [integrity
manifest](integrity.json) protects the published bytes.

```sh
python3 doc/evidence/xmage-ai-normalize/check_evidence.py --current
```

This command rechecks the actual retained engine pairs, source receipt and tape,
required intermediate ledgers, run/source checksums and current execution-source
binding. Ordinary discovery executes fresh native replay and compares it with the
checksummed actual [reference witness](xmage-witness.json.xz), including a
regression for the terminal departure incarnation. The witness is captured
reference evidence, not a synthetic sibling implementation. The explicit command
above additionally runs the real Java consumer; offline tests do not claim to
execute Java.

The loss-reason adapter has a separate CR 104.3c regression for the existing
`EmptyDraw` replay label. This label check is not an additional captured deck-out
game or a natural-draw qualification.

[Final full validation](validation.json) passed **458 Python tests and 763 Rust
tests in each debug/release profile**, with zero failed or ignored tests;
formatting, Clippy and documentation/program checks passed. The [complete log](torture-3.log.xz)
is retained. Earlier [fixture-mismatch](torture-1.log.xz) and
[interrupted](torture-2.log.xz) runs are explicitly superseded, not passes.

The [existing reference-pack rerun](reference-regressions.json) passed all sixteen
normal-reset games twice per engine, strict invalid tapes and all eight retained
reference families. Its [306 structured artifacts](reference-regressions.tar.xz)
and [per-file checksums](reference-regression-index.json) are retained. The shared
observer and library-ledger repairs therefore preserve the existing actual
reference foundation; no frozen expectation, source exception or negative
control was removed or weakened.
