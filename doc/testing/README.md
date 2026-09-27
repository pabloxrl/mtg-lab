# MVP test strategy

Status: planned tests and delivery obligations, **not implemented game support**.
This plan refines [RFC 0002](../rfcs/0002-first-mvp.md), preserving every acceptance
requirement and its [tracked owner](../programs/rfc-0002.json). It delivers the
missing design in [correction #52](https://github.com/pabloxrl/mtg-lab/issues/52),
registered through [operations #54](https://github.com/pabloxrl/mtg-lab/issues/54).
The earlier [failed M0 audit](../evidence/m0/README.md) remains historical evidence;
a fresh gate must assess the corrected main commit.

## The approach

Use **AI-played games as reproducible test inputs**, supported by focused,
independently justified rules tests. Start with small deterministic legal-random
and heuristic policies, using exactly the frozen red and green POC decks, their
mirrors and both starting seats. An LLM choosing each move adds cost and
nondeterminism without being necessary for this testing goal. Learned policies
can later add diverse games through the same recording contract.

Record what happened: opening decks, all random outcomes, every choice, and
meaningful intermediate states. Replace the players with strict scripted
controllers and replay those choices in mtg-lab, XMage and Forge. Each engine
must execute the rules itself. Compare states after the same semantic events,
not after the same internal callback or array index. A shared seed alone does
not make different engines shuffle or choose identically.

Start generation in the native engine when its relevant capabilities exist.
Admit independently sourced reference-origin games later through the same
neutral format; never assume either upstream engine already exports that format.
The source engine supplies a candidate trace, **not the expected answer**.
A disagreement becomes a reproducible, minimized case. Agents justify the
expectation from the pinned rules and card definitions, obtain independent
review, and preserve the original failure plus its regression test. Agreement
between three engines is useful corroboration, not a substitute for the rules.

Full games catch interactions and integration mistakes. Focused synthetic tests
cover rare rules, illegal requests, data failures and privacy boundaries that
an AI playing legal moves may never exercise. Preserve both: a minimized
synthetic regression does not replace the original reachable game.

## Plans and ownership

| Plan | Contents | Delivery owners |
| --- | --- | --- |
| [Capability designs](capability-test-plan.json) | 80 capabilities × positive, negative, interaction and regression; explicit setup, actions, expectations and independent basis | Per-case issue and milestone; design #52, implementation M1/M2 and later specialized owners |
| [Rules design notes](rules.md) | Rules/card coverage, synthetic reachability boundaries and oracle discipline | #17–#24; hardening #37/#38 |
| [AI games and reference replay](game-replay.md) | Decision/chance transcript, adapter expansion, campaigns, disagreement triage and acceptance | #17/#19/#21/#24/#37/#38 |
| [Independent data/RL cases](data-rl.md) | 24 additional ledger-based designs for recording, privacy, durable storage and real learning | #19/#20/#27–#33; qualification #39/#40 |
| [Contract decisions](contract-decisions.md) | Coordinator defaults for batch errors, zero-action seats, seeds, crash scope and other design ambiguities | Named owner in each decision |
| [System component tests](systems.md) | State, batching, privacy, trajectories, adapters, CLI, instrumentation and performance | #17–#21, #25, #27–#39 |

The component plans were drafted independently in fresh agent sessions with
only their assigned scope and repository requirements. They did not exchange
notes or read one another's drafts. This synthesis is coordinator work after
those independent assessments. Agreement is not claimed as independent execution
or proof of correctness.

## What must pass, and when

| Stage | Required test result |
| --- | --- |
| M0 | Complete reviewable designs; source/fixture pins; reject missing, duplicate or unresolved mappings. Existing real one-pass smoke works in both references. No full-game or 320-pass claim. |
| M1 | Relevant scalar tests fail for a behavioral reason before implementation; stack/cleanup/privacy and invalid-action invariants; real normal-reset slice games; deterministic replay and JSONL/reward boundaries; applicable reference checks. Slice games are not full frozen-deck qualification. |
| M2 | All frozen cards/mechanics, all M2 rules/card and scalar-interface category slots executed (batch rows remain due at M3), ≥100 distinct reviewed scenarios and XMage coverage for every card/capability; explicit mutation detections; full-pool AI capture and expanded XMage replay; honest baseline. A design count is not a scenario count. |
| M3 | Scalar/batch/binding/replay equivalence, failure-safe sharded data, actual masked training/update/evaluation/checkpoint/reload and trajectory reload for all three promised frameworks. |
| M4 | Machine CLI, persistent protocol and two scripted seat clients; no human game input; stale/wrong-seat/error/reattach/privacy checks. |
| M5 | Required ≥20 independent critical cases in both references and scripted full normal-reset games across required matchups/starting seats; long fuzz/mutation/mode campaigns, independent holdout and measured release qualification on the exact candidate. |

Run every implemented scoped conformance fixture in change CI, plus unit/lint,
replay, basic batch, headless CLI and relevant adapter smoke checks as delivered.
Rules changes also run impacted cached reference scenarios. Keep longer fuzzing,
full training compatibility and expanded reference runs in dedicated scheduled
jobs; the release gate must rerun its mandatory suites against the exact release
candidate. Shared-CI timing is diagnostic; performance gates use dedicated hardware.
All build/reference/training jobs use the pinned Docker toolchain, closed stdin,
explicit time/resource budgets and bounded process-tree cleanup.

New campaign sizes in the component plan are starting operational budgets, not
permission to weaken the RFC or count timeouts as successful games. A required
scenario that is unsupported, unavailable, disputed, skipped or timed out cannot
pass the relevant gate. Coverage reports must distinguish these statuses.

## Machine-checkable design, without invented evidence

`python3 scripts/test_plan.py` validates [the design catalog](capability-test-plan.json)
and runs in `./scripts/verify.sh`. Each capability/category has one stable design
ID and concrete setup, actions, expected result, basis, owner issue and milestone.
Several designs can later share a carefully reviewed fixture; one case may also
need several focused fixtures. These are design slots, not 320 distinct executed
scenarios. Text completeness is mechanically checked; its correctness is reviewed
against the independent basis and later proved through behavioral tests.

The catalog pins the rules and card manifest. Its `authored_fixtures` index links
the existing admitted corpus by exact content revision and declared capabilities.
Those are related authored inputs, not evidence that a new design has run or that
the linked fixture satisfies every category. Pending schema/smoke examples are
not promoted to admitted corpus fixtures. Changes to a linked fixture require
explicit review and an updated link; hash updates cannot bless changed expectations.

The capability registry's execution evidence remains separate and unchanged by
this plan. `python3 scripts/scenario.py coverage --require-passed` must still fail
until actual required passing evidence exists. Designs never turn capabilities
into `implemented`. Preserve design → authored/admitted fixture → actual execution
receipt → passing evidence as separate facts, each with exact revisions.

## How this works for the person driving the project

You describe the product behavior you want and any important boundaries. Agents
turn it into tracked work with tests and dependencies. For this MVP, the card pool
and acceptance criteria are already fixed, so you do not need to design cases,
play matches or review PRs.

As features become available, agents run the frozen decks automatically, save
replayable games, compare engines, investigate differences, and add reviewed
regressions. They implement changes, run the applicable tests, obtain a separate
agent review, merge through protected checks, and update the program evidence.
The coordinator advances the next stage only after its prerequisite gate passes.
A failed gate creates explicit corrective work; it does not silently waive a test.

Your useful status view is: what behavior is implemented, what has actually been
verified, which games or fixtures disagree, and what agents are doing next.
You are asked only for an essential product choice or missing access. A failed
match or an ordinary merge is agent work, not a request for you to intervene.
