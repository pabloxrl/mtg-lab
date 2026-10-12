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

The [current engine-validation plan](programs/engine-validation.md) is the accepted
2026-10-09 scope amendment. Finish the rules engine and playable decks, then use
Forge and XMage's existing AIs to generate reproducible game-based tests.
Training integrations and the original later-stage research interfaces are deferred.
Existing implementations and their regression tests remain intact. The
[capability reference](capabilities.md#what-works-today) identifies what works today.

## Tests govern delivery

CI runs the complete suite in parallel partitions with reusable build caches;
all partitions must pass the required `verify` gate. The local full-suite command
is unchanged. See [verification execution and caching](testing/torture-suite.md#parallel-ci-and-caches).

The [executable torture baseline](testing/torture-suite.md) runs through the
[Docker quickstart](getting-started.md#quickstart-run-the-checks) and required CI. Agents must retain coded regressions,
add independently justified tests with each behavior, run the complete suite and
obtain separate review before merging. Current executable coverage is verification
tooling, versioned episode RNG, object storage, opening choices, turns, land/mana transitions, supported creature and token casting, creature mana, targeted instants, flying/reach combat, rules endings and reproducible native-policy matches. The [M2 audit](evidence/m2-coverage-audit/README.md) maps all 175 M2 catalog cases and individually reviews 123 distinct semantic scenarios. [Sixteen terminal native/XMage games](evidence/admission/README.md) cover every ordered matchup/starter row with explicit reference exceptions. These checks do not qualify the planned reference-AI corpus.

Work is delivered as [small tested changes](programs/atomic-delivery.md), with
independent review, protected integration and exact-main verification. Existing
M2 acceptance stays intact. The legacy program stops after M2; reviewed
[registration work](https://github.com/pabloxrl/mtg-lab/issues/241) defines the
new M3 mechanics task graph before implementation dispatch; reference-corpus
registration remains later work.

## How games will become tests

Forge and XMage's existing AI players will play the frozen decks. Capture actual
random outcomes, every consequential choice and intermediate states; a seed or
human-readable log alone is insufficient. Translate the records into strict
replays, with each engine calculating its own rules. Compare intermediate semantic
states as well as the final result. Both references must ultimately support the
required complete-game capture and replay contract; existing smoke bridges are
not proof that this works.

Differences become reproducible, minimized cases justified against pinned rules
and card definitions. Neither generated output nor reference-engine agreement is
automatically the correct answer. Retain full original games and focused tests
for rare interactions, illegal actions, privacy and recording failures. Counts
and seeds add diversity; they do not establish coverage on their own.

## Delivery stages

| Stage | Outcome | Current evidence / exit |
| --- | --- | --- |
| Existing M0 | Scope and verifier foundations | Complete; [passing audit](evidence/m0-reaudit/README.md). |
| Existing M1 | First scalar slice and unattended games | PASS for the six-card slice; [gate audit](evidence/m1-gate/README.md) and linked delivery receipts. |
| E1 / existing M2 | Complete frozen toy decks and playable games | PASS for full-pool rules, scoped reference checks and scalar baseline in the [gate report](evidence/m2-gate/README.md); completion requires its linked review, protected merge and exact-main CI receipts. |
| Engine validation M3 (formerly E1a) | Reusable mechanics and runtime card definitions | [Registered plan](programs/engine-validation-m3.md): migrate only the current frozen pool with generic choices/replay and matched measurements. No unseen-definition admission or completion claim from registration. |
| E2 | First reference capture/replay pilot | Planned: 10 complete games, all eight matchup rows, repeated mtg-lab replay and intermediate checks. |
| E3 | Measured generation and replay cost | Planned: 100 attempts with complete accounting; startup, capture, conversion, replay, latency and memory measured separately. |
| E4 | Both references and frozen torture corpus | Planned: qualify the second reference through the same 10/100 gates; admit at least 1,000 distinct complete games, at least 500 from each source, balanced over deck/starting-seat rows, replayed in all three engines. |
| E5 | Engine acceptance | Planned: full corpus, targeted coverage and independent critical cases pass for the exact candidate; no unexplained divergences or hidden skips. |

See the [detailed plan](programs/engine-validation.md) for admission criteria,
measurement protocol, error adjudication and execution ownership. Game-generation
speed is not yet measured; the 100-attempt campaigns establish realistic estimates.
Generation is occasional; frozen replay avoids repeating AI search during testing.
A fast local subset may aid development, but does not replace existing mandatory
checks or full-corpus final acceptance.

The original M3–M5 requirements are preserved in the
[historical program](programs/rfc-0002.md) and pinned RFC ledger, but are deferred,
not current release gates or completed work. No new RL concepts or training work
are part of this plan. Completing E5 qualifies this engine-validation scope only.

The new M3 is distinct from deferred legacy M3 Batch/Python/RL. E2–E5 retain
their acceptance goals as later roadmap stages, not M3 prerequisites or an
immediate queue. No new real or synthetic card definitions are authorized in M3.
