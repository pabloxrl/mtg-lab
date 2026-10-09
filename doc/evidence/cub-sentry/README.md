# Exact Cub casting and single-Sentry reference checks

This bounded adapter delivery fills exactly three unchanged GH-210 catalog slots.
The original 23 Shivan cases remain unchanged; eight additional cases exercise
legal Cub casting, opponent-turn casting rejection, a single grown Sentry blocking
Shivan, and illegal Cub blocking, each with seats reversed. GH-210 still owns its
50 assigned cases, compositions, holdout and re-execution acceptance. This is not
an M2 gate, full-game or Forge verification claim. The integrated pack also retains
#254’s independently delivered activation-payment case, for 32 cases total.

## Independent expectations and regression

[Oracle](oracle.md) records the catalog and pinned card/rules basis. Literal
expectations were authored before adapter changes. [Compiled red](native-red.log)
shows the old adapter returning boosted Shivan state instead of Cub casting
observations. [Minimized reproduction patch](red.patch), applied to registration
base `3ca995625b32893197000071004efa1835db6eb2`, reproduces that assertion with:

```sh
cargo test -p mtg-core --locked --lib game::shivan_tests::shivan_reference_literal_checkpoints -- --exact
```

Normal Rust discovery executes the derived-observation assertions and strict
schema/omitted/extra/wrong-actor controls; normal Python discovery exercises
strict input validation and every object status/zone/damage comparator field.
No production rule, snapshot, policy or replay format changes. The existing
reference input version remains 1; new exact case IDs require the extended
adapter, and unsupported/malformed inputs fail visibly.

| Unchanged catalog slot | Exact cases (both seats) | Observed acceptance |
| --- | --- | --- |
| `rules-combat-creature-abilities-positive` | `exact_cub_cast_0`, `exact_cub_cast_1` | R/G payment, hand → stack → battlefield, 2/2 untapped/sick Cub, rejected attack |
| `rules-combat-creature-abilities-negative` | `exact_cub_reject_0`, `exact_cub_reject_1` | Explicit opponent-turn cast rejection; unchanged objects, mana, stack and actor |
| `rules-foundations_micro_v1-magnigoth-sentry-interaction` | `exact_single_sentry_0`, `exact_single_sentry_1` | One real Sentry grows to 7/7; five marked damage survives, seven kills real Dragon; life unchanged |

`exact_single_cub_illegal_0/1` additionally rejects a Cub blocking the flying
Dragon. [Preservation checks](preservation.json) retain every previous Shivan
row and expectation, the complete catalog, and the archived GH-210 diagnostic.

## Reproduction

Inside the managed container, prepare/build an isolated pinned XMage cache using
[the existing commands](../../../references/xmage/README.md). Then run:

```sh
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- python3 scripts/shivan_reference.py --cache "$MTG_REFERENCE_CACHE" --output .agent-artifacts/cub-sentry-reference
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- ./scripts/torture.sh
```

The reference command checks the pinned source/toolchain/dependency inventory,
runs all 32 cases twice in native and XMage, and executes omitted, extra and
wrong-actor scripts in both actual adapters. It retains input copies, authored
expectations, actual checkpoints, consumed choices, logs, hashes and named first
divergences. Missing observations, build failures and unrelated control failures
cannot produce an agreement receipt.

## Observation boundary

These are declared synthetic main-phase positions using real pinned cards.
Cub starts with R/G; R pays the generic one. Control age is initialized only at
setup. The bridges execute engine casting, payment, priority, combat and state
based actions; they never install expected results. Single-Sentry Growth is
cast by its defender after blocking, with one Forest tapped at that point.

Checkpoints expose scoped physical objects' zones, current characteristics,
marked damage, tapping and sickness; both players' life and R/G mana; spell
stack, active player and acting seat. Native rejection additionally compares
complete snapshots. XMage rejection compares these scoped fields and observes
whether the submitted declaration/cast was accepted. It does not claim full
hidden-state/RNG equality. New Cub attack and flying-block attempts are explicit.
XMage skips an empty attacker callback; the Cub attack attempt therefore occurs
at that step's priority boundary. Its timing boundary is documented, never
silently skipped. All scripted consequential passes/payment/targets/declarations
are consumed; the fixed sole Growth target is explicit in the bounded adapter.

Combat marks are observed from native applied work before its queued death and
XMage damage events: Dragon receives seven and Sentry five. Settled checkpoints
then require Dragon in graveyard and a surviving 7/7 Sentry with five damage,
with unchanged life. No Shivan activation or two-blocker substitution occurs.

## Integrated validation and delivery

