# Exact Cub and single-Sentry oracle (authored before adapter changes)

The eight `exact_` rows appended to `fixtures/reference/shivan.json` preserve the
23 prior rows and their literal expectations. The matching expectations were
written before changing either adapter. Seed 200 is a native setup seed, not an
assertion of shared random streams. These are explicit synthetic main-phase
positions with real pinned cards, not full games or production rules changes.

The original catalog remains unchanged. P0 `cub_cast` executes
`rules-combat-creature-abilities-positive`: hand plus R/G (R pays the generic
one), empty stack, cast and resolve. CR 601 requires spending both mana and
moving hand → stack → battlefield; CR 302.6 distinguishes untapped from newly
controlled and prohibits attacking. `cub_reject` executes the negative slot:
P0 passes, P1 explicitly attempts its non-flash Cub, preserving state.
Seat-reversed rows use the same rules with the active player exchanged.

`single_sentry` executes `rules-foundations_micro_v1-magnigoth-sentry-interaction`:
real Shivan Dragon (5/5 flying), one real Magnigoth Sentry (4/4 reach), block,
Giant Growth (+3/+3), resolve to 7/7, then simultaneous combat. CR 509/510 and
702.9/702.17 require five damage on the surviving Sentry and lethal seven from
Sentry to Dragon; neither player loses life. The extra Cub control rejects its
attempt to block flying and permits five unblocked player damage. No activation
or multiple-blocker substitution is used.

Sources: `data/cards/foundations_micro_v1.json`, `data/rules` pins documented in
`doc/testing/rules.md`, and verbatim catalog in `doc/testing/capability-test-plan.json`.
The registration's byte-preserved GH-210 `prerequisite-gap.txt` is retained;
its composition draft and all 50-case audit obligations remain owned by GH-210.

Observation arrays are [zone, power, toughness, damage, tapped, summoning_sick];
nonbattlefield status is null. Mana rows are [red, green]. Consumed choices and
explicit rejected attempts are part of comparison. Checkpoints name first
semantic divergences. Missing data never counts as agreement.

The direct combat-damage extension observes applied native marks while draining
existing work one quantum at a time, before the queued death, and XMage's
DAMAGED_PERMANENT events. The separately authored amounts are Dragon seven and
Sentry five. No marks are calculated by the bridge or installed into game state.
XMage skips its attacker callback when there are no legal attackers; the explicit
Cub attack attempt is submitted at that step's priority boundary and must fail.
At the pinned source, `TestPlayer.declareAttacker` delegates to
`PlayerImpl.declareAttacker`, which checks `PermanentImpl.canAttack`:
tapping and `hasSummoningSickness()` are independent guards, without a
priority-phase guard substituting for sickness. Their consulted-source hashes
are retained in `toolchain.json`; no upstream source is changed.
