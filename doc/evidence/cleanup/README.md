# Cleanup continuation acceptance

GH-207 keeps CR 514.3a pending triggers in the cleanup step after active-player
hand-size discard and simultaneous damage/temporary-effect removal. Trigger
ordering, required player target, resolution and exceptional priority use the
existing Game/work, Choice and semantic action paths. After both players pass
an empty stack, another cleanup starts. Thrill during that window spends itself
and one discarded card, then draws two: seven cards remain, so the repeated
cleanup offers no redundant discard. Ordinary cleanup has no priority, and an
oversized opponent hand is unchanged.

CR 514.1–3 and 704 independently specify the ordering. The pinned pool has
positive printed toughness and only positive temporary boosts, so removing all
boosts together with damage cannot produce a creature death. The new cleanup
completion work checks terminal losses and pending triggers before next-turn
boundary, untap or sickness removal. A terminal outcome clears subsequent work.
No new trigger source, public injection hook, rules API or schema is introduced.

[Compiled behavioral red](red.log) demonstrated both defects before the rule
change: the pending trigger escaped into next upkeep, and terminal settlement
occurred after next-turn work. Expectations were not generated from engine output.
All regressions remain in ordinary test discovery; no existing assertion changed.

## Reproduction and scope

```sh
cargo test -p mtg-core --locked --lib cleanup_trigger_tests
cargo test -p mtg-recorder --locked --test cleanup
python3 scripts/cleanup_reference.py --cache /home/agent/.cache/xmage --output /tmp/cleanup-reference
./scripts/torture.sh
```

Five core tests cover both active seats, exact discard count and wrong-seat
rejection without mutation, opponent privacy, damage/bonus removal, mandatory
trigger placement/targeting, resolution, repeated cleanup and terminal finality.
Quantum one, three and unbounded are checked; snapshots restore at each internal
yield and exposed boundary. The synthetic Driver test carries cleanup discard,
Pyromancer target and Thrill through the existing captured driver. Invalid
commands preserve snapshot, semantic history and typed trajectory. Quantum
one/unbounded produce identical history and typed capture. Reapplying semantic
history to the synthetic starting snapshot reproduces hand counts, life and turn.
This is explicitly a synthetic component replay, not a normal-reset game.

The recorder integration test separately starts from a normal reset and plays
three pass-only turns through the existing Driver/Run. Both players must discard
one at their first oversized cleanup. Capture on/off and quantum one/unbounded
preserve semantic history; typed JSONL round-trip and full normal-reset replay
verify the final state. No cards are cast in this normal-reset test; real Thrill
and Pyromancer cast evidence remains in their respective acceptance reports.

## Unchanged catalog and reference boundary

The four `rules-continuous-hand-size-cleanup-{positive,negative,interaction,regression}`
cases retain their catalog setups/actions/expectations and original owner #23.
Their crosswalk composition owner #211 retains aggregate reference acceptance.
This mechanic executes all four now; no basic mechanic reference is deferred.

[Shared input](../../../fixtures/reference/cleanup.json) and
[independent literal expectations](../../../fixtures/reference/cleanup-expectations.json)
encode each checkpoint as `[active hand, opponent hand, active graveyard,
opponent life, stack size, Pyromancer power, toughness, damage]`. Original
CR/Oracle expectations are: nine-card hand discards two, eight discards one;
5/4 with three damage becomes 2/1 with zero; Pyromancer deals two; Thrill spends
two cards/draws two; opponent eight is untouched; next upkeep adds no draw.

The XMage bridge uses its real EndPhase/CleanupStep and real Thrill cost/draw.
Hand setup, boost/damage and first-cleanup pending Pyromancer are explicitly
synthetic test hooks. Its watcher counts cleanup entries, proving a second
cleanup only for the exception; a strict priority controller rejects ordinary
cleanup priority and all unexpected choices. Native tests additionally reject
incorrect discard sets atomically; XMage checks its discard target cardinality,
not the native transaction API. The comparator regression mutates every field
of every checkpoint and removes every case. No Forge or reference full-game
agreement is claimed. Reference execution receipts are recorded with delivery.

## Delivery and limits

Full torture after current-main integration, applicable pinned references,
prescribed independent Codex review, protected merge and exact-main CI are
required; see the [issue workpad](https://github.com/pabloxrl/mtg-lab/issues/207)
for final candidate/merge evidence. This report is not itself a completion claim.
README adds this supported behavior and distinguishes synthetic from normal-reset
coverage. No setup/quickstart command changes. M2, complete RFC blocks and full
native-policy coverage are not certified; original component and #26 acceptance
remain authoritative. Partial ownership: R0002-B010, R0002-B011, R0002-B029.

The [reference receipt](reference.json) records four cases agreed twice against
pinned XMage `000d8a7abc0ac31cc24af08691423e0c24dc59e7`, with negative controls,
input/expected/bridge/runner and native-source hashes. [Native](native.json) and
[XMage](xmage.json) checkpoint results are retained. Setup failures (a read-only
cache, an incorrect test-bridge damage method and the cleanup-specific target
type) were repaired before these successful executions; none counted as agreement.
