# Played cleanup delivery evidence

GH-276 candidate verification passed. Protected delivery remains conditional on
independent review, PR checks, merge and successful CI on that exact main commit;
see the [completion workpad](https://github.com/pabloxrl/mtg-lab/issues/276#issuecomment-6102148373).
No M2 completion or general corpus-admission claim.

The version-7 test-only family executes two directed normal-reset 40-card games
with both starters, a discard prefix, a stacked Growth/Surprise/damage cleanup
prefix, and a separately labeled concession. All explicit choices and a final
checkpoint must be consumed. The [independent rules/card ledger](../../../fixtures/reference/full-pool-cleanup-oracle.md)
precedes reference comparison; no expected game result comes from captured output.

[Compiled behavioral red](native-red.log) records the existing real client
rejecting a legal cleanup discard at action 32 as `WrongKind`. The native client
now routes cleanup discards to the delivered cleanup decision instead of a casting
cost. No production game rules changed. Earlier tests and source/card/rules pins
remain intact; additions are 15 Python tests and two Rust tests in normal discovery.

## Candidate validation

- [Final torture log](torture.log): `python3 scripts/run_tests.py` passed 359 tests;
  formatting, Clippy, `cargo test --workspace --locked` and release validation
  passed. 1,500 Rust test executions across 134 result groups; zero failed/ignored.
- [Focused module log](focused.log): all 15 tests passed through
  `python3 -m unittest discover -s tests -p 'test_m2_repair_cleanup.py'`.
- [Reference run 1](reference-run-1.json) and [run 2](reference-run-2.json): both
  prescribed locked `full_pool_reference.py --family cleanup` runs passed.
  Each repeats every positive case twice per engine; fourteen real invalid tapes
  and twenty comparator controls reject at their intended boundary. Two complete
  natural games each consume 1,206 actions and observe 67 settled cleanups;
  the discard prefix consumes 34 actions/two cleanups, stacked-expiry consumes
  198/eight, and separate concession consumes two/zero.
- [Compatibility log](compatibility.log): London/mulligan, priority, spells,
  activations and triggers passed twice each through the same versioned runner.
  Their `cleanup-compat-<family>-<N>.json` receipts retain source/artifact hashes.

All Cargo/Maven/reference/heavy validation used the existing shared `heavy` lock.
The two final commands are documented in [the protocol](../../full-pool-reference.md#version-7-played-cleanup-and-rules-endings).
Validation integrated current main `9ff1216273338076a9cb147ffd93661c397a4da3`;
subsequent exact-candidate review and delivery receipts live in the PR/workpad.

## Provenance and limits

[Privileged run 1](privileged-run-1.tar.gz), [run 2](privileged-run-2.tar.gz), and
[compatibility captures](compatibility-privileged.tar.gz) retain raw inputs,
consumed choices/chance, engine checkpoints, actual negative prefixes, toolchain
and bridge hashes. They are test evidence, separated from policy-visible capture;
archive directory/file permissions remain restricted. Extract beside the receipts
so `privileged_directory` resolves, then verify their listed SHA-256 artifacts.
[Archive checksums](artifact-hashes.json) cover the delivered evidence files.
[Diagnostics](diagnostics.tar.gz) retains earlier unsuccessful probes and the
explicitly interim successful run; neither substitutes for final receipts.

The reference LOST-event state precedes losing-player object removal. Its raw
post-loop result is separately retained; no next priority callback is required.
Concession removes the player earlier and has a separate pre-concession observation.
Native terminal rewards are checked against outcome-derived seat expectations.
Omitted final acknowledgement, unused suffix after actual termination, and
concession mislabeled natural completion all reject in both real clients.

Additional cleanup and #257 simultaneous losses retain explicitly synthetic
provenance and are reexecuted in both final runs. The played fixtures do not claim
reachable pending-cleanup triggers or simultaneous-loss full games. Raw spell
announcement/payment staging differs between engines and is not labeled equal;
committed states and cleanup observations compare. No complete legal-set,
reference internal-RNG/rollback, new trigger/combat support, broad campaign,
dual-reference admission, or M2 gate completion is claimed.
