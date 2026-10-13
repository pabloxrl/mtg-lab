# Forge casting prefixes (L0-008)

The test-only consumer extends recorded openings into land plays, priority passes,
creature casts, basic mana payments, targets, instant responses, modal selection,
spell-created tokens, additional discard costs and resolution checkpoints. Forge
and native independently compute the transitions from normal forty-card setup.
No game state is injected and tactical AI is not rerun during replay.

## Run

Prepare an isolated external Forge cache using the unchanged
[capture preparation commands](../forge-ai-capture/README.md). Each invocation
requires a new output directory. Inside the managed container:

```sh
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- \
  python3 scripts/forge_casting.py --cache /home/agent/.cache/forge-GH-326 \
  --trace doc/evidence/forge-casting/source-gg-11/result.json.jsonl.gz \
  --empty-combat --output .agent-artifacts/forge-casting-response-example
```

For the token-mode/discard case, use
`doc/evidence/forge-casting/source-rr-3/result.json.jsonl.gz` with
`--empty-combat --stop-before 2984`. That endpoint is the first empty-stack
priority checkpoint after the first Thrill of Possibility resolves. The remaining
original records are retained with their count and uncompressed-byte digest.
`--stop-before` must identify an exact root callback after settled opening and
cannot cross an unsupported callback. Omitting it stops at the first unsupported
callback. `--empty-combat` consumes only recorded empty attacker selections;
nonempty combat remains unsupported.

The command verifies the complete immutable source trace, receipt, implementation
and source/card/rules pins. It executes one actual unskipped Forge test and then
the independent native test. `casting-prefix-agreed` certifies only that bounded
prefix. The ordinary suite discovers all source-derived native fixtures and the
Python reconstruction/provenance regressions. It does not invoke Java or AI for
these new fixtures. The live commands are additional delivery acceptance.

## Cases and boundaries

| Source | Starter | Source endpoint | Native actions / checkpoints | Witness |
| --- | --- | --- | --- | --- |
| Existing red/green seed 1 | 0 | 1005 | 75 / 65 | Lands, creatures, ordered mana and passes |
| Red/green seed 3 | 1 | 1128 | 105 / 84 | Bite Down's two targets and resolution |
| Green/green seed 11 | 0 | 1156 | 113 / 89 | Giant Growth responds to Bite Down; stack and +3/+3 effect agree |
| Red/red seed 3 | 1 | 2984 | 254 / 192 | Dragon Fodder, Goblin Surprise's token mode, Thrill's selected Mountain discard and draw |

Both engines consume exact choices; Forge also consumes every source callback,
random draw and rules event up to the endpoint. Native priority checkpoints compare
actor/turn/step, life, ordered libraries and graveyards, hand/battlefield/exile
membership, physical identities and relative incarnations, stack/targets/modes,
mana, controllers, tapping, sickness, effective characteristics, damage and boosts.
Native costs preserve each recorded basic-land activation followed by its matched
mana payment, including the source ability and physical land identity. Floating,
multi-unit, restricted and nonbasic mana payments are explicit unsupported cases.
Forge compares the richer event, resolution and SBA observations directly.
Expectations come from the immutable source observations, never native output.

Unconsumed nonempty combat, standalone activations, trigger ordering and
later callbacks are explicit unsupported boundaries. Native casting acceptance
rejects pending/placed triggers, including Forge's trigger-dispatch callback whose
internal waiting list has already been transferred into its arguments. Empty
combat handling does not implement attacks, blocks or damage. Live resolved-modal
coverage here is the token mode; a retained PumpAll source probe reaches an
unsupported trigger boundary. No complete-game replay, pilot, corpus admission,
1,000-game quota, production L0 runner or milestone completion follows from these
prefixes. A fifth native fixture checks `--stop-before 269` on existing seed 1:
settled opening with zero gameplay choices. Its applicable checkpoint corruption
still runs; nonempty fixtures retain every applicable choice corruption. Existing
decks, definitions, source pins, exceptions and regressions stay unchanged.

## Comparison of source allocation identifiers

Skipping AI simulations changes allocation counters; their numeric values are
not portable rules identities. The bridge uses stable bindings, with bijections
for card incarnations and effect timestamps, and checks every rules-bearing field:

- Physical IDs of the original eighty cards remain exact. Newly created Goblins
  bind by observed creation order, count, owner/controller and definition. Their
  physical binding then persists through all references.
