# Forge ability prefixes (L0-009)

These are nonterminal prefixes,
not admitted corpus games or an AI quota. The frozen decks, twenty definitions,
Goblin token and reference pins are unchanged.

The explicit `--abilities` mode extends the existing casting consumer. Forge
executes recorded choices with its actual rules engine. Native starts from the
same normal London opening and independently applies translated choices. Source
observations are comparison expectations, never replacement native state.

## Reproduction

Prepare a private external Forge cache as described in the
[capture guide](../forge-ai-capture/README.md). Use a new output directory:

```sh
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- \
  python3 scripts/forge_casting.py --cache /home/agent/.cache/forge-GH-327 \
  --trace doc/evidence/forge-abilities/source-rr-10/result.json.jsonl.gz \
  --empty-combat --abilities --output .agent-artifacts/forge-abilities-example
```

Omit `--abilities` to retain the previous casting boundary. Nonempty combat
remains unsupported. `--stop-before` may select an earlier supported root callback;
it cannot skip an unsupported transition. Complete-source receipt/hash checks
remain required, even though only a prefix is replayed.

## Independent expectations and retained coverage

- Frozen Druid text and CR 605: tapping the nonsick creature produces one green
  mana without adding an ability to the stack; the recorded payment consumes it.
- Frozen Cyclops text and CR 603/608: casting Goblin Surprise triggers +3/+0,
  resolving before the spell's separate +2/+0. The source witnesses 0/4, 3/4,
  then 5/4 at separate checkpoints.
- Frozen Invoker text and CR 602/608: eight generic mana, one chosen creature,
  +5/+5 and trample. The actual source chooses Druid: printed 1/3 becomes 6/8.
  Its reusable activation clears its targets after copying them onto the stack;
  the translator checks the committed copy and retains its original source/target.
- Frozen Pyromancer text and CR 603/608/120: its ETB trigger targets the recorded
  opponent and deals exactly two damage. Archer cast triggers are nontargeting.
- CR 603.3b: each controller orders their own simultaneous triggers, AP then NAP.
  Actual source groups include two through five Cyclops/Archer triggers.
- CR 113.7a/400.7: a stacked ability retains its original source incarnation even
  when that physical card becomes a new object in the graveyard. The authored
  translator regression retains original incarnation 0 while the new object is 1.

The normal native suite preserves `trigger_apnap_dead_source_and_every_placement_snapshot`,
`pyromancer_synthetic_multi_target_apnap_placement_and_end_step`, the M2 trigger
reference fixtures and departed Cyclops regressions. The new Forge authored
scheduler setup uses only frozen Pyromancers, explicit player targets and both
APNAP starting orders and departure before/after placement (four cases); it is
synthetic, not reachable-game or AI evidence. The receipt requires all four cases
and explicitly records zero authored AI quota.
The Shivan translation excerpt is also diagnostic only: the original source
activation occurs after an unsupported combat boundary.

## Current executable source witnesses

| Source | Endpoint | Actions / checkpoints | Behavior |
| --- | --- | --- | --- |
| Existing red mirror seed 2 | 2069 | 187 / 145 | Cyclops cast trigger and Goblin Surprise pump mode |
| Green mirror seed 20 | 1507 | 163 / 127 | Druid mana payment |
| Red mirror seed 10 | 7681 | 672 / 511 | Three Pyromancer target choices, simultaneous cast-trigger ordering |
| Existing red/green seed 4 | 8577 | 402 / 283 | Llanowar mana, Invoker eight-mana activation, Druid target and trample |

