# Ordinary Sentry/Bite shared reference inputs

Status: real native/XMage acceptance PASS for this bounded GH-256 extension.
Full torture passed before an independent main update (227 Python, 1,348 Rust
executions, zero failed/ignored); integrated revalidation, independent review
and protected delivery are still pending;
no aggregate-audit or M2 completion claim.

## Independent oracle and unchanged scope

This original test-only extension uses the pinned
[card manifest](../../../data/cards/foundations_micro_v1.json) and
[CR revision](../../../data/rules/cr-2026-09-25.json): Bite Down deals the controlled
creature's current power to a creature the caster does not control; it is not
fight. Bear Cub is 2/2, Magnigoth Sentry is 4/4, and Giant Growth gives +3/+3 until
end of turn. CR 608.2b/h supplies resolution-time legality/information, CR 120.6
separates marked damage from toughness, CR 400.7 separates zone incarnations,
and CR 514.2 removes damage and expires the boost at cleanup.

The [archived 31-case diagnostic](../m2-repair-registration/diagnostics/GH-212/prerequisite-audit.json)
and [gap report](../m2-repair-registration/diagnostics/GH-212/prerequisite-gap.txt)
remain unchanged. Six cases are expressed by the existing strict instant bridge;
GH-212 retains all 31-case aggregate acceptance. No production rules change,
source repin, Forge claim, keyword substitute, normal-reset reachability or
full-game certification is included.

| Exact catalog ID | Independently authored result |
| --- | --- |
| rules-foundations_micro_v1-bite-down-positive | Own 2/2 Cub deals two to opposing Sentry; 4/4, damage two, Cub damage zero. |
| rules-foundations_micro_v1-bite-down-negative | Own Cub to own Sentry is rejected; mana, targets, zones, identities and damage history remain unchanged. |
| rules-foundations_micro_v1-bite-down-regression | Responding Bite from Sentry kills the selected source Cub; the lower Bite retains its old source target, resolves with only the destination legal, and deals no damage. |
| rules-objects-bookkeeping-positive | The surviving damaged Sentry remains 4/4 with two marked damage. |
| rules-objects-bookkeeping-interaction | After Bite, Growth makes Sentry 7/7 with two damage; next upkeep shows 4/4 with zero. |
| rules-continuous-resolution-power-positive | Growth above Bite makes source Cub 5/5; its five damage kills Sentry. |

The JSON expectations are literal authored checkpoints, not exported native or
XMage results. Existing eleven shared scripts and expectations are retained.
The native compiled [red log](red-native.log) records rejection of the new Sentry
input at the adapter allowlist, before its extension. A build error is not red
behavioral evidence. The subsequent [real reference red receipt](red-reference/red.json)
records a successful compile, eleven passing legacy cases and six name-map
assertion failures at `InstantResponseTest.java:284`; its exact input, native
observations and Java log are retained.

Setup is explicitly synthetic turn-one upkeep with empty libraries, declared
hands/battlefield, zero mana, 20 life and no chance choices. Forest/Mountain
activations, both target groups, payment, passes, response order and empty
attack declarations are explicit. The normal first-turn draw is skipped; cleanup
coverage stops at next upkeep before drawing. Same-name decoys in the source
removal case distinguish physical identity. Reentry remains the inherited
observer-only native regression, not a new matched reanimation capability.

The negative catalog setup contains no opposing creature. Native casting rejects
at admission before a target/payment continuation. The reference must verify the
same illegal target pair and preserve the observed state across rejection.
No legal-action enumeration, hidden views, general combat, cleanup triggers or
extra cleanup is advertised. History here means observed damage/resolution and
stack targets; native snapshot equality additionally checks rejected-boundary
state. Reference observations do not certify every private internal field.

## Reproduction

Use the managed image's Rust/Java/Maven/Python tools and this issue's isolated
`MTG_REFERENCE_CACHE`, prepared from pinned build inputs. Heavy commands run
through the shared lock:

```sh
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- python3 scripts/instant_reference.py --cache "$MTG_REFERENCE_CACHE" --output /tmp/sentry-acceptance
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- ./scripts/torture.sh
```

Normal discovery includes the shared native inputs, strict consumption tests and
Python comparator controls via `cargo test --workspace --locked` and
`python3 scripts/run_tests.py`. Real reference receipts must retain all seventeen
cases twice per engine, original M1 controls and new Sentry controls, source,
bridge and toolchain hashes, exact inputs and first divergence. Missing output,
unavailable toolchains and build failures are failures, never agreement.

## Executed reference acceptance

The [receipt](acceptance/acceptance.json) records all seventeen shared cases
(original eleven plus six new) agreeing with literal expectations in both engines,
twice. There are no skipped, missing or disputed cases in this bounded pack.
The Bite-positive and bookkeeping-positive catalog slots intentionally share the
same play; these counts are named cases, not a distinct-scenario-floor claim.
Five new malformed one-case inputs reject omitted/extra/reordered/wrong-actor and
friendly-destination choices in both real adapters. A real same-name Cub retarget
fails first at the responding spell's target identity; nine mutated observations
fail at the named power, damage, cleanup, historical identity or stack field.
All inherited choice, departed-target, combat and cleanup controls also pass.
Exact inputs, consumed transcripts, observed checkpoints, failures and minimized
inputs are retained alongside the receipt. Its pins match the candidate's native
sources, shared JSON corpus, bridge, runner/helper, rules/cards and dependency
locks. Source and toolchain identities are verified before reference execution.

The ordinary negative case performs a real XMage cast attempt; the engine rolls
it back. Exported states before/after match, and the original spell retains no
selected targets. Native admission rejects before staging and preserves its
snapshot. The two APIs differ in where they reject the same illegal command;
neither commits mana, targets or zones. The callback translator supplies only
that command's declared targets during the attempted XMage cast.
