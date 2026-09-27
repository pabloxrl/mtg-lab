# M0 re-audit — GH-16

**PASS for M0 at main `2bd67ca5bbad37589630c3b25e5f5bbef0a8d7e2`.**
Audited 2026-09-27 inside the managed Linux ARM64 container, following delivery
of correction #52 and registration #54 in PR #55. M1–M5 remain unauthorized;
this report activates no task and makes no production-game or release claim.
The [previous failed audit](../m0/README.md) and its receipts are unchanged.

The [fresh machine-readable evidence](audit.json) records commands/exit codes,
49 input digests, all design IDs and six fixture links, rules/card fetch receipts,
actual reference checkpoints, raw-log hashes and inspected test summaries,
negative controls, dependency CI and coverage gaps. The conclusions below come
from current-main inspection and fresh execution, not closed child issues alone.
Candidate review, protected merge and exact-main CI are recorded in the
[GH-16 workpad](https://github.com/pabloxrl/mtg-lab/issues/16#issuecomment-5850984884)
and its report PR. This report-only change alters no behavior or expectation.

## RFC §10 exit criteria

| Criterion | Verdict | Independently inspected evidence |
| --- | --- | --- |
| Deck counts validated | PASS | Exact §3 multiplicities independently transcribed and compared; each deck has 40 cards, 16 lands and 24 spells. Twenty distinct definitions plus Goblin token; all eight ordered matchup/start-seat configurations. Fresh acquisition and offline validation verified all 21 pinned source projections. |
| Supported behavior mapped to tests | PASS at design stage | Separate catalog contains 320 concrete designs: 80 capabilities × positive/negative/interaction/regression. Each has setup, actions, expected result, independent basis, stable ID, owner and stage. Full mapping and six exact admitted-fixture links validate. Execution evidence remains empty and does not establish support. |
| Fixture expectations explained | PASS for initial authored corpus | Six originals / 18 checkpoints checked against rules/card inputs; explicit priority, target, Growth/Bite cleanup, London and privacy derivations below. Zero production/reference execution of these six is retained. |
| Both reference harnesses run headless scripted smoke | PASS | Real pinned XMage and Forge each executed the same neutral case twice, with exactly one unskipped Java test per run. Actual checkpoints, raw test summaries, log hashes and all three intended mutation failures inspected. |
| Legal/data distribution decision recorded | PASS for M0 | Conservative original-code/metadata boundary remains recorded in provenance policy and scoped card/bridge reviews. REL-01/02/03 remain open release blockers under #40. |

No mandatory M0 check was skipped. Neither empty execution arrays nor the expected
failure of `coverage --require-passed` is a design failure: the new separate
catalog supplies the specific test designs missing in the historical audit.
It does not fill those arrays, change `planned` to `implemented`, or replace the
later behavioral, reference and release gates.

## Correction and independent design review

`python3 scripts/test_plan.py` returned 80 capabilities, 320 designed slots,
six related authored fixtures and zero execution evidence added. Independent
set/count inspection agrees: 80 records in each category; 133 M1, 175 M2 and
12 M3 designs. All twelve families and all 21 card/token behavior IDs remain
represented through the registry. The validator's six regression tests passed,
including missing/duplicate slots/IDs, unknown owners/capabilities, stage mismatch,
stale pins/links, missing admitted fixtures and invented passing claims.

Read the concrete catalog against the pinned rules and freshly fetched card
text, including these independent checks:

- CR 103.5 requires each mulligan's bottoming before the next declaration round;
  103.8a skips only the starting player's first draw step. The reset designs
  preserve explicit replacement orders and both seats.
- CR 117.3/117.4 distinguishes retained caster priority, active-player priority
  after resolution, consecutive passes, and only the top stack object resolving.
  No response window appears inside payment or damage application.
- CR 608.2b prevents using information from an illegal targeted Bite source;
  its power cannot be recovered as cached last-known information. This differs
  from a previously created Pyromancer/Archer ability surviving its source under
  113.7a. A partially legal target set and an entirely illegal set differ.
- CR 510.1c permits all six nonnegative five-damage splits between two blockers.
  The Shivan designs use reach blockers and preserve modern combat without a
  legacy blocker-order choice. CR 702.2c/702.19b separately requires lethal
  assignment before trampling: boosted 7/6 Thornweald can assign 1+1+5 against
  two Cubs, takes four simultaneously and survives; against two 4/4 Sentries
  it instead takes eight and dies. Deathtouch alone does not grant trample.
- Stacked boosts use pinned card quantities: Dragon 5/5 + Growth + Surprise +
  one activation is 11/8; two Invoker boosts make Thornweald 12/11. Surprise's
  affected set is fixed on resolution (611.2c). Thrill consumes itself and a
  discard, then draws two: seven returns to seven in the cleanup regression.
- CR 514.2 simultaneously removes damage and expires temporary boosts. Empty
  library alone is not a loss (704.5b). Terminal reward and privacy expectations
  derive from the RFC contracts, not the reference engines.

The plan explicitly discloses synthetic APNAP pending triggers, before-SBA loss
states, repeat-cleanup exceptions and new-generation identity probes. These need
future schema/bridge boundary work and do not prove natural reachability. Compact
instructions to “resolve” require complete scripted passes when admitted as
fixtures. Regression designs identify seeded defect risks, not invented past
bugs. Full frozen-deck games, AI capture/replay, data/RL ledgers and system plans
retain later owners and independent oracle obligations. The current one-pass
bridges cannot replay full games, handle the complete card pool or certify these
320 designs. M2 scalar/card coverage and M3 batch rows remain separately staged.

All six catalog links resolve to `accepted-for-m0` originals and exactly match
fixture IDs, content revisions and declared capabilities; raw file digests are
also recorded. They mean related authored inputs, not that every linked design
category is satisfied by that fixture. Two pinned XMage London candidates remain
selected methods, not adapted/admitted or executed cases. Their MIT notices and
current-vintage caveats remain; Forge test code stays external.

## Initial fixture derivations

Inspected actual setups, scripts, named assertions and provenance:

- Empty stack: first pass transfers P0→P1 (117.3d); second ends postcombat main
  (117.4), then active P0 receives end-step priority (117.3a).
- Bite targeting: own source then opponent destination is required by the pinned
  card definition and 601.2c/e. Reversed or missing target slots reject atomically;
  the valid cast spends 1G and retains P0 priority. Invalid probes retain concrete
  game/RNG/decision/private-information snapshots.
- Growth response: explicitly activated Forest mana pays Bite and Growth; Growth
  resolves first to make P1's Bear 5/5 while Bite remains pending. P0's power-2
  Bear deals two, so P1's Bear survives. Cleanup leaves the same battlefield
  identity as an undamaged 2/2, with no intermediate lethal check (514.2).
- London: two exact green decks; opening seven each, one explicit replacement
  shuffle and one bottom choice leave P0 six cards and 34 in the library. Both
  remain at 20 life. After upkeep passes, starting P0 skips the first draw step,
  preserving its hand and ordered library (103.4/103.5/103.8a).
- Privacy pair: the hidden P1 hand/library permutations leave P0's current view,
  pass/concede candidate order, mask and wrong-actor error identical under the
  RFC knowledge boundary and 400.2/401.2/402.3. No future-outcome equality claim.

Validation and supplied-checkpoint comparator self-tests prove authoring and
mismatch detection, not game execution. The fresh smoke is separate: its
independent CR 117.3d expectation is P0→P1 priority with otherwise unchanged
exported state. Both engines actually returned 20/20 life, zero mana, empty
hands/battlefields/graveyards/exile/stack and the specified two-card library
orders at both upkeep checkpoints.

## Every M0-owned requirement block

Fresh origin/main manifest and verbatim inventory validated against the pinned
RFC source commit/checksum. All 13 distinct blocks assigned to M0 implementations
or gates are covered below. Operations #47/#54 own no RFC block. Shared blocks
retain every later obligation; M0 passage does not complete those blocks globally.

| Block | Applicable M0 result | Retained later obligations |
| --- | --- | --- |
| R0002-B007 | PASS frozen custom scope/decks, mirrors and both starting seats. | Game execution and release claims. |
| R0002-B010 | PASS exact card/Oracle/printing/characteristic pins and source checks. | Full-card implementation and runtime rejection. |
| R0002-B011 | PASS exact CR source/header/digest, nine behavior clauses and complete design mapping; current combat caveat retained. | General rules implementation and deliberate version/corpus migration. |
| R0002-B012 | PASS explicit non-goals and metadata/schema scope rejection. | Runtime unsupported-content rejection. |
| R0002-B024 | PASS original provenance, candidate notices/vintage review and distribution decision. | Adaptation admission, independent corroboration and release rights. |
| R0002-B025 | PASS neutral schema, explicit synthetic/reset distinction, scripts/checkpoints and independent fixture explanations. | Production and expanded reference execution. |
| R0002-B026 | PASS twelve families, 320 concrete category designs and six pinned links after #52. | Admitted executable fixtures and all required passing evidence at owning stages. |
| R0002-B027 | PASS initial privacy pair and strict mismatch/invariant comparator within M0 scope. | Reachable properties, fuzzing, replay/batch and differential campaigns. |
| R0002-B028 | PASS both pinned headless bridges, same smoke and actual negative controls with explicit observability limits. | Every-card/capability XMage coverage, ≥20 independent dual-reference scenarios and full scripted matchups. |
| R0002-B029 | PASS rerun detector regressions and real bridge mutants; historical behavioral red/green preserved. | Test-first behavior changes with independent expectations/reference checks. |
| R0002-B041 | PASS actual smoke with closed stdin, displays unset, JVM headless and bounded process timeouts. | Same unattended contract for all later CLI/training/test commands. |
| R0002-B042 | PASS all five M0 exit criteria at exact audited main. | M1–M5 gates and exact-candidate release verification. |
| R0002-B044 | PASS workspace/pins, initial corpus, comparator, complete test design and both bridge spikes. | M1 scalar/replay/views/CLI and M2 measured benchmark work remain planned. |

## Versions, counts and raw evidence

| Population | Planned/authored | Executed/agreed | Unsupported, disputed and limitations |
| --- | --- | --- | --- |
| Production capabilities | 80 planned, 320 design slots | 0 implemented; 0 executed/passed slots | 80 not implemented; 320 executed-coverage gaps; no support inferred from empty result counters |
| Initial admitted originals | 6 fixtures, 18 checkpoints | 0 production / 0 XMage / 0 Forge | All six unsupported by current smoke-only bridges; no recorded dispute because not run |
| XMage smoke | 1 neutral case | 1 executed / 1 agreed, repeated twice | 0 disputed/unavailable/skipped in fresh run; five unexported field groups |
| Forge smoke | Same 1 neutral case | 1 executed / 1 agreed, repeated twice | Same boundary; 0 disputed/unavailable/skipped in fresh run |
| XMage candidates | 2 selected methods | 0 executed | Not adapted/admitted; not independent corroboration by XMage |

The registry records zero failed/unsupported/unavailable execution records because
it contains no execution records; those zeros do not mean capabilities are
supported. Negative controls are not extra agreed scenarios. Unobservable groups
are permanent characteristics/status/damage, land plays, legal-choice enumeration,
effects/private views and outcomes. Winner/full-state/full-game verification is
not claimed. The bridge scope is one pass, not 80 supported capabilities.

- Rules: CR 2026-09-25, 977752 freshly fetched bytes, SHA-256
  `8d860e451f20f38865b725b42d82feb714c725373dd8f3b32b8652b3eeb070ca`.
  A second direct bounded fetch matched; numbered rules were read from those bytes.
- Cards: `foundations_micro_v1`, revision `2026-09-26.1`, manifest SHA-256
  `567dc050f6c3ad382fc58b634670b8d695ed395a1e31ee50c13c0dde2f27e235`.
  All 21 live selected-printing projections and Oracle text hashes validated.
- XMage source `000d8a7abc0ac31cc24af08691423e0c24dc59e7`; Forge source
  `95dc682bf92460f49cebd7a9578f06ccf60d5569`. Both bridge versions 1, no upstream
  patches, Temurin 21.0.9+10/Maven 3.9.11, Linux/aarch64. Archive, source/card,
  consulted-file, image and complete dependency pins are in hashed reference
  manifests; runners verified them before offline Java execution.
- Fresh XMage checkpoint SHA-256
  `4332776f10e7676561e5ebe736176de585427ea5b973708ef487956febbd89d8`;
  Forge `77d9ab7da1533005d28ebe36e9335fc695db2d3ce072f299b61fe3fdb647ed32`.
  Both positive repetitions were byte-identical within each engine. Exported
  semantic checkpoint arrays also match between engines. Positive pairs took
  18.24s/35.95s respectively; shared-container timings are diagnostic only.
- Both real acceptance commands detected life 19 versus expected 20, wrong actor,
  and wrong expected priority for the intended reasons. Raw log hashes match the
  receipts; positive Surefire summaries report one test, zero failures/errors/skips.
- Full verification passed: 114 Python + 8 Rust tests, docs/program/design checks,
  formatting and clippy. No new red/green is needed for report-only changes;
  existing detector tests and real bridge negative controls were rerun.

## Reproduction and delivery boundary

Use the audited main SHA and managed image tools. No host installation, Docker
invocation/socket access, alternate toolchain or unreviewed pin update was used.
The reference caches already existed; this certifies fresh cached acceptance,
not a fresh online dependency-resolution build. Empty caches require the documented
`prepare` and `build` commands in each reference README before acceptance.

```sh
./scripts/verify.sh
python3 scripts/check_program.py
python3 scripts/test_plan.py
python3 scripts/rules_source.py validate
python3 scripts/rules_source.py fetch-verify
python3 scripts/acquire_cards.py --cache /tmp/gh16-reaudit-cards
python3 scripts/card_manifest.py --cache /tmp/gh16-reaudit-cards
python3 scripts/scenario.py validate fixtures/scenarios/m0/*.json
python3 scripts/scenario.py coverage
# Expected exit 2: no production executed coverage exists.
python3 scripts/scenario.py coverage --require-passed
python3 scripts/xmage.py acceptance --cache /tmp/mtg-xmage
python3 scripts/forge.py acceptance --cache /tmp/mtg-forge
```

Acquisition requires a new card cache directory. Raw card/rules payloads and
reference build/log files remain external; only reviewed metadata and minimal
semantic observations are committed. The audit records the initial read-only
Git fetch failure and successful narrow approved retry; no failed check became a
pass. No rule, validator, CI, review, workflow or authorization policy changed.

All M0 predecessors (#9–#15, #47, #52, #54) are closed completed with linked
acceptance, review, merged PR/commit and exact-main CI workpads. Nine distinct
CI runs were independently re-fetched and confirmed successful push/main runs
on their recorded SHAs (#52/#54 share PR #55). The manifest has 35 tasks and
only M0 authorized; #54 is included transitively through #52. Parent remains
active. Final delivery must recheck controls, complete review/protected merge,
verify the exact merged-main CI and update both workpads before atomic closure.
No successor is eligible within M0 once this gate is delivered; M1 requires
coordinator activation through a reviewed operations change, not this worker.

REL-01 (repository license), REL-02 (Wizards applicability/allowed uses) and
REL-03 (exact release payload/dependency/bridge notices) remain OPEN under #40.
The scoped metadata and source-only bridge decisions do not clear public release
or GPL-linked runtime distribution. No new M0 corrective issue is needed on this
verdict. Later full-game, support, performance and release claims remain unmade.
