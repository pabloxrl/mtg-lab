# GH-76 in-memory trajectory acceptance

Scope: canonical native records, B036/B037 atomic in-memory portions. M1 remains
incomplete. No Magic rules implementation changed; mature-engine rules outcomes
were delivered in #72. This change tests the research reward/storage contract,
which the existing reference bridges do not export. No new upstream agreement
or full-game recorder coverage is claimed.

Independent basis: RFC 0002 §8 B036/B037; CR 510.2 (vanilla 2/2 simultaneous damage),
704.5a (life <= 0 loses), 104.4a (simultaneous loss draws). Original synthetic
observation ledger from DRL-001/004: actors `[0,0,1,1,0,1]`, links for P0
`0→1→4→final`, P1 `2→3→5→final`, rewards all zero except `[+1,-1]` at the final
boundary. Distinct logical IDs touched per interval are independently counted.
No expected result was generated from recorder output.

## Exact catalog mapping

| Case | Normal-discovery test | Independent expected result |
| --- | --- | --- |
| rules-terminal-rewards-once-positive | `trajectory_rules_terminal_rewards_once_positive` | Synthetic unblocked Cub, P1 life 2 → 0; `[1,-1]`, one terminal boundary, no truncation, nonacting P1 reward retained. |
| rules-terminal-rewards-once-negative | `trajectory_rules_terminal_rewards_once_negative` | Synthetic P1 life 5 → 3 after 2 damage; `[0,0]`, live boundary. Catalog describes the post-damage life three. |
| rules-terminal-rewards-once-interaction | `trajectory_rules_terminal_rewards_once_interaction` | Synthetic both life zero, real SBA settlement; draw, both rewards zero, one terminal boundary even with no actions. |

All live game mutations in these tests run the existing core combat/terminal
mechanics. Synthetic construction is explicitly labeled, not evidence of a
normal-reset full match.

## In-memory systems/DRL coverage

The nine additional original tests plus the conventions test in
`crates/mtg-core/src/trajectory_tests.rs` cover these relevant portions:

- SYS-DATA-001/002, DRL-001/004: exact decision/action/mask features, same-seat
  links, nonacting winner, logical/micro indices, durations, draw/concession,
  once-only returns, both winning seats, zero-decision seats, repeated reads.
- SYS-DATA-004, DRL-003/006: input metadata/features/mask/view mutations cannot
  alter captured data; final views survive explicit reset; cross-episode native
  token mismatch rejected without mutation.
- DRL-002: three external limit reasons; final live bootstrap target 0.75 under
  gamma=1; failed/incomplete readers rejected, completed-game/failed-recording
  distinction retained diagnostically.
- SYS-DATA-010, DRL-007: version/hash/policy presence and schema rejection,
  explicit reward/discount/capture conventions, exact optional statistics,
  absent probabilities rejected by consumers requiring them.
- DRL-008: unique opponent-private sentinel excluded from same-seat sequences,
  no replay/policy metadata in default sequence output.
- SYS-DATA-005 relevant all/off portion: normal reset, two keeps and four real
  priority passes under seed 773/episode 21; capture on/off identical player
  views and every serialized semantic state field including RNG. Native scope
  IDs are allocation capabilities, explicitly normalized as in snapshot tests.

Full JSONL/replay round-trip, complete scalar CLI driver, cross-feature pending
spells and full M1 ledger auditing remain #77/#20/#19. Batch schedules, seed-subset
capture, fragment/recurrent loading, durable failures and trainer integration
remain their registered later-stage owners. These are partial SYS/DRL mappings,
not whole-system acceptance claims.

## Red/green and test corrections

`red.txt`: seven compiled behavioral failures against no-op recorder methods,
including the three exact catalog cases. `conventions-red.txt`: compiled missing
reward-convention assertion. `sequence-red.txt` adds a compiled assertion for
missing per-seat discount/version metadata; fixed before the final full run. All assertions remain in ordinary Cargo discovery.
`green.txt` records the final focused run. `torture.txt` records the full suite
in the managed Docker worker (no nested Docker invocation).

Two new test setups initially failed for reasons unrelated to the implementation:
1. The opponent marker `mountain` also appeared legitimately in the native P0
   hand. Replaced the synthetic P1 marker with `opponent-private-sentinel`; the
   unchanged isolation requirement now has a unique marker.
2. Raw snapshots of separately allocated games necessarily differ in capability
   scope/store IDs and the resulting envelope hash. The equality check now uses
   exactly the existing snapshot-test normalization (scope/store and objects.id),
   retaining all semantic fields including RNG, decisions and generations. The
   independent reviewer must explicitly examine this correction and its coverage.
An initial extra `.unwrap()` on the byte-vector snapshot API was a compile error,
corrected before execution; it is not red behavioral evidence.

Three compiled mutants must fail named behavior assertions: reverse terminal
seat rewards, substitute opponent final view, and drop zero-decision-seat credit.
See `mutations.json` and individual mutation logs. Rust owned values and immutable
borrows enforce alias isolation; runtime input-mutation tests additionally check
all captured value categories. No test is skipped, deleted or weakened to deliver.

README impact: added current recorder API/limitations/evidence; removed obsolete
claims that rewards are wholly absent. Milestone table and quickstart commands
unchanged. Full suite exercises the inner quickstart validation; no Docker socket
or host installation used. Independent review and exact merged-main CI receipts
are recorded in the issue workpad and PR, not inferred from this report.

Final local verification: `cargo test -p mtg-core trajectory` passed 13 tests;
`./scripts/torture.sh` passed 132 Python tests, 163 Rust tests plus one doctest
in each debug/release profile, documentation/program/catalog checks, fmt and
Clippy. Fresh origin/main integration reported already up to date at baseline
`8f288fd9b487b95e4ed06c522f8e8770d6ef899b`; the full run was repeated afterward.
