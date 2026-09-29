# Canonical in-memory trajectories

`mtg_core::trajectory` provides owned records and per-seat readers for the M1
native scalar boundary. [Acceptance and original expectations](evidence/trajectory/README.md)
cover the atomic portion of RFC 0002 B036/B037. [JSONL validation/export](trajectory-jsonl.md) is provided by `mtg-recorder`;
full recorder integration is #20. No trainer, storage queue, Parquet, batch worker,
recurrent tensor loader or full-game collection driver is supplied here.

## Structured played-policy schema v2

`mtg_core::trajectory::v2` is the structured version of the canonical in-memory
contract. The original `mtg_core::trajectory` types and `SCHEMA_VERSION = 1`
remain source- and behavior-compatible; v2 has `SCHEMA_VERSION = 2`. There is
no lossy conversion between them. A v1 recorder rejects a v2 header and a v2
recorder rejects a v1/unknown header. Structured observation and Submission
versions are independently checked against `policy::SCHEMA_VERSION` (currently
1). Engine/rules/cards/action strings remain producer provenance, not migration
instructions. The existing [JSONL writer/reader](trajectory-jsonl.md) and [run manifest](run-manifest.md)
support explicit v1 and v2 schemas. The owned collection driver remains #154. No
collector, CLI, trainer or new rules are supplied by the in-memory contract.

Create a v2 `Frame::capture(&game, capacity)` at an existing policy boundary,
then `v2::Recorder::new(&header, &frame)` with `header.versions.schema = 2`.
Capture invokes `policy_observe` for both seats; it owns the complete authorized
`Observation`, including the real `Decision` candidate table/mask, factored combat
domains and provisional selections, pending spell/targets/sources/pool/cost,
committed stack targets and combat relationships. Capacity failure and internal
work/uninitialized states return `Unavailable`; nothing is silently dropped.
Private native episode identity is only a nonserialized continuity guard.

For each accepted policy command, retain the before frame, call the game's
`apply_policy`, capture after, and append a v2 `Choice` containing the **entire
Submission**, timing IDs, status and supplied-only `PolicyInfo`. No selected flat
index is invented: bottom/discard submissions retain all choices in order, and
combat submissions retain subset/map/allocation parameters. Candidates and masks
are the exact structured policy inputs, not a new numeric encoding. A zero-damage
recipient may be omitted just as in the existing combat API; the sparse semantic
submission is preserved without expanding or sorting it.

Validation rejects stale revisions/generations, foreign episodes, mismatched
before frames, unchanged/backward post-decision frames, invalid cardinalities,
masked/unknown/duplicate selections, invalid factored references/allocations,
nonfinite statistics, incompatible schemas and inconsistent timing/status, before
mutating the recorder. Like v1, this is a **trusted producer storage contract**:
it checks the observed domain, not an engine execution receipt. The producer is
responsible for associating the actual accepted command and settled result and
for not omitting decisions. It must not pair a different legal command with a
post-action frame or replace a failed command with a successful row. #154 owns
that lifecycle. Finalization outside an action permits a genuine terminal outcome
(e.g. concession); the producer must not use it to hide intervening decisions.

Timing is explicit and validated:

- IDs start at logical action 0, microchoice 0. Each accepted Submission is one
  decision, even when it contains several ordered bottom/discard choices.
- `Cast` begins a logical action; targets, finish-targets, provisional mana taps
  and payment choices continue it. `FinishPayment` commits it; cancel-targets or
  cancel-payment ends it as `Cancelled`. The cancelled attempt still counts as
  one logical action and all its decision rows remain in the history.
- Attacker subsets, blocker maps and damage allocations are `Continuing` until
  `FinishCombat`. Replacements/backtracking retain earlier rows and use the same
  logical ID with increasing microchoice IDs. They are not cancellation events.
- Other decisions (including an immediate mana tap outside a pending cast,
  pass, keep, mulligan and a whole bottom/discard submission) are `Committed`.
- A continuing row requires the same actor/logical ID and next microchoice ID.
  After commit/cancel, the next row uses the next logical ID and microchoice 0.
  A recorder may start at a policy boundary inside a continuation; its first
  observed row starts at 0/0 and counts the observed action suffix once.

The v1 half-open interval convention below also applies to v2. In particular,
`logical_actions_elapsed` counts touched logical IDs, including a partial action;
it is not an additive count of completed casts. `cancelled_actions_elapsed`
counts cancellation events in that same interval. The footer separately records
logical-action and cancelled-action counts. Rejected commands produce no rows,
advance no IDs and consume no recording rewards. Truncating mid-continuation
retains the pending authorized final observation, without implying cancellation
or commitment. Failure quarantines the record. Gamma remains 1.

Rewards are sparse terminal +1/-1/draw 0, delivered once through decision deltas
or an external boundary delta. Both seats get same-seat next/final observations;
a zero-decision seat gets only its final unassigned credit. Aggregate returns
are not additional rewards. Terminal, truncated and failed states stay distinct.
Owned observations, submissions, header and optional statistics survive input
mutation and game reset. Seat readers exclude the other seat's decision input
and restricted replay header reference. Seeds and privileged identities never
become policy data; the full episode still contains both seats' authorized views
and must not be published during live play.

[Executable acceptance, independent expectations and red/green evidence](evidence/trajectory-v2/README.md)
cover the bounded v2 contract. #117 retains all original aggregate collector
acceptance; #20/#120/#21/#22 retain integration and M1 gate responsibilities.

## Legacy v1 producer contract

