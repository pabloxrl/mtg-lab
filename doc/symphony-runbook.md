# Symphony operator runbook

Setup date: 2026-09-26. This runbook describes the GitHub Issues installation for
`pabloxrl/mtg-lab`. [Agentic operations](agentic-operations.md) explains the policy.

## Docker runtime

Operations issue #47 replaces the native macOS pilot with Docker Compose. The
controller, Codex app-server and builds all run in one non-root Linux container.
The image supports linux/arm64 and linux/amd64. Core pins live in
`docker/Dockerfile` and `docker/install-tools.sh`: Rust 1.98.1, Maven 3.9.11,
Temurin Java 21 (exact distribution pinned by image digest), Symphony v0.0.3 and
Codex 0.157.1. Ubuntu Python/Git/gh and system packages are resolved at image build;
rebuilding can update those packages. Preserve the built image digest when moving
an exact tested deployment. This is not a claim of byte-reproducible apt builds.

Prerequisite: Docker Engine plus Compose v2. On this Mac the existing Colima VM
provides Docker (`colima start --cpu 4 --memory 8`); Linux can use Docker Engine
directly. Do not install Java or Rust on the host for this project.

```bash
docker compose build
python3 scripts/symphony/docker-bootstrap.py
./scripts/symphony/service.sh start
```

On AppArmor-enabled Ubuntu hosts, bootstrap loads the named
`docker/mtg-lab-codex.apparmor` profile (via sudo, or inside the existing Colima
VM) and records its name in ignored `.env`. It permits Codex nested user
namespaces without changing global kernel restrictions. Other Docker hosts use
`apparmor=unconfined`. This profile does not add AppArmor filesystem restrictions;
Docker's mounts, non-root UID, read-only root and Codex bubblewrap enforce those.
`docker/seccomp.json` retains Docker syscall filtering with namespace setup calls
allowed for bubblewrap. No host capabilities or privileged mode are granted.
`/tmp` is executable because Symphony's packaged runtime and build/test binaries
need it. The runtime has no path to the host filesystem or Docker daemon.

