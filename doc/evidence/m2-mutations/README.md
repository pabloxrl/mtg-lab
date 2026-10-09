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
