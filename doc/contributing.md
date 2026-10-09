# Contributing

## Working with the agents

As the person driving the project, describe the problem, desired outcome and
important constraints in a GitHub issue. For the existing MVP, the scope and
implementation issues are already recorded in the
[parent program](https://github.com/pabloxrl/mtg-lab/issues/7).

The coordinator plans dependencies and activates authorized work. Symphony workers
implement it, test it, obtain a separate agent review, merge through protected
checks, and record evidence in each issue's workpad. Routine PR review and merging
do not require you. Essential product decisions and unavailable access may still
need your input. Normal milestone transitions use automatic bounded handoff.
Authorized work continues across sessions without a total elapsed-time or
lifetime dispatch-count stop. Three repair/review cycles trigger diagnosis and
replanning; real blockers and deliberate holds still need resolution. One-worker
concurrency, per-operation timeouts and bounded retry backoff remain in force.
A healthy idle service does not mean the whole program is finished. The
[continuous-delivery checks and deployment limits](evidence/continuous-delivery/README.md)
distinguish tested controller continuation from coordinator-owned image deployment;
arbitrary process termination during handoff is not guaranteed to recover.

If you operate the agent service, use the [Symphony runbook](symphony-runbook.md)
for authenticated setup, start/stop, persistent storage and recovery. From an
already configured checkout:

```sh
./scripts/symphony/service.sh status
curl --fail http://127.0.0.1:4318/api/v1/state
```

The local status endpoint shows currently running work. The parent issue and
individual workpads explain completed milestones, blockers and next actions.
See [agentic operations](agentic-operations.md) for the full driver guide.

## Keeping documentation accurate

Agents must assess README impact on every delivery. Update it in the same PR when
usable commands, prerequisites, supported behavior, architecture, limitations or
verified milestone status change. Explain a no-change decision in the issue
workpad or PR when the README is unaffected. Reviewers check the README against
the actual implementation and evidence, including commands they can reproduce.

Promote a feature only when its required evidence exists; keep planned work labeled
as planned. Maintain detailed supported behavior in the [capability reference](capabilities.md),
examples in [getting started](getting-started.md), and module details in
[architecture](architecture.md). Keep the root README a concise entry point
linking to these guides. Milestone report PRs update the
[roadmap stage table](roadmap.md#delivery-stages), with completion conditional
on successful delivery; failed audits retain an incomplete stage. Link durable
reports rather than copying transient worker counts or queue status. Keep setup
instructions aligned with executable scripts, preserve limitations, and use the
live program issue for current execution status.

### Following agent progress

The local [Symphony dashboard](http://localhost:4318/) includes a “What’s happening”
card that refreshes every five minutes. The working agent explains its current
work, why it matters for the milestone and project, what comes next and any blocker.
The card shows the note's age and says when an update is missing or old. It reuses
worker notes without another AI session; existing live metrics stay available.
See the [operator runbook](symphony-runbook.md#five-minute-activity-summary)
for the Docker deployment and reporting contract.
