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

## Version 3 played priority prefixes

The additive `priority` family consumes the version 3
[envelope](../fixtures/reference/full-pool-priority.json) using real full-deck
reset and explicit keep/mulligan prefixes. Six cases cover both starting seats,
two distinct copies each of Swab Goblin and Bear Cub, floating and payment-stage
basic mana, explicit colored and generic payments, and a mana response between
passes. Every priority pass and empty attacker declaration is on the tape. The
starting player skips the first draw step; both seats continue through their
second draw and stop after the fourth creature resolves on turn six.

The native client submits `actions::Record` through the delivered scalar policy
interfaces and appends real before/after captures to a version 2 recorder. It
also rejects every stale previously accepted policy submission without changing
state or RNG. The recorder is an open played prefix, not a complete episode.
XMage uses continuous priority, land, cast and mana callbacks. No adapter invokes
stack resolution directly. A named `first_cast_committed` stop pauses the same
game and strict continuation resumes its remaining tape.

[Literal expectations and their derivation](../fixtures/reference/full-pool-priority-oracle.md)
are independent of engine results. Committed checkpoints compare ordered zones,
turn/step/actor, land usage, all six mana values, tapping/sickness, stats, physical
occurrences, incarnations and stack source/action identity. Raw payment staging
has separate native and XMage expectations: native retains a transaction before
commit, while XMage moves the card onto the stack during announcement. Both raw
states remain in privileged artifacts; neither is rewritten to pretend they are
identical. The explicit remaining-cost/payment ledger also agrees. XMage sorts opening
hands for display; each raw hand order has its own literal assertion, while
cross-engine hands compare membership. The version-3 `incarnations` field starts at zero in the initial library and
counts witnessed zone transitions under CR 400.7. Raw counters are retained,
including XMage's direct mulligan
return that changes zone without incrementing its counter. No engine state is
injected by these mappings.

Twenty malformed or illegal tapes exercise actual clients, including missing,
extra and reordered passes, wrong actor/timing, second land, wrong/insufficient
mana, stale sources and unsupported callbacks. Comparator controls alter draw,
priority and incarnation fields. Vanilla creature timing admits one spell at a
time, so the stack-order comparator control is explicitly synthetic, made from
two separately witnessed stack entries; it is not a simultaneous-stack play claim.

```sh
python3 -m unittest discover -s tests -p 'test_m2_repair_priority.py'
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- python3 scripts/full_pool_reference.py --family priority --cache "$MTG_REFERENCE_CACHE" --output .agent-artifacts/priority-run-1.json
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- python3 scripts/full_pool_reference.py --family priority --cache "$MTG_REFERENCE_CACHE" --output .agent-artifacts/priority-run-2.json
```

The runner retains actual inputs, consumed chance/choices, raw identities,
checkpoints, oracle files, first-difference controls, source/dependency/toolchain
hashes and separate privileged artifacts. Unsupported choices fail. There is no
complete legal-set comparison, reference rollback/internal-RNG equality,
noncreature/trigger/activated-choice support, nonempty combat, cleanup discard,
full-game claim or M2 gate completion. Verification receipts belong in
[the priority evidence report](evidence/priority-prefix/README.md).

## Version 4 played spell prefixes

The additive `spells` family executes six normal-reset prefixes through ordinary
native semantic actions and pinned XMage player callbacks. Both players keep;
all chance, passes, empty attacks, casts, modes, physical targets, mana payments
and discard choices are supplied explicitly. It stops in main before cleanup.

The [authored schedules](../fixtures/reference/author_spells.py) and
[independent oracle](../fixtures/reference/full-pool-spells-oracle.md) cover
Fodder's two distinct 1/1 red Goblins, Thrill's physical Mountain discard and
ordered draw two, both Surprise modes, Growth responding to Bite (Cub 5/5),
and Bite responding to Growth on a token (the departed target stays absent).
A second token-producing spell gets a new creation-event ordinal. Native birth
identities and actual XMage CREATED_TOKEN events bind tokens before selection;
no name-based matching selects among duplicate copies or tokens.

Version 4 keeps the original version 1–3 inputs and acceptance. It adds explicit
`role` and `mode` fields. The native and XMage spell inputs have distinct,
explicitly authored discard chronology: native chooses its private additional
cost before mana, whereas XMage calls the additional-cost selector after mana.
The actual callback ledgers and raw announcement/payment states remain separate.
After commitment, both engines compare life, ordered library/graveyard/stack,
ordered target roles and incarnations, mana, characteristics and token creation
order. Hand and battlefield membership are compared across engines; raw orders
are retained, and the two drawn cards are checked in order. Native transient
modification slots and XMage continuous-effect source/duration objects supply
separate effect evidence; cleanup expiry is outside this family.

