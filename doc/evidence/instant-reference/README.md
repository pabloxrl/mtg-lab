# Shared instant-response executor (GH-136)

Historical delivery evidence for #136. The current executor additionally covers
[departed targets (#137)](../departed-reference/README.md), with incarnation-aware
observations and separate updated receipts. Scope statements below describe the
original delivery; the original receipt remains unchanged.

This delivery covers only Giant Growth responding to Bite Down on its destination
and on its source. M1, #115/#18 integration, departed targets, combat, cleanup,
full-pool, Forge and dual-reference/full-game release requirements remain open.

## Original requirements and expectations

The [script](../../../fixtures/reference/instant-responses.json) and
[literal expectations](../../../fixtures/reference/instant-expectations.json)
were authored here before either adapter, without consulting upstream scenario
expectations or generating truth from the native engine. The pinned
[CR manifest](../../../data/rules/cr-2026-09-25.json) identifies the September 25,
2026 rules and checksum. The [Oracle manifest](../../../data/cards/foundations_micro_v1.json)
pins Bear Cub (2/2), Giant Growth (+3/+3 until end of turn), Bite Down
(source creature deals damage equal to its power to target creature not controlled
by its controller), Forest and Mountain. XMage uses those FDN cards at its pinned
source revision; the exact source archive is verified before execution.

- CR 117.3c/117.3d/117.4: casting retains priority; passing transfers it;
  consecutive passes resolve only the top spell and return priority to the active
  player. The response caster is P1, while P0 receives priority after resolution.
- CR 601.2c/601.2f–h and 605: ordered source/destination targets, explicit green
  and red generic payment, and basic-land mana activation without using the stack.
- CR 608.2c/608.2h and 613.4c: Growth resolves first; the selected Cub becomes
  5/5. Bite reads source power on resolution. Destination response: two marked
  damage on a 5/5, which survives. Source response: five damage on the opposing
  2/2, which dies under CR 704.5g.
- CR 400.7: after a spell or creature changes zones, the report identifies its
  card lineage but does not claim the new object is an old legal target. No
  target-departure response is exercised here; that remains #137.

The `red.log` and `comparator-red.log` record intended behavioral assertion
failures with compiling/importable placeholders, before their implementations.
Expected JSON has never been regenerated from engine output.

## Shared format and boundary

Both engines begin comparison at **turn-one upkeep, P0 active and holding
priority**, with empty ordered libraries, explicit synthetic hands and permanents,
20 life each, empty stack/mana, untapped undamaged Cubs and lands. This is a
synthetic position, not reset-to-game reachability. No draw/shuffle/chance occurs
within the script. Native construction uses no opening-hand policy; XMage runs
its empty-hand setup prelude and installs the declared hands at the first upkeep
callback, before the initial checkpoint. There are no hidden mulligan decisions,
pre-script passes or different starting steps counted as shared execution.

`version: 1` contains cases with `id`, `setup`, and one flat ordered `script`.
Every mana/cast/target/pay/finish_cast/pass choice names its actor; named objects
are unique per owner/card in this bounded adapter. `checkpoint` names an
observation boundary and carries **no expected state**. Cast target callbacks and
manual mana-payment callbacks consume that same stream in XMage; the native
adapter calls the real typed continuations. Implicit target inference, automatic
mana payment, AI priority and default passes are not used. Unexpected choices
fail. XMage strict choice mode remains enabled for other prompts.

The native adapter is a `cfg(test)` module, not a production rules API. It invokes
existing core mana/target/cast/quantum/pass methods. The Java bridge independently
invokes pinned XMage and never imports or calls mtg-core. No production rule
implementation was added or changed.

## Observations and limits

Five settled checkpoints compare turn/step, active player, priority, life, mana,
bottom-to-top stack and ordered targets, named card zones, battlefield taps,
creature power/toughness/marked damage, and actual dealt-damage events. Native
quantum-one settlement records the applied damage modification before lethal
zone movement; an XMage watcher records `DAMAGED_PERMANENT`. Neither observer
calculates damage from source power. Internal damage observation is **not** a new
priority window. This distinguishes exactly five damage from merely seeing a death.

Non-battlefield statistics are null. Semantic names are lineage labels across
zone changes, with no departed/reentered-target support. Empty libraries and
exile are asserted; setup contains every card. Legal-action enumeration, hidden
views, full continuous-effect descriptions, summoning sickness, land-play counters,
graveyard/hand ordering, and internal engine IDs are not compared. No combat, cleanup or game outcome is
claimed. The full RFC acceptance remains with the integration/release owners.

## Reproduction

Inside the managed Linux ARM64 container, use the image's Rust, Temurin
21.0.9+10 and Maven 3.9.11. A writable copy of the image-provided cache is needed;
no Docker invocation or host installation occurs in the worker:

```sh
cp -a /home/agent/.cache/xmage /tmp/mtg-xmage
python3 scripts/instant_reference.py --cache /tmp/mtg-xmage --output /tmp/instant-acceptance
cargo test -p mtg-core instant_
python3 -m unittest discover -s tests -p test_instant_reference.py
./scripts/torture.sh
```

Use the existing [reference build instructions](../../../references/xmage/README.md)
if the external cache has not been prepared. The runner verifies source archive,
original source bytes, Java/Maven pins and all locked Maven dependencies. Runs
are offline, stdin closed and display unset, with 180-second native and
300-second XMage subprocess limits. Missing dependencies/build/runtime failures
remove any old success receipt, return nonzero and retain a failure artifact.

The acceptance runner executes both cases twice in **both real engines**, checks
literal expectations and exact consumed input, then compares repeated/cross-engine
observations. It separately executes omitted-target, extra-target, reordered-payment
and wrong-actor mutants through both adapters, requiring the expected diagnostic;
a compiler or unrelated error cannot count as a rejection. A legal changed Growth
target executes and must produce a first-divergence artifact. Normal discovery
also exercises every native omission/duplicate/wrong actor, consequential
reordering, exact consumption (including commutative mana reorder detection), and
comparator mutations of target, power, stack, damage, zones, mana and priority.

The corpus is the acceptance authority: callers cannot replace expectations or
supply success checkpoints. The comparator requires the approved setup and exact
script as well as actual observations. A legal but reordered mana sequence is a
different request, even if it reaches identical states.

## Provenance and retained evidence

Adapters and scenarios are original project-authored work, not borrowed upstream
tests claimed as independent agreement. XMage APIs and test setup patterns were
consulted; its [MIT notice](../../../references/xmage/UPSTREAM-LICENSE.txt) is
retained. Upstream source stays in the external cache with no patches. Forge code
or tests are not imported. No new card/rules-text redistribution is added.

[acceptance.json](acceptance.json) identifies scripts, literal expectations, both bridge sources,
all native source files, Cargo lock, rules/Oracle manifests, reference pins,
dependency lock, toolchains, repeated artifacts and negative-control receipts by
hash. Repeated JSON observations and changed-target first-divergence artifacts
are retained beside this report. Raw build logs stay in the external run output;
receipt hashes identify them. Normal/full-suite and independent candidate review
results are recorded in the issue workpad and PR. Only protected merge plus
successful exact-main CI completes this delivery.

Pre-review verification: all 140 Python tests and 272 Rust tests plus two doctests
in each debug/release profile passed in the managed container, including
formatting, Clippy, documentation, program and catalog validation. No existing
test was deleted, skipped or weakened. README commands and the new reference
command were exercised; the milestone table remains unchanged. Final candidate
review, PR checks and exact-main CI are linked from the delivery workpad.

Independent review initially passed with one advisory: native observations did not
assert the complete object inventory. The retained `inventory-red.log` reproduces
that gap before the fix. A normal-discovery regression now injects one unexpected
card into each of all nine zones; every mutation is rejected, while a complete
minimal inventory passes. Native checkpoints explicitly assert empty libraries
and exile, matching the documented scope. The revised candidate is reviewed again.
