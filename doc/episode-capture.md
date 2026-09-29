# Owned canonical episode capture

`mtg_core::episode::Driver` connects its real accepted submissions to the existing
`trajectory::v2::Recorder`. [Acceptance](evidence/episode-capture/README.md) covers
normal-reset play and explicitly identified component edges. This is the in-memory
capture component of #160, not the complete collector or an M1 gate verdict.

Use `Driver::reset_captured(config, master, ordinal, quantum, &header)` instead of
`reset` to enable capture for an episode. The header must satisfy canonical v2
validation and match the actual ordinal/starting seat. Other header provenance
and policy identity remain caller declarations; deriving run provenance belongs
to #163. No configured header budget is enforced here. `reset` starts an episode
without capture. Either reset requires explicit `finish` of the previous episode.
Capture cannot be toggled halfway through a game.

Advance through `InternalYield` until `Ready`, observe the authorized seat, and
submit the entire `policy::Submission`. Both `submit` and `submit_with_policy`
record every successful decision, including ordered bottom/discard lists,
provisional combat selections, targets, payment, cancellation and retries.
`submit_with_policy` copies supplied `PolicyInfo`; `submit` supplies no statistics.
Invalid statistics are rejected before applying the action, in either capture
mode. Log probabilities must be finite and nonpositive; values must be finite.
No probability, value, checkpoint or recurrent state is inferred.

The driver retains exact action-time observations, domains/masks and full
submissions using its own reset revision. It derives logical-action/microchoice
indices and committed/continuing/cancelled status from accepted operations and
their actual resulting boundaries. Internal opening-to-turn work creates no
extra decision; the next actor is resolved when that work reaches its boundary.
Concession is an out-of-band semantic operation, never a fabricated policy row.
Rules-terminal rewards come from the terminal action; concession rewards come
from the boundary. The existing v2 seat readers link to the same seat's next
input or final observation, including a seat that never acts again or never acts.

`trajectory()` is immutable diagnostic access. `finish()` returns an owned
`EpisodeResult` containing optional `trajectory()` data and `capture_requested()`.
Completed captures have a canonical footer and available seat readers. Finishing
unfinished work retains an unsealed episode (no footer, quarantined seat readers)
when an observable frame has been reached. An unfinished partial reset can have
no frame yet: its result has `capture_requested() == true`, no trajectory, an
explicit `Incomplete` status and its exact privileged state. This does not invent
a truncation, loss or configurable budget policy. Results survive later resets;
a repeated finish, stale submission, or post-finalization mutation fails.

Input capacity remains the driver's configured limit. Capturing a resulting
frame preserves its full domain even if that next input exceeds the limit:
that next observation/submission fails explicitly, while previous accepted rows
remain available diagnostically. Capturing never silently truncates candidates.
Unexpected capture invariant failures return a capture error and prevent further
mutation or successful finalization; no successful complete episode is advertised.
Error accounting and configurable budgets remain #161.

The global episode contains both seats' separately authorized inputs and is
sensitive dataset data. Give policies only their own observation or seat reader.
Privileged reset inputs, semantic history, full snapshots and RNG remain on the
separately named driver/result methods, never in canonical policy records.
Transport authentication is external. This API makes no disk writes, policy
choices, replay authorization, run publication or performance claim. Both history
and canonical capture grow with accepted decisions; it is a scalar memory API.
