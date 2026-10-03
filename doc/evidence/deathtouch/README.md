# Thornweald combat and Bite deathtouch — GH-199

Candidate implementation; completion requires separate review, protected merge
and successful CI on the exact merged main commit. The [issue workpad](https://github.com/pabloxrl/mtg-lab/issues/199#issuecomment-5965196467)
records final receipts. M2 and whole RFC requirements remain incomplete.

Pinned Thornweald Archer is **1G, 2/1, reach and deathtouch**. Oracle text hash
`37c215ecee9970d5b798af2ae9189bbd26cf6246c56594bb985028725c01a2e4`
in `data/cards/foundations_micro_v1.json`. Independent expectations follow
CR 702.2b/c, 702.19b, 510, 704.5h and Bite's pinned damage text, plus CR 608.2b
for a departed source. Neither card/rules pins nor original catalog change.

## Behavior and reproduction

Thornweald's positive damage kills a creature in combat or through Bite. Zero
assigned damage is not a damage event. Combat gathers every damage assignment
before destroying any creature; a lethally hit source still deals its damage.
Bite uses its creature source's deathtouch, with no return damage. An illegal
source at resolution supplies no damage. Trample lethal requirements are one
for a deathtouch source, counting already marked damage; without trample no
power can reach the defender through a remaining blocker.

No new command or alternate rules implementation is introduced. Scalar work,
policy Choice, semantic actions, snapshots/replay and typed trajectory conversion
share existing paths. Pending deaths are owned work items, including at snapshot
yields. Conservative source fingerprints reject earlier snapshots/replays; no
migration is claimed. Native policy IDs are `heuristic-deathtouch-v1` and
`legal-random-deathtouch-v1`; existing scoring/distributions consume the lethal
domain and now accept Thornweald casts.

```sh
cargo test --locked -p mtg-core deathtouch
cargo test --locked -p mtg-recorder --test deathtouch --test deathtouch_allocation
cargo test --locked -p mtg-policy deathtouch
python3 scripts/deathtouch_reference.py --cache /home/agent/.cache/xmage --output /tmp/deathtouch-reference
./scripts/torture.sh
```

`red.log` preserves three compiled unsupported-behavior failures. After adding
the pinned card definition, `rules-red.log` preserves three compiled assertion
failures: damage occurred but the 5/5/Sentry survived. These are behavioral reds,
not compilation failures. Build errors during test authoring are not evidence.

Normal discovery retains:

- `deathtouch_thornweald_blocks_five_five_flyer_simultaneously`: synthetic flying
  5/5 and real Thornweald both die, player life unchanged.
- `deathtouch_zero_split_and_no_trample_rejection`: all three 2-power splits
  across two 4/4 Sentries; only positive recipients die, eight return damage
  kills Thornweald, and incomplete allocations reject without mutation.
- `deathtouch_bite_kills_sentry_without_return_damage`: two kills the 4/4 while
  Thornweald remains 2/1 with zero marked damage.
- `deathtouch_trample_synthetic_seven_six_both_seats_quantum_and_rejection`:
  explicitly test-only +5/+5 and trample, 1/1/5 allocation; eight return damage
  kills Thornweald. Both seats, quantum 1/unbounded, snapshots at pending
  allocation and internal yields, semantic action and policy lethal domain.
  Duplicate, foreign, excessive, missing lethal, stale and wrong-actor inputs
  reject without state mutation.
- `deathtouch_normal_reset_combat_bite_capture_replay`: real green-deck land
  plays and payments cast Thornweald and Sentry; combat trades, or Bite kills
  Sentry with no return damage. Seed 199, capture on/off and quantum 1/unbounded
  histories agree; privileged replay, typed JSONL roundtrip and [1,-1] terminal
  rewards are checked using existing Driver/Run.
- `deathtouch_normal_reset_two_sentries_capture_replay`: real Thornweald and two
  Sentries are cast normally; one damage each kills both Sentries and eight
  return damage kills Thornweald. Full snapshot/RNG/history/capture nonmutation,
  typed malformed-allocation rejection, replay and JSONL equivalence.
- Native policy tests check heuristic 1/1/5 and every legal two-blocker 7-power
  random allocation, with independent integer enumeration.

## Catalog and reference boundary

[Copied catalog clauses](catalog.json) preserve IDs, owners and expectations.
The shared [fixture](../../../fixtures/reference/deathtouch.json) and
[literal expectations](../../../fixtures/reference/deathtouch-expectations.json)
run in `deathtouch_reference_literal_checkpoints` and pinned XMage
`DeathtouchTest`, twice per engine with field mutation controls.

| Original catalog case | Executed case or retained composition obligation |
| --- | --- |
| Thornweald interaction | `bite`: real Thornweald damage kills Sentry |
| Thornweald negative | `no-trample`: blocked Archer cannot send one to player |
| targets/zone-change interaction | `source-gone`: responding Bite kills Thornweald; original Bite does no damage to Cub |
| combat/deathtouch-trample regression | `growth-no-trample`: real Growth then blocked Archer cannot send four to player |
| combat/deathtouch-trample interaction, Tajuru negative | Existing trample suite's `marked` / `negative`, rerun with unchanged expectations |
| Thornweald positive, flying/reach interaction, simultaneous-damage interaction, Shivan interaction | `flyer` verifies synthetic 5/5 flyer and Thornweald trade; unchanged Shivan compositions await #200/#210 |
| Thornweald regression, deathtouch-trample positive/negative | `trample-sentries`, `trample-cubs`, `trample-negative` verify explicit synthetic +5/+5/trample; unchanged real Invoker activation compositions await #201/#210 |
| Bite interaction against Shivan | Sentry Bite executed now; unchanged Shivan case awaits #200/#210 |
| priority-response interaction, stacked-boosts regression, cleanup interaction, Invoker regression | Require future Invoker/Shivan activations; retained with registered composition packs |

`zero-split` additionally checks that the zero-damage Sentry lives while the
positive recipient and attacker die. XMage negative trample evidence observes
its actual minimum constraint rejecting requested total one. For single-blocker
nontrample cases XMage checks absent trample and actual full blocker damage;
it exposes no allocation prompt. Native rejection/nonmutation is checked directly.
No unsupported or unexecuted case is counted as reference agreement.

Synthetic hooks are compiled only in unit tests. They do not enable Shivan,
Invoker, arbitrary card powers/keywords, counters or a public alternate executor.
Native normal-reset evidence is separate. Upstream XMage pin/provenance and MIT
notice remain in `references/xmage`. No Forge or reference full-game claim.
Original #23/#24/#26 and composition owners retain whole acceptance.

README and policy/API documentation describe thirteen supported cards plus the
Goblin token, keeping M2 incomplete. Unsupported-Thornweald premises in the card
boundary and independent passive collector ledger become literal pinned 1G/2/1
support; all other unsupported identities and rejection assertions remain.
Independent review must check this requirement correction, stronger coverage,
and README accuracy. No CI/workflow/auth/budget changes.

The reference receipts `reference.json`, `trample.json` and
`instant-reference.json` bind executed checks to exact source hashes. Nine new
cases, fourteen retained trample cases and eleven retained instant-response cases
agree twice per engine, with strict negative controls. The bridge's first run
failed because the harness queued spell targets before activating mana; explicit
scripted mana before targeting corrected the setup without changing expected
states. That failed execution is not agreement. `quickstart.jsonl` records the
updated native README command: two completed games, zero failed/incomplete.
Full torture and independent review results are recorded in the issue workpad.
