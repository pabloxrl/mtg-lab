# Atomic M2 audit repair registration

Operations #268 registers thirteen bounded repairs after reconciling freshly fetched
main `1df9b65e95b2976ef9eeb3e46b66877580e2e20c`. This delivers contracts and a graph,
not implementation, newly executed engine disagreements, completed audits or an M2
verdict. [Complete child contracts](m2-audit-repair-contracts.md) are part of this
registration; the tables there specify owned interfaces, runnable positive/negative
acceptance, independent oracles, exclusions and concurrent siblings for EACH child.

## Reconciliation and preserved evidence

The [original archives and receipts](../evidence/m2-audit-repair-registration/README.md)
retain the exact unsuccessful diagnoses. #24's catalog inventory is 175 designs
with unchanged owners and assertions; it is not evidence that fewer than 100 suitable
scenarios exist. Neither audit executed a new rules disagreement. Original source
pins, test expectations, all 129 existing tasks, requirement text and owners survive.

| Diagnosis | Fresh-source interface evidence | Decision |
| --- | --- | --- |
| Opening lacks occurrences and real reset | `OpeningCountsTest.java::openingCounts` injects seven-card hands and 33 basics, sets `skipInitShuffling`, uses no-op `shuffleLibrary` and first hand UUID for bottoming. `opening_reference.py` accurately disclaims identity/order and full games. | Confirm missing new bridge contract, preserve count regressions. Split reset #269 from London choices #270. |
| Native initial order does not specify physical copy binding | `opening.rs::DeckConfig.order` is card-key strings; `apply_with_order` already takes physical handles for later mulligans; object `semantic_identity` distinguishes birth/incarnation. `played_replay.rs` remembers witnessed incarnations. | Version occurrence binding before initial permutation; consume it in actual reset in #269. No schema-only task or guessed same-name matching. #270 reuses the delivered handle-order hook. |
| Existing synthetic packs are not a continuous controller | `InstantResponseTest` rejects mulligans and its `priority`, `chooseTarget`, `playMana`, attacker/blocker and allocation callbacks start from injected state. `ThrillTest`, `SurpriseTest`, `CastTriggersTest`, `EtbTriggersTest`, `ShivanTest`, `InvokerTest`, `CleanupTest` expose distinct callback families; some directly resolve stacks or inject triggers. | Reuse API knowledge, not synthetic reachability claims. Split #271–#276 at actual choice families, with checkpoint identities owned alongside each family. No monolithic controller task. |
| Scalar sampled latency missing | `metrics.rs::Counters::report` exports `sampled_timing: not_measured`. `native.rs::PolicyTiming` has injected boundary clocks, separate policy/encoding totals, and a real collector consumer, but no bounded sampled histograms. | #278 implements collection plus summary through that existing consumer. A separate histogram schema would not independently deliver the missing behavior. |
| Frozen benchmarks reject two modes | `benchmark.rs::Config::validate` restricts both `scalar-windows-v1` and `scalar-full-pool-v1` to off/counters; tests deliberately assert rejection. Native Config already accepts trace configuration and Driver supplies diagnostics/replays. | #279 adds a NEW execution version using existing modes; it does not depend on #278. Preserve historical rejection tests unchanged. |
| Encoded/fixed-trace costs incomplete | Native `PolicyTiming.encode` already builds features; benchmark exposes encoding but the published #217 campaign keeps it off. Application and legal/view are separate boundary totals; effect dispatch needs direct profile evidence. | #280 provides validated trace/profile execution using existing actions and boundary hooks; #281 measures encoded/unencoded and four-mode workloads only after contracts exist. No new encoder or inference work. |

No false blocking diagnosis was found. Clarifications: old bridge/metric deliveries
remain valid for their advertised bounds; missing bridge support is not a rules
failure; provisional throughput misses and unavailable designated hardware are not
the blocker. No duplicate diagnostic build/reference run is needed for this source
reconciliation. Historical full torture/review receipts are archived, not relabeled
as current registration validation.

