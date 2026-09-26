# RFC 0001: A small, correct, high-throughput Magic research engine

Status: **Proposed** — implementation scope and acceptance criteria, not implemented functionality or measured performance.

Date: 2026-09-26. Related: [open-source engine survey](../open-source-mtg-engines.md).

Revision: automation-first simulation and agent training; CLI interaction with optional human play; no human required for development or tests; test-driven implementation; mandatory mature-engine differential verification; evidence-based RL compatibility requirements. Framework documentation reviewed 2026-09-26. These are design requirements, not integrations already tested in this repository.

## Problem definition

Researchers need a **correct, high-throughput Magic simulation environment that produces trustworthy experience for training and evaluating agents**. The engine must run many independent games unattended, expose each player's legal choices and permitted information, and capture reproducible trajectories with useful instrumentation. The challenge is to maximize useful simulations per CPU-second without sacrificing rules correctness, hidden-information boundaries, or data integrity.

The initial boundary is two fixed 40-card decks using 20 distinct Foundations cards—not all of Magic. CLI access and modern RL integration are required; human play is optional, and no development or acceptance workflow may depend on a human playing. Rust, adapters, and storage formats are proposed solutions to this problem, not the problem itself.

## Contents

- [Problem definition](#problem-definition)
- [Personas](#personas)
- [Use cases](#use-cases)
- [Decision summary](#1-decision-summary)
- [Objectives and metrics](#2-what-smallest-and-fastest-mean)
- [Frozen decks and rules scope](#3-concrete-game-scope)
- [Architecture](#4-architecture-and-ownership-boundaries)
- [Performance budget](#5-throughput-plan-and-performance-budget)
- [Instrumentation](#6-instrumentation-without-making-the-hot-path-expensive)
- [Test sources and torture suite](#7-the-torture-suite-strategy)
- [RL integration](#8-rl-integration-contract)
- [Trajectory capture](#trajectory-capture-is-a-first-class-output)
- [Automation-first CLI](#9-automation-first-cli-and-optional-human-play)
- [Milestones](#10-milestones-and-release-gates)
- [Alternatives and risks](#11-alternatives-and-risks)
- [First implementation work package](#12-first-implementation-work-package)

## Personas

These are roles, not necessarily separate people. The primary users generate, learn from, and evaluate simulated experience. Contributors and automated development agents maintain the environment that makes those results credible. Interactive players are a secondary audience.

### P1. RL researcher — primary

**Goal:** investigate learning, exploration, memory, and self-play in a partially observed, turn-based game.

Works mainly in Python with an existing trainer. Needs batched observations, legal-action masks, stable seat identities, reliable rewards and episode boundaries, and optional trajectory persistence. Wants to change policies and experiment configuration without editing rules code or implementing a custom environment integration for every framework.

Success means a documented example can collect experience, update a policy, evaluate it, and reload a checkpoint unattended. The researcher can distinguish an algorithmic failure from invalid actions, hidden-information leakage, or broken environment semantics. A high collection rate is useful only if the resulting samples are valid.

### P2. Experiment owner / evaluation researcher — primary

**Goal:** compare policies or experimental settings under controlled, repeatable conditions.

Runs many games from the CLI or scheduled jobs, with explicit decks, seeds, seat assignments, opponents, limits, and resource budgets. Needs structured outcome summaries, complete episode accounting, versioned configurations, and selected trajectories to investigate surprising results.

Success means another run can reproduce the setup and explain differences. Wins, draws, truncations, and failures remain separate; a policy is not rewarded for causing engine errors. Conclusions are scoped to the frozen pool and evaluation protocol, not presented as general Magic playing strength.

### P3. Trajectory consumer / data researcher — primary

**Goal:** use recorded experience for offline analysis, dataset construction, imitation learning, or compatible offline-RL experiments without rerunning every simulation.

Needs schema-versioned datasets, policy provenance, exact action-time observations/masks, per-seat sequences, terminal outcomes, and completeness checks. May use a different training stack from the one that generated the data. Needs to know which fields are absent, privileged, or unsuitable for a particular algorithm.

Success means loading a dataset into numeric batches preserves temporal order, player perspective, and reward semantics. Corrupt/incomplete episodes are detected, and opponent-private data cannot accidentally become policy features. Recorded data is not assumed suitable for every offline algorithm merely because it contains transitions.

### P4. Rules implementer / automated development agent — enabling

**Goal:** implement or repair a supported behavior with independent evidence that it is correct.

Uses rules references, focused fixtures, mature-engine bridges, and the regression suite. Needs deterministic failing cases, strict scripted choices, concise diagnostics, and replay/state-diff artifacts. A coding agent must be able to run the entire workflow without a human supplying game decisions or dismissing GUI prompts.

Success means demonstrating a failing behavior test, making a general implementation change, and passing the relevant independent checks without weakening expectations. Reference-engine disagreements are visible and adjudicated against the rules, not resolved by blindly choosing whichever result makes tests pass.

### P5. Performance / integration engineer — enabling

**Goal:** increase useful simulation and collection throughput without changing game semantics or making integrations fragile.

Profiles rules execution, state memory, batching, encoding, language-boundary overhead, policy inference, and recording. Needs frozen workloads, reproducible benchmarks, native/adapter equivalence checks, and independently configurable telemetry and capture.

Success means a measured improvement on an equivalent workload with unchanged conformance and trajectory results. Memory, recording cost, and end-to-end collector throughput are reported alongside core speed; moving work outside the timer is not an improvement.

### P6. CLI investigator / optional human player — secondary

**Goal:** inspect a particular game or decision and, when useful, manually try a legal alternative.

Needs readable state, stack and priority information, legal choices, seat-filtered views, and replay inspection. Does not need a browser, artwork, matchmaking, or a polished game client.

Success means the same actions available to agents can be understood and exercised through the CLI. This mode is a debugging and exploration aid, not a prerequisite for simulation, tests, or release qualification; scripted clients exercise it automatically.

## Use cases

The following workflows define what users must be able to accomplish. They are requirements for the proposed engine, not claims that these commands or integrations exist today. Detailed schemas and gates appear later in this RFC.

### UC1. Run a reproducible batch of games

- **Actors:** P2, P5; a noninteractive CLI job.
- **Workflow:** select the supported decks, policies, episode budget, seeds, worker count, and limits; run both seats automatically; optionally record a deterministic subset of episodes.
- **Outputs:** resolved run manifest, outcomes, throughput/memory metrics, failure counts, and references to any captured trajectories/replays.
- **Acceptance:** no terminal input or display required; each started episode is accounted for; changing worker scheduling does not change a given episode under the same seeded policies. Missing policies or unresolved choices fail explicitly rather than falling back to human input.

### UC2. Train an agent using an existing RL framework

- **Actors:** P1; a Python collector/trainer.
- **Workflow:** install one optional integration, configure a fixed opponent or external self-play policy mapping, collect batched decisions, apply masks, update the policy, evaluate, and save/reload its checkpoint.
- **Outputs:** valid learning batches, model checkpoints, run metrics, and optional durable trajectories with policy versions.
- **Acceptance:** the required framework smoke test performs actual learning updates unattended. Repeated decisions by one seat, opponent interleavings, terminal rewards, and truncations are handled correctly. The training path does not parse CLI output or require an interactive-session process.

### UC3. Compare two policy versions fairly

- **Actors:** P1, P2.
- **Workflow:** freeze the pool, rules, opponent, limits, and evaluation seed list; evaluate both candidates across starting seats and the supported matchups; keep evaluation results separate from training data used to select the policy.
- **Outputs:** per-matchup/per-seat outcome counts, episode-level results, uncertainty estimates where reported, and trajectories for selected failures or surprising decisions.
- **Acceptance:** the comparison identifies policy and environment versions and uses the same declared protocol. Truncated/failed games are not silently treated as wins, draws, or completed simulations. Paired seeds control initial conditions but are not claimed to make diverging policy trajectories identical.

### UC4. Capture and reuse a trajectory dataset

- **Actors:** P1, P3.
- **Workflow:** enable capture for all episodes or a declared episode-level subset; persist sharded data; validate it; load per-seat sequences or training batches in a separate analysis/training process.
- **Outputs:** versioned dataset manifest, decision records, episode outcomes, checksums, policy provenance, and optional restricted replay links.
- **Acceptance:** a record/reload round trip preserves observations, masks, actions, rewards, and boundaries. The next learning observation belongs to the same seat, not automatically the next actor. Missing behavior probabilities remain absent; readers reject incomplete episodes by default. Backpressure and storage failures cannot silently drop samples.

### UC5. Add or fix a rule behavior test-first

- **Actors:** P4, including an autonomous coding agent.
- **Workflow:** choose a supported capability; write a rule-referenced fixture that fails for the intended reason; establish expected checkpoints using independent reasoning and applicable reference executions; implement; refactor; run regressions and relevant differential checks.
- **Outputs:** behavior tests, provenance/rules references, red/green evidence, implementation change, and comparison artifacts.
- **Acceptance:** all consequential choices are scripted; no human game decisions are needed. A passing test cannot be obtained by deleting assertions, hardcoding fixture identities, or silently changing scope. Tests outside the advertised pool remain explicitly outside its coverage claims.

### UC6. Reproduce and diagnose a disagreement

- **Actors:** P4, P6; automated minimization tooling.
- **Workflow:** take a failing fixture, fuzz trace, or recorded episode; replay it under matching versions; identify the first divergent checkpoint; reduce the case; compare with pinned XMage and Forge where applicable.
- **Outputs:** minimal reproduction, semantic state diff, implicated rule/capability, reference-engine results, and a permanent regression fixture after adjudication.
- **Acceptance:** diagnosis does not depend on reproducing wall-clock timing or manually playing the game. Incompatible snapshots/replays are rejected clearly. Upstream bugs or rules-version differences remain documented exceptions rather than disappearing from the result counts.

### UC7. Optimize throughput without corrupting results

- **Actors:** P5.
- **Workflow:** run a frozen benchmark with recording off and on; profile; change representation, allocation, batching, or encoding; repeat on the same hardware/configuration; run conformance and scalar/batch/trajectory equivalence checks.
- **Outputs:** before/after results, profiles, memory curves, recording overhead, and correctness evidence.
- **Acceptance:** measured work includes declared reset/encoding/recording costs. Priority windows, legal choices, and termination semantics are unchanged; speed is not obtained by dropping trajectories, shortening games, or hiding failures. Core speed and inference-inclusive collection are separate results.

### UC8. Inspect or interact with a game through the CLI

- **Actors:** P6 or scripted clients used by P4.
- **Workflow:** inspect a saved replay or attach to a seat in an optional local interactive session; view permitted information; submit a legal choice; inspect subsequent priority/stack changes; detach or concede.
- **Outputs:** readable or structured seat-filtered views, validated action responses, and optional replay/trajectory artifacts.
- **Acceptance:** CLI actions use the same rules and legality checks as agents. Stale/wrong-seat commands do not mutate state; private information stays scoped to the authorized view. Automated transcript tests cover the workflow without requiring a person to play.

### Priority and boundaries

UC1–UC7 form the core research/development workflow; UC8 is a secondary access mode. All eight must be automatable. Broad card coverage, public multiplayer hosting, a graphical client, and a built-in distributed training platform are not implied by these personas or use cases. Detailed acceptance criteria below govern the initial implementation; new use cases that require additional mechanics or services need an explicit scope change.

## 1. Decision summary

Build a from-scratch, headless Rust engine primarily for unattended simulation and agent training, using a deliberately small two-player Magic card pool. Expose a CLI for simulation, experiments, tests, and debugging; human PvP is an optional interaction mode using the same rules implementation. No human is required to develop, test, benchmark, or qualify the engine. Start with two fixed 40-card decks drawn from Foundations printings, not Standard, not the whole Foundations set, and not a claim to support an official sanctioned format.

Optimize **correct completed simulations per CPU-second**, while exposing batched decisions for efficient policy inference. Treat memory footprint, Python integration overhead, and human-facing latency as separate constraints. Do not optimize by deleting priority windows, hiding legal choices, or treating unfinished games as completed games.

Build an independent conformance harness first and implement capabilities test-first: demonstrate a failing test, implement the behavior, then refactor under the full regression suite. Seed the corpus with original rules-derived cases and carefully adapted XMage tests. Verify execution against pinned XMage and Forge builds as a release requirement; both are fallible references, not the specification. Every supported capability needs positive, negative, interaction, and regression tests.

Deliver native batching and a Python binding, a PettingZoo AEC compatibility adapter, a fixed-opponent Gymnasium adapter, and an automation-first CLI with optional interactive play. Require executable training checks for RLlib, TorchRL, and SB3-Contrib. PettingZoo is an interoperability choice, not a claim of optimal throughput: the native batch API remains independent of any trainer. Keep expensive tracing, terminal rendering, and local IPC outside the simulation hot path. No browser, HTTP service, or web frontend is required.

Treat trajectory capture as a first-class output for training, evaluation, and analysis, distinct from debug logs and deterministic replays. Support versioned, player-observation-safe datasets from CLI simulations and Python collection, with complete episode accounting and measured recording overhead.

The proposed sequence is: **scope and verifier/reference bridges → test-driven scalar engine and CLI → measured native batching → validated RL adapters → torture testing and release qualification**. Mature-engine checks grow with each capability; they are not postponed until the end.

## 2. What “smallest” and “fastest” mean

### Card scope, not a format popularity contest

The smallest useful implementation is a custom, frozen card pool exercising a manageable set of rules. Standard is a multi-set constructed format and is not an appropriate minimum implementation boundary. A single-set Limited environment is smaller, but still brings many mechanics unrelated to the first research questions. Foundations' Beginner Box demonstrates fixed 20-card packets combined into 40-card decks; this RFC uses that product as inspiration, not its exact deck lists or tutorial rules. See the [official Beginner Box contents](https://magic.wizards.com/en/news/feature/foundations-beginner-box-contents).

The POC is named `foundations_micro_v1`. It is intentionally custom. We want recognizable Magic decisions and exact behavior for supported interactions, not broad card coverage. Smaller historical sets or novelty formats are not automatically simpler rules implementations.

### Separate the performance objectives

| Metric | Definition | Why it matters |
| --- | --- | --- |
| Valid completed games/s/core | Rules-terminal games divided by total elapsed time and allocated physical cores | Primary native simulation efficiency metric |
| Valid completed games/s/host | Same numerator across the configured worker pool | Actual machine throughput |
| Policy decisions/s | Externally exposed decisions processed, including choice continuations | Rollout supply; report decision-type distribution |
| Logical actions/s | Completed casts, activations, combat declarations, passes, etc. | Detect apparent speedups caused by action re-encoding |
| Resident games/GiB | Live game states per incremental memory usage | Capacity; not itself a speed measurement |
| End-to-end training samples/s | Samples reaching the learner, including encoding, inference, transport, and collection | What an RL experiment actually experiences |
| Decision latency p50/p95/p99 | Native compute and queue wait reported separately | PvP responsiveness and inference scheduling |

“Concurrent games per second” conflates capacity with throughput. Ten thousand games waiting for a human produce essentially no simulation throughput. The objective is to maximize useful rollout production at a stated resource budget, with reproducible behavior and bounded memory.

No global fastest-engine claim is possible without comparable benchmarks. This RFC defines the workload and measurement contract needed to make such comparisons meaningful.

## 3. Concrete game scope

### Frozen decks

Two proposed starter decks, each exactly 40 cards: 16 basic lands and 24 spells. These are engineering fixtures, **not claimed to be balanced decks**. Support red versus green and same-deck mirrors; evaluate both starting seats.

| Red deck | Copies | Green deck | Copies |
| --- | ---: | --- | ---: |
| Mountain | 16 | Forest | 16 |
| Swab Goblin | 4 | Bear Cub | 4 |
| Axgard Cavalry | 2 | Llanowar Elves | 3 |
| Crackling Cyclops | 3 | Druid of the Cowl | 2 |
| Firebrand Archer | 3 | Magnigoth Sentry | 2 |
| Dragon Fodder | 4 | Tajuru Pathwarden | 2 |
| Goblin Surprise | 2 | Thornweald Archer | 3 |
| Viashino Pyromancer | 2 | Giant Growth | 3 |
| Thrill of Possibility | 2 | Bite Down | 3 |
| Shivan Dragon | 2 | Wildheart Invoker | 2 |
| **Total** | **40** | **Total** | **40** |

This is 20 distinct card names plus a red 1/1 Goblin creature token definition. “Foundations” here includes FDN-numbered supplemental Beginner Box/Starter Collection printings, not just the main booster sheet. For example, [Shivan Dragon is FDN 763](https://scryfall.com/card/fdn/763/shivan-dragon). Card discovery can use [Scryfall's search API](https://scryfall.com/docs/api/cards/search); the engine must not depend on a live API at runtime.

Before implementation, create a versioned manifest containing each card's Oracle identifier, selected printing, characteristics, supported behavior identifier, retrieval date, and content hash. Pin Oracle text separately from the rules revision. Card names alone are not a stable executable specification. Do not include artwork in the core package.

### Rules baseline

Pin the [Comprehensive Rules effective September 25, 2026](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt), linked from the [official rules page](https://magic.wizards.com/en/rules). Future rules updates require a deliberate version change and corpus migration; they must not silently alter saved experiments. Store the source URL, effective date, retrieval date, and document checksum in the manifest. Reference relevant numbered rules in tests instead of redistributing the full document by default.

Supported behavior:

- Two players, 20 starting life, seven-card opening hands, London mulligans, and the starting player's skipped first draw. Best-of-one games; no sideboards.
- The complete turn/phase/step structure needed by the pool, priority and consecutive passing, the stack, spell resolution, state-based actions, and triggered abilities.
- Land plays, colored and generic mana, explicit payment choices, creature mana abilities, additional discard costs, modal spells, and target selection/revalidation.
- Creature casting, summoning sickness, tapping, activated abilities, attacking, blocking, combat damage, marked damage, and end-of-turn effects/cleanup.
- Flying, reach, haste, vigilance, trample, and deathtouch. Correct behavior for multiple blockers and simultaneous combat damage.
- Cast triggers, enters-the-battlefield triggers, controller ordering of simultaneous triggers, and active-player/nonactive-player ordering where applicable.
- Token creation and disappearance after leaving the battlefield, zone changes with new object identity, and temporary power/toughness modifications required by the pool.
- Loss from life total and attempted draw from an empty library, concession, and applicable simultaneous-loss/draw behavior. An empty library alone is not a loss.
- Maximum hand size and cleanup discards, including additional cleanup processing when the rules require it.

Implement these as general mechanics for the pool, not special cases that recognize deck lists or test sequences. Continuous effects need the relevant ordering rules; this does not commit the POC to every layer interaction in Magic.

Important migration trap: the 2024 Foundations rules update removed combat damage assignment order. Do not import older “order blockers, then respond” scenarios unchanged. Use current combat damage assignment, including trample and deathtouch requirements. See the [Foundations update bulletin](https://magic.wizards.com/en/news/announcements/foundations-update-bulletin).

### Explicit non-goals

No full Standard or full Foundations support; deck construction, drafting, sideboarding, tournament operations, multiplayer, or Commander. No planeswalker or battle implementation just because a supported card's text can reference those types: their absence in the pool removes those target candidates.

No artifacts, enchantments, equipment, auras, counters, replacement/prevention effect framework, copying, control changes, first/double strike, graveyard recursion, search effects, alternate costs, or arbitrary card scripting unless a scope review demonstrates that the frozen pool requires them. No card-text NLP interpreter. Unsupported cards and manifests must be rejected explicitly.

No GPU rules simulator, distributed game-state service, browser/GUI frontend, HTTP/WebSocket service, remote multiplayer hosting, matchmaking, account platform, or engine-specific learning algorithm in v0. No claim that passing this corpus proves general Magic correctness.

## 4. Architecture and ownership boundaries

```text
                   frozen card manifest + conformance corpus
                                      |
                         deterministic Rust rules core
                           /                     \
              native batched runner       CLI debug / optional play
                       |                          |
             Python binding / CLI jobs      seat-filtered views
                 /     |     \
        native batch  AEC   fixed-opponent Gymnasium
             |        / \              |
       custom loops RLlib TorchRL   SB3-Contrib
```

Proposed workspace boundaries:

| Component | Owns | Must not own |
| --- | --- | --- |
| `mtg-core` | Rules, state, decisions, legal actions, player views, deterministic RNG state | Python, sockets, wall-clock time, telemetry exporters, model inference |
| `mtg-cards` | Frozen definitions and small typed behavior/effect descriptions | Runtime HTTP fetches or general scripting VM |
| `mtg-batch` | Independent game slots, worker scheduling, batch encode/advance/reset | Alternative rules implementation |
| `mtg-python` | Safe array ownership, native binding, adapters | Per-card Python callbacks |
| `mtg-cli` | Unattended simulations, benchmark/test commands, machine-readable results, replays, optional seat-scoped interactive play | Browser assets, web server, independent rules, required per-step training transport |
| `mtg-conformance` | Fixture loading, assertions, provenance, differential tooling | Expected results calculated by the implementation under test |
| `mtg-bench` | Frozen workloads, runner metadata, reproducible results | A separate simplified “benchmark rules” engine |

### Why Rust for the core, rather than Python?

This is a workload-based design choice, **not a measured Rust-versus-Python result**. Python would be a reasonable choice if the primary objective were the fastest prototype or easiest rules experimentation. Here, the stated priority is many correct game simulations per CPU-second with low per-game memory, so native execution and control over representation/allocation are valuable from the outset.

| Consideration | Rust core | Pure-Python core |
| --- | --- | --- |
| Branch-heavy rules execution | Compiled state transitions and typed effect dispatch; compact storage is straightforward | Interpreter dispatch and Python object operations occur on the per-decision path unless work moves into native/JIT code |
| Resident-state memory | Dense numeric records, shared immutable definitions, and reusable buffers under explicit control | Convenient objects/dicts can cost more memory; careful arrays/native structures can mitigate this |
| CPU parallelism | Worker threads can execute independent native games in one process | Standard GIL-enabled CPython limits Python-bytecode parallelism; processes, native extensions, or free-threaded builds offer alternatives |
| Research iteration | More compile-time/API work and a Python binding to maintain | Faster interactive development, familiar tooling, direct integration with trainers |
| Correctness | Types and ownership help prevent some state/memory mistakes; do not prove Magic rules correctness | Easier to inspect/prototype; needs the same independent semantic tests |
| Integration | Batch work behind a thin Python extension; separate packaging/build burden | Simplest initial packaging; performance work may eventually require moving the core elsewhere |

Do not justify Rust with the blanket claim that Python cannot run threads in parallel. CPython provides optional [free-threaded builds](https://docs.python.org/3/howto/free-threading-python.html), with compatibility and performance considerations. Removing the GIL does not by itself remove Python object/dispatch costs, nor does it establish which implementation is faster for this workload. Native code invoked from Python can also run concurrently when it releases the GIL; the binding must follow [PyO3's parallelism guidance](https://pyo3.rs/main/parallelism.html).

The division is **Rust for the rules/state machine and batching; Python for policies, collectors, training, analysis, and experiment configuration**. Most researcher-facing code should remain Python. No per-card Python callbacks in the hot loop. A NumPy/JAX/Numba-first implementation could be investigated, but would require a suitable fixed-shape/compilable representation; it is not equivalent to obtaining high throughput from arbitrary Python classes. C++ is also viable; this RFC prefers Rust's ownership/type discipline for a new core, not an unsupported claim that Rust is inherently faster than C++.

Record a correctness-gated baseline at M2, with allocation and boundary-cost profiles. Revisit representation, batching, or the language decision if evidence contradicts the assumptions. Do not maintain a second complete Python rules engine merely for comparison: it would duplicate the semantic maintenance burden. A small equivalent Python workload can be an optional experiment, not a prerequisite or a substitute for full-game measurements.

### State and execution model

Use compact numeric identifiers and a shared immutable card table. Keep mutable state per game: zones, objects, players, stack, pending triggers/effects, decision continuation, and random stream state. Start with straightforward dense storage and reusable vectors; profile before introducing unsafe code or a complicated entity-component framework.

Use generation-tagged handles so a creature that changes zones is not accidentally the same target when it returns. Reuse allocations across resets. Store observation buffers for ready batches rather than permanently padding every resident game's full observation.

A game runs on one worker at a time. Parallelize **between games**, avoiding a mutex or task per rule transition. A bounded worker pool owns shards of game slots. Start with static sharding; introduce dynamic scheduling only when measurements justify it. Policy inference receives batches of ready player decisions. Do not allocate a thread, process, Python environment object, or network connection per headless game.

Advance deterministically until a player decision, terminal state, or internal scheduler yield. Use an explicit continuation representation for multi-stage effects and choices. A work quantum may yield execution to keep workers responsive; this is not a player action, rules event, terminal state, or RL transition. Resume without changing the result.

The core has no wall-clock dependence. Derive per-episode random streams from a master seed and stable episode identifier, independently of worker order. Keep environment shuffling separate from policy exploration and scheduling randomness. Record the PRNG algorithm/version; a seed without that context is insufficient for long-lived reproducibility.

### Core interface contract

The following are conceptual operations, not a committed Rust ABI:

```text
reset(config, episode_seed) -> Decision | Terminal
decision() -> {decision_id, actor, kind, candidates}
apply(decision_id, candidate_id) -> Decision | Terminal | InternalYield
observe(player) -> PlayerObservation
snapshot() -> VersionedSnapshot
restore(snapshot) -> Result
```

`apply` checks the current decision and action before mutation. Invalid or stale input returns a structured error and leaves state unchanged. A candidate identifier is valid only for its decision generation. Internal trusted batching may use a validated fast path, but it must have equivalent semantics and tests.

Snapshots include pending continuations and RNG state. Replays include manifest/rules/engine versions, initial configuration, seeds, and semantic decisions. Raw action indices are insufficient because candidate ordering can change between versions. Restore rejects incompatible versions unless an explicit migration exists.

### Legal actions without combinatorial explosion

Do not enumerate every possible attacker subset, blocker mapping, target tuple, and payment combination as a single giant flat list. Express choices as typed decision continuations: choose a spell/ability, mode, targets, payment, attackers, blockers, or damage allocation, followed by completion where appropriate.

Every supported legal combination must remain expressible. Continuations must preserve the rules' atomicity: intermediate casting/payment and combat-selection work does not give an opponent priority or expose provisional choices. Cancellation and backtracking are allowed only where they do not leak information or violate committed choices. A UI may assemble a whole logical command; the core still validates it through the same decision machinery.

The action encoder needs a reviewed upper-bound analysis for this pool. Fixed tensor capacities are an encoding contract, not an excuse to remove legal moves. Overflow raises a specific diagnostic and quarantines the episode; it is never silent action truncation. If a bound cannot be justified, use another continuation or a larger versioned schema.

## 5. Throughput plan and performance budget

### Provisional targets, not acceptance claims

Use a designated Linux x86-64 machine with at least eight physical cores and 32 GiB RAM for repeatable performance work, recording the exact CPU and software. Run correctness smoke tests on macOS ARM as well. These are initial engineering hypotheses to validate after the scalar prototype, not measurements or promises:

| Area | Initial target | Qualification |
| --- | --- | --- |
| Scalar native decisions | 100,000 externally exposed decisions/s on one physical core | Frozen pool, named native policy, including required legality work; report logical actions too |
| Host scaling | At least 4× single-core throughput with eight physical worker cores | Same workload and policy; CPU resources stated, SMT distinguished |
| State capacity | 10,000 resident headless games; average marginal core-state memory ≤64 KiB/game | Excludes shared tables and inference buffers, which must also be reported in total RSS |
| Python batched collection | At least 80% of the equivalent native encoded-collection throughput at batch size 128 | Same observation/action payload, no policy inference; include binding costs |
| Counters-only instrumentation | ≤5% throughput overhead versus instrumentation off | Identical workload, repeated measurements |
| Human CLI PvP | p95 accepted-command-to-view-update processing under 50 ms at a declared local load | Excludes human input and terminal display time; no throughput claim inferred from this |

There is deliberately no invented completed-games/s target. Once the workload is measured, report that rate directly alongside mean decisions/game and outcome fractions. For illustration only, 100,000 decisions/s and 500 decisions/completed game would imply 200 games/s before other overhead; neither input is established here.

Missing these hypotheses triggers profiling and a documented budget revision, not weakened rules or hidden benchmark changes. The first correctness milestone is not blocked by an unvalidated speed target. A later performance claim must show its measurement artifact.

### Required benchmark tracks

1. **Core transition:** fixed, validated traces exercising priority, casting, triggers, combat, and cleanup. Report both replay/application and legal-action generation costs.
2. **Native rollout:** complete seeded games with a deterministic heuristic policy and separately a legal-random policy. Include reset/shuffle and all choices. Report policy time rather than concealing it.
3. **Encoded collection:** the same workload plus player observation and candidate-feature construction. Compare native and Python batch paths with equivalent work.
4. **Inference-inclusive collection:** a fixed model, device, precision, batch policy, and opponent schedule. Separate engine, encoding, inference, transfer, and queue time.
5. **Capacity and stress:** resident-state memory curve, token-heavy positions, many legal targets, large stacks/trigger queues, and unequal game lengths.
6. **CLI:** command validation and player-view rendering latency, measured separately from headless simulation; terminal output is disabled in all throughput tracks.

Use a frozen workload version, deck hashes, episode seeds, action-schema version, and policy implementation/version. Report compiler flags, engine commit, CPU/core affinity, worker count, RAM, OS, Python/package versions where applicable, and instrumentation mode. Benchmark one core and a worker-count sweep; sweep batch sizes including 1, 32, 128, and 512.

Start with 10 seconds of warmup and at least five 30-second measurement windows; extend runs when variance is high. Publish the distribution and raw machine-readable results, not just the best run. Denominator time includes failures and reset overhead. Count only actual rules-terminal games in completed-game throughput; report wins, draws, external truncations, errors, and all started games separately. Do not improve throughput by lowering the horizon or accepting more errors.

Profile allocation count/bytes, legal-action generation, observation encoding, effect dispatch, cache behavior, and synchronization. A release report must identify the dominant costs. Prefer fewer allocations, compact state, batch crossings, and shared immutable data before unsafe micro-optimizations. No claim of allocation-free play until measured on adversarial cases as well as typical games.

## 6. Instrumentation without making the hot path expensive

Provide four explicit modes:

| Mode | Contents | Intended use |
| --- | --- | --- |
| `off` | Only indispensable error handling | Performance baseline |
| `counters` | Worker-local counts and sampled timing histograms | Default research runs |
| `sampled_trace` | Counters plus selected game/decision spans and checkpoints | Debugging and profiling |
| `full_replay` | Complete decision log, versions, seeds, and diagnostic checkpoints | Reproducing a failure or audited experiment |

Aggregate worker-local counters periodically; avoid a shared atomic update and clock read for every small rules operation. Export through a separate layer, initially JSONL summaries and benchmark JSON. Prometheus/OpenTelemetry integration can be added at the boundary without becoming a dependency of the rules core.

Required metrics include started/completed/truncated/failed games, decision counts by type, logical actions, rules transitions, encoding time, worker busy/idle time, ready-queue delay, batch fill, resets, memory high-water marks, capacity overflows, invalid/stale actions, and dropped diagnostic records. Report policy/inference time independently.

Keep metric label cardinality bounded. Never put game IDs, seeds, card names, full observations, or arbitrary error strings into metric labels. Put detailed identifiers in access-controlled logs. Public/player-facing telemetry must not reveal hands, library order, pending private choices, or RNG state.

Use bounded trace buffers. Debug traces may drop records with an explicit counter. A promised complete replay may not silently drop data: mark the recording incomplete or fail that recording operation. Logging backpressure must not quietly alter game rules or make benchmark results incomparable.

Version every run artifact. Include counters for every exceptional path so the fastest reported run cannot hide invalid episodes. Test semantic equivalence across instrumentation modes and measure their overhead separately.

Trajectory recording is independently configurable; instrumentation `off` does not imply that requested training data may be discarded. Measure trajectory serialization, compression, queueing, and storage overhead separately from the counters-only budget. See the [trajectory contract](#trajectory-capture-is-a-first-class-output).

## 7. The “torture suite” strategy

### The experiment and the transferable lesson

The likely reference is Nicholas Carlini's February 2026 [C-compiler experiment at Anthropic](https://www.anthropic.com/engineering/building-c-compiler). It used existing test suites, including GCC torture tests, regression checks, and GCC as a comparison oracle. The important lesson for this project is to invest in a reliable verifier and actionable failure artifacts before asking implementation work to proceed autonomously. Passing a large suite was not equivalent to complete compiler correctness.

GCC's [C testsuite documentation](https://gcc.gnu.org/onlinedocs/gcc-7.2.0/gccint/C-Tests.html) describes torture cases exercised under different compiler settings. Our equivalent is to run the same semantic game cases across scalar/batched execution, build modes, bindings, snapshots, worker schedules, and instrumentation modes. This is a testing strategy; it does not require reproducing the experiment's multi-agent workflow.

### Where the tests come from

There is no established engine-neutral Magic conformance corpus identified by this research that we can simply download and treat as complete. There are substantial existing engine-specific tests. Build a small portable corpus using the following hierarchy:

| Source | Concrete starting point | How we use it | Caveat |
| --- | --- | --- | --- |
| Official rules | Pinned Comprehensive Rules, especially starting the game, priority, casting, resolution, combat, triggers, and state-based actions | Write original examples with explicit expected intermediate states | Rules text is a specification, not executable tests |
| Official card rulings | [Foundations release notes](https://magic.wizards.com/en/news/feature/foundations-release-notes), then relevant card rulings | Convert applicable edge cases into scenarios for the selected pool | Only include cases that actually apply; record publication/version |
| XMage | [`Mage.Tests/src/test/java/org/mage/test/cards`](https://github.com/magefree/mage/tree/master/Mage.Tests/src/test/java/org/mage/test/cards), particularly `damage`, `cost`, `mana`, `targets`, `triggers`, `abilities`, and `rules` | Primary source of candidate scenarios to translate | Java harness and default setup are not portable; review rules vintage |
| XMage harness example | [`damage/AssignDamageTest.java`](https://github.com/magefree/mage/blob/master/Mage.Tests/src/test/java/org/mage/test/cards/damage/AssignDamageTest.java) | Study explicit setup, attack/block choices, strict choice mode, and assertions | Uses Thorn Elemental, outside our pool; harness example, not an accepted v0 fixture |
| XMage choice example | [`targets/TargetsSelectionOnResolveTest.java`](https://github.com/magefree/mage/blob/master/Mage.Tests/src/test/java/org/mage/test/cards/targets/TargetsSelectionOnResolveTest.java) | Study decision cardinality and exhaustive choice cases | Do not infer that all target-at-resolution behavior is in scope |
| Forge | [`forge-gui-desktop/src/test/java/forge/gamesimulationtests`](https://github.com/Card-Forge/forge/tree/master/forge-gui-desktop/src/test/java/forge/gamesimulationtests) | Required second execution reference for the critical-interaction suite | Different harness and licensing; not an automatically importable permissive corpus |
| Our implementation failures | Minimized fuzz traces, bug reports, replay mismatches | Every fix adds an independent regression fixture | A test that merely blesses current output is not a verifier |

The inspected XMage [test base](https://github.com/magefree/mage/blob/master/Mage.Tests/src/test/java/org/mage/test/serverside/base/CardTestPlayerBase.java) also illustrates why defaults must be translated: its game/deck setup is not our fixed 40-card format. Import the scenario's intended semantics, not hidden harness behavior or AI fallback choices. Script all consequential choices explicitly.

XMage's [repository license](https://github.com/magefree/mage/blob/master/LICENSE.txt) is MIT. Preserve required notices and record upstream commit, file, method, and modifications for adaptations; inspect individual files for additional notices. Forge is GPL-licensed; initially keep it as a separately installed reference and do not copy its test code into our corpus without resolving the licensing implications. The [earlier survey](../open-source-mtg-engines.md) discusses these projects' licensing. Choosing a repository license and reviewing redistributed card/rules data remains a release prerequisite, not something this RFC silently settles.

The engine is authored from scratch, but a corpus adapted from another project is not “clean-room.” Be precise about provenance. Do not vendor entire upstream trees merely to obtain a few scenarios.

### Fixture format and provenance

Use a versioned, engine-neutral JSON or YAML fixture schema. Compile fixtures into runtime test structures outside the hot path. Each fixture records:

- Stable ID, description, rule references, rules revision, card-manifest version, and required capabilities.
- Source kind (`original`, `adapted`, `regression`, or `differential`), upstream commit/path/method where applicable, licensing notice, and explanation of adaptations.
- Explicit initial state: active player, phase/step, priority, zones, life, mana, object status, and ordered libraries. Seeds alone are not sufficient for cross-engine comparisons.
- A complete script of semantic actions and choices, including passes, targets, payment, and trigger ordering when relevant.
- Assertions at named decision or settled-state checkpoints: zones, object identity, life, marked damage, power/toughness, stack, priority, legal choices, and player-visible information.
- Expected invalid-action behavior and invariant checks where relevant.

Use a synthetic-state constructor for focused rules tests, with validation of its declared assumptions, and a separate end-to-end constructor using normal reset/mulligans. Never confuse a convenient synthetic position with evidence that the position is reachable.

Illustrative fixture intent, to be encoded after the schema exists:

```text
ID: bite-down/responding-growth/cleanup
Setup: both players control an undamaged Bear Cub; sufficient untapped
       Forests; P0 has Bite Down; P1 has Giant Growth; explicit libraries.
Script:
  P0 casts Bite Down: own Bear Cub -> P1's Bear Cub; pays and passes.
  P1 responds with Giant Growth targeting their Bear Cub; pays and passes.
  P0 passes; Giant Growth resolves.
Assert: P1's creature is 5/5; Bite Down remains on the stack.
  Both pass; Bite Down resolves.
Assert: P1's creature survives as 5/5 with 2 damage marked.
  Explicitly pass through remaining decisions to cleanup.
Assert: it survives as 2/2 with no marked damage; growth expiration and
        damage removal must not create an intermediate lethal-damage check.
```

This combines priority, stack order, targeting, damage, temporary effects, and cleanup without adding another card. The executable version must spell out every checkpoint and choice; the prose above is not an implemented test.

### Required scenario families

| Family | Minimum behaviors to cover |
| --- | --- |
| Setup and randomness | Mulligan decisions/bottoming; first-player draw rule; deterministic reset; ordered-library draw; no hidden library leakage |
| Priority and stack | Pass with empty/nonempty stack; priority after casting and resolution; response chains; no skipped response window |
| Costs and mana | Insufficient/wrong mana; generic choices; creature mana ability timing; summoning sickness; Thrill discard as cost; no partial mutation on rejection |
| Target legality | Own/opponent restrictions; missing required targets; target leaving battlefield; partial target invalidation; legality checked again at resolution |
| Triggers | Cast versus resolution timing; Archer/Cyclops interaction with counterfactual failed casts; Pyromancer ETB targeting; multiple-trigger ordering |
| Combat | Legal attackers/blockers; flying/reach; haste/vigilance; multiple blockers; simultaneous damage; deathtouch/trample allocation; unblocked/blocked status |
| Continuous changes | Stacked Growth/Surprise/activated boosts; power read at resolution; correct expiration and cleanup |
| Objects and zones | Token death/removal; new zone identity; dead source with ability on stack; damage and attachment-free state bookkeeping |
| Terminal states | Lethal damage; simultaneous lethal outcomes; attempted empty-library draw; concession; terminal rewards exactly once |
| Decisions and encoding | Every legal combination expressible; masked action rejected; candidate IDs stale after advance; no action-capacity truncation |
| Privacy and PvP | Opponent hand/library changes do not reveal identities through observations, masks, candidate order, or error responses |
| Replay and batching | Scalar/batch equality; snapshot in a pending choice; worker-count independence; reset only requested rows |

Test both normal and adversarial combinations. The absence of counterspells does not justify skipping priority; responding with a buff is already strategically meaningful.

### Properties, fuzzing, differential checks, and mutation tests

Generate reachable states through legal play and exercise invariants after every transition. Useful properties include deterministic replay, observation redaction, unchanged state after invalid input, valid object handles, appropriate zone membership, and scalar/batch equivalence. Account explicitly for token creation/disappearance rather than asserting naive conservation of all objects.

Use metamorphic checks carefully: renaming internal IDs should preserve semantics; swapping seats must also swap the starting player and private information. Permuting unseen library identities should not change the current public view or legal mask when the player has no knowledge of those identities. Do not assert that future game outcomes are unchanged.

Fuzz legal action sequences, malformed input, checkpoint/restore, and batch reset schedules. On failure, minimize the action trace and state, then preserve the result as a deterministic fixture. Wall-clock fuzz budgets must report seeds, executions, and minimized failures.

For differential testing, first align rules revision, card definitions, initial library order, all choices, and the exact checkpoint. Compare semantic state, not engine-specific object IDs or UI events. A disagreement with XMage or Forge is a triage item, not automatic proof our engine is wrong. Resolve it against the pinned rules and add a reviewed expected result. Execution against mature engines is mandatory for release; a local fast test command may omit heavyweight reference builds, but cannot claim release verification.

### Mandatory reference-engine verification

Implement two test-only bridges, **XMage first, Forge second**. These are new integration work: neither is assumed to expose a ready-made compatible JSON API. Use their test harnesses to construct states, supply scripted player choices, and export canonical checkpoints. XMage's test base is linked above; Forge's [`BaseGameSimulationTest`](https://github.com/Card-Forge/forge/blob/master/forge-gui-desktop/src/test/java/forge/gamesimulationtests/BaseGameSimulationTest.java) and its `GameWrapper` provide an initial execution entry point. Prove the bridges with a small fixture before building broad adapters.

Run reference engines out of process in a reproducible test environment, never inside the training loop. Pin source commit, card-data revision, JDK/build dependencies, bridge version, and any local patches. Keep dependencies and licenses separate and reviewed; a process boundary alone is not a licensing exemption. Cache builds so repeated comparisons are practical.

The same neutral fixture drives all engines. The bridge may translate object IDs and decision APIs but must not calculate the expected game result or call our engine for legality. Script choices strictly: no AI fallback, automatic combat decisions, or independently seeded shuffles. For end-to-end cases, inject an identical shuffle result/ordered deck through a test-only hook; identical seeds across different RNG implementations are not sufficient.

Compare at rule-level synchronization points rather than every implementation-specific micro-step: priority handoffs, completed spell/ability resolutions after applicable state-based actions and triggers, completed combat declarations/damage, cleanup, and game end. Normalize life, zones, object characteristics/status, marked damage, mana, stack content/order, active player/priority, and results. Compare legal semantic choices where the reference harness exposes them. Any unobservable field must be marked explicitly; a winner-only comparison is not full-state verification.

Minimum release coverage:

- XMage executions cover every supported capability and at least one meaningful case for each of the 20 card definitions. Include legal-action rejection and intermediate checkpoints, not just final winners.
- A frozen critical suite of at least 20 independently authored scenarios runs on **both** XMage and Forge. Cover response ordering, costs/mana, targets, cast/ETB triggers, combat keywords and multiple blockers, cleanup, and terminal outcomes. The count is a floor; all listed families must be represented.
- Add scripted end-to-end games in red/green and both mirror matchups, swapping starting seats. Record checkpoints along the game, not merely identical outcomes. Scenarios adapted from an engine's own tests do not count as independent corroboration by that same engine.
- Publish planned/executed/agreed/disputed/unsupported counts by engine and capability, with the first divergent checkpoint and minimized reproduction for every mismatch. An unavailable engine or failed build is **not** a passing test.

No unexplained disagreement is allowed at release. If an upstream engine implements an older rule or has a demonstrable bug, preserve the failing reference result, link the rule and upstream issue where available, and add an independently reviewed expected outcome. An explicit exception may document why agreement is inappropriate; it must not silently remove the test or count as agreement. Report the limitation in release notes. If required bridge coverage is missing, the verification milestone remains incomplete until the scope is explicitly revised.

### Test-driven implementation workflow

For every capability or bug fix:

1. Write a rule-referenced fixture and focused unit/property tests before implementing the behavior. Record a failing assertion caused by the missing/incorrect behavior, not merely a syntax or build failure.
2. Run the applicable fixture against the reference engine(s) and inspect checkpoints. Establish expected results independently; disagreement is resolved before accepting a new expectation.
3. Implement the smallest general rule change that passes the tests. Do not special-case fixture IDs or delegate production rules execution to a reference engine.
4. Refactor, then run the mandatory conformance suite, relevant differential cases, replay/batch equivalence, and applicable mutation checks.
5. Keep the red/green evidence and regression IDs with the change. Optimizations must preserve behavior under the same tests and publish before/after benchmarks.

Test the bridges themselves: alter a target, starting life, choice, or checkpoint field in a known fixture and prove the comparison catches the difference. Otherwise three green engines could merely reflect a comparator that ignores the disputed state.

Seed deliberate defects to test the verifier: omit a priority window, make mana abilities use the stack, allow a summoning-sick creature to tap, forget a cast trigger, retain temporary boosts, remove damage too late in cleanup, accept stale actions, expose a hidden card, reverse terminal rewards, or confuse truncation with termination. Each selected mutant must be caught by an identified test. Track mutation results by capability, not just a flattering aggregate percentage.

### Test execution and anti-overfitting rules

Maintain a small deterministic `fast` suite, the complete required `conformance` suite, and longer `torture`/fuzz jobs. The latter vary debug/release builds, scalar/batch paths, Python/native encoders, worker counts, snapshots, and instrumentation modes. Use a deliberate pairwise matrix initially rather than an uncontrolled Cartesian explosion.

Keep future mechanics in a separate backlog. They must not inflate the denominator with skipped tests. Every test in the advertised v0 capability set must pass; no expected failures for supported rules. A planned floor of 100 individually reviewed semantic scenarios helps avoid a handful of happy paths, but the capability matrix and mutation checks determine adequacy—not the raw count or a required quota of imported tests.

Expected outputs must not be regenerated from the engine under test. Changing a fixture expectation requires a rule-based explanation and a visible review diff. Keep additional independently authored cases for milestone validation. Agents or humans implementing a capability may add tests, but must not silently delete difficult cases, loosen assertions, or reduce scope to turn CI green.

Failures print a concise summary, fixture/seed ID, first divergent checkpoint, and paths to replay/state-diff artifacts. An autonomous implementation loop should select a capability, run focused tests, implement, run the fast suite, then run the full mandatory suite before declaring success. Passing only the sample is never completion.

## 8. RL integration contract

### Native batching is the performance path

Provide an in-process Python extension built with PyO3/maturin. Release the GIL during native batch work. Cross the language boundary once per batch, not once per card effect. Start with safe, owned contiguous NumPy-compatible buffers and explicit lifetimes; zero-copy GPU transport is a future optimization, not a v0 promise.

Conceptual batch operations:

```text
reset_many(slot_ids, configs, seeds) -> ReadyBatch
step_many(slot_ids, decision_ids, candidate_ids) -> ReadyBatch
observe_many(slot_ids, players) -> ObservationBatch
snapshot_many(slot_ids) -> snapshots
```

Rows identify stable game slots and acting seats; do not assume alternating turns. A player can make several consecutive decisions, and priority may change many times in one turn. Batched rows must be semantically identical to running those games individually. Support resetting selected finished slots without resetting others.

### Observation and action schema

Expose versioned numeric tensors for public state, the acting player's private hand, public object features, stack/pending-decision features, candidate action features, and padding/legal masks. Use structured records in debugging views, not strings in the training hot path.

Do not reveal opponent hand identities, library order, engine PRNG state, or provisional private choices. Object ordering and action masks are also potential side channels. Public card history and remembered revealed information need a clear contract; a recurrent policy may consume observation history, but the engine must not supply forbidden information to simplify learning.

Use a bounded `Discrete(K)` candidate index per decision with a mask and candidate-feature tensor. Index 17 is a row in this decision's candidate table, not a globally stable spell. Persist the candidate representation and mask with each training sample. This supports candidate-scoring policies while keeping a standard discrete action interface.

Keep schema dimensions fixed within an environment version. Distinguish padding from illegal candidates. Zero or reject unused rows deterministically, and never silently clip state or actions. Document capacities and verify them against worst-case reachable positions in the frozen pool. A privileged full-state interface may exist for tests or an explicitly configured centralized critic, but must be separately named and inaccessible to the default policy observation path.

### Framework adapters

**Decision:** retain PettingZoo AEC as the portable multi-agent adapter, Gymnasium for fixed-opponent single-agent use, and an independent native batch API for maximum-throughput collectors. There is no single interface that is optimal for semantics, trainer coverage, and speed simultaneously. PettingZoo is an environment API, not a training algorithm or a native batching system. This review verifies documented integration paths, not a popularity ranking or measured performance winner.

| Interface | Role | Delivery expectation |
| --- | --- | --- |
| Native Python batch API | Efficient custom collectors and self-play | Required v0 fast path with native/Python equivalence tests |
| PettingZoo AEC | Standard sequential multi-agent environment | Required v0 adapter and API/seed tests |
| Gymnasium single-agent wrapper | One learning seat against a configured opponent | Required v0; advances opponent decisions internally |
| RLlib | Multi-agent training through `PettingZooEnv`; direct `MultiAgentEnv` adapter if evidence warrants it | Required masked training, evaluation, and checkpoint/reload check |
| TorchRL | TensorDict-oriented integration through `PettingZooWrapper` | Required masked collection and training check; optimized native `EnvBase` adapter deferred |
| SB3 / SB3-Contrib | Gymnasium fixed-opponent baseline, not direct multi-agent self-play | Required MaskablePPO training/evaluation check; not the throughput reference |
| CleanRL / custom PyTorch | Editable training script over Gymnasium or native batches | Document the collector/masking changes needed; no claim of drop-in multi-agent support |
| PufferLib | Candidate performance-oriented integration | Benchmark-led future work; not required for v0 compatibility |
| OpenSpiel | Game-tree, imperfect-information, and game-theoretic research | Defer a dedicated adapter until algorithms need its state/chance interfaces |

PettingZoo's [AEC API](https://pettingzoo.farama.org/api/aec/) matches sequential agent decisions. Its Parallel API represents simultaneous agent actions and is not the right abstraction for Magic priority. Parallelism here means many independent games, not pretending both players act simultaneously. Implement cumulative rewards, dead-agent handling, and masks according to the AEC contract, and run its [environment tests](https://pettingzoo.farama.org/content/environment_tests/).

RLlib's [documentation](https://docs.ray.io/en/latest/rllib/multi-agent-envs.html) recommends its native `MultiAgentEnv` for custom environments and also supplies `PettingZooEnv`. Start with the latter for reuse; build a direct adapter only if a pinned-version limitation or measured overhead justifies it. Neither choice guarantees that RLlib consumes our native batches efficiently. Request actions only from the current actor; deliver rewards and terminal signals to both seats. Exposing a mask is insufficient: the policy/module must mask logits.

TorchRL's [PettingZoo wrapper](https://docs.pytorch.org/rl/stable/reference/generated/torchrl.envs.PettingZooWrapper.html) requires `use_mask=True` for AEC. Its acting-agent `mask` is distinct from the legal `action_mask`; the training loop must use both correctly. Configure episode completion so that one inactive/dead agent does not incorrectly reset the whole game. This is a documented interoperability path, not evidence of native-batch speed.

[SB3's custom-environment interface](https://stable-baselines3.readthedocs.io/en/master/guide/custom_env.html) is Gymnasium-based; our adapter fixes an opponent and presents one learning seat. Use [SB3-Contrib MaskablePPO](https://sb3-contrib.readthedocs.io/en/master/modules/ppo_mask.html), its mask-aware evaluation utilities, and an environment `action_masks()` method for subprocess use. MaskablePPO currently does not support recurrent policies, so this baseline does not settle our longer-term partial-observability research needs. Generic environment checkers may sample illegal actions without masks: document that incompatibility and use a mask-aware test driver for stepping, while retaining strict rejection tests. Do not silently map an illegal action to pass just to satisfy a checker.

[CleanRL](https://docs.cleanrl.dev/) supplies editable single-file algorithms rather than an importable modular integration layer. An example needs collector, mask, and episode-semantics adaptation; “supports Gymnasium” does not establish arbitrary multi-agent support. [PufferLib](https://github.com/PufferAI/PufferLib) merits a later throughput experiment, but adopting another runner without checking sequential-decision semantics would not prove an improvement. [OpenSpiel](https://openspiel.readthedocs.io/en/latest/intro.html) is relevant to imperfect-information and sequential games, but its additional game-tree/chance contracts address a different need than a minimal modern deep-RL interface.

PettingZoo's [Parallel API](https://pettingzoo.farama.org/api/parallel/) is not a shortcut to faster Magic: parallel stepping has different semantics. Do not force an AEC-to-parallel conversion that changes when state updates or who observes what. Batch independent games below the adapter. For a policy shared by both seats, route observations into a common inference batch but retain per-game, per-seat trajectory and recurrent-state identities.

### What qualifies as verified framework compatibility

Before M3 is accepted, publish a compatibility matrix with exact Python, binding, trainer, and adapter versions, plus commands and CI artifacts. Do not label a path “verified” merely because documentation lists a wrapper. Each required training integration must:

1. Install from its own optional dependency group, without requiring the other trainers, a display, or CLI IPC.
2. Reset, collect at least 1,000 decisions across multiple episodes, perform at least one optimizer update with finite losses/gradients, evaluate with legal masking, and save/reload a checkpoint. A learning-curve improvement is not a smoke-test requirement.
3. Cover both learning seats, repeated decisions by the same actor, terminal reward delivery, explicit truncations/final observations, reproducible seeding, and no hidden-information leakage. Unit fixtures must force edge cases a short rollout may not naturally reach.
4. Assert zero invalid actions sampled by a masked policy; separately verify that deliberately invalid actions fail without state mutation. Retain masks/candidate features when recomputing training log probabilities, not just during sampling.
5. Measure collection overhead relative to the equivalent native encoded workload. Report framework batching/process settings explicitly; a Python wrapper is not assumed to expose the native fast path.

Keep separate optional extras such as `pettingzoo`, `rllib`, `torchrl`, and `sb3`; lock versions per tested environment and revisit on dependency upgrades. Start with feed-forward smoke models, but keep per-seat histories and recurrent-state routing possible for later partially observed agents.

### Episode, reward, and time semantics

Use sparse zero-sum terminal rewards: winner +1, loser −1, draw 0; intermediate reward 0. Concession is a game outcome, not a time-limit truncation. No hidden-information reward shaping in v0. Credit rewards by persistent seat, not by whoever happens to be the current actor.

External turn/decision/wall-time limits are truncations, not invented draws. Preserve the final observation and the termination/truncation distinction required for value bootstrapping; see [Gymnasium's time-limit guidance](https://gymnasium.farama.org/tutorials/gymnasium_basics/handling_time_limits/). Resource overflow or an engine error is a failed sample to quarantine, not an ordinary training truncation to bootstrap from corrupted state.

For the native API, use explicit reset of finished rows. Wrappers must document their autoreset behavior and preserve final observations/rewards; follow the relevant [Gymnasium vector API contract](https://gymnasium.farama.org/api/vector/). Never accidentally substitute the next episode's initial observation for a terminal one.

Choice continuations create multiple policy decisions for one logical action. Default to undiscounted episodic returns (`gamma = 1`) in the initial examples. If discounting is introduced, define whether it advances per policy decision, logical action, or turn; micro-choice decomposition must not silently change the research objective. Report both counts in datasets.

Fix the opponent policy/version for each episode. Record seat assignment, opponent ID, seeds, pool/schema/rules/engine versions, and truncation limits with trajectories. Supply deterministic random and simple heuristic opponents for smoke tests. Self-play scheduling and league management belong outside the core. Policy strength is not a correctness oracle.

### Trajectory capture is a first-class output

Trajectory capture is required for both CLI simulation and Python/RL collection, not a future logging feature. Use one canonical recorder contract across native, batched, and framework paths. The engine supplies decisions, observations, actions, rewards, and outcomes; the collector supplies policy metadata. The recorder must not require a particular trainer or calculate policy outputs inside the rules core.

Distinguish three artifacts:

- **Training trajectories:** exactly the player-visible inputs and chosen actions, with rewards and episode boundaries, suitable for online collection, offline analysis, or later training.
- **Deterministic replays:** initial configuration, private seeds/state where necessary, semantic actions, and checkpoints sufficient to reproduce engine execution. Link from trajectories using an opaque replay ID; do not insert private seeds into policy inputs.
- **Diagnostic traces:** sampled internal events and timing, which may be incomplete. These cannot substitute for either of the above.

Required schema, versioned independently of storage encoding:

| Record | Required contents |
| --- | --- |
| Dataset/run manifest | Dataset/schema versions; engine/rules/card/action/observation versions; deck/config hashes; policy/opponent versions; reward and discount conventions; capture selection policy; shard inventory and checksums |
| Episode header | Globally unique episode ID, run ID, seat/deck assignments, starting seat, policy IDs per seat, configured limits, and optional restricted replay reference |
| Decision/transition | Episode ID, monotonic decision index, acting seat, decision kind, logical-action/micro-choice ID, exact pre-action observation, candidate features and legal mask, selected candidate index and semantic action, resulting reward delta by seat, next actor, and terminal/truncation flags |
| Episode footer | Outcome/reason, per-seat return, decision/action counts, final authorized observations for both seats, completed/truncated/failed status, and recording completeness |
| Optional policy fields | Behavior-policy checkpoint/version, selected-action log probability, value estimate, recurrent-state reference/checkpoint, and exploration settings, when supplied by the collector |

Do not fabricate log probabilities for a human or heuristic policy: store them as absent and reject their use in algorithms requiring behavior probabilities. Retain the action-time mask and candidate features for policy-gradient recomputation. Snapshot/copy owned data before reusable batch buffers are overwritten; aliases into the next batch would silently corrupt the dataset.

Store a global decision stream and offer per-seat transition/sequence readers. In a turn-based game, the immediately following observation may belong to the opponent: **it is not the acting player's `next_observation`**. For a per-seat learning transition, link to that same seat's next decision or final observation and accumulate that seat's reward deltas over the intervening decisions. Deliver terminal rewards to both seats exactly once, including a seat that never acts again. Preserve the number of intervening decisions/logical actions and the chosen discount convention. Global step indices, per-seat indices, and episode IDs must survive batching, sharding, and resets.

Capture genuine terminal observations before autoreset. Truncations retain final observations and reasons; failed/corrupt episodes are quarantined. Closing a storage shard or a training rollout fragment is not an environment truncation: store fragment continuation IDs without changing game flags. Support recurrent sequence loading with explicit episode starts and padding/loss masks; record enough history or optional collector state to make any burn-in policy explicit.

Privacy is enforced at collection and loading. Default policy observations contain only that seat's permitted information. A research dataset containing both seats' separate observations still reveals both hands to an authorized dataset reader; it is not safe to publish during live play. Keep full-state/privileged-critic data and shuffle/RNG seeds in separately permissioned artifacts, opt-in and excluded from default feature loaders. Tests must prevent accidentally concatenating an opponent's private observations into a policy input.

For v0, provide JSONL for small inspectable fixtures and a sharded Parquet export for larger datasets, plus Python iterators/batch readers yielding numeric arrays. These are proposed output formats, not dependencies of `mtg-core`. Encode/compress/write in a recorder layer using bounded batch queues, not one filesystem operation per decision. Online training may consume batches without persistence; enabling a durable sink must not change game semantics.

Support capture of all episodes or a deterministic seed-based subset chosen at episode start. Selected episodes are recorded completely, including all policy decisions, rather than sampled individual steps. Record the selection rule so outcome-based capture does not masquerade as an unbiased dataset. Disabled capture should avoid materializing extra trajectory-only observations.

Never silently drop selected training transitions under backpressure. Default durable capture blocks the producer with measured queue wait; an explicitly configured fail-on-overflow mode aborts the recording/run clearly and marks affected episodes incomplete. Use finalized shards plus an atomically published manifest, retain explicit incomplete fragments after interruption, and validate checksums/counts on load. Disk-full and writer failures propagate to the CLI/collector. Readers reject incomplete episodes by default, with an explicit diagnostic opt-in.

Required automated acceptance tests:

1. Record, persist, reload, and compare observations, masks, semantic actions, rewards, and boundaries exactly against an in-memory reference trajectory; reconstruct the game through its linked replay.
2. Compare scalar and batched records after sorting by episode/decision ID; worker interleaving may change physical row order but not episode content.
3. Exercise same-seat consecutive decisions, opponent interleavings, a terminal reward for a seat not acting last, and termination versus external truncation. Verify per-seat returns and no reward duplication.
4. Exercise autoreset, buffer reuse, shard rollover mid-episode, fragment reassembly, process interruption, queue overflow, and disk/write failure without silently accepting incomplete data.
5. Check private-field isolation and show that capture on/off yields identical games under identical seeds/actions.
6. In each required RL integration, export a trajectory batch and reload it through the canonical reader. Benchmark capture disabled, in-memory, JSONL, and sharded export separately, reporting decisions/s, bytes/decision, writer throughput, queue wait, and memory high-water marks.

## 9. Automation-first CLI and optional human play

### Primary interface: unattended experiments

The CLI primarily launches reproducible simulation and verification jobs. Every noninteractive command works without a TTY, display, browser, stdin input, or human choosing an action. Both seats are supplied by seeded policies or explicit scripts. Human controls are never the fallback for a missing policy, an unresolved choice, or a reference-engine prompt: those conditions produce a structured error and nonzero exit status.

Illustrative command surface, not implemented yet:

```text
mtg simulate --episodes 10000 --decks red,green --policies heuristic,random --seed 42 --workers 8 --output run.json
mtg conformance --suite all --references xmage,forge --output conformance.json
mtg bench --workload foundations_micro_v1 --output benchmark.json
mtg replay verify failure.replay
mtg replay inspect failure.replay --seat 0 --format jsonl
mtg simulate --episodes 10000 --decks red,green --policies heuristic,random --seed 42 --trajectories runs/demo --trajectory-format parquet
mtg trajectories validate runs/demo
```

Provide validated configuration files for longer experiments. Record the resolved configuration, seeds, engine/rules/card/policy versions, outcome counts, resource limits, and instrumentation mode with results. Machine mode uses versioned JSON/JSONL on stdout or explicit output paths, diagnostics on stderr, and documented exit codes. No ANSI output or progress prompts when stdout is not a TTY. Missing dependencies, malformed input, unavailable references, and illegal scripted actions must fail explicitly; they must not hang awaiting confirmation.

Simulation runs take an explicit episode/work budget and optional truncation limits. Handle termination signals with a clean summary and clearly identify unfinished episodes. Native random and heuristic policies supply deterministic baselines. Learned policies and training loops normally use the Python batch API directly; a CLI may launch an experiment process, but high-throughput collection does not serialize every decision through text or spawn a subprocess per game/action.

For debugging and external-process agents, provide a persistent, versioned JSONL decision protocol with seat-filtered observations, decision IDs, legal choices, semantic action submissions, errors, and terminal results. This protocol is a convenience/integration surface, not the throughput reference. Specify how policies receive decisions requiring multiple choices and prohibit silent default decisions. Any privileged full-state test protocol is separate from policy-visible output.

### Optional interactive mode

Provide plain-text CLI play in two terminal sessions. A local session-owner process holds the authoritative engine and each terminal attaches to one seat over local IPC (Unix-domain sockets on the initial Linux/macOS targets). This is a small local coordinator, not a web application or remote multiplayer platform. It runs only for interactive sessions; training and ordinary agent-vs-agent simulations do not require it.

Show life, zones, the seated player's hand, battlefield, stack, active player/priority, legal choices, and a readable public history. Support help, inspection, action selection, passing, concession, and replay inspection. Start with numbered menus and line-oriented commands; a full-screen terminal UI is unnecessary. Reuse the structured protocol above for scripted debugging, separate from human formatting. Python training must never parse terminal text or shell out once per action.

Illustrative commands, not implemented yet:

```text
mtg play create --session demo --decks red,green
mtg play join --session demo --seat 0
mtg play join --session demo --seat 1
mtg replay inspect game.replay --seat 0
```

Seat handles and decision IDs scope commands; reject stale/wrong-seat input without mutation and handle duplicate requests idempotently. Send only the authorized player's view over each endpoint. Never transmit both hands and rely on terminal rendering to hide one. Keep private hand details out of shared stdout, logs, and command histories. Single-terminal hotseat may be a debug convenience, but clearing a screen does not protect scrollback and is not equivalent to separate player views.

Support detach/reattach while the owner remains alive, concession, and authorized replay export. A disconnect does not immediately become a rules loss. Full-information replays are restricted to explicitly privileged debugging or allowed post-game access. Local seat separation prevents accidental application-level information disclosure; it is not a security boundary against a malicious user who controls the host or shares unrestricted OS access. Remote authentication and transport security are outside v0.

Acceptance uses two scripted terminal clients to finish games automatically, exercising correct response windows, malformed/stale/wrong-seat commands, detach/reattach, EOF handling, and private-view redaction. CLI choices must map to the same engine actions as RL decisions; convenience defaults must not remove tactical options. Automated transcript tests cover interactive mode without a human or graphical display. Manual play is useful exploratory feedback, not a release dependency.

### Zero-human development and test contract

All CI, conformance, reference-engine runs, fuzzing, replay verification, benchmark jobs, and RL smoke experiments run unattended with bounded timeouts. Disable or replace upstream GUI/controller prompts in reference harnesses with strict scripted controllers. Capture logs and minimized failure artifacts automatically. Interactive failures may not block a worker indefinitely. Add subprocess tests that close stdin and unset display variables, asserting that every automation command still completes or fails clearly. If a chosen training/reference dependency cannot run headlessly, that integration is incomplete.

“No human required” means automated execution and actionable artifacts; it does not mean accepting arbitrary generated expected outcomes. Rules disagreements still need an evidence-backed adjudication recorded in the corpus, whether the development work is performed by a person or a coding agent.

## 10. Milestones and release gates

| Milestone | Deliverables | Exit criteria |
| --- | --- | --- |
| M0: Freeze scope and verifier | Card/rules manifests, capability matrix, fixture schema, provenance policy, initial fixtures, pinned reference-build/bridge spike | Deck counts validated; supported behavior mapped to tests; fixture expectations explained; both reference harnesses run a headless scripted smoke case; legal/data distribution decision recorded |
| M1: Test-driven scalar vertical slice | Reset, priority/stack, basic creatures, lands, Growth/Bite Down, replay, initial in-memory/JSONL trajectory recorder, strict runner, unattended CLI simulation | Red/green test evidence; scripted games complete without stdin; trajectory round-trip and reward/boundary tests pass; critical stack/cleanup/privacy cases pass locally and in applicable reference cases; invalid actions preserve state |
| M2: Complete pool and establish baseline | All 20 cards, keywords, costs/triggers, native policies, benchmark harness, counters, expanding reference coverage | Full scoped suite passes; initial ≥100 reviewed scenarios; designated mutants caught; XMage coverage for each card/capability; reproducible scalar performance/memory artifact |
| M3: Batch and Python | Worker pool, binding, tensors, sharded trajectory export/readers, PettingZoo/Gymnasium, RLlib/TorchRL/SB3-Contrib training examples | Serial/batch/replay/trajectory equality; recorder failure/privacy checks; adapter tests; required automated collection/update/evaluation/checkpoint and trajectory reload checks; pinned compatibility matrix; measured recording overhead/scaling |
| M4: CLI interaction completeness | Persistent JSONL protocol, optional interactive seat clients, scripted transcripts | All CLI modes tested without humans; stale-action/reattach/privacy checks pass; no training dependency on terminal rendering or IPC |
| M5: Torture and qualification | Fuzzing, mode matrix, mandatory XMage/Forge comparisons, profiles, release documentation | Required dual-reference critical suite and scripted full games executed; no unexplained scoped failures or hidden skips; performance targets met or revised with evidence; defects become regressions |

Each milestone must leave a runnable, documented system. Do not wait for the entire pool to exist before exposing correctness failures or measuring the shape of the workload. Conversely, a fast vertical slice is not the complete POC.

CI on every change: formatting/lints, unit tests, all scoped conformance fixtures, deterministic replay, basic batch equivalence, unattended CLI checks, and relevant adapter smoke checks. Rules-changing work also runs impacted cached reference scenarios. Nightly/dedicated jobs: longer fuzzing, mutation checks, expanded execution matrix, the mandatory dual-reference suite, training compatibility jobs, and performance/memory comparisons. Release qualification must run the required reference suite against the exact release candidate, even if a local fast run omitted it. Use dedicated hardware for performance regression gates; shared CI timing is diagnostic only. All jobs run without human input.

Suggested eventual commands such as `conformance --suite fast`, `conformance --suite all`, and `bench --workload foundations_micro_v1` are interface requirements, not commands available in this documentation-only repository today.

## 11. Alternatives and risks

| Alternative or risk | Decision / mitigation |
| --- | --- |
| Start from an existing engine | Rejected for this implementation goal; reuse knowledge and appropriately licensed tests, not the engine core |
| Implement all Standard cards | Rejected: large and moving validation surface before the research loop exists |
| Implement only vanilla creatures | Useful M1 slice, insufficient final POC: omits stack interaction, triggers, and meaningful instant timing |
| Python-only rules engine | Not selected given the throughput/memory priorities; easier prototyping acknowledged, performance rationale unmeasured until baselining; Python remains the experimentation interface |
| GPU-first engine | Deferred until profiles show CPU simulation is the bottleneck and branching/representation justify the investment |
| One process per game | Rejected for resident capacity; native game batching first |
| One enormous flat legal-action space | Rejected; typed continuations preserve choices without exponential enumeration |
| Single custom training API | Rejected as the only interface; retain fast native batching plus standard adapters |
| Tests share implementation mistakes | Independent expected states, rules references, mutation checks, mandatory mature-engine comparisons, and reviewed holdout scenarios |
| Legacy upstream tests encode old rules | Pin revisions and audit imported scenarios, especially combat |
| Simplified scope becomes approximate rules | Reject unsupported mechanics; preserve exact semantics within the declared capability matrix |
| Instrumentation dominates runtime | Worker-local counters, sampled timing, explicit modes, measured overhead |
| Hidden-information leakage yields misleading agents | Observation/mask/CLI/IPC invariance tests; separate privileged critic/debug API |
| Fixed decks encourage overfitting | Accept for engineering validation; do not infer general Magic playing strength from results |
| Reward/action encoding changes conclusions | Freeze schema and objective; record logical actions and micro-decisions separately |
| Fixed buffers silently discard legal play | Prove bounds or restructure choices; explicit failure and quarantine on overflow |

## 12. First implementation work package

Begin with M0 and the M1 vertical slice, not interactive UI work:

1. Create the Rust workspace and a minimal CI job, then pin the rules/card manifests and deck definitions.
2. Define the fixture schema and capability registry. Add original priority, target, cleanup, mulligan, and privacy scenarios; select applicable XMage candidates and retain provenance. Prove headless execution in both pinned reference harnesses with small bridges.
3. Implement and test the fixture runner/comparator. Ensure they detect seeded wrong outcomes. Write failing behavior tests before state, object identity, reset, and decision-continuation implementation.
4. Implement the land/creature/Growth/Bite Down slice test-first, snapshot/replay, and explicit player views; run matched reference cases. Add unattended agent-vs-agent CLI simulation with structured results.
5. Add a benchmark command and counters before broadening the card pool. Record a baseline, even if slow. Keep every test and development command noninteractive by default.

Success is a narrow, testable research platform optimized for unattended simulation and agent training, with CLI access and optional human play using the same engine. Coverage, speed, and observability should grow from that foundation together; none substitutes for the others.
