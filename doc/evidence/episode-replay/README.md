# Captured episode replay acceptance (GH-162)

Scope: partial R0002-B036/B037 local replay binding and authorization. This is
not publication, full collector integration, mature-engine agreement or an M1
verdict. #154/#117 and later gates retain their complete original obligations.

Normal discovery extends `crates/mtg-core/tests/capture.rs`'s real
`complete_played_ledger_targets_payment_cancel_retry_and_factored_combat` game
with replay registration, authorization, verifier reconstruction and complete
normalized final snapshot/RNG comparison. Its original assertions remain intact.
The driver has already reset for a later episode before the old result is bound,
so acceptance also covers owned reset lifetime. Seed 160, ordinal 0, both complete
frozen green decks and explicit initial card orders remain in that test.

Independent named checkpoints: CR 103 gives seven-card hands/33-card libraries;
CR 601 and Bear Cub's printed cost require the existing paid land taps; Growth
makes the 2/2 Cub 5/5, and CR 510's two blocking 2/2s mark four simultaneous damage
while both blockers die; CR 514 restores the undamaged 2/2 after cleanup. Ten
subsequent unblocked two-damage attacks change life from [20,20] to [20,0], and
CR 704 gives P0 the life-loss win. The original capture test checks these facts;
the added replay assertions check initial life, the 5/5 four-damage checkpoint,
final life, and all final state/RNG fields through the existing verifier. The
replay output is never used to invent these expected values.

`episode::replay::tests` adds real normal-reset keep/concession episodes for
registry ownership, per-artifact default denial, wrong ID, cross-run/ordinal,
retired IDs, empty-registry lifetime and concurrent distinct entries. Real
bounded capture tests exhaust the decision limit and accepted-record capacity;
those produce unavailable truncated and failed results. An unfinished real
normal reset is also unavailable. Deliberate private status/config/history/
snapshot and registry-payload mutations are declared component fault injections,
not substitutes for the full played-game test. Missing terminal history remains
rejected. Seat-reader serialization is checked for privileged-field leakage.

Test-first evidence: [played red](red.log) compiled and reached all existing
independent game assertions, then failed because the placeholder registry
returned `Err(Binding)` instead of an available replay. [Component red](unit-red.log)
compiled and failed missing available/unavailable behavior. No compile failure
counts as a behavioral red. All added regressions are in ordinary cargo discovery;
no existing assertion, test or requirement was removed or weakened.

Commands:

```sh
cargo test -p mtg-core --lib episode::replay
cargo test -p mtg-core --test capture complete_played_ledger
./scripts/torture.sh
```

Delivery evidence (full torture, exact candidate independent review, protected
merge and exact-main CI) is recorded in the issue's single
[Agent workpad](https://github.com/pabloxrl/mtg-lab/issues/162#issuecomment-5904208581)
and linked PR. No rules behavior changed; no new reference-engine claim is made.