## Executable graph and independently testable boundaries

Every child has explicit #268 prerequisite. #268 requires delivered #80, retaining
verified M1 gate #22 transitively. Existing delivered dependencies below were
re-fetched as completed with acceptance/review/merge and successful exact-main CI.

| Child | One observable delivery | Direct prerequisites and API/contract reason | Independent sibling work |
| --- | --- | --- | --- |
| #269 | Actual occurrence-preserving reset through first opening declaration | #268 registration; #64 actual reset/object allocation | #278, #279; #280 also independently executable |
| #270 | Strict London choice/chance chronology through first upkeep | #268; #269 occurrence/reset; #65 handle-based mulligan/bottom rules | #278–#280 |
| #271 | Played priority, land and creature-cast prefixes | #268; #270 opening; #208 full-pool semantic action/native recording | #278–#280 |
| #272 | Spell modes/targets/discard costs and token identities | #268; #271 played casts; #209 cost/token/mode reference contracts; #212 target/identity reference contracts | #273, #275 |
| #273 | Creature mana and activated choices, including in-payment sources | #268; #271 played permanents; #254 delivered payment-stage semantics | #272, #275 |
| #274 | Cast/ETB trigger ordering and pending/stack/source identities | #268; #272 real noncreature spells and Bite source death; #211 trigger reference contract | #273, #275, #276 |
| #275 | Attack/block and modern damage-choice transcript | #268; #271 nontriggering creature play; #210 keyword/damage reference contract | #272, #273 |
| #276 | Cleanup/discard and rules-terminal checkpoint execution | #268; #272 temporary spell effects; #257 terminal outcome contract | #273–#275 |
| #277 | Initial normal-reset terminal-game admission | #268; #273 activation, #274 trigger, #275 combat, #276 terminal adapters (opening/spells inherited) | #281 |
| #278 | Bounded sampled latency collection and summaries | #268; #214 local metrics; #215 modes/capture semantics; #216 injected boundary timing consumer | #269, #279, #280 |
| #279 | New four-mode benchmark execution contract | #268; #215 actual trace/replay modes; #216 workload/window accounting; #208 normal full-pool policies | #269, #278, #280 |
| #280 | Validated fixed-trace application/legal/effect profiling | #268; #208 actual semantic executor; #216 existing boundary clocks/encoding | #269, #278, #279 |
| #281 | Four-mode/encoded/profile measurement evidence | #268; #278 sampled timing, #279 frozen workload, #280 trace profiles; #217 correctness-gated baseline/stress receipts | #269–#277 reference lane |

#24 adds only #277; #25 adds only #281. All old direct edges remain. #26 keeps
#23/#24/#25 and therefore every original and new M2 ancestor. No sharing of Java,
Rust or collector files creates a dependency. The four independent roots after
registration are #269/#278/#279/#280: the first three supply the requested opening,
timing and four-mode lanes; #280 is independently executable because its existing
consumer already accepts fixed actions and boundary clocks, not a fabricated lane.
The three-worker cap still applies. Normal handoff queues at most one eligible
root; coordinator can fill remaining free slots after receipt/control checks.

#271 owns basic priority/stack/zone observations; #272 token/target/effect identities;
#273 activated source/ability identities; #274 trigger event/incarnation/LKI;
#275 combat and damage/SBA identity; #276 settled cleanup and outcome observations.
Each child tests its actual normal-reset prefixes and all required local fields,
then #277 checks their complete composition. A missing field cannot become a silent
null/default. Unsupported callbacks fail explicitly throughout intermediate work.
The bounded test-only family runner must distinguish a declared prefix stop from
terminal completion; extra unused input is an error in either case.

## Quota, acceptance and owner crosswalk

