# GH-275 combat repair evidence

Status: local acceptance passed; protected delivery remains pending. Full torture passed on the
integrated `dac05699500db9f79bfed8cc3d17f01fca5236aa` base: 370 Python tests,
758 Rust tests in each of debug and release (1,516 executions), formatting and
Clippy, with no ignored tests. Both final-source reference runs and all eleven
focused tests passed. Exact-candidate independent
review, protected merge and exact-main CI remain delivery gates. No M2 completion
is claimed.

Normal discovery adds eleven Python tests and eight Rust tests: the real played
combat client, five core owner-order regressions and two policy regressions.
They include tokens excluded from card permutations and restorable terminal
snapshots after concession at either owner-choice stage.

## Executed acceptance

- [Full torture log](combat-torture.log): ordinary Python discovery, formatting,
  Clippy and full locked Cargo debug/release discovery; no filters or skips.
- [Focused command log](combat-focused-final.log): eleven real-client tests in
  417.569 seconds, including independently authored fixture reproduction and
  strict observation/choice mutations.
- [Run 1](combat-run-1.json) and [run 2](combat-run-2.json): each executes 19 cases
  twice per engine, compares 4,247 committed checkpoints, rejects 33 invalid tapes
  and detects 17 comparator controls per engine, plus three unsupported callback
  controls. Both inventories contain the same 200 source hashes, verified against
  the final candidate's source bytes.
- [Archive index](archive-index.json): exact hashes and contents for the final
  sources, both privileged runs, intermediate sources/observations and complete
  validation history. Privileged observations contain hidden hands and libraries;
  they are diagnostic evidence, never policy or player-observation inputs.

