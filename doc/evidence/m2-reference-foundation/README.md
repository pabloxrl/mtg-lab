# Full-pool legacy and decision coverage

Status: bounded native/reference acceptance and full torture PASS after integration
of main `17aeefe91c9a23bd91854a8e8c3d7960a06b6be4`. Independent review and protected
delivery are tracked in the issue/PR workpad; no M2 gate verdict.
The earlier prerequisite audit in this directory is historical. Repairs #256 and
#257 are verified on main `362aad0eeb5c7ab38d4b710de3f63c266ec48a52` and supply
the ordinary Sentry/Bite and mixed-reason terminal contracts.

## Scope and independent expectations

The [assignment](../../../fixtures/reference/m2-foundation-assignment.json) retains
all 31 catalog cases verbatim, their original owners and explicit native/reference
observation boundaries. Native raw-input, privacy, capacity and snapshot contracts
are not XMage API contracts. A bounded reference mapping does not assert that
XMage exports every native intermediate field or enumerates all policy choices.
The original component #24 and gate #26 retain full acceptance.

New native tests independently enumerate the integer solutions of `x+y=5`:
`(0,5),(1,4),(2,3),(3,2),(4,1),(5,0)`. CR 510.1c permits every split without an
assignment-order/lethal-first constraint. A 5/5 Shivan dies to eight damage from
two 4/4 Sentries; a blocker dies exactly when assigned at least four. Otherwise
its power/toughness remain 4/4 and the assigned amount is marked damage. Both
seats, policy submission, private provisional selections and restore are covered.
Six distinct old Goblin tokens give `2^6=64` attacker subsets, including empty
and full. Every accepted subset must commit exactly its physical tokens/taps.

Capacity is the explicit caller-supplied domain bound, not a claim that 80 tokens
is the pool's maximum reachable board. Boards with 79/80/81 distinct tokens must
return complete domains or explicit overflow. The policy adds its finish row.
Raising the bound must recover the final token; no candidate-prefix fallback is
permitted. Fresh Cub/Swab attack and masked Cub-versus-Shivan block attempts reject
through raw scalar calls without mutation. Native Invoker/Thornweald composition
checks both blocker-step priority windows and an actual responding Growth. The
Archer composition kills its source with an actual opposing Bite above the trigger
and verifies exactly one life loss before the lower Growth resolves.

Existing Thrill pending-discard snapshots, cancellation, wrong-seat/stale rejection,
ordered draws and work-quantum tests are retained. Existing hidden-library twins,
policy error masking and restore-generation tests are re-executed as supplemental
native coverage. No new production rule, alternate driver, learner or batch/Python
integration is introduced. These new tests specify existing behavior; no missing
rule/behavioral red or production repair is claimed.

## Frozen holdout

The [input](../../../fixtures/reference/m2-foundation-holdout.json) and
[literal expected checkpoints](../../../fixtures/reference/m2-foundation-holdout-expectations.json)
were authored by the GH-212 coverage worker independently of the prerequisite
mechanic implementations, before execution using the pinned Oracle definitions and CR
608.2b/h, 120.6, 400.7 and 117. Source and destination are distinct opposing
Magnigoth Sentries with the same name. P0 casts Bite; P1 responds with Growth on
its own Sentry. Growth resolves first: 7/7; Bite then deals source power four,
leaving 7/7 with four damage and the source unmarked. Life remains 20/20. Casts
consume the stated targets, mana and passes; stack, zones and physical identities
remain distinct. Expectations reuse the existing literal script schema but are
not generated from engine observations. Independent review must assess this
rules-derived oracle and its independence from production implementation.

This is a synthetic response/identity holdout, not a complete normal-reset game
or an independent third engine. Once frozen it remains part of normal discovery.
The real native/XMage comparison uses the existing strict instant bridge with no
new bridge operation. Corrupt damage, power, owner, omitted checkpoint and missing
choice controls must be detected. Retained normal-reset Driver/Run, semantic replay
and typed recorder tests continue in full torture; synthetic positions do not
replace their reachability evidence.

