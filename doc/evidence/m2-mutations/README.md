# M2 named semantic mutation acceptance

GH-213 delivers the bounded matrix in [the guide](../../testing/m2-mutations.md)
and [executable allocation](../../../fixtures/mutations/m2.json). Every row has
its own retained test, fixed input, independent rule/contract basis and named
assertion. These supplemental minimized tests preserve every prior regression.
No production rule change, new mechanic/card, whole-RFC-block or M2 completion
claim is made. #24 and #26 retain aggregate acceptance.

The [initial classifier red](runner-red.log) records ordinary Python assertion
failures before the runner's behavioral classification was implemented. It was
an executable failure, not an import/compile error. The normal engine assertions
are written before the source mutations and are expected to pass on the existing
rules. Mutation logs supply their deliberate behavioral reds; build or setup
failures never count as detections.

Reproduce inside the toolchain:

```sh
python3 scripts/m2_mutations.py --output .agent-artifacts/m2-mutation-check
./scripts/torture.sh
```

Managed workers use the shared heavy-work lock around both commands. The twelve
mutants run individually, each after its unmodified exact-test baseline. No
mutated source is retained in the original checkout. Successful detection requires
a separately successful build, exactly one discovered test and the designated
assertion panic. Runner tests also exercise survival, wrong scenario, wrong
assertion, empty discovery, build/launch failure and timeout.

README impact: the root guide index links the new usable mutation command. No
setup, support or milestone status changes; M2 remains incomplete. Existing
normal-reset full-pool and replay/trajectory tests remain mandatory, and synthetic
mutation positions do not replace them.

## Executed matrix

[Receipt and copied-source hashes](matrix/receipt.json): all twelve unmodified
baselines PASS, all twelve individually compiled mutants DETECTED at their
named assertions, runner exit 0. Executed on integrated head
`38978e831abde58e8a1b3678fd5aa78baac16787`, based on main
`7b7affc387d0464995661d15a39f732e7937e543`. Every case retains separate baseline and
mutant `.build.log`, `.list.log` and `.test.log` files in the receipt directory.
The copied-source hashes include documentation as it stood before publishing
this evidence; later receipt/report additions are not changed rules or test inputs.

The first matrix correctly failed overall: eleven detections, plus an
[unrelated truncation-test lifecycle panic](initial-truncation-harness-failure.log).
The new test redundantly tried to advance a Driver already stopped by its accepted
keep. Removing that extra action preserves the independently required truncation
assertion and matches the existing Driver budget contract. The full matrix above
was rerun after that correction and current-main integration. No existing test
was removed, weakened or rebaselined; no production bug fix was needed.

No production rules or reference bridge changes are made. The rules assertions
are independently derived from the pinned CR/Oracle and preserved contracts.
Existing [creature-mana](../creature-mana/README.md),
[cast-trigger](../cast-triggers/README.md) and cleanup/reference obligations remain;
this delivery does not claim a new external-reference run or new game capability.
Complete torture, independent review and protected exact-main delivery evidence
are required in the PR and issue completion workpad; the matrix alone is not a
whole-suite or milestone verdict.
