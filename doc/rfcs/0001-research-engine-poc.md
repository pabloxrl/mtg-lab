# RFC 0001: A small, correct, high-throughput Magic research engine

Status: **Proposed** — implementation scope and acceptance criteria, not implemented functionality or measured performance.

Date: 2026-09-26. Related: [open-source engine survey](../open-source-mtg-engines.md).

## Contents

- [Decision summary](#1-decision-summary)
- [Objectives and metrics](#2-what-smallest-and-fastest-mean)
- [Frozen decks and rules scope](#3-concrete-game-scope)
- [Architecture](#4-architecture-and-ownership-boundaries)
- [Performance budget](#5-throughput-plan-and-performance-budget)
- [Instrumentation](#6-instrumentation-without-making-the-hot-path-expensive)
- [Test sources and torture suite](#7-the-torture-suite-strategy)
- [RL integration](#8-rl-integration-contract)
- [Human PvP](#9-minimal-human-pvp)
- [Milestones](#10-milestones-and-release-gates)
- [Alternatives and risks](#11-alternatives-and-risks)
- [First implementation work package](#12-first-implementation-work-package)

## 1. Decision summary

Build a from-scratch, headless Rust engine for a deliberately small two-player Magic card pool. Human PvP and agent training must use the same rules implementation. Start with two fixed 40-card decks drawn from Foundations printings, not Standard, not the whole Foundations set, and not a claim to support an official sanctioned format.

Optimize **correct completed simulations per CPU-second**, while exposing batched decisions for efficient policy inference. Treat memory footprint, Python integration overhead, and human-facing latency as separate constraints. Do not optimize by deleting priority windows, hiding legal choices, or treating unfinished games as completed games.

Build an independent conformance harness before expanding the engine. Seed it with original rules-derived cases and carefully adapted XMage tests. Use other engines as fallible differential references, not the specification. Every supported capability needs positive, negative, interaction, and regression tests.

Deliver native batching and a Python binding, a PettingZoo AEC adapter, a fixed-opponent Gymnasium adapter, and a minimal two-browser PvP application. Validate RLlib integration with a runnable example. Keep expensive tracing and networking outside the simulation hot path.

The proposed sequence is: **scope and verifier → correct scalar engine → measured native batching → RL adapters → PvP → optimization and release qualification**. A small PvP debug client may be developed earlier, but must not dictate the core architecture.

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

No GPU rules simulator, distributed game-state service, commercial-quality client, matchmaking, account platform, or engine-specific learning algorithm in v0. No claim that passing this corpus proves general Magic correctness.

## 4. Architecture and ownership boundaries

```text
                   frozen card manifest + conformance corpus
                                      |
                         deterministic Rust rules core
                           /                     \
              native batched runner       authoritative PvP server
                       |                          |
                 Python binding             two browser clients
                  /          \
       PettingZoo AEC       fixed-opponent Gymnasium
          |     |                    |
       RLlib  TorchRL          single-agent trainers
```

Proposed workspace boundaries:

| Component | Owns | Must not own |
| --- | --- | --- |
| `mtg-core` | Rules, state, decisions, legal actions, player views, deterministic RNG state | Python, sockets, wall-clock time, telemetry exporters, model inference |
| `mtg-cards` | Frozen definitions and small typed behavior/effect descriptions | Runtime HTTP fetches or general scripting VM |
| `mtg-batch` | Independent game slots, worker scheduling, batch encode/advance/reset | Alternative rules implementation |
| `mtg-python` | Safe array ownership, native binding, adapters | Per-card Python callbacks |
| `mtg-server` | Seat authentication, authoritative commands, reconnect, redacted events | Client-trusted game state or required simulation infrastructure |
| `mtg-conformance` | Fixture loading, assertions, provenance, differential tooling | Expected results calculated by the implementation under test |
| `mtg-bench` | Frozen workloads, runner metadata, reproducible results | A separate simplified “benchmark rules” engine |

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
| Human PvP | p95 command-to-update server processing under 50 ms at a declared load | Excludes Internet latency; no throughput claim inferred from this |

There is deliberately no invented completed-games/s target. Once the workload is measured, report that rate directly alongside mean decisions/game and outcome fractions. For illustration only, 100,000 decisions/s and 500 decisions/completed game would imply 200 games/s before other overhead; neither input is established here.

Missing these hypotheses triggers profiling and a documented budget revision, not weakened rules or hidden benchmark changes. The first correctness milestone is not blocked by an unvalidated speed target. A later performance claim must show its measurement artifact.

### Required benchmark tracks

1. **Core transition:** fixed, validated traces exercising priority, casting, triggers, combat, and cleanup. Report both replay/application and legal-action generation costs.
2. **Native rollout:** complete seeded games with a deterministic heuristic policy and separately a legal-random policy. Include reset/shuffle and all choices. Report policy time rather than concealing it.
3. **Encoded collection:** the same workload plus player observation and candidate-feature construction. Compare native and Python batch paths with equivalent work.
4. **Inference-inclusive collection:** a fixed model, device, precision, batch policy, and opponent schedule. Separate engine, encoding, inference, transfer, and queue time.
5. **Capacity and stress:** resident-state memory curve, token-heavy positions, many legal targets, large stacks/trigger queues, and unequal game lengths.
6. **PvP server:** concurrent rooms and command latency, measured separately from headless simulation.

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
| Forge | [`forge-gui-desktop/src/test/java/forge/gamesimulationtests`](https://github.com/Card-Forge/forge/tree/master/forge-gui-desktop/src/test/java/forge/gamesimulationtests) | Secondary scenario research and optional external differential execution | Different harness and licensing; not an automatically importable permissive corpus |
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

For differential testing, first align rules revision, card definitions, initial library order, all choices, and the exact checkpoint. Compare semantic state, not engine-specific object IDs or UI events. A disagreement with XMage or Forge is a triage item, not automatic proof our engine is wrong. Resolve it against the pinned rules and add a reviewed expected result. Keep differential tooling optional for normal CI because upstream builds are heavy and may disagree on rules versions.

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

| Interface | Role | Delivery expectation |
| --- | --- | --- |
| Native Python batch API | Efficient custom collectors and self-play | Required v0 fast path with native/Python equivalence tests |
| PettingZoo AEC | Standard sequential multi-agent environment | Required v0 adapter and API/seed tests |
| Gymnasium single-agent wrapper | One learning seat against a configured opponent | Required v0; advances opponent decisions internally |
| RLlib | Multi-agent collection/training example through PettingZoo or `MultiAgentEnv` | Required runnable smoke experiment with explicit action masking |
| TorchRL | TensorDict-oriented integration through its PettingZoo wrapper or a later native adapter | Document and smoke-test compatibility; optimized adapter deferred |
| SB3 / SB3-Contrib | Fixed-opponent single-agent baseline, not direct multi-agent self-play | Document masked baseline; not the throughput reference |

PettingZoo's [AEC API](https://pettingzoo.farama.org/api/aec/) matches sequential agent decisions. Its Parallel API represents simultaneous agent actions and is not the right abstraction for Magic priority. Parallelism here means many independent games, not pretending both players act simultaneously. Implement cumulative rewards, dead-agent handling, and masks according to the AEC contract, and run its [environment tests](https://pettingzoo.farama.org/content/environment_tests/).

RLlib provides [multi-agent environment APIs and PettingZoo integration](https://docs.ray.io/en/latest/rllib/multi-agent-envs.html). The example must only request actions from the currently acting player and correctly deliver rewards/termination to both seats. Exposing a mask is insufficient: the policy/module must actually mask logits. Pin tested dependency versions rather than claiming compatibility with every release.

TorchRL exposes a [PettingZoo wrapper](https://docs.pytorch.org/rl/stable/reference/generated/torchrl.envs.PettingZooWrapper.html). Treat this as a compatibility path; measure its collection overhead before promising native-batch throughput. [SB3-Contrib MaskablePPO](https://sb3-contrib.readthedocs.io/en/master/modules/ppo_mask.html) is a possible masked single-agent baseline; the opponent wrapper must expose masks in the form that the collector uses. Generic checkers or collectors that sample actions without respecting masks require explicit handling, not weakened core legality.

### Episode, reward, and time semantics

Use sparse zero-sum terminal rewards: winner +1, loser −1, draw 0; intermediate reward 0. Concession is a game outcome, not a time-limit truncation. No hidden-information reward shaping in v0. Credit rewards by persistent seat, not by whoever happens to be the current actor.

External turn/decision/wall-time limits are truncations, not invented draws. Preserve the final observation and the termination/truncation distinction required for value bootstrapping; see [Gymnasium's time-limit guidance](https://gymnasium.farama.org/tutorials/gymnasium_basics/handling_time_limits/). Resource overflow or an engine error is a failed sample to quarantine, not an ordinary training truncation to bootstrap from corrupted state.

For the native API, use explicit reset of finished rows. Wrappers must document their autoreset behavior and preserve final observations/rewards; follow the relevant [Gymnasium vector API contract](https://gymnasium.farama.org/api/vector/). Never accidentally substitute the next episode's initial observation for a terminal one.

Choice continuations create multiple policy decisions for one logical action. Default to undiscounted episodic returns (`gamma = 1`) in the initial examples. If discounting is introduced, define whether it advances per policy decision, logical action, or turn; micro-choice decomposition must not silently change the research objective. Report both counts in datasets.

Fix the opponent policy/version for each episode. Record seat assignment, opponent ID, seeds, pool/schema/rules/engine versions, and truncation limits with trajectories. Supply deterministic random and simple heuristic opponents for smoke tests. Self-play scheduling and league management belong outside the core. Policy strength is not a correctness oracle.

## 9. Minimal human PvP

Provide a lightweight browser client with two independently seated connections and an authoritative server. Show life, zones, hand, battlefield, stack, whose decision it is, legal actions, and a readable public event history. Text-first cards are sufficient; artwork and polished animation are out of scope.

Commands contain room/seat credentials, expected decision ID, and action payload. Validate server-side. Reject stale commands without changing state; handle duplicate transport requests idempotently. Never send a complete state object and rely on the browser to hide private fields. Each client receives only its authorized view.

Support room creation/join, reconnect, concession, and a downloadable authorized replay. Full-information replay is restricted to debugging or explicitly allowed post-game access; it must not leak a live opponent's hand. A disconnect is not immediately a rules loss. Any timeout policy is server configuration, distinct from deterministic headless rules.

PvP acceptance requires complete games in two browser sessions, correct response windows, invalid/stale-command tests, a disconnect/reconnect test, and automated checks that each player's network payload omits the other's private data. UI convenience must not remove valid tactical choices.

## 10. Milestones and release gates

| Milestone | Deliverables | Exit criteria |
| --- | --- | --- |
| M0: Freeze scope and verifier | Card/rules manifests, capability matrix, fixture schema, provenance policy, initial original and adapted scenarios | Deck counts validated; every supported behavior mapped to tests; fixture expectations independently explained; legal/data distribution decision recorded |
| M1: Correct scalar vertical slice | Reset, priority/stack, basic creatures, lands, Growth/Bite Down, replay, strict fixture runner | Complete small games; critical stack/cleanup/privacy cases pass; invalid actions preserve state; no performance claim required |
| M2: Complete pool and establish baseline | All 20 cards, keywords, costs/triggers, native policies, benchmark harness, counters | Full scoped suite passes; initial ≥100 reviewed scenarios; designated mutants caught; reproducible scalar performance/memory artifact |
| M3: Batch and Python | Worker pool, batched binding, versioned tensors, PettingZoo/Gymnasium, RLlib smoke example | Serial/batch/replay equality; adapter/API tests; end-to-end collection with masked actions; measured binding/encoding overhead and worker scaling |
| M4: PvP | Authoritative server and text-first browser client | Two independent human seats can finish games; reconnect/stale-action/privacy checks pass |
| M5: Torture and qualification | Fuzzing, mode matrix, optional reference-engine comparisons, profiles, release documentation | No unexplained scoped failures; no hidden skips; performance targets met or explicitly revised with evidence; all defects found in qualification become regression tests |

Each milestone must leave a runnable, documented system. Do not wait for the entire pool to exist before exposing correctness failures or measuring the shape of the workload. Conversely, a fast vertical slice is not the complete POC.

CI on every change: formatting/lints, unit tests, all scoped conformance fixtures, deterministic replay, basic batch equivalence, and relevant adapter smoke checks. Nightly/dedicated jobs: longer fuzzing, mutation checks, expanded execution matrix, differential checks when configured, and performance/memory comparisons. Use dedicated hardware for performance regression gates; shared CI timing is diagnostic only.

Suggested eventual commands such as `conformance --suite fast`, `conformance --suite all`, and `bench --workload foundations_micro_v1` are interface requirements, not commands available in this documentation-only repository today.

## 11. Alternatives and risks

| Alternative or risk | Decision / mitigation |
| --- | --- |
| Start from an existing engine | Rejected for this implementation goal; reuse knowledge and appropriately licensed tests, not the engine core |
| Implement all Standard cards | Rejected: large and moving validation surface before the research loop exists |
| Implement only vanilla creatures | Useful M1 slice, insufficient final POC: omits stack interaction, triggers, and meaningful instant timing |
| Python-only rules engine | Rejected for the throughput target; Python remains the experimentation interface |
| GPU-first engine | Deferred until profiles show CPU simulation is the bottleneck and branching/representation justify the investment |
| One process per game | Rejected for resident capacity; native game batching first |
| One enormous flat legal-action space | Rejected; typed continuations preserve choices without exponential enumeration |
| Single custom training API | Rejected as the only interface; retain fast native batching plus standard adapters |
| Tests share implementation mistakes | Independent expected states, rules references, mutation checks, selective differential comparisons, and reviewed holdout scenarios |
| Legacy upstream tests encode old rules | Pin revisions and audit imported scenarios, especially combat |
| Simplified scope becomes approximate rules | Reject unsupported mechanics; preserve exact semantics within the declared capability matrix |
| Instrumentation dominates runtime | Worker-local counters, sampled timing, explicit modes, measured overhead |
| Hidden-information leakage yields misleading agents | Observation/mask/network invariance tests; separate privileged critic/debug API |
| Fixed decks encourage overfitting | Accept for engineering validation; do not infer general Magic playing strength from results |
| Reward/action encoding changes conclusions | Freeze schema and objective; record logical actions and micro-decisions separately |
| Fixed buffers silently discard legal play | Prove bounds or restructure choices; explicit failure and quarantine on overflow |

## 12. First implementation work package

Begin with M0 and the M1 vertical slice, not the frontend:

1. Create the Rust workspace and a minimal CI job, then pin the rules/card manifests and deck definitions.
2. Define the fixture schema and capability registry. Add original priority, target, cleanup, mulligan, and privacy scenarios; select applicable XMage candidates and retain provenance.
3. Implement state, object identity, deterministic reset, decision continuations, and the fixture runner. Ensure the runner demonstrably detects seeded wrong outcomes.
4. Implement the small land/creature/Growth/Bite Down slice, snapshot/replay, and explicit player views.
5. Add a benchmark executable and counters before broadening the card pool. Record a baseline, even if slow.

Success is a narrow, testable research platform with one rules engine serving both humans and agents. Coverage, speed, and observability should grow from that foundation together; none substitutes for the others.
