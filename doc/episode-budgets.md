# Bounded owned episodes

`episode::Driver::bounded(capacity, Budget, Box<dyn Clock>)` adds episode limits
to the [owned capture API](episode-capture.md). This is #161's in-memory component;
[executable acceptance](evidence/episode-budgets/README.md) does not complete the
collector, its integration audits, or M1. `Driver::new` remains unrestricted by budgets, while sharing once-only outcome
and recording-failure accounting. No CLI command is added.

`Budget` contains canonical `trajectory::Limits`, a nonzero `work_quantum` ceiling,
and a nonzero `records` ceiling. Decision, turn and millisecond limits are optional;
provided zero limits and zero input capacity are rejected before a game starts.
The caller implements `Clock::now_ms()` with monotonic elapsed milliseconds (for
example, relative to its own `Instant`). The rules core does not read a clock.
A clock going backwards during a live episode fails it. Reset starts a new timer
before reset work. Captured headers must match the enforced limits exactly;
`EpisodeResult::budget()` preserves the actual configuration independently.

Use the normal reset, advance, observe, submit/concede and finish lifecycle.
`Progress::Stopped(Status)` reports a budget stop from reset/advance;
`status()` reports stops after accepted input. A successful submission still
means its complete decision was accepted and recorded, including the decision
that reaches a limit. Subsequent input and advancement fail. Pure observations
are read-only and do not poll the clock. `finish()` returns the owned result once;
reset requires that result to have been taken. Neither operation repeats rewards
or counts. Invalid reset configuration does not start a new episode.

## Boundaries and precedence

Checks run after reset work, before and after each advance quantum, before each
submission/concession, after its accepted atomic operation and capture, and at
finish. A boundary that has already stopped is immutable. Within a live boundary:

1. Recording or representation-capacity errors produce `Failed`, never completion
   or an ordinary bootstrappable truncation. An invalid user choice/statistic is
   instead a rejected input, with no accepted decision or record.
2. A genuine rules-terminal outcome, successfully captured when requested, is
   `Completed`, even when that operation also reaches a limit.
3. Otherwise reached limits choose `Decisions`, then `Turns`, then `WallTime`.
   A detected backwards clock is a failure rather than a limit.

Decision limits count every accepted policy submission, including opening,
ordered multi-choice submissions, pending targeting/payment, cancellation and
combat continuations. They do not count internal work or out-of-band concession.
`accepted_decisions()` preserves this count even if a capture error prevents a
canonical row. No pending action is committed, cancelled or replaced at a stop.

A turn limit N permits N turns. It stops at the first resumable-work or accepted
operation boundary whose current turn number exceeds N, before another policy
input is accepted. A synchronous operation can finish cleanup and enter the next
upkeep; that actual final state is retained. A wall-time limit is elapsed time
since reset, including time waiting for the caller. Work is never advanced just
to produce a more convenient final observation.

Reset/advance use `min(requested quantum, configured work_quantum)` on the existing
core work API. Policy submission remains synchronous and atomic, as in the
registered contract: these are boundary checks, not preemption inside a rules
operation, allocation or observation. Time can overrun within one such operation.

## Records, outcomes and ownership

`records` bounds semantic history entries (including concession) and therefore
also bounds canonical decision rows. Capacity is checked before accepting the
next encoded operation; exhaustion fails visibly without applying it. Retained
rows are never evicted. This is a record-count bound, not a total-byte/RSS bound:
individual observations, full submitted choices, policy metadata, snapshots and
caller-retained results have variable sizes. No disk queue or publication exists.

`Accounting` has started/completed/truncated/failed/incomplete counts, independent
of capture. Started increments only on a successful core reset. A stop accounts
once immediately; explicit finish of still-live work accounts once as Incomplete.
At most one episode is live per driver and it must be finished before reset.
There is no automatic retry/reset, implicit game loss or synthetic terminal action.

At observable boundaries truncation seals the existing canonical recorder with
zero reward and the actual same-seat final inputs. Real completion retains its
sparse terminal reward once, including nonacting and zero-decision seats. Failed
capture retains all available rows in a quarantined diagnostic episode; its
footer describes the last captured frame, while the owned result retains the
actual state/history and any available final observations. Training readers reject
it, even if the game itself ended. Capture error details are returned by the
failing call; `Status::Failed` retains its category.

An internal-work stop may have no authorized final observations yet.
`final_observations()` is then `None`; the exact privileged snapshot is retained.
Any existing capture prefix stays unsealed and quarantined. Thus a truncated
execution is accounted, but is not advertised as a complete training sample when
its final inputs are unavailable. `finish()` never fabricates or advances a frame.
Results and their frames survive all subsequent resets. Both-seat final arrays
and global trajectories are dataset data: policies receive only their own seat.

Disk persistence, seed selection, replay access and run publication remain their
registered downstream tasks. No rules, sampling algorithm, concurrency setting,
workflow gate or milestone verdict changes here.
