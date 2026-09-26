# Open-source Magic: The Gathering engines for simulation and agent research

Research date: **2026-09-26**. Intended use: deciding what to reuse, study, or build for `mtg-lab`.

## Contents

- [Findings and recommendation](#1-findings-and-recommendation)
- [Scope and evidence](#2-scope-discovery-and-evidence)
- [Evaluation dimensions](#3-evaluation-dimensions)
- [Comparison tables](#4-principal-engine-comparison)
- [Detailed engine assessments](#5-detailed-assessment-of-the-leading-options)
- [Training systems and benchmarks](#6-training-systems-and-research-benchmarks)
- [Historical engines and experimental projects](#7-historical-engines-and-the-experimental-long-tail)
- [Related tools and exclusions](#8-related-software-that-is-not-a-substitute)
- [Performance evidence and benchmark plan](#9-what-the-available-performance-evidence-actually-says)
- [Design implications for mtg-lab](#10-implications-for-a-lean-mtg-lab-engine)
- [Build versus reuse](#11-build-versus-reuse-decision)
- [Revisions and source trail](#12-revisions-and-source-trail)

## 1. Findings and recommendation

The surveyed evidence does not establish a winner that simultaneously offers broad, trustworthy Magic coverage, a small native core, a complete agent interface, and independently measured high simulation throughput. The strongest choices solve different parts of that problem.

| Goal | First candidates to evaluate | Why | Main unresolved issue |
|---|---|---|---|
| Simulate realistic decks with extensive existing rules support | **Forge, XMage** | Large established implementations; existing AI, cards, and tests | Cost of adapting player decisions, observations, and state branching for training |
| Start training against an established engine | **MageZero + its XMage fork** | Existing self-play, search, feature encoding, and Python training pipeline | Deck-local abstractions, imperfect-information handling, and throughput |
| Use a documented research API with a separate rules library | **Argentum** | Explicit stepping, legal actions, masked observations, branching, batching, and trainer modules | Supported card pool, structured choices, allocation and transport costs |
| Extend a native engine | **Phase; Manabrew if Forge compatibility is desired** | Rust cores; distinct approaches to state and card implementation | Actual behavioral coverage and missing training adapters |
| Study a compact research core | **managym, tinymtg, Arcana** | Directly relevant design ideas and accessible interfaces | Substantial limitations and, for these three, no verified top-level reuse license |
| Study search and established AI design | **Magarena, Magicgrove** | Implemented game-tree search, card evaluation, and simulation | Older rules/card support and modernization effort |

**Recommendation for this repository:** first define a bounded, two-player research card pool and a measurable simulation contract. Use Forge and XMage as independent behavioral references. Evaluate Argentum and Phase before committing to a new core; evaluate MageZero if the priority is producing agent experiments quickly. A new Rust or C++ core becomes justified if measured integration or throughput limitations prevent those options from meeting the research objective.

This is a recommendation about engineering effort and research validity, **not a claim that Rust is faster than Java for these implementations**. No cross-engine benchmark was run for this report.

## 2. Scope, discovery, and evidence

### What counts as an engine

A rules engine must advance game state and implement at least some legal actions and card interactions. A card database, deck editor, manual tabletop, draw-probability calculator, or language model answering rules questions is not sufficient for unattended self-play.

The survey covers established engines, current native implementations, training wrappers, functional-language experiments, and historical or smaller prototypes. It also records public-source projects with unclear licenses and related tools that are often mistaken for rules engines. Those categories are deliberately separated: public source availability is not evidence of an open-source license.

### How the survey was conducted

Discovery used general web searches, GitHub repository searches for Magic rules engines and reinforcement learning, and historical ecosystem lists. Findings were checked against upstream READMEs, module documentation, selected implementation files, license files, repository metadata, and a relevant research paper. Examples of search terms included `Magic gathering rules engine simulator reinforcement learning`, `magic gathering engine`, `managym`, `Incantus`, `Firemox`, and `magicgrove`.

The historical [Slightly Magic engine list](https://slightlymagic.net/wiki/List_of_MTG_Engines) was used for discovery only. Its old card counts and release dates were not treated as current measurements. GitHub API requests occasionally returned HTTP 403; affected fields are left unknown rather than inferred from search rankings.

**Coverage limit:** this is a comprehensive survey of discoverable, relevant projects, not proof that every experimental repository or private/unindexed fork has been found. Small repositories are listed with limited evidence rather than given invented capability scores. Forks and frontends are grouped by their underlying engine when they do not supply independent rules implementations.

### Evidence conventions

- **Source-observed:** a specific implementation or license file was inspected. This establishes what the code contains, not that it executes correctly.
- **Upstream-reported:** a README, project audit, or paper describes a capability, count, or result. It has not been independently reproduced here.
- **Assessment:** an engineering inference or recommendation based on those sources.
- **Unknown:** insufficient evidence. This does not mean unsupported.

No engines were installed, built, benchmarked, or subjected to conformance testing. All test counts, coverage figures, and performance figures below are upstream claims unless explicitly stated otherwise. Source links generally follow default branches; section 12 records observed revisions for the principal candidates.

## 3. Evaluation dimensions

For research, graphical quality and the number of downloadable card images matter much less than these properties:

| Dimension | What to evaluate | Why it matters |
|---|---|---|
| Rules fidelity | Priority, stack, choices during resolution, state-based actions, triggers, replacement effects, layers and dependencies | Agents can exploit incorrect transitions |
| Executable card coverage | Exact behavior for every card in the experimental pool, including interactions | Parsed metadata or generated files are not working cards |
| Headless execution | Core usable without rendering, human input, or unnecessary services | Simplifies large-scale workers and measurement |
| Decision interface | Acting player, legal choices, targets, costs, modes, combat assignments | A policy needs more than a list of castable spells |
| Information boundaries | Player observations, revealed information, face-down objects, history | Prevents policies and search from seeing hidden state |
| Reproducibility | Explicit randomness, stable action ordering, replay, versioned data | Enables paired experiments and bug reproduction |
| Branching and restoration | Clone/fork, undo, snapshots, random-state isolation | Often dominates MCTS cost |
| Throughput | Decisions and games per second under a specified workload | Determines training cost |
| Memory and scaling | Bytes per live game, allocations per step, worker scaling | Limits parallel self-play and tree size |
| Language integration | In-process Python/native calls, JVM adapters, batched IPC | Crossing the training boundary can dominate execution |
| Baseline opponents | Random, heuristic, search, trained models | Needed to detect both progress and regressions |
| Extensibility | Card DSL, typed effects, parser, custom cards, test fixtures | Determines cost of expanding the benchmark |
| Maintenance | Recent substantive work, buildability, test discipline | Affects long-term research reproducibility |
| Reuse terms | Engine, card scripts, third-party code, assets | Determines what can be incorporated and redistributed |

An engine can be headless without providing a resumable `step` interface. Likewise, a snapshot useful inside one process may not be serializable for distributed checkpointing.

## 4. Principal engine comparison

### 4.1 Rules, architecture, and cards

Counts in this table describe what upstream says they count. They are not normalized or directly comparable.

| Project | Core/runtime | Rules and card scope | Representation / extension model | Research assessment |
|---|---|---|---|---|
| [Forge](https://github.com/Card-Forge/forge) | Java; separate game and AI modules | Broad real-card and format support; Commander, constructed and limited play | Shared engine plus a large card-script corpus | Strong practical reference and baseline simulator |
| [XMage](https://github.com/magefree/mage) | Java; engine, server, cards, AI plugins, tests | Upstream reports 32,000+ unique cards; duels and multiplayer | Java card classes composed from common abilities/effects | Strong broad-coverage reference; usable base for custom agents |
| [Magarena](https://github.com/magarena/magarena) | Java with card scripting | Established AI-oriented dueling implementation; current full-pool coverage not verified | Shared rules with card definitions/scripts; implemented MCTS | Useful search reference; modernization likely required |
| [Wagic](https://github.com/WagicProject/wagic) | C++; JGE/platform application stack | Large historical card-game implementation; modern coverage not audited | Custom card/rule definitions and C++ abilities | Native source does not by itself make it a lean research library |
| [Argentum](https://github.com/wingedsheep/argentum-engine) | Kotlin/JVM; standalone rules library | Explicit stack, layers, replacement effects and multiplayer support; set-oriented implementation tracking | Immutable state and Kotlin card DSL | Particularly useful documented environment contract |
| [Phase](https://github.com/phase-rs/phase) | Rust; native and WASM | Broad rules ambition; 34,300+ parsed cards, with thousands explicitly still unimplemented | Pure reducers, persistent state, parser and card database | Promising native base; executable coverage must be measured |
| [Manabrew](https://github.com/witchesofthehill/manabrew) | Rust port plus Java Forge reference/backend | Rust works for selected matchups; broad parity still in progress | Forge scripts, typed intermediate representations, parity harness | Attractive when preserving Forge semantics is the objective |
| [Arcana](https://github.com/levineuwirth/arcana) | Rust core, AI utilities, PyO3 | Reports 20,590 registered generated cards; audit identifies substantial approximation and engine gaps | Effect primitives, generated Rust cards, deterministic core | Relevant research design, but not a verified general-purpose training substrate |
| [managym](https://github.com/jacklionheart/managym) | C++ with pybind11 | Explicitly focused on basic lands and simple creatures | Agent → flow → state → infrastructure modules | Small RL-oriented starting point with a very restricted game |
| [tinymtg](https://github.com/Talor-A/tinymtg) | TypeScript/Bun | Reports about 5,000 playable cards; lists missing major mechanics | Compact core, structured effects, callbacks, Forge importer | Readable reference; missing visibility is a major research issue |
| [Majik](https://github.com/bg9m9r/majik) | C#/.NET 10 core and separate server | State machine, priority, stack, effects; Modern metadata includes an implementation flag | Factories, spell-template binders, event bus | Credible additional library candidate; RL contract not established |
| [Magicgrove](https://github.com/pinky39/grove) | C#; .NET Framework application | README reports 1,118 cards, focused on M15 and Urza's block | Game-tree search, heuristics, implemented card collection | Historical AI and limited-format research reference |

Sources for the important distinctions: [Forge modules](https://github.com/Card-Forge/forge), [XMage README](https://github.com/magefree/mage/blob/master/readme.md), [Argentum gym](https://github.com/wingedsheep/argentum-engine/blob/main/gym/README.md), [Phase README](https://github.com/phase-rs/phase/blob/main/README.md), [Manabrew README](https://github.com/witchesofthehill/manabrew/blob/main/README.md), [Arcana audit](https://github.com/levineuwirth/arcana/blob/main/docs/audit-2026-08.md), [managym README](https://github.com/jacklionheart/managym/blob/main/README.md), [tinymtg README](https://github.com/Talor-A/tinymtg/blob/main/README.md), [Majik README](https://github.com/bg9m9r/majik/blob/main/README.md), and [Magicgrove README](https://github.com/pinky39/grove/blob/master/README.md).

### 4.2 Simulation and agent interface

“Adapter needed” is an assessment of the work required to expose a uniform research contract, not a statement that an engine cannot run automated games.

| Project | Headless / agent entry point | Observation and legal-action interface | Branching / replay evidence | Main adaptation cost |
|---|---|---|---|---|
| Forge | Source contains `SimulateMatch`; player-controller abstraction | Rich controller callbacks; tensor interface needs an adapter | Simulation seed option exists; search simulator also exists | Capture all decisions and project player-safe observations |
| XMage | Engine/test harness; replaceable AI players | Player decision APIs; no standard Gym interface established in upstream survey | Simulation-oriented AI code; exact research replay contract needs validation | Direct engine integration or adaptation of MageZero fork |
| Magarena | AI and simulation test code | Internal choices/search; training wrapper needed | MCTS implementation visible in source | Extract driver, audit hidden-state access, modernize |
| Wagic | C++ game/AI integration | Custom AI interface; research observation schema not established | General RL snapshot/replay contract unknown | Separate simulation from application concerns |
| Argentum | `GameEnvironment`, `MultiEnvService`, optional HTTP server | `reset`, `step`, `observe`, legal actions; complex choices require structured responses | Documented constant-time fork and in-memory snapshot/restore | Encode actions/features and cover structured decisions |
| Phase | Separate `engine` and `phase-ai` crates | Reducer and AI legal actions; no Python training wrapper verified | Immutable state with structural sharing | Training bindings, observation tests, snapshot/RNG audit |
| Manabrew | Rust engine and Java parity harness | Gameplay/trace interfaces; training adapter not verified | Same decks, seed, and deterministic choices in parity tests | Complete needed parity, then expose research API |
| Arcana | Rust AI tools; Python whole-episode path | **Python `MtgEnv.step()` is a stub**; `run_episode` is documented as working | Deterministic core; audit says full-state serialization unfinished | Finish step binding, resolve fidelity/performance gaps |
| managym | Python `Env` via pybind11 | Acting-player observations and object-referencing actions documented | Clone, replay, and checkpoint guarantees not established | Expand rules and establish reproducibility contract |
| tinymtg | Synchronous and asynchronous execution reported | Player-response mechanism; visibility listed as unimplemented | Deterministic rewind/replay/fork reported | Information hiding, rule gaps, training adapter |
| Majik | `GameFacade` / drivers; separate bot project | DTOs and decisions exist; RL schema not verified | Exact clone and replay guarantees unknown | In-process runner and complete decision adapter |
| Magicgrove | Internal AI simulations | Search-oriented APIs; no training wrapper verified | User save/load reported; not a measured MCTS snapshot API | Cross-platform extraction, modern rules, bindings |

Source-observed anchors: [Forge simulation driver](https://github.com/Card-Forge/forge/blob/master/forge-gui-desktop/src/main/java/forge/view/SimulateMatch.java), [Forge player controller](https://github.com/Card-Forge/forge/blob/master/forge-game/src/main/java/forge/game/player/PlayerController.java), [XMage AI plugins](https://github.com/magefree/mage/tree/master/Mage.Server.Plugins), [Magarena MCTS](https://github.com/magarena/magarena/blob/master/src/magic/ai/MCTSAI.java), [Wagic AI](https://github.com/WagicProject/wagic/blob/master/projects/mtg/src/AIPlayerBaka.cpp), [Arcana Python step implementation](https://github.com/levineuwirth/arcana/blob/main/arcana-py/src/env.rs), and [managym agent contract](https://github.com/jacklionheart/managym/blob/main/managym/agent/README.md). Other entries derive from the upstream documentation linked in section 4.1.

### 4.3 Maintenance and licensing snapshot

Dates below are GitHub API **`pushed_at` dates**, not verified dates of the latest meaningful engine improvement. Repository pushes can include branches, automation, or documentation. All rows with dates were observed on the research date; activity alone is not a quality score.

| Project | Observed push date | Status signal | Source license evidence |
|---|---|---|---|
| Forge | 2026-09-26 | Active established project | GPL-3.0 family; [license](https://github.com/Card-Forge/forge/blob/master/LICENSE) |
| XMage | 2026-09-26 | Active established project | MIT; [license](https://github.com/magefree/mage/blob/master/LICENSE.txt) |
| Magarena | 2023-04-24 | Long gap in upstream pushes; not marked archived | GPL-3.0; [license](https://github.com/magarena/magarena/blob/master/LICENSE.txt) |
| Wagic | 2026-08-30 | Still receiving pushes | BSD-style source, separate asset terms; [summary](https://github.com/WagicProject/wagic/blob/master/LICENSE), [engine license](https://github.com/WagicProject/wagic/blob/master/projects/mtg/LICENSE) |
| Argentum | 2026-09-26 | Active newer project | MIT text inspected; [license](https://github.com/wingedsheep/argentum-engine/blob/main/LICENSE) |
| Phase | 2026-09-26 | Active newer project | MIT OR Apache-2.0 per [README](https://github.com/phase-rs/phase#license) |
| Manabrew | 2026-09-26 | Active pre-release port | Own code AGPL-3.0-or-later; vendored Forge GPL-3.0-or-later per [licensing description](https://github.com/witchesofthehill/manabrew#license) |
| Arcana | 2026-08-04 | Research line explicitly frozen; application hardening documented | No top-level license found in inspected tree; API license null |
| managym | 2026-01-17 | **Archived** | No top-level license found in inspected tree; API license null |
| tinymtg | 2026-09-17 | Recent experimental work | No license found in inspected tree or package metadata; Forge-derived content also needs provenance review |
| Majik | 2026-09-21 | Active newer project | Apache-2.0 per [README](https://github.com/bg9m9r/majik#license) and API metadata |
| MageZero | 2026-09-22 | September 2026 second-alpha announcement | MIT in repository API metadata; inspect the separate XMage fork too |
| Magicgrove | Not checked | README describes version 4 and 2020 revival | GPL-3.0; [license](https://github.com/pinky39/grove/blob/master/LICENSE) |

GitHub metadata is available at `https://api.github.com/repos/{owner}/{repository}` for each linked repository. Its automated license classifier is not definitive: it returned `NOASSERTION` for Argentum, Wagic, and Incantus even though readable license texts exist. Conversely, a null result by itself does not prove that no permission exists anywhere in a repository. “Unverified” entries are not recommended for copying into this project until their actual terms are established.

## 5. Detailed assessment of the leading options

### 5.1 Forge: practical baseline for broad-deck simulation

Forge separates `forge-game`, `forge-core`, and `forge-ai`, although its simulation launcher initializes application-level `FModel`. The inspected launcher implements repeated games, match/tournament options, optional game logs, per-player AI profiles, and a seed flag. Its `simulateOffthreadGame` method is a stub, so the presence of that method should not be mistaken for a usable parallel simulation API. [Simulation source](https://github.com/Card-Forge/forge/blob/master/forge-gui-desktop/src/main/java/forge/view/SimulateMatch.java)

For this project, Forge is valuable as an executable comparison target and a baseline opponent. A training adapter needs to intercept more than priority decisions: target selection, optional effects, payment, ordering, and choices during resolution also matter. Its [player-controller abstraction](https://github.com/Card-Forge/forge/blob/master/forge-game/src/main/java/forge/game/player/PlayerController.java) is the relevant integration surface.

**Assessment:** start here for broad unattended match simulation. For training, use a persistent process and measure the controller adapter before extracting or rewriting the engine. Do not infer per-game memory from desktop application RAM recommendations.

### 5.2 XMage: broad rules coverage with a permissive foundation

XMage's README reports extensive card support, multiplayer, server-side hidden-information enforcement, and roughly 9,000 tests. These support its inclusion as a mature reference, but do not prove every card interaction correct. Its repository separates the engine, card implementations, AI plugins, and tests. [XMage documentation](https://github.com/magefree/mage/blob/master/readme.md)

**Assessment:** XMage is especially attractive if permissive source licensing and realistic card coverage are priorities. A custom in-process agent should use an explicit observation projection; a safe multiplayer client protocol does not automatically make every internal AI API safe for learning. Evaluate the existing MageZero integration before implementing another Java/Python bridge.

### 5.3 Magarena: useful search implementation, older ecosystem

Magarena contains an actual MCTS implementation with worker infrastructure and a cache. The inspected class also accepts a `cheat` parameter; fairness cannot be assumed without checking how a selected configuration uses it. This is a concrete reason to audit baseline opponents as well as learned policies. [MCTS source](https://github.com/magarena/magarena/blob/master/src/magic/ai/MCTSAI.java)

**Assessment:** good material for learning how an established implementation controls search complexity. Less compelling than Forge/XMage for a new broad-format platform because of the observed maintenance gap. Neither competitive strength nor rollout speed was measured here.

### 5.4 Wagic: native code, but extraction is real work

Wagic is a configurable C++ card-game application built for multiple desktop and handheld/mobile platforms. Its source has separate engine and resource licensing. [Project overview](https://github.com/WagicProject/wagic), [license summary](https://github.com/WagicProject/wagic/blob/master/LICENSE)

The AI implementation directly references the game observer, current target chooser, stack layer, and game phases. That demonstrates an implemented game/AI path, but not an existing compact RL interface. [AI source](https://github.com/WagicProject/wagic/blob/master/projects/mtg/src/AIPlayerBaka.cpp)

**Assessment:** investigate when C++ reuse is a strong preference. Budget for a headless driver, observation projection, and reproducibility audit. Native compilation is insufficient evidence that extraction will be cheaper than wrapping a JVM engine.

### 5.5 Argentum: strongest documented research-facing contract in this survey

Argentum's gym module provides an environment wrapper, masked observations, legal actions, fork/snapshot/restore, and a multi-environment service. Forking shares immutable state; this supports its constant-time fork claim but says nothing by itself about transition allocation costs. Complex decisions such as target selection, ordering, and mana-source selection require structured responses rather than a single flat action ID. [Gym documentation](https://github.com/wingedsheep/argentum-engine/blob/main/gym/README.md)

The separate trainer module supplies MCTS and self-play interfaces with pluggable features, evaluators, and sinks. Structured decision expansion can report unsupported choices and fall back to a resolver; that boundary needs attention if the research claim requires learning all decisions. [Trainer documentation](https://github.com/wingedsheep/argentum-engine/blob/main/gym-trainer/README.md)

**Assessment:** a high-priority prototype candidate. Keep tree search in the JVM and batch neural inference where possible. Its observation digest should not automatically be used as a complete imperfect-information search key: observation history, beliefs, and hidden-world samples can differ despite identical current observations. Confirm experimental card coverage and benchmark direct-library versus HTTP execution.

### 5.6 Phase: promising native library, with coverage requiring scrutiny

Phase separates the Rust engine from AI, WASM bindings, and server code. It documents pure reducers and immutable state using persistent data structures. The README also explicitly distinguishes a large parsed card database from the many cards still requiring implementation. [Architecture and coverage context](https://github.com/phase-rs/phase/blob/main/README.md)

**Assessment:** one of the first native candidates to benchmark. Use the native core, establish the exact supported experimental decks, and measure transition and branch costs. Browser availability does not establish native self-play speed, deterministic replay, or Python integration.

### 5.7 Manabrew: Forge-compatible port and differential-testing model

Manabrew includes a Rust engine, a pinned Java Forge reference, Forge card scripts, and a parity harness. It reports that the Java path is more broadly usable while Rust coverage catches up. Its comparison approach uses identical decks, seeds, and deterministic decisions. [Project and parity overview](https://github.com/witchesofthehill/manabrew/blob/main/README.md)

**Assessment:** particularly relevant if `mtg-lab` wants to consume Forge's card definitions. The parity methodology is useful even if the code is not reused. Compatibility preserves upstream quirks as well as correct behavior; disagreements should ultimately be resolved against the rules, not simply by assuming either implementation is right. Account for the project's AGPL terms and Forge's separate GPL terms when considering reuse.

### 5.8 Arcana: substantial implementation, material research limitations

Arcana has unusually relevant research components: legal-action utilities, information-set handling, search policies, and Python episode execution. However, its more specific Python documentation and implementation contradict an overly broad reading of the top-level “Gymnasium-compatible” description: **`MtgEnv.step()` raises `NotImplementedError`**. The documented working Python path is `run_episode`, including callable policy support. [Python module documentation](https://github.com/levineuwirth/arcana/blob/main/arcana-py/README.md), [step source](https://github.com/levineuwirth/arcana/blob/main/arcana-py/src/env.rs)

The August audit reports approximations marked in roughly 48% of card files, unfinished state serialization, missing layer dependency ordering, and a session path that clones state while retaining an unbounded event log. It records fixes for earlier information leakage and action-order truncation, so those fixed defects should not be presented as still open. [Upstream audit](https://github.com/levineuwirth/arcana/blob/main/docs/audit-2026-08.md)

The README explicitly says its original performance targets were never formally measured and its RL research tracks are frozen negative results. [Current README](https://github.com/levineuwirth/arcana/blob/main/README.md)

**Assessment:** study the design and documented failures, but do not adopt on the strength of the card count or speed targets. Licensing, exact card semantics, the per-action binding, and hot-path costs are all adoption gates.

### 5.9 managym: small native RL environment with a narrow game

managym is a C++ environment with pybind11, intended for the companion [manabot](https://github.com/jacklionheart/manabot) project. Its declared card focus is basic lands and simple creatures. The agent contract distinguishes the turn's active player from the player currently responsible for a choice and filters hidden cards from observations. [Overview](https://github.com/jacklionheart/managym/blob/main/README.md), [agent contract](https://github.com/jacklionheart/managym/blob/main/managym/agent/README.md)

**Assessment:** useful for a creature-combat curriculum or as an interface reference. It is archived, lacks a verified top-level license, and is not evidence that broader Magic can be added cheaply. Native pointer ownership also makes correct state cloning an explicit design task rather than an automatic capability.

### 5.10 tinymtg: compact and candid about missing mechanics

tinymtg describes a roughly 10,000-line TypeScript core with deterministic replay/forking and synchronous rollouts. Its unsupported list includes visibility, planeswalkers, cost changes, alternative costs, double-faced/split cards, and first/double strike. Forge definitions feed its imported card pool. [README](https://github.com/Talor-A/tinymtg/blob/main/README.md)

**Assessment:** worthwhile for understanding a compact rules implementation, but inappropriate for general imperfect-information training until visibility and the required mechanics are implemented. Confirm reuse terms before taking code or imported cards. “Small” and “correct for the supported subset” are separate questions from “complete Magic.”

### 5.11 Majik: modern .NET alternative

Majik separates the core library, API contracts, server, bot, and tests. It documents explicit turn/priority services, named card factories, and Oracle-text template binders. Its embedded Modern card metadata uses an `IsImplemented` flag; the size of that metadata pool must not be reported as working-card coverage. Its README also says bot integration tests are mostly skipped. [Architecture and development documentation](https://github.com/bg9m9r/majik/blob/main/README.md)

**Assessment:** include if C# is attractive to the team. Establish an unattended game loop, complete decision coverage, repeatable seeds, and branching behavior before investing in training. The core's separation means server database requirements need not necessarily be requirements of a research worker.

### 5.12 Magicgrove: focused historical AI implementation

Magicgrove documents minimax-style simulation with heuristic pruning, hidden-information handling, game saving, and limited tournament/deck generation. Its Windows/.NET Framework application and selected older sets make it a different proposition from a modern broad-card engine. [README](https://github.com/pinky39/grove/blob/master/README.md)

**Assessment:** useful for baseline and search design, especially limited formats. Its reported card count is small enough to understand its scope, but no conclusion about correctness, speed, or porting effort follows from that count alone.

## 6. Training systems and research benchmarks

These must not be counted as independent, equally comprehensive rules engines merely because they offer training commands.

| Project | Underlying simulation | Research support | Limitations / relevance |
|---|---|---|---|
| [MageZero](https://github.com/WillWroble/MageZero) | [Custom XMage fork](https://github.com/WillWroble/mage) | Deck-specific self-play, MCTS, state encoding, policy/value learning, training CLI | Strong practical integration reference; evaluate abstractions and hidden information |
| [Commander AI Lab](https://github.com/KoalaTrapLord/commander-ai-lab) | Forge backend **and a distinct Python approximation** | Batch matches, decision logs, training and deck-building pipeline | Backends must not be treated as semantically equivalent; platform is much broader than a lean core |
| [manabot](https://github.com/jacklionheart/manabot) | managym | Companion training layer identified by managym | Not an independent engine; training implementation not audited here |
| [MTG-Causal-RL](https://arxiv.org/html/2605.06066v1) | Paper describes a Gymnasium MTG environment | Masked actions, partial observations, archetype transfer and causal evaluation | Paper inspected; linked anonymous source endpoint was inaccessible, so license and code behavior remain unverified |

### MageZero: existing integration, not universal MTG mastery

MageZero's September README announces a second public alpha and describes a deck-local approach. Its policy heads and feature encoding constrain the research problem; success against a selected opponent pool is not evidence of a generally strong Magic player. The README also identifies imperfect information and heap-intensive simulation as continuing challenges. [MageZero README](https://github.com/WillWroble/MageZero/blob/main/README.md)

**Assessment:** the strongest existing end-to-end starting point discovered for training with a mature engine. Reproduce a small training/evaluation run before adapting it. Keep its fork revision, opponent decks, search settings, and action abstractions in every experiment manifest.

### Commander AI Lab: distinguish orchestration cost from engine cost

The project reports about 0.04 simulations/second for a Forge path that launches a JVM subprocess per game, while its Python backend models a selected set of combat/Commander behaviors. That does not establish Forge's steady-state speed or show that the Python backend implements the same game. [Backend comparison](https://github.com/KoalaTrapLord/commander-ai-lab/blob/main/README.md)

**Assessment:** useful orchestration examples, but a poor default dependency for a lean kernel. Benchmark a persistent Forge process before drawing performance conclusions from this wrapper.

### MTG-Causal-RL: narrower benchmark with explicit evaluation design

The paper reports a 3,077-dimensional partial observation, 478 masked actions, and five deck archetypes. Its contribution includes paired-seed evaluation and causal diagnostics. The linked source release could not be inspected in this survey, so these remain paper claims, and general comprehensive-rules coverage is not established. [Paper](https://arxiv.org/html/2605.06066v1), [linked source release](https://anonymous.4open.science/r/mtg-causal-rl-E927)

**Assessment:** relevant methodology and a candidate for later reproducibility review, not a verified replacement for a general engine.

## 7. Historical engines and the experimental long tail

These projects broaden the survey beyond the most visible applications. “No verified training API” means no such API was established by the inspected evidence, not a definitive absence from every branch.

| Project | Language / design | Evidence and scope | License / activity evidence | Research disposition |
|---|---|---|---|---|
| [Incantus](https://github.com/Incantus/incantus) | Python; engine source under `src/engine` | Historical CCG engine; no top-level README returned by API | MIT/X11-style [license](https://github.com/Incantus/incantus/blob/master/LICENSE) inspected; last observed push 2012-12-30 | Architectural reference; extensive modernization and conformance work |
| [jmagicdev/jmagic](https://github.com/jmagicdev/jmagic) | Java | Original network-play application; [README](https://github.com/jmagicdev/jmagic/blob/master/README.txt) describes hosted games and configurable priority passing | License not established; observed push 2014-10-22 | Historical engine; do not confuse with the project below |
| [lucasdavid/jmagic](https://github.com/lucasdavid/jmagic) | Java, agents/actions/validation observers | Separate project; players explicitly submit actions including draw and damage operations | MIT per README/API; observed push 2021-07-27 | Educational agent framework; automatic full rules coverage not established |
| [Firemox](https://sourceforge.net/projects/firemox/) / [public mirror](https://github.com/hcross/firemox) | Historical Java card-game project | SourceForge distribution and mirror located; mirror README is minimal | Exact mirror license and current buildability unverified; release listing is historical | Preserve in inventory; not a preferred new research base |
| [mtg-pure](https://github.com/thomaseding/mtg-pure) | Haskell, typed card DSL and functional engine | Demonstrated terminal gameplay; AI explicitly low priority; integer overflow deviations acknowledged | BSD-3-Clause in API metadata; observed push 2026-07-22 | Particularly useful for typed effect/card representation research |
| [Pawl](https://github.com/tfausak/pawl) | Haskell | Repository describes an MTG rules engine; minimal README does not establish usable coverage | 0BSD in API metadata; observed push 2026-09-26 | Investigate implementation before ranking as a runnable substitute |
| [Cardboard](https://github.com/Julian/cardboard) | Python | Repository identifies itself as an MTG engine; little descriptive documentation returned | MIT in API metadata; observed push 2012-02-07 | Historical prototype; build and capabilities unverified |
| [MagicEngine2](https://github.com/CubeArtisan/MagicEngine2) | C++ | Repository describes automated games; no README returned | License unverified; observed push 2019-07-02 | Potential simulation reference; not enough evidence to recommend |
| [MTG Paradox Engine](https://github.com/MTG-Paradox-Engine/mtg-paradox-engine) | TypeScript | README provides build/test commands but no rules-coverage or research contract | License unverified; observed push 2020-03-26 | Prototype; needs capability and license inspection |
| [MTGEngine](https://github.com/marthinwurer/MTGEngine) | Java | Repository explicitly describes creature-only AI research | MIT in API metadata; observed push 2016-04-08 | Small experimental baseline, not broader Magic |
| [Phelddagrif](https://github.com/timdestan/phelddagrif) | Scala | Repository describes a rules engine; capability audit not performed | License unverified; observed push 2017-12-18 | Discovery-only candidate |
| [Hyperdraft](https://github.com/discordwell/Hyperdraft) | Python event/interceptor core | MTG plus other card games; README distinguishes real-set metadata from implemented effects | License and maintenance metadata not verified | Mechanic prototyping reference; not demonstrated comprehensive Magic |
| [deckmaste.rs](https://github.com/msmorgan/deckmaste.rs) | Rust, Oracle-text recovery and typed semantic representation | Executable vertical slice and ambitious parse/render/play pipeline | **PolyForm Noncommercial 1.0.0** per README | Source-available comparison, outside the unrestricted open-source shortlist |

Additional discoverable engine repositories include [kurokikaze/mtg-engine](https://github.com/kurokikaze/mtg-engine), [cmeister2/corrosion](https://github.com/cmeister2/corrosion), [Joey9801/mtg-engine-rs](https://github.com/Joey9801/mtg-engine-rs), and [nomatics/mox](https://github.com/nomatics/mox). Only repository-level discovery evidence was established for these; coverage, buildability, agent APIs, and licensing are not ranked. Including their names is not an endorsement of usability.

**Historical names with unresolved reuse status:** BotArena, Manalink, and Multiverse appear in older discussions of MTG implementations. The present survey did not establish a sufficiently clear, independently reusable open-source engine and license for each. They therefore remain discovery leads rather than scored alternatives. In particular, a mod or source fragment associated with a commercial game is not automatically a standalone open-source engine.

## 8. Related software that is not a substitute

| Tool/category | Why it appears in searches | Why it does not satisfy the simulation requirement |
|---|---|---|
| [Cockatrice](https://github.com/Cockatrice/Cockatrice) | Major open-source MTG play application | Virtual tabletop; not a comprehensive rules executor for autonomous self-play |
| [Pisces](https://github.com/michael-celani/pisces) | Calls itself a goldfishing engine | README describes manual tabletop actions, zones, and camera mirroring |
| [CSCE585 MTG-game-engine](https://github.com/csce585-mlsystems/MTG-game-engine) | ML and “engine” in its name | Its documented objective is Monte Carlo card-draw probability, not full adversarial gameplay |
| [NeoForge](https://github.com/AdrianLopez98/NeoForge) and other Forge clients | New application frontends | Depend on Forge; do not add independent rules-engine coverage merely by presenting a different UI |
| XMage web clients | Browser-based play | Frontends/proxies over XMage, not separate simulation kernels |
| [MTG SDK](https://github.com/MagicTheGathering/mtg-sdk-python), MTGJSON, Scryfall | Programmatic card access | Card data alone does not execute spells, timing, or interactions |
| Generic RL toolkits | Training, batching, opponent evaluation | Require a separately implemented MTG environment |
| Legends of Code and Magic environments | Card-game RL and similar name | Different game; results cannot establish MTG rules coverage |
| Arena / Magic Online | Official digital gameplay | Not open-source engines available for embedding in this repository |

This classification prevents misleading comparisons between a rules-complete ambition, a card search database, and a simplified statistical model. Primary descriptions: [Cockatrice](https://github.com/Cockatrice/Cockatrice), [Pisces](https://github.com/michael-celani/pisces/blob/master/README.md), [draw-probability project](https://github.com/csce585-mlsystems/MTG-game-engine/blob/main/README.md), and [MTG SDK](https://github.com/MagicTheGathering/mtg-sdk-python).

## 9. What the available performance evidence actually says

### 9.1 Published claims versus measured comparisons

| Project | Available evidence | Valid interpretation | Invalid interpretation |
|---|---|---|---|
| MageZero | README reports ~250 games/hour using 13 CPU threads and 300-simulation MCTS; ~150 single-thread MCTS simulations/second at 4 GHz | One upstream end-to-end search/training workload | XMage itself can execute only this many random-policy games |
| Commander AI Lab | ~0.04 simulations/second for a Forge wrapper spawning a JVM per game | Wrapper/startup-heavy measurement | Steady-state Forge kernel throughput |
| Arcana | Original target >20,000 games/second; README explicitly says targets were never formally measured | An unvalidated target | Achieved speed or superiority over mature engines |
| Argentum | Constant-time fork/snapshot reported via shared immutable state | Branch creation need not deep-copy the complete state | All transitions are constant-time or allocation-free |
| Phase / Manabrew / managym / tinymtg | Native cores or synchronous simulation paths described | Reasons to benchmark | Verified throughput rankings |
| Other surveyed engines | No comparable benchmark established by inspected sources | Performance remains unknown | Evidence that they are inherently slow |

Sources: [MageZero metrics](https://github.com/WillWroble/MageZero/blob/main/README.md), [Commander AI Lab backend table](https://github.com/KoalaTrapLord/commander-ai-lab/blob/main/README.md), [Arcana target disclaimer](https://github.com/levineuwirth/arcana/blob/main/README.md), [Argentum state operations](https://github.com/wingedsheep/argentum-engine/blob/main/gym/README.md).

A games/second number combines card complexity, policy quality, action abstraction, game length, and search budget. A faster policy can make an engine look faster by conceding or playing badly. A more detailed engine can appear slower because it exposes more real decision points. Neither number alone measures research efficiency.

### 9.2 Proposed benchmark for an actual selection

Use three workloads, each with a frozen card/rules version and explicit support manifest:

1. **Core combat:** lands and simple creatures, blocking, damage, death, and mulligans. Useful for low-overhead comparison, but insufficient for broad Magic claims.
2. **Interactive duels:** instants, counterspells, targeted removal, draw, triggers, and meaningful priority windows. Exercises actual agent interaction.
3. **Stress scenarios:** large combat choices, replacement effects, dependent continuous effects, copying, recursive triggers, and long games. Exposes fidelity failures and tail latency.

Only compare engines on a common supported semantic subset. If an engine lacks a required mechanic, mark that workload unsupported rather than silently substituting an approximation. Run separate broader workloads to measure the benefit of each engine's additional coverage.

| Measurement | Suggested protocol |
|---|---|
| Startup / loading | Time cold process, registry load, and first game separately |
| Transition speed | Time fixed scripted trajectories with rendering and logging disabled |
| Full environment step | Include legal-action generation and observation encoding; exclude model inference initially |
| Policy overhead | Measure random, simple heuristic, and fixed-budget search separately |
| Branch cost | Measure fork + one action + discard, not just cloning an empty state |
| Memory | Report steady and peak RSS, bytes per game, allocation rate, and tree-node memory |
| Scaling | Repeat with 1, 2, 4, 8, … workers; record aggregate and per-worker throughput |
| Python/JVM boundary | Compare direct calls with batched calls and local IPC where available |
| Reliability | Count completed, truncated, invalid-action, crash, unsupported, and divergent games separately |
| Reproducibility | Repeat seed/action traces and snapshot continuations; compare normalized state hashes |

Record hardware, OS, compiler/runtime, optimization flags, warm-up, engine commit, data hashes, deck lists, seed list, worker counts, and action semantics. Use several independent timing batches and report dispersion as well as the mean. A nominally identical seed does not produce identical shuffles across engines: use explicit initial states/deck orders or map random events for differential tests.

For MCTS, report both environment transitions/second and policy decisions/second at a fixed node budget. For learning, report wall-clock time to a predefined held-out performance level. Those measurements answer different questions and should remain separate.

## 10. Implications for a lean `mtg-lab` engine

The following is a proposed design direction, not an implementation already present in this repository.

### 10.1 Define the research game before optimizing it

Start with two-player games and a named, versioned subset of cards/mechanics. Two-player constructed play avoids adding multiplayer elimination, coalition incentives, and Commander-specific bookkeeping before the basic research loop works. If Commander is the actual research target, make that an explicit first-class requirement instead of assuming two-player findings will transfer.

Separate three support levels for every card: metadata available, executable behavior implemented, and behavior covered by tests. Reject unsupported experimental decks. Do not silently turn an unsupported spell into a vanilla object or no-op; a policy can learn to exploit that discrepancy.

### 10.2 Small native core and a complete decision boundary

Rust and C++ are reasonable candidates for a new kernel, with Python for training. The useful property is control over memory, batching, and state ownership, not the language label itself. Keep UI, network services, image lookup, deck websites, and model inference outside the transition kernel.

A conceptual interface:

```text
reset(config, seed)                    -> state
decision(state)                       -> actor, decision_kind, legal_choices
observe(state, player, knowledge)      -> player_observation
apply(state, choice)                  -> next_state, public_events, outcome
fork(state)                           -> independent_branch
snapshot(state) / restore(snapshot)   -> versioned_continuation
```

`actor` can differ from the active player. Decision kinds must include choices inside resolution, not just main-phase actions. `outcome` must distinguish termination, draw, external truncation, and engine failure. Illegal actions should return a structured error, not destroy a worker.

The core need not be purely immutable. Compare compact mutable states plus cloning/undo against persistent structures using representative branches. Prefer the simpler design until profiling shows a concrete bottleneck.

### 10.3 Keep action semantics explicit

Do not flatten arbitrary Magic into a fixed action array without documenting limits. Targets, modes, variable costs, chosen amounts, attack/block assignments, trigger ordering, and payment choices create combinatorial decisions.

Use structured or autoregressive decisions with legality checks at each stage. If some decisions are automated by heuristics, report that as part of the environment definition: the learned policy is then solving an abstracted game. Preserve a path to expose those decisions later.

Record semantic action descriptions in replay data. A transient legal-action index is only meaningful for the particular decision at which it was generated.

### 10.4 Make hidden information a tested boundary

Keep full simulator state separate from player-visible state and remembered information. Masking the opponent's hand is only the beginning: library order, revealed cards, face-down identities, private choices, logs, object IDs, and random seeds can leak information.

Test observation invariance under changes to information the player has not learned. Keep privileged training critics separate from deployable policies, and never allow a search procedure to use the actual hidden world when the experiment claims imperfect-information play.

A transposition key based only on current visible objects may conflate different histories or beliefs. Choose keys appropriate to the search algorithm and store exact simulator snapshots separately.

### 10.5 Optimize the measured hot path

Promising implementation choices to test include compact object IDs, shared immutable card definitions, bounded/optional event histories, reusable action buffers, precompiled effect representations, and batch-native stepping. Avoid parsing Oracle text or card scripts inside every rollout.

Randomness must be explicit and captured in branches/checkpoints. Stable iteration and tie-breaking matter as much as recording a seed. Avoid shared global RNG state between workers.

Do not target GPU execution first unless the research subset has sufficiently bounded state and control flow. Irregular stack resolution and variable-length choices make a GPU simulator a separate architectural investment. CPU simulation with batched accelerator inference is a reasonable initial hypothesis to measure.

### 10.6 Treat established engines as fallible references

Create short fixtures that both the new engine and at least one mature engine can execute. Compare normalized zones, life, counters, priority, stack, triggers, and legal choices at chosen checkpoints. Use two independent references for difficult disagreements when practical, then resolve against the applicable rules version.

Prioritize interactions over isolated card tests: simultaneous triggers, replacement choices, zone-change identity, last-known information, continuous-effect dependencies, and choices during resolution. A passing parser or a large generated-card corpus cannot replace those checks.

Apply deterministic resource limits for runaway simulations, but distinguish engine truncation from a game draw. Do not award an agent a win because an opponent's effect exceeded a simulator budget.

## 11. Build-versus-reuse decision

| Strategy | Initial value | Main cost | Choose it when |
|---|---|---|---|
| Wrap Forge/XMage | Broad playable card pool and useful existing opponents | Complete action/observation bridge; memory and branching work | Research requires realistic decks soon |
| Adopt MageZero | Existing training pipeline over a mature engine | Deck-local assumptions and fork maintenance | Agent experiments matter more than owning the kernel |
| Extend Argentum | Explicit environment and search interfaces | Supported-pool validation; JVM/transport optimization | Its cards and rules cover the planned experiments |
| Extend Phase | Native core with separate AI module | Coverage validation and research bindings | Native execution is important and the supported subset suffices |
| Extend Manabrew | Forge compatibility and differential-testing infrastructure | Port incompleteness and copyleft-compatible integration | Reusing Forge scripts is a central requirement |
| Build a new bounded core | Full control over data, stepping, batching and scope | Highest correctness and card-authoring burden | Measurements show existing options cannot satisfy essential requirements |

Suggested selection gates, in order:

1. **Scope:** choose experimental decks, formats, allowed abstractions, and required mechanics.
2. **Reuse:** confirm the actual engine/card-code licenses for any code to be incorporated.
3. **Correctness:** run representative fixtures and complete games with no silent unsupported behavior.
4. **Research interface:** demonstrate all required decisions, player-safe observations, and repeatable continuation.
5. **Performance:** benchmark useful decisions and games on the intended hardware.
6. **Decision:** choose the lowest total implementation/maintenance cost that meets the prior gates.

For the current goal, the practical initial evaluation set is **Forge, XMage/MageZero, Argentum, and Phase**, with **Manabrew** added if Forge-compatible card scripts are attractive. Study **Magarena, managym, tinymtg, and Arcana** for specific implementation ideas and failure modes. Do not select a core solely because it advertises native speed or a large card count.

## 12. Revisions and source trail

### 12.1 Observed default-branch heads

These heads were obtained with `git ls-remote … HEAD` on 2026-09-26 after reading the sources. They identify the snapshot around which this assessment was made; live branch links can change, and files were not all fetched in an atomic snapshot. For reproducible evaluation, check out a fixed commit and save every external card/rules dataset too.

| Project | Observed HEAD |
|---|---|
| Forge | `95dc682bf92460f49cebd7a9578f06ccf60d5569` |
| XMage | `000d8a7abc0ac31cc24af08691423e0c24dc59e7` |
| Argentum | `0635ec676b0e1a15e8090c819c7e2fefe83e9c55` |
| Phase | `16f141360620d7fb8444d724ac8c9a54a1ce6726` |
| Manabrew | `35868343c77132714e43b6a227092658f956296a` |
| Arcana | `98e61559adbd7b6d0c7e9051194593d788421f16` |
| MageZero | `11a5974668c3f0f3d19559d7dcbf6e323f140808` |
| managym | `81c09cf4a776b61ab6de19fafd5c7bf087361c3a` |
| tinymtg | `5fe63ea543d6d1533d3d2fc746bb7084e5107363` |
| Majik | `6f6b2aaf78923df41a35d60501e1879aa667bb40` |

### 12.2 High-value files to inspect in a follow-up prototype

| Question | Primary source |
|---|---|
| What can Forge's batch runner actually do? | [SimulateMatch.java](https://github.com/Card-Forge/forge/blob/master/forge-gui-desktop/src/main/java/forge/view/SimulateMatch.java) |
| Where would a Forge agent connect? | [PlayerController.java](https://github.com/Card-Forge/forge/blob/master/forge-game/src/main/java/forge/game/player/PlayerController.java) |
| What does XMage already test and simulate? | [Mage.Tests](https://github.com/magefree/mage/tree/master/Mage.Tests), [AI plugins](https://github.com/magefree/mage/tree/master/Mage.Server.Plugins) |
| How does existing MTG MCTS look? | [Magarena MCTSAI.java](https://github.com/magarena/magarena/blob/master/src/magic/ai/MCTSAI.java) |
| What should a training wrapper expose? | [Argentum gym](https://github.com/wingedsheep/argentum-engine/blob/main/gym/README.md), [gym-trainer](https://github.com/wingedsheep/argentum-engine/blob/main/gym-trainer/README.md) |
| How does the C++/Python boundary represent players and choices? | [managym agent contract](https://github.com/jacklionheart/managym/blob/main/managym/agent/README.md) |
| Which Arcana claims need qualification? | [Audit](https://github.com/levineuwirth/arcana/blob/main/docs/audit-2026-08.md), [Python documentation](https://github.com/levineuwirth/arcana/blob/main/arcana-py/README.md), [step implementation](https://github.com/levineuwirth/arcana/blob/main/arcana-py/src/env.rs) |
| How are Rust and Java Forge compared? | [Manabrew parity guide](https://github.com/witchesofthehill/manabrew/blob/main/docs/PARITY_TESTING.md) — follow-up reading, not independently audited here |

### 12.3 Remaining evidence gaps

The next useful work is empirical: reproducible builds, supported-deck manifests, conformance fixtures, decision/observation audits, and a shared benchmark. It should establish actual per-step and per-branch costs, not expand the comparison with unsupported numerical ratings. Smaller projects with sparse documentation and unresolved licensing remain candidates for investigation rather than confirmed substitutes.
