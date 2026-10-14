# Provisional XMage ten-game pilot (L0-003)

Version 1 qualifies **ten distinct, complete ComputerPlayer7 games**, covering
RG/GR/RR/GG and both starting seats. Each executes twice in native and twice in
`patched-combat-sba-v1` XMage with intermediate assertions and negative controls.
The [manifest](pilot.json) pins every deployable artifact and full canonical game
identity. Delivery is conditional on the independent exact-candidate review,
protected merge and exact-main CI linked in the [completion workpad](https://github.com/pabloxrl/mtg-lab/issues/321#issuecomment-6131617940).
This is provisional two-engine qualification: **zero final all-three admissions**.
No Forge, 100-attempt campaign, 1,000-game acceptance or native latency is claimed.

## Qualified games

| Attempt | Row/starter | Canonical ID prefix | Actions | Source checkpoints |
| --- | --- | --- | ---: | ---: |
| 00-red-green-0 | RG/0 | `771369974b8b` | 436 | 301 |
| 01-red-green-1 | RG/1 | `6e37accc1de3` | 466 | 333 |
| 02-green-red-0 | GR/0 | `4bbad1e7f3c3` | 597 | 413 |
| 03-green-red-1 | GR/1 | `d74920c1132a` | 440 | 301 |
| 04-red-red-0 | RR/0 | `87fdb81fedcc` | 329 | 242 |
| 05-red-red-1 | RR/1 | `dfabc153ac29` | 268 | 211 |
| 06-green-green-0 | GG/0 | `32098d8f907e` | 749 | 549 |
| 07-green-green-1 | GG/1 | `6a05e35b4cc3` | 372 | 260 |
| 08-red-green-0 | RG/0 | `9f35aa2484fd` | 608 | 410 |
| 09-green-red-1 | GR/1 | `f096238a0e41` | 621 | 430 |

Ten source games were started and completed; none were discarded or rerolled.
All eight original conversion/native failures and the later RG/0 pending-set
failure remain in the history. The final RG/0 qualification reruns both engines;
the other nine unchanged engine executions were rechecked with the corrected
comparator ([receipt](diagnostics-v1/recomparison-v2.json)). Qualification
receipts retain their original execution-source hashes. Full native regression
replays all ten using the current candidate. The complete toolchain record is
[here](toolchain.json); Java/AI/source/patch details are pinned per capture and
qualification receipt. These diagnostic timings are not engine-speed measurements.

## Origins and accounting

The original eight ComputerPlayer7 games are retained byte-for-byte under
`baseline-v1`, seeds 32100–32107, one per RG/GR/RR/GG and starting-seat row.
`baseline-v1/diagnosis.json` records their original earliest unsupported or
divergent observations. Two additional games, seeds 32108–32109, are under
`additional-v1`; the plan was frozen before generation. Timed AI searches mean
seeds alone cannot regenerate these games. Replay the immutable choices and
chance instead. Recording-equivalence snapshots are probes of the same source
state, not additional games or quota entries.

Each capture includes configuration, source/toolchain/AI pins, original outcome,
and compressed semantic trace. Source snapshots contain the existing project
bridge, not an upstream installation. Raw Maven logs remain private under the
[provenance policy](../../provenance-policy.md). Historical inventory hashes can
refer to those private logs; the pilot manifest enumerates deployable artifacts.
Existing [XMage notices](../../../references/xmage/UPSTREAM-LICENSE.txt) and
[capture provenance](../xmage-ai-capture/README.md) remain applicable. No card
text, artwork or full rules documents are added.

Canonical identity version 1 includes fixed seats, frozen definitions, deck
composition, chronological decisions and actual ordered chance. Physical copies
are renamed by first ordered shuffle position; subsequent uses and incarnations
retain that binding. Token identities retain creation groups and consequential
uses. Source labels, seeds, UUIDs and diagnostic formatting do not distinguish
games. Suspected duplicates compare canonical bytes as well as SHA-256; the
first qualified origin owns an identity. Altered-provenance duplicate controls
remain separate synthetic probes and contribute zero games.

## Consumer corrections and adjudication basis

These corrections consume the original tapes; they do not change frozen decks,
card definitions or engine mechanics. Independent exact-candidate review must
confirm these adjudications and the preserved upstream exceptions.

| Retained observation | Translation and independent basis |
| --- | --- |
| Archer cast triggers, Cyclops/Archer ordering, Pyromancer target | CR603.3 and 117.5 place waiting triggers before priority. Bind actual trigger instances to source incarnation/event and actual selected order. Preserve the source pass while moving its replay boundary after placement. Check exact pending sets independently in each engine, including XMage's automatic singleton placement. |
| Additional RG/0 pending collection | CR603.3b assigns consequential order when the controller places triggers. Source 2546 and native action 283 contain the same two waiting triggers in different collection orders. Compare the full pending multiset, retaining controller/occurrence/event and rejecting missing or duplicate members; keep chosen placement and stack order exact. The original failed qualification and behavioral red remain in `diagnostics-v1`. |
| Both Goblin Surprise modes and affected-object sets | CR700.2 and 611.2c; use the selected mode and witnessed object set, not current board membership as a substitute. Check +2/+0 and Cyclops +3/+0 effects at intermediate states. |
| Simultaneous combat deaths | CR404.3 requires owner order; CR704.3 settles state-based actions before priority. Preserve witnessed upstream automatic graveyard order as an explicit exception, not an AI decision. The existing `patched-combat-sba-v1` reference patch remains unchanged; this is not agreement with unmodified upstream. |
| Departed Pyromancer's haste grant | CR400.7/611.2c: a grant on a departed battlefield object is no longer an active grant. The native observation adapter now filters invalid/departed handles; it does not change gameplay. |
| Repeated London mulligans and bottoming | CR103.5: each new mulligan reshuffles the earlier bottomed cards. Only final-round bottoms follow the final shuffle. Consecutive source singleton bottom callbacks become one ordered decision, with every intermediate source snapshot checked against the consumed prefix. |
| London raw zone-change counters | Semantic incarnations follow CR400.7. Pinned XMage returns mulligan cards directly without incrementing its raw counter. Bind later triggers through the witnessed, validated transition chain, including this explicit exception; reject a forged increment or source counter. |
| Cleanup and losing-player departures | CR514.1 uses the captured physical discard. Terminal comparisons retain pre-departure states and explicit witnessed losing-player removals; the final winner alone is insufficient. |

Behavioral regressions live in `tests/test_xmage_ai_pilot*.py` and
`tests/test_xmage_ai_trigger_choices.py`. They include actual native execution,
immutable source witnesses, corrupted pending sets/counters and canonical
duplicate controls. The original five invalid-tape controls remain unchanged;
each game also gets controls at its actual physical-choice/payment/order cursors.
The reference runner's `xmage-index.json.failure.json` is its last deliberately
rejected negative-control tape; the per-game qualification receipt and negative
results distinguish that expected rejection from a failed positive replay.

## Reproduction

Validate retained evidence without Java or network access:

```sh
python3 scripts/check_xmage_pilot.py
```

Replay every provisional game twice in native with complete source assertions:

```sh
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- \
  python3 scripts/check_xmage_pilot.py --native-output .agent-artifacts/pilot-native
```

The output directory must be new. This is the diagnostic native consumer;
the prebuilt release batch runner and its timing contract belong to #322.
For live twice-native/twice-patched-XMage qualification of a retained source:

```sh
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- \
  python3 scripts/xmage_ai_normalize.py \
  --source doc/evidence/xmage-ai-pilot/baseline-v1/00-red-green-0/trace.jsonl.xz \
  --output .agent-artifacts/pilot-live-rg0 --cache "$MTG_REFERENCE_CACHE"
```

Use the pinned prepared reference cache and managed toolchain. This command
executes real consumers and preserves first differences; parser-only tests cannot
qualify a game. Final admission must independently recheck every candidate in
native, XMage and Forge and record explicit overlap IDs. This pilot's final
admission list remains empty.
