# RFC 0001: A trustworthy, high-throughput Magic research engine

Status: **Proposed project charter** — enduring purpose, principles, and product requirements; not a release plan or a statement of implemented capabilities.

**Current delivery scope (2026-10-09):** the [engine-validation plan](../programs/engine-validation.md)
prioritizes playable toy decks and Forge/XMage game replay. Learning-related
personas, use cases and integrations below are future context, not current
delivery requirements or authorization. No new RL work belongs in this scope.

Originally authored: 2026-09-26. Related: [open-source engine survey](../open-source-mtg-engines.md).

## Problem definition

Researchers need a **correct, high-throughput Magic: The Gathering simulation environment that produces trustworthy experience for training and evaluating agents**. It must run many independent games unattended, expose each player's legal choices and permitted information, and capture reproducible trajectories with useful instrumentation. The central challenge is to maximize useful simulations per CPU-second without sacrificing rules correctness, hidden-information boundaries, or data integrity.

We are building a from-scratch, headless research engine with CLI access and efficient programmatic integration into modern reinforcement-learning workflows. Human play is an optional way to interact with the same engine, not a dependency of simulation, development, testing, or evaluation.

## Purpose of this RFC

This document defines what the project exists to do and what must remain true as it evolves. It is the reference for resolving design tradeoffs: **does a change make the engine a more trustworthy and useful research instrument?**

It does not select a starter card pool, format, deck size, feature milestone, performance threshold, or delivery sequence. It neither promises universal Magic coverage nor makes a small fixed pool the project's identity. Supported capabilities must always be explicit, and their implementation must remain faithful to the declared rules.

Implementation languages, framework adapters, API schemas, storage formats, deployment choices, and release scope belong in separate design or delivery documents. Those choices must satisfy this charter; they must not redefine it implicitly. [RFC 0002: First MVP](0002-first-mvp.md) defines the first delivery boundary and carries forward the earlier prototype-specific material without making it a permanent project requirement.

## Contents

