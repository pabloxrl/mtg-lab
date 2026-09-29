# Strict structured trajectory persistence (#153)

Bounded R0002-B036/B037 evidence only. #152 supplies the canonical contract;
#154 owns collection, #117 every original aggregate collector check, #20 full
integration, #120/#21 CLI and #22 the M1 gate. No played-game collection or M1
completion is claimed here.

## Independent expectations

The normal-discovery `crates/mtg-recorder/tests/structured_contract.rs` uses
hand-authored `structured.json`, `structured.jsonl` and
`structured-manifest.json`. RFC 0002 §8 requires exact action-time inputs and
submissions, owned buffers, once-only sparse terminal credit, private seat
isolation, explicit versions, integrity and visible incomplete/error outcomes.
The #152 contract in `doc/trajectories.md` specifies policy-v1 command vocabulary,
full ordered choices, factored membership and commit/cancellation timing.

Seven synthetic complete record boundaries cover payment cancellation with
pending spell/targets/sources/pool/remaining cost, ordered two-card bottom and
cleanup discard, a two-attacker subset, two blockers assigned to one attacker,
a power-two split of one damage to each blocker, and a Growth target choice.
External P1 concession supplies boundary reward `[1,-1]`, decision reward `[0,0]`,
terminal flags and seat-local final observations. These are declared synthetic
storage inputs, not a claim that a collector played those games. CR 103.5/514
justify ordered multi-card choices; CR 601/508–510 justify complete pending and
combat payloads. The observed action suffix starts at logical/microchoice 0/0
as #152 explicitly allows. Null targets/sources mean departed references; sparse
zero-damage recipients remain valid, as documented by policy-v1.

Every literal field is stored in the fixture; no production output supplies an
expected observation. Canonical fixture bytes and both checksums were assembled
with Python `json` sorted compact encoding and `hashlib.sha256`, before the
implementation. Initial fixture vocabulary was corrected against the existing
policy contract (`growth_target`, `cleanup_discard`) and expanded to complete
flat candidate tables before implementing strict structured validation. Initial
compiled reds are unchanged; no old v1 fixture or test changed.

A separate handwritten three-row ledger (P0 cancellation, P1 pass, P0 pass, P1
concession) expects P0 intervals `[2,1]` and rewards `[0,1]`, P1 interval 2 and
reward -1. A zero-action seat gets only unassigned terminal credit. Core conversion
also captures a normal-reset boundary, truncates without a game action, resets
and compares owned serialized final inputs. It is not a substitute played-game
collector integration.

## Regression coverage and evidence

- `red.log`: three compiled intended failures for v2 validation, writer bytes
  and manifest support before implementation (with compile-only API scaffolding).
- `seat-red.log`: compiled intended seat-reader failure before its implementation.
- `kind-red.log`: a bottom domain mislabeled as priority was accepted; strict
  command vocabulary/cardinality now rejects it.
- `payment-red.log`: the initial validator overrestricted a native standalone
  mana payment (no casting spell). A normal-reset core capture proves policy-v1
  permits `pending: null` there; the durable validator now preserves it. Actor-only
  pending isolation and missing required-field rejection remain enforced.
- Normal-discovery regressions retain independently re-sealed malformed records,
  required/missing/extra fields, wrong/unknown versions, masks, revisions, domain
  and multi-choice errors, provenance/count/checksum mismatches, incomplete runs
  and fragments, and cross-version rejection.
- A gated sink proves producer blocking and measured wait. Fail-on-overflow,
  undersized budgets, partial writes, disk/final-flush errors poison or fail
  finalization. File publication is no-replace; failed fragments remain explicit.
- Mutation of producer buffers cannot change queued rows. Optional statistics
  remain absent; supplied finite floats preserve bits. Seat loaders exclude
  opponent hand/pending inputs and restricted replay links.

Run `cargo test -p mtg-recorder --test structured_contract`, then the full
`./scripts/torture.sh`. Final verification and separate review receipts accompany
the delivery PR; no reference-engine agreement is claimed for this storage task.

README now distinguishes v1/v2 durable API support from the v1-only CLI and the
planned collector. Setup/quickstart commands and milestone verdicts are unchanged;
the complete suite exercises the existing commands. The independent reviewer must
check README accuracy, unchanged v1 fixtures/tests and this limited ownership.

Fixture review notes: card strings were corrected to frozen pool keys, the
reserved Forest remains publicly untapped until payment commits, bottom/discard
inputs have seven/nine cards and complete flat tables, their final observations
remove the two selected cards and adjust library/graveyard data, and combat
snapshots use the declared steps/active seat and committed combat relationships.
These corrections come from the pre-existing card/policy/rules contracts, not
recorder output. They strengthen the new synthetic fixtures; all v1 fixtures and
tests remain byte-identical. Buffer tests now budget the larger of their two
independent rows, preserving the same Overflow/blocking assertions after the
fixture expansion. No asserted failure was changed to success to bless output.
The first full torture attempt stopped on two Clippy collapsible-if findings;
those style findings were repaired without disabling any check.

Final managed-container verification: both complete torture runs PASS, including
the final run after fresh-main integration. Each run includes 143 Python tests
and 334 Rust checks (including two doctests) per debug/release profile, plus
documentation/program/catalog checks, formatting and Clippy. All 15 new contract
tests are in normal discovery and run in both profiles. `verification.json`
pins source and log hashes. No existing test was removed, skipped or weakened.
