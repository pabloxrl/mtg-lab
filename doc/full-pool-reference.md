# Full-pool reference prefix protocol

The version 1 test-only envelope in
[full-pool-reset.json](../fixtures/reference/full-pool-reset.json) identifies
physical copies at creation, before permutation. It does not change the old
opening-counts fixture or the production replay format. Source, card and rules
pins remain unchanged. Only the `reset` family is in this delivery's scope.

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