Eighteen invalid tapes exercise role/controller, incarnation, missing/disordered
target, unknown/duplicated token selection, illegal mode, missing/wrong discard,
unpaid/insufficient resources, cancellation, extra callback and truncated/extra
tape boundaries. Additional reference callback probes reject repeated token
births and unsolicited target/mode/mana calls. Native semantic decode/application
checks state/RNG nonmutation and stale submissions; failed complete envelopes
also preserve the caller. These are distinct from the reference's selected-cast
playable-action query and callback/resource rejection evidence. Neither proves
complete legal-set equality or reference rollback/internal-RNG equality.

```sh
python3 -m unittest discover -s tests -p 'test_m2_repair_spells.py'
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- python3 scripts/full_pool_reference.py --family spells --cache "$MTG_REFERENCE_CACHE" --output .agent-artifacts/spells-run-1.json
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- python3 scripts/full_pool_reference.py --family spells --cache "$MTG_REFERENCE_CACHE" --output .agent-artifacts/spells-run-2.json
```

The focused Python module launches the real native client and is in normal
discovery. The runner writes a public hash receipt plus a separate restricted
privileged directory containing complete inputs, actual choices/chance,
checkpoints, first-divergence controls and source/toolchain/dependency evidence.
No trigger/activated-choice dispatch, nonempty combat, cleanup expiry,
terminal-game admission, full-game agreement or M2 completion is claimed.

Retained red assertions, acceptance receipts and limitations are in the
[spell-prefix evidence report](evidence/spells-prefix/README.md).

## Version 5 played activation prefixes

The test-only `activations` family extends the same runner while retaining
reset, mulligan, priority and spells versions. Six normal-play schedules reach
Elf, Druid, Cavalry, Shivan and Invoker through actual land plays and creature
casts. Explicit source choices cover empty and floating pools, Cavalry haste,
Shivan single/repeated activations, Invoker targeting/payment and surplus mana.

Native reservations, source/target incarnations, distinct raw stack objects,
actual consumed choices and turn-scoped effects are retained. XMage exposes its
actual target/payment callbacks and `EndOfTurn` effects. Native cancellation is
checked at every continuation; reference cancellation at every actual callback.
Raw staging differs and is asserted separately, with committed states compared.
See the [oracle and limits](../fixtures/reference/full-pool-activations-oracle.md)
and [acceptance evidence](evidence/activation-prefix/README.md).

```sh
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- python3 -m unittest discover -s tests -p 'test_m2_repair_activations.py'
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- python3 scripts/full_pool_reference.py --family activations --cache "$MTG_REFERENCE_CACHE" --output .agent-artifacts/activations-run-1.json
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- python3 scripts/full_pool_reference.py --family activations --cache "$MTG_REFERENCE_CACHE" --output .agent-artifacts/activations-run-2.json
```

Outputs are privileged diagnostic artifacts, separate from player captures.
Unsupported callbacks and missing observations fail explicitly. These are bounded
prefixes ending before cleanup, not complete games, a complete legal-set comparison,
a general ability language or an M2 gate verdict.

## Version 6 played trigger prefixes

The test-only `triggers` family retains all earlier families. Three normal-reset
prefixes cast Archer/Cyclops, two identical Archers across two noncreature casts,
and Pyromancer followed by a legally played Bite killing it before its ability
resolves. Explicit controller order, source incarnation, ability and event ordinal
identify pending and placed triggers. The event ordinal counts witnessed trigger-generating batches within the prefix; raw choice-boundary provenance is retained. Targets are chosen after Pyromancer resolves;
its source remains incarnation 3 after the physical card moves to incarnation 4.

Native pending batches and XMage's actual ordering/placement callbacks retain raw
source and ability provenance. XMage calls `chooseTriggeredAbility` only for
multiple waiting triggers; a lone trigger uses `triggerAbility`. During its ETB
target callback, XMage already has an announced stack ability, while native still
has a pending trigger. Each engine's staging is asserted separately; settled
semantic checkpoints are compared. Unsupported callbacks and missing fields fail.

```sh
python3 -m unittest discover -s tests -p 'test_m2_repair_triggers.py'
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- python3 scripts/full_pool_reference.py --family triggers --cache "$MTG_REFERENCE_CACHE" --output .agent-artifacts/triggers-run-1.json
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- python3 scripts/full_pool_reference.py --family triggers --cache "$MTG_REFERENCE_CACHE" --output .agent-artifacts/triggers-run-2.json
```

The focused module executes the real native client and its negative tapes in
normal discovery. Public receipts hash separately restricted privileged artifacts.
The [independent oracle](../fixtures/reference/full-pool-triggers-oracle.md) and
[acceptance report](evidence/triggers-prefix/README.md) state exact coverage.
Existing [APNAP compositions](evidence/m2-triggers/README.md) remain explicitly
synthetic supplemental evidence; no triggers are injected into these played
prefixes. No complete legal-set comparison, reference rollback/internal-RNG,
nonempty combat, terminal-game admission or M2 completion claim is made.
