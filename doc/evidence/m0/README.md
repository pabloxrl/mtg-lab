# M0 gate audit — GH-16

**FAIL: M0 remains open. M1 is not activated.** Audited main:
`4760ba1d9dc2f7edbc3af4095715298df5c7510e`, 2026-09-26, managed Linux ARM64.
[Correction #52](https://github.com/pabloxrl/mtg-lab/issues/52) requires coordinator
registration and delivery before this gate can be rerun. This report changes no
rules, expectations, program authorization, CI or workflow policy.

The [machine-readable evidence](audit.json) includes input SHA-256 digests, fresh
rules/card fetch receipts, both real reference acceptance receipts, actual
checkpoints, raw-log digests, mutation-fixture digests, and the complete list of
missing capability links. Historical child completion was checked as a prerequisite;
the findings below come from current-main files and fresh executions.

## Exit criteria

| RFC §10 criterion | Finding | Evidence and boundary |
| --- | --- | --- |
| Deck counts validated | PASS | Exact RFC §3 multiplicities, 40 cards per deck (16 lands / 24 spells), 20 distinct cards plus Goblin token, all eight deck/starting-seat configurations. Offline validation and fresh acquisition of all 21 pinned sources passed. No game implementation is inferred. |
| Supported behavior mapped to tests | **FAIL** | Registry contains 80 planned capabilities / 320 positive, negative, interaction and regression slots, but zero test records. All eight neutral fixture files combined mention only 27 capabilities; 53 have no fixture mention. Even these 27 mentions are weaker than category-specific test mappings. |
| Fixture expectations explained | PASS for initial authored corpus | Six original fixtures / 18 checkpoints have independent numbered-rule derivations; review below. Two XMage candidate methods are selected, not adapted fixtures or executed evidence. |
| Both reference harnesses run a headless scripted smoke | PASS | Two repeated real XMage runs and two real Forge runs of the same fixture, one unskipped Java test per run; each engine detects life, actor-choice and expected-priority mutations. Raw logs and actual checkpoints inspected, not just exit status. |
| Legal/data distribution decision recorded | PASS for M0 decision | Conservative source/metadata-only decision in `doc/provenance-policy.md`, selected-field review in `doc/card-manifests.md`, and actual bridge/dependency reviews in both reference READMEs. Public release remains blocked by REL-01/02/03. |

The failed mapping criterion is **not** a demand to implement M1 rules or execute
all future tests in M0. A complete test design can remain explicitly planned.
However, four empty arrays per capability identify categories, not tests. There
is no traceable complete test design in the inspected registry or fixture set.
The registry's schema already permits planned fixture-linked records (see
`test_evidence_requires_exact_reviewed_fixture` in `tests/test_scenario.py`).
#52 requests concrete designs/IDs, independent bases and later owners/stages,
with completeness validation, without promoting plans to passing evidence.
`coverage --require-passed` correctly fails with 320 missing slots; that failure
alone is expected before engine implementation and is not the mapping finding.

## All M0-assigned requirement blocks

The freshly fetched program/inventory validate against their source commit and
SHA-256. The inventory preserves the RFC verbatim. Every distinct block assigned
to an M0 implementation or gate is accounted for here; shared later obligations
remain with their manifest owners. Operations #47 owns no RFC block.

| Block | Applicable M0 evidence / verdict | Retained obligations |
| --- | --- | --- |
| R0002-B007 | Frozen-deck metadata and exact counts PASS; custom format, mirrors and starting seats explicit. | Actual game execution and release scope remain later work. |
| R0002-B010 | Card/printing/Oracle/characteristic hashes and independent RFC deck validation PASS; 21 fresh source projections verified. | All-card behavior and runtime rejection need engine tests. |
| R0002-B011 | Exact CR 2026-09-25 source/header/digest fetched and verified; nine behavior clauses inventoried; current combat migration warning retained. Mapping incomplete as above. | General rules implementation, current combat execution and corpus migration remain required. |
| R0002-B012 | Non-goals and explicit scope rejection represented in schema/metadata validation; no unsupported mechanic silently introduced. | Runtime unsupported-content rejection remains required. |
| R0002-B024 | Original corpus provenance, two pinned XMage candidates with MIT notices/current-vintage audit, Forge external-only distribution decision PASS. | Adaptation admission, independent corroboration and release rights remain required. |
| R0002-B025 | Neutral versioned schema, synthetic/normal-reset distinction, exact scripts/checkpoints/provenance and independent Growth/Bite/cleanup derivation PASS. | Production and expanded reference execution are not delivered by schema validation. |
| R0002-B026 | All twelve families inventoried, but complete capability-to-test mapping FAIL. | Every family needs normal/adversarial behavior tests and actual execution as implemented. |
| R0002-B027 | Initial privacy permutation fixtures and strict comparison/mismatch artifacts PASS at authoring/verifier scope. | Reachable-state properties, fuzzing, replay/batch and expanded differential runs remain later. |
| R0002-B028 | Both separately installed pinned bridges, strict same-fixture smoke, observable fields and actual negative controls PASS at M0 spike scope. | Every capability/card XMage coverage, ≥20 independent dual-reference cases and scripted full matchups remain M2/M5 requirements. |
| R0002-B029 | Existing behavioral detector red/green evidence preserved in child PRs; fresh rejection/self-tests and real bridge mutants PASS. No behavior edited here. | Every future rules change remains test-first with independent expectations, matched references and regression/mutation evidence. |
| R0002-B041 | Smoke subprocesses use closed stdin, unset displays, JVM headless mode and bounded timeouts; actual runs PASS. | Every future automation/CLI/training command must satisfy the same contract. |
| R0002-B042 | M0 exit audit FAIL on test mapping; other four criteria pass within stated boundaries. | No milestone completion; later gates and exact-candidate release verification unchanged. |
| R0002-B044 | Workspace, pins, initial fixtures, comparator and both bridge spikes exist; missing test mapping prevents M0 handoff. | M1 scalar rules/replay/views/CLI and M2 benchmark work remain planned; no early activation. |

