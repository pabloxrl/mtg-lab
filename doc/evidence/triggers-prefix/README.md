# GH-274 played trigger adapter evidence

**Candidate acceptance PASS; delivery completion remains conditional on separate
review, protected integration and exact-main CI.** Those candidate/merge receipts
are retained in the [issue workpad](https://github.com/pabloxrl/mtg-lab/issues/274#issuecomment-6100953641)
and its linked PR. No M2 gate verdict is claimed.

| Check | Final evidence |
| --- | --- |
| Focused real-client discovery | [10 Python tests pass](triggers-focused.log); one new native test is also in normal discovery. |
| Full current-main torture | [343 Python tests; 1,496 Rust debug/release executions](torture.json), zero failed/ignored Rust tests; [complete log](triggers-torture.log). Includes `python3 scripts/run_tests.py` and `cargo test --workspace --locked`. |
| Actual native/XMage triggers, twice | [Run 1](triggers-run-1.json), [run 2](triggers-run-2.json): three prefixes/repeats, 427 settled checkpoints each, 16 invalid tapes, two callback probes, 14 comparator controls per engine. |
| Earlier family compatibility, twice each | [London 1](triggers-compat-mulligan-1.json)/[2](triggers-compat-mulligan-2.json), [priority 1](triggers-compat-priority-1.json)/[2](triggers-compat-priority-2.json), [spells 1](triggers-compat-spells-1.json)/[2](triggers-compat-spells-2.json), [activations 1](triggers-compat-activations-1.json)/[2](triggers-compat-activations-2.json). |
| Retained APNAP integration | [Nine existing synthetic compositions agree twice](synthetic-apnap.json). Their provenance remains synthetic. |

Every receipt source hash and raw artifact hash was checked against the final
source before archival. [Privileged observations](privileged-observations.tar.gz)
retain complete inputs, actual chance/choice consumption, raw checkpoints,
negative controls, toolchain and dependency/bridge/source hashes. They are
separate from public receipts and player captures. The [interim diagnostics](interim-diagnostics.tar.gz)
are also privileged and are not counted as passing acceptance.
[Artifact hashes](hashes.json) cover the logs, receipts and archives.

The authored [fixture recipe](../../../fixtures/reference/author_triggers.py)
reproduces all four positive/negative JSON tape files byte-for-byte without reading
engine output. Normal discovery runs every new positive/negative case. Original
fixtures, pins, catalog owners and expectations remain unchanged.

Partial additive R0002-B010/B011/B026/B028/B030 and GR-031 coverage only;
#24/#25/#26 retain their original integration and milestone acceptance.

The version-6 test-only family executes three actual normal-reset played prefixes:
Archer/Cyclops with an explicit bottom-to-top order; two identical Archers and two
separate Fodder cast events; and Pyromancer targeted after creature resolution,
then killed by a legally played Bear Cub/Bite response while its ability survives.
The [independent oracle](../../../fixtures/reference/full-pool-triggers-oracle.md)
uses the unchanged card pins and CR 603/101.4/113.7a/601.2i. Expected final life is
[20,19], [20,16] and [20,18]; Cyclops is 3/4 after its trigger.

[The compiled behavior red](played-etb-red.log) retains the intended assertion:
`legal played Pyromancer ETB must accept its post-resolution trigger choice`,
failing at `/play/47 /unsupported spell callback`. The original
[test source](played-etb-red.rs) uses normal reset and legal cast/payment/passes.
Compile failures and later diagnostic corrections are not behavioral-red evidence.

See [commands and boundaries](../../full-pool-reference.md#version-6-played-trigger-prefixes).
Native source handles and semantic identities, and XMage raw ability/source UUIDs,
source zone-change counters and actual callbacks, accompany the canonical keys.
Repeated events and duplicate source occurrences are distinct. Pending and target
placement staging differ between engines and are asserted separately.

The inherited empty-attacker check now queries `canAttack` only for creatures:
XMage's method assumes the caller has selected creatures. A normal played prefix
with only lands exposed that adapter defect. Prior families and tests remain.

Existing [GH-211 APNAP/cleanup evidence](../m2-triggers/README.md) is preserved.
Its opposing-controller APNAP compositions are explicitly synthetic and supplement
the played prefixes; they are not claimed reachable with simultaneous opposing
triggers in the frozen pool. No trigger injection occurs in this family.
No full-game, general trigger language, full legal-set, reference internal-RNG or
rollback, cleanup/combat expansion or M2 completion claim is made.

Raw native handles contain a storage scope allocated independently for each
`Game` (`objects.rs` uses an atomic scope allocator). Cross-run repeat comparison
normalizes only that scope, while every within-run stack source must match the
full witnessed handle, including scope, epoch, generation and slot. Raw artifacts
retain all values. Canonical and raw-source rebound mutations are both rejected.
The interim full run exposed this new test comparison error; it was not a rules
failure or a passing full-suite result. Final validation follows the correction.
