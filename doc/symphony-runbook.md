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
`docker/mtg-lab-codex.apparmor` profile persistently under `/etc/apparmor.d` (via sudo, or inside the existing Colima
VM), enables its boot-time AppArmor loader and records its name in ignored `.env`. It permits Codex nested user
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
Docker port forwarding. The controller admits at most three issue workers. Heavy verification/reference
commands share one resource lock; each issue has its own writable build/reference cache.

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
for the Linux candidate. The coordinator explicitly replans/reactivates the paused
task at cutover, retaining previous evidence and diagnostic counters.

## Continuous delivery and real blockers

Up to three independent implementation issues run at a time; separate candidate
review uses an additional short-lived Codex process. Dependency, pause, readiness
and hold checks still apply to every issue. Full torture, Cargo and reference/Maven
commands run under the shared `heavy` lock. Workers describe resource waits in
their progress notes, allowing other workers to read/edit without overlapping
memory-heavy verification. Parent-workpad updates and successor reservations use
a separate `handoff` lock around one bounded re-fetch/update/confirm script.
Neither lock weakens or skips required checks. Authorized work continues through review,
CI, exact-main verification and handoff across sessions. There is no total issue
elapsed-time or lifetime dispatch-attempt ceiling. After three repair/review
cycles, record an evidence-based diagnosis and revised approach; continue when
a viable in-scope step exists. Independent testability determines atomic scope.

The hook retains `.symphony-attempts.json` (`started`, `attempts`, and any extra
fields) as diagnostic history, incrementing attempts after execution checks.
Age/count never comments, blocks, or removes readiness. Malformed diagnostics
fail visibly without rewriting the original file or changing authorization;
coordinator repair must preserve that evidence and all existing controls.
Healthy continuation requires no counter deletion, archival or manual reset.

The pinned Symphony v0.0.3 controller returns after `max_turns: 12` and schedules
a continuation check after one second on normal worker completion. It rechecks
active state and routing/readiness before further execution. Abnormal worker
exits use exponential backoff starting at 10 seconds and capped at 300 seconds;
tracker lookup failures retain retry state. These retries have no lifetime
attempt ceiling. Transient service or usage unavailability waits/retries through
that behavior, without credential workarounds or readiness removal. An actual
input-required response, unavailable access, essential decision, irreparable
failure or genuine scope change remains visible for resolution.

Turn and stall timeouts remain 10 minutes, reviewer subprocess timeout 15 minutes,
and CI timeout 30 minutes (including image construction). These bound individual
operations; they are not total delivery deadlines. Account/provider limits still
apply; no hard dollar budget is configured. The
[credential-free acceptance checks](evidence/continuous-delivery/README.md)
exercise packaged controller continuation/retry decisions and hook regressions.
They do not establish recovery from every process crash or provider outage.

To resume after resolving a genuine blocker, the coordinator records its resolution,
removes only the resolved `agent-blocked`/`agent-held` control, then explicitly
reactivates the intended issue under normal dependency and program checks. Preserve
other holds, the branch, workpad and counters. A deferred resume grant can be
consumed only by the existing bounded handoff, after all controls pass.

## RFC 0002 program: operator workflow

The RFC delivery program uses upstream Symphony's normal GitHub dispatch and
three workers. Each worker hands off to the next dependency-ready issue; there is no
additional scheduler or Symphony fork. The committed
[program manifest](programs/rfc-0002.json) fixes the parent issue, ordered task
allowlist, dependencies, assigned requirements, and authorized milestones. The
[requirements inventory](programs/rfc-0002-requirements.json) preserves the RFC
blocks; implementation summaries cannot silently narrow their acceptance scope.
The parent is tracking-only and never receives `agent-ready`.

