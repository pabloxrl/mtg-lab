# London opening reference evidence

The integrated candidate passes full torture: **291 Python tests and 1484
Rust debug/release test executions**, zero failed/ignored. The complete
[log](torture.log.gz) includes `python3 scripts/run_tests.py` and
`cargo test --workspace --locked`, formatting, Clippy, documentation, program
and catalog checks. The new normal-discovery coverage is seven Python tests
and one Rust test executing 32 real opening cases and 32 input controls.
Base: `e1ee7bfc2715a04e0f7e3a29a827466d342565d3`. Independent review,
protected merge and exact-main CI receipts belong in the PR and
[issue workpad](https://github.com/pabloxrl/mtg-lab/issues/270#issuecomment-6096229458);
this report alone does not claim delivery completion.

The [independent oracle](oracle.md) records CR 103.5, actual source-pin
verification, explicit fixture arithmetic and the per-player comparison
boundary. The [original test](red-test.py) and [assertion failure](red.log)
precede implementation. The literal expectations were never regenerated from
engine output. [Fixture authoring arithmetic](author-fixtures.py) uses only
CR-derived round plans and the retained reset permutations.

Both prescribed actual pinned native/XMage executions pass on the final source:
[run 1](mulligan-run-1.json), [run 2](mulligan-run-2.json). Each compares 32
cases, repeated once per engine (64 comparison prefixes per engine), with
32 input rejection controls, nine comparator controls and three unexpected
callback probes. These are 32 distinct cases, not 64 distinct games. Both
receipts and every recorded source/artifact hash were verified after main
integration. The profiling delivery changed no covered opening dependency.

The separate privileged archives contain inputs, actual consumed chance and
choices, observed checkpoints, both raw callback histories, the independent
oracle, native/reference rejection diagnostics, pinned source/dependency hashes,
compiler/runtime executable hashes and logs:

- [Run 1 privileged archive](mulligan-run-1.privileged.tar.gz)
- [Run 2 privileged archive](mulligan-run-2.privileged.tar.gz)
- [Earlier attempts and failures](prior-attempts.privileged.tar.gz)

Extract an archive into an isolated diagnostic directory; its top-level folder
matches `privileged_directory` in the corresponding receipt. Verify the contained
files using `privileged_artifacts`. These contain both hands and library orders;
they are not player-facing observations or policy features. The digest inventory
covers the retained files. Earlier compile/lifecycle/lint failures and the
superseded-base interrupted run are unsuccessful evidence, not hidden skips.

The [protocol and runnable commands](../../full-pool-reference.md#version-2-london-opening-continuation)
use an additive version 2 envelope. Original version-1 reset and count-only tests,
all original catalog cases/owners/expectations and source/card/rules pins remain.
No production rule, replay format, workflow, CI or repository setting changed.
README adds the scoped test-only family and keeps milestone status unchanged.
Independent review must check README accuracy and the explicitly documented
raw-scheduling comparison boundary.

Native transcript application rejects transactionally without changing the
caller's complete state/RNG. Explicit chance inputs consume no native RNG words.
XMage rejection stops at the first divergence; its rollback and internal RNG
are unobservable and reported separately. Raw cross-player scheduling differs
between engines and is retained, never falsely reported equal. Compared fields
are each player's real declaration/redraw-bottom boundaries and both final
hands/libraries, plus the complete separately ordered chance and choice streams.
Adapter rejection and selected-action acceptance are tested; exhaustive legal-set
comparison is not claimed.

This is a first-upkeep prefix. No priority action, first draw, full game, Forge,
game admission, RFC 0003/RL implementation or M2 completion is claimed. Original
owners and the #24/#25/#26 audits retain their complete acceptance.