A trusted collector builds a `Header`: globally unique run UUID plus an episode
ordinal never reused within that run; engine/rules/card/action/observation/schema
versions; SHA-256 deck/configuration hashes in seat order; fixed policy IDs and
versions for both seats; starting seat; decision/turn/wall-time limits; and an
optional opaque restricted replay reference. The run UUID must be allocated
uniquely across processes/restarts by the collector. The recorder validates its
shape, not global uniqueness against external datasets. Policy IDs include their
version; changing opponents requires a new episode. Seeds and private replay
configuration belong in that separately permissioned artifact, never features.

`Frame::capture(&game)` copies both authorized player views and binds a private,
nonserialized native episode token. It rejects uninitialized/internal-work/private
spell-continuation states where the current player-view API is unavailable. It
never inspects a privileged snapshot or consumes randomness. Capture the initial
frame, then call `Recorder::new(&header, &frame)`.

For every policy decision, keep the pre-action frame, submit the action through
the engine's existing validated command, capture the resulting frame, and call
`append(&before, &choice, &after)`. The trusted action encoder supplies the exact
candidate table (semantic string and numeric feature row), legal mask, chosen
index, decision kind, and logical-action/micro-choice IDs. The semantic string is
a versioned encoder payload, not an instruction executed by the recorder. The
selected semantic action is copied from that row, never separately guessed.
This storage contract does not generate candidate sets or certify an arbitrary
collector's legality table; engine command validation remains authoritative.
Action feature dimensions and semantic grammar belong to the declared encoder
version. Nonempty engine/rules/card/action version identifiers are retained
verbatim; only recorder and player-view schema versions are interpreted here.

`append` validates continuity against the last owned frame, native episode
identity, actor, masks/selection, consecutive action/micro-choice IDs and finite
optional statistics. Rejection changes nothing. There is no implied alternation.
Logical IDs start at zero, remain equal for successive micro-choices, and advance
by one for the next action. Micro-choice IDs start at zero and increment within
an action. The collector must record every decision, not multiple commands as
one row. The API does not audit commands a collector never submits to it.

Records own observations, candidate features/masks and policy metadata. Mutating
input buffers, resetting the game, or editing a returned seat sequence cannot
rewrite a retained episode. `episode()` only exposes shared immutable references;
cloning an episode makes an independent snapshot, including an incomplete prefix.
All fields serialize through Serde for inspection; persistence and decoding are
not advertised by this in-memory implementation.

## Rewards, endings and time

Serialized episodes explicitly declare `SparseZeroSumTerminal`,
`UndiscountedEpisodic` (gamma = 1) and `AllDecisions`. Intermediate rewards are zero;
terminal winner +1, loser -1, simultaneous draw zero. A terminal post-action frame
automatically closes the recorder and puts the reward delta on that decision.
Subsequent appends/finalization return `AlreadyEnded`.

For an ending outside a policy action, call `finish(&final_frame, End::Completed)`.
It requires an actual rules-terminal view; concession is a rules outcome. The
footer's `boundary_reward` contains this external reward delta. It is zero when
the delta already lives on a terminal decision. `returns` is an aggregate, not
another reward event. For global accounting sum decision deltas plus
`boundary_reward`, never add the aggregate again.

`End::Truncated(Decisions|Turns|WallTime)` requires the unchanged final live frame
and records its reason. Limits are declared metadata; the collector enforces
them. This preserves authorized bootstrap observations and does not invent a
draw. `End::Failed(reason)` marks recording completeness false and quarantines the
sample, even if the retained view shows a completed game. On an engine/encoding
failure where no new authorized view exists, pass the last valid frame: it is a
diagnostic snapshot, not a bootstrap target. `footer()` is explicit diagnostic
access; ordinary `seat()` loading rejects unfinished and failed records.

Legacy v1 private spell-continuation capture remains unavailable:
`Frame::capture` returns `Unavailable` there. Use structured v2 for these views. Callers must propagate
that failure/quarantine, never silently drop that decision or claim complete
recording. No autoreset occurs. Capture final frames before reset; a new native
episode token cannot finalize or extend the old recorder.

## Per-seat data

`episode.seat(Seat::P0)` or `P1` returns only that seat's decision observations,
choices, next observations and final observation. It never includes the other
seat's decision input, header policy metadata or restricted replay reference.
A research caller holding the full `Episode` can inspect both hands separately;
that artifact is not safe to publish during live play. Candidate features and
optional policy state must also be seat-authorized by the trusted collector.

Transitions link to the same seat's next decision, or its genuine final view.
They accumulate that seat's deltas over the half-open global interval
`[source decision, next same-seat decision)`; the last transition extends through
the final recorded decision plus any footer boundary reward. `decisions_elapsed`
counts source/intervening decisions and excludes the destination and separate
final event. `logical_actions_elapsed` counts distinct logical IDs touched by
that interval, including an action begun before the source. Thus separate seat
intervals can touch the same logical action; these durations are not additive
completed-action counters. The footer counts distinct action IDs in the whole
stream, including an unfinished action at truncation. No nonunit discount mode
or completed-cast counter is implied.

A seat with zero decisions has no fabricated transition/action. Its
`SeatSequence.unassigned_reward` is the sole reward-bearing final-seat record.
For an acting seat that field is zero. Sum transition rewards plus unassigned
reward to get `total_return`. Repeated reads are pure, not reward consumption.
Episode/global/per-seat IDs survive copying/reset and remain explicit. Per-seat
exports also retain version identifiers and the discount convention.

Missing log probability/value/checkpoint/recurrent state/exploration stay `None`,
never fabricated zeros. `require_probabilities()` rejects incomplete/failed
records or any absent behavior probability. Consumers must still assess whether
the supplied collector statistics and policy provenance suit their algorithm.

Run the named acceptance module in the managed toolchain container:

```sh
cargo test -p mtg-core trajectory
./scripts/torture.sh
```