## Reproduction and evidence

Within the managed container, using this issue's isolated pinned reference cache:

```sh
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- python3 scripts/m2_foundation_pack.py --cache "$MTG_REFERENCE_CACHE" --output /tmp/m2-foundation
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- ./scripts/torture.sh
```

The pack executes prerequisite native/XMage runners and their negative controls,
all core tests, every mapped native assertion, and the frozen holdout twice per
engine. A failed child, missing observed case, absent named native test or corrupt
checkpoint prevents a joined receipt. It reports mapping counts by capability;
31 assigned IDs are not 31 distinct scenarios and do not certify the scenario
floor. Full workspace torture, candidate-bound independent review, protected
integration and exact-main CI remain separate delivery requirements.

| Legacy card | Fresh mapped reference coverage |
| --- | --- |
| Forest | Instant target/payment/response scripts, including holdout |
| Mountain | Instant explicit generic payment and cast-trigger land/mana negatives |
| Bear Cub | Ordinary Bite/Growth scripts and strict Shivan-pack actual cast/sickness |
| Swab Goblin | Vanilla combat and Surprise-boosted trade; fresh-attack rejection native |
| Giant Growth | Response-order, current stats and damage/cleanup ledgers |
| Bite Down | Ordinary Sentry damage, friendly rejection, departed source and deathtouch |

Earlier M1 gate receipts remain byte-identical and are linked/hashed in the joined
receipt as retained evidence, never counted as newly executed M1 gate acceptance.
No Forge, full reference-game, per-field parity, full RFC block, performance or
M2-completion claim belongs to this delivery. README links only this
verified bounded coverage; the milestone remains incomplete.

## Executed bounded reference receipt

The [integrated joined receipt](integrated/reference-receipt.json) records all 31
assigned native cases, 29 bounded XMage case mappings (capacity encoding is
native-only), twelve prerequisite packs and the frozen holdout. Every mapped
native test passed in normal core discovery. The holdout agreed twice per engine
and its corrupted damage-event checkpoint failed comparison. Twenty focused
native M2 tests passed; these do not replace full torture.

The [compressed raw evidence](integrated/reference-artifacts.tar.gz) retains all
760 files: inputs, observations, consumed choices, positive/negative execution
logs, source pins, child receipts and comparator controls. The
[archive inventory](integrated/reference-archive.json) records its SHA-256; all
759 nested receipt artifact hashes and 611 child native source fingerprints were
checked against the integrated candidate. Extract with
`tar -xzf reference-artifacts.tar.gz`; paths in the joined receipt are relative to
the resulting `references/` directory. Earlier M1 receipts have their original
hashes in the joined report and remain unchanged in the repository.

Full torture passed **246 Python tests and 1,408 Rust debug/release executions,
zero failed/ignored**, after integration of current main; see the
[validation receipt](integrated/validation.json) and [full log](integrated/torture.log).
The tested source head is `118c56cf5d7546366bf47d462c5583eb0a7458c6`; subsequent
evidence packaging changes only this documentation and retained artifacts.
Reference execution took 596.909 seconds and full torture 857.299 seconds in the
managed container. These are verification runtimes, not throughput claims.

The earlier [receipt](pre-integration/reference-receipt.json),
[archive inventory](pre-integration/reference-archive.json),
[raw reference archive](pre-integration/reference-artifacts.tar.gz),
[validation receipt](pre-integration/validation.json) and
[full log](pre-integration/torture.log) retain the prior-base run: 240 Python tests,
1,406 Rust executions, zero failed/ignored. Main advanced via PR #264 during that
run; it was integrated with both independent test modules and README links
preserved, and the complete reference/torture sequence was repeated above.
Independent review, protected integration and exact-main CI remain separate
requirements whose final evidence belongs in the delivery workpad/PR.
