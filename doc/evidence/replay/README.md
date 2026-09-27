# GH-75 semantic opening replay acceptance

Atomic R0002-B016/B025 and SYS-REPLAY **opening replay** delivery. No full catalog
case is assigned to GH-75. #19 retains spells and cross-feature replay acceptance;
#79 retains CLI. M1 is incomplete. All scenarios here use normal reset/opening,
not synthetic game states. Expectations derive from pinned CR 103.5 and the
lossless RFC ledger, frozen card manifest, and the independent Python arithmetic
oracle in [mulligan vectors](../mulligan/vectors.py).

## Executable contract

`cargo test -p mtg-core --test replay` runs the normally discovered
[replay tests](../../../crates/mtg-core/tests/replay.rs).

- Literal seven-card opening, 33-card libraries, 20 life, both starters and
  successful transition into turns; explicit red/green mirror orders have seven
  Mountains/Forests independently of record/verify agreement.
- Two real both-seat mulligan rounds, then one seat keeps and the other bottoms
  two distinct choices in specified order. Every hand/library card is checked
  against GH-65's independent Python vectors for both starters at both rounds.
  Semantic occurrence references come from that oracle ledger, not engine output.
- Exact consumed-choice checks on recording and verification, missing and extra
  choices, malformed input, raw-index actions, extra index fields, unknown or
  missing schema fields, wrong versions/pins, altered config/seeds/RNG and missing
  checkpoints.
- Wrong actor/kind/card/cardinality, duplicate bottoms and invalid occurrences;
  legal action and legal card substitutions are detected at the first affected
  checkpoint. Multiple damaged checkpoints report the first and a literal field
  path/expected/actual. No fallback choices.
- Fresh-process replay with closed stdin checks the independent full two-round ledger.
  The documentation example is an executable Rust doctest.
- Identical bytes across independent allocations supplement independent expected
  states; round-trip agreement alone is not the rules oracle.

## Test-first evidence

[red.txt](red.txt) records compiled stub execution before implementation. Two
behavioral assertions failed directly: life was `[0,0]` instead of `[20,20]`, and
malformed replay was accepted. Two additional tests could not access the stub's
missing envelope fields; these are not claimed as behavioral divergence evidence.
The assertions remain in normal discovery. [strict-field-red.txt](strict-field-red.txt)
is a further real regression: Serde's tagged unit variants accepted an extra raw
index field. Empty struct variants enforce unknown-field rejection; no assertion
was weakened. An intermediate typo calling a nonexistent turn accessor was a
compile failure, not red evidence; the test now checks the documented
`start_turns` boundary.

[missing-checkpoint-red.txt](missing-checkpoint-red.txt) preserves a compiled
failure for a missing final decision field: optional-field deserialization had
silently treated omission as null. The checkpoint now retains the exact required
JSON value, so missing fields fail and nested extra fields cannot disappear.

[green.txt](green.txt) records all ten named acceptance checks;
[example.txt](example.txt) executes the documented API example.
[torture.txt](torture.txt) records the full suite after integrating current main.
Three compiled behavioral mutants were caught, with production source restored:
[ignored checkpoints](mutant-ignore-checkpoints.txt),
[accepted incomplete scripts](mutant-accept-incomplete.txt), and
[ignored card occurrence](mutant-ignore-card-occurrence.txt). No existing tests are
removed, skipped, weakened or rebaselined. No workflow policy changed.

## Boundaries

The library returns privileged owned bytes/a fresh verified game. It does not
add player-visible replay output, authenticate authors, perform migrations, or
replay spells. Initial config orders are supported; subsequent mulligans use the
pinned RNG. There are no new Magic rules or impacted cached reference cases;
no mature-engine replay agreement is claimed. The existing reference bridges do
not consume this custom replay envelope.

README adds the usable library behavior and durable acceptance link, retains
M0/M1 verdicts, and names later owners. Docker quickstart commands are unchanged;
their regression command runs inside the managed Docker worker without invoking
Docker from the worker. Review/PR/exact-main CI receipts live in the issue workpad
and PR once delivery completes.
