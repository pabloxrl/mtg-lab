# GH-114 played M1 replay acceptance

The [normal-discovery tests](../../../crates/mtg-core/tests/played_replay.rs) start
from two complete frozen 40-card green decks with literal orders, master 114 and
episode zero. No game state is injected. Keep both hands, play Forests, explicitly
pay for Bear Cubs, and cast Bite Down from P0 during P1's main phase. P1 responds
with Giant Growth on its Cub. P1 retains priority after casting; the stack is Bite
then Growth. Two passes resolve Growth; two further passes resolve Bite. Frozen
Cub (2/2), Growth (+3/+3) and Bite text plus CR 117/601/608 imply a 5/5 with two
damage, both Cubs alive, and unchanged life. CR 514 removes damage and the boost
together, leaving an undamaged 2/2 at turn seven upkeep. Ten explicitly unblocked
P0 Cub attacks produce successive P1 life totals 18, 16, …, 0; CR 510/704 imply
P0 wins by life loss. No concession substitutes for this rules-terminal script.
Late turns also exercise reachable maximum-hand cleanup discards.

Assertions inspect literal intermediate target identities, stack order, priority,
mana, Growth/damage and cleanup values, combat relationships, and terminal loss.
Captured checkpoints are consistency evidence, not generated golden expectations.
A fresh subprocess verifies the artifact with stdin closed/display unset and then
compares complete normalized core states after every action for work quanta 1, 3
and 17 against scalar execution. Pass/resolution, combat completion, turn startup
and cleanup discard use the real bounded core APIs. Remaining no-work action
choices use the delivered semantic codec. This is scheduler-work equivalence,
not a claim about M3 workers/batch APIs. The subprocess is bounded and must report
one passing test. Large replay tests serialize within this test binary to bound
peak memory; every test remains in default discovery.

Negative controls retain the final outcome while corrupting an earlier life,
priority or target checkpoint; verification must report the first affected index.
They also cover missing/extra choices, engine/rules/cards/RNG/shuffle/config/action
pins, changed master/episode, RNG state, duplicate birth identity, a legal same-name
Cub substitution, stale incarnation, raw-index extra fields and absent checkpoints.
The original opening replay suite remains unchanged, including independently
computed repeated-mulligan vectors and first-draw/reset regressions elsewhere.

[Compiled red](red.txt): the complete legal script reached its independent game
assertions before both new tests failed at the deliberately unimplemented recorder.
No import/compile error counted as red evidence. Initial test-harness corrections
and interrupted exploratory runs are not verification passes. A priority mutation
was corrected to change P1 to P0 after the first upkeep pass; changing P1 to itself
would not be a valid negative control. No game expectation was rebaselined.

No rules behavior, policy legality or existing test changed. The new replay uses
normal reset and the existing semantic action path. It is privileged full-state
persistence, separate from seat observations, opening-format CLI tools and datasets.
It records pinned native PRNG execution, not a cross-engine chance-event tape.
Original GR-010/011/012/020–022 and integration acceptance remain #19, with reference
adapter obligations #24/#38. M1 is incomplete until #22. M2–M5 remain unchanged.

README and API documentation describe the new core format and CLI limitation.
Final local verification, review and exact-main CI are recorded in the issue
workpad/PR; local receipts accompany this report when completed.

[Final full torture receipt](verification.json) records 134 Python tests and 257
Rust tests including two doctests in each debug/release profile. [Played-game
test output](green.txt) preserves both profiles. [Cached XMage receipt](xmage-combat.json)
records five existing combat scenarios and 20 matching checkpoints; it is not
full-game or replay-format reference agreement. The final suite ran after
fresh-main integration. Separate review and protected CI evidence remain in the
issue workpad and PR, tied to the committed candidate.
