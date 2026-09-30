# Played CLI prerequisites

Operations [#175](https://github.com/pabloxrl/mtg-lab/issues/175) registers only the
six existing issues #176–#181. This plan refines the four proposed contracts in
[#120's preserved unsuccessful workpad](https://github.com/pabloxrl/mtg-lab/issues/120#issuecomment-5907720166):
A becomes #176; B becomes independent native execution #177 then script routing
#178; C becomes #179; D becomes independent replay #180 and dataset #181 commands.
The original proposal and [integration plan](m1-integration-prerequisites.md)
remain history, not discarded acceptance. #120 becomes the complete composed
CLI audit, followed by #21 and gate #22. Registration implements no capability.

## Delivered API evidence

Inspected main `be99697359f3806e44b4435297da90495cc50b88` (2026-09-30).
These are actual interfaces, not proposed sibling implementations:

| Existing contract | Source at inspected main | Consequence for the split |
| --- | --- | --- |
| Semantic `Record`, `decode(&Game, bytes, capacity)` and `Decoded` submission/concession | [actions.rs](https://github.com/pabloxrl/mtg-lab/blob/be99697359f3806e44b4435297da90495cc50b88/crates/mtg-core/src/actions.rs) | Decoder needs the authoritative private Game. #176 must decode inside its owner, not restore a parallel Game or expose mutable access. |
| `Driver::{bounded,reset,reset_captured,advance,observe,submit,submit_with_policy,concede,finish}`, `Budget`, `EpisodeResult` | [episode.rs](https://github.com/pabloxrl/mtg-lab/blob/be99697359f3806e44b4435297da90495cc50b88/crates/mtg-core/src/episode.rs) | Driver privately owns Game/history/capture; observations translate core revision to owner revision and submit translates back. Semantic input is missing; native submissions already work. Existing budget/clock/record-limit and terminal finalization contracts must be reused. |
| `LegalRandom::choose`, `Heuristic::choose` return structured `Submission` | [random](https://github.com/pabloxrl/mtg-lab/blob/be99697359f3806e44b4435297da90495cc50b88/crates/mtg-policy/src/lib.rs), [heuristic](https://github.com/pabloxrl/mtg-lab/blob/be99697359f3806e44b4435297da90495cc50b88/crates/mtg-policy/src/heuristic.rs) | #177 needs no semantic-input sibling; policies receive authorized observations only, preserve separate policy RNG and existing six-card M1 support limits. |
| `simulate::Config`, `run`, `step`; `commands::execute` | [simulate.rs](https://github.com/pabloxrl/mtg-lab/blob/be99697359f3806e44b4435297da90495cc50b88/crates/mtg-cli/src/simulate.rs), [commands.rs](https://github.com/pabloxrl/mtg-lab/blob/be99697359f3806e44b4435297da90495cc50b88/crates/mtg-cli/src/commands.rs) | Current CLI accepts pass-v1 only, owns a bare Game, verifies opening replay and reads v1 trajectories. Preserve legacy behavior; reuse its command/output boundary while routing new modes through Driver. |
| `collector::Run::{header,persist}`, `Storage`, `publication::{publish,read_replay}`, `episode_replay::Registry::{register,resolve}` | [collector.rs](https://github.com/pabloxrl/mtg-lab/blob/be99697359f3806e44b4435297da90495cc50b88/crates/mtg-recorder/src/collector.rs), [publication.rs](https://github.com/pabloxrl/mtg-lab/blob/be99697359f3806e44b4435297da90495cc50b88/crates/mtg-recorder/src/publication.rs), [registry](https://github.com/pabloxrl/mtg-lab/blob/be99697359f3806e44b4435297da90495cc50b88/crates/mtg-core/src/episode_replay.rs) | Real manifest-last publication and per-artifact authorization already exist. Disk replay helper requires retained EpisodeResult: #180 is deliberate local-file verification, not persisted registry authority. No new recorder or grant format. |
| `opening::replay::played::{record,verify}` | [played_replay.rs](https://github.com/pabloxrl/mtg-lab/blob/be99697359f3806e44b4435297da90495cc50b88/crates/mtg-core/src/played_replay.rs) | #180 can use library-produced terminal played artifacts independently of simulate/capture; opening remains a distinct supported format. |
| `from_core_v2`, `Writer::new_v2/append_v2`, `read_v2`, `Manifest::parse/load_v2` | [recorder](https://github.com/pabloxrl/mtg-lab/blob/be99697359f3806e44b4435297da90495cc50b88/crates/mtg-recorder/src/lib.rs), [manifest](https://github.com/pabloxrl/mtg-lab/blob/be99697359f3806e44b4435297da90495cc50b88/crates/mtg-recorder/src/manifest.rs) | #181 can generate actual played v2 records using delivered core/recorder libraries. It need not wait for future CLI publication or invent a fake capture implementation. |

Existing executable examples include `crates/mtg-core/src/actions.rs` tests,
`episode.rs`, `budgets.rs`, `capture.rs`, `played_replay.rs`, and
`crates/mtg-recorder/tests/collector_audit.rs` and `structured_contract.rs`.
The scalar collector audits already exercise normal-reset played publication.
These are reusable predecessors/evidence, not permission to replace independent
expected checkpoints with implementation output.

## Minimal dependency graph and ownership

| Issue | Direct prerequisites | Narrow deliverable | Explicit exclusions |
| --- | --- | --- | --- |
| #175 | #158 | Register reviewed contracts; no RFC ownership | Production implementation, counter resets, CI/workflow/settings changes |
| #176 | #175, #161, #113 | Owned privileged semantic-record submission, revision translation, validated concession | CLI, new action format, mutable Game escape, second engine/collector |
| #177 | #175, #161, #118, #119, #79 | Bounded native-policy simulation through Driver, capture disabled | Semantic scripts, persistence, new policy algorithms, performance qualification |
| #178 | #176, #177 | Strict bounded semantic scripts in the delivered run loop | Reimplementing run lifecycle, capture, verification commands |
| #179 | #178, #164, #117 | Canonical capture/publication with explicit roots, limits, replay authority and publication outcomes | New recorder/schema, sampling/shards, persisted grants, verification CLI |
| #180 | #175, #114, #79 | Explicit bounded opening/played replay verification routing | Simulate/capture, new replay format, opaque-ID authority resolution, broad inspection UI |
| #181 | #175, #153, #79 | Explicit bounded v1/v2 JSONL and run-manifest validation | Simulate/capture, new schema/publisher, replay access or following replay references |
| #120 | #179, #180, #181 | Audit EVERY original composed CLI clause | Reimplementing full CLI in an audit |
| #21 | #120 (unchanged) | Full original command/integration/catalog audit and initial active-policy measurements | Waiving original acceptance based on passing children |
| #22 | Unchanged | Complete M1 gate | Registration or child completion treated as M1 completion |

#120's old #117 prerequisite is retained through #179; #118/#119 through
#179 → #178 → #177; #79 through #177/#180/#181. No independent old edge is lost.
Every new child has #175 transitively. #161 includes delivered owner/capture
predecessors; #153/#114 provide real generation/read contracts for independent
verification tests. No additional dependency correction is needed. #180/#181
can execute independently of the native/script/publication chain. All six are
partial B020/B039/B041 owners only; all old owners, verbatim RFC blocks/source
pins, catalog bytes/case owners/expectations and later gates remain unchanged.
Native parents place operations #175 under program #7 and #176–#181 under
aggregate #120, matching the existing #158/#154 convention; blocked-by sets
match the table. The #120 native replacement and
recovery occur only after this registration's protected merge and exact-main CI.

## Independently testable child contracts

All new behaviors require compiled behavioral red before implementation,
independently justified expectations, positive and negative tests in normal
discovery, retained minimized seeds/scripts, full managed-container torture,
clean-candidate prescribed separate review, protected merge and exact-main CI.
Compile/import failure is not red. Synthetic corruption/fault injection is
identified explicitly and supplements real normal-reset played acceptance.

### #176 — semantic input at the owner boundary

Use hand-authored existing semantic records for normal-reset opening, land,
creature, target/payment, combat and concession. Assert rule-derived life/zones,
ordered accepted semantic history and canonical capture against a literal ledger;
compare bounded/unrestricted and capture on/off state/RNG. Reject malformed,
missing, wrong-seat/incarnation, stale/illegal, internal-yield and post-end input
without unauthorized state/RNG/history/trajectory changes. Check record exhaustion
and limit-finalization according to the delivered owner contract; budget-induced
finalization is explicit, not an accepted script choice. Rejection must not poison
later valid input. Concession uses the same authorization and exactly-once path.
This needs only existing decoder and Driver; no CLI test double is needed.

### #177 — native run lifecycle

Run real LegalRandom/Heuristic from both starting seats headlessly with stdin
closed and display variables unset. Repeat seeds, compare semantic results with
direct native-library execution AND independently justified checkpoints; equality
alone is not a rules oracle. Independently count tiny episode/decision/work/record
limits. Test deterministic deadline/stop hooks, pre-start and mid-game signals,
malformed versions/policies/bounds, stdout errors and episode-count overflow.
A bounded real OS signal smoke supplements deterministic hooks, without flaky
sleep-based expected outcomes. Preserve pass-v1, bench and config compatibility.
Report requested/started/not-started plus completed/truncated/failed/incomplete;
caller/output errors never become rules draws or wins. No missing policy fallback.
The existing Driver and both policy libraries make this testable without #176.

### #178 — script routing

Use #176 in #177's real loop. A hand-authored normal-reset script across land,
creature, targets, payment and combat reaches literal life/zone/decision
checkpoints and terminal outcome. Both seats, multiple episodes and concession
prove exact consumption and script privacy. Omitted/extra/reordered/malformed,
wrong-seat/incarnation, stale/illegal records and byte/count overflow fail with
structured status, truthful accounting and no implicit/native fallback. Library
checks prove pre-action state/RNG/history/capture nonmutation; subprocess checks
prove the public contract. Declared budget stops mid-script are truncation, not
successful consumption. No publication or verifier sibling is needed.

### #179 — publication

Actual native and script subprocess games publish canonical manifest/JSONL and
separately authorized replay using existing Run/Registry/publisher. Reload through
delivered libraries and compare exact observations, domains/masks, semantic
choices, rewards/boundaries, final game/RNG and independent script ledger.
Capture on/off with identical seeds/actions preserves gameplay. Validate explicit
capture/storage budgets and distinct dataset/private roots before mutation.
Test overlapping/conflicting roots, overflow, queue/writer/flush/sync/publication
faults (including rename/link boundaries actually used), missing/corrupt artifacts,
unauthorized replay access, interrupts/deadlines during gameplay and publication.
Gameplay outcome and failed/uncertain publication are separate; unfinished and
never-started requested episodes cannot disappear. Replay IDs grant nothing.
No future verification CLI is needed: delivered library readers are sufficient.

### #180 — replay verification

Feed actual library-produced normal-reset terminal played replays and retained
opening artifacts through closed-stdin/no-display commands. Assert explicit
format/version routing, literal summary/checkpoints and documented status.
Mutate independently justified checkpoint fields, semantic records and versions;
reject divergence, unsupported/ambiguous format, nonterminal/truncated history,
malformed/oversized bytes. No fallback parser acceptance. Ordinary errors redact
private seeds/hands/paths/semantic payloads; verification does not silently enable
privileged inspection. User selection of a local file is explicit access, not an
opaque-ID grant. Existing #114 and #79 suffice, without #177–#179.

### #181 — dataset validation

Produce real played v2 data using delivered core capture and recorder libraries;
validate JSONL/manifests headlessly with independently counted episodes/decisions
and machine summaries. Retain v1 fixtures/commands. Completed-only is default;
explicit diagnostics admit valid truncated/failed metadata and identify its
noncompletion. Reject absent/mixed/unsupported versions, missing/duplicate episodes,
checksum/count/provenance mismatch, unsealed/corrupt fragments and over-budget
input. Never salvage a misleading completed prefix. Private diagnostics remain
redacted; selecting dataset files never follows replay references or authorizes
replay access. Synthetic corruption fixtures remain distinct from actual played
acceptance. No future capture CLI is required; #153 and existing core APIs suffice.

## Complete original acceptance crosswalk

Sources: [original #120 body](https://github.com/pabloxrl/mtg-lab/issues/120),
[its preserved workpad](https://github.com/pabloxrl/mtg-lab/issues/120#issuecomment-5907720166),
[original #21 body and amendments](https://github.com/pabloxrl/mtg-lab/issues/21),
and the unchanged verbatim B020/B039/B041 ledger. Rows split compound clauses
so none silently disappears. “Audit” requires fresh executable composed evidence,
not checkboxes inferred from child completion.

| Original clause | Component owner/evidence obligation | Aggregate owner |
| --- | --- | --- |
| #120 native policies in existing simulate | #177 real both-seat deterministic random/heuristic Driver games | #120 → #21 |
| #120 explicit semantic scripts in simulate | #176 owner decode/submit; #178 exact script consumption, literal checkpoints | #120 → #21 |
| #120 existing verify commands | #180 played/opening routing; #181 v2/v1/manifest routing | #120 → #21 |
| #120 canonical capture | #179 real Run/publisher, exact canonical field equality | #120 → #21 |
| #120 opaque replay links | #179 explicit owner authorization/disjoint roots; #181 never resolves links | #120 → #21 |
| #120 explicit episode budget | #177 requested/started/not-started accounting, overflow/bounds negatives | #120 → #21 |
| #120 explicit work budget | #177 bounded owner advancement/limits; #178 script budget-stop distinction | #120 → #21 |
| #120 explicit capture budget | #179 record/storage limits, no silent drop; #176 retains owner limits | #120 → #21 |
| #120 headless structured errors | #177–#181 closed stdin/no display, versioned machine output/status and redacted errors | #120 → #21 |
| #120 no second engine, recorder or performance qualification | #176–#181 reuse delivered contracts; code/review checks | #120 → #21; later qualification unchanged |
| #120 played games produce manifest | #179 real publication plus #181 command reload | #120 composed producer → validator |
| #120 played games produce JSONL | #179 canonical v2 plus #181 command validation | #120 composed producer → validator |
| #120 played games produce replay | #179 authorized publication plus #180 command verification | #120 composed producer → verifier |
| #120 exact reload equality | #179 literal ledger and full field/state/RNG equality; #180/#181 negative mutations | #120 cross-command fresh-process equality; #21 |
| #120 missing scripts preserve state | #176 reject before mutation; #178 missing-choice run failure | #120 → #21 |
| #120 illegal scripts preserve state | #176 full state/RNG/history/capture invariance; #178 public errors/no fallback | #120 → #21 |
| #120 writer failure boundaries | #179 queue/write/flush/sync/publication failures, uncertain publication explicit | #120 composed failure → diagnostic validation; #21 |
| #120 SIGTERM boundaries | #177 real signal and deterministic stop tests; #179 publication interruption | #120 requested-episode accounting across artifacts; #21 |
| #120 deadline boundaries | #177 injected clock; #179 publication deadline diagnostics | #120 → #21 |
| #120 truncation boundaries | #177/#178 owner finalization; #179 publish diagnostics; #181 completed-only/opt-in | #120 → #21 |
| #120 ALL requested episodes accounted | #177 no-start/partial run/count overflow; #179 failures including never-started | #120 → #21 |
| #120 capture on/off same seed | #176 owner equality; #179 full native/script gameplay/RNG equality | #120 → #21 |
| #120 existing commands supported | #177 pass/bench/config; #180 opening verify/seat inspection; #181 v1; existing conformance tests retained | #120 full command matrix → #21 |
| #21 noninteractive simulation | #177 native and #178 script subprocess tests | #120 then #21 final audit |
| #21 fixture/replay checks | #79 retained fixture comparator with true mismatch/error cases; #180 played routing | #21 full command matrix after #120 |
| #21 structured results | #177–#181 versioned output, stdout/stderr and documented exit codes | #120 → #21 |
| #21 deterministic random opponents | Delivered #118 plus #177 seeded CLI integration | #120 → #21 |
| #21 deterministic heuristic opponents | Delivered #119 plus #177 seeded CLI integration | #120 → #21 |
| #21 episode/work/truncation budgets | #177/#178 limits and owner stop semantics; #179 capture limits | #120 → #21 |
| #21 interruption summaries | #177 signals/deadlines, #179 storage uncertainty | #120 → #21 |
| #21 stdin closed / display unset executable subprocess tests | #177–#181 every automation mode, retained #79 tests | #120 → #21 command matrix |
| #21 malformed input explicit failure | #176–#181 independent mutations and status/error assertions | #120 → #21 |
| #21 missing decisions explicit failure | #176/#178 no implicit choices or policy fallback | #120 → #21 |
| #21 no required UI, IPC or trainer | #177–#181 headless library clients, dependency review | #120 → #21 |
| #21 benchmark command/counters before broader pool | Retain #79 scalar-pass smoke command/counters; #177 preserves compatibility | #21 checks command and extends only bounded integration measurement |
| #21 outcome/failure accounting in benchmark | #177 honest active run counts; retained #79 tests | #21 initial active-policy rollout measurement includes failures/reset/policy cost |
| #21 initial measured baseline even if slow | #177 supplies runnable active modes, no performance claim | #21 records actual scoped native random AND heuristic baseline, versions/workload/hardware/raw distributions; passive smoke cannot substitute |
| #21 invalid actions unchanged | #176 library invariance and #178 rejected public script checks | #120 → #21 |
| #21 trajectory boundaries | #179 canonical capture, #181 strict loading with unfinished status | #120 → #21 literal rewards/final views and composed interruption cases |
| #120/#21 test-first, independent expectations, normal discovery, minimized regressions | Each implementation child, no weakened assertions or generated oracles | #120/#21 review every retained expectation |
| #120/#21 full torture, relevant cached references for rules changes, separate review, protected merge/exact-main CI | Every delivery; no new reference agreement claimed by registration | #120/#21 same gates, then #22 |
| #120/#21 README precise scope, dashboard/workpad, normal bounded handoff | Each delivery documents actual commands/limits/evidence; five-minute notes | #120/#21; no stage completion before #22 |
| #21 reviewed catalog/system/game-replay/data-RL/contract-default plans and all original owners | Catalog unchanged; #21 enumerates every directly owned case and applicable planned family, including SYS-CORE-009 public configuration | #21 full catalog/integration audit; #22 stage verdict |
| #21 authoritative B020/B039/B041 clauses and later obligations | Partial children only; no full-block satisfaction inferred | #21 M1 aspects; existing M2–M5 owners retain broader workloads, batching, protocols, trainers, reference/performance qualification |
| #120/#21 atomic amendments, bounded gaps only, explicit resumption and controls | #120 audit after #179/#180/#181; no full reimplementation in any prerequisite | #21 unchanged after #120, #22 unchanged; oversized gaps require coordinator decision |

B020's broader transition/encoding/inference/capacity/CLI latency benchmark tracks,
worker/batch sweeps and release profiling stay with existing later owners. #21
still records the honest initial active-policy baseline; this split does not
waive its measurement clause. B039's validated config, resolved versions/seeds,
resource/instrumentation metadata, JSON/JSONL machine mode, stderr diagnostics,
no prompts/ANSI when non-TTY, explicit dependency/input/policy errors and bounded
signal handling remain #120/#21 acceptance across the child command matrix.
Its persistent decision protocol and interactive controls remain M4; training and
high-throughput Python remain M3, without per-action subprocesses. B041's bounded
headless CI/reference/fuzz/replay/benchmark/RL and evidence-backed adjudication
remain mandatory with their existing stage owners. No child claims full-pool
reference or release qualification. The 320-case catalog is unchanged byte for
byte; its expectations are requirements, not claimed execution results.

## Registration validation, recovery and README

Validate manifest schema/unique issues/known kinds and requirements, cycles,
prior-stage and complete milestone gates, reverse ownership, pinned RFC source,
all original task metadata except #120's direct edges, original ancestor retention,
all old owners, and byte-identical catalog. Negative controls must reject cycles,
gate bypass, lost old owners and changed verbatim text. Inspect all native
blocked-by sets and new parent links, retaining #120's old links until merge.
Run full `./scripts/torture.sh` after fresh-main integration and obtain the
prescribed clean-candidate separate Codex review; preserve findings/resolutions.
No production or existing executable test changes are needed for registration.

README requires no change: commands, setup, supported behavior, delivered
architecture, limitations and milestone status are unchanged. Current README
accurately states passive-only simulate, v1-only validation, opening-only CLI
replay and pending CLI/M1 integration. Reviewer must independently check this;
existing quickstarts remain checked by ordinary command regression tests.

Only after protected merge and successful exact-main CI: preserve #120's original
body and unsuccessful workpad verbatim, append this prerequisite amendment and
explicit operator-authorized deferred resumption, and atomically remove only its
resolved sizing block while adding `agent-resume-authorized`. Leave it unready;
#21's existing grant and all later gates stay intact. Replace only #120's native
dependencies with #179/#180/#181. Controller shutdown/one-time ignored-counter
archival is coordinator-owned; this worker does not access other workspaces or
reset counters.

Update the single Program workpad with final operation/review/acceptance/merge/
exact-main evidence. Fresh main is the allowlist: refresh parent controls, all
task states, paginated label histories and dependency completion receipts/CI.
Select exactly one eligible child in manifest order (#176 if its controls and
#161/#113 evidence still pass), using only the verified operation's pre-closure
completion exception. Record selection, recheck controls immediately, confirm
ready, then atomically close #175. A retry reuses these issues/PR/workpads and an
already-ready successor; it never creates duplicates or queues a second task.
Ordinary handoff continues independent branches even if another blocks, without
waiving failed prerequisites or marking #120/M1 complete at registration.
