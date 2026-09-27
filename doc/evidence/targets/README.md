# Targeted Growth and Bite — GH-70

Atomic R0002-B017/B026/B029 target/effect portion. [API and limits](../../targets.md).
GH-18 retains cross-feature integration, GH-71 combat, GH-72 terminal outcomes;
no complete shared requirement block or M1 milestone is claimed.

Independent expectations use the pinned Growth (+3/+3 until end of turn) and
Bite (own creature deals current power to opposing creature) definitions,
CR 117.3b/c/d and 117.4 (priority/LIFO), 601.2c/f–i (targets/payment),
400.7 and 608.2b/h (new identity, revalidation, no illegal-source information),
613.4c (additive boosts), 704.5g (lethal damage before priority) and 514.2
(simultaneous damage removal and effect expiration). Expectations are literal
rules-derived values and independent Cartesian products, not engine output.
The pinned catalog and lossless requirement ledger are unchanged.

[Red](red.txt): before rule implementation, the compiled test fails because a
payable Growth with legal creatures is not offered. All further assertions were
authored before their corresponding effect code. [Green](green.txt) records the
named acceptance command. All regressions are part of ordinary test discovery.
The initial full-suite attempt found a Clippy collapsible-if in a new test;
formatting was corrected without changing expectations.

The normal-reset integration script uses frozen ordered green mirror decks,
explicit keeps, real lands and payments, a postcombat Cub cast, then two seats'
Growth responses during upkeep. It asserts 2→5→8 power, each response window,
and only one spell per pass round. It stops before unsupported combat. Bite
positions explicitly inject battlefield creatures/pools and postcombat timing;
all subsequent casts/targets/payments/responses/deaths/cleanup are real rules
API transitions. These are not claimed as normal-reset full-game evidence.
The catalog's Sentry is a synthetic 4/4 characteristic-only object, not enabled
M2 casting/Reach. Synthetic controller changes and zone reentries isolate
revalidation; no control-change or recursion spell is implemented.

