# M1 scalar vertical-slice gate — GH-22

Audit target: exact main `a8f262238e182e1302bc7ff37d20a66369f3315f`.
Execution is inside the managed Linux ARM64 toolchain container. This is an
independent rerun of the delivered composition, not an inference from child
closure. **PASS for the M1 scalar acceptance criteria.** Fresh full torture passed 158
Python tests and 956 Rust executions across debug/release, zero failed/ignored;
all applicable reference reruns and headless quickstarts passed. This local
verdict is conditional on the delivery gates below.
Delivery requires the separate candidate review, protected merge and successful
CI on that exact main commit recorded in the
[workpad](https://github.com/pabloxrl/mtg-lab/issues/22#issuecomment-5943196087).

The scope is Forest, Mountain, Bear Cub, Swab Goblin, Giant Growth and Bite Down,
with vanilla combat and the opening/turn/terminal machinery they require.
The frozen decks contain other cards whose effects remain unsupported and
uncastable. This is not full-pool Magic, the full MVP, or performance/release
qualification. M2–M5 obligations remain unchanged.

## Exact-main prerequisites

Fresh `origin/main` manifest and verbatim requirement inventory validate against
their source commit/checksum. All 63 M1 implementation/operations tasks are
closed with `state_reason: completed`; every task has a completion workpad linking
acceptance, independent review, merged PR and exact-main CI. The
[dependency receipt](dependencies.json) independently queries each merge's
successful main-branch push CI. The five direct integration owners are #17–#21;
the graph validator checks that their transitive closure includes every M1
implementation and operations prerequisite. There are 97 registered program
tasks, 44 unchanged RFC blocks and 320 unchanged catalog designs.

Historical unsuccessful attempts remain in their workpads. Later reviewed
prerequisite deliveries and final completion reports resolve those attempts;
the earlier failures are not passing evidence. The current gate changes no
program ownership, requirement, expectation, workflow or CI policy.

## M1 exit criteria and independent expectations

| RFC §10 criterion | Executable acceptance and independent basis |
| --- | --- |
| Red/green evidence | Original component receipts retain compiled assertion failures, green runs and detected mutants. See the ledger below. This report-only audit invents no new behavior red. Compilation errors and unavailable references are not passes. |
| Scripted games complete without stdin | Full normal-discovery CLI suites run real native/script policies through the owned Driver, with stdin closed, displays absent, deadlines and kill/reap. `script_played_combat_reaches_literal_rules_terminal_without_concession` reaches an actual life-loss outcome. Native games cover both starters and capture on/off. |
| Trajectory round-trip and rewards/boundaries | `collector_audit`, capture/publication, `composed_cli` and trajectory-v2 tests execute actual Driver → recorder → JSONL/manifest → reload → authorized replay. Same-seat links, opponent interleavings, never-acting/never-acting-again seats, zero-decision rewards, terminal/truncated/failed/incomplete distinctions and once-only rewards are asserted. |
| Critical stack/cleanup/privacy locally and applicable references | Native Growth/Bite response and departed-target scripts, quantum/private-view/snapshot regressions and strict live XMage comparisons. Cleanup must leave the same Cub as an undamaged 2/2; expiration and damage removal cannot expose an intermediate lethal check. Opening/combat/terminal reference scopes are explicitly narrower than native game tests. |
| Invalid actions preserve state | Real opening, land/payment, cast/target, combat, cleanup, policy/semantic submission and script-routing tests compare complete state/RNG/history/capture before and after wrong-actor/stale/illegal/missing/duplicate actions. Subsequent valid submissions recover through the same owner. |

The literal normal-reset seed-160/178 spell/combat ledgers use pinned CR and
Oracle expectations: a Growth-protected Cub is 5/5 with two marked damage after
Bite, then 2/2 with no damage after cleanup. Ten unblocked two-power attacks
remove twenty life. The CLI's shorter 101-choice script plus explicit concession
has independently specified `[20,15]` life, `[5,7]` hand counts, `[31,31]` libraries,
102 consumed semantic records and `[1,-1]` returns. Concession is not confused
with an independently played combat win.

The seed-154 passive full-game ledger independently constructs every authorized
observation/domain/mask/action/time/reward for both starters: 68 turns, 1,140
decisions and the nonstarter losing on an attempted empty draw. The library
being empty alone is not a loss. These expectations are separate from the
implementation output. Full-field equality between capture/replay/processes
is additional metamorphic evidence, not a second rules oracle. Native policy
winners and timing samples are never adopted as expected rules results.

## Every M1-owned RFC block

Each row covers applicable scalar clauses and retains later shared-block work.
The linked integration reports contain the original clause-by-clause and stable
SYS/DRL mappings; the gate reruns their normal-discovery tests on the target above.

| Block | M1 acceptance | Retained later obligations |
| --- | --- | --- |
| B013 Architecture | [Core audit](../core-integration/README.md): one Rust rules owner; CLI/policy/recorder are clients, references stay outside production. Actual dependencies retain these boundaries. | Batch, Python, trainers, interactive clients. |
| B014 Rust rationale | Same audit treats Rust as a design choice, not a measured language-speed claim. [Initial native baseline](../unattended-integration/README.md) measures the actual client. | M2 profiles/full-pool baseline and M3 boundary measurements. |
| B015 State/execution | Independent RNG vectors, generation-safe objects, reused reset, separate policy randomness, real effect/turn/combat quantum continuations and exact state/RNG equality. | Worker scheduling/batch fairness, scale profiling. |
| B016 Core interface | Actual decision validation, typed continuations, internal yields without invented player decisions, snapshots/replay with version/RNG checks; [private/replay audit](../private-replay-integration/README.md). | Batch fast-path equivalence and later mechanics. |
| B017 Complete legal choices | [Scalar audit](../scalar-integration/README.md): independent target pairs, generic payment colors, attacker subsets, blocker maps and damage splits. Capacity failures preserve state; private intermediate selections give no extra priority. | Token/keyword/full-pool bounds and fixed tensor schema. |
| B020 Benchmarks | Runnable passive and native-rollout commands, total/policy timing, complete outcome denominators and retained repeated initial baseline before pool expansion. | Six full benchmark tracks, warmup/distributions, designated hardware, memory/profiles/scaling and performance qualification. |
| B025 Fixture/replay provenance | Pinned original CR/Oracle scripts, explicit synthetic versus normal-reset constructors, strict semantic actions, rejection and intermediate checkpoints, full played snapshots/replay. | Expanded corpus/reference/full-game coverage and later batch paths. |
| B026 Scenario families | All 133 M1 catalog designs mapped below and exercised through existing tests. Opening, priority, mana, targets, vanilla combat, Growth/cleanup, identity, outcomes, choices, privacy and scalar replay. | M2 triggers/keywords/tokens/remaining cards; M3 batching/worker/tensors. |
| B029 Test-first verification | Preserved compiled red/green, independent expected checkpoints, mutant controls, fresh full regression and applicable live reference execution. No expectation regeneration or test weakening. | Every future capability keeps the same contract; M5 larger torture/mutation matrix. |
| B033 Observation/action schema | Structured scalar authorized views/domains and complete ordered submissions, hidden-hand/library twins, private pending choices, candidate order/masks/errors, explicit capacity failure. | Fixed numeric tensors, Python/batch buffers and privileged-critic/trainer integration. |
| B036 Rewards/time | [Trajectory audit](../trajectory-integration/README.md): stable-seat sparse +1/-1/0 rewards exactly once, explicit reset/final observations, logical versus micro decisions, gamma=1, actual limits/failure quarantine and policy provenance. | Wrapper autoreset, batch identities, trainer integration. |
| B037 Trajectory product | Canonical v1/v2 in-memory and bounded JSONL, complete versioned manifests/rows/footers, ownership, same-seat readers, privacy, explicit backpressure/errors and manifest-last publication; actual capture/replay equality. | Parquet/shards/reassembly, recurrent/numeric/Python readers, deterministic subset execution, worker/adapter equivalence and comparative capture overhead. |
| B039 Unattended experiments | [CLI audit](../unattended-integration/README.md): actual native/script policies, versioned JSON/errors, seeds/versions/config/budgets, interruption/accounting, capture/validate/replay and initial benchmark. | Persistent JSONL decision protocol M4; full-pool and batch collection remain their owners. |
| B041 Zero-human contract | Closed stdin, no display, strict scripts, bounded test/reference subprocesses, explicit missing choices/dependencies and preserved first-divergence artifacts. | Same contract for future RL/fuzz/full-game/reference qualification. |
| B042 Milestone/release gates | The five M1 exit criteria above and runnable documented scalar system; full torture includes verify, debug/release, CLI, scalar replay and record equality. | M2–M5 gates; batch/adapter tests when implemented; exact-release dual-reference and performance qualification. |
| B044 First work package | Workspace/pins/verifier and both bridge spikes from M0; test-first scalar land/creature/Growth/Bite, views/snapshot/replay, actual unattended games, initial recorder and benchmark/counters. | Broader pool, benchmark qualification and all later feature obligations. |

Block IDs above abbreviate `R0002-`. None is declared globally complete merely
because its M1 portion passes. No batch or trainer capability is advertised, so
its future acceptance is not represented as a skipped M1 test.

## Catalog and behavioral-red ledger

The unchanged catalog has 133 M1 cases: #64 (1), #65 (12), #67 (8), #68 (5),
#69 (12), #70 (49), #71 (9), #72 (14), #73 (3), #76 (3), #18 (8), #19 (8),
#20 (1). Every exact ID resolves to existing test source or a reviewed report
mapping it to named executable tests. This inventory is a traceability check,
not a test count or generated expected-output store. #22 owns no new catalog
design; it retains aggregate acceptance of all 133.

The [machine-readable catalog crosswalk](catalog.json) preserves each original
expected result and independent basis. The [source inventory](sources.json)
hashes the actual audited code, fixtures, reference bridges/pins and program
inputs. These files do not alter the design catalog's status or its execution
fields.

Original compiled red/green and independent-basis reports remain unchanged:
[RNG](../rng-v1/README.md), [objects](../objects/README.md),
[opening](../opening/README.md), [mulligan](../mulligan/README.md),
[quantum](../quantum/README.md), [turns](../turns/README.md),
[mana](../mana/README.md), [casting](../casting/README.md),
[targets](../targets/README.md), [combat](../combat/README.md),
[terminal](../terminal/README.md), [views](../views/README.md),
[snapshot/replay repair](../private-replay-integration/README.md),
[trajectories](../trajectory/README.md), [JSONL](../trajectory-jsonl/README.md),
[played recorder](../episode-capture/README.md),
[publication](../collector-publication/README.md),
[native CLI](../native-cli/README.md), [script CLI](../script-cli/README.md),
[capture CLI](../captured-cli/README.md), and
[active benchmark](../unattended-integration/README.md).
The component reports distinguish intended assertion failures from compile/API
mistakes. This audit makes no rule or runtime change and adds no replacement test.

## Reference boundaries and remaining limitations

Original identical neutral scripts and independent expected checkpoints drive
native/XMage Growth/Bite execution. The bridge does not ask the native engine
for legality or results. Negative controls change choices/targets/checkpoints
and must fail for their intended reason; timeouts/build failures cannot count
as detecting a mutant. Opening uses synthetic all-basic hands/libraries and
compares counts/actors, not shuffle/card order. Combat and terminal receipts
cover their exported fields. Privacy is tested at the native policy boundary;
an unobservable reference field is not claimed as matched.

The extra-cleanup priority premise is explicitly synthetic: the supported positive
boosts cannot themselves generate the later trigger/SBA source. Native full-game
reachability is distinct from synthetic reference cases. Full reference games,
every-card XMage and the mandatory ≥20 independent dual-reference suite remain
later qualification requirements. No broader Forge coverage, universal rules
correctness, general playing strength or full-MVP completion is implied.

Dataset/replay authorization assumes the trusted local owner, not a malicious
host. Two-seat datasets remain private relative to live seats; default policy
readers do not combine the opponent's observation or resolve restricted replay
links. Count/per-artifact limits are not total RSS/disk quotas. Cooperative
deadlines cannot preempt blocked filesystem calls. Incomplete internal-work
stops can lack final views and are explicitly excluded from completed data.
Snapshot/replay source fingerprints are conservative and reject incompatible
versions without migration. No Parquet, training, performance target or
power-loss guarantee is claimed.

## Reproduction

From this checkout in the managed toolchain container, run:

```sh
./scripts/torture.sh
python3 scripts/opening_reference.py --cache /home/agent/.cache/xmage --output /tmp/m1-opening.json
python3 scripts/instant_reference.py --cache /home/agent/.cache/xmage --output /tmp/m1-instant
python3 scripts/combat_reference.py --cache /home/agent/.cache/xmage --output /tmp/m1-combat.json
python3 scripts/terminal_reference.py --cache /home/agent/.cache/xmage --output /tmp/m1-terminal.json
python3 scripts/forge.py acceptance --cache /tmp/mtg-forge
```

Reference commands sharing a cache run sequentially. Existing pinned caches are
required; use each reference's documented prepare/build commands if absent.
No worker invokes Docker or installs host software. A cache/build failure stops
its acceptance; it is never a skipped or successful comparison.

[Opening](opening.json), [combat](combat.json), [terminal](terminal.json) and
[instant acceptance](instant/acceptance.json) pin actual executions. All 102
instant receipt-listed artifacts and 38 native source hashes were independently
rechecked after execution. The receipt states exactly which fields the bridge
observes and which it cannot.

Fresh [XMage smoke](xmage-smoke/acceptance.json) and
[Forge smoke](forge/acceptance.json) each ran the same neutral priority fixture
twice, with three separately detected life/choice/checkpoint mutants per engine.
This is one narrow dual-reference case, not the later twenty-case critical suite.
The initial Forge probe reported missing external source; pinned prepare/build
in `/tmp/mtg-forge` then succeeded with the exact dependency inventory before
fresh offline acceptance. The failed probe is retained and not counted as a pass.

[Quickstart commands](quickstarts.json) were run directly with stdin closed,
DISPLAY/WAYLAND_DISPLAY unset and 90-second external timeouts. They cover passive,
scripted and native simulation, the active benchmark and manifest validation.
The retained stdout files are actual observations, not new test oracles. Native
benchmark quickstart uses the debug executable to check command behavior, not
the README's release-build timing configuration. The previously reviewed release
baseline remains the initial measurement; this audit makes no new speed claim.

README impact: the evidenced stage verdict and gate link must accompany this
report; supported commands/setup/card boundaries remain unchanged. The separate
review must check both the gate scope and README accuracy. Final review and
delivery receipts are linked from the workpad rather than guessed before merge.
