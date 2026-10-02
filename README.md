# mtg-lab

A Magic: The Gathering research engine for running reproducible games, training AI
players, and collecting trustworthy game data. The intended system is headless:
experiments, testing and training run without a person playing either seat.

**Current stage: M1 scalar vertical slice — acceptance passed.** The repository
has runnable games, recorder/replay integration and real Forge/XMage reference checks.
The scalar Rust slice, native random and heuristic policies, and a bounded headless
`simulate` command are implemented. Full-pool AI matches, Python bindings and
training integrations remain planned;
there is no `train` command. The [M1 gate audit](doc/evidence/m1-gate/README.md)
verifies the historical six-card scalar slice; milestone completion is conditional on its
independent review, protected merge and exact-main CI receipts in the linked workpad.

This snapshot is grounded in the [passing M0 audit](doc/evidence/m0-reaudit/README.md)
and the [M1 acceptance report](doc/evidence/m1-gate/README.md).
For current work, blockers and stage activation, see the
[live implementation program](https://github.com/pabloxrl/mtg-lab/issues/7).
The README describes delivered capabilities; it is not a live worker dashboard.

## What works today

| Area | Delivered | Boundary |
| --- | --- | --- |
| Frozen scope | Exact rules revision, card/token identities, two deck lists and source verification | Metadata validation does not implement card behavior. |
| Test design | 320 concrete designs across 80 capabilities, with issue owners and implementation stages | These are designs, not 320 passing game tests. |
| Scenario tooling | Versioned neutral fixtures, six admitted original cases and checkpoint comparison | The six cases are authored; their engine execution is future work. |
| Reference engines | Pinned headless XMage and Forge priority-pass smoke; XMage [eleven matched Growth/Bite scenarios](doc/evidence/matched-growth-bite/README.md): destination/source responses, departed targets, Bite-killed blocker and cleanup with/without discard; combat, terminal and [opening count checks](doc/evidence/core-integration/README.md) | Synthetic reference positions; opening checks inject the initial hands and compare counts, not shuffle/card order. No full-game or complete-state verification. Receipts are Linux ARM64. |
| Episode RNG | [Specified SplitMix64 v1](doc/rng.md), stable episode derivation and separate owned environment/policy streams, with [known-answer tests](crates/mtg-core/tests/rng.rs) | RNG primitive used by opening shuffle/reset and the separately versioned native random policy; batch runners remain planned. |
| Object storage | [Generation-safe slots and ordered zones](doc/objects.md), shared frozen card identities, slot reuse and reset epochs, with [literal-ledger tests](doc/evidence/objects/README.md) | Storage primitive used by opening reset; no rules decisions or player observations in the storage primitive. |
| Opening choices | [Validated reset, deterministic shuffle and London mulligans](doc/opening.md), independent seat declarations, cumulative ordered bottoming and top-card draw primitive, with [reset](doc/evidence/opening/README.md) and [mulligan evidence](doc/evidence/mulligan/README.md); [core integration audit](doc/evidence/core-integration/README.md) | Completed opening connects to turns and the rules outcomes below. Inspection and explicit shuffle injection are privileged. |
| Turn progression | [Empty-stack priority, untap, first/subsequent draws, mana boundaries and cleanup discards](doc/turns.md), with [turn acceptance](doc/evidence/turns/README.md) and [bounded turn/cleanup checks](doc/evidence/turn-settlement/README.md) | Vanilla combat, targeted-effect cleanup and rules outcomes are implemented below. |
| Lands and mana | [Forest/Mountain plays, tap mana and private payment continuations](doc/mana.md), with [independent payment enumeration and regressions](doc/evidence/mana/README.md) | Standalone payments use floated mana and have safe payer-only choices below; Elf/Druid mana abilities and casting payments are implemented below. |
| Creature casting | [Bear Cub/Swab Goblin casts, payment-time land activations, stack resolution and summoning sickness](doc/casting.md), with [casting acceptance](doc/evidence/casting/README.md) | Sorcery timing; creature-only LIFO tests use declared synthetic stacks. |
| Targeted instants | [Giant Growth/Bite Down, factored targets, response chains, revalidation, damage and simultaneous cleanup](doc/targets.md), with [target acceptance](doc/evidence/targets/README.md) | Normal-reset Growth response script, synthetic Bite edges and real Growth/Bite reset integration below; other spell mechanics remain planned. |
| Creature mana | [Elf/Druid acceptance and catalog mapping](doc/evidence/creature-mana/README.md) | Llanowar Elves 1/1 and Druid of the Cowl 1/3; immediate tap-for-G and transactional casting payments, sickness/rejection, combat, replay and typed recording. No haste or nonmana activation. |
| Dragon Fodder and Goblin tokens | [Token acceptance and catalog mapping](doc/evidence/tokens/README.md) | 1R sorcery through cast/payment/stack; two red 1/1 tokens, sickness, blocking, death/cessation, stale identities, replay and typed recording. No haste, modal spells or cast triggers. |
| Vanilla combat | [Attacker/blocker choices, current damage allocation, simultaneous damage and lethal cleanup](doc/combat.md), with [combat acceptance](doc/evidence/combat/README.md) and [bounded settlement checks](doc/evidence/combat-settlement/README.md) | Bear Cub/Swab Goblin, Elf/Druid and Goblin tokens; signed life totals, no combat keywords. Normal reset-to-combat scripts, synthetic edge cases and five matched XMage combat scenarios. |
| Scalar slice integration | [Land, creature, Growth/Bite acceptance and catalog crosswalk](doc/evidence/scalar-integration/README.md) | Normal-reset native games, private quantum work and matched synthetic references; six-card supported slice only. M1 gate, full-pool and full-game reference qualification remain outstanding. |
| Rules outcomes | [Lethal life, attempted empty draws, concession and explicit reset](doc/terminal.md), with [terminal acceptance](doc/evidence/terminal/README.md) | Stable seats and best-of-one finality; normal-reset red/green mirror scripts and matched synthetic XMage boundaries. Recorder rewards and passive CLI games are available below. Terminal state uses the core views below. |
| Core player views | [Seat-filtered structured observations and opening candidate commands](doc/views.md), with [privacy acceptance](doc/evidence/views/README.md) | Own hand, public zones/counts and historical revelations; no hidden library order, opponent hand or RNG. This older API is unavailable during private continuations; use the structured policy API below for spell choices. Tensors remain planned. |
| Structured policy choices | [Versioned seat-routed opening, pass, land, spell, combat and discard decisions](doc/policy-decisions.md), with [privacy and rejection acceptance](doc/evidence/policy/README.md) | Visible row references, legal masks and revision/generation validation, including restore invalidation. Acting-seat Growth/Bite targets and staged payment/cancellation, committed public stack targets, and [spell privacy/response tests](doc/evidence/policy-spells/README.md). Factored attacker subsets, blocker mappings, modern damage allocations and cleanup discards with [both-seat policy scripts and privacy checks](doc/evidence/policy-combat/README.md); public committed combat relationships. [Private-view and replay integration](doc/evidence/private-replay-integration/README.md) covers the supported M1 choices; tensor and transport integrations remain planned. |
| Core work quantum | [Resumable reset, shuffle and opening work](doc/opening.md#bounded-work) plus [priority and M1 spell settlement](doc/spell-settlement.md), [combat damage settlement](doc/combat.md#bounded-damage-settlement) and [turn/cleanup settlement](doc/turns.md#bounded-turn-and-cleanup-work), with [opening checks](doc/evidence/quantum/README.md) and [spell checks](doc/evidence/spell-settlement/README.md) | Scalar paths drain the same work. Internal yields expose no player view/decision; turns, draws, untap, cleanup/discard and combat damage support yields; combat declarations remain atomic choices. No scheduler or batch runner yet. |
| Core snapshots | [Versioned full-state save and atomic restore](doc/snapshot.md), with [snapshot acceptance](doc/evidence/snapshot/README.md) | Privileged compatible-engine saves; RNG, opening choices and spell/combat/turn/cleanup settlement yields preserved. Fresh handle scopes on load; no migration. [Pending-choice integration](doc/evidence/private-replay-integration/README.md) covers fresh-process targets/payment, responses, combat allocation and cleanup; missing required state fields reject atomically. |
| Semantic action records | [Standalone versioned encoding/decoding](doc/actions.md), with [hand-authored fresh-process acceptance](doc/evidence/actions/README.md) | All implemented opening, land/mana, cast/target/payment, combat, cleanup decisions and concession. Privileged birth identity plus zone incarnation; strict rejection without mutation. Complete M1 replay envelope/checkpoints are available below; opening replay remains supported. |
| Opening replays | [Versioned semantic opening scripts and first-divergence verification](doc/replay.md), with [replay acceptance](doc/evidence/replay/README.md) | Privileged rules/cards/engine/config/RNG pins, strict consumed choices and ordered bottom identities. Opening format; [integration acceptance](doc/evidence/private-replay-integration/README.md) includes replay/version/privacy checks. CLI verification and final seat inspection are available below. |
| In-memory trajectories | [Owned canonical records and per-seat readers](doc/trajectories.md), with [reward/boundary acceptance](doc/evidence/trajectory/README.md) | Legacy v1 plus [structured v2 acceptance](doc/evidence/trajectory-v2/README.md): exact policy observations/domains, ordered submissions, pending privacy, cancellation timing and once-only seat rewards. V1 is preserved; v2 durable storage is supported below. The scalar composition has [played-game acceptance](doc/evidence/collector-audit/README.md); the [aggregate scalar collector audit](doc/evidence/scalar-collector-audit/README.md) covers the composed path. The [M1 trajectory integration audit](doc/evidence/trajectory-integration/README.md) adds restored-terminal reward and fresh-process reload checks. The [composed CLI audit](doc/evidence/composed-cli/README.md) covers capture and validation; [final CLI integration evidence](doc/evidence/unattended-integration/README.md) is available; the [M1 gate audit](doc/evidence/m1-gate/README.md) records scoped acceptance. |
| JSONL trajectories | [Bounded scalar writer and strict validator](doc/trajectory-jsonl.md), with [handwritten roundtrip/failure evidence](doc/evidence/trajectory-jsonl/README.md) | Canonical v1 and [structured v2](doc/evidence/structured-jsonl/README.md) owned records, checksums/counts, atomic file publication and explicit backpressure/errors. One run per file; episode/file byte budgets required. CLI validates explicit v1/v2 JSONL and run manifests with [played-data acceptance](doc/evidence/trajectory-cli/README.md); Parquet remains planned; the [scalar integration audit](doc/evidence/trajectory-integration/README.md) covers the recorder; the [composed CLI audit](doc/evidence/composed-cli/README.md) covers producer-to-validator checks; [final CLI integration evidence](doc/evidence/unattended-integration/README.md) is available. |
| Dataset manifests | [Scalar run manifest API](doc/run-manifest.md), with [handwritten acceptance](doc/evidence/run-manifest/README.md) | Explicit v1/v2 contracts bind versions, seat provenance, capture declarations and completion to one checksummed JSONL file. Completed runs only by default; explicit diagnostics for unfinished runs. Actual owned results can be persisted through the collector adapter below; CLI validates explicitly selected dataset/manifest files with completed-only defaults and diagnostic opt-in; no seed-selection implementation. |
| Owned scalar episodes | [Driver API](doc/episode-driver.md), with [played-game and rejection acceptance](doc/evidence/episode-driver/README.md) | Normal reset/quantum advancement, seat views, accepted semantic history, concession and owned completed/incomplete results. No mutable Game escape. [Privileged semantic input](doc/episode-driver.md#privileged-semantic-input) accepts existing records with live seat/freshness checks through the same submit/concede paths ([acceptance](doc/evidence/semantic-input/README.md)). [Canonical v2 capture](doc/episode-capture.md) records all accepted choices, optional supplied statistics and once-only seat rewards ([acceptance](doc/evidence/episode-capture/README.md)); capture on/off preserves state/RNG. [Bounded episodes](doc/episode-budgets.md) enforce decision/turn/injected-clock limits, work-quantum and record-count ceilings, and once-only completed/truncated/failed/incomplete accounting ([acceptance](doc/evidence/episode-budgets/README.md)). Internal-work stops retain unsealed diagnostic prefixes when final views are unavailable. [Authorized local replay binding](doc/episode-replay.md) verifies completed captured results against their actual reset/history and final state/RNG; per-artifact caller authorization controls release, and unfinished results explicitly have no replay ([acceptance](doc/evidence/episode-replay/README.md)). No policy selection; coordinated local publication is described below. The scalar composition has [independent full-game and publication acceptance](doc/evidence/collector-audit/README.md); the [aggregate scalar collector audit](doc/evidence/scalar-collector-audit/README.md) also covers ordered mulligan publication for both starting seats. The [trajectory integration audit](doc/evidence/trajectory-integration/README.md) checks the complete scalar record path; the [composed CLI audit](doc/evidence/composed-cli/README.md) covers capture and validation; [final CLI integration evidence](doc/evidence/unattended-integration/README.md) is available; the [M1 gate audit](doc/evidence/m1-gate/README.md) records scoped acceptance. |
| Owned result persistence | [Derived provenance and sealed bundle API](doc/collector-persistence.md), with [played-game and failure acceptance](doc/evidence/collector-persistence/README.md) | Actual completed/truncated v2 captures pass through the existing writer and manifest loader; failed/incomplete ordinals remain diagnostic accounting only. Config/deck/version/seat/limit provenance is derived and checked; policy identity/statistics remain trusted caller inputs. Returns bounded owned sealed bytes and metadata for a separate publisher. Optional opaque replay references are preserved, with authorization and byte validation deferred to publication. No sampling; CLI wiring is described below. |
| Scalar simulation | [Validated headless `mtg simulate`](doc/simulate.md), explicit seeds/budgets, passive and native M1 policies, versioned JSONL summaries and signal accounting, with [subprocess acceptance](doc/evidence/simulate/README.md) | Legacy `pass-v1` is preserved. [Schema 2 native simulation](doc/simulate.md#native-simulation-schema-2) runs LegalRandom/Heuristic through the owned Driver with explicit decision/work/record bounds and optional capture ([native acceptance](doc/evidence/native-cli/README.md)). [Schema 3 privileged scripts](doc/simulate.md#semantic-scripts-schema-3) submit exact ordered choices through that same Driver ([acceptance](doc/evidence/script-cli/README.md)). [Optional canonical capture](doc/simulate.md#canonical-capture-schemas-2-and-3) publishes manifest/JSONL and separately authorized replays ([acceptance](doc/evidence/captured-cli/README.md)). No protocol, UI, trainers or performance qualification. |
| Native random opponent | [Versioned `mtg-policy` M1 policy](doc/random-policy.md), with [both-seat complete games, reachable action scripts and privacy/vector tests](doc/evidence/random-policy/README.md) | Uses only seat-authorized observations, independent episode/seat RNG and declared factored distributions. Forest/Mountain, Bear Cub/Swab Goblin, Growth/Bite, Dragon Fodder, Llanowar Elves, Druid of the Cowl and vanilla token combat; other deck cards remain uncastable. Native simulate wiring is available; optional canonical CLI capture. |
| Native heuristic opponent | [Versioned deterministic M1 strategy](doc/heuristic-policy.md), with [literal choices, hidden twins and reproducible complete games](doc/evidence/heuristic-policy/README.md) | Fixed scoring and tie rules over authorized observations; all M1 choice families. Same nine-card plus token subset as the random policy; no full-pool support. Playing strength is not a rules oracle. |
| Played-game replays | [Complete M1 semantic replay API](doc/replay.md#complete-played-game-replay-v1), with [normal-reset response/combat acceptance](doc/evidence/played-replay/README.md) | Privileged core API; terminal-only streams, pinned versions/config/randomness, full semantic checkpoints and first divergence. Opening and played formats are distinct; CLI verification supports both with redacted diagnostics ([acceptance](doc/evidence/played-replay-cli/README.md)). Seat inspection remains opening-only. [Integration audit](doc/evidence/private-replay-integration/README.md) covers private views, snapshot suffixes and strict replay rejection; full-game reference qualification remains outstanding; the [M1 gate audit](doc/evidence/m1-gate/README.md) records scalar acceptance. |
| Local run publication | [Validated dataset and authorized replay publication](doc/collector-publication.md), with [real capture and failure acceptance](doc/evidence/collector-publication/README.md) | Exclusive run IDs, separate private replay storage, synced artifacts and manifest-last no-replace publication. Diagnostics preserve failed/truncated/incomplete accounting. Errors retain recovery files; uncertain commits are explicit and never automatically retried. Authorized replay reads require the retained owned result; policy dataset loading does not grant replay access. CLI capture reuses this publisher. No hostile-host isolation or sharding; [M1 acceptance and retained limits](doc/evidence/m1-gate/README.md). |
| Headless tools | [Replay verify/seat inspect, trajectory validate, checkpoint conformance and scalar benchmark smoke](doc/headless-commands.md), with [subprocess evidence](doc/evidence/headless-cli/README.md) | Opening and terminal played replay verification with explicit format routing; opening seat inspection and sealed v1/v2 JSONL/run-manifest validation; completed-only run loading with explicit diagnostic noncompletion; conformance compares supplied checkpoints, not reference execution. Passive smoke and `native-rollout-v1` expose raw timings and outcome accounting; [initial active-policy baseline](doc/evidence/unattended-integration/README.md), not M2 performance qualification. |
| Development | Docker toolchain, required CI, separate agent review and protected automated merging | The registered MVP stages are authorized; each still waits for its preceding audited gate. |

M0 verifies that the requirements, test designs and basic reference infrastructure
are ready. It does not certify a playable Magic engine. The original failed M0
audit is retained alongside the successful re-audit so the evidence remains traceable.

## Core module responsibilities

The canonical rules API is now rooted at [mtg_core::game](crates/mtg-core/src/game.rs).
It owns game state and shared decision identities.
[Opening setup](crates/mtg-core/src/opening.rs) handles reset, initial hands and
mulligans; the private [work executor](crates/mtg-core/src/work.rs) resumes owned
opening, turn, spell and combat work. The existing mtg_core::opening API remains
a compatibility re-export of the same types and implementation.

This extraction does not change rules, serialized field schemas or supported
cards. The existing conservative engine fingerprint includes the moved sources,
so snapshots and replays produced before the refactor are explicitly incompatible.
The private [cast state boundary](crates/mtg-core/src/cast_state.rs) now owns
pending target, cast and payment transitions. Rule entry points keep their
legality checks; policy views read the continuations through immutable accessors.
[Compatibility evidence](doc/evidence/cast-boundary/README.md) covers restoration,
cancellation, rejection and unchanged normal-reset history/capture. Other Game
state remains shared by rule modules. The private
[typed card definitions](crates/mtg-core/src/card_definitions.rs) provide the nine
supported cards’ costs, creature characteristics and behavior tags to casting,
targets, mana and combat. Frozen identities/hashes and unsupported-content errors
remain pinned; the existing synthetic Sentry characteristics do not grant
play support. [Compatibility evidence](doc/evidence/card-definitions/README.md)
covers the manifest, rejection boundaries and unchanged played history/capture.
The recorder uses [explicit typed conversions](crates/mtg-recorder/src/conversion.rs)
from authorized core records into its owned v1/v2 wire contracts. Wire validators
retain validation ownership; [compatibility evidence](doc/evidence/recorder-conversion/README.md)
covers canonical bytes, optional statistics and failure handling.

## Quickstart: run the checks

You can verify the current repository today. You do not need a GitHub token,
Codex login or a running Symphony worker for these checks.

Prerequisites: Git, Python 3, Docker Engine and Docker Compose v2, with Docker
running. Java, Rust and Maven run inside Docker. On macOS, use your existing
Docker provider; the project's current development setup uses Colima. Allow
roughly 8 GiB of VM memory for the managed setup and enough disk for build caches.
The first build needs network access and takes longer than later runs.

```sh
git clone https://github.com/pabloxrl/mtg-lab.git
cd mtg-lab
./scripts/verify-docker.sh
```

This builds the toolchain, checks documentation and the implementation/test plans,
runs every discovered Python test and Rust tests in debug and release, and exercises an isolated credential-free Symphony
runtime/sandbox smoke. Source is mounted read-only; build output stays in the
container. On AppArmor-enabled hosts, the runtime smoke's host-preparation helper
may need sudo to install the project's named profile; see the
[Docker runbook](doc/symphony-runbook.md#docker-runtime).
It does not dispatch work or use the live worker's credentials/workspace volume.
The isolated smoke container and its temporary volume are removed afterward.

A successful run reports passing documentation/program/design checks, tests,
`nested-sandbox-ok` and `credential-free-controller-ok`. Test counts will grow
with the implementation; a green check proves only the capabilities actually
covered by those tests.

### Try a small working example

After the quickstart has built `mtg-lab-toolchain:local`, run the synthetic
checkpoint comparator:

```sh
docker run --rm --cap-drop ALL --security-opt no-new-privileges \
  --mount "type=bind,source=$PWD,target=/workspace,readonly" \
  --workdir /workspace -e CARGO_TARGET_DIR=/tmp/target \
  mtg-lab-toolchain:local \
  cargo run --quiet --locked -p mtg-fixture -- fixtures/comparator/equal.json
```

It returns JSON with `"status":"pass"`. Replace `equal.json` with `life.json` to
see an intentional mismatch at `life.p1` and exit code 1. Both checkpoints are
supplied by the fixture: this demonstrates comparison, not a simulated game.
See the [comparator contract](doc/fixture-comparator.md) for its schema and errors.

To run two passive games unattended, use the same toolchain image:

```sh
docker run --rm --cap-drop ALL --security-opt no-new-privileges \
  --mount "type=bind,source=$PWD,target=/workspace,readonly" \
  --workdir /workspace -e CARGO_TARGET_DIR=/tmp/target \
  mtg-lab-toolchain:local \
  cargo run --quiet --locked -p mtg-cli -- simulate --config fixtures/simulate/pass-v1.json
```

This emits a run header, two completed episode summaries and aggregate counts as
JSONL. Both seats explicitly use `pass-v1`: keep seven, pass priority, discard at
cleanup. It plays no cards; the nonstarting seat loses to an empty draw on turn 68.
This is an automation/accounting baseline, not a measure of policy strength or
complete card support. [Configuration, exit codes and limits](doc/simulate.md).

Run the explicit land/Cub/Growth/combat script (101 decisions, then P1 concedes):

```sh
env -u DISPLAY -u WAYLAND_DISPLAY cargo run --quiet --locked -p mtg-cli -- simulate --config fixtures/simulate/script-v3.json </dev/null
```

The script is privileged input, includes every pass/payment/target/combat choice,
and has no policy fallback. It ends with life `[20,15]`, consumes exactly 102
records, and reports script consumption separately from game outcome. Payloads
are redacted from output; resolved game configuration and seeds remain privileged
run metadata. Limits stop the script explicitly; unused records never imply success.

Run two bounded native games (green mirror, heuristic versus legal-random):

```sh
env -u DISPLAY -u WAYLAND_DISPLAY cargo run --quiet --locked -p mtg-cli -- simulate --config fixtures/simulate/native-v2.json </dev/null
```

This uses `heuristic-mana-v1` and `legal-random-mana-v1`; prior M1 policy IDs
reject explicitly. It uses the owned Driver and the delivered policy subset (M1 plus Dragon Fodder/Goblin tokens and Elf/Druid mana creatures). Inspect
completed/truncated/failed/incomplete counts; explicit work exhaustion leaves an
incomplete episode. Capture defaults to disabled; schemas 2/3 can opt into
[bounded canonical publication](doc/simulate.md#canonical-capture-schemas-2-and-3)
with existing disjoint roots and explicit local-owner authorization. The
[composed CLI audit](doc/evidence/composed-cli/README.md) checks simulation through
dataset validation and replay verification, exact reload equality and failure
accounting. The [final CLI integration audit and initial baseline](doc/evidence/unattended-integration/README.md) cover the command matrix; the [M1 gate audit](doc/evidence/m1-gate/README.md) records scoped acceptance.

Inside the same toolchain container, try a bounded scalar command smoke:

```sh
cargo run --quiet --locked -p mtg-cli -- bench --workload scalar-pass-v1 --config fixtures/simulate/pass-v1.json
cargo run --quiet --locked --release -p mtg-cli -- bench --workload native-rollout-v1 --config fixtures/simulate/native-v2.json
cargo run --quiet --locked -p mtg-cli -- trajectories validate crates/mtg-recorder/tests/episode.jsonl
cargo run --quiet --locked -p mtg-cli -- trajectories validate crates/mtg-recorder/tests/episode.jsonl --manifest crates/mtg-recorder/tests/manifest.json
```

The benchmark emits raw elapsed time and separate completed/truncated/failed counts;
native rollout also reports policy time, accepted decisions and work calls. Neither
is performance qualification. The trajectory commands validate the handwritten
synthetic recorder fixture (one episode, one decision), not a captured full game.
See the [headless command contract](doc/headless-commands.md) for opening/played replay verification, opening seat
inspection, strict checkpoint comparison, dependency/input limits and exit codes.

To inspect the completed test-design mapping without executing games:

```sh
docker run --rm --cap-drop ALL --security-opt no-new-privileges \
  --mount "type=bind,source=$PWD,target=/workspace,readonly" \
  --workdir /workspace -e PYTHONDONTWRITEBYTECODE=1 \
  mtg-lab-toolchain:local python3 scripts/test_plan.py
```

At M0 this reports 80 capabilities, 320 designed slots, six related authored
fixtures and zero execution evidence added. Real reference acceptance requires
separately prepared pinned source/dependency caches; follow the
[XMage](references/xmage/README.md) and [Forge](references/forge/README.md)
instructions inside Docker. The quickstart does not run those heavyweight bridges.

## The first playable scope

The first MVP uses two custom 40-card decks: red and green, each with 16 lands
and 24 spells, covering 20 distinct card names plus a red 1/1 Goblin token.
Supported matchup plans include red/green, both mirrors and both starting seats.
These are research decks, not a claim of sanctioned-format legality or balance.

The scoped mechanics include priority and the stack, casting and mana, targets,
triggers, combat, selected creature keywords, tokens, temporary effects, cleanup
and game outcomes. Unsupported content must be rejected explicitly.
[Exact decks and source pins](doc/card-manifests.md) and
[RFC 0002](doc/rfcs/0002-first-mvp.md) define the boundary.

The planned implementation separates:

- A Rust rules core and native batch runner, with deterministic state and replay.
- A Python binding and observation-safe trajectory readers for research.
- PettingZoo/Gymnasium adapters and tested RLlib, TorchRL and SB3-Contrib workflows.
- A machine-readable CLI, with optional terminal play through the same rules.
- Separately installed Forge/XMage test references, outside the production core.

These are delivery commitments; the table above identifies what currently exists.

## Tests govern delivery

The [executable torture baseline](doc/testing/torture-suite.md) runs through the
Docker quickstart above and required CI. Agents must retain coded regressions,
add independently justified tests with each behavior, run the complete suite and
obtain separate review before merging. Current executable coverage is verification
tooling, versioned episode RNG, object storage, opening choices, turns, land/mana transitions, vanilla creature casting, targeted instants, vanilla combat and rules endings; most of the 320 game designs and full AI matches remain planned.

Work is delivered as [small tested changes](doc/programs/atomic-delivery.md):
M1 has separate RNG, identity, opening, rules, replay, data and CLI deliveries.
The original component issues check their integration. Later stages are split
against delivered interfaces before dispatch, preserving every acceptance gate.
You describe outcomes and resolve essential product/access questions; agents
handle planning, testing, review, merging and queue progression.

## How games will become tests

AI players will play the frozen decks while the runner records every decision and
random outcome. Strict scripted controllers will replay the same sequence in
mtg-lab, XMage and Forge, comparing intermediate states as well as the result.
Sharing a seed alone cannot make different engines play or shuffle identically.

Differences become reproducible, minimized cases with expectations derived from
the pinned rules and card definitions. Neither AI-generated output nor agreement
between reference engines is automatically treated as the correct answer.
Focused tests also cover rare interactions, invalid actions, privacy leaks,
recording failures, batching, real training updates and performance accounting.

The [test strategy](doc/testing/README.md) explains the full plan. Planned designs,
authored fixtures, actual execution and passing evidence stay distinct. Passing
the design validator cannot certify engine behavior.

## Delivery stages

| Stage | Outcome | Current evidence |
| --- | --- | --- |
| M0 | Freeze scope, design tests, prove verifier/reference foundations | Complete; [passing audit](doc/evidence/m0-reaudit/README.md). |
| M1 | First scalar engine slice, private views, replay, initial recordings and unattended games | PASS for the six-card scalar slice; [gate audit](doc/evidence/m1-gate/README.md). Completion requires its linked review/merge/exact-main CI receipts. |
| M2 | Complete frozen card pool, expanded XMage coverage and scalar baseline | Planned. |
| M3 | Native batching, Python, durable datasets and real training integrations | Planned. |
| M4 | Complete machine protocol and scripted terminal interaction | Planned. |
| M5 | Broader fuzz/mutation tests, dual-reference full games and release qualification | Planned. |

The [tracked program](doc/programs/rfc-0002.md) preserves every RFC acceptance
requirement. A milestone completes only after its independently reviewed audit
and checks on the merged commit pass. [Operations #59](https://github.com/pabloxrl/mtg-lab/issues/59)
authorizes the existing MVP stages in advance; successful gates hand off to the
next eligible task. Workers cannot expand scope or bypass dependencies.

## Working with the agents

As the person driving the project, describe the problem, desired outcome and
important constraints in a GitHub issue. For the existing MVP, the scope and
implementation issues are already recorded in the
[parent program](https://github.com/pabloxrl/mtg-lab/issues/7).

The coordinator plans dependencies and activates authorized work. Symphony workers
implement it, test it, obtain a separate agent review, merge through protected
checks, and record evidence in each issue's workpad. Routine PR review and merging
do not require you. Essential product decisions and unavailable access may still
need your input. Normal milestone transitions use automatic bounded handoff.
Authorized work continues across sessions without a total elapsed-time or
lifetime dispatch-count stop. Three repair/review cycles trigger diagnosis and
replanning; real blockers and deliberate holds still need resolution. One-worker
concurrency, per-operation timeouts and bounded retry backoff remain in force.
A healthy idle service does not mean the whole program is finished. The
[continuous-delivery checks and deployment limits](doc/evidence/continuous-delivery/README.md)
distinguish tested controller continuation from coordinator-owned image deployment;
arbitrary process termination during handoff is not guaranteed to recover.

If you operate the agent service, use the [Symphony runbook](doc/symphony-runbook.md)
for authenticated setup, start/stop, persistent storage and recovery. From an
already configured checkout:

```sh
./scripts/symphony/service.sh status
curl --fail http://127.0.0.1:4318/api/v1/state
```

The local status endpoint shows currently running work. The parent issue and
individual workpads explain completed milestones, blockers and next actions.
See [agentic operations](doc/agentic-operations.md) for the full driver guide.

## Repository map

| Path | Purpose |
| --- | --- |
| [crates/](crates/) | Rust workspace: core RNG/storage/opening/turn/mana/casting/target/combat/terminal transitions and seat-filtered views, scalar JSONL recorder and executable fixture comparator. |
| [scripts/](scripts/) | Manifest/scenario validators, verification and reference runners. |
| [data/](data/) | Frozen rules/card metadata and scoped capability registry. |
| [fixtures/](fixtures/) | Original scenario and comparator inputs. |
| [references/](references/) | Reference bridge source, pins and minimal execution receipts. |
| [tests/](tests/) | Python verification-tool tests. |
| [docker/](docker/) | Container toolchain and runtime configuration. |
| [doc/](doc/README.md) | RFCs, test plans, operating guides and audit evidence. |

Start with the [project charter](doc/rfcs/0001-project-charter.md) for purpose,
[RFC 0002](doc/rfcs/0002-first-mvp.md) for delivery scope, and
[AGENTS.md](AGENTS.md) for contribution rules. Repository licensing and exact
release/data redistribution decisions remain open release prerequisites, recorded
in the [provenance policy](doc/provenance-policy.md); M0 is not release clearance.

## Keeping this README accurate

Agents must assess README impact on every delivery. Update it in the same PR when
usable commands, prerequisites, supported behavior, architecture, limitations or
verified milestone status change. Explain a no-change decision in the issue
workpad or PR when the README is unaffected. Reviewers check the README against
the actual implementation and evidence, including commands they can reproduce.

Promote a feature only when its required evidence exists; keep planned work labeled
as planned. Milestone report PRs update the stage table, with completion conditional
on successful delivery; failed audits retain an incomplete stage. Link durable
reports rather than copying transient worker counts or queue status. Keep setup
instructions aligned with executable scripts, preserve limitations, and use the
live program issue for current execution status.

### Following agent progress

The local [Symphony dashboard](http://localhost:4318/) includes a “What’s happening”
card that refreshes every five minutes. The working agent explains its current
work, why it matters for the milestone and project, what comes next and any blocker.
The card shows the note's age and says when an update is missing or old. It reuses
worker notes without another AI session; existing live metrics stay available.
See the [operator runbook](doc/symphony-runbook.md#five-minute-activity-summary)
for the Docker deployment and reporting contract.
