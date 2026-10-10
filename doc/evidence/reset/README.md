# Occurrence-preserving normal-reset prefixes

Delivery for [GH-269](https://github.com/pabloxrl/mtg-lab/issues/269), partial
R0002-B010/B011/B026/B028/B030. No M2, full-opening, full-game or Forge verdict.
The [protocol and commands](../../full-pool-reference.md) define the bounded
first-declaration stop, separate privileged evidence and unsupported callbacks.

## Independent oracle and test-first evidence

CR 103 and frozen deck multiplicities determine the literal
[expectations](../../../fixtures/reference/full-pool-reset-expectations.json).
The [authored permutations and copy-swap rationale](../../../fixtures/reference/full-pool-reset-oracle.md)
are independent of native and reference output. Sixteen inputs cover the eight
deck/starter rows plus swaps of same-name copies, with repeated resets.

Before modifying native reset, the [baseline client](baseline-client.rs.txt)
compiled and ran through the shared heavy lock:

```sh
cargo test -p mtg-core --locked --lib full_pool_reset_literal_checkpoints
```

It failed the intended `creation-bound occurrence positions` assertion:
one executed, one failed, zero ignored. The [raw failure](privileged/red.log)
shows post-permutation insertion identities disagreeing with the authored
pre-permutation identities. This was not a compile/import failure. The same
literal occurrence assertion remains in normal test discovery.

The existing reset preflight was extracted without changing its validation or
initialization semantics. A cfg(test)-only chance hook binds semantic birth
identities during allocation, permutes actual libraries, and uses the existing
opening draw/declaration routines. The old seeded and name-only reset paths,
serialized work, source/card/rules pins and OpeningCountsTest remain unchanged.
A native compatibility test compares card-name projections with the old API.

The new XMage constructor selects exact pinned Foundations printing metadata,
creates all forty cards per seat, and runs normal duel initialization with
initial shuffling and draws enabled. It exports state at the first declaration
callback and stops without submitting a keep/mulligan choice. No derived hand,
life, turn or outcome is injected to manufacture agreement.

## Retained reference attempts

The first real reference run compiled and executed, then rejected
`/runtime chance before`. The pinned `Deck.getMaindeckCards()` collects cards
into an unordered set, so its iteration order is not the creation order. The
contract requires the pre-event **multiset**, while the post-event permutation
and checkpoint library order are literal. The adapter now checks the witnessed
multiset without changing any expected checkpoint, canonicalizes its serialized
multiset by creation bindings, and retains the raw order separately. The
[first failure log](privileged/first-reference-failure.log.gz) remains evidence
of the adapter defect, not an engine disagreement or a passing run.

The issue-local cold XMage build was killed at smoke startup; its
[log](privileged/cold-build-killed.log.gz) is retained. The unchanged cached smoke
retry passed before running the new reset bridge. No source, card, rules,
toolchain or dependency pins were changed.

The next real execution reached all sixteen prefixes and their repeats, but
Gson omitted the witnessed null `turn_active_seat` field when serializing.
The strict comparator rejected the missing field. The
[failed observations and logs](privileged/missing-null-attempt.tar.gz) are
retained; the exporter now uses `serializeNulls()`. The existing normally
discovered `test_every_observed_field_is_required` includes this exact
missing-null-field control. No expected state changed and a missing field
never became agreement.

## Executed acceptance

The [focused Python result](focused.log) passes six tests, exercising the real
native client over sixteen cases, repeated resets, 1,280 stale-handle rejections
and nineteen invalid-input controls. Three new Rust tests and six new Python
tests participate in normal discovery; no existing test or oracle was removed,
skipped, weakened or replaced.

Both pinned real executions agree with the independently authored literal oracle:
[run 1 receipt](reset-run-1.json) and [run 2 receipt](reset-run-2.json).
Each run executes 32 resets per engine: eight matchup/starter rows and eight
same-name copy swaps, each repeated. Both compare exact hand membership,
ordered libraries, the creation-bound occurrence map, life, starting/declaration
seat and the observed null turn-active seat. Each detects nineteen malformed
inputs, twelve missing/swapped-checkpoint controls and two explicit unsupported
callback probes. The callback probes are supplemental; they are not played
mulligan or priority continuations.

The corresponding [run 1 archive](privileged/reset-run-1.tar.gz) and
[run 2 archive](privileged/reset-run-2.tar.gz) retain inputs, the independent
oracle, both engines' actual checkpoints and consumed chance/choice records,
repeat observations, raw XMage pre-shuffle orders, rejection diagnostics,
source/bridge/dependency/toolchain hashes and process logs. Extract an archive
beside its receipt to restore the named privileged directory. Receipt hashes
were checked against every archived file and current source; both runs use
identical source and pin hashes. These archives expose hands and library order
and are privileged verification evidence, never player-policy inputs.

Full torture passed before integration (270 Python tests; 1,426 native
executions) and after integrating main `4068c855d1bcfbe9f1c63dfd5c98a44752e8c95c`
(274 Python tests; 1,446 native executions), with zero failures/ignored tests.
The [first log](torture.log.gz) and [integrated log](integrated-torture.log.gz)
are retained. The [final full run](final-torture.log.gz) after the null-export correction also
passes: 274 Python tests and 1,446 native debug/release executions, zero
failures/ignored tests, plus formatting, clippy, documentation, program and
catalog checks. All Cargo/Maven/reference work used the shared heavy lock.
`python3 scripts/run_tests.py` and `cargo test --workspace --locked` ran in
normal discovery as part of each full torture invocation.

README impact: added a scoped test-only reference-prefix link and documented
commands. No production CLI, rules/source pin, replay format or verified milestone
status changed. Independent review must check README accuracy. Final separate
review, protected merge and exact-main CI receipts belong in the
[issue workpad](https://github.com/pabloxrl/mtg-lab/issues/269#issuecomment-6093324326)
and linked delivery PR. Completion remains conditional on successful delivery.

The [artifact digest inventory](sha256.json) covers this report, receipts and
retained logs/archives (excluding the inventory itself).
