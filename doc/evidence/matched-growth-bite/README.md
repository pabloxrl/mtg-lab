# Combined Growth/Bite reference audit (GH-115)

This audit reruns the eleven original shared scripts together on fresh main
`6c1cd5e2bdffa4455d8ff3deaf0c70178f8fd896`, after the executor, departed-target,
blocker and cleanup components merged. It covers the original #115 acceptance,
not just child completion statuses or historical priority-pass smokes. No rules,
adapters, expectations, test discovery or workflow policy change in this delivery.
The acceptance result is in [acceptance.json](acceptance.json); protected delivery
and exact-main CI are recorded in the [workpad](https://github.com/pabloxrl/mtg-lab/issues/115).

This is partial R0002-B017/B026/B029 evidence. The complete scalar integration
remains #18, M1 completion remains #22, and Forge, all twenty cards, at least twenty
independent dual-reference critical cases, full games and M2–M5 remain required.

## Original acceptance crosswalk

The [shared input](../../../fixtures/reference/instant-responses.json) contains
setup and choices only. The [literal expectations](../../../fixtures/reference/instant-expectations.json)
were independently authored before the component adapters from the pinned
[CR](../../../data/rules/cr-2026-09-25.json) and
[Oracle definitions](../../../data/cards/foundations_micro_v1.json), not derived
from either engine or copied from upstream test expectations. This audit preserves
those bytes. The following reasoning was checked against the original requirements
and component specification; engine agreement does not establish the expectations.

| Original obligation | Executed cases and independently required result | Checkpoints per execution |
| --- | --- | ---: |
| Growth responds on Bite's destination | `destination`: CR 117/601/608 and Oracle require Growth above Bite; the 2/2 becomes 5/5, then survives two marked damage. Priority returns to active P0 after each resolution, even though P1 cast Growth. | 5 |
| Source power at resolution | `source`: Growth makes the source 5/5 before Bite resolves; actual damage is five, not the source's earlier two power. The opposing 2/2 dies (CR 608.2h/613.4c/704.5g). | 5 |
| Growth target departs | `growth-gone`: responding Bite kills the sole target. Its old incarnation stays named on stack; zero legal targets means Growth does not resolve (CR 400.7/608.2b). Same-name decoys receive no boost. | 5 |
| Bite source departs / partial invalidation | `bite-source-gone`: responding Bite kills the selected source. One remaining legal target means the lower spell resolves, but the illegal source cannot act and deals no last-known-power damage (CR 608.2b). | 5 |
| Bite destination departs / partial invalidation | `bite-destination-gone`: a second Bite kills the destination. The lower Bite resolves with one legal target but deals no additional damage; no same-name replacement is substituted. | 5 |
| Both Bite targets depart | `bite-both-gone`: two responding Bites remove both targets; the lower spell has zero legal targets and does not resolve. Earlier actual damage events remain distinct from this non-resolution. | 7 |
| Bite kills a committed blocker | `bite-killed-blocker`: exact attack/block/response choices lead through the blocker's death to combat damage. CR 509.1h/510.1c require the vanilla attacker to remain blocked with no surviving blockers and deal no player damage; life remains 20/20. | 9 |
| Cleanup without discard | `cleanup-7-2`, `cleanup-7-4`: seven cards, a 5/5 with two/four marked damage, no discard choice; next upkeep has the surviving 2/2 with zero damage and the same ordered hand. CR 514.2 removes damage and expires Growth simultaneously; no lethal intermediate state or cleanup priority. | 14, 16 |
| Cleanup with discard | `cleanup-8-2`, `cleanup-8-4`: CR 514.1 first requires the exact interior Forest `keep3` to be discarded. At that decision the Cub is still 5/5 with two/four damage and priority is null. Then CR 514.2/514.3 gives the same safe cleanup and next-turn upkeep, with exactly seven remaining cards. | 15, 17 |

Total: eleven cases, 103 declared checkpoints and 409 ordered script entries per
engine execution (entries include checkpoints). Each engine executes twice:
44 case executions and 412 checkpoint observations across the four retained runs.
This counts the named scenarios, not full games or distinct catalog requirements.

## Exact execution and observation audit

Both adapters start at synthetic turn-one P0 upkeep priority, 20 life per player,
zero mana, empty stack/libraries/exile and the same complete object inventory.
Permanents begin untapped/undamaged and under their owner's control; hands and
object insertion order are explicit. XMage installs the synthetic hands at its
first upkeep callback, before comparison. Native uses the declared zero-seed
`splitmix64-v1` setup; no chance result is used. No shuffle or draw occurs: the
starting draw is skipped and cleanup cases stop in next-turn upkeep before draw.
This is not normal-reset reachability evidence.

The [native adapter](../../../crates/mtg-core/src/instant_reference_tests.rs) calls
real rules decisions; the [XMage bridge](../../../references/xmage/InstantResponseTest.java)
uses actual XMage casts, targets, mana payments, declarations and discard callbacks.
Neither accepts expected checkpoints as input. Semantic IDs translate actual
object identities and zone-change incarnations, including old stack targets.
No AI fallback or independently chosen consequential decision is permitted.
Native's empty combat-damage completion is an API boundary with no allocation
choice; nonempty unscripted allocations fail. XMage's strict controllers reject
unexpected choice kinds and cleanup priority callbacks.

The [runner](../../../scripts/instant_reference.py) compares every declared field
and the exact consumed script against the unchanged corpus, then compares repeats
and engines. Missing/extra fields, checkpoints, cases or choices fail. Each run
contains both intermediate and final states, not merely a winner or final zones.

| Field | Compared observation / explicit limit |
| --- | --- |
| Turn, step, active player, priority | All named settled priority boundaries and next upkeep. Discard is a decision without priority; XMage's stale priority-player field is normalized to null only in its actual discard callback. |
| Stack and targets | Bottom-to-top spells, controller, ordered semantic targets and their selected incarnations; departed targets remain historical, not rebound by name. |
| Objects and zones | Complete named inventory, card/owner checks, current zone and incarnation, taps, empty libraries/exile, exact hand insertion order and graveyard oldest-to-newest order. Hand order is a bridge contract, not a prohibition on rearranging a Magic hand. |
| Stats, damage, resolution | Battlefield creature power/toughness/marked damage, actual damage events, legal-target count and actual resolution status. Non-battlefield creature stats are null. Damage is observed, not calculated from expected source power. |
| Combat and cleanup | Committed attackers, surviving blockers, remembered blocked flag, exact discard identity/count, boost/damage before discard and after cleanup, next player and untapped land. |
| Life and mana | Both life totals and six-color mana vectors at every checkpoint. |
| Unobserved / unexercised | Full legal-action enumeration, player-private views, internal IDs, full continuous-effect descriptions, summoning-sickness and land-play counters; library order is unexercised because libraries are empty. No general combat allocation, cleanup triggers/additional cleanup, next-turn draw, game outcome, or matched battlefield-reentry spell. |

## Negative controls and durable regression coverage

Existing regressions remain in ordinary discovery, unchanged:

- `instant_shared_script_literal_expectations` executes all eleven native cases
  with literal checkpoint equality and exact consumed scripts; the focused native
  suite also covers strict ordering, inventory, identities, departures, combat and
  cleanup choices.
- [Python comparator tests](../../../tests/test_instant_reference.py) reject
  every omitted/duplicated/reordered/wrong-actor consumed choice, altered setup,
  supplied success, missing cases/fields, target/power/stack/damage/zone/mana/priority
  mutations, departed identities/resolution, blocked status and cleanup faults.
- The full runner re-executes strict controls in both actual engines: omitted/extra
  targets, wrong actor, payment order, missing post-departure pass, invalid/duplicate
  attack/block declarations, omitted/reordered responses and malformed cleanup
  discards. Expected diagnostics are required; build or unrelated failures do not pass.
- Legal changed targets and a different legal Forest discard execute in both
  engines but must diverge from the independent request. Reduced departure and
  blocker inputs, observations, logs and first-divergence records are retained.
- The runner mutates observed blocked status and twenty cleanup observations;
  these are comparator fault injections, not claims of engine-produced bad states.
  Historical compiled-rule mutants remain separately identified in the
  [blocker](../blocker-reference/README.md) and [cleanup](../cleanup-reference/README.md)
  component evidence; they are not falsely counted as newly compiled in this audit.
- The unavailable-reference regression ensures stale acceptance is removed and a
  failure artifact remains. Failed or unavailable executions never count as agreement.

No new behavior or defect was introduced or found, so there is no fabricated red
phase and no replacement of existing tests. Original test-first failures remain
in the [executor](../instant-reference/README.md),
[departure](../departed-reference/README.md), blocker and cleanup evidence. This
integration delivery retains a fresh combined execution rather than adding a
second implementation or duplicating unchanged tests.

## Reproduction, provenance and delivery checks

Inside the managed Linux ARM64 worker, reuse the prepared writable external cache.
If absent, copy `/home/agent/.cache/xmage` once to `/tmp/mtg-xmage`; do not nest
another copy inside an existing cache. No Docker invocation, socket or host install
is needed. These commands were exercised for this audit:

```sh
python3 scripts/instant_reference.py --cache /tmp/mtg-xmage --output /tmp/gh115-acceptance
cargo test -p mtg-core --locked --lib instant_
python3 -m unittest discover -s tests -p test_instant_reference.py
./scripts/torture.sh
```

[acceptance.json](acceptance.json) pins source/archive, consulted APIs, card/rules
manifests, Java/Maven/dependencies, Rust/Python, both adapters, all native source
files and inputs. Every artifact in its file inventory is retained beside this
report; the four combined JSON outputs include the complete repeated observations.
Execution is offline, headless, stdin closed and subprocess-time-bounded. Original
scenarios/adapters are project-authored; consulted XMage APIs retain their pinned
[MIT notice](../../../references/xmage/UPSTREAM-LICENSE.txt). No upstream scenario
expectations, Forge code or new card/rules text is copied.

[verification.json](verification.json) records the fresh-main audit, artifact and
source integrity, focused tests and full managed-container torture result. The
root README links this precise aggregate coverage; its milestone table is
unchanged. Independent candidate review, protected merge and exact-main CI are
separate delivery requirements linked in the workpad, not inferred from local
reference agreement. No M1 completion claim follows from this report.