- [Problem definition](#problem-definition)
- [Personas](#personas)
- [Use cases](#use-cases)
- [Core principles](#core-principles)
- [Rules fidelity and evolving coverage](#rules-fidelity-and-evolving-coverage)
- [Correctness and independent verification](#correctness-and-independent-verification)
- [Performance and scale](#performance-and-scale)
- [RL interoperability](#rl-interoperability)
- [Trajectories and reproducibility](#trajectories-and-reproducibility)
- [Instrumentation and operability](#instrumentation-and-operability)
- [Architectural boundaries](#architectural-boundaries)
- [Non-goals](#non-goals)
- [What success means](#what-success-means)

## Personas

These are roles, not necessarily separate people. Researchers are the primary users. Contributors and automated development agents maintain the environment that makes research results credible. Interactive players are a secondary audience.

### P1. RL researcher — primary

**Goal:** investigate learning, exploration, memory, decision-making, and self-play in a partially observed game.

Works mainly in Python with an existing trainer. Needs efficient batched access, legal-action information, reliable rewards and episode boundaries, stable player identities, and optional trajectory persistence. Wants to change policies and experiment configuration without editing rules code or rebuilding an integration for every framework.

Success means collecting experience, updating a policy, evaluating it, and restoring a checkpoint unattended. The researcher can distinguish an algorithmic failure from broken environment semantics or hidden-information leakage. Collection speed matters only when the samples are valid.

### P2. Experiment owner / evaluation researcher — primary

**Goal:** compare policies and experimental settings under controlled, repeatable conditions.

Runs large batches through the CLI or scheduled jobs with explicit game configurations, policies, seeds, seat assignments, limits, and resource budgets. Needs structured outcomes, complete episode accounting, versioned configurations, and selected trajectories to investigate surprising results.

Success means another run can reproduce the setup and explain differences. Wins, draws, truncations, and failures remain separate. Conclusions are scoped to the evaluated capabilities and protocol rather than presented as general Magic playing strength.

### P3. Trajectory consumer / data researcher — primary

**Goal:** reuse recorded experience for analysis, dataset construction, imitation learning, or suitable offline-RL experiments without rerunning every simulation.

Needs versioned datasets, policy provenance, action-time observations and legal choices, per-player sequences, outcomes, and completeness checks. May use a different training stack from the producer. Needs to know which fields are absent, privileged, or unsuitable for a particular algorithm.

Success means loading data preserves temporal order, player perspective, and reward semantics. Incomplete or corrupt episodes are detectable, and opponent-private data cannot accidentally become policy features. A collection of transitions is not automatically suitable for every learning algorithm.

### P4. Rules implementer / automated development agent — enabling

**Goal:** implement or repair behavior with independent evidence that it is correct.

Uses rules references, focused fixtures, mature-engine comparisons, and the regression suite. Needs deterministic failures, strict scripted choices, concise diagnostics, and replay/state-diff artifacts. A coding agent must be able to execute the development workflow without someone playing games or dismissing prompts.

Success means demonstrating a failing behavior test, making a general implementation change, and passing independent checks without weakening expectations. Disagreements are adjudicated against the rules, not whichever output makes tests pass.

### P5. Performance / integration engineer — enabling

**Goal:** improve useful simulation and collection throughput without changing semantics or making integrations fragile.

Profiles rules execution, memory, batching, encoding, language boundaries, inference, and recording. Needs reproducible workloads, native/adapter equivalence checks, and independently configurable instrumentation and capture.

Success means a measured improvement with unchanged correctness and data integrity. Memory consumption and end-to-end collection costs accompany core throughput results. Moving work outside the measurement window is not an improvement.

### P6. CLI investigator / optional human player — secondary

**Goal:** inspect a game or decision and, when useful, manually exercise a legal alternative.

Needs readable state, stack and priority information, legal choices, authorized player views, and replay inspection. Does not need a graphical client, artwork, matchmaking, or a hosted game service.

Success means agent-accessible decisions can also be understood and exercised through the CLI. This is an exploration and debugging aid, not a prerequisite for testing or qualification; scripted clients can exercise the same path.

## Use cases

These workflows define durable user outcomes. Their specific commands, packages, and release availability are implementation concerns. All must be automatable, including any interaction path that also supports human input.

### UC1. Run a reproducible batch of games

- **Actors:** P2, P5; an unattended job.
- **Workflow:** select a supported game configuration, policies, seeds, episode budget, execution resources, and limits; run the players automatically; optionally capture selected episodes.
- **Outputs:** resolved experiment manifest, outcome accounting, performance metrics, failures, and trajectory/replay references.
- **Success:** no display or human input is required. Every started episode is accounted for. Identical seeded policies and compatible engine/configuration versions reproduce episode behavior independently of worker scheduling. Missing choices fail explicitly instead of falling back to a human.

### UC2. Train an agent using an existing RL framework

- **Actors:** P1; a programmatic collector/trainer.
- **Workflow:** connect through a documented integration, configure opponents or an external self-play workflow, collect decisions in batches, train, evaluate, and save/restore the policy.
- **Outputs:** learning batches, checkpoints, experiment metrics, and optional durable trajectories with policy provenance.
- **Success:** real learning updates run unattended. Repeated decisions by one player, opponent interleavings, legal choices, rewards, and episode boundaries retain their meaning. Training does not depend on parsing terminal output or running an interactive session.

### UC3. Compare policies fairly

- **Actors:** P1, P2.
- **Workflow:** freeze the game/rules configuration, opponents, limits, and evaluation protocol; compare policies across relevant seat assignments and matchups; keep evaluation evidence distinct from training and policy-selection data.
- **Outputs:** stratified outcome counts, episode-level results, uncertainty where reported, and trajectories for selected decisions.
- **Success:** versions and experimental conditions are explicit. Failed or truncated games do not become wins or disappear from the denominator. Shared seeds control declared randomness; they do not imply identical paths after policies make different decisions.

### UC4. Capture and reuse a trajectory dataset

- **Actors:** P1, P3.
- **Workflow:** select a documented capture policy, record experience, persist it, validate it, and load per-player sequences or learning batches in another process.
- **Outputs:** dataset metadata, decision records, outcomes, integrity checks, policy provenance, and optional restricted replay links.
- **Success:** round trips preserve observations, legal choices, actions, rewards, and boundaries. Player perspective is maintained across opponent turns. Missing policy statistics remain absent; incomplete episodes are rejected by default. Storage failures cannot silently lose selected samples.

### UC5. Add or fix behavior test-first

- **Actors:** P4, including an autonomous coding agent.
- **Workflow:** define a capability and its rules basis; write a failing behavior test; establish expected checkpoints independently; compare with mature engines where applicable; implement, refactor, and run regressions.
- **Outputs:** implementation, rules references, test provenance, failure/pass evidence, and differential artifacts.
- **Success:** consequential choices are scripted. Deleting assertions, recognizing fixture IDs, or quietly narrowing the advertised capability is not a valid fix. Adding capability expands explicit coverage rather than creating an undocumented special case.

### UC6. Reproduce and diagnose a disagreement

- **Actors:** P4, P6; automated minimization tooling.
- **Workflow:** replay a fixture, fuzz trace, or captured episode under compatible versions; locate the first divergence; minimize the case; compare with pinned mature-engine executions where applicable.
- **Outputs:** reproduction, semantic state diff, implicated rule, reference results, and an independently justified regression test.
- **Success:** reproduction does not require manual play or matching wall-clock timing. Version incompatibility is explicit. Upstream bugs and rules differences remain visible explanations, not erased test results.

### UC7. Optimize without changing the experiment

- **Actors:** P5.
- **Workflow:** benchmark a frozen workload, profile it, change execution or representation, and rerun correctness, data-integrity, and performance checks under comparable conditions.
- **Outputs:** before/after results, profiles, memory usage, recording overhead, and equivalence evidence.
- **Success:** work and costs are declared consistently. Speed does not come from removed choices, shortened games, hidden failures, or dropped records. Core simulation performance remains distinct from inference-inclusive collection.

### UC8. Inspect or interact through the CLI

- **Actors:** P6 or scripted clients used by P4.
- **Workflow:** inspect a replay or authorized live player view, examine legal choices, submit an action when interaction is enabled, and inspect resulting decisions and state changes.
- **Outputs:** readable or machine-readable views, action results, and optional trajectory/replay artifacts.
- **Success:** the CLI uses the same rules and legality checks as agents. Invalid, stale, or wrong-player commands do not mutate state. Private information remains authorized, and transcript tests require no person to play.

## Core principles

1. **Correctness is a constraint, not a speed knob.** Optimizations may change execution, not the meaning of supported play.
2. **Research comes first.** The primary product is reliable simulation and experience generation, not a consumer game client.
3. **Automation is the default.** Simulation, development checks, verification, benchmarking, and training run without human game decisions.
4. **One authoritative rules implementation.** Agents, CLI users, replay tools, and interactive clients exercise the same semantics.
5. **Player knowledge is part of correctness.** Legal-action representations, observations, logs, and datasets must respect information boundaries.
6. **Evidence accompanies claims.** Support, compatibility, reproducibility, and performance are demonstrated, not inferred from architecture or test counts.
7. **Trajectories are a product, not leftover logs.** Their completeness, provenance, and temporal meaning are first-class requirements.
8. **Coverage can grow without changing the contract.** Explicit limitations are acceptable; silent approximation is not.
9. **The research interface outlives individual tools.** A particular language, trainer, data format, or deployment must not become the definition of the project.

## Rules fidelity and evolving coverage

The engine implements Magic rules for explicitly declared capabilities. It is not a collection of deck-specific scripts and must not substitute approximate rules while claiming faithful simulation. Preserve priority, the stack, costs, targets, triggers, combat, state-based actions, and other semantics wherever the advertised capabilities require them.

The rules authority is the applicable version of the [official Magic rules](https://magic.wizards.com/en/rules), together with the applicable card definitions and rulings. Each experiment pins these inputs; a newer publication must not silently change an existing experiment. Support manifests identify rules/card revisions, configuration, implemented capabilities, and known limitations.

Do not hardwire a starter set, deck list, or release's missing mechanics into the permanent architecture. Conversely, this charter does not promise every card, format, or player configuration. Future coverage is a deliberate engineering decision with its own evidence and cost.

Reject unsupported content explicitly. Report missing capability separately from illegal play and from engine failure. Enforce legal choices at the engine boundary, with invalid or stale input leaving game state unchanged.

Action representations must preserve all supported legal decisions. Factor complex choices when necessary without inventing response windows, exposing provisional private choices, or removing strategic options. Representation limits must be declared; overflow is a diagnosed failure, not permission to truncate legal play silently.

## Correctness and independent verification

### Test-driven development

Write rule-referenced behavior tests before implementing or fixing behavior. Demonstrate failure for the intended reason, establish expected outcomes independently, implement the general rule, and refactor under regression protection. Optimizations face the same semantic checks as feature work.

Expected results must not simply be generated from the implementation under test. Changing an expectation requires an explicit rules-based explanation. Tests must cover positive and negative behavior, interactions, invalid input, and prior defects. Coverage maps to capabilities and rules, not just a total number of passing cases.

The verifier is a durable project asset. Use portable fixtures with explicit initial state, ordered randomness where needed, complete scripted choices, rules references, provenance, and assertions at meaningful checkpoints. Distinguish synthetic test positions from end-to-end evidence of reachable play.

### Mature engines as independent references

Verification against mature rules engines is a project requirement, not optional reassurance after implementation. XMage and Forge are reference targets; their [XMage scenario tests](https://github.com/magefree/mage/tree/master/Mage.Tests/src/test/java/org/mage/test/cards) and [Forge simulation tests](https://github.com/Card-Forge/forge/tree/master/forge-gui-desktop/src/test/java/forge/gamesimulationtests) are starting points for fixture research and execution bridges. Reference runs remain outside production simulation and training.

Align rule/card versions, initial conditions, consequential choices, and checkpoints before comparing. Identical seeds do not guarantee identical randomness across different engines. Compare semantic state and legal decisions where observable, not only winners or engine-specific IDs. State explicitly which fields and capabilities a bridge can actually verify.

Neither agreement nor majority vote replaces the specification. Preserve and investigate disagreements, minimize reproductions, and record adjudications. A known upstream defect or version mismatch requires an explicit explanation; an unavailable engine, unsupported scenario, or failed build cannot count as agreement. Claims of verification include executed coverage, gaps, and exceptions.

Combine original rules-derived scenarios, appropriately adapted upstream tests, and independent interaction cases. A test borrowed from an engine is not independent corroboration by that same engine. Retain provenance and required notices, review licenses, and distinguish authored-from-scratch code from adapted test material. A process boundary does not eliminate licensing obligations.

### A verifier that can catch its own blind spots

Use properties, fuzzing, metamorphic checks, mutation testing, differential runs, and deterministic replay alongside example tests. Deliberately seed faults to prove the harness and comparator notice wrong state, missing choices, leaked information, incorrect rewards, and broken boundaries.

Exercise equivalent scenarios across scalar/batched execution, bindings, worker schedules, snapshots, and recording modes. Minimize failures and preserve them as regression cases. Independently authored validation cases help detect overfitting to a known suite.

The relevant lesson from the [compiler-verifier experiment](https://www.anthropic.com/engineering/building-c-compiler) is to invest in a reliable verifier and actionable feedback before relying on autonomous implementation. Passing a large corpus is evidence of tested behavior, not proof of complete rules correctness.

All checks run unattended with bounded execution and useful artifacts. No human input is needed to execute verification; evidence-based judgment is still needed to resolve ambiguous specifications or disagreements. This charter does not mandate a particular coding-agent orchestration system.

## Performance and scale

The objective is to maximize **correct, useful experience generated per unit of compute and memory**. Completed games per CPU-second is a primary engine metric; end-to-end valid training samples per second matters to researchers. No universal speed claim is meaningful without a defined workload and resource budget.

| Dimension | What to measure | What it prevents us from confusing |
| --- | --- | --- |
| Simulation throughput | Correct completed games per elapsed second and per CPU-second, with resources stated | Parallel capacity versus compute efficiency |
| Decision throughput | Policy decisions and logical game actions per second | Faster execution versus merely changing action decomposition |
| Resident capacity | Live games per memory budget and total memory usage | Holding more games versus progressing them faster |
| Collection throughput | Valid samples reaching training, including encoding, inference, and transfer | Fast core microbenchmarks versus useful research throughput |
| Latency | Decision compute, queueing, and batch wait distributions | Processing cost versus scheduling delay |
| Recording cost | Throughput, memory, and storage overhead with capture enabled | Fast unrecorded runs versus sustainable data generation |

Many independent games must execute efficiently, with batching available at the policy interface. Concurrency should not require a heavyweight client or isolated runtime for each game. Separate semantic player decisions from internal scheduling, and prevent scheduling from changing randomness or outcomes under otherwise identical inputs.

Benchmarks pin workloads, policies, limits, versions, instrumentation, and hardware. Report repeated measurements and variability, not just best runs. Include reset and other declared work consistently; account for failed, truncated, and unfinished episodes. Changing the workload, horizon, or validity criteria is not an engine speedup.

Profile before making claims about bottlenecks or choosing complex optimizations. Compare semantically equivalent execution with and without encoding, inference, and capture. Quantitative budgets belong with a workload and an implementation decision, not in the project's permanent definition.

## RL interoperability

Provide efficient programmatic access from Python and integration paths into modern RL frameworks. Researchers should not need to reimplement Magic rules, parse CLI text, or replace their training system to use the engine.

Maintain a trainer-independent batched interface alongside standard environment adapters. Batch independent games without pretending that sequential in-game decisions are simultaneous. Players may act repeatedly and unpredictably within a turn; actor identity is explicit rather than inferred by alternation.

Observations, legal-action information, action identity, rewards, and episode boundaries are versioned contracts. Training-time recomputation must use the same action-time candidate representation and legality information. Keep per-game and per-player history/recurrent-state identity intact through batching and resets.

Termination, external truncation, and execution failure are distinct. Preserve genuine final observations before resetting an environment. Rewards belong to stable player identities, including players who do not act again before the game ends. Reward definitions, shaping, discounting, and the meaning of a learning step are explicit experiment choices; micro-choice encoding must not silently change the learning objective.

Supported integrations need executable evidence: collection, actual policy updates, mask-aware evaluation, checkpoint restoration, boundary handling, and trajectory export/load. Publish tested dependency versions and limitations. Documentation listing a wrapper is not proof that our environment works with it, and compatibility is not evidence of optimal throughput.

PettingZoo, Gymnasium, RLlib, TorchRL, and other frameworks are ecosystem integration choices, not the project's identity. Adapter selection and performance comparisons belong in technical design documents. Keep optional integrations isolated so one trainer does not become a dependency of every engine user.

Policy learning, opponent selection, and self-play orchestration use the engine; they do not determine its rules. An explicitly privileged critic or analysis interface must be separate from default player observations. Policy strength never serves as a rules-correctness oracle.

## Trajectories and reproducibility

### Distinct artifacts with distinct promises

| Artifact | Purpose | Required guarantee |
| --- | --- | --- |
| Training trajectory | Learn from and analyze player experience | Correct perspective, actions, rewards, timing, boundaries, and declared completeness |
| Deterministic replay | Reproduce game execution and diagnose behavior | Sufficient configuration, versions, randomness, and semantic actions for compatible reproduction |
| Diagnostic trace | Inspect implementation events and performance | Declared sampling/completeness; never presented as complete experience by default |

Trajectory capture is available from simulation and training workflows through a common semantic contract, independent of trainer or storage format. Online consumption and durable recording may coexist; enabling capture must not alter rules or action availability.

Datasets identify engine/rules/card/observation/action versions, experiment configuration, policies, seat assignments, selection policy, and integrity information. Decision records preserve actor, ordering, exact policy-visible input, legal choices, selected action, rewards, and episode status. Optional policy statistics are supplied by the collector, never fabricated when unavailable.

Support per-player sequences as well as global decision order. The immediately following decision may belong to an opponent and is not automatically a player's next learning observation. Aggregate intervening rewards correctly, preserve decision/logical-action timing, and credit terminal outcomes exactly once. Storage fragments are not game terminations; fragmented episodes retain continuity.

Never silently drop selected training transitions. Recording has explicit backpressure and failure behavior, bounded memory, and measured overhead. Interrupted or corrupt recordings are identifiable and excluded by default. Capture selection is declared so a biased selection of interesting games does not masquerade as an unbiased dataset.

A dataset containing separate observations from multiple players can collectively expose private information even when each observation was valid for its owner. Protect access accordingly and prevent loaders from combining unauthorized information into policy features. Privileged state, RNG seeds, and full-information replays remain distinct from player-facing data.

### Reproducibility as a contract

Seeds alone are insufficient. Record random-stream versions, rules/card definitions, configuration, policies where relevant, and semantic actions. Keep environment randomness separate from policy and scheduler randomness. Deterministic simulation must not depend on wall-clock timing or worker interleaving.

Replay and snapshot compatibility is explicit. Either reproduce under compatible versions, perform a declared migration, or reject the artifact clearly. Do not promise that arbitrary engine upgrades or nondeterministic policy inference produce identical results.

Test recording and replay through round trips, scalar/batch equivalence, privacy checks, reset boundaries, buffer reuse, interruptions, and storage failures. Data integrity is part of engine quality, not solely a downstream training concern.

## Instrumentation and operability

Make experiments observable without making expensive tracing mandatory. Separate lightweight metrics, detailed diagnostics, trajectory capture, and replay recording; each has an explicit cost and completeness contract.

Measure outcomes, decisions, failures, resource use, worker utilization, queueing, encoding, inference, and recording separately. Aggregate metrics should remain bounded in cardinality and avoid private game content. Detailed artifacts carry versioned identifiers and appropriate access restrictions.

CLI jobs are noninteractive by default, work without a display, expose structured results and clear exit status, and handle interruptions predictably. Missing policies, unresolved choices, unavailable references, and invalid configurations produce actionable failures instead of indefinite prompts. Machine-readable interfaces are distinct from human formatting.

Logs and failure reports identify the episode, relevant versions, first divergent checkpoint, and reproduction artifacts without flooding routine output. Failures are visible in experiment accounting; diagnostic sampling cannot conceal broken games or incomplete data.

Interactive inspection is optional and uses authorized views and the same validated actions as programmatic clients. Clearing a terminal is not a privacy boundary. Tests must exercise interactive paths automatically; no research or development workflow depends on human play.

## Architectural boundaries

These are responsibilities, not a prescribed package layout or technology stack:

- **Rules and state:** authoritative game semantics, legal decisions, player views, and reproducible state evolution. Independent of terminal rendering, trainers, storage services, and telemetry exporters.
- **Execution and batching:** efficient scheduling of independent games and collection of ready decisions, without alternative rules or changed outcomes.
- **Research integration:** programmatic interfaces, environment adapters, and policy interaction. No duplicated rules and no trainer-specific assumptions in the core.
- **Evidence and data:** conformance, reference comparisons, replays, trajectories, and instrumentation with explicit provenance and information boundaries.
- **Interaction:** CLI automation and optional human inspection/play as clients of the same engine, not required simulation infrastructure.

Build the rules engine from scratch while reusing justified infrastructure and appropriately licensed test knowledge. This does not mean avoiding libraries, nor does it permit treating another engine as the hidden production implementation.

Choose languages and representations for measured performance, correctness, maintainability, and research ergonomics. Rust for a native core and Python for research is one candidate architecture, not an immutable requirement of this charter or a measured speed result. Preserve efficient Python use regardless of the eventual implementation boundary. Language and binding decisions require their own rationale and evidence.

## Non-goals

- A consumer Magic client, browser frontend, artwork pipeline, public matchmaking system, or hosted tournament service.
- A new general-purpose RL training framework or mandatory experiment-hosting platform.
- A coding-agent orchestration product; automated contributors are users of the development workflow, not the engine's purpose.
- Approximate Magic presented as exact simulation, or a speed benchmark that removes meaningful decisions.
- Universal card/format support implied by the project name, or performance claims without comparable evidence.
- A perpetual restriction to any starter pool, set, deck size, prototype, or release plan.

These boundaries keep the project focused on a research engine. They do not prevent separate clients, trainers, or orchestration systems from using it.

## What success means

The project succeeds when a researcher can reliably **simulate, collect, learn, evaluate, and explain**:

- Run supported games at useful scale without a human or graphical client.
- Trust the declared rules coverage because independent tests and mature-engine comparisons support it and limitations are visible.
- Train and evaluate through familiar tooling without sacrificing player perspective or episode semantics.
- Capture usable trajectories and reproduce important behaviors with explicit version and provenance contracts.
- Understand where compute, memory, and storage are spent, and improve throughput without corrupting results.
- Expand capabilities without maintaining different rules for agents, humans, tests, and benchmarks.

Future implementation and delivery RFCs must specify their own scope, dependencies, measurable acceptance criteria, and tradeoffs under this charter. No milestone, benchmark, or passing test count substitutes for these enduring commitments.