Main advanced during delivery: [PR #260](https://github.com/pabloxrl/mtg-lab/pull/260)
merged as `9a4e8812ecf9fc019cb2c76812058840da89a543`. Shared bridge/native/fixture/runner
conflicts were resolved additively. [Preservation checks](preservation-integrated.json)
verify all 24 current-main inputs and literal expectations are retained, plus the
eight exact cases. The inventory assertions now require 24 + 8 = 32; no behavior
expectation or regression was removed or weakened. Both payment and exact-case
control families and observations remain. This issue adds no activation-payment
behavior; that implementation and its acceptance belong to #254.

The [integrated reference receipt](reference-integrated/receipt.json) records
**32 cases agreeing twice**, with **369 detected controls**, on the integrated
source. All 58 native source hashes and every fixture/runner/bridge/log/observation
hash were verified. Adjacent `0/` and `1/` directories retain actual observations
and consumed choices; the three altered-choice directories retain actual failed
scripts and logs from both engines.

Full shared-lock torture after integration passed: **224 Python tests and 1,380
Rust debug/release executions, zero failed or ignored**. The [integrated record](validation-integrated.json)
hashes the [full log](torture-integrated.log). Existing [six flying/reach](related-integrated/flying_reach/receipt.json),
[thirteen creature-mana](related-integrated/creature_mana/receipt.json) and
[eleven instant-response](related-integrated/instant/acceptance.json) cases also
agree twice on this source. All their native source hashes and 102 instant
artifact hashes were verified. The [complete integrated related archive](integrated-related-artifacts.tar.gz)
retains every raw input/output/log under `related-integrated/`; extract with
`tar -xzf integrated-related-artifacts.tar.gz` to audit receipts.

The documented reference command was exercised; other root quickstart commands
are unchanged by this issue. README adds this scoped capability/evidence link and
preserves #254’s independently reviewed payment-support documentation. A fresh
independent review of the integrated candidate, protected PR verification and
successful exact-main CI remain required before delivery. Their final receipts
are in the [issue workpad](https://github.com/pabloxrl/mtg-lab/issues/255#issuecomment-6083474442).

## Historical pre-integration validation

The following records concern candidate `38fac67218621028411db92761c5e9f1b4580a6f`
on base `5aeab2f10e3aa91a88ac1792fd1f50a1fa744419`; their source hashes describe
that historical tree. [Initial independent review](review-initial.json) passed;
the integrated candidate requires a new review. These records do not substitute
for the integrated validation above.


The [initial reference receipt](reference/receipt.json) records all 31 cases agreeing
twice in native Rust and pinned XMage, including all eight new exact cases.
Its source, fixture, bridge, runner, log and observation hashes were verified.
The adjacent `0/` and `1/` directories retain actual checkpoints and consumed
choices; `omitted/`, `extra/` and `wrong_actor/` retain rejected scripts and both
engine logs. Comparator controls identify the first differing tap, sickness,
damage, zone and other scoped object field. [Toolchain and source pins](toolchain.json)
retain the managed Rust/Java/Maven/Python versions and dependency hashes.

Full locked torture after integrating `5aeab2f10e3aa91a88ac1792fd1f50a1fa744419`
passed: 224 Python tests and 1,348 Rust debug/release executions, zero failed or
ignored. [Validation record](validation.json) hashes the [complete log](torture.log).
The documented reference command is exercised below; other root quickstart
commands are unchanged. README adds only this scoped capability/evidence link.

Existing related packs also agree twice: [six flying/reach cases](related-references/flying_reach/receipt.json),
[thirteen creature-mana cases](related-references/creature_mana/receipt.json), and
[eleven instant-response cases](related-references/instant/acceptance.json),
including their existing negative controls. [Complete related run archive](related-reference-artifacts.tar.gz)
retains all raw inputs, observations and logs under `related-references/`; extract
with `tar -xzf related-reference-artifacts.tar.gz` for receipt hash verification.

Independent candidate review, protected PR verification and successful CI on
the exact main merge commit are required before completion. Their final receipts
are linked from the [issue workpad](https://github.com/pabloxrl/mtg-lab/issues/255#issuecomment-6083474442).

Three initial adapter/control repairs were diagnosed separately: synthetic mana
needed an XMage source; an illegal attack had no selection callback; an omitted
payment needed a precise cursor diagnostic. Their failed logs are retained as
`xmage-initial-failure.log`, `xmage-callback-failure.log` and
`xmage-payment-control-failure.log`. These are not agreement evidence. The revised
approach uses the existing player submission APIs, explicit callback boundaries
and engine-derived damage events without changing production rules or assertions.
