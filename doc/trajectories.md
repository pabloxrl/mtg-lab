# Canonical in-memory trajectories v1

`mtg_core::trajectory` provides owned records and per-seat readers for the M1
native scalar boundary. [Acceptance and original expectations](evidence/trajectory/README.md)
cover the atomic portion of RFC 0002 B036/B037. JSONL validation/export is #77;
full recorder integration is #20. No trainer, storage queue, Parquet, batch worker,
recurrent tensor loader or full-game collection driver is supplied here.

## Producer contract

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

Private spell-continuation truncation requires the missing complete view contract
owned by #19; `Frame::capture` returns `Unavailable` there. Callers must propagate
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
