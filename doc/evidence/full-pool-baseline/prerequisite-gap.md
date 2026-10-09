# Full-pool baseline: correctness prerequisite

Audit base: `522b7da8b86cd412c03c709bf88d8540c0385e16` (2026-10-09).
This is a local unsuccessful audit, not a published performance artifact.

GH-217 requires a correctness-gated full-pool baseline. The existing GH-23
[compiled regression report](https://github.com/pabloxrl/mtg-lab/issues/23#issuecomment-6079276519)
identifies omitted activation-payment mana choices. Source inspection confirms
the same paths on this base: `usable_activation_source` requires already-floating
mana for Shivan and Invoker, and `policy_observe` offers Pay but no TapMana during
activation payment. CR 602.2b applies the casting payment procedure, including
601.2g and the mana-ability opportunity in 605.3a. Floating mana before activation
does not provide the missing legal continuation. These are inherited independent
rules expectations, not outputs blessed from the implementation.

The three retained GH-23 synthetic tests were copied into the existing normal
test-discovery module `crates/mtg-core/src/shivan_tests.rs`. They check Shivan with
an untapped Mountain, Invoker with eight untapped Forests, and the missing mana
source choice during payment even when some mana already floats. Synthetic
positions reproduce the defect; they do not stand in for normal-reset benchmark
games or reference verification. Execution receipt is recorded in the workpad.

Operations [#252](https://github.com/pabloxrl/mtg-lab/issues/252) already owns
registration of the bounded activation-payment repair. Proposed coordination:
make its verified repair a real prerequisite for this correctness-gated baseline,
then explicitly reactivate GH-217. Reuse the existing benchmark and full-pool
policies after their version/choice contract is repaired. No task registration,
dependency mutation, mechanic implementation or policy workaround belongs here.

The other blocked reference audits remain their owners' obligations. This finding
does not invent reference-completion dependencies for the timing harness. The
specific legal-choice omission prevents a full-pool correctness qualification;
diagnostic timing alone cannot satisfy GH-217.

Environment inspection found GNU gprofng with heap tracing and PC sampling;
perf, Valgrind and heaptrack are absent. Those absences are not this blocker:
gprofng remains a plausible independent allocation/profile tool to probe on
resumption, alongside Linux process RSS. No tool installation was attempted.

No benchmark, torture pass, reference pass, independent final review, PR or
delivery is claimed. Production code is unchanged. README remains unchanged:
no usable command, supported behavior, architecture or verified milestone was
delivered, and M2 remains incomplete. Full validation and README review remain
required for the eventual measurement delivery.