- Runtime/source timestamps bind per physical card, including current, transient
  zone-change and last-known objects retained in stack/event observations.
  Selected choice results never create these bindings, so stale target references
  remain invalid. Native uses relative zone incarnations from settled opening.
- Mode clones and power/toughness effect allocation timestamps bind separately.
  Selected modes, target objects, effect amounts and effective characteristics
  remain compared. The bridge never writes expected state into either engine.
- Historical `paying_mana_abilities` retain mutable live abilities. In existing
  seed 1, records 568/580 identify Forest ability 153's actor as seat 1; record
  588 changes it to seat 0 after opponent AI cost probing, with controller still
  seat 1. Only that historical reference's `actor` is excluded. Committed cost
  callback actors, mana lots, sources/controllers and the enclosing spell actor
  remain checked.
- Forge `TokenInfo.getProtoType` pins a PaperToken edition on the first prototype
  query, including AI simulation queries. The source red mirror selected 10E;
  replay without those probes selected FDN. Only token printing `edition`,
  `collector_number` and `art_index` are excluded as presentation metadata.
  Token name, definition/characteristics, identity and every effect remain checked;
  original-card printings remain exact. Original raw metadata is unchanged.

These explicit representation bounds are part of independent review. Live
`token-power` and `effect-amount` corruptions specifically guard against hiding
rules differences behind normalization. Other `--negative` controls are
`target-incarnation`, `payment`, `pass-actor`, `callback-order`, `response-order`,
`mode` and `discard`. Each mutates only validated consumer input and requires an
actual engine rejection, never a successful gameplay receipt. Native also checks
invalid choices/order and state mutations on every ordinary fixture invocation.

## Frozen source matchup selection

The additive `scripts/forge_casting_capture.py` accepts `--decks green,green`,
`red,red`, `red,green` or `green,red`, plus the original cache/output/seed options.
It selects existing frozen deck definitions and retains the original capture
bridge, source and AI. Its `frozen-deck-selection-v1` receipt binds the original
manifest, derived seat-selection manifest, additive runner and original runner.
Validation rejects any other manifest change. Original capture receipts still
require their original runner and red/green pair. The opening command also uses
the validated selected manifest when executing this profile.

`--uninstrumented` is an original-source audit control, not a recorded game.
The green seed-0 record/control audits agree exactly. Source screening used the
existing callback cap; a truncated screen found seed 11's response, followed by
a separate complete original-AI capture before replay. Failed, truncated and
unused attempts remain distinct; none counts as an admitted corpus game. An early
green seed-0 capture used the runner before the audit CLI option was added; its
original runner bytes and later pin rejection remain in the private history.
The published seed-0 comparison uses a fresh capture with the final runner.

## Evidence and provenance

The [execution summary](execution/summary.json) records eight positive runs and
nine live mutation controls, with identical Forge/native JSON between repeats.
Each run includes pinned receipts, semantic consumer input, independently computed
results and the actual unskipped Forge test summary. The [full torture receipt](validation/torture.json)
and [compressed log](validation/torture.log.gz) record 508 Python tests and 1,530
Rust test executions across debug/release, with zero failures or ignored tests.
The [source attempt inventory](source-attempts.json) accounts for 56 attempts
(51 complete, including the uninstrumented control; five truncated), with zero
admitted games. [Capture/control equality](capture-control/comparison.json),
[selected-deck opening](execution/selected-opening/receipt.json), and
[zero-action boundary](execution/empty-boundary/receipt.json) are retained separately.
The [artifact index](artifact-index.json) hashes every published evidence artifact;
the [private history index](attempt-index.json) locates and hashes all retained
local attempts through final torture, before independent review. The [workpad](https://github.com/pabloxrl/mtg-lab/issues/326#issuecomment-6136832842)
records full locked torture, exact-candidate review, protected merge and exact-main
CI; these gates remain pending until their linked results pass.

Original adapter source, semantic captures and metadata follow the existing
[provenance policy](../../provenance-policy.md). Captures are privileged evidence,
not player-observation datasets. Forge source/card scripts/data, compiled bridge and
linked GPLv3 runtime remain separately installed; no runtime or artwork is
redistributed. Event prose is hashed by the unchanged recorder. Private JVM logs,
failed probes and unused source attempts are retained outside workspace cleanup
in the issue-specific managed cache with an indexed hash inventory, following the
existing capture retention practice. That cache is not delivery acceptance
proof; published selected source tapes, execution receipts and CI/review evidence
are. No release-wide licensing clearance is implied.
