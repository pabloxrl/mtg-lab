# Cast, target and payment state boundary — GH-131

One internal refactor over main `bfc422b347a8743245fc99f86b691ade7d7dffe3`.
`cast_state.rs` owns the existing `PendingCast`, `Targeting` and `Payment`
representations, their private fields, and transitions. Rule entry points retain
legality checks, error ordering and preflight checks; policy/observation code
uses immutable accessors. No new card, rule, public API, framework or schema.

The existing three `TurnState` slots and serialized field names/order remain.
The new source is included in the existing conservative engine fingerprint:
older-engine snapshots/replays are explicitly incompatible, as before any core
source change. Same-engine restore still rebases capabilities and rejects stale
source handles/decisions. No migration is claimed.

## Independent acceptance and extraction comparison

Tests were added and run against the original implementation before extraction.
This behavior-preserving refactor claims a green compatibility baseline, not new
rule implementation or a fabricated behavioral red. Existing M1 compiled
red/green and independent expectations remain authoritative in the
[casting](../casting/README.md), [targets](../targets/README.md) and
[private/replay](../private-replay-integration/README.md) acceptance reports.
No existing regression assertion was removed or changed.

- `cast_boundary_restore_finish_or_cancel_at_every_target_and_payment_stage`:
  explicit synthetic Bite Down, own/enemy Cub and two untapped lands. Restore
  all eight stages (both targets, completed targets, initial payment, each tap
  and each colored/generic unit), then finish or cancel. CR 601.2c/f–i requires
  one spell on stack, the selected targets, both sources tapped and zero mana
  after payment. Cancellation keeps the spell in hand, both lands untapped,
  no stack entry and no spent mana. Existing pass state and opponent observation
  are preserved; pending choices remain private. Old capabilities reject.
- `cast_boundary_rejections_preserve_snapshot_rng_and_private_choices`:
  wrong-seat/stale submissions at every stage, illegal land target, premature
  payment and duplicate source preserve exact snapshot bytes, including RNG,
  decision generation and provisional choices.
- `cast_boundary_normal_reset_history_and_capture_fingerprint` runs the existing
  seed-160 literal played ledger through the actual Driver: creature casts,
  target/payment cancellation and retry, Growth, factored combat, terminal
  rewards, capture on/off, and semantic replay. Its original literal rules
  assertions and rejection/history/capture nonmutation checks all execute.
  The additional digest compares complete semantic history, captured decisions,
  footer and normalized final state/RNG; only owner capability scopes are
  normalized. It is metamorphic extraction evidence, not a new rules oracle.

[Before](before.log) and [after](after.log) tests pass. Both played-ledger digests:
`5c2c6791a1d695b10516331546b99df450d014fab067e35c7502ccb73262aeb2`.

Reproduction (inside the managed toolchain container):

```sh
cargo test -p mtg-core cast_boundary -- --nocapture
./scripts/torture.sh
python3 scripts/instant_reference.py --cache /home/agent/.cache/xmage --output /tmp/gh131-instant
```

Full torture, applicable live reference execution and separate candidate review
are required in addition to this compatibility baseline. Their receipts and
delivery status are recorded in the
[issue workpad](https://github.com/pabloxrl/mtg-lab/issues/131#issuecomment-5948181798).
Delivery requires protected merge and passing CI on that exact main commit.
Partial R0002-B011/B016/B029 ownership only; full component/M2 acceptance remains
with its original owners and gate. Root README now describes this narrow
boundary; no quickstart command, setup or supported card changes.
