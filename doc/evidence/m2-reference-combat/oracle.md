# Combat composition oracle and provenance

Status: authored expectations; execution and delivery receipts are separate.
The original catalog and its owners remain unchanged in
`fixtures/reference/m2-combat-pack.json`. Rules revision and Oracle sources are
pinned by `doc/rules.md`, the capability catalog and the frozen card manifest.
These are original scripts using the existing Game and XMage operations, not
adapted upstream test scenarios or a second rules implementation.

The transferred draft's five scripts and literal expectations were preserved
before this resumption's execution. The inherited holdout was authored before
observing either engine, in the earlier GH-210 session. Its arithmetic is reviewed
here independently from the bridge; final prescribed review must assess it too.
No output from either engine is used to author an expected checkpoint.

- **invoker-cubs:** Thornweald starts 2/1. Eight generic mana puts a targeted
  ability on the stack; resolution gives +5/+5 and trample, hence 7/6. CR 702.2c
  and 702.19 permit one assigned to each Cub and five to the defender. Both
  Cubs die; four simultaneous damage leaves Archer alive. Cleanup removes
  damage and the modifier/keyword together (CR 514), leaving 2/1, no trample.
- **invoker-cubs-reject:** 0+1 to blockers leaves six for the player, but one
  blocker has not been assigned lethal. Native rejects without snapshot change.
  XMage exposes minimum lethal sum two via its constrained allocation API;
  this is a legality-domain observation, not a raw rollback assertion.
- **double-growth:** two actual +3/+3 spells make Cub 8/8 (CR 613.4c); cleanup
  restores 2/2. Mana decreases from two green to one to zero.
- **blocked-growth:** synthetic departure removes the sole declared blocker;
  actual Growth makes the nontrampling attacker 5/5. CR 509.1h/510.1c retains
  blocked status, so no damage reaches the player. Cleanup restores 2/2.
- **holdout-invoker-growth:** two real Invoker resolutions make Thornweald
  12/11; a real Growth makes it 15/14. Two Sentries each require only one
  assigned damage because of deathtouch, leaving thirteen for the defender.
  Eight simultaneous return damage does not kill the Archer. After cleanup
  it is 2/1 with no damage or temporary trample. Duplicate trample never
  multiplies damage. Seventeen green pays the two activations and Growth.
- **cleanup-shivan-thorn:** one battlefield contains Dragon and Thornweald.
  Invoker consumes eight green and makes Archer 7/6 with trample. Dragon's
  own red activation makes it 6/5. Common cleanup restores 5/5 and 2/1,
  retains printed flying/reach/deathtouch and removes temporary trample. The
  prerequisite Invoker `thorn_cleanup` already combines these creatures with
  source death; this added script supplies finer intermediate stack/mana points.

All positions are explicitly synthetic, including direct blocker departure and
initial mana/native turn placement. Actual spells, abilities, damage, state-based
actions and cleanup use existing engine rules. These scripts do not count as
normal-reset played games; existing prerequisite Driver/Run/recording tests
remain mandatory in full torture. Current CR 510.1c has no damage assignment
order; the six actual Shivan split scripts are rerun unchanged.

Comparator controls preserve a one-field wrong player-damage checkpoint and a
retained temporary keyword at cleanup, with the first differing named checkpoint.
The old `prerequisite-gap.md` is historical: #255 supplied the exact missing Cub
and single-Sentry scripts through protected PR #261, verified on main
`fb8689267ee5d80750998c55e9580074ac12dc8d`. It does not waive any assigned slot.
