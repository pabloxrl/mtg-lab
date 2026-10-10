# Full-pool reference prefix protocol

The version 1 test-only envelope in
[full-pool-reset.json](../fixtures/reference/full-pool-reset.json) identifies
physical copies at creation, before permutation. It does not change the old
opening-counts fixture or the production replay format. Source, card and rules
pins remain unchanged. Version 1 supports the `reset` family; version 2 adds the bounded London
continuation described below.

Each seat's occurrence list follows frozen deck entry order, then zero-based
copy index. An occurrence string `seat/card-key/copy-index` is unique within the
game. A shuffle event carries its global sequence, actor, kind, exact pre-event
occurrence multiset (serialized in creation order) and full post-event permutation (library top first). Consumers must
reject missing, extra, duplicate, reordered or unsupported input. Equal counts
are insufficient. Empty choices mean no declaration has yet been submitted.

Native allocation binds occurrences to semantic birth identities before a
permutation is applied. Drawing uses the engine's zone transitions, retaining
birth identity while native incarnation/handles change. A repeat reset binds
fresh handles; the prior handles must all be invalid. The XMage constructor
loads the forty real cards per seat into a normal duel with test-mode draw
suppression disabled. Its initial shuffle hook supplies only chance outcomes;
XMage computes life initialization and draws. Its UUID-to-occurrence map is
bound at card construction, before library permutation. XMage loads its deck
through an unordered set; the bridge checks the witnessed pre-shuffle multiset,
serializes that multiset in creation order, and retains the raw pre-shuffle
library order separately in privileged evidence. Post-shuffle order is compared literally.

The bounded stop is the first `chooseMulligan` callback. It reports
`completion: prefix`, sorted hand membership, top-first ordered libraries,
occurrence-to-card/seat/zone mapping, life, starting and declaration seats, and
actual consumed chance/choice records. No mulligan decision, opening completion,
first turn, legal-set comparison or full-game outcome is claimed. Input rejection controls validate the envelope
and adapter; they are not raw engine legal-action rejections or selected-action
acceptance. No player action is submitted in this family. `turn_active_seat` is explicitly null before turn execution: the opening actor is the declaration
seat and the initial turn owner is the starting seat.

Expected checkpoints are literal, independently authored
[oracle data](../fixtures/reference/full-pool-reset-expectations.json), with the
[CR 103 derivation](../fixtures/reference/full-pool-reset-oracle.md). Sixteen cases
cover all eight deck/starter rows and same-name copy swaps. The prefix adapter
must not load oracle data as game state.

Artifacts contain both hands and library orders. They are privileged diagnostic
evidence, kept separate from player observations and public aggregate receipts.
Production policy/trajectory inputs do not consume this protocol. Missing fields
or unsupported callbacks fail; they cannot become agreement.

Run the focused normal-discovery tests (they execute the native client):

```sh
python3 -m unittest discover -s tests -p 'test_m2_repair_reset.py'
```

After preparing the pinned issue-local XMage cache using the existing
[reference setup](../references/xmage/README.md), run the real comparison twice:

```sh
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- python3 scripts/full_pool_reference.py --family reset --cache "$MTG_REFERENCE_CACHE" --output .agent-artifacts/reset-run-1.json
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- python3 scripts/full_pool_reference.py --family reset --cache "$MTG_REFERENCE_CACHE" --output .agent-artifacts/reset-run-2.json
```

The JSON receipt carries status, counts and hashes. Its sibling `.privileged-*`
directory contains actual inputs, oracle, native/XMage checkpoints, consumed
chance, repeat-reset observations, rejection diagnostics, toolchain metadata and
logs. Directory/file permissions are 0700/0600. Failed runs publish no success
receipt; existing diagnostic artifacts are retained. Python native checks use
the checked-in `scripts/symphony/resource_lock.py` helper with the same shared
lock directory when invoked standalone, and inherit the held lock during torture.
This also works in the toolchain-only CI image, which declares a controller root
without containing that checkout.

[Executed acceptance evidence](evidence/reset/README.md) retains the test-first
failure, both real reference runs, negative controls and regression logs.

## Version 2: London opening continuation

The additive `mulligan` family uses the version 2
[envelope](../fixtures/reference/full-pool-mulligan.json). Version 1 and the
original count-only runner retain their contracts. Version 2 consumes normal
reset shuffles, explicit keep/mulligan declarations and ordered cumulative
bottom selections, then stops at the starting player's first upkeep priority
callback. It never submits a priority action or draws the first turn's card.

Chance and choice each have their own zero-based, gap-free sequence. Both carry
an actor and an explicitly null source (these opening operations have no card
ability source). Choice `round` is the actor's completed redraw count: zero for
initial declarations and n for the bottom after the nth redraw. This scopes a
bottom selection to its current seven-card hand; reused old round/occurrence
inputs fail. A bottom list is top-to-bottom library suffix order, not card-name
membership. Native candidates are resolved against current engine handles.
XMage selects each specified UUID through its actual one-card target callback;
there is no default or guessed card choice.

The pinned source processes each mulliganing player’s shuffle/redraw/bottom
serially, whereas native redraws all mulliganing players before bottom choices.
CR 103.5 describes simultaneous mulligans. The comparator aligns **per-player**
declaration and redraw/bottom boundaries and the final state of both seats;
it does not claim equality of intermediate cross-player scheduling. Both raw
callback histories are retained separately. The chance stream and the choice
stream must each match their entire input exactly. In particular, all players
declare before their round's mulligans, and all bottoms finish before the next
declaration round. This normalization never edits either engine's game state.

The literal [oracle](../fixtures/reference/full-pool-mulligan-expectations.json)
and [derivation](evidence/mulligan-chronology/oracle.md) cover 32 cases: both
starters, all four deck pairs, no mulligans, one mulligan, unequal multiple
mulligans and three mulligans by both players. Physical same-name copies and
reversed multi-card bottoms distinguish ordered identity from counts.

```sh
python3 -m unittest discover -s tests -p 'test_m2_repair_mulligan.py'
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- python3 scripts/full_pool_reference.py --family mulligan --cache "$MTG_REFERENCE_CACHE" --output .agent-artifacts/mulligan-run-1.json
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- python3 scripts/full_pool_reference.py --family mulligan --cache "$MTG_REFERENCE_CACHE" --output .agent-artifacts/mulligan-run-2.json
```

The focused Python module runs the actual native client and its negative
controls; it is not a parser-only test. Inputs with wrong actor/kind/source,
stale round/copy, wrong cardinality, duplicate bottom, missing/extra/reordered
chance or choices, incomplete ledgers and unsupported boundaries reject.
Native admission executes on a new game and commits only a complete accepted
prefix, so rejected input leaves the caller's state/RNG unchanged. Explicit
permutations consume no native RNG words. Reference rejection stops at the
first divergence; reference rollback and internal RNG fields are unobservable,
reported separately rather than counted as agreement. Adapter rejection,
selected-action acceptance and complete legal-set comparison are distinct:
this family claims the first two, not exhaustive legal-set equivalence.

Receipts and privileged artifacts follow the same separation as version 1.
No full game, admission, Forge or M2 completion claim follows from this prefix.
