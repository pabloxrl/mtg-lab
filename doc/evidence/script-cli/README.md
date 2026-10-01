# Strict script routing acceptance — GH-178

Partial R0002-B020/B039/B041 only. #120 retains every composed CLI clause, #21
integration/catalog acceptance and #22 the M1 gate. All previous owners,
catalog cases and assertions remain. No rules change, second executor/recorder,
persistence, verification-command expansion or performance qualification.

## Independent original script

[Retained input](../../../fixtures/simulate/script-v3.json) was authored without
reading replay output. Ordered frozen-green decks undergo normal reset (seed
178, episode 0); both keep seven. Under CR103 the starting seat skips its first
draw. Two Forests are played (CR305), Bear Cub is cast with explicit two-mana
payment (CR601), then Giant Growth targets Cub with explicit green payment and
both players pass for resolution (CR608). Cub attacks unblocked on turn 5
(CR508–510), dealing 5. The defender concedes (CR104.3a).

Literal ledger: 101 decisions followed by one concession, 102 consumed records;
life `[20,15]`, libraries `[31,31]`, hands `[5,7]`, four Forests plus a 5/5 Cub on
the battlefield and Growth alone in P0's graveyard. P0's hand arithmetic is
7 + 2 draws − 2 lands − Cub − Growth = 5; P1 is 7 + 2 − 2 = 7. The initial hand
expectation mistakenly said 6; [retained failure](hand-count-red.txt) and
[separate independent correction review](expectation-review.json) authorize only
that correction, with every script record and other assertion retained.

The [original lethal continuation](../../../fixtures/simulate/script-lethal-v3.json)
adds eight unblocked attacks after the first, dealing 5 + 8×2 = 21 damage for
life `[20,-1]`, and ends through rules loss without concession. Explicit cleanup
discards give final hands `[8,7]` and libraries `[23,23]`. Its 434 decisions are
101 + 8 + 7×24 + 8×16 + 13 + 16. The initial test sum was mistyped as 534;
[retained pre-execution failure](lethal-count-red.txt) and
[independent review](lethal-expectation-review.json) approve 434 from the literal
schedule and rules. The reviewer found a third affected consumption assertion;
all three count assertions were corrected as requested, with every record and
other expectation retained. This focused review is not final candidate approval.
The next [retained failure](lethal-kind-red.txt) caught a fixture wire-kind typo:
cleanup records must use the delivered `cleanup_discard` decision name
(`policy.rs` and `policy_combat_tests.rs`), with `discard` as the choice kind.
Only those 13 decision-name strings changed; actions, identities, ordering,
cardinality and every expected checkpoint stayed identical.

## Executable coverage

- `crates/mtg-cli/tests/script_simulate.rs`: closed stdin/display-free bounded
  subprocesses; literal original checkpoints, exact consumption, repeated output,
  swapped seats, two complete episodes, either-seat concession; synthetic omitted,
  extra, reordered, malformed, wrong-seat/episode/incarnation, stale-position and
  illegal record corruptions; byte/count/version/privacy/mode validation; work
  and decision truncation, exact byte-bound acceptance and oversized file rejection. Errors do not echo malformed private payloads or fall
  back to native choices. Genuine completed game accounting remains separate from
  an extra-record caller failure.
- `crates/mtg-cli/src/script_tests.rs`: actual routing through delivered Driver;
  every original record equals the canonical accepted semantic history. Missing,
  malformed, wrong-seat/episode, stale-position and illegal records preserve the
  complete pre-action Driver debug state (Game/RNG, pending work, revisions,
  history, capture, status and accounting) and input cursor. Wrong incarnation
  and illegal second land are rejected; subsequent valid execution recovers.
  Capture on/off final state/RNG/history are equal; captured rewards are `[1,-1]`.
  Deterministic signal/deadline checks stop after one accepted choice and preserve
  remaining input plus not-started episode counts.
- All regressions are in ordinary Cargo discovery. Existing native/pass/replay/
  benchmark tests are retained. The [initial compiled behavioral red](behavioral-red.txt)
  fails at public schema acceptance, not at compilation/import. No new reference
  agreement is claimed because this changes CLI routing, not game rules.

Validation and final clean-candidate independent review/merge/CI receipts are
recorded in the issue workpad and PR; pending delivery is not aggregate completion.
