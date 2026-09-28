# Bite-killed blocker through the shared script (GH-138)

One original five-object script extends the [shared instant executor](../instant-reference/README.md)
from turn-one upkeep through combat damage. The same setup and ordered choices
run in native Rust and pinned XMage. No production rules code changes.
This is partial R0002-B017/B026/B029 evidence; #115/#18, M1 qualification,
cleanup, Forge, full-pool and dual-reference/full-game release gates remain open.

## Independent expectations

The [shared input](../../../fixtures/reference/instant-responses.json) case
`bite-killed-blocker` and [literal checkpoints](../../../fixtures/reference/instant-expectations.json)
were authored before adapter changes, using the pinned [CR](../../../data/rules/cr-2026-09-25.json)
and [Oracle manifest](../../../data/cards/foundations_micro_v1.json).
No engine output or upstream test expectation generated these outcomes.
Bear Cub is 2/2; Bite Down costs {1}{G} and makes its selected creature deal
its power in damage to the selected opposing creature.

Synthetic setup is exactly two untapped, undamaged Cubs controlled since before
the turn (one per seat), P0's untapped Forest and Mountain, and P0's Bite in hand.
Both players have 20 life and zero mana; libraries and all other zones are empty.
Turn one starts at P0 upkeep priority. Hands are installed at that exact XMage
boundary, avoiding hidden mulligan choices. The first player's draw step is
skipped under CR 103.8a; there is no draw, shuffle or chance in this script.
The native synthetic RNG uses `splitmix64-v1`, seed/episode 0; no random result is used.

CR 117 requires explicit passes through upkeep, precombat main and beginning of
combat. CR 508/509 require the P0 attacker and P1 blocker declarations, then
priority windows. P0 taps Forest/Mountain, casts Bite targeting the attacking Cub
and blocking Cub, pays green/red explicitly and finishes casting. P0 passes to
P1, who passes to resolve. The actual two damage kills the blocker under
CR 704.5g; its graveyard incarnation is a new object under CR 400.7.
**CR 509.1h preserves blocked status; CR 510.1c gives a blocked attacker with no
remaining blockers no combat damage assignment. Both life totals remain 20.**
Two more explicit passes reach combat damage; no cleanup is executed.

Nine settled checkpoints cover initial setup, main, beginning combat, attackers,
blockers, Bite on stack, defender response window, blocker death and combat damage.
Every checkpoint compares actual step/active player/priority, life/mana, complete
named inventory and zones/incarnations, taps, creature stats/damage, stack order
and historical target identities, damage events, last resolution, committed
attacker identity, surviving blocker identities and actual remembered blocked flag.
The flag comes from native `combat()` and XMage `CombatGroup.getBlocked()`,
never from counting surviving blockers or from the expected fixture.
Existing instant/departure cases now additionally require empty combat at all
of their checkpoints; their prior assertions remain intact.

## Strict translation and regression evidence

[red.log](red.log) retains the compiling baseline failure at the newly required
precombat-main checkpoint, where the old executor asserted upkeep only.
Native normal discovery retains the complete literal script, every omitted or
duplicated choice/checkpoint, illegal/duplicate/foreign attackers and blockers,
nonattacking block targets, wrong declarers and reordered response passes.
A [compiled native mutant](compiled-mutant.json) replaces the remembered flag with
a check for surviving blockers. The real shared script first diverges at
`blocker-died.combat[0].blocked`; production source is restored before acceptance
and full torture. [The observed mutant](forgotten-blocked-native.json) and its
[execution log](forgotten-blocked-native.log) are retained.
Python normal discovery detects forgotten blocked status, stale blocker membership,
player damage, wrong target/step, missing combat observations and consumed-script
corruption, alongside all existing unavailable-reference and comparator tests.

[illegal-attacker-red.log](illegal-attacker-red.log) records the first reference
control exposing that XMage `canAttack` alone is not its complete candidate
filter. The adapter now also requires membership in XMage
`getAvailableAttackers`; it rejects a land at the declaration itself.

Both adapters consume explicit declaration arrays and validate them with their
own engine. No AI declaration or response fallback is used. The native core
exposes a damage-completion boundary even when there is no allocation choice;
the translator completes it only after proving its allocation list is empty.
XMage reaches the equivalent damage boundary automatically. Any actual allocation
choice is rejected as unsupported, rather than auto-selected. This is a bounded
vanilla declaration/response extension, not a general combat harness.

The real reference runner also executes minimized illegal/duplicate declarations,
an omitted declaration, and omitted/reordered response controls in both engines.
Only the intended diagnostic counts as rejection; a build failure, missing
reference or timeout cannot pass. First-divergence artifacts and mutation receipts
are retained with [acceptance.json](acceptance.json).

## Reproduction, provenance and limits

Inside the managed Linux ARM64 Docker worker (no Docker invocation or socket),
copy the image cache only if `/tmp/mtg-xmage` does not already exist. Otherwise
reuse the prepared cache and start with the reference command:

```sh
cp -a /home/agent/.cache/xmage /tmp/mtg-xmage
python3 scripts/instant_reference.py --cache /tmp/mtg-xmage --output /tmp/blocker-acceptance
cargo test -p mtg-core --lib instant_
python3 -m unittest discover -s tests -p test_instant_reference.py
./scripts/torture.sh
```

The reference command executes all seven shared cases twice per engine and
checks every literal checkpoint and consumed action before issuing success.
It separately executes the existing instant/departure controls and new blocker
controls. Actual source/archive, Java/Maven and dependency hashes are verified;
execution is offline, headless, stdin closed and bounded by subprocess timeouts.
The retained JSON receipts and new control logs are committed here; other raw
build/control logs remain in the output directory with hashes in the receipt.
The receipt pins source/card/toolchain/dependencies, scenarios, expectations,
bridges and native source hashes. Additional consulted combat APIs are hashed in
[XMage pins](../../../references/xmage/pins.json); its [MIT notice](../../../references/xmage/UPSTREAM-LICENSE.txt)
is retained. The scenario, expectations and bridge extension are original
project-authored material, not adapted upstream test cases. No Forge code or
new rules/card text is redistributed; repository release-license questions remain.

Observation does not cover legal-action enumeration, private player views,
arbitrary damage allocation, zone ordering, all keywords, cleanup or completed
games. The damage-event stream covers permanent damage, while player damage is
checked through exact life totals. Synthetic upkeep setup is not evidence of
normal reset/reachability or full-game execution. Prior departed-target reentry
coverage remains explicitly a native observer test, not a matched reanimation spell.

Full torture, separate candidate review, protected merge and exact-main CI are
recorded in the [issue workpad](https://github.com/pabloxrl/mtg-lab/issues/138).
The root README reference row links this bounded evidence; milestone status is
unchanged. Delivery is conditional on all of those gates passing.
