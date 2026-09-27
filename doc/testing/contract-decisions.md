# Defaults for the test contracts

These coordinator decisions resolve implementation choices identified independently
in the component plans. They refine the RFC; they do not claim implemented APIs.
Owners may propose a reviewed, equally faithful contract if implementation evidence
requires it. No routine product decision from the driver is needed.

| Choice | Default and owning issue |
| --- | --- |
| Batch mixed-validity input | #27: validate the batch envelope first; reject malformed envelope or duplicate slot submissions without changing any game. For well-formed distinct rows, commit valid rows and return per-row errors for invalid rows, which leave that game's state/RNG/decision unchanged. Results preserve input row correspondence. Test this policy explicitly. |
| Public/remembered history | #19/#27: retain legitimately observed own/public information in declared history; a card's later hidden-zone location/order is not refreshed from privileged state. Hidden permutations are compared only when the observer's entire legitimate information history is identical. This is not a promise to deduce all possible information from game strategy. |
| Seat with no action | #20/#28/#29: provide an episode final record with both seats' authorized final observations and final reward deltas. Create no invented policy action or transition for a seat with no decision. Readers must reconcile final records and last transitions without double-crediting rewards. |
| Game and recording result | #20/#28: separate game outcome from recording completeness. A rules-terminal game with writer failure remains a recorded game outcome but produces an incomplete, default-excluded training episode and an explicit run/collector recording failure. Never label its dataset complete. |
| Crash boundary | #28: guarantee documented process-interruption behavior on the supported local filesystem. Finalize and verify shards before atomically publishing a manifest; retain/quarantine incomplete fragments and identify orphan finalized shards for explicit recovery. Define flush/fsync/rename ordering and retry deduplication before testing. Do not claim machine/power-loss durability without separate evidence. |
| Seeds and provenance | #19/#20/#28: keep private shuffle/RNG state and complete chance tape in restricted replay metadata, linked by opaque IDs from seat-safe records. Default feature loaders use an explicit allowlist and exclude all provenance seeds/IDs that enable hidden-state reconstruction. Authorized two-seat dataset access is distinct from live-player access. Preserve reproducibility through the restricted manifest. |
| Transition time | #20/#28/#29: same-seat transition interval starts at its source decision (inclusive) and ends at the next same-seat decision (exclusive), or the final event. Count policy decisions in that interval and logical actions when they commit; final events are not policy decisions. Store both counts. Default gamma=1; an optional nonunit-gamma consumer must name its time unit and use independently calculated exponent tests. |
| Limit during a continuation | #17/#20/#29: stop at a safe engine boundary; record truncation reason, pending continuation kind and each seat's authorized final view of the committed state. The acting seat may retain its own permitted provisional choices. Never expose them to the opponent, grant a new response window, invent a completed action or report a rules draw. Wall-clock stop latency is bounded by the execution quantum. |
| Data corruption | #28: validate checksums plus episode/decision continuity, uniqueness, candidate/mask/action consistency and reward/footer reconciliation. Reject or quarantine affected whole episodes by default; a diagnostic reader can expose explicitly incomplete fragments. Recomputed checksums do not override semantic validation. |
| Checkpoint equivalence | #30/#31/#32: compare deterministic evaluation on frozen inputs, semantic actions and schema metadata after fresh-process reload. Set dtype/device-appropriate numeric tolerances in the versioned test fixture before running it; retain independent mask arithmetic checks. Exact stochastic training continuation is not claimed without saving and testing all required optimizer/RNG/collector state. |
| Dependency versions | #29–#32: choose compatible exact package locks when implementing each integration; exercise the real installed versions and pin consulted upstream API tests/source. A plan that mentions a wrapper is not a compatibility receipt. |

The detailed [systems](systems.md) and independent [data/RL](data-rl.md) plans
retain their discovery notes so reviewers can see why these decisions are needed.
The table above is the execution default where those notes identify an open choice.
