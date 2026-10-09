# GH-210 prerequisite reference gap

**Historical diagnosis, resolved by #255.** Protected [PR #261](https://github.com/pabloxrl/mtg-lab/pull/261)
merged the exact missing executions as `fb8689267ee5d80750998c55e9580074ac12dc8d`;
[exact-main CI passed](https://github.com/pabloxrl/mtg-lab/actions/runs/37962359957).
The original audit below is retained verbatim after this notice. It is not a
current blocker or a GH-210 completion claim.

Audit base: current main `0f46ba11c1c8902ac7337af3377b132ee361c1ea`.
Status: **not delivered; coordinator split required**. This is a source/fixture
inventory, not a runtime failure, engine defect, or reference agreement claim.

The exact crosswalk assigns 50 catalog slots to #210. Its atomic reference
boundary requires re-execution of existing per-mechanic fixtures plus bounded
cross-mechanic scripts. It explicitly directs missing prerequisite bridge
contracts to coordinator splitting. The following single-mechanic executions
cannot be supplied by the committed fixtures:

| Unchanged catalog ID | Required execution | Existing evidence and gap |
| --- | --- | --- |
| `rules-combat-creature-abilities-positive` | Cast Cub for 1G in own main, resolve, observe untapped and unable to attack that turn | Native `casting_literal_payment_stack_resolution_and_priority` checks untapped/sickness. `CreatureManaTest` payment/floating casts Cub but exports only Cub presence, not its tap state or attack legality; its sickness mode concerns the mana creature. No matched complete Cub checkpoint exists. |
| `rules-combat-creature-abilities-negative` | P1 has priority in P0 turn and attempts non-flash Cub cast; reject | Native `casting_missing_colors_masked_timing_foreign_and_wrong_actor` covers timing. No committed XMage fixture scripts this opponent-turn creature-cast rejection or exports its observed legality/nonmutation. The creature-mana `opponent` case checks priority for a mana ability, not spell timing. |
| `rules-foundations_micro_v1-magnigoth-sentry-interaction` | One Sentry blocks Shivan, receives Growth, survives five damage as 7/7 and kills Dragon with seven | `FlyingReachTest` growth uses a test-modified flying 5/5 Bear Cub, explicitly disclaimed as Shivan evidence. `ShivanTest` grown_split has TWO Sentries and assigns 0+5; the grown Sentry receives zero, so it is not the literal single-blocker case. Neither existing receipt executes the assigned case. |

Sources inspected: `doc/testing/capability-test-plan.json`,
`doc/programs/m2-test-crosswalk.md`, all `fixtures/reference/*.json`, and
`references/xmage/{CreatureManaTest,FlyingReachTest,ShivanTest,HasteTest}.java`.
All original catalog owners and expectations remain unchanged. Related tests
are useful regressions but do not fill an exact-case receipt by inference.

Proposed smallest coordinator split: register a prerequisite reference correction
that adds strict native/XMage fixture execution and observable checkpoints for
these three unchanged cases using real pinned cards. Include legal main-phase
casting, opponent-priority timing rejection/nonmutation, and actual single-Sentry
Growth combat; retain independent CR/Oracle expectations, normal test discovery,
real pinned runs, review and exact-main CI. No new engine mechanic, card, CI policy,
reference-engine expansion or acceptance waiver is requested. Then explicitly
reactivate #210 to compose and rerun the complete assigned pack.

## Preserved draft and checks

The imported native composition test, five-case fixture, stub receipt validator,
Python tests and original failing Python log are retained. This audit session
added literal CR-derived composition expectations and an unexecuted XMage
composition script; they are unreviewed draft material, not acceptance evidence.
The independent-holdout claim still needs review before publication.

`python3 scripts/check_program.py` passed on freshly fetched/integrated current
main. The resumed Python validator tests fail behaviorally (five assertions that
missing/stale receipts must raise); saved as `resumed-python-red.log`. There is no
passing native compilation, reference run, full torture, separate candidate review,
PR or delivery claim. Queued native test/cache preparation commands were stopped
before lock acquisition after the scope gap was confirmed. A resource wait itself
is not the blocker. The installed Java/Maven versions match the reference pins;
the issue's XMage cache was not yet prepared.

README impact: no published command, supported behavior, architecture or verified
milestone changed, so no README update is warranted for this blocked draft.
Preserve all existing tests and this checkout; no resets, test skips, workflow or
program-task registration changes were made.
