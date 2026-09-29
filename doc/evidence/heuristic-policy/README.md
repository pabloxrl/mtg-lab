# Deterministic heuristic acceptance — GH-119

This is partial R0002-B020/B039/B041 component evidence, not M1 completion.
#21 retains original integration/catalog acceptance; #22 remains mandatory.
M2–M5 obligations are unchanged. No rules implementation changed, so no new
reference-engine agreement is claimed or required for this policy-only change.

## Independently specified choices

The [strategy contract](../../heuristic-policy.md) was written before the policy.
[Literal examples](../../../crates/mtg-policy/tests/heuristic.rs) use synthetic
authorized observations to force land versus creature versus pass, Growth in
response, Bite, color-aware payment, target controller/lethality ranking,
attackers, blocking, modern allocation, bottom/discard and explicit errors.
These expectations are strategic choices, not inferred from implementation
output and not evidence of rules correctness or playing strength.

[red.log](red.log) records six compiling test groups failing against the explicit
unsupported stub. No compile/import failure is counted as behavioral red.
[green.log](green.log) records the full native policy package tests.
The unknown enabled command/content cases must error even when pass is legal.

## Real games and information boundaries

[Normal-reset tests](../../../crates/mtg-policy/tests/heuristic_games.rs) run both
starting seats with ordered green mirrors and ordinary shuffled green mirrors
(environment seed 42, episode 7), each twice. The policy makes every decision;
there is no forced tactical script or injected battlefield. All submissions,
final authorized observations and seat decision counts reproduce exactly.
[Game receipt](games.log) records observed counts/outcomes, not expected
winner literals. Independently asserted CR704.5a/b predicates require each
reported loss to have life <= 0 or the losing seat drawing with an empty library;
winner accounting must match the actual losing seats. Bounds 0/1/20 produce
explicit unfinished observations, never invented terminal draws. Evaluation
leaves the complete game, including RNG, unchanged.

Hidden-library twins execute matching actions through real priority, Growth/Bite
targets, payment and combat before any changed suffix can be drawn. Separate
opening twins have different opponent hands but identical authorized inputs and
choices, from both seats. Only the structured observation enters the policy.
The complete frozen decks still contain uncastable M2 cards; these games verify
the documented six-card M1 slice, not full-pool support.

## Delivery verification

The full managed-container torture receipt is [torture.log](torture.log).
The first invocation stopped at the documentation link check because this report
was not yet written; it is not counted as a passing run. A subsequent run
found two Clippy sorting-style errors, corrected without strategy changes.
The final full suite passed: 143 Python tests and 302 Rust tests plus two doctests
per debug/release profile, including all nine new heuristic regressions.
[Source hashes](verification.json) bind this receipt to the implementation/tests.
Independent
exact-candidate Codex review, PR verify and exact merged-main CI results are
recorded in the [issue workpad](https://github.com/pabloxrl/mtg-lab/issues/119)
and its linked PR. No existing tests are removed, skipped or weakened.
README gains the native heuristic capability and precise limits; existing
quickstart commands are unchanged and exercised by normal regression discovery.
No collector, CLI selection/capture, training, or performance claim is added.