| Exact assigned catalog case | Executable test (prefix `targets_`) |
| --- | --- |
| rules-priority-passing-negative | response_chain_priority_current_power_and_lethal |
| rules-priority-passing-interaction | hold_priority_bite_above_growth_and_five_no_redirection |
| rules-priority-passing-regression | normal_reset_real_response_chain |
| rules-priority-post-cast-resolution-positive | response_chain_priority_current_power_and_lethal |
| rules-priority-post-cast-resolution-negative | response_chain_priority_current_power_and_lethal |
| rules-priority-post-cast-resolution-interaction | response_chain_priority_current_power_and_lethal |
| rules-priority-post-cast-resolution-regression | response_chain_priority_current_power_and_lethal |
| rules-priority-response-chain-positive | cleanup_damage_and_expiration_simultaneous |
| rules-priority-response-chain-negative | response_chain_priority_current_power_and_lethal |
| rules-priority-response-chain-interaction | response_chain_priority_current_power_and_lethal |
| rules-priority-response-chain-regression | response_chain_priority_current_power_and_lethal |
| rules-priority-response-window-positive | cleanup_damage_and_expiration_simultaneous |
| rules-costs-generic-payment-interaction | generic_payment_preserves_future_growth_choices |
| rules-costs-atomic-rejection-interaction | required_restricted_stale_and_atomic |
| rules-targets-controller-restrictions-positive | growth_below_bite_zone_identity_no_retarget |
| rules-targets-controller-restrictions-negative | required_restricted_stale_and_atomic |
| rules-targets-controller-restrictions-interaction | required_restricted_stale_and_atomic |
| rules-targets-controller-restrictions-regression | required_restricted_stale_and_atomic |
| rules-targets-required-targets-positive | required_restricted_stale_and_atomic |
| rules-targets-required-targets-negative | required_restricted_stale_and_atomic |
| rules-targets-required-targets-regression | required_restricted_stale_and_atomic |
| rules-targets-zone-change-positive | growth_below_bite_zone_identity_no_retarget |
| rules-targets-zone-change-negative | growth_below_bite_zone_identity_no_retarget |
| rules-targets-zone-change-regression | growth_below_bite_zone_identity_no_retarget |
| rules-targets-partial-invalidation-positive | partial_invalidation_no_lki_or_redirection |
| rules-targets-partial-invalidation-negative | partial_invalidation_no_lki_or_redirection |
| rules-targets-partial-invalidation-interaction | partial_invalidation_no_lki_or_redirection |
| rules-targets-partial-invalidation-regression | hold_priority_bite_above_growth_and_five_no_redirection |
| rules-targets-resolution-revalidation-positive | normal_reset_real_response_chain |
| rules-targets-resolution-revalidation-negative | growth_below_bite_zone_identity_no_retarget |
| rules-targets-resolution-revalidation-interaction | cleanup_damage_and_expiration_simultaneous |
| rules-continuous-resolution-power-negative | partial_invalidation_no_lki_or_redirection |
| rules-continuous-cleanup-positive | cleanup_damage_and_expiration_simultaneous |
| rules-continuous-cleanup-negative | response_chain_priority_current_power_and_lethal |
| rules-continuous-cleanup-regression | cleanup_damage_and_expiration_simultaneous |
| rules-objects-zone-identity-positive | generic_payment_preserves_future_growth_choices |
| rules-objects-zone-identity-negative | growth_below_bite_zone_identity_no_retarget |
| rules-objects-zone-identity-interaction | response_chain_priority_current_power_and_lethal |
| rules-objects-zone-identity-regression | growth_below_bite_zone_identity_no_retarget |
| rules-decisions-all-combinations-positive | independent_all_pairs_and_capacity_boundaries |
| rules-decisions-all-combinations-negative | required_restricted_stale_and_atomic |
| rules-decisions-stale-candidates-positive | required_restricted_stale_and_atomic |
| rules-decisions-stale-candidates-interaction | required_restricted_stale_and_atomic |
| rules-decisions-capacity-positive | independent_all_pairs_and_capacity_boundaries |
| rules-decisions-capacity-negative | independent_all_pairs_and_capacity_boundaries |
| rules-foundations_micro_v1-giant-growth-positive | normal_reset_real_response_chain |
| rules-foundations_micro_v1-giant-growth-negative | required_restricted_stale_and_atomic |
| rules-foundations_micro_v1-giant-growth-interaction | cleanup_damage_and_expiration_simultaneous |
| rules-foundations_micro_v1-giant-growth-regression | cleanup_damage_and_expiration_simultaneous |

Additional retained checks cover stacked Growth, private continuation isolation,
wrong/foreign/stale IDs, absent target roles, missing targets, stale zone handles,
final-commit revalidation, no payment bypass, provisional land cancellation,
reset isolation, controller revalidation and finite numeric/generation overflow.
Full-state Debug comparisons include objects, RNG, decisions and pending work.

Capacity tests independently form all ordered own/opponent pairs for 1×1, 2×2,
4×4, 16×16, 40×40 and 80×80, comparing every selectable pair and cardinality.
Each exact role bound succeeds; one below fails explicitly before mutation.
An asymmetric 1×5 case verifies preflight of the later target list. Normal M1
states contain 80 total cards at most; larger two-role stress is synthetic.
No fixed tensor capacity or future token bound is claimed.

[Mutation evidence](mutations.txt): compiled defects retaining cleanup modifiers,
caching Bite power at two, and ignoring destination controller revalidation are
caught by named behavioral assertions. Mutants are restored before final checks.
[Full torture](torture.txt) PASS after fresh-main integration: 120 Python tests,
all 91 Rust tests in each debug/release profile, documentation/program/catalog
checks, formatting and Clippy. The named target command runs 13 new tests.

README impact: supported targeted-instant/API/evidence row, corrected related
turn/casting limitations; milestone table unchanged. Quickstart commands are
unchanged and managed-container inner checks run through torture. No Docker is
invoked inside the worker.

Reference scope: the cached XMage neutral priority smoke is rerun because shared
stack/priority code changed; [receipt](xmage.json). Its current bridge cannot
execute Growth/Bite or export their characteristics/damage, so this is not
reference agreement for the effects. Forge is not freshly executed. Expanded
matched effect and dual-reference execution remain with integration/release
owners, not silently passing results. Review and exact candidate/main CI receipts
are recorded in the issue workpad and PR.