The red/green source was located through the previous delivery’s checksummed
[retention index](../forge-casting/attempt-index.json), read without modifying that
archive, then copied byte-for-byte here. Its complete original capture receipt
validates against the unchanged source runner and pins. Registration/prefixes are
not the 1,000-game acceptance. Exact-candidate review, protected merge, exact-main
CI and bounded handoff are recorded in the
[delivery workpad](https://github.com/pabloxrl/mtg-lab/issues/327#issuecomment-6140053736);
this source evidence alone does not certify those delivery gates.

All original traces remain immutable. Rejected consumer mutations retain changed
trigger source/context, duplicated or reversed order, player target and payment
controls, plus activation target/payment and temporary keyword-grant corruptions. The bridge binds generated ability IDs to observed source/controller
identity and compares retained context. Repeated JSON values are shared only
when exactly equal; streaming deserialization removes no record or field.
No tactical AI runs during replay, and no unsupported callback is silently passed.

Original mtg-lab bridge/test sources and observations follow the existing
[provenance policy](../../provenance-policy.md). Upstream installations, card
scripts and linked reference binaries remain outside these evidence artifacts.


## Identity and comparison limits

Forge consumes every source callback, chance draw and rules event. Rules-bearing
state comparison retains the existing documented
[allocation/presentation normalization](../forge-casting/README.md#comparison-of-source-allocation-identifiers),
including the transient mana-ability actor exception whose historical payer is
checked separately. Generated trigger definitions and activated/triggered stack copies bind
bijectively by witnessed source/controller and definition; the full remaining
context is still compared. Keyword-effect allocation timestamps use the same
bijection as power/toughness effects. Native references use previously witnessed
physical-ID/timestamp incarnations, and reject unseen incarnations. No reference
is silently redirected to the current graveyard/battlefield incarnation.

Native compares exact action consumption and priority checkpoints: actor, turn,
step, life, ordered libraries/graveyards, hand/battlefield/exile membership, mana,
physical identities, stack/source/targets/order, tapping, sickness, damage,
effective characteristics and boosts. Its mismatch reports locate the first field.
The source fixtures are translated from immutable Forge observations, never from
native output. Existing casting-mode fixtures and negative controls remain intact.
Floating/multi-unit/restricted mana and unknown keyword changes remain explicit
unsupported cases. Nonempty combat, cleanup/terminal replay and full admission
belong to later registered children. The authored Shivan excerpt proves translation
and a minimized identity regression only; live activation coverage here is Invoker.

## Live corruption controls

The original source is never edited. `--negative NAME` mutates only the consumer
input and requires a failure at the intended seam. Use the matching trace:

| Trace / boundary | Controls |
| --- | --- |
| `source-rr-10`, `--abilities --stop-before 2061` | `trigger-source`, `trigger-context`, `trigger-order`, `trigger-order-reversed`, `trigger-target` |
| `source-rg-4`, `--abilities` | `pending-trigger-source`, `activation-target`, `activation-payment`, `keyword-grant` |
| Existing casting `source-gg-11`, default casting mode | `target-incarnation`, `payment`, `pass-actor`, `callback-order`, `response-order`, `effect-amount` |
| Existing casting `source-rr-3`, default mode, `--stop-before 2984` | `token-power`, `mode`, `discard` |

All commands also use `--empty-combat`. The ordinary native test repeats every
fixture and rejects applicable payment/life, trigger source/event/controller/order/
target, activation source/target/payment and temporary-trample mutations. The
Python tests bind source-derived fixtures to independent printed-text/rules
expectations and retain the two authored departed-source regressions.

## Validation and attempt accounting

The [execution summary](execution/summary.json) links eight positive ability runs
(four source prefixes, each repeated with identical complete Forge/native outputs),
two preserved casting positives and eighteen live corruption controls. Each
ability run also executes the four explicitly authored APNAP/departure cases.
Every receipt pins its source and implementation; compressed artifacts retain
their original uncompressed SHA-256.

[Full locked torture](validation/torture.json) passed 514 Python tests and 1,532
Rust test executions across debug/release, with zero failures or ignored tests,
plus formatting, Clippy and documentation checks. Its
[complete log](validation/torture.log.gz) and input hashes identify the tested
implementation, tests and fixtures after integration of protected main.
Generated evidence and final documentation were completed afterward and their
links checked again before exact-candidate review.

The [source-attempt inventory](source-attempts.json) accounts for all forty new
started games: twenty-eight complete captures and twelve truncated attempts,
none admitted. The reused red/green seed 4 remains attributed to its original
delivery rather than counted as a new attempt. The
[diagnostic retention index](attempt-index.json) preserves all local attempts,
including failed translations/replays and the aborted first torture wrapper
(which failed to inherit the shared lock descriptor); only the corrected full
run above is passing evidence. The private archive is a diagnostic aid, never
acceptance evidence. Its latest index reuses checksum-verified immutable history
files and adds new diagnostics separately. After an interrupted duplicate copy
filled the disk, redundant working copies were removed only after verifying both
the archived contents and original hashes; every attempt remains preserved. The [artifact index](artifact-index.json) checksums the
published bundle. Complete games admitted and AI quota credited are both zero.

The [first candidate's CI timeout and repair](validation/timeout-repair.json)
remain explicit: its checks partition exceeded the unchanged thirty-minute
limit despite passing local torture and [initial review](validation/review-first-candidate.json).
The consumer now reuses the opening produced by its complete source validation
instead of validating the identical tape twice. Test mutations copy only the
changed record; all input observations, assertions, fixtures and negative controls
remain. The six ability tests measured 85.4 seconds before and 48.2 seconds after
under the same local lock with normal GC; these are diagnostic timings, not corpus
performance qualification. Protected CI, full revalidation and separate review
of the repaired candidate remain the delivery authority recorded in the workpad.


The [post-merge timing repair](validation/post-merge-timing-repair.json) records
both exact-main attempts on the first merge: every verification command passed,
but the unchanged job deadline expired during cache post-processing. Related
Python tests now share three identical fully validated immutable source prefixes
within their class. All 78 assertions in the changed files remain identical;
mutation inputs are isolated and teardown clears the shared setup. Consumer
code, source tapes, native fixtures, CI policy and timeouts are unchanged.
The 28 live receipts above retain matching implementation/native-source hashes;
they are retained runs, not newly executed runs for this test-only repair.
The [previous full torture](validation/pre-post-merge-repair-torture.json) and
[merged candidate review](validation/review-merged-candidate.json) remain available.
The full torture linked above is the latest repair's run; its new exact-candidate
review, protected repair PR and successful exact-main CI are required for delivery
and recorded in the issue workpad. Root README commands and behavior remain
accurate, with no new usable capability or milestone-status claim in this repair.
