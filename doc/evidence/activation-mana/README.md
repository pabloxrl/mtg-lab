# Activation-payment mana choices

GH-254 repair candidate; verification and protected delivery are pending.
This is not completion of the GH-23 mechanics audit or the M2 gate.

CR 602.2b applies 601.2g/h to activated abilities; 605.3a permits mana abilities
inside payment without using the stack or passing priority. Pinned Shivan is a
5/5 flying creature whose R ability gives +1/+0. Pinned Invoker's eight-generic
ability gives a target creature +5/+5 and trample: a Bear Cub becomes 7/7.
CR 302.6/702.10 restrict sick Elf/Druid tap abilities unless they have haste;
Shivan/Invoker's nontap costs do not have this restriction. CR 113.7a/611.2 and
object incarnation rules govern source independence and temporary effects.
The [card and rules pins](../../card-manifests.md) remain unchanged.

## Reproductions and acceptance

The three [original compiled failures](original-red.log) come from the unchanged
[archived GH-23 patch](../m2-repair-registration/diagnostics/GH-23/activation-mana-red.patch).
The [additional compiled failures](protocol-red.log) cover private cancellation,
Invoker target/source timing, both native policy adapters, typed recorder command
validation and normal-reset Shivan/Invoker played recordings. Build errors are
not counted as behavioral evidence. These tests remain in normal discovery.

Synthetic core positions separately cover source eligibility, private staging,
surplus, cancellation, stale/wrong actor/zone/source/color/payment rejection,
incarnation revalidation, overflow, semantic actions and pending snapshots.
An independent bounded oracle enumerates source subsets and exact R consumption;
it does not calculate expected outcomes using engine legality or payment helpers.
Played Driver/Run tests use real ordered-deck reset, cast the creatures, choose
sources during activation payment and check capture on/off, quantum 1/unbounded,
semantic replay, typed conversion and JSONL persistence. Existing floating-first
scripts still execute separately with their original choices and expectations.

## Compatibility inventory

| Contract | Decision |
| --- | --- |
| Native policy algorithms | New `legal-random-activation-mana-v1` and `heuristic-activation-mana-v1` IDs. Reject the prior full-pool IDs; RNG primitive/version is unchanged. Legal choice sets and seeded histories change. |
| Policy observation/submission schema 1 | Existing TapMana variant and activation_payment kind. The acting player's existing optional pending pool/remaining fields now describe activation payment; sources retains the activation source first, then selected mana sources. No opponent pending fields. Live revision/generation checks remain mandatory. |
| Semantic action version 1 | Existing TapMana with semantic object birth/incarnation and activation_payment decision kind. Float-first records retain their meaning; no numeric command is reinterpreted. |
| Snapshot envelope 1 / internal continuation | PendingActivation gains required sources, without a deserialization default. Conservative source fingerprint changes and rejects older engine snapshots before decoding. No migration or cross-engine snapshot compatibility claimed. |
| Played replay / Driver replay | Existing engine provenance rejects a different source fingerprint. Compatible actions retain their semantics; old runners and durable evidence remain pinned. |
| Typed trajectory / JSONL schema 2 | Existing TapMana and pending fields are used, including Continuing status within payment. Existing typed fixtures remain readable and roundtrip unchanged; this is additive validation, not a replacement default. |
| Simulation schema 2 / benchmark inputs | Native policy identifiers are updated explicitly. Old configurations fail unsupported-policy validation rather than silently choosing the new strategy. |

## Reference checks

The existing Shivan/Invoker fixtures gain an additive `payment_sources` case.
Their original float-first cases and literal expectations remain unchanged.
Pinned XMage invokes actual land mana abilities inside `playMana`, after target
selection. Each source alias is explicit (`pay1` through `pay8` for Invoker).
Checkpoints export the selected target, source alias, resulting mana, tap count,
announced stack size and priority callback delta; final stats are also compared.
The native private transaction normalizes its pending ability as the announced
stack object and reserved taps as paid tap costs. Native public taps/pool remain
unchanged until commit; this normalization is explicit, not claimed public-state
identity during an incomplete action. No mana ability adds a stack object.

These are synthetic selected-checkpoint reference cases, not full reference games.
Normal-reset played recording evidence is native and separately identified.
No Forge, reference-AI corpus, general ability language or layer-system claim.

Run the existing reference commands using this issue's private cache and the
shared heavy lock, then run the complete torture suite:

```sh
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- python3 scripts/shivan_reference.py --cache "$MTG_REFERENCE_CACHE" --output .agent-artifacts/shivan-reference
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- python3 scripts/invoker_reference.py --cache "$MTG_REFERENCE_CACHE" --output .agent-artifacts/invoker-reference
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- ./scripts/torture.sh
```

Twice-executed [Shivan](shivan/receipt.json) and [Invoker](invoker/receipt.json)
receipts retain exact fixture/expectation, bridge, toolchain and native source
hashes, consumed choices and first-divergence negative controls. Checkpoint exports
are retained alongside each receipt. Final source-matched reruns are pending.

The full-suite insufficient-mana fixture correction has [explicit independent
review](expectation-review.md) and stronger mixed-payment regression coverage.
Full torture, exact-candidate review and protected merge/exact-main CI: pending.
