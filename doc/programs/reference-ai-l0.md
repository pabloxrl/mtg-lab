# Reference-AI corpus and mandatory native L0 replay

Status: **Registered delivery contract, not implemented corpus or measured speed.**
Operator priority of 2026-10-12, coordinator [#318](https://github.com/pabloxrl/mtg-lab/issues/318),
tracking parent [#7](https://github.com/pabloxrl/mtg-lab/issues/7), GitHub milestone 8.
The separate [registry](reference-ai-l0.json) owns namespace `reference-ai-l0`,
stage `L0`, requirements `L0-001`–`L0-018` and final gate #337.

Generate games with the existing XMage and Forge AIs, freeze faithful transcripts,
and replay the **entire admitted corpus after every change** in native L0. A
curated deeper subset adds diagnostics and costly live references. This is the
immediate product priority; it has **no M3/E1a prerequisite**. EV-M3 remains an
authorized separate program, intentionally held; preserve its registry, history,
drafts and controls. Legacy M3–M5/RL stay deferred. Do not change either frozen
40-card deck, twenty nontoken definitions or the existing Goblin token.

## Inspected baseline and first source

Use the verified M2 interfaces: [M2 gate](../evidence/m2-gate/README.md),
[initial terminal admission](../evidence/admission/README.md),
`references/xmage/FullPoolAdmissionTest.java` and `scripts/admission_reference.py`.
Its sixteen native-heuristic/authored games are preserved regressions, **zero
reference-AI quota**. Admission uses `patched-combat-sba-v1` with hashed
`CombatSbaGame.java`, including the reviewed graveyard-order/trample limitations;
never relabel it unmodified upstream. The complete existing regression suite,
all negative controls and independently justified expectations remain required.

XMage is first because it already has strict full-game replay, occurrence and
incarnation bindings, explicit payment/trigger/combat choices and checkpoints.
That is replay feasibility, not proof of AI capture. Read-only source inspection
at pinned XMage `000d8a7abc0ac31cc24af08691423e0c24dc59e7` found:

| Source path | SHA-256 / implication |
| --- | --- |
| `Mage.Server.Plugins/Mage.Player.AI/src/mage/player/ai/ComputerPlayer.java` | `ca3f0b8e0e7aacc7475e862a7e967ca68c45be9d38b9bb7ac8ee6d80bbeb35e2`; base priority always passes and combat selection is empty. It is not the gameplay bot. `chooseMulligan` suppresses mulligans in test mode. |
| `Mage.Server.Plugins/Mage.Player.AI.MAD/src/mage/player/ai/ComputerPlayer7.java` | `640b75e2c472f53aa901ab1709e526204d0c66980f8498ae4bed5a7e450da12b`; concrete server bot extends ComputerPlayer6, searches simulation copies and executes selected actions. Capture only committed live-game choices, never search branches. |
| `Mage.Tests/src/test/java/org/mage/test/player/TestComputerPlayer.java` | `9386e731ad7dfbf29a85962b8030f1812adf8615af17171dd926bec502e4a10a`; test wrapper redirects inner callbacks to scripted TestPlayer. Existing use does not demonstrate AI generation. |

Select ComputerPlayer7 as the first concrete AI, pin skill/search configuration,
source/toolchain/bridge/patch hashes and RNG configuration. Observe delegated
choices and actual chance results; do not replace search or tactical decisions
with our heuristic. The capture child must prove complete real play, preservation
of choices with recording, unsuppressed opening behavior, and separation of
simulation branches. Upstream timing-based search can affect regenerated games;
the immutable captured choices/chance, not seeds alone, define replay.

Forge at `95dc682bf92460f49cebd7a9578f06ccf60d5569` has
`forge-ai/src/main/java/forge/ai/PlayerControllerAi.java` (SHA-256
`a8c0efea0a55b46416fbaccbcf7aa075f5f2ff13e579fa051f5a3334bb381206`).
It constructs `AiController` and exposes combat allocation, choice, trigger and
payment hooks. The current `references/forge/NeutralSmokeTest.java` uses a mocked
test controller, synthetic basics and one pass before stopping. It proves neither
AI generation nor full-game replay. Forge needs separate raw capture, opening,
casting/payment, abilities and combat/terminal consumers before normalization
and pilot qualification. Inspection is not execution evidence; each child owns
working source-game tests, not placeholder parsers. Existing toolchain and source
pins remain inputs; record/review changed integration hashes without a silent
upstream correctness claim. No preparatory architecture task precedes capture.

## Common capture, admission and provenance contract

Every applicable child and the final gate must retain these requirements:

- Actual source AI chooses gameplay. Capture immutable original traces and actual
  shuffle/mulligan chronology, physical copies/incarnations/tokens, modes,
  targets, costs/payment, ordering, combat allocations, passes, trigger context,
  intermediate observations and rules-terminal results as they occur. Record
  source/AI configuration, source/toolchain/bridge/patch pins, deck/card/rules
  hashes and seeds. Missing observations or unsupported callbacks fail visibly.
- Align semantic checkpoints at opening decisions, priority changes, spell/ability
  resolution and SBA, combat, cleanup and termination. Required state includes
  actor/turn/step, life, ordered zones, physical identities and characteristics,
  stack/targets, mana and active effects. Outcome-only checks cannot pass.
  Exact choices and chance must be consumed; never overwrite computed state to
  manufacture agreement. Privileged raw traces stay separate from AI inputs.
- Keep every started attempt, including failed/truncated/unsupported/divergent/
  duplicate attempts and earliest difference. Complete distinct rules-terminal
  games only; natural rules draws are valid, concessions/timeouts are not filler.
  Canonical gameplay/chance content defines identity: fixed seat roles, deck/card
  definitions, chronological normalized choices and actual chance order with
  stable physical-copy/incarnation/token bindings. Exclude source UUIDs, seed,
  timestamps, filenames, engine labels and diagnostic formatting from this
  identity. Version normalization and hash collision-resistant canonical bytes;
  compare canonical content on a suspected duplicate. Keep all provenance on
  duplicates. Source attribution is deterministic first admitted origin, recorded
  explicitly; a global identity contributes to only one source quota.
- Ten distinct complete games per source cover RG/GR/RR/GG and both starting seats
  (eight rows), each replayed twice. Then account for a declared 100-attempt
  campaign per source. Pilots may overlap the final corpus only with explicit IDs
  and accounting; repeats are never new games. XMage pilot qualification is
  provisionally native/patched-XMage only. Final admission rechecks it in all
  three engines; provisional games and full admissions are separate manifests.
- **Every final admitted game executes in native and both pinned reference
  adapters**, with required intermediate checks. A rules-correct native result
  differing from an upstream bug requires visible original results, rules basis,
  independent review and a minimized regression; it is an exception, not agreement.
  Missing observability/adapter support is not an adjudication exception. Never
  blindly capture native golden output or update a baseline to erase a failure.
- Publish immutable versioned manifests and deployable, checksummed local artifacts
  incrementally as games are accepted. Follow the existing conservative
  [provenance policy](../provenance-policy.md); preserve source/license notices
  where applicable, and keep bulky upstream installations outside corpus artifacts.
  Initial corpus: >=1,000 globally distinct games, >=500 truly generated by each
  AI. For each source's first 500, every row has 62 or 63. More attempts compensate
  for duplicates/failures, never change the admission contract or quotas.

## Mandatory fast L0 and deeper analysis

Ordinary L0 uses a **prebuilt release native runner**, shares immutable inputs and
batches games in a long-lived process; parallelism stays within measured resource
limits. It invokes no Java/reference engine, AI, network/model or generation.
Regeneration and all-three reference admission run occasionally offline from the
ordinary development path. Every admitted entry executes after every change,
alongside existing regressions. Optional local subsets are feedback aids only.

Assert exact action/chance consumption, legal progression, identities, all required
ordered-zone/effect/stack/life and other semantic checkpoints, and outcomes. Compact
canonical checkpoint records/digests may replace verbose storage only when they
cover identical required semantics with schema/pins, collision-resistant hashing
and first-field localization. Rich raw internals and expensive diagnostic streams
belong to deep analysis; no required semantic assertion moves out of L0.

Validate manifest completeness and every artifact/checksum. Missing/unavailable
files, skipped entries, failed build, incomplete replay or mismatch fail the gate.
Persist seed, original trace, cursor and first-field difference; support targeted
deep reruns without changing the failed L0 verdict. No hidden network retrieval,
debug-instrumented reconstruction or automatic baseline replacement.

Measure **cold process start + load + checksums + validation + full native replay
+ comparison wall time**. Report compilation separately, as well as warm throughput,
generation, conversion and JVM startup. Publish corpus size/successful games,
actions/checkpoints, p50/p95 per-game times, total elapsed, peak memory, artifact
bytes, concurrency, source/runner/schema pins and hardware. #323 proposes a budget
from measurements; separately reviewed operations #332 establishes/enforces the
latency budget and regression thresholds. Initial 1,000-game purpose is seconds
to low minutes of interactive development time, not hours. Report an actual miss
and optimize preserved assertions; no fabricated speed claim or quota reduction.
#331 repeats measurement for Forge and #337 validates the actual full-sized result.

#336 deterministically selects a versioned deep subset by mechanic, interaction,
rare-case coverage, both sources, matchup rows and independent holdout provenance.
Size follows coverage and measured cost. Execute richer internal checks, expensive
invariants and live-reference reruns; AI output is never the oracle. Preserve the
independently authored holdout, >=20 critical dual-reference scenarios and all
existing negative controls/minimized defects. #332 defines affected-rules,
corpus-admission, dedicated-run and final-gate scheduling. Existing torture,
independent review and protected merge remain mandatory until explicitly reviewed
equivalent-or-stronger policy is delivered; feature children cannot change CI.

## Requirement ownership and runnable delivery boundaries

The registry contains full deliverables, executable positive/negative acceptance
and exclusions. All children additionally require verified #26, #277 and #318.
These are actual already-delivered M2 contracts and registration, never M3. Each
row below owns its requirement and the relevant common contract above; #337 audits
**every row and every common clause**, including cross-feature acceptance. Source
capture owns faithful raw observations; normalization/pilots own strict replay;
runner/measurements/policy own after-change L0; admission/campaigns own global
accounting and all-three qualification; deep suite owns coverage/holdout.

| Issue / requirement | Observable delivery | Direct child prerequisites |
| --- | --- | --- |
| #319 / L0-001 | XMage real-AI raw game capture | None |
| #320 / L0-002 | Normalize actual XMage AI games into strict replays | #319 |
| #321 / L0-003 | Qualify ten distinct XMage AI games | #320 |
| #322 / L0-004 | Release native batch replay and immutable L0 artifacts | #321 |
| #323 / L0-005 | Measure a 100-attempt XMage campaign and full native replay | #322 |
| #324 / L0-006 | Forge real-AI raw game capture | #319 |
| #325 / L0-007 | Forge ordered opening replay from actual AI tapes | #324, #320 |
| #326 / L0-008 | Forge priority, casting and payment replay | #325 |
| #327 / L0-009 | Forge activated and triggered choice replay | #326 |
| #328 / L0-010 | Forge combat, cleanup and terminal replay | #327 |
| #329 / L0-011 | Normalize Forge AI transcripts for all-three replay | #328 |
| #330 / L0-012 | Qualify ten distinct Forge AI games | #329 |
| #331 / L0-013 | Measure a 100-attempt Forge campaign | #330, #322 |
| #332 / L0-014 | Operations: wire complete L0 and reviewed deep scheduling | #323 |
| #333 / L0-015 | Incremental all-three corpus admission and publication | #321, #330, #322, #332 |
| #334 / L0-016 | Incrementally admit 500 distinct XMage AI games | #333, #323 |
| #335 / L0-017 | Incrementally admit 500 distinct Forge AI games | #333, #331 |
| #336 / L0-018 | Deterministic curated deep replay and live-reference subset | #333 |
| #337 / all | Integrated corpus correctness/performance gate | Every child #319–#336 |

First visible output is #319's real captured game, then #320's strict normalized
replay. #324 can capture Forge independently after #319; heavy builds/reference
runs remain serialized. #322 makes provisional pilot games usable immediately;
#333 promotes all-three-verified games to incremental final L0 versions before
500/source campaigns finish. Campaign workers serialize manifest publication via
normal main integration and preserve each other's entries; no shared writable
reference tree or Cargo target. A child owns one observable deliverable; if an
unforeseen prerequisite exceeds it, report concrete evidence for coordinator
registration rather than silently widening scope or hiding failures.

## Registry validation and locked bounded handoff

Fetch main and run `python3 scripts/check_l0_program.py --ref origin/main` before
child work, push/merge and handoff. #318 alone validates the candidate without
`--ref` until protected registration merge. `rfc`/`rfc_sha256` in the shared schema
pin this contract's bytes (not the historical RFC0002). Validation checks separate
namespace/authority/milestone, no legacy or M3 tasks, full requirement ownership,
acyclic real dependencies and final gate coverage including operations. Never
change legacy authorization or use candidate metadata to authorize children.

The unchanged WORKFLOW controls apply, with this registry replacing the legacy
allowlist for L0. #318 may activate only the first eligible capture child after
protected registration and exact-main CI. Successive successful children perform
one bounded handoff without another routine approval. #337 ends the L0 queue;
it never resumes M3 or closes parent #7. Feature workers cannot register tasks.

1. Prepare acceptance/review/PR/merge/exact-main-CI receipts. Under the existing
   `handoff` lock, in one bounded script, fetch current main, validate its registry,
   and re-fetch parent/current/all task/prerequisite states, paginated label
   events and **all repository** open ready/running claims. Verify dependency
   completion workpads, separate review and successful main push CI on the exact
   protected merge from GitHub. Bare closure or cached completion flags fail.
2. Parent closed/paused/cancelled or current closed/held/blocked/unready stops work
   using WORKFLOW's stop procedure. Never clear controls or restore authorization.
   Count unique other open ready/running claims across all programs against the
   existing three-worker limit. Preserve M3 holds. A failed current task satisfies
   no dependency; independently eligible work may continue before blocking it.
3. Upsert this issue's requirement IDs, outcome, limitations and receipt links in
   the single parent Program workpad under the lock, preserving all other entries.
   The optional read-only `--snapshot PATH` selector plans at most one eligible
   child in manifest order; its input is verified live facts, not their proof.
4. Record the successor reservation before activation. Re-fetch current/parent/
   target/dependency controls, claims and events immediately before each mutation.
   Prior ready removal needs an existing coordinator `agent-resume-authorized`
   grant, consumed atomically with adding ready; never clear held/blocked labels.
   Confirm parent write and readiness. A retry uses its recorded reservation;
   an already-ready reserved task is successful handoff, not another free slot.
   A held/withdrawn reservation requires visible reconciliation.
5. API failure or unconfirmed handoff leaves this task open/ready for retry. Record
   no eligible successor/capacity explicitly. Current successful work may satisfy
   its own dependency before closure only with final verified receipts; #318 uses
   that exception for registration launch. Close the current issue with the one
   atomic completed-state/label update only after confirmed handoff. Failed work
   follows the unchanged unsuccessful handoff/block procedure.

Snapshot format and receipt fields are the existing shared selector contract in
[the M3 guide](engine-validation-m3.md#registry-validation-and-bounded-worker-handoff):
`issues`, `open_claims`, paginated `events`, `current`, `current_success`, optional
`reserved_successor` and `receipts`. For L0 the prerequisite set is #26/#277/#318;
no #241 or M3 issue is required. Never treat the shape validator as proof that
GitHub evidence is genuine. Shared selector changes retain M3 tests and semantics.

## Delivery and completion claims

All children need behavior tests with independent expectations in normal discovery,
full locked torture after latest-main integration, independent exact-candidate
Codex review including README accuracy/no weakened tests, protected merge and
successful exact-main CI. Persistent managed workspaces, resource/handoff locks,
three workers and GitHub-only integrations remain unchanged. This registration
changes documentation and read-only validation/control selection only: no engine,
capture implementation, controller deployment, settings or credential workaround.
Current checkout AGENTS points workers here even with an older rendered prompt.

#337 must publish true-origin counts, successful **full-corpus** cold L0 execution
and runtime/budget result plus deep-suite/live-reference receipts, requirements,
coverage gaps, upstream exceptions, hashes and exact candidate/main evidence.
Neither registration, raw capture, pilots, syntax checks nor a subset establishes
large-corpus acceptance. Existing M2 completion and historical evidence stand.
