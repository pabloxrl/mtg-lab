# Played combat oracle, envelope version 9

These schedules and literal ledgers were authored from the unchanged frozen
`data/cards/foundations_micro_v1.json` and CR `cr-2026-09-25` before executing
combat. `author_combat.py` reads decks/pins, never engine output. The prior
[#210 evidence](../../doc/evidence/m2-reference-combat/README.md) is retained.
This extends the same normal-reset runner; it does not implement another rules
engine or inject hands, creatures, haste, damage, or combat state.

| Case | Attacker / blockers | Assignment and simultaneous marks | After SBA |
| --- | --- | --- | --- |
| Shivan unblocked | 5/5 flying / none | 5 to defending player | life 20–15 |
| Shivan–Sentry | 5/5 flying / 4/4 reach | 5 to Sentry, 4 to Shivan | Sentry dies; Shivan has 4 damage |
| Shivan–Thornweald | 5/5 flying / 2/1 reach/deathtouch | 5 to Thornweald, 2 to Shivan | both die |
| Shivan multiple | 5/5 / Sentry 4/4, Thornweald 2/1 | 4+1; Shivan receives 4+2 | all die |
| Modern split | same | 3+2; Shivan receives 4+2 | Sentry survives with 3 damage; others die |
| Pathwarden single | 5/4 vigilance/trample / Swab 2/2 | 2 to Swab, 3 to player; 2 to Pathwarden | Swab dies; life 17–20 |
| Pathwarden multiple | 5/4 / Cavalry 2/2, Swab 2/2 | 2+2, 1 to player; Pathwarden receives 2+2 | all die; life 19–20 |
| Pathwarden haste | same as single, cast this turn | legal opposing Cavalry tap/target/stack resolution grants haste | same damage outcome; Pathwarden remains untapped |
| Thornweald multiple | 2/1 deathtouch / Cavalry 2/2, Swab 2/2 | 1+1; Thornweald receives 2+2 | all die |

CR 508–510 supplies active/defending actors, declarations, blocked status,
assignment totals, simultaneous damage, then priority after SBAs. CR 702.9/17
supplies flying/reach; Cub cannot block Shivan. CR 702.2 makes one positive
Thornweald damage lethal. CR 702.19 requires lethal on **each** blocker before
trampling to the player. CR 702.20 leaves attacking Pathwarden untapped.
Modern CR 510 does not impose the obsolete blocker assignment order: the 3+2
Shivan split is legal even though the first Sentry receives nonlethal damage.
No response window is inserted between assignment and damage.

Pathwarden has no printed haste. CR 302.6 rejects its same-turn attack. The
paired positive casts Cavalry on turn three, lets it begin later controller
turns, and uses the delivered version-5 activation adapter on turn ten to grant
haste to the newly cast opposing Pathwarden. CR 602 and 702.10 require actual
target selection, source tap payment and stack resolution before the attack.
A separate negative moves Cavalry's cast to turn nine: it has not begun another
controller turn on turn ten, so it cannot pay its tap cost. No keyword is injected.

The independent legal-declaration ledger includes every eligible creature and
blocker/attacker pair in these bounded states. In particular, the Shivan cast on
turn eleven **can block** on turn twelve: CR 302.6 restricts attacks and tap/untap
costs, not blocking. These are complete eligible attacker and block-pair sets;
this is not enumeration of all possible subset configurations or all game actions.

## Reference contracts and adjudication

XMage `Player.getAvailableAttackers` supplies the creature-filtered attacker
set; `Permanent.canAttack` alone is a restriction helper and does not filter
noncreatures. The strict normal-game controller identifies itself as non-AI,
so `Combat.selectBlockers` never enters the AI retry/default-block path. Normal
`GameOptions.testMode` and `skipInitShuffling` remain false.

The pinned modern `CombatGroup.blockerDamage` calls
`getMultiAmountWithIndividualConstraints` with ordered recipients tied to its
actual combat group. Messages have no object IDs. The adapter witnesses IDs
from that same unique group and verifies explicit input identities against it;
it never parses card names to guess identities. Default amounts are retained
as provenance but never used as choices.

The upstream trample dialog exposes a total lethal minimum, with per-recipient
minimum zero. Thus `[1,3]` against two undamaged 2/2 blockers fits the dialog's
sum minimum 4 and maximum 5, but violates CR 702.19b because the first blocker
has not received lethal damage before the remaining point hits the player.
The original callback constraints are retained in the rejected run. The adapter
explicitly rejects that rules-invalid choice using the engine's witnessed
per-blocker lethal values. This is an **adjudicated upstream constraint gap**,
not a claim that upstream raw constraints rejected it. Other negative results
separate tape validation, engine legal-set membership and selected-action
acceptance. Existing #210 cases and all original pins remain unchanged.

Native simultaneous marks are observed by replaying the accepted damage finish
on a snapshot-restored real engine at quantum one, immediately before the first
SBA zone move. Its final semantic checkpoint and RNG must match scalar execution;
policy revision metadata differs because the probe uses the bounded rules API.
XMage's `DAMAGED_BATCH_COULD_HAVE_FIRED` watcher observes the real completed
combat damage batch before SBAs. These are privileged observations, not policy
inputs or newly introduced decisions.

## Explicit simultaneous graveyard orders

CR404.3 lets each owner choose the relative order of simultaneous arrivals;
CR704.3 makes applicable SBAs one simultaneous event. CR101.4 requires APNAP
choices before those actions; CR101.4b makes earlier owner choices public.
Version 9 therefore carries an explicit `graveyard_order` row for every owner
with multiple dying cards. `event: combat-sba/TURN` identifies the single
supported normal combat-damage SBA event in that turn. This family has no extra
combat, first strike or later damage events. `selection` is bottom-to-top and
uses the battlefield occurrence/incarnation before the move. No missing order
is inferred, even if an implementation has a deterministic iteration order.

The original Shivan multiple-blocker case retains Sentry then Thornweald as the
owner's chosen arrivals. Its reverse chooses Thornweald then Sentry, with the
same 4+1 allocation and deaths. The Pathwarden multiple case analogously chooses
Cavalry/Swab or Swab/Cavalry. Identity variants play physical occurrence `/1`
instead of `/0` for Cavalry, Swab, Sentry and Thornweald by swapping only the
corresponding shuffle and scripted identities, leaving card definitions and
the literal lethal ledger unchanged. The original `/0` occurrences remain in
the library. These are independent authored inputs, not output corrections.

The both-owners case attacks with Cavalry and Swab on turn nine. Cub blocks
Cavalry; Thornweald blocks Swab. All four creatures deal 2 damage; all four die
(2/2 Cavalry/Swab/Cub, 2/1 Thornweald). Seat 0 chooses first, then seat 1,
while every creature still has its simultaneous mark and both graveyards are
unchanged. The reverse and identity variants preserve this ledger. Native
synthetic tests additionally swap the active seat and preserve an existing
Forest at the bottom of each graveyard.

The separately hashed `CombatSbaGame.java` reference integration uses the pinned
protected SBA hook and `ZonesHandler.moveCards` batch API. The pinned upstream
lethal path never calls its optional graveyard-order helper. This is explicitly
**patched-reference agreement**, not unmodified-upstream agreement. Upstream
source/card/rules/dependency pins are unchanged. The original failing ordered
checkpoints and upstream execution are retained in `doc/evidence/combat-prefix/`.
Unsupported indestructibility or destruction/zone replacement behavior fails
explicitly. The repair is bounded to this nontriggering combat path.

The historical extra-allocation control retains the same extra input at the
same consumed cursor. Its rejection category is now wrong actor because the
new owner-order boundary belongs to the defender. The extra graveyard-choice
control is rejected after that choice is consumed, when priority belongs to
the active player. Neither control permits a second allocation or owner choice.
Every original outcome assertion remains; legacy synthetic clients explicitly
choose candidate order at the newly required boundary. No engine default is
used by the strict reference transcripts.