Your role is to read the parent's `Program workpad` for completed requirements,
evidence, current work, and blockers. Agents implement bounded tasks, run an
independent review, merge through required CI, verify main CI, update the parent,
and enqueue at most one eligible successor before closing the current task,
provided fewer than three other tasks are already ready/running. Reservations and
shared workpad updates are serialized; existing workers keep their ownership.
The current task's verified completion evidence permits this handoff before
closure, so Symphony cannot terminate the worker halfway through scheduling its
successor. Retries reuse the recorded successor and existing labels. A gate task
checks the whole milestone and commits an acceptance report; passing individual
PRs alone does not complete a milestone.

Operations [#59](https://github.com/pabloxrl/mtg-lab/issues/59) authorizes the
existing M0–M5 program. Each stage still waits for its preceding gate's reviewed
acceptance report, protected merge and exact-main CI. A successful gate queues
one eligible successor through the normal bounded handoff; routine milestone
transitions no longer wait for a separate coordinator activation. Authorization
is not completion, and workers cannot add tasks or expand scope. New corrective
work/registration still needs reviewed coordinator operations. Pause/hold controls,
removed ready labels, failed dependencies and per-operation limits remain binding.
Essential product decisions or missing access remain blockers; independent eligible
work may continue. No human PR review is required.

Controls:

- **Pause the program:** put `program-paused` on the parent. Workers check it
  before work, push, merge, and handoff. On observing the pause, the worker
  records it, adds `agent-held`, clears `agent-running`, and removes `agent-ready`
  last, preventing repeated dispatch. Stop the service for immediate shutdown;
  a label cannot undo an operation already in flight. After clearing the parent
  pause, the coordinator explicitly removes the intended task's hold and
  reactivates it after normal checks. Clearing the
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
  then explicitly reactivate it under the controls above; retain its diagnostics.
  Do not mark a task complete to unblock its dependents.

Completed dependencies require a completed issue plus its workpad's acceptance,
independent review, merged commit, and successful exact-commit main CI evidence.
Malformed/cyclic manifests, missing evidence, and unavailable APIs fail closed.
Only allowlisted tasks in authorized milestones can be enqueued. Workers do not
create an unbounded backlog or expand their own scope; an oversized task is
reported for coordinator replanning. A blocked task does not freeze independent
work, but its dependents and the milestone gate remain ineligible.

Normal session completion and controller-observed worker failure retry the still-open
ready issue. Restart can rediscover ready issues and reuse persistent workspaces.
After an operator force-closes a task or an arbitrary hard process failure bypasses
handoff, the queue can become idle; automatic recovery is not guaranteed. Ask the
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
  unverified limitation; parallel admission smoke checks do not establish arbitrary
post-merge crash recovery. The operator explicitly authorized bounded concurrency
in [operations #244](https://github.com/pabloxrl/mtg-lab/issues/244).
- Game correctness, reference bridges, and MVP delivery remain separate work.

## Five-minute activity summary

Open <http://localhost:4318/>. The “What’s happening” card above the live dashboard
shows each worker's own short explanation: current work, why it matters, the
milestone goal, the wider project goal, the next check and any reported blocker.
It loads immediately, refreshes every five minutes while the page is open, and
refreshes when you return to the tab. No second model session or additional AI
usage is incurred. Original live metrics continue updating independently.

Workers use `scripts/symphony/report_progress.py` to write an ignored, atomic
`.agent-artifacts/operator-summary.json` in their issue workspace at startup,
meaningful transitions and every five minutes during active work. Long-running
tools can delay a note. The card displays both when the agent wrote it and when
the dashboard checked; notes older than five minutes are explicitly marked old.
A missing, malformed, future-dated or previous-session report never appears as a
current update. Each task is validated against its own session start; one worker's
progress never appears on another worker's card. Open `agent-ready` issues waiting
for a slot are shown as queued. The queue is read from the repository with the
existing GitHub credential; unavailable or truncated queue data is stated explicitly.
Open `agent-blocked` and `agent-held` issues also appear, even without `agent-ready`.
GitHub task controls and controller worker state are separate: a running worker
remains visible during a label transition, and each issue appears only once.
Ready-label counts remain separate from blocked/held counts (which can overlap).
An empty ready queue does not imply no blocked work. Each control source reports
available, unavailable (unknown count), or truncated (a lower-bound count).
Each label read is limited to 200 displayed numeric issue identities, with a
three-second timeout; the three reads run concurrently. No dispatch labels change.
For stopped controlled tasks, a safe existing note may appear as historical prose,
with its own timestamp and age; it may predate the control and is not a current
reason. Missing notes direct the operator to the issue rather than implying no
blocker. Running notes still require the current session start. Controller states
and GitHub controls take precedence over old prose. The
card is an explanation, not proof of milestone completion; GitHub workpads,
reviews and exact-main CI remain authoritative.

The container now runs the unchanged pinned Symphony binary on internal 4318,
a local read-only summary endpoint on 4319, and nginx on 4317. Compose maps host
`127.0.0.1:4318` to the nginx frontend on 4317. Only that frontend is published.
Nginx adds the card assets to the HTML and preserves existing API and WebSocket
routes. The card stays outside Phoenix's managed DOM so live updates cannot erase
it. All components run as the existing non-root user; logs/temp files use `/tmp`.
Node is included in the runtime image only to execute the browser-script tests.
The summary reader uses only allowlisted fields from the worker's explicit note,
not transcripts, prompts, reasoning, raw commands or credentials. It never executes
report text or grants work. Direct JSON is at `/operator-summary.json`.

For deployment, rebuild the image and recreate the container at an idle worker
boundary; a restart alone does not load new baked code. A coordinator can hold
pending, not-yet-dispatched tasks while the active task finishes, recording exactly
which holds it added. Preserve pre-existing holds/blocks. Confirm no running or
retrying session before recreation, verify the new card/API/WebSocket path, remove
only the temporary holds, then queue one eligible task under the normal controls.
Do not restart a working agent merely to update the dashboard. `runtime-smoke.sh`
exercises the real proxy and a WebSocket ping/pong with invalid dispatch credentials
in an isolated volume, plus browser-script behavior and the Python summary tests
in the full torture suite.

## Parallel operation and workspace transfers

All implementation tasks execute through Symphony. The concurrency limit is three;
a larger ready queue waits for capacity and stays visible on the dashboard. The
worker handoff keeps a bounded supply of eligible tasks and never grants itself
permission to expand the program. Operators may seed an explicitly reviewed set
of independent ready tasks; readiness is not proof that a dependency is complete.

Managed workers receive `MTG_REFERENCE_CACHE=~/.cache/xmage-GH-N` and a checkout-local
`CARGO_TARGET_DIR`. Prepare/copy only verified reference build inputs into each
cache; never share a mutable XMage/Maven source tree or a Cargo target across tasks.
The container still has four CPUs and 7 GiB RAM. Run heavy commands with:

```sh
python3 "$SYMPHONY_CONTROL_ROOT/scripts/symphony/resource_lock.py" heavy -- ./scripts/torture.sh
```

Use the same wrapper for reference runners and other Cargo/Maven commands. A queued
command waits for the foreground owner; it does not count as a passed check. Keep
foreground jobs attached and wait for all children: deliberately detached processes
that close the inherited lock descriptor are outside the lock's protection. The
`handoff` lock uses the same CLI for a bounded script containing current control
rechecks and GitHub mutations; never hold it across model turns or CI waits.

To migrate an external lane, first hold it and obtain its owner's stopped-work
handoff. Preserve its complete source, uncommitted changes, workpad, diagnostics
and evidence. Make a standalone clone of a host linked worktree before importing
with `import-workspace.py`; a host `.git` pointer cannot be used inside Docker.
Copy via stdin without a host mount or credentials, and refuse an existing target.
Retain the source checkout. Reconcile branch/base and files in the destination,
record the transfer, remove only migration-owned holds and explicitly apply
`agent-ready` after normal dependency/program checks. A bare `agent-running`
label from the external owner is replaced by actual controller ownership; never
run the external lane and Symphony worker at the same time.
