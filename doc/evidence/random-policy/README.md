# GH-118 native legal-random M1 opponent

Bounded component of R0002-B020/B039/B041. This delivers the native Rust policy;
#21 retains original CLI/integration/catalog acceptance, #22 remains the M1 gate,
and M2–M5 requirements are unchanged. No full-pool, heuristic, recorder, CLI capture,
trainer, performance or reference-engine full-game claim.

## Contract and independent expectations

[Sampling/support/API](../../random-policy.md) is versioned independently from
[environment RNG](../../rng.md). The policy depends on `mtg-core` only for public
observation/submission types and an independently owned pinned RNG primitive;
its evaluation API receives no Game, privileged storage, library or environment
RNG. No rules core, existing test, workflow, CI or review enforcement was changed.
README describes the native capability separately from the still-passive CLI.

- `selection.rs::literal_masked_selection_vectors_both_seats`: existing
  independently specified SplitMix64 seat vectors (`mtg-core/tests/rng.rs`) reduced
  modulo three. Seat 0 expected rows `[2,1,1,2]`; seat 1 `[0,0,0,1]`. Threshold is
  one; none of these published words is rejected. Masked unsupported rows are
  not samples. Synthetic flat tables use authorized visible-reference shapes.
- Ordered without-replacement vector: first row 2, then row 0 from three rows.
  This follows the same literal words, modulo three then two, preserving remaining
  row order. Uniform permutations imply uniform cleanup sets as well.
- Factored vectors: fair replacement/finish coin, independent attacker bits,
  blocker choices over none/A/B. Two-power/two-blocker allocation is uniform over
  `(0,2),(1,1),(2,0)`; three-blocker allocations follow the documented conditional
  distribution, explicitly not uniform over compositions. Literal expected
  commands and exact sum conservation are independent of implementation output.
- Episode-one low-bit vectors for both seats; unrelated episode/seat evaluations
  cannot advance another stream. Invalid masks/empty domains do not consume RNG.
  Primitive rejection tests supply raw words: zero is rejected for bound three,
  then residues `1,2,0,1,2,0`; bounds one and u64::MAX are covered.
- Unsupported versions/kinds/enabled commands/content fail, with no pass fallback.
  An enabled Shivan Dragon alongside legal pass must return UnsupportedContent;
  when masked it remains uncastable. This preserves the declared M1 card boundary.

The initial four [compiled behavioral failures](red.log) precede implementation
of the policy. Initial handwritten residue arithmetic was checked/corrected from
already-published RNG literals **before implementation**, then red rerun. A
separate [compiled unsupported-content failure](content-red.log) preceded that
validation. Neither compile/import failure nor implementation-generated outcomes
serve as expected behavior.

## Real-game acceptance

[Normal-discovery integration tests](../../../crates/mtg-policy/tests/games.rs)
reset valid 40-card frozen decks using explicit orders, never synthetic state
edits. These fixed orders are test inputs, not policy features.

- Both starting seats: keep, lands, four resolved Cubs, real attack/block windows,
  one Cub blocked by two Cubs, policy-generated 1+1 damage. CR 510 permits this
  split: attacking 2/2 dies to four damage; both blockers survive; life stays 20.
  Growth makes the remaining own Cub 5/5; Bite uses that source to kill an opposing
  Cub. CR 601/608, current CR 508–510/704 and the pinned Oracle text justify exact
  stats, survivor counts and unchanged life. Red mirrors separately play/tap two
  Mountains and cast a summoning-sick 2/2 Swab Goblin for 1R.
- Forced scripts use bounded searches over policy seeds to produce required legal
  choices in each reachable state. They force acceptance coverage; they are not
  represented as uninterrupted random-policy episodes.
- Six uninterrupted random-policy games: policy seed/episode `(0,0),(42,7),(1,1)`
  from each starting seat, each replayed from reset. Both seats act, lands and
  spells are selected; all finish within the explicit 20,000-command test bound.
  Compare the complete semantic submission sequence, counts, full rules/object/
  pending/RNG state and outcomes. Only process-local store/scope capability IDs
  are normalized, as required by [object identity](../../objects.md). An initial
  test accidentally compared raw construction IDs; correction removes no semantic
  assertion and same-game nonmutation still compares raw full state. Actual loss
  conditions also require nonpositive life or an empty library on the losing
  seat's Draw step. Winners are not harvested as expected-results literals.
- Limits 0/1/20 leave the engine nonterminal and are explicitly classified as
  truncation, with exactly the permitted commands. No draws, wins or passive-game
  relabeling. The accounting loop is a test harness, not a production runner.
- Hidden twins change opponent hand identities and hidden library orders, retain
  equal actor observations and produce equal samples. Separate reachable target,
  staged-payment and factored-combat twins also compare samples. Evaluation leaves
  complete serialized game/RNG state unchanged. Wrong-seat, stale accepted command
  and guessed out-of-domain candidate submissions are rejected without mutation.

## Verification receipts and limits

[Focused green run](green.log) includes the actual seeded-game counts/outcomes.
[Final selection check](selection-final.log) includes the added literal cleanup
vector. [Full torture PASS](torture.log), with [source hashes and counts](verification.json):
143 Python tests, 293 Rust tests plus two doctests per debug/release profile.
Current main was fetched/integrated before validation. One initial Clippy helper
ordering failure was fixed without changing behavior; final suite passes.
Full managed-container `./scripts/torture.sh`, separate committed-candidate Codex
review, protected merge and exact-main CI are mandatory delivery evidence; links
and final status are retained in the GH-118 workpad/PR. No game rule changes were
made, so no new cached reference case is applicable and no new reference agreement
is claimed. Existing conformance/regression tests remain in the complete suite.

Reproduce in the managed image:

```text
cargo test --locked -p mtg-policy -- --nocapture
./scripts/torture.sh
```

The policy does not accept privileged config or validate the whole deck. Current
core legality only exposes Forest/Mountain, Bear Cub/Swab Goblin, Growth/Bite and
vanilla combat; other frozen cards may be drawn/bottomed/discarded, never cast.
Cancellation/backtracking can produce long games. Callers must retain errors and
explicit truncations, bind seats correctly, and recreate streams per episode.
