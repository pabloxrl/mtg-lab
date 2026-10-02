# Creature mana acceptance — GH-195

Candidate behavior: Llanowar Elves (G, 1/1) and Druid of the Cowl (1G, 1/3)
can be cast, fight in combat, and tap for G at priority or during casting.
CR 302.6 forbids sick tap-symbol activations; CR 605 makes these abilities
immediate, with no stack entry or opponent response; CR 601 governs payment.
The pinned [card manifest](../../../data/cards/foundations_micro_v1.json)
supplies independent characteristics and Oracle text. Full delivery remains
conditional on separate review, protected merge and exact-main CI in the issue
workpad. No whole RFC block or M2 gate completion is claimed.

## Executable evidence

The [compiled behavioral red](red.log) predates the rules implementation:
old Elf is absent from mana sources, and Elf plus Forest cannot pay for Cub.
The same tests now pass. They retain sickness, ownership, stale-handle, tapped,
duplicate-source, cancellation and rejection-nonmutation assertions.

| Obligation | Normal-discovery executable |
| --- | --- |
| Immediate G and no stack/response; source restrictions | `casting::tests::creature_mana_priority_is_immediate_and_rejects_illegal_sources` |
| GG Cub payment and cancelled payment preserves taps/pool | `casting::tests::creature_mana_staged_cancel_and_commit_are_atomic` |
| Pending snapshot at every payment stage; finish/cancel and final source revalidation | `casting::tests::creature_mana_pending_snapshot_and_illegal_payment_sources` |
| Sick Druid blocks Cub; 1/3 with two damage, Cub with one | `combat::tests::creature_mana_druid_blocks_cub_and_survives_two_damage` |
| Pinned costs and both printed bodies | `card_definition_tests::card_definitions_six_card_manifest_and_public_candidates` (extended beyond historical M1 six) |
| Both creatures really cast after normal reset; sickness, cancellation, semantic replay and capture on/off | `tests/capture.rs::creature_mana_normal_reset_capture_cancel_and_replay` |
| Existing Driver/Run typed conversion, JSONL roundtrip, exact trajectories with work quantum 1/MAX | `mtg-recorder/tests/creature_mana.rs::creature_mana_typed_roundtrip_and_quantum_equivalence` |
| Independent scalar payment enumeration | unchanged `mana_all_small_payment_combinations_independent_enumerator` |
| Shared native/XMage catalog scenarios | `creature_mana_reference_tests::creature_mana_reference_literal_checkpoints` and `references/xmage/CreatureManaTest.java` |

The [shared input](../../../fixtures/reference/creature-mana.json) and
[hand-authored oracle](../../../fixtures/reference/creature-mana-expectations.json)
cover 13 cases. [Pinned reference receipt](reference.json) records two real
native/XMage runs and comparator controls for mana, stack, priority, taps,
stats, damage, Cub presence and life. The first Java execution failed because
its strict sick-creature casting controller omitted Forest payments; explicit
Forest activations corrected the bridge. That execution is not a reference pass.
The comparison is synthetic setup, not full-game or Forge agreement. Native
normal-reset tests independently provide played evidence. XMage's opponent
case observes the priority restriction, not a fabricated out-of-turn callback;
actual rejected out-of-turn submission is executed natively.

## Unchanged catalog execution allocation

All original owners/assertions in the [M2 crosswalk](../../programs/m2-test-crosswalk.md)
remain authoritative. This mechanic executes the following now; #209 retains
its aggregate/composition audit:

- `rules-costs-creature-mana-{positive,negative,interaction,regression}`:
  priority, sick, floating Druid payment and opponent-priority cases.
- `rules-costs-mana-rejection-interaction`: floated G plus Elf payment.
- `rules-foundations_micro_v1-llanowar-elves-{positive,negative,interaction,regression}`:
  priority, real newly cast sick Elf, Elf plus Forest, and attacked Elf in second main.
- `rules-foundations_micro_v1-druid-of-the-cowl-{positive,negative,interaction}`:
  priority, real newly cast sick Druid, and Druid blocking Cub.

Druid's Cavalry haste regression and the Cavalry/Elf composition require the
future haste mechanic and remain #209 composition obligations. This delivery
does not implement haste or nonmana activated abilities.

Support expansion necessarily corrects two old unsupported-content assumptions:
the definition test now independently asserts both cards' pinned costs/bodies,
and the casting unsupported-card probe uses still-unsupported Axgard Cavalry
instead of newly supported Elf. No regression is removed/skipped; independent
review must explicitly assess this requirement correction and replacement.

## Reproduction

```sh
cargo test -p mtg-core creature_mana
cargo test -p mtg-recorder --test creature_mana
python3 scripts/creature_mana_reference.py --cache /home/agent/.cache/xmage --output /tmp/creature-mana-reference
./scripts/torture.sh
```

Use the existing `Choice::TapMana`, semantic mana action and staged cast payment
commands. No new wire variant or side implementation is introduced. Source
fingerprinting intentionally rejects earlier-engine snapshots/replays. The existing native policies now explicitly accept these sources and casts;
`legal-random-mana-v1` / `heuristic-mana-v1` version that expanded domain.
The prior token policy IDs reject explicitly, and the native quickstart selects
the new IDs. Sampling and existing heuristic scores/ties are unchanged;
full-pool policy qualification remains #208. Quickstart command syntax is unchanged; the native configuration selects the new IDs.

The first full torture run caught the existing policies rejecting the newly
enabled sources. [Compiled policy red](policy-red.log) reproduces the explicit
UnsupportedContent failure before the bounded policy fix. Both policy tests now
pass. Full torture results for the complete candidate are recorded in the
final issue workpad/PR; neither focused tests nor reference agreement replace it.

The updated native quickstart completed both requested green-mirror games with
zero failures, truncations or incomplete episodes. Its output includes both
Elf and Druid in actual play. [Quickstart summary](quickstart.json) records the
command, policy versions, output checksum and run accounting.

Independent review of candidate `d821c680a1c8756634c0e9c279a639b7a7668373`
requested correction of an old heuristic assertion that Elf was unsupported.
The rejection now explicitly uses unimplemented Axgard Cavalry; the new Elf/Druid
positive selection tests remain. The reviewer also identified a stale README
limitation, corrected to reference creature mana support. Both findings are
retained in the final PR review history; the revised candidate requires a fresh
review and full torture pass.