| Preserved obligation | Repair contribution | Aggregate/later owner retained |
| --- | --- | --- |
| Exact decks/pins, both starters; GR-001/010/011/012/020/021/022 | #269/#270 occurrence/reset/mulligan; #271 first draw | #24 B010/B011/B026/B028/B030 |
| Priority/cost/target/spell; GR-030 | #271/#272/#273 actual continuous choices | #24 all twelve families, original packs and rejected inputs |
| Triggers, combat, cleanup, terminal; GR-031/032/033 | #274/#275/#276 identity and intermediate observations | #24 original independent oracles, APNAP/simultaneous-loss synthetic limits retained |
| Initial normal-reset full-pool XMage path; GR-040 | #277 admits two distinct native/XMage games per eight rows, 16 total, one heuristic-origin and one directed each | #24 audits initial path; does not receive later Forge/dual/AI-corpus ownership |
| >=100 individually reviewed scenarios, twenty meaningful cards, all twelve families, named mutants, holdouts and per-capability planned/executed/agreed/disputed/unsupported | New prefix/game receipts supplement unchanged #209–#213 receipts | Entire admission/aggregate remains #24; 175 catalog rows or repeated runs cannot satisfy distinctness |
| B008 latency, B021 sampled histograms; SYS-METRIC-001/002 | #278 exact sampling/buckets/merge/reset/overflow/privacy/error/policy/encoding tests | #25 full instrumentation audit |
| B020/B021 comparable four-mode execution and overhead | #279 freezes versioned sampling/capacity/replay/persistence/accounting; #281 measures | #25 full scalar composition, historical #214–#217 receipts unchanged |
| B014/B020 transition/legal/encoding/effect/allocation profiles; SYS-PERF-001/002/003 | #280 validated fixed-trace profiling; #281 encoded/four-mode measurements and affected memory | #25 keeps memory/stress/limits and dominant cost evidence, no designated-host invention |
| Gate/all remaining acceptance | No gate clause removed or satisfied by registration | #26 independently audits all M2; #241 after verified #26 |

The issue #24 operations-#54 amendment explicitly requires the initial full-pool
XMage path and retains the release floor of 16 games. The accepted replay design
GR-040 spells out two per row (heuristic and directed); its 2026-10-09 amendment
supersedes the old 24-game pilot/240-attempt generation order, not technical identity,
strictness or initial acceptance. #277 uses that 16-game set as the initial
**two-engine subset**, not a new Forge/dual qualification. All-three-engine
qualification, >=20 independent dual critical cases and later corpus gates remain
preserved historical #38/#40 obligations under the #241 current-plan transition.
The E2/E3 ten-game pilots/100-attempt measurements and E4 >=1,000 AI-generated games
(>=500 each source) are later work, neither imported here nor discharged by these
scripted M2 games. No implementation of RFC 0003 or new RL work is authorized.

## Authorization, readiness and recovery

All children remain unready until reviewed registration merges and exact-main CI
passes, then only completed prerequisites and normal controls admit dispatch. Native
GitHub blocked-by/sub-issue links mirror this additive graph; they are not completion
receipts. #24/#25 stay blocked until verified registration. Under the existing
handoff lock, #268 may then record coordinator resume grants and clear ONLY the
now-owned documented prerequisite-gap blocks from the archived audits. Re-fetch
controls/workpads/events; preserve unrelated holds and withdrawals. Add
`agent-resume-authorized` while dependencies remain pending, never ready early.
Reuse GH-24/GH-25 workspaces/evidence. Do not close either audit.

Serialize the compact #7 evidence upsert, resume grants and at-most-one eligible
successor reservation in one bounded handoff operation. Preserve all six archives
and every other issue entry; link contracts instead of pasting them into #7.
No resource-lock, concurrency, workflow enforcement, CI, sandbox or settings changes.
Implementation happens exclusively in Symphony with heavy commands serialized and
reading/editing/lightweight checks allowed in parallel.
