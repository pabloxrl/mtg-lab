# Independent fixture correction review

A separate read-only Codex session reviewed the proposed correction to
`invoker_payment_target_stale_cancel_reject_without_spending` on 2026-10-09.
Verdict: **Approve the proposed correction, with stronger replacement coverage.**

The original seven-mana rejection also supplied an untapped controlled Forest.
Under CR 602.2b applying 601.2g/h and 605.3a, it can pay Invoker's literal eight
cost. GH-254 explicitly requires mixed floating/generated mana; preserving that
rejection would preserve the defect. Tapping the fixture's Forest restores the
intended insufficient-mana condition while retaining every rejection, snapshot,
target, stale-decision and cancellation assertion. Its eight-floating path remains.

The reviewer required an additional normal-discovery test proving announcement,
target before generation, private reservations, no extra stack/priority window,
eight payments committing exactly one ability, a tapped Forest and zero remaining
mana, then Bear Cub resolving to 7/7 with trample. The implemented
`invoker_seven_floating_plus_forest_pays_inside_activation` asserts all these and
rejects a duplicate commit without mutation. No archived diagnostics or catalog
expectations change. Final prescribed candidate review must inspect implementation;
this preliminary requirement review does not replace it.