The reproduction commands are in the [version-9 protocol](../../full-pool-reference.md#version-9-played-combat-and-simultaneous-graveyard-orders).
Both actual reference commands and focused discovery ran under the shared heavy
lock. The root README now describes this bounded family, played-replay version 2
and the patched-reference limitation. Its milestone table is unchanged.

The required independent reviewer must explicitly assess APNAP choice timing,
simultaneous zone transitions, native invalid-choice state preservation,
reference provenance, retained/stronger compatibility coverage and README
accuracy. The [issue workpad](https://github.com/pabloxrl/mtg-lab/issues/275#issuecomment-6102101331)
records the exact-candidate review and subsequent protected delivery receipts;
local test success alone is not a delivered milestone verdict.

## Behavior and independently justified corrections

The candidate adds 19 normal-reset played combat schedules, including both
opposite owner orders, swapped physical arrival occurrences and simultaneous
multi-owner deaths. The strict version-9 transcript requires every owner choice;
the engine marks damage before choices and applies deaths only after all owners
have chosen. Played-replay version 2 records the new boundary. Native heuristic
uses candidate order and legal-random chooses a permutation; these policy
conventions are never silent transcript defaults.

The reference integration is separately maintained in
[CombatSbaGame.java](../../../references/xmage/CombatSbaGame.java). It uses actual
pinned engine state and batch zone movement, leaving upstream sources unchanged.
Any passing comparison is explicitly patched-reference agreement. The original
unmodified-upstream mismatch below remains independent evidence, not agreement.

The [timing excerpts](graveyard-timing-rules.json) pin CR101.4/404.3/704.3. The
[new boundary assertion red](graveyard-boundary-red.log) precedes implementation.
The [concession regression red](graveyard-concession-red.log) records the
orphaned-batch snapshot defect before terminal cleanup was corrected, under
pinned CR104.3a.
The [token regression red](graveyard-token-red.log) independently reproduces
CR111.6/404.3: a token and one card must not create a card-order decision.
[Profile compatibility headers](profile-compatibility.json) retain all original
independent profiling checkpoints; only the conservative native core fingerprint
is refreshed, with full execution still required. Upstream source/card/rules
pins are unchanged. Legacy synthetic fixtures explicitly submit the new owner
choice without removing or weakening their old outcome assertions.

The replay version change also requires CLI compatibility corrections. The
[label assertion red](replay-label-red.log) shows the stale `played-v1` label on
an accepted v2 replay. The CLI now reports `played-v2`; tests retain explicit
played-v1 and opening-v2 rejection, unknown versions, incompatible discriminator
pairs, and malformed cross-format payloads with valid destination versions.
These expectation corrections follow the declared format contract, not captured
engine output. Independent review must check their stronger retained coverage.

The existing core capture/semantic-input and recorder two-blocker ledgers now
submit explicit owner choices before their original death assertions. All prior
outcome, damage, life, capture on/off, quantum, privacy, corruption and replay
checks remain. The capture ledger additionally checks that every creature is
still present before the owner chooses. Full validation history retains each
missing-choice failure that motivated these consumer updates.



Normal discovery also exposed a native-child SIGKILL under container memory
pressure; the cumulative [memory counters](memory-before-streaming.json) and
failed full-discovery log are retained. The exporter now borrows the checkpoint
view and streams the complete report instead of cloning the aggregate and
allocating a whole-output byte buffer. The [encoding compatibility proof](streaming-export-equivalence.json)
shows byte-for-byte identity for all 242,791,871 bytes, including every case,
repeat and negative tape. This is serialization compatibility evidence, not a
replacement rules oracle; full normal discovery and actual reference runs remain
mandatory.

## Preserved historical blocker and diagnostic evidence

The following record describes the earlier unsuccessful draft and scope block.
The operator subsequently authorized its repair inside this issue. Historical
claims below are not current acceptance status.

### Earlier draft and strict replay blocker

Status: **not delivered; no passing combat acceptance receipt**. Preserved draft
on `agent/issue-275`, base `dac0569`. The coordinator's Cavalry/Pathwarden scope
correction is implemented in the draft only. No PR, independent candidate
review, full torture or merge has occurred.

Nine actual normal-reset played combat schedules and 24 invalid tapes execute
in both native and pinned XMage. The seven-test focused Python module passed
in 92.827 seconds, including real native execution and 16 comparator mutations.
The Rust combat client adds one test to normal discovery. These are diagnostic
results; full Python/Cargo discovery and delivery validation remain pending.

The actual reference command still fails:

```sh
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- \
  python3 scripts/full_pool_reference.py --family combat \
  --cache "$MTG_REFERENCE_CACHE" --output .agent-artifacts/combat-ordered-blocker.json
```

In `shivan-multiple` at `before/304`, the native green graveyard is Sentry then
Thornweald; XMage's is Thornweald then Sentry. The comparator reports
`$.graveyard[1][0]`. Both creatures received lethal damage simultaneously. The
same explicit transcript contains no owner graveyard-order choice. Neither
combat/SBA implementation offers one on this path. Do not sort this difference
away or reorder the authored blockers to conceal it.

The [official pinned excerpt](pinned-rules-excerpt.json), freshly fetched and
SHA-256 verified, states:

> 404.3. If an effect or rule puts two or more cards into the same graveyard at
> the same time, the owner of those cards may arrange them in any order.

CR704.3 requires the applicable SBAs simultaneously. The repository's
[replay contract](../../testing/game-replay.md) requires ordered graveyards.
Native `combat.rs::queue_combat_damage` queues deaths from its damage-change
iteration. Pinned XMage `GameImpl.checkStateBasedActions` calls `destroy` on
each lethal permanent in battlefield iteration order. This is a missing strict
choice contract, not permission to adopt either output as the oracle.

Proposed coordinator prerequisite: explicit owner-chosen simultaneous graveyard
ordering through native rules/actions and pinned-reference capture, with
occurrence/incarnation identity, strict chronology, invalid/missing/extra-order
controls and real played positive acceptance. Alternatively explicitly authorize
that expansion here. The feature worker has registered no new task and waived
no requirement. Resume this preserved draft after the decision and prerequisite.

Evidence:

- `baseline-test.py` and `combat-red.log`: preserved original intended real-client
  behavioral failure, before combat implementation.
- `early-life-red.py` / `early-life-red.log`: an actual played observation mutation
  was not rejected before checker hardening; one intended assertion failure.
- `ordered-graveyard-repro.py` / `ordered-graveyard-red.log`: actual ordered
  checkpoint disagreement, preserved without canonicalizing away zone order.
- `ordered-graveyard-checkpoints.json`: minimized observed checkpoints.
- `exact-source-ordered-blocker.log`: final actual command reaches the precise
  first divergence after both engines' positive/negative checks.
- `exact-source-blocker.tar.gz`: privileged complete inputs, consumed choices,
  checkpoints, negative runs, logs and source inventory, plus exact draft source
  bytes. Every recorded source hash was checked against the preserved draft.
- `diagnostic-native-xmage.tar.gz`: earlier diagnostic observations and draft
  sources, explicitly not exact-candidate acceptance.
- `focused-diagnostic.log`: seven passing focused tests; the full reference
  comparison remains failing and its order-sensitive comparator has a regression.

The draft also records XMage's total-only trample constraint gap in the
[independent combat oracle](../../../fixtures/reference/full-pool-combat-oracle.md).
Native/reference staging, raw identities and privileged artifacts remain separate.
README was not changed because no supported behavior, usable delivered command,
architecture or verified milestone status changed. Review must check README
accuracy when this draft becomes a deliverable. M2 remains incomplete.