## Independent fixture inspection

Expectations were read from the fixtures and checked against the pinned rules
and frozen card definitions, not calculated from reference output:

- Empty stack: CR 117.3d transfers priority on the first pass; 117.4 ends the
  postcombat main after both pass; 117.3a gives active-player priority in end step.
- Bite targeting: its ordered own-creature/opponent-creature target restrictions
  reject reversed or missing slots (601.2c/e); invalid command snapshots cover
  game, RNG, decision and private information. Valid casting retains priority.
- Growth/Bite/cleanup: the responding +3/+3 makes the defending Bear 5/5 before
  Bite resolves. The source Bear remains power 2, so the target survives two
  marked damage. CR 514.2 simultaneously removes damage and expires the boost;
  no intermediate lethal 2/2-with-two-damage state exists. The same battlefield
  object remains. Explicit mana, targets, priority passes and checkpoints agree
  with this derivation; cleanup is a settled internal checkpoint, not a decision.
- London mulligan: CR 103.4/103.5 gives 20 life and seven initial cards; one
  mulligan redraws seven and bottoms one before the next declaration round.
  Six remain, library length 34; 103.8a skips the starting player's first draw.
- Privacy pair: hidden opponent hand/library permutations preserve P0's present
  authorized observation, ordered pass/concede choices, mask and wrong-actor
  error (400.2, 401.2, 402.3 and RFC interface requirements). Empty library alone
  is not a loss. No claim is made that future draws or outcomes agree.

These six cases have **zero production and zero reference executions**. Comparator
self-tests use supplied observations and prove detection, not actual rules
behavior. The separate one-pass smoke derives its expected handoff from CR
117.3d; the bridges read actual life, ordered zones, mana, stack and priority.
Raw smoke checkpoints show P0 → P1 priority with otherwise unchanged exported
state, 20 life each and preserved two-card Forest/Mountain libraries. Five groups
remain explicitly unobservable: object characteristics/status/damage; legal
choices; outcomes; effects/private views; land plays used. One smoke is not a
full-state, full-game, normal-reset or broad-card verification claim.

## Counts, versions and raw evidence

| Population | Planned / authored | Executed / agreed | Unsupported / disputed |
| --- | --- | --- | --- |
| Production capabilities | 80 planned, 320 required category slots, 0 linked test records | 0 / 0 | 80 not implemented; 0 recorded disputes |
| Initial corpus | 6 originals, 18 checkpoints | 0 in production or references | All 6 outside current smoke bridge scope; 0 recorded adjudicated disputes |
| XMage smoke | 1 independent neutral case | 1 / 1 (two repetitions) | 6 initial-corpus cases unsupported; five unobservable field groups; 0 disagreements |
| Forge smoke | Same 1 neutral case | 1 / 1 (two repetitions) | Same gaps; 0 disagreements |
| XMage candidates | 2 selected methods | 0 / 0 | Not yet adapted/admitted/executed |

Zero recorded disputes does not establish correctness of unexecuted behavior.
The registry's evidence counters are separately 0 planned records, 0 executed,
0 passed, 0 failed, 0 unsupported records and 0 unavailable records; these empty
counters do not mean support. `audit.json` names all 53 capabilities with no
fixture declaration. No skipped or expected-failure game tests count as support.

- Rules SHA-256: `8d860e451f20f38865b725b42d82feb714c725373dd8f3b32b8652b3eeb070ca`,
  977752 fetched bytes; card pool `foundations_micro_v1`, revision `2026-09-26.1`.
