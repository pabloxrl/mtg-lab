# Neutral scenarios and scoped capability registry v1

GH-12 delivers the **design and offline validation** aspects of RFC 0002
`R0002-B025/B026`. No production engine, game execution, corpus admission or
reference agreement is implemented here. GH-13/14 own reference bridges, GH-15
owns the initial reviewed corpus, and later engine/coverage tasks retain their
execution obligations. The [comparator v1](fixture-comparator.md) input, binary,
fixtures and tests are unchanged. A neutral scenario is not comparator input.

## Use

Python 3, standard library only, offline and without stdin/display:

```sh
python3 scripts/scenario.py validate fixtures/scenarios/priority-pass.json
python3 scripts/scenario.py coverage
python3 scripts/scenario.py coverage --require-passed
python3 -m unittest discover -s tests -p test_scenario.py
```

The last coverage command intentionally exits **2** today: there is no executed
engine coverage. This is an acceptance test of missing-coverage detection, not a
skipped conformance test. Successful shape/metadata validation exits 0 with
`status: valid`, never `pass`. Diagnosed input/coverage errors exit 2 with a JSON
error. CLI argument misuse uses argparse's exit 2 and stderr. Files must be regular
UTF-8 JSON, at most 4 MiB each; duplicate keys, unknown fields, nonfinite numbers,
wrong types and unknown versions fail. No network lookup or automatic migration.

## Version and integrity boundaries

The [neutral JSON Schema](../schemas/neutral-scenario-v1.json) uses
`scenario_version: 1`; the [registry schema](../schemas/capability-registry-v1.json)
uses `registry_version: 1`. These are independent of comparator `schema_version`,
provenance version, card manifest revision, and future action/observation/runtime
schemas. The schemas specify shape; [scenario.py](../scripts/scenario.py) also
validates references, source pins, constructor assumptions and coverage. It
implements only the explicitly used JSON Schema keywords and rejects unknown
keywords; it does not retrieve `$id` URLs or external schemas.

`rules` pins the exact rules version and raw-document SHA from the committed
[rules manifest](../data/rules/cr-2026-09-25.json). `cards` pins the exact manifest
`revision` and **raw file** SHA-256, including formatting. Per-card Oracle text
hashes cover every card used by the constructor. Unknown versions and mismatched
pins are errors; changing a source requires a deliberate fixture migration.
Reference source/card/bridge/patch pins are explicit metadata, not build receipts.
An unresolved reference should be absent from `reference_evidence`, never a
placeholder hash or a passing result. Later bridge tasks must resolve and verify
those external artifacts and report unavailability explicitly.

`provenance.fixture_revision` is SHA-256 of canonical UTF-8 JSON of the complete
fixture **with only that field removed**: sorted object keys, compact separators,
unescaped Unicode, no nonfinite numbers. `reproduction.trace_sha256` uses the same
encoding of the `script` array. These detect stale results and accidental edits;
a checksum does not establish correctness or authorship.

## Constructors and state

Exactly one constructor is selected by `setup.kind`:

- `synthetic` supplies `state` and nonempty `assumptions`. Every assumption has
  an original explanation and equality assertions that are evaluated against the
  supplied state during validation. Phase/step agreement, two distinct seats,
  object identities, unique zone membership, supported card definitions, token
  status, stack/effect references and control timestamps are checked. This is
  structural consistency plus declared assumptions, **not a reachability proof**
  or implementation of state-based actions. A future engine constructor must
  validate its additional semantic invariants before accepting the position.
- `normal_reset` supplies starting seat, two frozen deck IDs and explicit ordered
  libraries. Each is exactly the pinned 40-card multiset. No arbitrary state
  override is allowed: the future runner must use normal reset (20 life, seven
  cards, London mulligans). Both seats' opening choices are explicit. Every
  mulligan has an explicit replacement shuffle result with the full deck multiset.
  The runner must consume these in order without its own random shuffle or AI.
  Constructor selection alone is not evidence of a completed end-to-end game.

Seats are integers 0 and 1. Libraries are top-to-bottom; stack entries are
bottom-to-top. Synthetic zone arrays contain stable object IDs, not card names;
normal-reset library arrays contain pinned card IDs because objects do not yet
exist. Object records include card ID, generation, owner/controller, tapped flag,
controlled-since turn, damage, current power/toughness, keywords and token status.
Zone transitions create a new identity/generation in future execution. State also
records active player, turn/phase/step/priority, all five scoped player zones,
life, six mana counts, land plays, stack targets/modes and temporary effects.
Stack entries/effects carry explicit nullable `last_known_source` object records
when their original source identity has left its zone; this permits dead-source
abilities without pretending the new graveyard object has the old identity.
The core must never infer hidden libraries from a seed. Synthetic state and the
fixture itself are privileged test data, not policy observations.

## Explicit scripts and checkpoints

Every action has an ID, actor, semantic kind, explicit nullable source, and ordered
choices. Choice IDs are unique in the valid script. Values are semantic object/card
references (including `seat:0`/`seat:1` targets), mode IDs,
`{resource, amount}` payments (resource names identify a mana color or source),
`{blocker, attacker}` pairs, or `{source, target, amount}` damage allocations. No wildcard,
`auto`, AI or default decision is allowed. Relevant casts/activations must include
explicit empty arrays when a slot does not apply; absence is not a default.

