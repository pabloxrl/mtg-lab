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
   it, then appends exactly once on acceptance. Errors preserve state/history.
   Observe after submission to see the next decision or terminal view, or call
   advance for pending internal work. `advance` after terminal returns `Ended`.
5. Either seat may `concede(seat, episode_id)` under the core concession contract.
   Concession during internal work (including reset) is rejected until advancement
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
replay envelope, replay resolver, trajectory frames, timing, policy selection,
limits/truncation policy, outcome accounting across runs, disk I/O or publication.
Memory grows with accepted history; no bounded-memory or throughput claim is made.
Canonical capture and later composition remain #160–#164, with all original
integration acceptance retained by #154/#117 and M1's gate #22.
