# Owned scalar episode driver

`mtg_core::episode::Driver` owns normal reset, advancement, policy submission,
concession and one final result. It wraps the existing rules and semantic action
APIs; it is not the full collector. Related to #159. [Acceptance evidence](evidence/episode-driver/README.md).

1. Construct `Driver::new(candidate_capacity)` and call
   `reset(&config, master_seed, episode_ordinal, quantum)`.
2. Call `advance(quantum)` while progress is `InternalYield`. `Ready` includes
   opening, priority, payment, targeting and combat decisions. Each advance
   executes at most one existing core work quantum; completing opening may
   require another advance to start turns. Yields create no semantic records.
3. Call `observe(authorized_seat)` for the existing structured policy observation.
   Only the actor receives a decision/domain; the other seat sees its own
   permitted state. Construct the existing `policy::Submission` using this
   observation's revision, generation, schema version and complete choices.
4. Call `submit(seat, &submission)`. Submission is synchronous, including core
   settlement; a quantum is not a preemptible submission or run budget. The
   driver encodes and validates the privileged semantic record before applying
   it, then appends exactly once on acceptance. Rejected submissions preserve state/history.
   Observe after submission to see the next decision or terminal view, or call
   advance for pending internal work. `advance` after terminal returns `Ended`.
5. Either seat may `concede(seat, episode_id)` under the core concession contract.
   Concession during internal work (including reset and the opening-to-turn gap) is rejected until advancement
   settles. This adds one semantic concession record, not a fabricated policy decision.
6. Call `finish()` once. It returns an owned `EpisodeResult` with
   `Completed(outcome)` only for a real core outcome; all other states are
   `Incomplete`, including partially executed reset. Further mutation is rejected.
   Call reset explicitly to start another episode; resetting before finish is
   rejected even at terminal. Invalid reset preserves the prior state/history.

Reset creates a fresh Game after validation. Driver-local revision tokens reject
submissions from earlier resets; generations retain core decision freshness.
Concession uses the core episode token. Route each command to its intended driver
and authenticated seat externally: this local API is not transport authentication.
There is no mutable Game reference, restore, unrecorded apply or automatic retry.

`privileged_snapshot`, `privileged_history`, and a result's `privileged_inputs`
are expressly privileged/debug interfaces, never policy inputs. Inputs retain the
actual cloned Config, master seed and ordinal. Result fields are private and its
getters are read-only; owned copies outlive driver reset or destruction. Snapshots
include full private state and RNG; semantic history includes hidden identities.
Debug formatting of driver/results is also privileged. Policy observations do
not contain those artifacts. Supplying a seat is not permission to expose a
privileged artifact to that seat.

The history uses the existing versioned `game::actions` bytes. It can be reapplied
from the same actual reset inputs with `actions::apply`, starting turns after
opening. Incomplete histories need not replay to a settled or terminal state;
particularly an unfinished reset has no player actions. This driver supplies no
replay envelope, replay resolver, policy selection,
limits/truncation policy, outcome accounting across runs, disk I/O or publication.
Memory grows with accepted history and optional capture; no bounded-memory or
throughput claim is made. Later composition remains #161–#164, with all original
integration acceptance retained by #154/#117 and M1's gate #22.

## Canonical capture

The driver also supports [optional owned canonical v2 capture](episode-capture.md)
through `reset_captured` and `submit_with_policy`. The original `reset` execution
path remains capture-disabled. Budget, replay resolver and publication work is
separate; both modes retain privileged history.

## Privileged semantic input

`submit_record(authorized_seat, context, bytes)` accepts the existing
[`actions` v1 semantic record](actions.md) through the same validated `submit`
or `concede` path. It never exposes a mutable Game or applies through a second
execution loop. Capture uses absent/default policy statistics and appends the
canonical accepted submission exactly once; caller JSON whitespace/order is not
preserved. This is privileged replay/debug input, not a policy observation or
transport authentication. The caller authenticates the seat and routes to the
intended Driver.

For a decision, supply `RecordContext::Decision { revision, generation }` from
that seat's current `observe` decision. The record's actor must match the
caller-authorized seat; the context must match the current driver revision and
core decision generation. The driver translates the decoded core revision to its
owner revision before ordinary submission. For concession, either seat may
supply `RecordContext::Concession { episode: driver.episode_id().unwrap() }`;
priority is unnecessary, but an old/foreign episode token or a decision context
is rejected. Live context is separate from saved records: semantic v1 remains
unchanged and stores neither revisions nor generation tokens.

Advance internal work explicitly before input. Malformed, unsupported, incomplete,
illegal, wrong-seat, stale-context and stale-object records reject without
changing game/RNG/history/capture. Existing budget boundary checks still apply:
a due limit can finalize truncation, and an exhausted record slot quarantines
recording without accepting the attempted choice. Subsequent input cannot mutate
a stopped, terminal or finalized episode. There is no retry, fallback choice,
CLI change, new rules, or throughput qualification. See the
[normal-reset acceptance and compiled behavioral failure](evidence/semantic-input/README.md).