Bootstrap transfers the existing GitHub CLI credential and only Codex's auth cache
over stdin into the private `mtg-lab_agent-home` volume. It does not copy host
Codex settings, MCP connectors or other home files. Credentials never enter build
arguments, images, committed files or Compose environment values. A missing local
Codex cache requires `codex login` first; alternatively authenticate inside the
container using `codex login --device-auth`. See [official authentication guidance](https://developers.openai.com/codex/auth/).
Repeated bootstrap preserves refreshed container Codex credentials. Reauthenticate
inside the volume if they expire; do not repeatedly overwrite refreshed tokens
with stale copies from the Mac.

The image has a read-only root filesystem, drops Linux capabilities, runs as UID
1001 and has no Docker socket or host filesystem mounts. `/tmp` is disposable;
workspaces, logs, authentication and build caches under `/home/agent` survive
container recreation. Runtime limits are four CPUs, 7 GiB RAM and 1024 processes.
Only localhost port 4318 is published. The container binds 0.0.0.0 internally for
Docker port forwarding. The current single-worker configuration is retained.

Agents still have repository credentials and share a trusted user/volume; this
is not isolation between tasks or a credential vault. Codex's workspace-write
sandbox and built-in auto-review remain enabled. Do not mount the Docker socket,
use privileged containers, or add host directories to repair a build. Missing
system packages require a reviewed image update by the coordinator.

CI and local validation use `./scripts/verify-docker.sh`, which builds the same
`toolchain` stage, mounts source read-only and puts outputs in the container.
CI also runs a credential-free controller and sandbox smoke check in an isolated
Compose project with disposable volumes. The runtime stage adds the committed controller/workflow; rebuild it after
operations changes. Ordinary feature work uses fresh clones of current main.

## Start, stop, and inspect

From the repository:

```bash
./scripts/symphony/service.sh status
./scripts/symphony/service.sh stop
./scripts/symphony/service.sh start
./scripts/symphony/service.sh restart
```

`start` uses Compose, `stop` preserves the volume, and `restart` reconnects the
controller. Docker's restart policy resumes the container when the daemon starts.
The Mac still must remain awake; Compose does not start Colima after a reboot.
For continuous operation move the image and private volume to an always-on Linux
host. Never start the former launchd service alongside Compose. Native `run.sh`
now refuses delivery; the old launchd job is disabled during migration.

Never use `docker compose down -v` unless deliberately deleting all workspaces,
caches and authentication. Back up the volume before host migration. Stop the
controller before backup/restore; do not run two controllers for this repository.

```bash
curl --fail http://127.0.0.1:4318/api/v1/state
docker compose logs --tail 80 symphony
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
   Completion closes the issue and clears execution labels in one REST update;
   clearing `agent-ready` first can stop the worker before it closes the issue.

Remove `agent-ready` to pause. Close as not planned to cancel. Both take effect
on reconciliation rather than instantly; stop the service for immediate shutdown.
Canceling does not undo a merged change. Request a revert in a new issue.

No human PR approval is part of the workflow. Required GitHub checks enforce
integration. The review step is a workflow requirement with recorded evidence;
it is not an independent GitHub required status attestation. The worker currently
uses the owner's identity, so administrative policy changes remain possible for
that identity and are forbidden by instructions rather than credential scoping.

## Importing the stopped pilot

Pause the program and stop/disable the old launchd service before importing.
The importer refuses an existing destination and preserves the native clone:

```bash
python3 scripts/symphony/import-workspace.py ~/.local/share/mtg-lab-symphony/workspaces/GH-13
```

Build outputs are excluded because macOS artifacts cannot be reused as Linux
executables. Keep original review reports as history; rerun verification/review
for the Linux candidate. The coordinator explicitly replans the paused task and
resets its time window once at cutover, retaining the previous evidence.

## Limits and resuming a blocked issue

One implementation issue runs at a time. Review uses an additional short-lived
Codex process. Policy allows three repair/review cycles and a 90-minute issue work
window. The before-run hook pauses re-dispatch after five attempts or expiry of
that window. It does not interrupt an already running turn. Silence timeouts are
10 minutes, the reviewer subprocess timeout is 15 minutes, and CI times out after
30 minutes (including image construction). `max_turns: 12` is not a total run or spending limit. No hard dollar
budget is configured; usage is subject to the account's limits.

To resume after resolving a blocker, stop the service first, preserve any desired
evidence, remove that issue's ignored `.symphony-attempts.json` from its `GH-N`
workspace, remove `agent-blocked`, add `agent-ready`, then start the service.
Do not delete the branch or workpad: workers use them to avoid duplicate delivery.
Do not add `agent-ready` while editing the counter with the service running.

## RFC 0002 program: operator workflow

The RFC delivery program uses upstream Symphony's normal GitHub dispatch and
one worker. The worker hands off to the next dependency-ready issue; there is no
additional scheduler or Symphony fork. The committed
[program manifest](programs/rfc-0002.json) fixes the parent issue, ordered task
allowlist, dependencies, assigned requirements, and authorized milestones. The
[requirements inventory](programs/rfc-0002-requirements.json) preserves the RFC
blocks; implementation summaries cannot silently narrow their acceptance scope.
The parent is tracking-only and never receives `agent-ready`.

Your role is to read the parent's `Program workpad` for completed requirements,
evidence, current work, and blockers. Agents implement bounded tasks, run an
independent review, merge through required CI, verify main CI, update the parent,
and enqueue at most one eligible successor before closing the current task.
The current task's verified completion evidence permits this handoff before
closure, so Symphony cannot terminate the worker halfway through scheduling its
successor. Retries reuse the recorded successor and existing labels. A gate task
checks the whole milestone and commits an acceptance report; passing individual
PRs alone does not complete a milestone.

The **initial rollout authorizes M0 for automatic queue progression**. M1–M5
remain tracked backlog until activated. After M0's gate passes, the worker records
the result and stops at that boundary. The coordinator may activate subsequent
stages through reviewed operations changes under your existing instruction to
deliver the program, without routine approval from you. Feature workers cannot
expand their own scope or reinterpret issue comments as authorization. You do
not review implementation PRs. Essential product decisions or access
failures are recorded as blockers; independent eligible work may continue.

Controls:

- **Pause the program:** put `program-paused` on the parent. Workers check it
  before work, push, merge, and handoff. On observing the pause, the worker
  records it, adds `agent-held`, clears `agent-running`, and removes `agent-ready`
  last, preventing repeated dispatch. Stop the service for immediate shutdown;
  a label cannot undo an operation already in flight. After clearing the parent
  pause, the coordinator explicitly removes the intended task's hold and
  reactivates it, following the counter-reset procedure if needed. Clearing the
  parent label alone never restores dispatch authorization.
- **Hold a task:** add `agent-held` and remove `agent-ready`. The handoff never
  removes a hold. Even without `agent-held`, a previous removal of `agent-ready`
  prevents automatic re-enqueue; workers inspect issue label history. Remove
  the hold and add `agent-ready` explicitly when resuming.
- **Cancel:** close a task as not planned, or close the parent/add
  `program-cancelled` for the whole program. A worker observing program
  cancellation records the stop and holds/de-queues its current task just as
  for a program pause, without reporting it completed. Canceled dependencies remain
  unsatisfied; agents cannot count them as delivered or silently skip them.
- **Resolve a blocker:** supply the decision/access information in the issue,
  then follow the bounded-counter reset procedure above and reactivate it.
  Do not mark a task complete to unblock its dependents.

Completed dependencies require a completed issue plus its workpad's acceptance,
independent review, merged commit, and successful exact-commit main CI evidence.
Malformed/cyclic manifests, missing evidence, and unavailable APIs fail closed.
Only allowlisted tasks in authorized milestones can be enqueued. Workers do not
create an unbounded backlog or expand their own scope; an oversized task is
reported for coordinator replanning. A blocked task does not freeze independent
work, but its dependents and the milestone gate remain ineligible.

Handoff normally resumes from the still-open delivering issue after a crash.
After an operator force-closes a task, a hard process failure bypasses the handoff,
or the before-run retry limit is exhausted, the queue can become idle. Ask the
coordinator to reconcile the manifest against GitHub evidence and activate the
next eligible issue. This is agent-managed progression with bounded recovery,
not a continuously running dependency scheduler. Initial setup also reconciles
existing delivered evidence instead of redispatching completed tasks.

## Verification and upgrade procedure

`./scripts/verify.sh` runs local Markdown link checks, the pinned RFC coverage and
dependency validator with its negative tests, formatting, strict clippy, and
workspace tests. The empty core crate is scaffolding, not evidence of game
correctness. The delivered fixture comparator adds two unit tests and six
integration tests against synthetic checkpoints; see its
[usage and schema](fixture-comparator.md).

Changes to CI, review scripts, or workflow policy are separate operations tasks.
Feature workers must not weaken them. Upgrade Symphony by choosing an upstream
release, verifying its checksum, reviewing configuration changes, stopping the
service, updating the executable pin, and exercising a bounded task before
resuming ordinary work.

## Validation record

- Native executable checksum: passed during installation.
- Rust workspace formatting, clippy, and compilation: passed during bootstrap.
- Main is protected by required `verify` checks, with admin enforcement and no
  required human reviews. Auto-merge is enabled.
- [Seeded CI failure PR #2](https://github.com/pabloxrl/mtg-lab/pull/2): the
  intentionally uncompilable candidate failed `verify`; GitHub kept the PR
  BLOCKED with auto-merge requested. Auto-merge was disabled and the PR closed
  without merging. Its temporary branch and local worktree were removed.
- [Runtime compatibility PR #3](https://github.com/pabloxrl/mtg-lab/pull/3): separate
  Codex review passed, protected integration succeeded, and merge commit
  `ce480e1a9264b213c706dc29227d441eaa527321` passed
  [main CI](https://github.com/pabloxrl/mtg-lab/actions/runs/36265004212).
- Live dispatch: Symphony picked up issue #1 through `agent-ready`. Initial
  app-server argument and Git sandbox mismatches were exposed and fixed; the
  latter was resolved using built-in auto-review. No CI or protection bypass.
- Independent code review: the first comparator candidate was rejected for
  accepting JSON shapes outside its documented schema. The worker added failing
  regression cases, fixed the parser, passed all eight tests, and obtained a
  fresh passing review before publishing.
- Restart recovery: stopped the service with eight in-progress task files,
  retained the checkout, and started it again. The worker reused `agent/issue-1`,
  the existing scaffold and fixtures, and the existing workpad.
- [First delivery PR #5](https://github.com/pabloxrl/mtg-lab/pull/5), for
  [issue #1](https://github.com/pabloxrl/mtg-lab/issues/1): the worker implemented,
  reviewed, repaired, pushed, and merged the comparator without human PR approval.
  [PR CI](https://github.com/pabloxrl/mtg-lab/actions/runs/36265525763) and
  [main CI](https://github.com/pabloxrl/mtg-lab/actions/runs/36265584943) passed;
  merge commit `a50fcde0586d3f2f0a2c21b83da55eefb6d93496`.
- Post-merge regression recovery has not been fault-injected. The recovery
  instructions are configured, but a verified revert/repair drill remains a
  readiness item before increasing concurrency or claiming unattended recovery.
- Game correctness, reference bridges, and MVP delivery remain separate work.
