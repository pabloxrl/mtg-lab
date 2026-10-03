# Tajuru vigilance and trample allocation — GH-198

Candidate implementation; completion is conditional on separate review, protected
merge and passing CI on the exact merged main commit. The [issue workpad](https://github.com/pabloxrl/mtg-lab/issues/198#issuecomment-5964375215)
records those final receipts. M2 and whole RFC blocks remain incomplete.

Pinned Tajuru Pathwarden is **4G, 5/4, vigilance and trample**. Source:
`data/cards/foundations_micro_v1.json`, Oracle text hash
`e5ce26740a0cda934aa26522367252302e43b949d2b5626db2f66210862794db`.
Independent expectations follow CR 510, 702.19b/c, 702.20b and the Foundations
removal of damage assignment order. No card/rules pin or original catalog changes.

## Behavior and reproduction

Tajuru attacks untapped but still needs to satisfy summoning sickness. The existing
`assign_combat_damage` / policy `assign_damage` command assigns amounts to surviving
blockers. For trample, any unassigned power goes to the defender, allowed only when
every blocker has lethal assigned after counting marked damage. Assigning the whole
power to blockers permits every nonnegative split, including nonlethal splits.
One surviving blocker now needs a trample choice; no remaining blockers assigns all
power to the defender. No extra priority window or blocker order is introduced.

The same scalar work, semantic action, snapshot/replay and typed trajectory paths
carry this behavior. `trample_lethal` is an optional per-blocker factored domain;
legacy nontrample serialized shapes remain unchanged. Conservative engine source
fingerprints reject old snapshots/replays; no migration is claimed. Native policies
use `heuristic-trample-v1` and `legal-random-trample-v1`; prior versions reject.

```sh
cargo test --locked -p mtg-core trample
cargo test --locked -p mtg-core tajuru
cargo test --locked -p mtg-recorder --test trample
cargo test --locked -p mtg-policy
python3 scripts/trample_reference.py --cache /home/agent/.cache/xmage --output /tmp/trample-reference
./scripts/torture.sh
```

`red.log` retains two compiled behavioral failures (`UnsupportedCombat`) before
implementation. `policy-red.log` retains the compiled heuristic failure to send
excess damage to the defender. Build/API errors encountered while writing tests
are not counted as behavioral red.

- `tajuru_vigilance_trample_and_unordered_blocker_splits`: 2+2+1 kills both Cubs
  and Tajuru; all six blocker-only splits accepted; invalid allocations unchanged;
  selected allocations survive snapshot restore.
- `tajuru_departed_blockers_and_single_blocker_marked_lethal`: departed blocker
  means five to defender; one marked damage means one to Cub/four to defender.
- `trample_both_seats_quantum_restore_and_atomic_rejections`: both seats,
  scalar/quantum-1, restore during each internal yield, duplicate/foreign/overflow/
  insufficient/stale/wrong-actor rejection, opponent provisional-choice privacy.
- `trample_normal_reset_two_cubs_capture_replay`: fixed green mirror deck order,
  seed 198; real land plays/payments cast two Cubs and Tajuru, turn 11 attack,
  untapped attacker and 2+2+1 allocation; literal life [20,19], all three dead.
  Driver/Run, quantum 1/unbounded, capture on/off, semantic history, privileged
  replay and typed JSONL roundtrip agree. Invalid allocation preserves full
  snapshot/RNG, history and capture. Genuine concession supplies [1,-1] rewards.
- Policy tests require heuristic lethal allocation and every legal random split
  over two Cubs, with malformed lethal-vector rejection.

## Unchanged catalog execution

[Exact copied catalog clauses](catalog.json) retain original owners and expectations.
`trample_reference_literal_checkpoints` and the real pinned `TrampleTest` execute:

| Catalog suffix | Shared reference case |
| --- | --- |
| Tajuru positive / negative / interaction / regression | unblocked / negative / two / departed |
| haste-vigilance negative / interaction | sick / next-block |
| deathtouch-trample interaction (Tajuru with marked Cub) | marked |
| blocked-status interaction | departed |

Fourteen original synthetic reference cases also include one fresh blocker and all
six blocker-only splits. Expectations are authored from the above rules, never
copied from engine output. Each executes twice per engine with strict scripted
choices and comparator mutation controls. XMage negative allocation evidence is
its actual minimum constraint (2) rejecting requested sum 1 before assignment;
native tests additionally prove nonmutation. Setup/departure/marked/sickness hooks
are explicitly synthetic; native normal-reset play is separate evidence.
Upstream `000d8a7abc0ac31cc24af08691423e0c24dc59e7` provenance and MIT notice remain
in `references/xmage`. No Forge or reference full-game claim.

Future deathtouch, Invoker and Shivan compositions remain with their registered
mechanics and #210; full-pool policy qualification remains #208. #23/#24/#26 retain
full original acceptance. No deathtouch/flying implementation, new executor,
workflow/CI/auth/budget change or test weakening belongs to this delivery.

The old unsupported-Tajuru premise in the support table is replaced with pinned
4G/5/4 support assertions and stronger combat/rejection coverage; every remaining
unsupported card still rejects. The independent reviewer must explicitly assess
that requirement correction and README accuracy. README, policy docs and the
native quickstart configuration are updated for new support and policy versions.

The existing independent passive collector ledger also gains literal pinned
Tajuru 5/4 characteristics in visible zones and a masked cast row (no mana is
produced). Its full-field comparisons, rejection controls and terminal outcomes
remain intact. This replaces the former unsupported-card premise, not the
ledger with implementation-generated expectations; independent review is required.
