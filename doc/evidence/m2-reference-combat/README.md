# Bounded keyword and activated-ability reference pack

GH-210, partial R0002-B010/B011/B026/B028/B030. **Local acceptance PASS. Protected delivery remains conditional on review and CI.**
Original component #24 and gate #26 retain full acceptance. This pack changes
no production mechanic, source pin, catalog expectation or original owner.

The [exact allocation](../../../fixtures/reference/m2-combat-pack.json) preserves
all 50 assigned catalog rows verbatim and maps each to concrete prerequisite or
composition executions. Every row names its observation boundary. The count is
catalog coverage, not fifty distinct scenarios or a whole-card-pool verdict.
The [oracle](oracle.md) records independently justified arithmetic and the inherited
pre-execution holdout. The historical [gap report](prerequisite-gap.md) remains
for provenance; #255 delivered that prerequisite repair in protected PR #261.

## Reproduce in the managed toolchain

Use the issue's isolated cache; do not share a writable XMage source tree or Cargo
target with another worker. The cache is build input/output, not acceptance proof.
If it is absent, prepare and build the pinned reference first:

```sh
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- \
  python3 scripts/xmage.py prepare --cache "$MTG_REFERENCE_CACHE"
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- \
  python3 scripts/xmage.py build --cache "$MTG_REFERENCE_CACHE"
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- \
  python3 scripts/m2_combat_pack.py --cache "$MTG_REFERENCE_CACHE" \
  --output .agent-artifacts/m2-combat
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- \
  ./scripts/torture.sh
```

The pack executes eight suites: Shivan (including the exact Cub/Sentry repair),
haste, trample, deathtouch, flying/reach, Invoker, Surprise and six bounded combat
compositions. Every child runs twice in actual native Rust and pinned XMage.
The joined receipt rejects missing/stale source hashes, changed expectations,
missing cases/repetitions, altered logs and mismatching observations. A failed
child leaves no joined success receipt. No reference build error or unavailable
callback counts as agreement.

`python3 -m unittest discover -s tests -p test_m2_combat_reference.py -v`
checks the assignment/receipt validators and named comparator controls without
launching an engine. The Python tests and the Rust literal-checkpoint composition
test are in normal discovery. Existing normal-reset Driver/Run, quantum, snapshot,
semantic replay, typed trajectory and invalid-action nonmutation regressions
remain in full torture. The new pack implements no rule or alternative executor.

## Observation boundaries

Compositions compare power/toughness/marked damage, life, mana, stack depth and
temporary trample at named activation/resolution/combat/cleanup checkpoints.
The combined cleanup case also observes retained printed flying/reach/deathtouch.
Shivan's repaired cases compare additional payment/zone/tap/sickness/damage fields
in both seats. Prerequisite receipts retain their own narrower observations.

All reference positions are synthetic. Blocker departure is an explicit test
injection; it does not claim to be a played removal spell. Native raw invalid
submissions compare snapshots. Where XMage exposes only constrained damage
choices or an empty target set, the receipt claims that legal domain, not a raw
invalid-command rollback. No full-state, Forge, reference-AI, normal-reset
reference-game, whole RFC-block, scenario-floor or M2 completion claim.

The damage-split and retained-keyword negative controls preserve minimized wrong
observations and name `after-damage` and `after-cleanup` respectively. Wrong
results never become the oracle. Existing prerequisite discrepancies remain
visible under their original adjudications; no waiver is introduced.

## Delivery checks

On integrated main `362aad0eeb5c7ab38d4b710de3f63c266ec48a52`, the documented
pack command passed all eight suites: **101 concrete scripts**, each run twice
in native Rust and pinned XMage. The [joined receipt](receipt.json) maps the
unchanged 50 catalog rows to those executions. The [complete raw archive](reference-runs.tar.gz)
contains both repetitions' observations and logs; the eight adjacent
`*-receipt.json` files retain source hashes and negative controls. Both named
composition mutations were detected at their required checkpoints.

[Full torture](torture.json) passed **244 Python tests and 1,394 Rust test
executions** across debug/release, with zero failed or ignored Rust tests.
The [compressed raw log](torture.log.gz) is retained with its uncompressed hash.
The pinned [reference build receipt](reference-build.json) and
[dependency evidence](dependencies.json) are retained separately.

The original behavioral-red validator logs remain alongside the final evidence.
The first actual composition build exposed Java APIs unavailable under XMage's
Java 8 source contract. Only those API calls were corrected; no expectation or
rule changed. `java8-compile-failure.*` and `reference-first-attempt.tar.gz`
preserve that failure; `composition-first-pass.tar.gz` and
`before-terminal-torture.*` preserve the successful pre-integration checks.
`before-sentry-integration.tar.gz` preserves the passing pack and full torture
before the latest Sentry/instant delivery was integrated and rerun.
Compiler errors are not behavioral-red evidence or reference agreement.

The single [GH-210 workpad](https://github.com/pabloxrl/mtg-lab/issues/210#issuecomment-6076401142)
records the separate exact-candidate review, protected PR and exact-main CI when
available. Local acceptance alone does not complete delivery or the M2 gate.

README assessment: the root command/evidence index gains this pack. Setup, game
support, architecture, quickstart simulations and milestone table are unchanged;
no stronger support or M2 gate claim is made. The new documented pack command
was exercised above; independent review must check README accuracy.