- XMage `000d8a7abc0ac31cc24af08691423e0c24dc59e7`; Forge
  `95dc682bf92460f49cebd7a9578f06ccf60d5569`; bridge versions 1, no upstream patches.
  Both use Temurin 21.0.9+10 and Maven 3.9.11 on Linux/aarch64. Exact source/archive,
  consulted files, base-image and complete dependency pins remain in the hashed
  `references/*/{pins,dependencies}.json`; no new lock was generated or accepted.
- XMage checkpoint SHA-256:
  `4332776f10e7676561e5ebe736176de585427ea5b973708ef487956febbd89d8`;
  Forge: `77d9ab7da1533005d28ebe36e9335fc695db2d3ce072f299b61fe3fdb647ed32`.
  Repeated positive runs took 20.25s and 22.58s respectively (diagnostic timings).
- Both actual acceptance runners verified cached source/toolchain/dependency
  pins, compiled/executed the Java tests offline and detected all three mutations
  for the specified life/actor/priority reason. Checked raw logs require exactly
  one test, zero failures/errors/skips for each positive run. Repetition checkpoint
  bytes and receipt/log hashes matched. Negative controls are not extra agreed cases.
- `./scripts/verify.sh`: PASS, 108 Python tests and 8 Rust tests, docs/program checks,
  formatting and clippy. No game behavior was changed; fresh red/green is inapplicable
  to this report-only change. Existing detector tests and real mutations were rerun.

## Reproduction

Run from the audited main SHA with the pinned image tools. Logs/card/rules caches
are external, privileged inspection artifacts, excluded from distribution.

```sh
./scripts/verify.sh
python3 scripts/check_program.py
python3 scripts/rules_source.py validate
python3 scripts/rules_source.py fetch-verify
python3 scripts/acquire_cards.py --cache /tmp/gh16-cards
python3 scripts/card_manifest.py --cache /tmp/gh16-cards
python3 scripts/scenario.py validate fixtures/scenarios/m0/*.json
python3 scripts/scenario.py coverage
# Expected exit 2: no production coverage is certified yet.
python3 scripts/scenario.py coverage --require-passed
python3 scripts/xmage.py acceptance --cache /tmp/mtg-xmage
python3 scripts/forge.py acceptance --cache /tmp/mtg-forge
```

Card acquisition needs a new cache directory. This audit reused the previously
built Forge cache. The initial XMage command using `/home/agent/.cache/xmage`
failed explicitly with read-only `acceptance.json`; no smoke pass was credited.
Copying that known reference cache with
`cp -a /home/agent/.cache/xmage /tmp/mtg-xmage` supplied a writable cache, then
full acceptance passed. For an empty reference cache first run each runner's
`prepare` and `build` commands as documented in its reference README. This audit
certifies cached acceptance, not a fresh online dependency-resolution build.
No host software, Docker socket or alternate toolchain was used.

Reproduce the mapping finding independently:

```sh
python3 - <<'PY'
import json
from pathlib import Path
registry = json.loads(Path('data/capabilities-v1.json').read_text())
fixtures = [json.loads(p.read_text()) for p in Path('fixtures/scenarios').rglob('*.json')]
mentioned = set().union(*(set(f['required_capabilities']) for f in fixtures))
capabilities = registry['capabilities']
print('capabilities', len(capabilities))
print('slots', sum(len(c['required_evidence']) for c in capabilities))
print('records', sum(len(v) for c in capabilities for v in c['required_evidence'].values()))
print('mentioned', len(mentioned))
print('unmapped', sorted(c['id'] for c in capabilities if c['id'] not in mentioned))
PY
```

## Unresolved release and delivery questions

REL-01: repository license needs an explicit owner decision. REL-02: Wizards
terms/fan-policy applicability and allowed data uses remain unresolved. REL-03:
exact release payload, dependency/bridge notices and redistribution review remain
open under #40. Separate reference processes do not grant licensing clearance;
Forge GPL-linked runtime, upstream source, Oracle text, artwork and unchecked logs
are not admitted by this report. Original report text, metadata/digests and minimal
semantic smoke observations follow the existing M0 distribution boundary.

Delivery dependencies #9–#15 and operations #47 are closed completed, with linked
acceptance/review/PR/merge/exact-main CI in their workpads. Current manifest has 33
tasks, M0 alone authorized, and gate #16 depends on all eight M0 predecessors.
This failed gate must remain open after report delivery. #52 is a proposed focused
correction, intentionally not dispatched or added to the allowlist by this worker.
There is no independent eligible authorized successor: all other M0 tasks are
completed and M1–M5 are unauthorized. The parent workpad must record the failure,
report PR/merge/CI and coordinator registration requirement before #16 is blocked.
A reviewed correction and explicit coordinator reactivation are required to resume.
The separate report review and exact delivery CI evidence are recorded in the
[GH-16 workpad](https://github.com/pabloxrl/mtg-lab/issues/16#issuecomment-5850984884)
and its PR; merging this evidence report does not complete M0.
