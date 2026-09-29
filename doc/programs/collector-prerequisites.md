# Six independently testable scalar collector contracts

Related to [#158](https://github.com/pabloxrl/mtg-lab/issues/158). This is reviewed
registration intent, not implemented collector capability or an M1 verdict.
It supersedes the oversized **C implementation boundary** in the historical
[trajectory plan](trajectory-prerequisites.md), preserving that history and the
[unsuccessful #154 workpad](https://github.com/pabloxrl/mtg-lab/issues/154#issuecomment-5886104103).
Exactly six pre-created children are authorized. No worker may register more.

## Inspected prerequisites and dependency table

Inspected main: `0f8c51b44dd76943d43bbcd1a7aeac886bfa7a48`. Unlike the earlier
A/B/C registration, canonical v2 memory and durable contracts are now delivered.
The missing work is ownership/composition, not replacement schemas or rules.

| Issue | One observable deliverable | Direct prerequisites | Independently executable component boundary |
| --- | --- | --- | --- |
| #158 | Register/review graph and gated recovery; operations, no RFC ownership | #151 | Graph/ledger/catalog/native-link preservation, full torture and separate review |
| #159 | Owned scalar episode execution and privileged semantic history | #158, #113 | Real Game plus existing semantic encode/apply; no recorder or replay envelope needed |
| #160 | Every owned accepted decision captured into canonical v2 episodes | #159, #152 | Delivered driver plus real v2 Recorder; in-memory episode/readers, no storage |
| #161 | Episode budgets and truthful once-only completion accounting | #160 | Delivered capture, existing work progress and controlled clock; no storage/replay |
| #162 | Completed episode bound to separately authorized replay | #160, #114 | Real captured terminal result and existing played replay in an in-memory resolver; no publisher |
| #163 | Persist actual results with derived run provenance | #161, #153 | Real bounded capture plus existing v2 writer/Manifest; owned sealed bytes, replay reference may be absent |
| #164 | Publish validated dataset and separately authorized replay artifacts | #163, #162 | Compose delivered sealed bundle and real resolver; actual local filesystem tests |
| #154 | Final collector integration audit of EVERY original clause below | #164 | End-to-end real driver/capture/limits/storage/replay/publication; bounded integration repairs only |
| #117 | Original aggregate collector audit, unchanged acceptance | #154 | Repeat original independent played-game acceptance after final integration |

No evidence-backed edge correction is needed. #154's former #153/#114 edges
remain transitive through #164 → #163/#162. #117's earlier #114/#116/#76 remain
transitive through #154, #153 and #152. #159 gets semantic actions and their
policy/opening prerequisites via #113; it uses existing bounded reset/resume,
not a new settlement scheduler. #161 uses existing resumable work and tests
limits at resumable-work and accepted-decision boundaries: existing synchronous
policy submission is atomic, not a promised preemptible instruction budget.
No prerequisite needs an unimplemented future sibling. At each child's dispatch,
its listed predecessors must already have protected delivery and exact-main CI;
this table does not claim all six are delivered now.

Concrete existing APIs (paths relative to the inspected commit):

- [Game](../../crates/mtg-core/src/game.rs),
  [reset/reset_quantum](../../crates/mtg-core/src/opening.rs) and
  [resume/Progress](../../crates/mtg-core/src/work.rs) own authoritative state and
  existing bounded work. [policy_observe/apply_policy](../../crates/mtg-core/src/policy.rs)
  provide seat-filtered structured decisions and reject stale/wrong-seat/version
  submissions. #159 wraps them without exposing mutable Game access; retains
  actual Config/master/episode inputs privately, encodes before mutation and
  appends accepted semantic operations exactly once. An owned result survives reset.
- [actions::encode/encode_concession/decode/apply](../../crates/mtg-core/src/actions.rs)
  from #113 preserve semantic birth/zone identity. Driver tests can re-apply this
  stream directly; #159 does not need #162's replay authorization or envelope.
- [trajectory::v2::Frame::capture, Recorder::new/append/finish, Episode::seat](../../crates/mtg-core/src/trajectory/v2.rs)
  from #152 accept caller-provided before/after frames, full Submission and
  Choice timing/status. They do not attest that an action actually happened.
  #160 owns that association, derives Continuing/Committed/Cancelled timing,
  and keeps the privileged action stream out of policy records. Concession is
  an out-of-band boundary reward, never a fabricated policy decision.
- [played replay record/verify](../../crates/mtg-core/src/played_replay.rs) from
  #114 reconstruct Config/master/ordinal and semantic actions and reject
  nonterminal history. #162 binds that existing envelope to actual captured
  state/RNG, with explicit per-artifact authorization and default denial.
  Unfinished/truncated/failed results have replay unavailable; #162 tests an
  unfinished #160 result without requiring #161's new budget policy.
- [from_core_v2, Writer::new_v2/append_v2/finish_with_metrics, read_v2](../../crates/mtg-recorder/src/lib.rs)
  and [Manifest::load_v2](../../crates/mtg-recorder/src/manifest.rs) from #153
  validate owned v2 records and declarations. #163 derives real provenance and
  inventories from actual inputs/sealed bytes; policy identity/statistics remain
  explicitly trusted caller inputs. Existing `Capture::AllEpisodes` suffices.
  No seed-subset algorithm is added. Failed/incomplete entries have no training
  rows; diagnostic loading never accepts unsealed or corrupt bytes.
- Existing `write_file_v2` publishes one file with a no-replace hard link,
  sync and explicit partial fragment. It does not publish a coordinated run.
  #164 composes that contract with validated manifest/replay artifacts and
  defines collision/retry/error semantics; it need not replace the writer.
  Its atomic visibility mechanism may use the existing hard-link operation;
  tests inject the actual publication operation, not just a hypothetical rename.

Prerequisite receipts: [#151](https://github.com/pabloxrl/mtg-lab/issues/151#issuecomment-5883828758),
[#113](https://github.com/pabloxrl/mtg-lab/issues/113#issuecomment-5860572832),
[#114](https://github.com/pabloxrl/mtg-lab/issues/114#issuecomment-5860763258),
[#152](https://github.com/pabloxrl/mtg-lab/issues/152#issuecomment-5884821342),
[#153](https://github.com/pabloxrl/mtg-lab/issues/153#issuecomment-5885383344).
All are closed completed with acceptance, independent review and protected merge
reports. Their exact-main push CI was independently rechecked successful:
[151](https://github.com/pabloxrl/mtg-lab/actions/runs/36529372323),
[113](https://github.com/pabloxrl/mtg-lab/actions/runs/36357823859),
[114](https://github.com/pabloxrl/mtg-lab/actions/runs/36360076091),
[152](https://github.com/pabloxrl/mtg-lab/actions/runs/36533195460),
[153](https://github.com/pabloxrl/mtg-lab/actions/runs/36538335197).
These receipts establish existing APIs, not future collector acceptance.

## Clause-by-clause ownership and executable checks

The following is the union of #154's Scope, Independent acceptance and preserved
original #117 Deliverable/Required executable acceptance, including #117's
coordinator amendment. Each row retains an implementation owner AND audit
owners **#154 and #117**. Those audits must link concrete normal-discovery tests,
run the composed path, and add missing positive/negative regressions. Child
success alone is never full collector acceptance. Checks here are requirements
for future executable tests, not claims of already executed collector tests.

| Clause from original acceptance | Component owner(s) | Executable positive check | Executable negative check |
| --- | --- | --- | --- |
| Own scalar reset/advance/submission lifecycle | #159 | Normal-reset literal land/creature/combat script reaches independently specified terminal life/board; semantic stream re-applies to full normalized state/RNG | Insert wrong-seat, stale generation/revision, unsupported version and illegal submission; exact snapshot/history unchanged; legal suffix still succeeds |
| Accepted actions cannot bypass history; every accepted decision, no missing/extra decisions | #159 history; #160 canonical capture | Count independently enumerated script operations and decision rows, including all continuations; exactly one accepted operation per history entry | Rejected submission adds no row; missing/duplicate semantic/decision sequence rejected; public API cannot mutate Game outside capture |
| Action-time structured observations, candidates, legal masks, factored domains and full submissions | #160 | Literal action-time ledger checks exact own hand/public state, candidates/mask and ordered full selections for each played decision | Mutate/drop mask, choice or domain in retained record; strict recorder/reader rejects; wrong-seat view must not equal privileged view |
| All microchoices and logical timing, including pending spell/target/payment cancel/retry | #160 | Played cast → target → payment, cancel and retry; assert literal logical IDs, microindices and Continuing/Committed/Cancelled statuses | Repeated/cancelled provisional choice cannot become committed action or increment logical count twice; invalid timing fails |
| Combat, multi-bottom/discard and complete ordered choices | #160 | Reach nontrivial attack/block/damage, ordered bottom and cleanup decisions; independently enumerate small legal domains and full selections | Missing second bottom/discard or illegal allocation fails without state/record mutation; no fabricated flat selected index for factored action |
| Both seats; consecutive/interleaved decisions; same-seat next/final views | #160 | Assert exact P0→P0 and P0→P1→P0 intervals and same-seat final observation | Opponent observation cannot be emitted as acting seat's next input; lost interleaving reward/index caught by literal ledger |
| Rewards/boundaries, once-only returns, nonacting and zero-decision terminal seats | #160 capture; #161 finalization | Rules-terminal and out-of-band concession cases give literal [-1,+1] or reversed return once; zero-decision seat uses unassigned reward | Repeated finish/reset cannot duplicate reward or accounting; a seat not acting last must not lose its terminal reward |
| Normal-reset played game against independent all-decision ledger | #159 execution prefix; #160 capture; #154/#117 full audit | Persist/reload actual completed played game and compare every expected observation/domain/submission/reward/boundary against independently authored script/checkpoints | Deliberately omit/duplicate a decision or change named expected checkpoint; comparison must fail, not bless regenerated output |
| Capture enabled/disabled identical full state/RNG under same seeds/actions | #160 | Paired actual executions have equal full normalized snapshot and RNG after each named checkpoint and at end | Add a state/RNG mismatch to comparison input; equality check fails; disabled mode returns no trajectory-only capture |
| Pending-choice privacy and private-field isolation | #160 policy capture; #162 replay; #164 publication | Hidden-hand/library twins expose identical authorized opponent input; actor alone sees private pending choices; seat readers contain only that seat's observations | Unauthorized replay or opponent-private/seed/history field in policy output rejected; compare both seat endpoints, not renderer redaction |
| Owned buffers and reset lifetime | #159 result; #160 frames; #163 durable bundle | Hold episode A, execute/reset B and overwrite reusable buffers; A's owned frames, result and sealed bytes stay exact | Stale A submission into B fails; attempts to append after finalization or mix episode frames fail |
| Optional policy statistics only when supplied, absence truthful | #160; #163 persistence | Supplied log probability/value survives exact round-trip; unsupplied fields remain absent | Probability-required consumer rejects absent values; no invented zero/log probability for scripted/heuristic choices |
| Explicit limits, budget/truncation/failure accounting | #161 | Independently counted decision/turn boundaries and injectable-clock expiry stop at documented boundary, including pending choice; generous limits match unbounded core script | Invalid limit, capacity exhaustion and injected capture error cannot advance past boundary or report rules completion; terminal-at-limit precedence asserted |
| Every started episode, distinct completed/truncated/failed/incomplete status | #161 episode; #163 run | Literal mixed ordinal ledger accounts once for each started episode, preserves final views and real reasons | Repeated finish/reset, missing/duplicate ordinal or failed entry masquerading as completed is rejected; no fabricated terminal action/loss |
| Bind actual config/version/deck/seat/limit/policy provenance | #163 | Manifest fields equal actual reset/capture inputs and version pins; explicit caller policy identities retained | Mismatched config/deck/seat/version/limits rejected; private seeds absent; caller-declared unrelated provenance cannot replace actual input |
| Persist/reload canonical JSONL exactly, counts/checksums/completeness | #163 | Real completed and deliberately truncated played results round-trip via existing writer/read_v2/Manifest::load_v2; independently count bytes/rows and hash sealed bytes | Corrupt bytes, counts, versions, missing fields or incomplete seal fail; failed/incomplete run rejected by default; diagnostic mode preserves truthful status and only valid rows |
| Backpressure and writer/sink errors propagated | #163 | Bounded block-mode queue drains without losing decisions; supplied metrics record wait; complete seal validates | Inject append/drain/seal/flush error and fail-on-overflow; error reaches caller, original result survives, no successful complete bundle |
| Opaque separately authorized replay linkage reconstructs actual game | #162 | Real completed result resolves with explicit authorization, verifies via existing replay API and equals named independent checkpoints and final full state/RNG | Default denial, wrong ID/run/episode/config/history, corrupt replay, nonterminal history or expired resolver access cannot return another artifact or fabricate a completed replay |
| Publication integrity, corrupt publication fails visibly | #164 | Publish real validated bundle, reopen actual manifest/JSONL and authorized replay, compare independent ledger and reconstructed game | Write/sync/link-or-rename/manifest failure, interruption at each visibility boundary, missing/corrupt artifact, collision/retry or unauthorized replay fails without advertising a valid partial run or clobbering prior run |
| Truncated/failed replay/publication truthfulness | #162 availability; #163 accounting; #164 diagnostics | Explicit diagnostic incomplete run retains valid recorded rows and reasons, replay unavailable | Default completed-run load rejects diagnostic run; truncated/nonterminal history cannot produce a complete replay; unsealed fragment never accepted even diagnostically |

Cross-cutting original delivery clauses are also retained, not silently omitted:

| Clause | Owner(s) | Positive executable/evidence check | Negative check |
| --- | --- | --- | --- |
| Test-first, independent expectations, minimized scripts/seeds; no synthetic substitute siblings | Every child, #154, #117 | Compiled intended behavioral red before implementation, rules/contract-derived literal oracle, then passing normal-discovery regression; retain minimized script and seed | Compile/import error alone rejected as red; a synthetic fake collector cannot satisfy played-game test; deliberately wrong expected checkpoint detected |
| Full torture, fresh-main integration, separate review, protected merge, exact-main CI; relevant references for rules changes | Every delivery | Run full `./scripts/torture.sh` after main integration; clean committed candidate `python3 scripts/symphony/review.py origin/main`; CI on exact merged SHA passes | Any failing/skipped required check, reviewer execution/parse failure, dirty/stale candidate or wrong-SHA CI blocks delivery; no bypass |
| README accuracy, scoped behavior; no CLI/new rules/batch/Parquet/trainer/alternate recorder or policy change | Every child and both auditors | Reviewer checks documented implemented limits against tests and affected quickstart commands | Unsupported collector/M1/full-pool claim or forbidden scope in diff blocks approval; registration changes no product claims |
| Original owners/catalog/gates retained; no completion from a child alone | #158 registration; #154/#117 audit; #20/#120/#21/#22 and later owners | `scripts/check_program.py`, exact preservation comparison and full aggregate clause audit; #22 alone establishes M1 verdict | Remove an original owner, RFC text, catalog expectation or gate dependency: validation/preservation checks fail; incomplete clause cannot be credited by another child's pass |

## Independent test strategy and exclusions

Use small normal-reset scripts with explicit deck/config/seed/seat and literal
rules-derived checkpoints (opening counts, paid land taps, public targets,
combat/life and terminal outcome). Independently write the all-decision ledger;
semantic history is an additional completeness cross-check, never its oracle.
Use CR and the existing documented policy/trajectory timing contracts to justify
expected values. Enumerate small factored domains independently. Retain minimized
failures in normal test discovery. Use controlled clocks and instrumented I/O,
not sleeps or output from the new implementation as expected results.

Declared synthetic resolver-identity and broken-stream fault tests may isolate
components; they cannot replace real played-game acceptance. #159 tests real
execution and action history only, #160 adds real memory capture, #161 limits,
#162 real replay access, #163 real sealed persistence, and #164 real publication.
No prerequisite repeats the obligation to build the entire collector. #154 and
#117 independently audit every row through the completed composition and report
bounded defects; another oversized implementation requires coordinator decision.

All children have only partial B036/B037 ownership. Every original owner and
catalog case stays unchanged. #20 retains full trajectory integration, #120 CLI
wiring, #21 the full CLI/baseline audit, #22 the M1 gate. M2–M5, full-pool,
reference-engine, sharding/fragment-reassembly, batched/autoreset and trainer
acceptance remain with their existing owners; this scalar registration does not
waive or claim them. No CLI, new game rules, sampling algorithm, alternate
recorder, remote authentication/cloud storage, Parquet or unrelated refactor is
authorized. Local replay separation does not promise security against a hostile
host user. No workflow/CI/review-policy/concurrency change or counter reset.

## Registration verification and gated recovery

Validate supported schema, unique tasks, known owners/dependencies, no cycles,
all milestone gate ancestors, and original transitive prerequisites. Compare
against the inspected main: source pins and all 44 verbatim RFC blocks identical;
only append six owners to B036/B037; every 320-case catalog byte and owner intact.
Operations #158 owns no RFC block. Validate native blocked-by links against every
committed edge; group #158 under #7 and #159–#164 under #154, preserving all old
sub-issue links. #154's native dependency replacement is deferred until protected
registration merge and successful exact-main CI, matching the recovery sequence.
New children remain unready before that point. Full managed-container torture,
fresh-main integration and prescribed independent clean-candidate Codex review
are mandatory, with findings/resolutions preserved in the PR. Use Related to.

Only after protected merge and successful exact-main CI: append the reviewed
amendment to #154's entire original body and existing workpad; preserve the full
unsuccessful assessment. Record the explicit operator-authorized deferred
resumption grant from #158. Atomically remove only the resolved sizing
`agent-blocked` label and add `agent-resume-authorized`, preserving all other
labels. #154 remains unready until #164 and normal checks pass. #117's existing
deferred grant stays intact. The coordinator's prior stopped-controller archival
of #154's attempt counter is outside this worker: do not access/reset it.

Refresh current-main allowlist, every task state, parent/current controls,
prerequisite completion/review/exact-main CI and paginated label events. Upsert
verified #158 evidence and current blockers into the single Program workpad.
Select at most the first eligible child (#159 if controls remain unchanged),
using #158's final verified completion report under the pre-closure exception.
Recheck controls immediately; confirm ready landed before atomically closing
#158 with dispatch labels removed. Do no work after closure. Normal handoff
continues through eligible children and both audits without routine approval.
Registration completes neither #154, #117 nor M1.

README assessment: no root README change needed. Registration changes no usable
command, setup, supported behavior, delivered architecture, limitation or verified
milestone. Existing rows correctly say full-game collector remains planned.
Quickstart commands are unchanged; full torture checks the existing underlying
commands in the managed container. Independent review must check this assessment.
Live queue status belongs in the Program workpad, not README.
