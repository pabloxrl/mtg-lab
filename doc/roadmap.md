# Scope and roadmap

## The first playable scope

The first MVP uses two custom 40-card decks: red and green, each with 16 lands
and 24 spells, covering 20 distinct card names plus a red 1/1 Goblin token.
Supported matchup plans include red/green, both mirrors and both starting seats.
These are research decks, not a claim of sanctioned-format legality or balance.

The scoped mechanics include priority and the stack, casting and mana, targets,
triggers, combat, selected creature keywords, tokens, temporary effects, cleanup
and game outcomes. Unsupported content must be rejected explicitly.
[Exact decks and source pins](card-manifests.md) and
[RFC 0002](rfcs/0002-first-mvp.md) define the boundary.

The planned implementation separates:

- A Rust rules core and native batch runner, with deterministic state and replay.
- A Python binding and observation-safe trajectory readers for research.
- PettingZoo/Gymnasium adapters and tested RLlib, TorchRL and SB3-Contrib workflows.
- A machine-readable CLI, with optional terminal play through the same rules.
- Separately installed Forge/XMage test references, outside the production core.

These are delivery commitments; the [capability reference](capabilities.md#what-works-today) identifies what currently exists.

## Tests govern delivery

The [executable torture baseline](testing/torture-suite.md) runs through the
[Docker quickstart](getting-started.md#quickstart-run-the-checks) and required CI. Agents must retain coded regressions,
add independently justified tests with each behavior, run the complete suite and
obtain separate review before merging. Current executable coverage is verification
tooling, versioned episode RNG, object storage, opening choices, turns, land/mana transitions, supported creature and token casting, creature mana, targeted instants, flying/reach combat, rules endings and reproducible native-policy matches. Most of the 320 game designs and full-pool reference matches remain planned.

Work is delivered as [small tested changes](programs/atomic-delivery.md):
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

The [test strategy](testing/README.md) explains the full plan. Planned designs,
authored fixtures, actual execution and passing evidence stay distinct. Passing
the design validator cannot certify engine behavior.

## Delivery stages

| Stage | Outcome | Current evidence |
| --- | --- | --- |
| M0 | Freeze scope, design tests, prove verifier/reference foundations | Complete; [passing audit](evidence/m0-reaudit/README.md). |
| M1 | First scalar engine slice, private views, replay, initial recordings and unattended games | PASS for the six-card scalar slice; [gate audit](evidence/m1-gate/README.md). Completion requires its linked review/merge/exact-main CI receipts. |
| M2 | Complete frozen card pool, expanded XMage coverage and scalar baseline | Planned. |
| M3 | Native batching, Python, durable datasets and real training integrations | Planned. |
| M4 | Complete machine protocol and scripted terminal interaction | Planned. |
| M5 | Broader fuzz/mutation tests, dual-reference full games and release qualification | Planned. |

The [tracked program](programs/rfc-0002.md) preserves every RFC acceptance
requirement. A milestone completes only after its independently reviewed audit
and checks on the merged commit pass. [Operations #59](https://github.com/pabloxrl/mtg-lab/issues/59)
authorizes the existing MVP stages in advance; successful gates hand off to the
next eligible task. Workers cannot expand scope or bypass dependencies.

