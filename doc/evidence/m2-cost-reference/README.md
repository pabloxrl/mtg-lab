# GH-209 costs, tokens and modal reference pack

This pack composes the delivered mana, haste, discard, modal, token and cast-trigger
bridges. It changes no rules implementation. The [joined reference receipt](reference/receipt.json) records all eight suites
passing on the integrated candidate. The [full torture receipt](verification.json) records 204 Python checks and 1,260
Rust debug/release executions passing with zero failed/ignored tests. Delivery
completion additionally requires independent review, protected merge and successful
CI on that exact main commit in the [issue workpad](https://github.com/pabloxrl/mtg-lab/issues/209#issuecomment-6076400283). M2 and whole-pool verification remain open.

## Independent expectations and scope

The [56-case mapping](catalog.json) preserves every original catalog row verbatim,
including setup, actions, expected result, rules basis and original owner. It adds
executable names and explicit reference boundaries. `test_m2_cost_pack.py` rejects
missing/duplicate assignments, changed expectations/owners and absent native tests.
This is an execution allocation, not 56 newly authored scenarios.

The existing unchanged mechanic inputs/oracles are rerun by their original runners.
The new [composition input](../../../fixtures/reference/m2-cost-composition.json)
and [literal oracle](../../../fixtures/reference/m2-cost-composition-expectations.json)
cover:

- A freshly cast Elf cannot tap until Cavalry's haste ability resolves; it then
  immediately produces G without a mana ability on the stack (CR 302.6, 602, 605,
  702.10).
- A newly cast Druid receives haste and produces G during Bite payment. Its one
  power marks one damage on Cub; the ability creates no response window (CR 601,
  605, pinned Druid and Bite).
- Thrill commits its discarded Mountain before the Archer trigger. Opponent life
  changes 20 → 19 before the ordered Forest/Cub draws; Growth remains in library
  (CR 601.2h/i, 603, 608, 121 and pinned Oracle).
- The inherited coordinator-authored holdout has two Archers, opponent life five,
  discards Surprise, and draws Forest then Mountain, leaving Cub. Discard is not
  casting Surprise: exactly two triggers change life 5 → 4 → 3. The literal input
  and oracle were present in the imported draft before either engine was executed
  in this managed checkout. No expectations were generated from engine outputs.
- Mountain is played without a stack entry and taps for R; after Forest is played,
  a second Mountain play rejects, leaving the card in hand (CR 305, 605).

Every new position is explicitly synthetic setup. Actual casting, payments,
activations, choices, triggers and resolution run in the existing engine APIs and
pinned XMage; there is no alternate rules evaluator. The Java bridge does not read
the expectation file. Native policy hand rows use the sorted visible hand; public
battlefield rows follow battlefield order. Initial imported-draft [failures](draft-controller-failure.txt) were
controller row-selection errors, not rule defects or passing comparisons.

The original Archer-before-Fodder, fixed Surprise affected set, empty/one-card
Thrill draws, unique Goblin identities and death/cessation cases remain in the
cast-trigger, Surprise, Thrill and token runners. Original native tests supply
malformed-count, stale/wrong-seat, private pending snapshot and rejection-nonmutation
coverage where reference APIs are different. The catalog notes identify those
boundaries rather than claiming XMage supports native snapshot/input protocols.

## Normal-reset and regression obligations

The pack retains the prerequisite normal-reset Game/Driver/Run tests, semantic
replay, typed trajectory/JSONL, capture on/off and bounded quantum checks in normal
torture discovery. See the [mana](../creature-mana/README.md),
[haste](../haste/README.md), [tokens](../tokens/README.md),
[Thrill](../thrill/README.md), [Surprise](../surprise/README.md) and
[cast-trigger](../cast-triggers/README.md) receipts for their exact test names.
Synthetic reference setups do not replace these real played paths. No mechanic,
policy schema, recorder contract or supported-card claim is added here.

`test_m2_cost_reference.py` alters each checkpoint field and requires a first
semantic divergence; the live composition receipt retains explicit life, zone,
mana, discard-choice and checkpoint mutations. Missing build, unknown script kind,
invalid script fields and missing cases fail. A new attempt removes a stale receipt
before checking build inputs; unavailable references cannot inherit old agreement.

## Reproduction

Use the image's pinned Java/Maven/Rust toolchain and isolated issue cache. All heavy
commands must use the shared lock. Cache creation and cold build, when necessary:

```sh
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- python3 scripts/xmage.py prepare --cache "$MTG_REFERENCE_CACHE"
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- python3 scripts/xmage.py build --cache "$MTG_REFERENCE_CACHE"
```

Execute the pack and full regression suite:

```sh
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- python3 scripts/m2_cost_pack.py --cache "$MTG_REFERENCE_CACHE" --output .agent-artifacts/m2-cost/reference
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- ./scripts/torture.sh
```

The [pinned build receipt](build.json) retains the failed cold attempt and unchanged
successful cached retry. The pack runs eight existing/composed reference suites sequentially and publishes a
joined receipt only after each passes. Full torture is a separate mandatory gate,
including the native combat fixture and all rejection/played-path tests. A pack
receipt alone does not certify full regression acceptance. No Forge, reference AI
game generation, full-game reference equality, whole RFC block or M2 gate verdict.
