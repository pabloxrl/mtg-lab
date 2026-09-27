# GH-74 snapshot acceptance

Atomic R0002-B016/B025 / SYS-REPLAY core serialization portion. No full catalog
case is assigned to #74; #19 retains cross-feature snapshot acceptance and #75
owns semantic replay. M1 remains incomplete.

Original tests in `crates/mtg-core/tests/snapshot.rs` are in normal Cargo discovery.
Requirements supply atomic rejection, complete state, continuation/RNG preservation
and incompatible-version rejection. CR 103.5 and the frozen deck manifest supply
the ordered draw/bottom ledger. The random redraw checks reuse GH-65's independent
[Python arbitrary-precision reference vectors](../mulligan/README.md), including
both starting seats, full hand/library order and a second replacement round.
No expected card sequence is generated from the Rust implementation.

Three compiled behavioral failures against placeholder snapshot/restore methods
preceded implementation ([red receipt](red.txt)): corrupt input was accepted;
restored pending mulligan lacked a decision; restored internal reset yielded
NotStarted rather than the independently expected opening decision. All three
assertions remain in the final suite. Intermediate missing-derive/test API build
errors were corrected; those are not behavioral red evidence.

Ten named snapshot tests cover:

- Literal malformed bytes, unsupported versions 0/2/u32::MAX, incompatible engine,
  truncation, trailing bytes, checksum/payload changes and unchanged destination.
- Typed/structural corruption with a recomputed digest: unknown card, invalid zone
  slot, conflicting scope, malformed candidate set, impossible mulligan count,
  unknown root field. This is limited structural validation, not a claim that
  arbitrary attacker-authored state has been semantically validated.
- Pending first mulligan declaration and the independently expected next actor,
  candidates and bottom count; ordered-card reset at an internal yield.
- Every single-unit random reset/redraw/bottom yield across both seats, with
  independently expected full card sequences after two mulligan rounds.
- Saved explicit chance order while the other seat decides; all seven cumulative
  mulligan rounds and reversed ordered bottom choices, with a separate card ledger.
- Complete serialized-state equality modulo fresh process-local scope at each
  boundary, preservation of reset/object/decision generations, old and foreign
  handle/action rejection, source isolation and subsequent reset.
- Loading bytes in a separate test process and rejecting restored handles in a
  newly created game; unstarted, priority-pass, remembered-card, concession and
  reset boundaries.

[Named green receipt](green.txt) records `cargo test -p mtg-core snapshot`.
Full torture and separate review receipts are recorded below before delivery.

README impact: adds the usable core snapshot API, durable evidence and limitations;
no milestone table change. Existing quickstart commands are unchanged; managed
container torture exercises their underlying repository checks. Workers do not
invoke Docker or host tooling. No policy/enforcement changes or skipped tests.

Reference applicability: snapshot format, RNG continuation and capability scopes
are implementation contracts. Existing bridges do not expose this core's internal
snapshot representation, so no upstream snapshot agreement is claimed. No game
rule semantics changed; later spell-chain integration remains #19.

Three compiled mutation checks were caught, then the production source was
restored byte-for-byte: [dropped queued work](mutant-drop_pending_work.txt),
[reset RNG](mutant-reset_rng.txt), and [reused source scope](mutant-reuse_source_scope.txt).
Each fails a preserved behavioral snapshot assertion; no mutation survives.

Full `./scripts/torture.sh` passed before finalization: 132 Python tests and 140
Rust tests each in debug/release, formatting/Clippy, documentation links, pinned
program ledger and capability catalog. Fresh origin/main was integrated without
changes. [Final torture receipt](torture.txt) records the required repeat after
integration and mutation removal. Independent review is attached to the delivery
PR and tied to its committed candidate; no review result is inferred from tests.
