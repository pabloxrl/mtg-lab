# Individually inspected scenario selections

123 selections across all twelve required families. Separate candidate review must assess these admissions; its exact-head result and protected delivery are recorded in the linked audit workpad. One row contributes one scenario, even if its test loops over seats, allocations, or repetitions. Native-only contract rows are not XMage agreement.

The input/oracle/test/receipt hashes and exact boundaries are in [reviewed-scenarios.json](reviewed-scenarios.json) and [reference-index.json](reference-index.json). Fresh current-source pack execution is indexed in [fresh-reference.json](fresh-reference.json). All inputs and expected outcomes predate this audit; no engine output was promoted to an oracle.

| Scenario | Family | Independent semantic distinction and expected behavior |
| --- | --- | --- |
| `creature-mana/llanowar-elves-priority` | Costs and mana | One G immediately; source taps, stack stays empty and priority stays with P0. |
| `creature-mana/llanowar-elves-sick` | Costs and mana | Fresh Elf cannot pay its tap cost; untapped and zero G remain. |
| `creature-mana/llanowar-elves-payment` | Costs and mana | Elf supplies G within Cub casting payment; no response window inside payment. |
| `creature-mana/llanowar-elves-floating` | Costs and mana | One prior G plus Elf G pays Cub's 1G; floating versus in-payment source timing is explicit. |
| `creature-mana/llanowar-elves-opponent` | Costs and mana | Nonpriority source activation rejected; no tap or mana. |
| `creature-mana/llanowar-elves-attacked` | Costs and mana | Attacking taps Elf; cannot tap it again for mana; one combat damage lowers defender to 19. |
| `creature-mana/druid-block` | Combat | 1/3 Druid and 2/2 Cub survive with two and one damage respectively; no mana activation. |
| `haste/cub` | Combat | Cavalry taps; fresh Cub gains haste and attacks for two. |
| `haste/opposing` | Target legality | Opposing Cub is a legal target and gains haste; it expires at cleanup before the opposing turn. |
| `haste/cleanup` | Continuous changes | Granted haste expires at cleanup; target remains present. |
| `haste/sick` | Costs and mana | Fresh Cavalry cannot activate its tap-cost ability. |
| `haste/tapped` | Costs and mana | Already tapped Cavalry cannot pay another tap cost. |
| `haste/source_dies` | Objects and zones | After activation, source departure does not remove target's haste or attack. |
| `haste/target_dies` | Target legality | Departed target gains no haste and is not recreated. |
| `haste/dead_before` | Objects and zones | Absent source cannot announce an activation. |
| `haste/cast` | Costs and mana | Actual 1R Cavalry cast resolves as a fresh untapped 2/2; no haste granted automatically. |
| `trample/unblocked` | Combat | Five damage to defender; vigilance leaves Pathwarden untapped. |
| `trample/two` | Combat | Two Cub blockers each take two, one tramples; all three creatures die simultaneously. |
| `trample/one` | Combat | Cub takes two, defender three; Pathwarden survives with two marked. |
| `trample/marked` | Combat | Previously marked Cub needs one more; four tramples, Pathwarden takes two. |
| `trample/departed` | Combat | Declared blocker departs; trample allows all five to defender despite blocked status. |
| `trample/sick` | Combat | Vigilance/trample do not permit a fresh creature to attack. |
| `trample/negative` | Combat | Assigning only one to unmarked 2/2 while trampling rejects before damage. |
| `trample/next-block` | Combat | Vigilant attacker stays untapped and can block on the following turn. |
| `deathtouch/flyer` | Combat | Synthetic 5/5 flyer and reach/deathtouch 2/1 both die to simultaneous combat damage. |
| `deathtouch/bite` | Target legality | Thornweald's two noncombat Bite damage kills the larger target; source receives no reciprocal damage. |
| `deathtouch/source-gone` | Target legality | Bite's departed source cannot deal damage; the remaining 2/2 target survives. |
| `deathtouch/no-trample` | Combat | Deathtouch alone does not allow damage to defending player through a blocker. |
| `deathtouch/growth-no-trample` | Combat | Growth makes Thornweald 5/4, but still cannot assign to player while blocked without trample. |
| `deathtouch/trample-sentries` | Combat | Synthetic 7/6 deathtouch/trample assigns one to each 4/4, five to player, then dies to eight. |
| `deathtouch/trample-cubs` | Combat | Same lethal threshold against two 2/2; 7/6 survives four damage and player takes five. |
| `deathtouch/trample-negative` | Combat | Zero to one living blocker cannot satisfy lethal before damage to player. |
| `deathtouch/zero-split` | Combat | Zero deathtouch damage does not kill; only the blocker receiving two dies. |
| `shivan/double` | Continuous changes | Two R activations stack to 7/5, never increase toughness. |
| `shivan/sick` | Continuous changes | No tap cost: fresh Dragon can pay R and become 6/5. |
| `shivan/cleanup` | Continuous changes | All temporary power boosts expire to 5/5. |
| `shivan/no_red` | Costs and mana | Missing R cannot announce the activation. |
| `shivan/foreign` | Costs and mana | Cannot activate opponent's Dragon; mana remains available. |
| `shivan/dead_before` | Objects and zones | Absent Dragon cannot announce; reserved mana is not consumed. |
| `shivan/dead_after` | Objects and zones | Source departure after payment leaves no creature to boost; ability still resolves. |
| `shivan/cast` | Costs and mana | 4RR resolves a 5/5 flyer and consumes six mana. |
| `shivan/short_cast` | Costs and mana | Five mana including RR cannot pay six total; Dragon does not enter. |
| `shivan/bite` | Target legality | Two resolved boosts make Bite read seven current power; 8/8 target survives with seven marked. |
| `shivan/haste` | Combat | Cavalry haste plus one boost lets fresh Dragon attack for six. |
| `shivan/sentry` | Combat | Reach Sentry legally blocks Dragon; Dragon survives with four marked. |
| `shivan/cub_illegal` | Combat | Ground Cub cannot block flying Dragon; defender takes five. |
| `shivan/excess` | Decisions and encoding | A six-point allocation from a five-power Dragon rejects atomically. |
| `shivan/grown_split` | Combat | Response Growth changes a blocker to 7/7 before damage; declared five is nonlethal. |
| `shivan/payment_sources` | Costs and mana | Land mana supplied inside activation payment gives no intervening priority; R is committed once. |
| `invoker/own` | Continuous changes | Eight mana boosts own Cub to 7/7 and grants trample. |
| `invoker/mixed` | Costs and mana | Mixed colors pay generic eight, leaving zero mana. |
| `invoker/opposing` | Target legality | Any creature is allowed; opposing Cub becomes 7/7 with trample. |
| `invoker/tapped` | Costs and mana | Tapped Invoker can activate because its ability has no tap cost. |
| `invoker/seven` | Costs and mana | Seven mana cannot pay eight; Cub remains 2/2 and mana is retained. |
| `invoker/missing` | Target legality | Missing mandatory target rejects before spending eight mana. |
| `invoker/illegal` | Target legality | Noncreature target rejects before spending eight mana. |
| `invoker/dead_source` | Objects and zones | Paid ability survives source death and boosts the chosen Cub. |
| `invoker/departed` | Objects and zones | Old target incarnation cannot transfer the boost to its replacement. |
| `invoker/cleanup` | Continuous changes | Temporary +5/+5 and granted trample expire together. |
| `invoker/thorn_dead` | Objects and zones | Paid ability survives source death and gives 2/1 Thornweald +5/+5, retaining reach/deathtouch. |
| `invoker/thorn_cleanup` | Continuous changes | Granted boost/trample expire while printed reach/deathtouch remain. |
| `invoker/thorn_double` | Continuous changes | Two boosts compose to 12/11, preserving printed keywords. |
| `invoker/thorn_combat` | Combat | Boosted deathtouch/trample attacks through blockers with five excess player damage. |
| `invoker/cast` | Costs and mana | 2GG resolves the 4/3 source without granting anyone a boost. |
| `invoker/short_cast` | Costs and mana | Three mana cannot cast 2GG. |
| `invoker/payment_sources` | Costs and mana | Eight distinct in-payment sources pay once each, after target selection, with zero priority callbacks. |
| `thrill/ordered` | Setup and randomness | Discard commits before response; Forest then Cub draw, Growth remains on top. |
| `thrill/one` | Terminal states | First draw succeeds; attempted second from empty library loses after resolution. |
| `thrill/empty` | Terminal states | Drawing from already empty library loses; zero cards enter hand. |
| `thrill/alone` | Costs and mana | No other hand card can pay discard; Thrill cannot discard itself. |
| `thrill/wrong_color` | Costs and mana | Two green mana cannot pay 1R; no discard or spending. |
| `thrill/short_mana` | Costs and mana | One red mana lacks the generic component; no partial payment. |
| `cast-triggers/archer_growth` | Triggers | Archer trigger goes above Growth, deals one before Growth resolves. |
| `cast-triggers/cyclops_growth` | Triggers | Self-targeted Growth follows Cyclops trigger: 0/4 -> 3/4 -> 6/7 -> 0/4 at cleanup. |
| `cast-triggers/mixed_growth` | Triggers | Chosen order resolves Cyclops boost before Archer damage, then Growth. |
| `cast-triggers/two_growth` | Triggers | Two physical Archers each produce one trigger and one damage. |
| `cast-triggers/dead_source` | Objects and zones | Archer trigger survives synthetic source departure. |
| `cast-triggers/dead_target` | Target legality | Growth's target departs, but its prior cast still produced Archer damage. |
| `cast-triggers/archer_fodder` | Triggers | Archer damage precedes two token creation on Fodder resolution. |
| `cast-triggers/cyclops_fodder` | Triggers | Cyclops boost precedes Fodder's two tokens. |
| `cast-triggers/creature` | Triggers | Casting a creature creates neither noncreature trigger. |
| `cast-triggers/no_mana` | Triggers | Failed unaffordable announcement creates no trigger. |
| `cast-triggers/cancel` | Triggers | Cancelled casting before commitment creates no trigger. |
| `cast-triggers/thrill_fail` | Triggers | Unpaid additional discard prevents Thrill cast and both triggers. |
| `cast-triggers/land` | Triggers | Playing a land is not casting a noncreature spell. |
| `cast-triggers/mana` | Triggers | Activating land mana is not casting a spell. |
| `cast-triggers/bite_reject` | Triggers | Friendly Bite destination rejects with no triggers; choosing the legal enemy then committing creates exactly Archer and Cyclops triggers. |
| `cast-triggers/apnap` | Triggers | Synthetic pending triggers are stacked active-player first and resolve nonactive-player first. |
| `etb-triggers/rules-triggers-etb-target-positive` | Triggers | Creature resolves first; ETB trigger then targets opponent and deals two. |
| `etb-triggers/rules-triggers-etb-target-negative` | Triggers | Creature is not a legal target for player-targeted ETB; valid player remains selectable. |
| `etb-triggers/rules-triggers-etb-target-interaction` | Triggers | Self-target at two life causes own rules loss. |
| `etb-triggers/rules-objects-dead-source-positive` | Objects and zones | Actual Bite kills Pyromancer, but queued ETB still deals two to opponent. |
| `etb-triggers/rules-priority-turn-structure-interaction` | Priority and stack | Injected end-step trigger resolves before onward turn advancement. |
| `etb-triggers/self_nonlethal` | Triggers | Player can target self at 20, ending at 18 rather than forcing opponent. |
| `cleanup/rules-continuous-hand-size-cleanup-positive` | Continuous changes | Discard two from nine to seven; boosts and marked damage disappear together. |
| `cleanup/rules-continuous-hand-size-cleanup-interaction` | Continuous changes | Pending trigger creates exceptional priority, resolves two damage, and requires another cleanup. |
| `cleanup/rules-continuous-hand-size-cleanup-regression` | Continuous changes | Thrill during exceptional cleanup pays discard, draws two, and repeats cleanup with all ledgers retained. |
| `m2-cost-composition/cavalry-elf` | Costs and mana | Cavalry grants fresh Elf haste, allowing immediate tap for G. |
| `m2-cost-composition/cavalry-druid-bite` | Costs and mana | Fresh Druid gains haste and supplies mana within Bite payment. |
| `m2-cost-composition/archer-thrill` | Costs and mana | Archer damage precedes Thrill's ordered draws; discarding does not trigger Archer. |
| `m2-cost-composition/holdout-two-archers-discard-surprise` | Triggers | Independent holdout: discard Surprise without casting it, two Archer triggers reduce opponent from five to three, then draw Forest/Mountain. |
| `m2-trigger-composition/cyclops_bite_survivor` | Triggers | Cyclops boost resolves first; Bite deals three to 4/4 Sentry, leaving it alive with three marked. |
| `m2-trigger-composition/lethal_fodder` | Terminal states | Archer's one damage ends game before lower Fodder creates tokens. |
| `m2-trigger-composition/holdout_departed_cyclops` | Objects and zones | Independent holdout: departed Cyclops cannot boost or receive Growth; two labeled Archers still deal two, ending at 18. |
| `m2-combat/double-growth` | Continuous changes | Two Growth spells make Cub 8/8 and cleanup returns it to 2/2. |
| `m2-combat/blocked-growth` | Combat | Blocked nontrampler remains blocked after injected blocker departure, even with Growth; no player damage. |
| `m2-combat/holdout-invoker-growth` | Combat | Independent holdout: two Invoker boosts plus Growth make Thornweald 15/14; assign one to each Sentry and thirteen to player; cleanup retains printed keywords. |
| `m2-combat/cleanup-shivan-thorn` | Continuous changes | Power boost and granted trample expire, while Dragon flying and Thornweald reach/deathtouch remain. |
| `full-pool-mulligan/red-green-0-keep` | Setup and randomness | Keep seven each; exact occurrence/order checkpoints and two initial chance events, no redraw. |
| `full-pool-mulligan/red-green-0-one` | Setup and randomness | Only starter mulligans once; exact named Mountain goes to bottom and six remain. |
| `full-pool-mulligan/red-green-0-unequal` | Setup and randomness | Both redraw once, only starter redraws again; ordered two-card bottom distinguishes physical duplicates. |
| `full-pool-mulligan/red-green-0-both` | Setup and randomness | Both players take three mulligans; staged declarations, bottom selections and chance consumption remain exact. |
| `full-pool-priority/red-green-0-keep` | Priority and stack | Normal keep through two creature resolutions; starter skips first draw and nonstarter draws the exact next occurrence. |
| `full-pool-priority/red-green-0-mulligan` | Priority and stack | Mulligan-adjusted library/hand feed the continuous cast prefix without reset injection. |
| `full-pool-priority/red-green-0-keep-mana-response` | Priority and stack | Opponent's explicit mana response preserves the creature spell and correct priority handoffs. |
| `native/m2_all_64_distinct_token_subsets_and_explicit_capacity_boundary` | Decisions and encoding | Independent 2^6 subset enumeration: all 64 legal token selections, including empty/full, commit exact identities; short domain fails completely. |
| `native/m2_last_token_and_finish_survive_high_board_capacity_boundary` | Decisions and encoding | 79/80 fit declared capacity 80, 81 reports needed 81; raising the bound recovers the final token and finish row. |
| `native/policy_hidden_twins_and_invalid_inputs_preserve_full_state` | Privacy and PvP | Private hand/library twins produce identical authorized views and candidate encodings; stale/wrong-seat/invalid submissions preserve full snapshots. |
| `native/policy_library_only_twins_payment_errors_and_masked_rows` | Privacy and PvP | Unseen-library permutations preserve masks/order/errors through explicit payment; masked rows reject without mutation. |
| `native/policy_restore_invalidates_pre_restore_submissions_without_state_change` | Replay and batching | Restore preserves semantics but invalidates old decision generations; stale submissions cannot mutate restored state. |
| `native/thrill_private_pending_snapshots_cancel_stale_and_quantum` | Replay and batching | Snapshots inside private discard/payment resume correctly; cancellation/stale rejection and quantum variants preserve literal outcomes. |
| `native/terminal_empty_draw_versus_empty_library_and_first_draw_skip` | Terminal states | Empty library alone is ongoing, attempted draw loses; starter skips only its first draw. |
| `native/terminal_main_priority_concession_each_seat_and_stale_actions_after_lethal` | Terminal states | Either seat may concede at main priority; rules terminal makes later actions stale without mutation. |

Seat-only variants, symmetrical blocker splits, repeated invocations, duplicate ETB aliases and the duplicated ordinary-cleanup input are excluded from this floor. They remain required passing regressions. The sixteen complete admitted games are supplemental and add zero to this selected floor. Four pack holdouts remain frozen; three are selected here and the same-name Sentry response is supplemental in the foundation receipt.
