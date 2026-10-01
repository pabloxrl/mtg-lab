# Continuous unattended delivery acceptance

Scope: operations [#189](https://github.com/pabloxrl/mtg-lab/issues/189). The
operator's explicit requirement removes per-issue elapsed-time and lifetime
attempt stops; all authorization, verification and protected-delivery gates
remain. No product requirement, catalog ownership or milestone credit changes.

## Executable evidence

Normal Python discovery includes:

- `tests/test_before_run.py`: executes the real hook in temporary GH-189
  workspaces with controlled subprocess/API/time/container boundaries. Old
  timestamps, 80 previous dispatches, and repeated merged-awaiting-CI receipts
  must continue without GitHub mutations. Original timestamp/extra fields and
  existing receipts survive. Identity/fresh-main/execution checks still run;
  invalid containers, incomplete prerequisites, malformed diagnostics and Git
  failures reject. No live credentials or GitHub writes occur in these tests.
- `tests/test_delivery_continuation.py` and
  `tests/fixtures/symphony/continuation.exs`: execute the image's pinned Symphony
  v0.0.3 runner and orchestrator, unpacked into a disposable home with a fake
  token. Two actual runner sessions return after 12 turns each with the issue
  still active. Only workspace/hook and Codex process boundaries are stubbed;
  the runner, scheduler and workflow parser are the real packaged modules.
  Normal worker exit schedules a one-second continuation, while abnormal exits
  schedule 10 seconds, 20 seconds and a 300-second cap even at attempt 81.
  Timer deadlines are asserted without sleeping. Missing ready, closed state
  and non-dispatchable status stop continuation; an occupied worker slot rejects
  a second issue. State-refresh failure remains an error, never completion.
- Workflow assertions preserve program pause/cancel, held/blocked, removed-ready
  and deferred-grant instructions. These are worker/handoff policy gates, not
  native controller enforcement of parent labels. Operators remove ready when
  holding a task. The hook never restores any authorization.

Before implementation, the hook tests reproduced three age/count exits at
`Issue paused at retry/time limit`, plus malformed-state writes and unwanted
GitHub mutations. After the change, those same behavioral cases pass. Expected
outcomes come from #189, independently of prior hook behavior. No existing test
was removed or weakened.

Bounded credential-free commands inside the managed image, from the checkout:

```bash
timeout 30s python3 -m unittest discover -s tests -p test_before_run.py -v
timeout 110s python3 -m unittest discover -s tests -p test_delivery_continuation.py -v
timeout 100s python3 scripts/symphony/continuation_smoke.py
./scripts/torture.sh
```

The smoke uses `/usr/local/bin/symphony` installed by the pinned image, not a new
host installation. Its usage-only invocation intentionally exits 1; both status
and usage marker are checked. Missing/wrong runtime fails, never skips. It starts
no live controller supervisor, HTTP service, GitHub client or Codex worker.
Full torture and separate clean-candidate review results, candidate/merge SHAs,
and exact-main CI links are retained in the issue workpad and related PR.

## Recovery boundaries

Inspection of the installed release's `AgentRunner` and `Orchestrator` compiled
modules confirms the tested session return and retry paths. Normal completion
returns `:ok` at max turns and schedules a fresh continuation check; the internal
completed-session set is not GitHub issue completion. Retry poll API errors
schedule another bounded-backoff attempt. The smoke tests session/worker events
and state-fetch failure, not every external provider error or a whole outage.
Turn/stall, hook, request, review and CI timeouts remain unchanged.

Still-open ready issues can be rediscovered after restart with persistent
workspaces. Existing historical restart evidence remains in the runbook.
Recovery after arbitrary termination, especially between handoff mutations,
is not guaranteed. Force-closed issues, deliberate holds/blocks, input-required
responses, unavailable authorization and genuine scope changes require visible
reconciliation. Counters never substitute for that diagnosis. Historical receipts
remain history; current policy requires no routine counter reset.

## Deployment handoff (coordinator only)

Code delivery and deployment are separate. A merged PR and exact-main CI do not
prove that the currently running baked controller has this policy. Follow the
[runbook deployment procedure](../../symphony-runbook.md#verification-and-upgrade-procedure)
at an idle boundary; a restart alone cannot update baked code.

1. Preserve the temporary `agent-held` on #120, its existing deferred resume
   grant, and every pre-existing label/control. Let #181 finish normally.
2. Confirm no running/retrying worker, rebuild/recreate the controller from the
   delivered main commit using the existing service runbook, preserving volumes,
   workspaces and authentication. No bootstrap or credential copying is needed.
3. Compare deployed `/opt/mtg-lab/WORKFLOW.md` and
   `/opt/mtg-lab/scripts/symphony/before_run.py` to that commit; record image/commit
   identity. Run the bounded credential-free commands above from `/opt/mtg-lab`
   and the existing isolated `runtime-smoke.sh` on the coordinator host. That
   script uses disposable smoke volumes, not the live service volume.
4. Record deployed verification before claiming runtime policy changed. Remove
   only the coordinator's temporary #120 hold, recheck original controls,
   dependencies and completion evidence, then activate exactly #120 by consuming
   its existing grant atomically. Never manufacture another grant or clear other
   holds/blocks. No product milestone is credited.

This worker has no host mounts or Docker socket and cannot perform deployment.
The issue's completion report must retain this coordinator obligation explicitly.
