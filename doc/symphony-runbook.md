# Symphony operator runbook

Setup date: 2026-09-26. This runbook describes the GitHub Issues installation for
`pabloxrl/mtg-lab`. [Agentic operations](agentic-operations.md) explains the policy.

## Installed components

| Component | Pin / location |
| --- | --- |
| Symphony | Upstream v0.0.3, commit `1c0fb6c8e8ef9031a2c861e62af5f9e66cee39cb` |
| Mac executable | `~/.local/share/mtg-lab-symphony/bin/symphony-v0.0.3-macos_arm64` |
| SHA-256 | `b85d78b25cd5cacff92424416f6a3af7cafee5d675f56b5cd26d22d601c2026d` |
| Codex | CLI 0.157.1, existing ChatGPT authentication |
| Worker model | `gpt-6-astra`, medium reasoning, explicit app-server config overrides |
| Rust | 1.98.1, including rustfmt and clippy |
| Workflow | Root `WORKFLOW.md` |
| Host | Current Mac; service runs while the user session and machine are available |
| Dashboard | `http://127.0.0.1:4318` (local only) |
| Workspaces and logs | `~/.local/share/mtg-lab-symphony/` |

The binary checksum is verified against the upstream release. No Symphony source
fork or custom scheduler is used. The checkout at `~/repos/symphony` is available
for inspection; the service runs the pinned binary, not that checkout's main.

The service retrieves the existing GitHub CLI credential from the host keychain.
No token is committed or placed in the launchd plist. Symphony strips its tracker
token from the Codex child environment. The app-server launcher disables all
personal/company MCP servers in the host config at startup. Git credentials still use the
host's GitHub CLI integration. This is a trusted personal-machine pilot, not a
container security boundary or a repository-scoped service identity.

## Start, stop, and inspect

From the repository:

```bash
./scripts/symphony/service.sh status
./scripts/symphony/service.sh stop
./scripts/symphony/service.sh start
./scripts/symphony/service.sh restart
```

The launchd job is `com.mtg-lab.symphony`. Its plist is at
`~/Library/LaunchAgents/com.mtg-lab.symphony.plist`. It starts at login and restarts
after a crash. `stop` unloads it for the current login session; for a persistent
disable use `launchctl disable gui/$(id -u)/com.mtg-lab.symphony`, and use `enable`
with the same target before starting again. Sleep/logout interrupts availability.
Do not run a second copy against the same repository and workspace directory.

```bash
curl --fail http://127.0.0.1:4318/api/v1/state
tail -n 80 ~/.local/share/mtg-lab-symphony/logs/service.stderr.log
```

Symphony logs and the dashboard expose operational details; the dashboard binds
only to loopback. For continuous remote operation, install the corresponding
pinned Linux release and supply dedicated host authentication; remote hosting
has not been provisioned by this Mac setup.

## Submit and control work

1. Open an issue using **Problem for agents**. State the desired outcome,
   acceptance criteria, and relevant constraints/RFC sections.
2. Add `agent-ready` to start. Only a maintainer should apply that label.
3. Read the issue's `Agent workpad` comment for progress. `agent-running` indicates
   execution; `agent-blocked` means the issue needs a decision or intervention.
4. Agents create a branch and PR, run checks and a separate review, and enable
   squash auto-merge. They close the issue only after the merge commit's CI passes.

Remove `agent-ready` to pause. Close as not planned to cancel. Both take effect
on reconciliation rather than instantly; stop the service for immediate shutdown.
Canceling does not undo a merged change. Request a revert in a new issue.

No human PR approval is part of the workflow. Required GitHub checks enforce
integration. The review step is a workflow requirement with recorded evidence;
it is not an independent GitHub required status attestation. The worker currently
uses the owner's identity, so administrative policy changes remain possible for
that identity and are forbidden by instructions rather than credential scoping.

## Limits and resuming a blocked issue

One implementation issue runs at a time. Review uses an additional short-lived
Codex process. Policy allows three repair/review cycles and a 90-minute issue work
window. The before-run hook pauses re-dispatch after five attempts or expiry of
that window. It does not interrupt an already running turn. Silence timeouts are
10 minutes, the reviewer subprocess timeout is 15 minutes, and CI times out after
15 minutes. `max_turns: 12` is not a total run or spending limit. No hard dollar
budget is configured; usage is subject to the account's limits.

To resume after resolving a blocker, stop the service first, preserve any desired
evidence, remove that issue's ignored `.symphony-attempts.json` from its `GH-N`
workspace, remove `agent-blocked`, add `agent-ready`, then start the service.
Do not delete the branch or workpad: workers use them to avoid duplicate delivery.
Do not add `agent-ready` while editing the counter with the service running.

Dependencies are recorded in issue text and checked by agents. This GitHub adapter
does not implement a full dependency DAG scheduler. A parent paused for child work
must be explicitly reactivated after children finish; the workpad must identify it.

## Verification and upgrade procedure

`./scripts/verify.sh` runs local Markdown link checks, formatting, strict clippy,
and workspace tests. The initial empty crate is scaffolding, not evidence of game
correctness. The first agent task adds a fixture comparator and real behavior tests.

Changes to CI, review scripts, or workflow policy are separate operations tasks.
Feature workers must not weaken them. Upgrade Symphony by choosing an upstream
release, verifying its checksum, reviewing configuration changes, stopping the
service, updating the executable pin, and exercising a bounded task before
resuming ordinary work.

## Validation record

- Native executable checksum: passed during installation.
- Rust workspace formatting, clippy, and compilation: passed during bootstrap.
- GitHub required checks, live dispatch, automatic integration, restart recovery,
  and post-merge recovery: record actual outcomes below as rollout proceeds.
- Game correctness, reference bridges, and MVP delivery remain separate work.
