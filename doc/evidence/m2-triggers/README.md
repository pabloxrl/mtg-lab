# GH-211: bounded cast, ETB and cleanup reference pack

Candidate execution passed on integrated main `0f46ba11c1c8902ac7337af3377b132ee361c1ea`; protected delivery remains subject to the required review and CI. No milestone completion is claimed.
This pack composes the existing #194/#205/#206/#207 mechanics and reference
operations. It introduces no engine rule, public card, policy or bridge operation.
Original #23/#24 ownership and #26's complete acceptance remain unchanged.

## Reproduce

In the managed Linux ARM64 image, use this issue's isolated cache. Run every
Cargo/Maven/reference command under the shared heavy-work lock:

```sh
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- \
  python3 scripts/m2_trigger_pack.py --cache "$MTG_REFERENCE_CACHE" \
  --output .agent-artifacts/m2-triggers
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- \
  ./scripts/torture.sh
```

An absent cache needs the pinned `scripts/xmage.py prepare` and `build` commands,
under the same lock. Toolchain and dependency pins are unchanged. The pack runs
22 cast cases, 14 ETB cases, four cleanup cases and nine compositions twice in
both native Rust and actual pinned XMage. Counts include repeated inputs for
separate catalog obligations; they are not a count of distinct reviewed scenarios.
The [38-case assignment](../../../fixtures/reference/m2-trigger-assignment.json)
identifies every unchanged catalog ID, executable case and observation boundary.
The runner rejects omissions and writes the aggregate receipt only after all
four component runs pass. A failed or unavailable reference never counts as agreement.

## Independently specified behavior

[Inputs](../../../fixtures/reference/m2-trigger-composition.json) and
[literal expectations](../../../fixtures/reference/m2-trigger-composition-expectations.json)
were authored from pinned cards and CR 603.2/603.3b, 101.4, 113.7a, 608.2b and
704.5a before either engine executed them. No golden was generated from output.
Every setup is explicitly synthetic. APNAP fixtures inject pending triggers;
they do not claim a naturally reachable frozen-pool APNAP event.

- Growth on Cub creates one Archer and one Cyclops ability before Growth.
- Cyclops becomes 3/4 before Bite reads its power and kills the opposing Cub; a second case leaves a 4/4 Sentry with exactly three damage.
- Archer deals lethal damage before Fodder resolves, so no Goblins are created.
- Both active seats choose local APNAP orders. Source labels distinguish two
  physically different Archers even when life totals and stack counts match.
- Pending Archer abilities survive their sources' declared synthetic departures.
- The independent holdout orders Archer A, Cyclops C, Archer B above Growth(C),
  then removes C. B deals one, C's ability does nothing, A deals one, and Growth
  fails its sole target. Both Archers remain 2/1; final life is [20,18]. This
  oracle was specified in the [migration handoff](https://github.com/pabloxrl/mtg-lab/issues/211#issuecomment-6076773090) before implementation resumed.

Each new native composition restores a snapshot before each resolution. Existing
normal-reset Driver/Run, quantum, semantic action and typed recorder tests remain
in ordinary discovery and are rerun by torture; synthetic composition fixtures
are supplemental evidence, not replacements for played-game evidence.

The existing ETB end-step bridge now additionally asserts the next upkeep after
trigger resolution, matching the native advancement assertion. Existing literal
ETB JSON remains unchanged. The cleanup bridge asserts one ordinary or two
exceptional cleanup entries and exports hand/graveyard/life/stack/stat ledgers;
its repetition count is an internal assertion, not a JSON-compared field.

## Negative controls and limits

Ordinary Python test discovery mutates every composition checkpoint field,
removes cases/resolution checkpoints, reverses source order, and omits the final
cleanup checkpoint. The pack retains single-checkpoint source-order and
skipped-cleanup counterexamples with expected/actual values and first divergence.
These are comparator mutations, not engine defects or additional agreed scenarios.

Native raw invalid orders/targets/discard counts reject without mutation.
XMage uses strict local choice callbacks, `canTarget` and required cardinality;
it does not consume the native transaction API. In particular, no-mana Growth
is tested through XMage non-playability and cancellation is at target selection.
Each assignment records this boundary instead of claiming identical API rejection.

No Forge expansion, full reference games, full hidden-state comparison, whole
RFC-block completion or M2 gate pass is claimed. The source-derived labels and
selected creature characteristics are privileged test observations, never policy
inputs. Existing upstream MIT notices and distribution constraints apply; no
upstream scenario or raw card text is copied.

README impact: add a link to this usable bounded reference command/evidence
surface; supported engine behavior, setup, native quickstarts and milestone
status remain unchanged. The independent reviewer must check this scope.

## Validation record

[Full torture](torture.json) passed after integrating main
`0f46ba11c1c8902ac7337af3377b132ee361c1ea`: 203 Python tests and 1,260
Rust checked/release executions, zero failed or ignored. The new composition
runs in normal Rust discovery and its comparator controls in normal Python
discovery. Formatting, lint and documentation checks passed as part of torture.
[The complete reference receipt](reference.json) accounts for all 38 assigned
catalog cases: 22 cast, 14 ETB, four cleanup and nine composition invocations
agreed twice in native Rust and pinned XMage. These 49 invocations include
repeated inputs and are not 49 distinct admitted scenarios. Every assigned case
retains its explicit observation/API boundary; all capability groups report zero
disputes or unavailable executions for those bounded observations.
[Composition observations](composition-observations.json) preserve actual outputs
from both engines separately from the independently authored expectations.
[Source-order](source-order-minimized.json) and
[skipped-cleanup](skipped-cleanup-minimized.json) artifacts retain minimized
comparator failures. Component receipts pin fixture, oracle, bridge, runner,
upstream and native source hashes; all recorded native source hashes were checked.

Separate candidate-bound review, protected merge and exact-main CI are mandatory;
[the issue workpad](https://github.com/pabloxrl/mtg-lab/issues/211#issuecomment-6076570380)
records that delivery evidence. This receipt does not certify whole RFC blocks,
normal-reset reference games, Forge coverage, #24's scenario floor or the M2 gate.

Initial fixture-harness corrections were a stale handle scope after snapshot
restore and duplicate XMage aliases across seats. These were not production
rules defects or engine behavioral-red evidence. The existing engine and literal
expected ledgers were unchanged; fixture labels now preserve incarnation across
restore and use globally unique XMage aliases. Every original regression remains.
