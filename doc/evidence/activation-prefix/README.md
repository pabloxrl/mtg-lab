# Played activation prefix evidence (GH-273)

Local validation passed; delivery remains conditional on independent review,
protected merge and successful exact-main CI. No M2 completion claim. The [real-consumer behavioral red](behavioral-red.log) and its
[baseline test](behavioral-red.py) show the delivered played-priority consumer
producing nonempty observations but no activation stages, before adapter changes.
The initial [authored input hashes](authored-input-sha256.txt) predate execution
of the new client. Expanded positive/negative/cancellation inputs preserve the
original schedules and add coverage; none are generated from engine output.

The [independent oracle](../../../fixtures/reference/full-pool-activations-oracle.md)
defines literal outcomes, required payment chronology, raw representation
boundaries and cancellation coverage. Full validation/review/integration evidence
will be recorded here and in the
[issue workpad](https://github.com/pabloxrl/mtg-lab/issues/273#issuecomment-6100061488).

Development diagnostics were adapter/fixture issues, not rules disagreements:
an initial missing Java import; the native actor error wrapper (`Policy(WrongActor)`,
confirmed in the delivered semantic decoder); and a cancellation-authoring list
alias that renumbered shared event dictionaries. After those three diagnostics,
the verification approach retained separate raw native/reference staging,
independently authored per-control error boundaries and cancellation only at real
reference callbacks. No engine-derived expected stats, weakened existing tests or
source repins were used. Diagnostic logs remain separate from acceptance runs.

A later checker regression has its own [intended assertion red](paid-flag-red.log)
and [green](paid-flag-green.log): a flipped native `paid` flag escaped validation
until explicit typed paid-state assertions were added. The permanent regression
runs against the real native client in normal Python discovery. Reference unpaid
colored/generic costs are also checked literally. An additional untapped enemy
Forest control isolates controller legality while retaining the original control.

The [development diagnostics](development-diagnostics.tar.gz) retain unsuccessful
adapter attempts separately from acceptance. The initial Java build log and
original hash-matched authored input/script are retained here as compressed files.
The pre-hardening full regression log is `activation-torture.log.gz`; it is not
the final acceptance run.

The final [focused log](activation-focused-final.log.gz) records twelve new Python
tests passing through the real native client. One new Rust test is also part of
ordinary workspace discovery. The [second reference receipt](activations-run-2.json)
and its [privileged archive](activations-run-2.privileged.tar.gz) preserve actual
inputs, consumed choices, observations, toolchains and source/dependency hashes.
Archive members are privileged diagnostics, never player policy captures.

The [final integrated-main torture log](activation-torture-final.log.gz) passes
333 Python tests, formatting, Clippy, and 1,494 Rust test executions (747 each in
debug and release), including all new normal-discovery cases. The integrated
base is `60218ef1a75b645b5d98edaa369cd85412626321`; no original test was removed,
skipped, weakened or replaced. Full torture used the existing shared heavy lock.

Both final actual native/XMage activation runs agree: [run 1](activations-run-1.json)
and [run 2](activations-run-2.json), with [run 1 privileged artifacts](activations-run-1.privileged.tar.gz)
and the run 2 archive linked above. Each executes six played cases twice per
engine and compares 1,397 committed checkpoints, with fifteen invalid tapes and
fifteen comparator controls per engine, twenty-three native cancellation stages,
twenty actual XMage cancellation callbacks and three unsupported callback probes.
Every final receipt source hash and archived artifact hash was verified. Run 1
used a `activations-final-1` output stem to preserve the earlier diagnostic
receipt; its original privileged directory name and hashes remain intact.

The full command interfaces are in the [runner guide](../../full-pool-reference.md).
The changed focused/reference commands were executed with the pinned issue-local
cache and image toolchain. Root README now lists the bounded version-5 family;
independent review must check its accuracy. Final review, protected PR, merge and
exact-main CI receipts remain in the issue workpad linked above.

[Prior-family compatibility evidence](prior-family-compatibility.tar.gz) contains
two actual native/XMage priority runs and two actual spells runs, all agreed,
with unchanged inputs/expectations and hash-verified privileged artifacts.
