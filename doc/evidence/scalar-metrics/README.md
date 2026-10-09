# Scalar instrumentation evidence (#214)

Status: focused acceptance passes; full delivery remains conditional on the
required complete suite, independent review, protected merge and exact-main CI.
Base: `0f46ba11c1c8902ac7337af3377b132ee361c1ea`.

Scope: the #80 atomic contract for off/counters at existing scalar Driver/native
boundaries. Partial R0002-B008/B020/B021 ownership; SYS-METRIC-001 counter paths
and SYS-METRIC-002 bounded public fields. Original #25/#26 acceptance remains.
No new card mechanic, alternate rules engine, batch/worker/inference work or RL.

Independent expectations come from the pinned RFC definitions, the existing
policy and trajectory contracts, and literal traces: two keeps, passes, a land,
one cancelled cast and one paid G-cost cast. The ordered normal-reset deck has
80 cards, 14 opening draws and two explicit shuffle bypass units. A passive
normal game must end on P1's empty draw because P0 skips the first draw (CR
103.8a/104.3c). Injected clocks exercise existing owner limits and backward-time
failure; elapsed wall time is never a golden value.

[Initial compiled red evidence](red.txt) distinguishes missing-counter failures
from the corrected draft mana input. Tests remain in normal Cargo discovery.
Validation commands must use the shared heavy lock in the managed image:

```sh
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- cargo test -p mtg-core --test metrics
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- cargo test -p mtg-cli --test native_simulate optional_public_counters
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- ./scripts/torture.sh
```

Focused checks pass: 10 integration tests, two recorder/materialization tests
and one native JSONL test, with zero failed or ignored. Full torture, quickstart,
review, protected PR and exact-main CI receipts belong to the
[delivery workpad](https://github.com/pabloxrl/mtg-lab/issues/214#issuecomment-6076402053)
and its linked PR; this report alone does not certify delivery.
This change introduces no new rules semantics; existing pinned reference evidence
remains authoritative for those rules. Instrumentation needs hand-counted boundary
and equivalence oracles, not reference engines' unrelated instrumentation totals.
