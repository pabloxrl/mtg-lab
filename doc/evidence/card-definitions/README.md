# Typed supported card definitions — GH-132

One internal refactor over main `6e36df0d180ffddd01a5ed93210871d0eab7c54c`.
`card_definitions.rs` centralizes costs, base power/toughness, basic mana colors
and supported behavior tags for Forest, Mountain, Bear Cub, Swab Goblin, Giant
Growth and Bite Down. Casting, targeting, mana and combat read this typed source.
No card expansion, Oracle interpreter, source repinning, public API or wire schema
change. Identity ordering and all frozen hashes remain unchanged.

The existing Magnigoth Sentry synthetic-position characteristics remain explicitly
unsupported for casting/combat. Its 4/4 target/inspection hook is preserved for
existing fixture expectations; this does not implement reach or advertise support.
Other frozen cards/token receive no default characteristics or play permission.
The new source participates in the existing conservative snapshot/replay engine
fingerprint, so previous-engine artifacts remain explicitly incompatible. There
is no snapshot migration claim.

## Independent acceptance

Four new Rust tests ran successfully against the original implementation before
extraction ([before](before.log), [after](after.log)). This is a green refactor
baseline, not a new-rule behavioral red. An initial test-authoring method-name
compile error was corrected before baseline execution; it is not red evidence.
Every existing regression expectation remains intact. Original rule red/green
and CR/Oracle expectations remain in the [casting](../casting/README.md),
[targets](../targets/README.md) and [M1 gate](../m1-gate/README.md) reports.

- `card_definitions_six_card_manifest_and_public_candidates`: independent six-card
  M1 list; pinned manifest costs, content hashes, instant type and 2/2 creature
  stats; explicit basic colors; public cast/land candidates. CR 107.4/302/305/601.
- `card_definitions_every_other_frozen_identity_rejects_play_without_mutation`:
  every other identity (14 cards plus token), sufficient mana, no cast/land
  candidate, exact cast/target errors, snapshot/RNG nonmutation, and unsupported
  combat after synthetic battlefield insertion.
- `card_definitions_fixture_characteristics_do_not_enable_sentry`: legacy 4/4
  characteristics remain, with no casting cost or supported combat.
- `card_definitions_unknown_and_corrupt_identity_rejection`: unknown keys and
  invalid serialized IDs reject; all 21 frozen wire IDs round-trip unchanged.
- `test_every_frozen_card_rejects_changed_source_and_content_hashes`: each of all
  21 records rejects corruption of each content/source-projection/Oracle/raw hash
  with an explicit digest error. Existing file-pin/source-integrity rejection
  tests remain intact; [manifest tests](hashes.log) pass. Manifest validation is
  the existing offline admission boundary, not new runtime source fetching.

The existing normal-reset seed-160 Driver test, with its independently literal
rules ledger, capture on/off, semantic replay and rejection assertions, passes
[before](played-before.log) and [after](played-after.log). Both complete played
history/capture/normalized-final-state digests equal
`5c2c6791a1d695b10516331546b99df450d014fab067e35c7502ccb73262aeb2`.
This digest is extraction equivalence evidence, not a rules oracle.

Reproduction inside the managed container:

```sh
cargo test -p mtg-core card_definitions -- --nocapture
cargo test -p mtg-core cast_boundary -- --nocapture
python3 -m unittest discover -s tests -p test_card_manifest.py
./scripts/torture.sh
python3 scripts/instant_reference.py --cache /home/agent/.cache/xmage --output /tmp/gh132-instant
```

Full torture after current-main integration, actual pinned XMage comparisons,
and prescribed separate candidate review are required in addition to these
focused baselines. Receipts and final protected merge/exact-main CI status live
in the [issue workpad](https://github.com/pabloxrl/mtg-lab/issues/132#issuecomment-5949614601).
Only partial R0002-B010/B011/B029 ownership; original component and M2 gate retain
full acceptance. README describes the delivered internal architecture while
retaining six-card support and M2-pending limitations. No quickstart changes.
