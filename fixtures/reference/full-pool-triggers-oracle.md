# Played trigger oracle (version 6)

These expectations are authored from the unchanged frozen card manifest and CR
603.2, 603.3b–d, 101.4, 113.7a and 601.2i, before reference execution.

A committed noncreature cast triggers each controlled Firebrand Archer and
Crackling Cyclops once, before the spell resolves. Creature casts, land plays
and mana abilities do not trigger them. Archer deals one to the opponent;
Cyclops is printed 0/4 and receives +3/+0 until end of turn, hence 3/4.
Two Archers have separate physical source occurrences; two Fodder casts have
separate event ordinals. The controller's authored bottom-to-top order resolves
in reverse. A failed or cancelled cast creates no cast event or trigger.

Viashino Pyromancer is cast without a target. Its creature resolution produces
an ETB trigger, whose player target is selected during placement. The opponent
legally casts Bite Down using a previously cast Bear Cub (2/2), killing the 2/1
Pyromancer. The independently existing ability still deals two to the selected
player. Its source remains the battlefield incarnation (3), while the physical
card in the graveyard has incarnation 4. Rebinding the ability to incarnation 4
violates CR 113.7a/400.7.

Expected final life: Archer/Cyclops [20,19]; two Archers triggering twice [20,16];
Pyromancer killed in response [20,18]. All prefixes start with normal reset,
complete frozen deck multisets and explicit shuffle/keep choices. No trigger is
injected into them. Pending, placement and settled observations must retain
engine provenance; raw staging differences are not treated as agreement.

The frozen pool does not produce simultaneous opposing-controller triggers in
these played prefixes. Existing GH-211 native/APNAP and actual reference
composition scenarios remain explicitly synthetic and retain their original
oracle, setup and acceptance. They are supplemental integration evidence, not
claimed reachable played prefixes or complete games.
