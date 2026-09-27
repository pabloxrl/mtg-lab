# Creature casting and stack resolution — GH-69

Atomic R0002-B017/B026/B029 casting portion. [API](../../casting.md).
Other effects remain GH-70, combat GH-71 and cross-feature acceptance GH-18.
No complete shared RFC block or M1 milestone is claimed.

Independent expectations use pinned CR 107.4/118/601.2f–i (exact/generic costs,
complete payment and cast), 117.3b/c and 117.4 (priority and consecutive passes),
302.1/302.6 (creature timing and control since turn start), 405/608.3 (LIFO and
creature entry) and 305/605 (basic mana abilities, no creature sickness for lands).
The frozen card definitions specify Bear Cub 1G and Swab Goblin 1R, both vanilla.
RFC invalid-action and continuation boundaries require exact nonmutation and no
intermediate opponent priority or provisional choice disclosure. Expected output
was not generated from the implementation.

[Red](red.txt): compiled nonfunctional casting APIs failed three independently
payable-cast assertions before implementation. The negative test already passed.
All original assertions remain in normal discovery. Later synthetic LIFO test
setup initially retained eight hand cards and correctly reached cleanup; the
synthetic extra card was removed from setup, without changing the expected turn
or any rules assertion. This setup failure is not claimed as feature red.

[Green](green.txt): eight casting tests, including a normal reset/keep/two-land/
postcombat-cast/next-untap test for both card types and both acting seats. The
postcombat sequence avoids unimplemented combat without synthetic turn jumps.
Unit tests explicitly seed floating pools, mixed lands, extra hand cards and
multi-spell stacks; they do not claim those injected states are normal-reset
reachability evidence. Two sorcery creatures cannot legally form a response
chain: nested casts reject unchanged; synthetic LIFO coverage and a real land
mana response exercise ordering/pass reset, and instant effects remain GH-70.

| Exact assigned catalog case | Executable test (prefix `casting_`) |
| --- | --- |
| rules-costs-mana-rejection-positive | literal_payment_stack_resolution_and_priority |
| rules-costs-mana-rejection-negative | missing_colors_masked_timing_foreign_and_wrong_actor |
| rules-costs-generic-payment-positive | generic_choices_and_atomic_rejected_final_command |
| rules-costs-generic-payment-negative | generic_choices_and_atomic_rejected_final_command |
| rules-costs-generic-payment-regression | land_activation_choices_cancel_and_retry |
| rules-costs-atomic-rejection-positive | literal_payment_stack_resolution_and_priority |
| rules-costs-atomic-rejection-regression | generic_choices_and_atomic_rejected_final_command; land_activation_choices_cancel_and_retry |
| rules-decisions-masked-rejection-positive | literal_payment_stack_resolution_and_priority |
| rules-decisions-masked-rejection-negative | missing_colors_masked_timing_foreign_and_wrong_actor |
| rules-foundations_micro_v1-forest-interaction | fresh_forest_mountain_negative_and_unsupported_cards |
| rules-foundations_micro_v1-mountain-negative | fresh_forest_mountain_negative_and_unsupported_cards |
| rules-foundations_micro_v1-mountain-interaction | land_activation_choices_cancel_and_retry |

Literal checkpoints cover exact pools and selected taps, new zone identity,
owner/controller, untapped creature entry, sickness through opponent versus own
turn starts, stack order, step and priority after each pass/cast/resolution.
Full Debug comparisons include RNG, private continuation, decision and storage
on rejected commands. Tests also retain cancellation, reset, wrong actor,
foreign/stale IDs and handles, incomplete/duplicate final commits, exhausted
decision counter, unsupported cards and illegal timing. Existing exhaustive
payment enumeration and storage/turn rejection tests remain unchanged.

[Mutation evidence](mutations.txt) records three compiled mutants caught by named
assertions: omitted sickness, FIFO resolution, and no mana deduction. Mutants are
restored before all final checks. [Full torture](torture.txt) PASS after fresh current-main integration: 120 Python
tests and 78 Rust tests in each debug/release profile, plus docs/program/catalog,
formatting and Clippy. It runs ordinary discovery, not just new tests.

README impact: new supported-capability row, API/evidence links and corrected
casting limitations in README/mana/turn documentation. No milestone-stage change.
Existing quickstart commands are unchanged; managed-container inner verification
runs through full torture. Docker is not invoked from the worker.

Reference scope: existing cached XMage neutral priority smoke is rerun because
priority passing changed: [fresh receipt](xmage.json) records two agreed runs.
Its bridge supports no creature cast/payment scenario,
so that result cannot establish upstream casting agreement. Forge is not freshly
executed. Expanded card/dual-reference coverage remains with integration/release
owners; unavailable coverage is not a passing result. Exact candidate review and
PR/main CI are recorded in the issue workpad and PR.
