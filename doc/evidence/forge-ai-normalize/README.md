# Forge AI normalization (L0-011)

The normalizer binds the immutable actual Forge capture to the shared version-10
strict gameplay/chance transcript and version-1 canonical game identity. It
retains all eight aligned source opening checkpoints, the source-derived gameplay
checkpoint fixture, capture receipt, original trace
hash and receipt hash separately. Checkpoints never come from native output.
The delivered physical-copy normalization names equal-card copies by their first
ordered shuffle positions; seats, subsequent copy/incarnation choices, ordered
chance and gameplay remain significant. Source labels and diagnostic formatting
cannot create a new game identity.

## Reproduction

Prepare private Forge and XMage caches using their existing documented commands.
Run under the shared heavy-work lock in Symphony:

```sh
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- \
  python3 scripts/forge_ai_normalize.py \
  --source doc/evidence/forge-ai-capture/acceptance/game/result.json.jsonl.gz \
  --forge-cache /home/agent/.cache/forge-GH-329 \
  --xmage-cache "$MTG_REFERENCE_CACHE" \
  --output .agent-artifacts/forge-normalized-example
```

The output directory must be new. `normalized.json` contains the common
transcript, canonical content identity, source-derived checkpoint fixture and
separate provenance. `transcript.json` is the shared gameplay/chance input;
`native-fixture.json` retains the source-specific observation adapter. Forge
executes its original callback/chance chronology; native and patched XMage
consume its exact semantic translation. The runner checks that the executed
fixture equals the frozen normalized fixture. It never installs checkpoint
state into any engine. Each of two repetitions executes Forge/native plus the
native/XMage comparison (which itself repeats XMage twice). All source boundary,
priority, simultaneous-damage, cleanup and terminal comparisons remain enabled.

Six full-game controls corrupt a source copy, mode, target incarnation, random
tape, payment or checkpoint and require the existing precise rejection reason.
Original capture bytes are unchanged. Started, failed and successful receipts
are distinct. An exception cannot become an agreement or quota entry.

`validate(game, source)` independently retranslates the immutable source and
rejects changes to any normalized field, including checkpoints. `merge_origin`
uses the same canonical bytes as the XMage pilot, compares content on a matching
hash, preserves every distinct origin and records deterministic first attribution.
Its cross-source duplicate test is explicitly synthetic: it tests accounting,
not the existence of an independently generated second-source duplicate.
The identity index itself certifies no replay or admission.

## Scope and independent requirements

The unchanged source has 13,676 records and 7,022 random draws. CR103.5 supplies
London opening order; CR400.7 distinguishes physical copies/incarnations;
CR601/603 require mode, target, payment and trigger choices; CR508–510/704/514
supply combat, simultaneous state-based actions and cleanup requirements. The
recorded ending is red's life loss on turn 29. Detailed independently justified
observations and source-specific limits remain in the
[combat report](../forge-combat/README.md), including pinned Forge blocker
scheduling, process-local identity normalization, and the explicitly patched
XMage combat/SBA reference. No source pin or adjudication is changed here.

This delivers one normalization/replay boundary. Ten-game qualification,
publication/final admission, AI quota and the 1,000-game gate remain with their
registered owners. The frozen decks/cards/token and M3 holds remain unchanged.

## Executed acceptance and retained evidence

The [fresh execution receipt](execution.json) passes two Forge/native repetitions
and two native/XMage invocations, each with two reference executions. Each Forge
run consumes exactly 13,676 records and 7,022 random outcomes; each shared
comparison covers 818 actions, 610 gameplay checkpoints, 13 simultaneous-damage
observations and 28 cleanups. Canonical game identity is
`10ca8f0c5dc2a743bf3a5d27b2aca131222ea2bd572ba76c807bcd56528e1e95`.

The [revalidation receipt](revalidation.json) adds all eight source opening
checkpoints to every retained native/XMage run and repeats the complete shared
comparisons after verifying every execution artifact checksum. The source input,
semantic transcript, checkpoint fixture and actual engine implementations are
unchanged. This reuses real execution evidence under operations #348; it is not
an additional JVM execution or a regenerated expected result. The final command
performs these opening assertions directly. The earlier executing normalizer's
exact bytes remain archived with its original receipt/hash.

| Source corruption | Required first difference observed |
| --- | --- |
| Duplicate copy | `/forge/44/duplicate occurrence` |
| Mode | `/forge/5303/mode clone selection` |
| Target incarnation | `/forge/1458/stale occurrence/incarnation` |
| Missing chance | `/forge/3/record kind random` |
| Payment | `/forge/772/execution arguments` |
| Checkpoint | `/forge/2/state/players/0/life expected=19 actual=20` |

All [eight new tests](normalization-tests.log) pass in normal discovery. The
[focused validation](validation.json) also passes all 27 existing Python Forge
opening/casting/ability/combat tests, all four native Forge replay tests, and
fresh twice-native execution of all ten existing provisional XMage pilot games. They also
use the independently delivered real #328 native/XMage witnesses to reject a
changed library order and a missing opening point. The original inputs supply
expectations; these witnesses are not mock implementations. The
[opening checkpoint red](opening-checkpoints-red.log) preserves the intended
failure before adding the missing normalized observations. Separate exact-candidate
review, protected integration and exact-main CI remain delivery gates; their
receipts belong in the issue workpad/PR.
The initial behavioral regression failed because the existing full-game
translation lacked a shared content identity. A second regression demonstrated
that full-game fault injection rejected existing opening controls as unknown;
the full consumer now delegates these controls to the unchanged opening mutator.
No rule behavior, test assertion or existing negative control is weakened.


Original acceptance and revalidation bytes, including negative consumer inputs,
are retained in the content-addressed `attempt-index.json`. Each entry gives its
original path, byte count and SHA-256, plus the gzip object's storage path and
compressed SHA-256. Decompress the named object and verify both hashes to recover
the exact original. Equal files share storage only; original attempt paths and
verdicts remain distinct. No new AI capture, admission or quota is inferred.

After integrating the independently delivered native L0 runner from main
`fb493a1a112840ed8dd6df9be23d290ec299d096`, the
[integration receipt](integration.json) records another successful eight-test
normalization run, four native Forge regressions and seven batch-runner tests.
Fresh native and two XMage executions agree on the complete Forge-derived game,
including all eight opening observations. The complete ten-game provisional
corpus also passes cold/warm offline native L0 execution. Exact logs, executable
hashes and receipts are retained under `normalize-integrated/` in the attempt
index. The unchanged Forge consumer's original two executions and six live
corruption controls remain the source-specific evidence.
