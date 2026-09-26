# M0 authored corpus and checkpoint comparison

GH-15 delivers original rule-derived scenarios and an offline strict comparator.
It does not implement game rules. CR September 25, 2026 was fetched and verified
against `8d860e451f20f38865b725b42d82feb714c725373dd8f3b32b8652b3eeb070ca`.
Expectations are authored from numbered rules and pinned card definitions, never
regenerated from the engine under test. The required separate candidate review
and its findings/resolutions are recorded in [GH-15](https://github.com/pabloxrl/mtg-lab/issues/15)
and its PR before admission by merge.

## Authored cases

| Fixture | Intermediate expectations and independent basis |
| --- | --- |
| [Empty stack](../fixtures/scenarios/m0/priority-empty.json) | First pass transfers priority; second ends postcombat main and offers active-player priority in end step (117.3a/d, 117.4). |
| [Responding Growth/Bite Down](../fixtures/scenarios/m0/growth-bite-cleanup.json) | Explicit Forest mana activations, costs, targets and passes. Caster retains priority; Growth resolves first to 5/5 with Bite still on stack; Bite deals 2; cleanup leaves the same 2/2 with zero damage (117, 601, 605, 608, 514.2, 704.5g). No lethal intermediate state between expiration and damage removal. |
| [Bite target restrictions](../fixtures/scenarios/m0/targeting-bite.json) | Reversed/missing targets reject atomically, with all four invariant snapshots; own then opponent is valid (601.2c/e, 733.1). |
| [London mulligan](../fixtures/scenarios/m0/mulligan.json) | Normal reset with frozen 40-card decks and replacement order. Both start at 20/seven; redraw seven, bottom one, then keep six. First player's draw step is skipped (103.4, 103.5, 103.8a). |
| [Privacy A](../fixtures/scenarios/m0/privacy-a.json), [Privacy B](../fixtures/scenarios/m0/privacy-b.json) | Same P0 observation, ordered semantic candidates, mask and wrong-actor error despite different unknown P1 identities (400.2, 401.2, 402.3; RFC observation contract). Future outcomes are not asserted equal. |

The six fixtures have 18 named checkpoints. Synthetic positions explicitly
make no reachability claim. No triggers, extra turns, hidden choices or unstated
random operations are assumed. The cleanup checkpoint is an internal settled
rule-level hook before advancing to the next turn, not an extra player decision.
For reset, `initial` is after opening draws but before declarations. `keep-1`
ends the first declaration round, so the following checkpoint observes P0's
redraw before bottoming. Each mulligan includes its bottom choice **before** the
next keep/mulligan round; the earlier schema validator incorrectly required
keep before bottom and is corrected here against 103.5.

## Canonical observations

Run validation and comparator self-tests without a production engine:

```sh
python3 scripts/scenario.py validate fixtures/scenarios/m0/*.json
python3 -m unittest discover -s tests -p 'test_checkpoints.py'
python3 -m unittest discover -s tests -p 'test_corpus.py'
```

The new test-only [comparator](../scripts/checkpoints.py) accepts the neutral
fixture and a supplied observation document. Rust comparator v1 and all its
self-tests remain unchanged. This separately versioned contract extends coverage
to **every declared assertion field**, including nested state, targets, choices,
private views and missing values:

```text
python3 scripts/checkpoints.py fixture.json actual.json --artifacts new-directory
```

Actual JSON has exactly these fields:

- `checkpoint_version`: integer 1; `fixture_id` and `fixture_revision`: exact fixture pins.
- `initial`: independently observed canonical initial object. Synthetic setup fields
  are compared in full; additional top-level projections may be supplied for
  assertions. Normal-reset initial expectations are the fixture's named assertions.
- `consumed_script`: actual semantic action/choice transcript, same shape as `script`.
  Adapters must record consumed decisions, not echo the requested script.
- `checkpoints`: ordered `{name, after, kind, state}` records, exactly matching the
  fixture's checkpoint metadata. A checkpoint after `initial` must equal `initial`.
- `invalid_results`: ordered `{action, at, error, before, after}` records for every
  invalid probe. Both snapshots have exactly `state`, `rng`, `decision`, and
  `private_information`; all four must remain identical. `state` is a complete validated neutral v1
  state. `rng` is `{algorithm, state_hex}` with nonempty serialized bytes.
  `decision` is `{id, actor, kind, candidates}` with a concrete nonempty candidate
  list. `private_information` is `{views}` with both ordered
  `{seat, observation}` records; each observation includes typed `own_hand`,
  `opponent_hand_count` and two `library_counts`, plus any other observed fields.
  Null/empty/unavailable snapshots are rejected. Adapters must capture these
  fields from execution; the comparator cannot authenticate fabricated evidence.
  Probes are isolated
  insertions at the named point, not part of the valid action transcript.

Pointers resolve against `state`. Standard setup fields use neutral scenario v1.
The M0 fixtures additionally declare `/characteristics/<id>` (power/toughness),
`/damage/<id>` (integer), `/identity/<id>` (id/generation/zone), `/views/<seat>`
(exact permitted view), `/legal_choices/<seat>` (ordered semantic commands),
and reset `/hands`, `/hand_ids`, `/libraries` (seat-indexed arrays). Library
and hand card-name arrays use pinned card IDs, not Oracle text. Object aliases
are adapter-normalized from setup IDs; each zone change increments `.gN`, so
`bite.g1` is the stack object after casting. Stack entry IDs are casting action
IDs. Mulligan redraw aliases are `p<seat>-m<round>-<draw-index>`, zero-based.
These test projections are privileged data, not a policy API or production ABI.
Adapters must never compute expected game state. A missing/unobservable declared
field fails comparison; it cannot be omitted or reported as agreement.

Envelope/version/order errors exit 2; mismatches exit 1; comparison agreement
exits 0 with scope `supplied-checkpoints-only`. Diagnostic precedence is initial
setup, consumed transcript, checkpoints in fixture order/assertions in declared
order, then isolated invalid probes. Within a structured assertion, sorted object
keys and array order identify the first differing leaf. JSON boolean and numeric
types differ. `expected_present`/`actual_present` distinguish missing from null.
Whole asserted objects reject extra fields, detecting private-information leaks.
Unasserted top-level checkpoint projections are not a full-state verification claim.

The required fresh artifact directory receives `fixture.json`, `actual.json`,
and `diff.json`, with paths in stdout. These are **privileged test artifacts**;
keep them out of player-visible output and do not publish unchecked payloads.
Existing directories are rejected to preserve prior evidence. Parsing/shape
errors are structured failures, never passing cases. Inputs use the existing
4 MiB, duplicate-key/nonfinite-rejecting JSON loader. No stdin, display or engine
is needed. A copied expectation is useful only as comparator self-test data,
not as reference or production execution evidence.

## Coverage accounting and provenance

| Population | Authored | XMage executed | Forge executed | Production executed |
| --- | ---: | ---: | ---: | ---: |
| New GH-15 originals | 6 | 0 | 0 | 0 |
| Existing GH-13/14 priority smoke | 1 | 1 | 1 | 0 |
| Selected XMage adaptation candidates | 2 methods, not fixtures | 0 | 0 | 0 |

Existing execution receipts remain in [XMage](../references/xmage/evidence.json)
and [Forge](../references/forge/evidence.json), with explicit field limitations.
No new fixture is supported by those smoke-only bridges. This is an authoring
boundary, not a missing-engine pass or a skipped advertised capability. The
capability registry remains planned; `coverage --require-passed` still fails.
M1 owners must implement and execute matching rules/bridge cases test-first;
M2 and M5 retain broad corpus, independent holdout and dual-reference obligations.
No release floor (100 reviewed cases / 20 independent dual-reference cases) or
milestone completion is claimed here.

[XMage candidate records](../fixtures/xmage-candidates.json) pin exact files,
methods, receipts, modifications and notices. Two London methods apply; the
Thorn Elemental damage and arbitrary resolution-time targeting examples are
excluded with reasons. Combat audit explicitly checks current 510.1c/d,
702.19 and 702.2: no legacy blocker-order decision/response window is imported.
Upstream vintage is undeclared and reconciled against the pinned current rules.
The [full MIT notice](../references/xmage/UPSTREAM-LICENSE.txt) is retained;
file notices are preserved in the candidate records. No upstream Java tests
are copied or translated in these six original fixtures. The source inspection
also corroborated the separately rule-derived London validator repair.
An XMage adaptation is never independent corroboration by XMage. Forge tests
remain external; REL-01/02/03 public-release obligations remain unresolved.

## Detector evidence

The initial permissive comparator stub ran 13 tests with **31 assertion failures**,
not import/compile errors. Green uses identical expectations. Controls cover wrong
starting life, target, consumed choice, priority checkpoint, hidden extra field,
missing-versus-null, boolean-versus-number, unused choices, misaligned checkpoints,
all 15 declared field kinds, first-divergence precedence, and CLI artifact contents.
Additional red/green checks cover initial observation projections and London's
bottom-before-next-declaration order. Invalid-probe checks exercise every unchanged
snapshot category. These are verifier/validator tests, not mutant game-engine runs.

Independent review round 1 found that null invalid-action snapshots could agree.
Four rejection regressions failed before concrete snapshot validation was added;
state shape, RNG bytes, decision identity/candidates and both private views are now
required. The finding and final review remain preserved in the PR.
