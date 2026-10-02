# Dragon Fodder and token lifecycle acceptance

GH-194 adds Dragon Fodder to the authoritative cast/payment/stack path. Paying
1R creates no token until resolution; resolution creates two distinct untapped
red 1/1 Goblins controlled and owned by the caster. Tokens obey summoning sickness,
can block immediately, and cease after lethal damage/SBAs. Old handles cannot
address a reused slot. Existing policy choices, semantic actions, snapshots,
Driver/Run capture and typed recorder conversion carry the behavior.

Delivery remains conditional on the linked issue workpad's independent review,
protected merge and exact-main CI. This is partial R0002-B010/B011/B029 evidence,
not full-pool/M2 or whole-block acceptance. Modal creation, haste and cast/ETB
triggers remain their named mechanic/composition owners.

## Independent oracle and test-first evidence

The unchanged frozen card manifest pins Dragon Fodder's 1R sorcery and Goblin's
red 1/1 creature definition. CR 111, 302.6, 307, 400.7, 601, 608 and 704.5d/g
supply timing, identity, payment, resolution and token cessation expectations.
Literal counts/life are authored expectations, never regenerated from outputs.
[Compiled behavioral red](behavioral-red.log) records three ordinary tests
failing because payable Fodder was not offered. Implementation followed that run.

The existing all-other-identities rejection test necessarily excludes the newly
supported Fodder/token pair. Every remaining unsupported identity and all old
six-card assertions remain. New tests replace those two obsolete support claims
with stronger positive, illegal timing, token-not-castable and pinned-characteristic
coverage. The independent reviewer must explicitly check this requirement
correction; no test is removed or skipped.

Native policies previously rejected a newly offered Fodder, breaking an existing
CLI accounting regression. [Compiled policy red](policy-red.log) reproduces the
rejection. A narrow whitelist addition and Fodder's documented vanilla-development
score preserve existing input scores/tie rules. Policy IDs become
`legal-random-tokens-v1` and `heuristic-tokens-v1`, because supported content is
part of their pinned contract; old IDs reject explicitly. Native configuration
and existing policy-ID literals migrate together, preserving every behavioral
assertion. [Compiled version red](version-red.log) precedes that migration.
This is no full-pool strategy
expansion (#208 retains that acceptance). Unsupported cards still fail explicitly.

## Ordinary-discovery coverage

`crates/mtg-core/src/token_tests.rs` covers zero/two/four tokens, unique handles,
no premature creation, correct stats, owner/controller/taps, attack rejection,
immediate blocking, two attackers with one death and one player damage, Growth
4/4 survival and cleanup to undamaged 1/1, lethal Bite and stale Growth, deliberate
slot reuse, wrong timing/unpaid rejection and exact state preservation. Resolution
restores at every quantum-1 work boundary and compares full normalized state to
unbounded execution. Storage capacity and second-birth exhaustion preflight before
any token is committed; rejected resolution leaves the spell and all state intact.
Synthetic positions and numeric-boundary hooks are explicitly test-only.

`fodder_normal_reset_policy_capture_and_semantic_replay` in core `tests/capture.rs`
uses the actual frozen red deck with a declared post-shuffle order, plays two
Mountains, casts Fodder on turns 3 and 5, and asserts zero/two/four token checkpoints.
Every submission tests stale/wrong-seat state/history/capture nonmutation and
capture on/off equality. The terminal concession has literal [1,-1] rewards;
semantic and registered played replays reproduce the final state.

Recorder `played_fodder_tokens_survive_typed_conversion_and_jsonl_roundtrip` uses
normal random reset seed 194, actual played lands/payment and Run/Driver capture.
It records two real tokens, compares the typed boundary with the former wire
conversion, persists/loads canonical JSONL exactly, and checks final 1/1/sickness,
untapped tokens and terminal rewards. Wire omission of absent optional fields is
preserved; raw core JSON and wire JSON are not required to have identical keys.

## Matched native and pinned XMage scenarios

`tokens_reference_observations_match_literal_oracle` always runs in normal Rust
discovery. `scripts/token_reference.py` executes that native observer and five
original Java scenarios against the existing pinned XMage revision
`000d8a7abc0ac31cc24af08691423e0c24dc59e7`, checking both against
`fixtures/reference/token-expectations.json`. The Java bridge observes pending
zero tokens, unique created IDs, red/Goblin characteristics, sickness, and final
live/grave token and life counts. It rejects off-turn combat sorcery availability
and casting a second sorcery over the first. Explicit token aliases select the
same Growth/Bite target and combat participant. Original scenarios/bridge are
written here, not copied from upstream tests; existing XMage license remains
in `references/xmage/UPSTREAM-LICENSE.txt`.

The initial Growth/block bridge picked different same-name tokens. It failed
rather than accepting that result. Scheduled XMage combat does not accept aliases;
the repaired bridge uses explicit ID declarations, preserving the 4/4-survives
expectation. Native and XMage observe the same terminal semantic checkpoints;
the native synthetic test sets relevant turns directly, whereas XMage traverses
turns. No shared shuffle/full-game/payment-source trace or Forge agreement is claimed.
Native-only storage overflow is not a Magic rules reference capability.

| Unchanged catalog cases | Native ordinary test / XMage case |
| --- | --- |
| `objects/token-create` positive, negative, regression | creation test / `repeat` |
| `dragon-fodder` positive, negative, regression | creation and timing tests / `repeat` with timing/sickness probes |
| `objects/token-removal` positive, negative | lethal and creation tests / `lethal`, `repeat` |
| `objects/token-removal` interaction | two-attackers test / `combat` |
| `objects/token-removal` regression | pending Growth/slot reuse / `stale-growth` (slot reuse additionally native-only) |
| `goblin-token` positive, negative, regression | creation and combat tests / `combat`, creation-turn sickness probes |
| `goblin-token` interaction | Growth/block/cleanup test / `growth-block` |

Catalog prefixes remain `rules-objects-...` and
`rules-foundations_micro_v1-...`; original owners and verbatim expectations are
unchanged. Token-create interaction requires Goblin Surprise (#203/#209), and
Fodder interaction requires Firebrand Archer (#205/#209). Those two future
composition cases are not certified here. #209 retains composed re-execution.
[Executed reference receipt](reference.json) records exact source, bridge and log hashes.
Python comparator tests detect missing cases and all six observed-field corruptions.

The [existing Growth/Bite reference acceptance](instant-reference.json) also
passes all eleven scenarios twice and detects its strict negative controls.
The [updated native quickstart](quickstart.json) completes both requested games
without failed, truncated or incomplete episodes.

## Reproduction and remaining delivery gates

```sh
cargo test -p mtg-core tokens_
cargo test -p mtg-core --test capture fodder_normal
cargo test -p mtg-recorder --test core_conversion played_fodder
cargo test -p mtg-policy --test heuristic heuristic_fodder
python3 scripts/token_reference.py --cache /home/agent/.cache/xmage --output /tmp/token-reference
./scripts/torture.sh
```

The managed Linux image supplies Rust/Java/Maven; the external pinned cache needs
write access for test-only bridge/build products. No host software or Docker is
invoked. Full torture, separate review and exact-main delivery receipts are linked
from the single [issue workpad](https://github.com/pabloxrl/mtg-lab/issues/194#issuecomment-5953585165).
No new CLI command/setup is required; existing policy/semantic submissions cast
Fodder and select token combat. The root README now distinguishes seven-card plus
token support from the historical six-card M1 gate. M2 remains incomplete.