| Action kind | Required ordered choice kinds |
| --- | --- |
| `cast` | `mode`, `targets`, `payment`, `discard` |
| `activate` | `targets`, `payment` |
| `target_trigger` | `targets`; `source` identifies the pending trigger |
| `play_land` | `land` |
| `attack`, `block`, `assign_damage` | `attackers`, `blockers`, `damage` respectively |
| `order_triggers` | `trigger_order` |
| `pass`, `keep`, `mulligan`, `bottom`, `discard`, `concede` | same named choice |

Pass/keep/mulligan/concede values must be empty. A cast must identify its source;
all choices retain the actor. `target_trigger` supplies targets when a pending
trigger is put on the stack; for Pyromancer this occurs after creature resolution,
not during casting. Its source is the stable pending-trigger ID. Non-targeted
triggers need no such action; controller ordering remains `order_triggers`.
This format describes complete logical commands;
a bridge may map them to different internal microsteps without adding priority
windows. Target/payment legality remains the rules engine's responsibility.
For example, an explicit empty target choice can be an intentionally illegal
command; omitting the target slot is a malformed script.

`ChoiceScript` is a small strict bridge utility: consume the next actor/kind,
return a copy of its explicit values, and call `finish()` at the declared end.
Wrong actor/kind, extra decisions and unused choices raise errors without advancing
on a mismatch. Static validation cannot predict engine decisions. A bridge must
match actions/choice IDs to its semantic decisions and enforce this consumption
contract; claiming script completeness solely from schema acceptance is invalid.

Checkpoints name a decision or settled state after `initial` or an action ID, in
script order. The final action must have a checkpoint. Assertions name a semantic
field, exact JSON Pointer path, independent basis ID and literal expected JSON
value. There are no computed expectations or wildcard comparisons. Supported
fields include zones, object identity, life, marked damage, power/toughness, stack,
priority, legal choices, player-visible information, mana, active player, status,
outcome, rewards and invariants. Life/damage are integer assertions, priority is
a nullable seat, stack/legal choices are ordered arrays, and structured fields
are objects. Paths refer to the future normalized checkpoint contract; adapters
must expose that field or report it unobservable, never silently ignore it.
Only synthetic assumption pointers are evaluated here; expected checkpoints are
never calculated by this validator.

Invalid-action probes are separate from the valid script. Each names its insertion
checkpoint, exact expected error, independently justified basis and **all four**
invariants: unchanged game state, RNG state, decision state and private information.
A runner must compare before/after snapshots even if the error code matches.

## Provenance and admission

All fields from [GH-11's provenance policy](provenance-policy.md) are represented:
origin kind/author/date/ancestry, exact sources and sections, license source hashes,
file/full notices and retained paths, modifications, rules/card/Oracle/ruling pins,
independent per-checkpoint derivations, vintage audit, reproduction, reference
results and admission review. Original code explicitly declares no adaptation;
adaptations need commit/notices/review and cannot import Forge tests under M0.
Regression records include defect, failing revision, trace, minimization, replay
hash and behavioral red/green; differential records name aligned inputs, both
references, first divergence and independently reviewed adjudication.

Review defaults are never inserted: the author must state `pending`,
`accepted-for-m0` or `blocked`. Admission requires accepted vintage review and
linked review evidence. Source excerpts, legal conclusions and expectations need
substantive independent review; text fields cannot mechanize that judgment.
Release blockers in the license record remain separate from scoped M0 admission.
The bundled [priority-pass example](../fixtures/scenarios/priority-pass.json)
is **pending**, original schema-test data. Its expectation is explained by CR
117.3d and RFC boundary requirements, not an engine result. It supplies no passing
capability evidence and does not replace GH-15's initial corpus.

## Coverage registry

The [registry](../data/capabilities-v1.json) maps the verbatim nine §3 supported
behavior clauses and every minimum behavior of all twelve §7 families to stable
capability IDs. It also retains all 21 reserved card/token behavior IDs from the
card manifest (20 card definitions and the Goblin token). There are 80 entries,
each requiring positive, negative, interaction and regression evidence: 320 slots.
The validator independently reads the RFC and card manifest; removing a family,
clause, mapped capability or evidence category fails. IDs should be retained across
schema migrations; semantic changes require explicit review and evidence renewal.

All entries currently say `implementation: planned` with empty evidence arrays.
This is the scoped delivery target, **not advertised engine support**. An
`implemented` entry requires passing evidence in every mandatory category.
Evidence links a supplied, admitted neutral fixture by exact content digest and
capability, engine, exact engine commit and result artifact URL/digest. Planned
entries cannot contain execution artifacts; executed, passed, failed, unsupported
and unavailable remain distinct counts. These are validated evidence records,
not executions performed by this tool; external receipt contents must be audited
by the runner/reviewer that admits them. Multiple evidence categories may refer
to the same case only where independent review establishes each claimed behavior.

Supply all referenced fixtures to `coverage` when adding evidence. `--require-passed`
rejects every missing mandatory slot. It does not certify reference-release floors,
mutation sensitivity, independent corroboration of adapted tests, or game reachability:
those later RFC gates still require ≥100 reviewed cases, XMage coverage of every
capability/card, ≥20 independently authored dual-reference critical scenarios,
scripted matchups, explicit observable-field gaps and adjudicated disagreements.
No counter is a substitute for those obligations.
