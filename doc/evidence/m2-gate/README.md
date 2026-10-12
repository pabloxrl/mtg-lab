# M2 complete-pool and scalar-baseline gate — GH-26

Audit target: exact main `c212c4e4914eca700875d31cd20df6ef1d32c41d`.
**Acceptance verdict: PASS for M2 / E1 at the audited source.**
Milestone completion is conditional on separate candidate review, protected merge
and successful CI on the exact merged main commit, recorded in the [single workpad](https://github.com/pabloxrl/mtg-lab/issues/26#issuecomment-6116279928).

This is the independent M2 gate under the accepted
[engine-validation scope](../../programs/engine-validation.md), also called E1.
It audits the complete frozen red/green decks and reproducible scalar baseline.
It does not certify RFC 0003's runtime card catalog, the later Forge/XMage
AI-generated corpus, universal Magic, designated-host performance or the
deferred M3–M5 learning/batching/interactive work. No new RL work is included.

## Prerequisites and evidence method

The freshly fetched main manifest, pinned RFC bytes and lossless ledger validate:
143 tasks, only M0–M2 authorized, and every M2 implementation/operations task in
the gate's transitive dependencies. All **50 M2 prerequisites** are closed with
`state_reason: completed`. [Dependency receipts](dependencies.json) retain their
completion workpads, merged PRs and freshly queried successful main-branch push
CI on each exact merge. [Final candidate reviews](dependency-reviews.json) are
the actual structured results preserved in those PRs; every reviewed tree equals
its delivered merge tree. Intermediate failed reviews are not selected as final
acceptance. #207's original cancelled main runs are not passes; its subsequently
reviewed receipt PR #239 and successful exact-main CI establish delivery.

[M0 and M1 gate receipts](previous-gates.json) were separately re-fetched.
Their historical evidence and all normal-discovery regressions remain intact.
The [mechanics audit](../m2-mechanics-audit/README.md),
[scenario/reference audit](../m2-coverage-audit/README.md) and
[instrumentation audit](../m2-instrumentation-audit/README.md) supply the detailed
component crosswalks; their closure alone is not this gate's acceptance.

This change adds a gate report and reproducible evidence checks, not rules or
expected outcomes. No artificial behavioral red is claimed. Original compiled
reds, independent card/CR expectations, minimized failures, failed audit history
and review corrections remain in the child reports. No test, fixture oracle,
source pin, manifest ownership, workflow or enforcement policy changes here.

## Every M2 exit criterion

| RFC §10 exit | Evidence and acceptance boundary |
| --- | --- |
| Full scoped suite passes | Full `./scripts/torture.sh`, including verify, normal Python discovery, all workspace Rust debug/release tests, formatting and strict Clippy. Real native CLI/Driver/Run games cover RG/GR/RR/GG × both starters × both policies, genuine rules endings, capture/replay/quantum equality and invalid-action nonmutation. The [local validation receipt](validation.json) and raw archive are recorded below. |
| At least 100 reviewed scenarios | **123 distinct semantic selections**, individually inspected against the [case ledger](../m2-coverage-audit/scenarios.md), literal inputs/oracles and named tests. All twelve required families are represented. Repeated runs, mirrored seats, symmetric allocations, aliases, twelve mutants and sixteen complete games add zero to this floor. Eight selections are native API contracts, not XMage agreement. |
| Designated mutants caught | The [twelve named production mutations](../../testing/m2-mutations.md) require a passing unmodified baseline, successful mutant compilation, exact test discovery and failure at the named assertion. Compilation/import errors, timeouts and unrelated panics never count. Existing original and #24 current-source receipts remain preserved; all twelve gate reruns also passed baseline and detected the intended mutant. |
| XMage for each card/capability | All twenty nontoken cards have consumed/resolved witnesses in [normal-reset admission](../admission/README.md); Goblin creation/cessation is covered by Fodder/Surprise and token histories. The complete catalog retains 175 native mappings, 173 bounded XMage mappings and two native-only capacity mappings. All 80 registry IDs are accounted for, including explicit native API parity limits and three deferred batch IDs. |
| Reproducible scalar performance/memory artifact | Hash-verified raw windows, measured binary/source/toolchain pins, independent OS RSS, warmup/distributions and reproduction commands in [the original baseline](../full-pool-baseline/README.md) and [four-mode campaign](../four-mode-measurement/README.md). Recomputing all 100 later windows gives 7,204 attempts/completions and 7,181,389 decisions; all committed-action rates and four 10,000-state memory curves reconcile with raw data. These are historical pinned measurements, not timings of this audit head. |

## All sixteen M2-owned requirement blocks

IDs abbreviate `R0002-`. Shared blocks pass only their applicable M2 scalar
clauses; this table neither removes later clauses nor declares whole blocks
globally complete. Original source text remains authoritative.

| Block | M2 acceptance evidence and retained obligations |
| --- | --- |
| B008 metrics | Natural-completion numerator, all-attempt denominator, decisions, per-row outcomes and **committed** logical actions are independently recomputed. Cancelled choices are excluded from completed-action rates. Marginal resident capacity and bounded sampled latency are separately reported. Physical-core/host scaling and learner throughput remain unqualified. |
| B010 frozen scope | Exact two 40-card decks, 16 lands/24 spells each; twenty printed definitions and Goblin token, Oracle/printing/checksum pins and no runtime API/artwork dependency. [Card-by-card index](../m2-mechanics-audit/cards.json) plus actual resolved admission witnesses; all eight ordered matchup/starter rows. |
| B011 scoped rules | [Complete mechanics clause crosswalk](../m2-mechanics-audit/README.md#executable-and-reference-crosswalk) and current reference audit cover setup/mulligans/first draw, all turn/priority/stack/SBA work, mana/additional costs/modes/targets, six keywords/current combat, triggers/APNAP, token/incarnation/effect lifetime, life/empty-draw/concession/simultaneous loss and repeated cleanup. Explicit synthetic APNAP/exceptional-cleanup cases do not claim normal-game reachability. General layers and unsupported cards remain rejected. |
| B014 architecture/language baseline | One authoritative Rust engine, measured allocations and boundaries, explicit source fingerprints and no reference delegation in production. Actual profiles identify SHA-256/JSON/allocation costs; no Rust-versus-Python speed claim or second engine. Runtime declarative card migration belongs to E1a. |
| B016 core contract | Encapsulated cast/target/payment state, owned quantum continuation, strict decision generations, snapshot/restore and semantic replay. New trigger/activation choices preserve atomic costs and private pending selections. [Activation-payment repair](../activation-mana/README.md) covers land/creature sources after target selection, cancel/commit, overflow and stale/wrong-seat rejection. Batch fast paths remain deferred. |
| B019 provisional budgets | Actual scalar rates and counters overhead, four 10,000-core-state curves, repeated windows and declared shared ARM64 resources are retained. 100k decisions/s and historical heuristic ≤5% overhead are missed, not waived or represented as met. Missing hardware counters/designated x86-64 host preclude performance qualification; the RFC explicitly does not block first correctness acceptance on an unvalidated speed hypothesis. |
| B020 benchmark tracks | Native heuristic/legal-random rollouts include reset, policy and failures; encoded/unencoded comparisons retain the same workload. Fixed validated traces separate application/legal-view/encoding and witnessed effect dispatch; allocation and resident/stress samples cover normal/token/target/stack cases. Raw warmups, windows, variance extensions, metadata and limitations remain. Worker/batch/binding/inference/interactive and durable-storage qualification are deferred. |
| B021 instrumentation | Off/counters/sampled_trace/full_replay, fixed-cardinality local counters, bounded histograms/diagnostics, explicit drops/overflow, complete-or-fail replay and independent trajectory capture. Literal injected clocks/counts and mode/capture semantic hashes are retained. Common benchmark profiling clocks qualify overhead interpretation; separate latency buckets are not window histograms. No hidden-data metric labels, synthetic zero for unavailable queues, or discarded requested capture. |
| B026 twelve families | [123-case ledger](../m2-coverage-audit/reviewed-scenarios.json) covers setup (5), priority (4), costs (26), targets (9), triggers (20), combat (24), continuous changes (13), objects (10), terminal (5), decisions (3), privacy (2), scalar replay (2). Normal/adversarial choices and independent explicit 64-subset/six-split/capacity oracles remain. The family named “Replay and batching” certifies scalar replay only. |
| B028 reference contract | Same neutral physical/chance/choice inputs drive separate pinned native/XMage executions with no AI/default choices. Intermediate ordered zones, life, mana, stack, characteristics, targets, identities/effects, actors and outcomes are compared within declared boundaries. Sixteen complete games cover all eight rows twice, with literal independent checkpoints, repeated execution and wrong/missing/extra-input controls. Per-capability planned/executed/agreed/disputed/unsupported counts and the two upstream exceptions below remain explicit. Forge breadth, ≥20 independent dual-reference critical cases and the AI corpus remain later E2–E5/#241 obligations. |
| B029 test-first/reference/mutation discipline | Child compiled reds and independent rules/card oracles are retained; normal discovery keeps every behavior and minimized defect. Fresh scoped regression/reference execution and the named-mutant contract supplement rather than replace those expectations. Report-only changes create no new rules red. |
| B030 anti-overfitting/execution | Full discovery has no skipped advertised capability; debug/release plus capture/quantum/snapshot/instrumentation comparisons remain. Four independently authored pack holdouts stay frozen. They are known validation inputs, not a newly secret holdout or the future dual-reference suite. Counts alone are not the gate. |
| B036 stable episode semantics | Full-pool policies and repaired activations retain per-seat sparse terminal rewards exactly once, natural terminal/concession versus external truncation/failure, genuine final views, explicit reset and policy versioning. [Scalar trajectory audit](../trajectory-integration/README.md) and real full-pool recorder tests retain same-seat/opponent interleavings and terminal nonacting seats. No new learning API or wrapper behavior. |
| B037 trajectories | Typed conversion and expanded choices preserve exact owned v2 records through Driver/Run, semantic replay and JSONL publication/reload. Privacy, cancellation, snapshot, quantum/capture equality and write/interruption/corruption tests remain mandatory. Sharded Parquet, numeric/Python readers, trainers and batched recorder qualification remain deferred. |
| B039 unattended client | Both native policies cover every full-pool decision through existing semantic actions; unknown content/version, invalid scripts and missing choices fail explicitly. Closed-stdin/no-display command tests, budgets, signals and complete outcome accounting remain. No CLI IPC in the rules loop; persistent interactive protocol is deferred. |
| B042 milestone/release gates | All five M2 exits above, retained M0/M1 evidence, runnable documented system, full local regression and required protected CI. This gate stops legacy handoff after M2; #241 owns reviewed registration of E1a/E2–E5. Exact-release dual-reference/corpus acceptance is not inferred from M2. |

## Reference disagreements and limits

The [capability table](../m2-coverage-audit/capability-accounting.json) and
[22-ID supplement](../m2-coverage-audit/registry-supplement.md) account for all
80 registry IDs without turning native-only concepts into reference agreement.
The two native-only catalog mappings are representation capacity contracts.
The supplement distinguishes four native-only API concepts from three deferred
batch concepts; none is a silently missing scoped game-rule bridge.

Two adjudicated upstream discrepancies remain visible:

1. XMage's simultaneous graveyard owner ordering required the separately hashed
   `CombatSbaGame.java` integration. CR404.3/704.3/101.4 and the preserved original
   `shivan-multiple`, `before/304`, `$.graveyard[1][0]` mismatch justify the repair.
   Current played agreement is **patched-reference** evidence, not agreement
   with unmodified XMage. Both owner orders, physical-copy swaps and invalid
   order atomicity have native regressions.
2. XMage's trample dialog exposes a total minimum but accepts an illegal
   per-blocker allocation. CR702.19b and the independent combat oracle require
   lethal assigned to each relevant blocker before excess player damage.
   The strict adapter enforces this witnessed constraint; that rejection is
   **adapter-enforced**, not an unmodified-upstream raw rejection.

The [original evidence and adjudications](../m2-coverage-audit/README.md#upstream-exceptions-and-provenance)
remain unchanged. No unexplained divergence, missing reference or invisible
required field counts as a pass. Selected-action legality and native raw
rejection do not imply complete cross-engine legal-set/API equality. Synthetic
setup, normal-reset scripted games and future reference-AI-generated games are
three distinct evidence classes. The sixteen admitted games explicitly keep
seven; the separate 32-case London family supplies real mulligan/bottom coverage.

## Measurement interpretation

The original memory estimator yields 4,902.0928 / 5,675.4176 / 5,216.6656 /
7,261.3888 marginal bytes per core state for typical/token/target/stack classes.
All four reach 10,000 states in three cycles; allocator retention remains
visible. These exclude optional policy/history/replay/capture/client buffers,
are not full concurrent-client capacity and prove neither leak freedom nor
allocation-free play. OS RSS is independent of inline/serialized `Game` size.

The later four-mode campaign retains all 18 invocations/100 windows, including
variance extensions, across sixteen policy/encoding/mode configurations.
All 7,204 measured attempts naturally complete; repeated seeds across modes
are repeat executions, not 7,204 distinct scenarios. Warmup is separate; each
configuration keeps ≥10 seconds warmup and ≥5×30-second measurement windows.
The [instrumentation audit](../m2-instrumentation-audit/README.md) explains why
historical `logical_actions_per_second` counts begun groups and publishes the
correct committed rates from untouched raw counters. Off counters are unavailable,
not zero. Shared-host rates and common profiling clocks remain diagnostic.

The measured binaries keep their original hashes and commits; newer engine
fingerprints are never inserted into old measurement artifacts. This gate
verifies reproducibility/accounting and reruns current correctness checks; it
does not claim a new performance campaign. Cache/hardware events, multiworker
scaling and designated-host budgets remain unmeasured/unqualified.

## Reproduction and final validation

Run inside the managed toolchain, from the repository root:

```sh
python3 doc/evidence/m2-gate/check_evidence.py
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- ./scripts/torture.sh
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- python3 scripts/full_pool_reference.py --family admission --cache "$MTG_REFERENCE_CACHE" --output .agent-artifacts/m2-gate-admission.json
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- python3 scripts/m2_mutations.py --output .agent-artifacts/m2-gate-mutants
```

Reference cache preparation/build and the four pack commands remain the existing
documented tools. The cache is issue-isolated build input, not evidence. Extract
privileged replay archives only into private diagnostic directories. The first
command checks inherited hashes/archives and independently recomputes accounting;
it does not run engines or replace semantic review/full torture.

The full gate torture invocation passed 397 Python tests and 762 Rust tests in
each of debug and release, with zero ignored Rust tests. All 23 distinct native
functions selected by the 123-case ledger passed in both profiles.

The initial cold XMage build exited -9 after compilation and before its smoke
check. Its log and container resource counters are retained: the build permits
a 6 GiB Maven heap in the 7 GiB container, and the counters recorded memory kills.
This is consistent with memory pressure, not a successful reference result.
The unchanged command passed on retry using completed compilation outputs;
no tests, dependency locks, source pins or reference expectations were changed.

[Execution commands, timings and source hashes](execution.json) record ten
successful commands: pinned XMage build/smoke, cost/combat/trigger/foundation
packs, terminal-game admission, twelve named mutations and passive/script/native
quickstarts. Admission reran all eight retained families, checked invalid tapes,
and repeated each of sixteen distinct games twice per engine (32 runs each).
All twenty nontoken card IDs have consumed/resolved witnesses. Repetition does
not increase scenario or game counts.

[Raw execution archive](execution.tar.gz) and its [SHA-256 member index](execution-index.json)
retain all observations, logs, receipts, full torture output and the failed cold
build. The archive contains privileged full-state test/replay evidence; extract
only into private diagnostic directories. The [validation receipt](validation.json)
binds the completed torture counts and both-profile selected-test results.
The [recomputed evidence check](evidence-check.json) verifies inherited and fresh
archives, source bytes and accounting. Its executable [checker](check_evidence.py)
fails if the fresh gate archive/receipt is missing; it does not replace engine
execution or independent semantic review. [Audit source hashes](sources.json)
pin unchanged rules, tests, fixtures, reference bridges and governing inputs.

The root README, capability overview and roadmap reflect this scoped PASS,
conditional on the linked review and protected delivery receipts. No gameplay
command or setup change was needed. Existing passive, scripted and native
quickstarts passed with closed stdin and no display in the managed container,
without invoking Docker. Subsequent source changes require their own validation;
this report does not silently transfer acceptance to a different implementation.
