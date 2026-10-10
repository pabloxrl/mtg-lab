# Played priority prefix oracle (work in progress)

This is an independently authored test specification, not an execution receipt.
The unchanged CR source was fetched and checked against the committed SHA-256
`8d860e451f20f38865b725b42d82feb714c725373dd8f3b32b8652b3eeb070ca`.

The two frozen vanilla creatures are Bear Cub ({1}{G}, 2/2) and Swab Goblin
({1}{R}, 2/2), each with no rules text. Full frozen decks, literal occurrence
permutations and explicit London declarations precede every played prefix.

The starting seat skips its entire first draw step (103.8a). Subsequent upkeep
passes reach a draw step with one card drawn before priority (117.3a). Both
seats pass explicitly at every priority boundary. A land play does not use the
stack and leaves priority with its player (305.1, 117.3c); a second land that
turn is illegal (305.2). A creature requires its controller's main phase,
priority and an empty stack (302.1). Opponent-turn casting must reject.

Casting comprises announcement, basic-land mana production, colored and generic
cost payment, then commit (601.2a, f–i). Both floating mana and mana produced
during casting are exercised. Two consecutive passes resolve the top spell;
its controller does not receive an extra unscripted pass (117.4). The active
player receives priority after resolution (117.3b). Each vanilla spell enters
as an untapped 2/2 creature (608.3a); it is summoning sick until its controller's
next turn (302.6). Its physical occurrence persists, while hand, stack and
battlefield incarnations are distinct. Same-name copies cannot collapse.

Empty attacker selections are explicit transcript entries (508.1a), followed
by the active player's priority (508.2). With no attackers, blocker and damage
steps are skipped (508.8). No nonempty combat, triggered abilities, cleanup
discard or terminal-game acceptance is intended.

Native staging and XMage's internal casting callbacks must be retained as raw
evidence. Any semantic alignment must cite the actual observed source fields;
missing fields, inferred identities or overwritten game state cannot count as
agreement. Complete legal-set comparison is a separate claim from accepted
selected actions and transactional rejected native commands.

`author_priority.py` writes six fixed six-turn inputs (two starting seats,
with keep or one starter mulligan, plus both starters with a mana response),
twenty deliberately invalid inputs, six independent final ledgers and 760
native boundary expectations. It reads
only the pinned manifest and authored inputs, never either engine's output.
It is fixture-authoring arithmetic for this one prescribed vanilla script, not
an alternative game consumer or a source of state injected into either engine.
The final trace state is checked against a separately expressed final ledger.

Each player plays three lands and resolves copy 0 on their second turn and
copy 1 on their third. Copy 0 uses floating mana; copy 1 stages two basic mana
activations during payment. Both spend their colored cost before their generic
cost explicitly. The prefix stops after the nonstarter's second creature resolves
on turn 6. The starter has drawn on turns 3 and 5; the other seat has drawn on
turns 2, 4 and 6. Both first creatures are awake by this stop, while both second
creatures are sick. Land copies 0 and 1 are tapped, copy 2 is untapped. Life is
20 each, mana/stack/graveyards/exile are empty, and all 80 occurrences remain.

In the two mana-response variants, the caster passes after the turn-3 cast.
The opponent activates their already played basic land, retains priority and
passes. The caster must then pass before the creature resolves (117.4): the
mana activation interrupted the first pair of passes. The resulting floating
mana survives to the end of that main phase and then empties (106.4).

The off-turn negative changes only the green permutation to place Llanowar Elves
in hand. On turn 3 green has an untapped Forest from turn 2, so this illegal
creature cast is affordable and isolates the timing rule. No creature-mana
activation is requested or accepted by this fixture.

Pinned reference representation: `Mulligan.drawHand` calls
`MulliganDefaultHandSorter` (stable lands before creatures for these hands).
The reference raw-order oracle applies that source-defined presentation order;
later draws append. Both raw hand orders are asserted, and cross-engine hand
comparison uses membership. Libraries, battlefield and stack remain ordered.
`LondonMulligan.mulligan` returns the hand through `Library.addAll`/`hand.clear`
without raw counter increments. Canonical reference incarnations therefore count
actual observed zone transitions; the bridge retains raw counters and permits
that zero increment only at a witnessed mulligan hand-to-library return. No
expected state is inserted into the engine. The original mistaken raw-reference
assumptions and the source-based correction are retained in the evidence report
for mandatory independent review; native expectations and inputs are unchanged.
