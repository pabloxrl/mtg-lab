# Agentic development operations

Status: **Adoption plan, not a deployed system.** Updated 2026-09-26.

## Decision

Adopt the existing OpenAI Symphony Elixir implementation, use Codex for coding work, Linear for task intake and progress, and GitHub checks and auto-merge for integration. The project driver describes problems and sets priorities. Agents own planning, implementation, verification, review, and delivery. Routine plan approval and human PR review are not part of the intended workflow.

This replaces the custom autocoder platform proposal. We will configure and operate existing software rather than build a scheduler, agent framework, task database, or dashboard. Project-specific engineering belongs in the Magic engine and its verifier.

Symphony has documented operational use at OpenAI, but its public implementation is an engineering preview. Its supplied workflow includes human approval before merging. Our unattended integration policy is an adaptation that must be validated here; it is not a proven default or a guarantee that arbitrary requests will succeed. See the [Symphony repository](https://github.com/openai/symphony), [operational account](https://openai.com/index/open-source-codex-orchestration-symphony/), and [upstream workflow](https://github.com/openai/symphony/blob/main/elixir/WORKFLOW.md).

## Responsibilities and sources of truth

| Component | Responsibility |
| --- | --- |
| Project driver | Describe desired outcomes, prioritize work, resolve essential product ambiguity, and supply required access |
| Linear | Requests, dependencies, execution status, decisions, and completion reports |
| Symphony | Dispatch and supervise coding runs using the existing implementation |
| Codex workers | Investigate, plan, implement, run checks, address review findings, and maintain task records |
| Repository `WORKFLOW.md` | Versioned execution instructions and runtime configuration |
| Repository `AGENTS.md` | Concise navigation, conventions, and links to authoritative requirements and commands |
| GitHub | Code history, PR audit trail, required checks, and automatic integration |
| Engine verifier | Executable evidence that behavior meets the task and engine contracts |

[RFC 0001](rfcs/0001-project-charter.md) defines the enduring product principles and requirements. [RFC 0002](rfcs/0002-first-mvp.md) defines the first MVP's implementation boundary and acceptance criteria; scoped tasks derive from that delivery plan. A task may refine its implementation but cannot silently discard the charter's requirements. A change in product scope is recorded explicitly, with affected requirements and tasks identified.

The root workflow file, agent instructions, CI jobs, and runtime deployment described here still need to be created. This document does not activate automation.

## Adoption boundary

Use a pinned upstream Symphony revision on a dedicated off-machine worker, with the supported runtime and toolchain for that revision. Start with one concurrent implementation task. Increase concurrency only after dependency handling and integration have been exercised successfully. Kubernetes and a custom orchestration service are not prerequisites.

Use upstream workspace handling, retries, logs, and supported status facilities. Configure the Linear project, eligible states, workspace setup, execution limits, and Codex access. Pin the deployed revision and document upgrades rather than tracking upstream changes automatically.

Adapt the existing workflow for this repository:

- Replace the routine human-review handoff with agent review and required executable checks.
- Let agents create and execute smaller tasks within a submitted objective. Unrelated discoveries go to the backlog.
- Give each task a persistent work record with its plan, assumptions, evidence, and current obstacle.
- Configure GitHub required checks and auto-merge without requiring the driver to approve PRs.
- Define bounded repair attempts, infrastructure retries, task timeouts, and spending limits before enabling unattended execution.

These are repository policies to configure and test, not claims that Symphony supplies all of them automatically. Its scheduler runs agents; task updates, review behavior, and delivery depend on workflow instructions and available tools. Consult the [service specification](https://github.com/openai/symphony/blob/main/SPEC.md) and [setup guide](https://github.com/openai/symphony/blob/main/elixir/README.md) for the pinned version.

## Delivery workflow

1. **Accept the problem.** The driver places a request in `Todo`. The agent reads the request, RFC, repository, and related decisions. It records acceptance criteria and reasonable assumptions, then starts without a plan-approval pause.
2. **Plan the work.** The agent creates small implementation tasks and explicit dependencies when the problem is too large for one change. The parent issue retains ownership of the complete outcome. Child completion alone does not prove the parent is done.
3. **Establish verification.** Before implementing behavior, author or select focused acceptance cases grounded in requirements and rules. For a bug, reproduce the failure. For a refactor, establish behavior-preservation checks. For performance work, establish a comparable baseline.
4. **Implement and repair.** Work in an isolated workspace. Run focused checks during development and the required integration suite before delivery. Diagnose failures and repair the implementation without weakening acceptance criteria.
5. **Review independently.** Use a separate agent context to examine the change, its requirements, and evidence. Findings should identify concrete defects, reproductions, missing checks, or contract violations. Resolve findings and rerun affected checks. A favorable model opinion alone does not establish correctness.
6. **Integrate.** Open a PR as an audit artifact. Address CI and review feedback, synchronize with current `main`, and enable auto-merge when all required conditions are satisfied. Any change to the candidate or its integration base invalidates affected prior evidence.
7. **Verify completion.** Run the configured post-merge checks. Report delivered behavior, requirement coverage, evidence, and unresolved limitations. Close the parent only when the complete requested outcome has been checked.

GitHub's [auto-merge mechanism](https://docs.github.com/en/pull-requests/how-tos/merge-and-close-pull-requests/automatically-merging-a-pull-request) performs the merge once configured requirements pass. Agents handle the work needed to reach that state. Neither an open PR nor an enabled auto-merge flag means the task is complete.

## Verification for this repository

Adopt the engine RFC's verifier-first strategy. The initial repository contains documentation, so bootstrapping the workspace, test runner, and CI is the first work package.

Required evidence grows with implemented capabilities:

- Formatting, Rust checks, unit tests, and scoped conformance fixtures.
- Rules-referenced expected intermediate states and invalid-action behavior.
- Pinned XMage and Forge comparisons at the coverage required by the milestone.
- Deterministic replay, player privacy, and scalar/batch equivalence as those features arrive.
- Trajectory integrity, adapter tests, and training smoke checks when those interfaces are introduced.
- Deliberately incorrect outcomes that demonstrate the fixture runner and comparator detect faults.

Tests written before implementation can still encode a misunderstood rule. Reference comparisons, independent scenarios, and targeted defect injection complement test-first work. Missing dependencies, skipped mandatory checks, and reference-engine disagreements must remain visible; none counts as a pass.

Agents may add tests and propose verifier changes. They must not delete difficult cases, regenerate expectations from the engine under test, or weaken checks to pass their own change. Changes to acceptance expectations, CI enforcement, or the verifier require a separate, independently reviewed change with evidence under the previous trusted checks. A change must not authorize its own bypass. Enforce integration requirements through GitHub settings and CI as well as workflow instructions.

Keep required acceptance behavior frozen during an implementation attempt. If the criterion itself is wrong, preserve the failed evidence and correct it through the verification workflow before starting a newly evaluated attempt. A held-out suite, if adopted, runs separately from the coding workspace; a read-only file is not hidden.

“Reviewed” in the engine RFC can be fulfilled by independent agent review backed by evidence. It does not introduce a routine human code-review requirement. Unresolved questions about desired product behavior still belong to the driver.

## Failure handling and completion policy

| Situation | Intended response |
| --- | --- |
| Build, test, or actionable review failure | Agent investigates and repairs within configured limits |
| Repeated implementation failure | Replan, split the task, or start a fresh attempt; preserve failed evidence |
| Transient infrastructure failure | Retry with bounded backoff; keep distinct from an incorrect implementation |
| Dependency incomplete | Wait for the dependency while unrelated eligible work continues |
| Required access missing | Request the exact credential or permission action needed |
| Essential product ambiguity | Ask one concrete outcome question with a recommendation; continue independent work |
| Budget exhausted or no verified solution | Mark the task unresolved and summarize attempts; never declare success or quietly shrink scope |
| Post-merge regression | Agent prepares a revert or repair, runs required checks, and restores the last verified behavior; check dependent changes before reverting |

Post-merge recovery is a workflow to configure and demonstrate, not an assumed Symphony feature. The worker needs scoped credentials and a reproducible environment; ordinary code execution must not expose unrelated personal credentials. Use host/service controls and provider limits for resource enforcement where available rather than relying exclusively on prompts.

`Done` means merged, required post-merge checks passed, the requested outcome covered, and a completion report recorded. Release publication is a separate task and must satisfy its delivery plan's release gates against the exact candidate, consistent with RFC 0001. A merged feature does not imply a qualified release.

## Rollout and readiness

1. **Bootstrap:** create the Rust workspace, reproducible setup, initial verifier, and required CI. Establish the Linear project and worker access.
2. **Install:** deploy the pinned Symphony implementation and configure this repository's workflow, review process, required checks, and automatic integration.
3. **Prove one complete delivery:** select a bounded, rule-referenced interaction scenario in a scoped implementation task. Demonstrate correct behavior with independent evidence and an automatically merged change.
4. **Exercise failure paths:** show that a seeded defect blocks integration, an interrupted run recovers without duplicate delivery, and a controlled post-merge regression results in a verified recovery. Confirm essential blockers are surfaced clearly.
5. **Expand:** proceed through the separately agreed delivery plan, then increase task concurrency and scope based on observed results.

Keep the adoption work finite. Success is delivering engine changes autonomously, not expanding the automation platform. Track delivered requirements, escaped defects, unresolved work, recovery time, execution cost, and required driver interventions. Do not use PR count as the primary success metric.

## Your guide: driving the work

### One-time setup

Provide access to the GitHub repository, a Linear project, the worker host, and Codex authentication. Set an operating budget and define the desired delivery scope under RFC 0001's charter. Agents can perform the installation and repository setup using that access. Where a service requires you to authenticate or change an account setting, you receive the specific action needed.

### Start a problem

Create a Linear issue and move it to `Todo`. Describe the result you want; you do not need to prepare implementation tasks or write a detailed RFC for every change.

```text
Problem: Implement the first unattended game simulation slice.

Desired outcome: Two scripted players can complete a game using the
capabilities explicitly listed in this task, and the run produces
structured results and a replay.

Constraints: Follow RFC 0001. No browser, manual game choices, or scope
expansion into additional cards.

Success: The CLI runs without stdin, the required semantic cases pass,
and replay reproduces the final state.
```

Agents derive the plan, break down work, implement, review, verify, and integrate. Putting an objective in `Todo` authorizes those routine steps within its scope. Unrelated new ideas remain in `Backlog` until you choose them.

### Check progress when useful

Read the parent issue in Linear. The proposed board states are:

| State | What it means for you |
| --- | --- |
| `Backlog` | Idea recorded; execution has not been requested |
| `Todo` | Authorized and queued |
| `In Progress` | Agents are planning, implementing, or repairing |
| `Blocked` | A dependency, resource limit, unresolved failure, or required decision prevents progress; the issue explains which |
| `Merging` | Validation and automatic integration are being completed |
| `Done` | Integrated and verified, with a completion report |
| `Canceled` | Stop requested; no further delivery is authorized |

These states must be mapped to the deployed Symphony configuration. A blocked issue states explicitly whether you need to act. You do not need to watch agent sessions, respond to routine status messages, or inspect PR diffs.

### Steer or stop

Comment on the parent issue to clarify the desired outcome or change priority. Agents incorporate the update at the next workflow checkpoint and record any affected acceptance criteria. Material scope changes trigger replanning.

Move an issue to `Canceled` to stop further work. Cancellation is observed through polling, so it is not an instantaneous emergency stop. For an immediate stop, stop the worker service; this procedure must be documented during setup. Canceling does not undo already merged changes—request a revert as a separate outcome.

### Handle an exception

Answer only when the system names a decision or action it cannot resolve: for example, choosing between incompatible product outcomes, restoring access, or extending an exhausted budget. For implementation failures, you receive a diagnosis and the proposed next attempt rather than a request to debug code or review a PR. You may defer or cancel unresolved work.

### Receive the result

The completion report tells you what now works, how to run it, which checks establish that result, and any limitations. PRs, test artifacts, and decisions are linked for optional inspection. No final “please approve this PR” step is expected.

Your recurring work is to choose the next problem, describe the outcome, and adjust direction when needed. Agents carry the engineering work through to verified integration.
