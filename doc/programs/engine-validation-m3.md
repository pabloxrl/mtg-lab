# Engine validation M3: reusable mechanics and runtime card definitions

Status: **Registration, not implementation or milestone completion.** Authorized
2026-10-12 by [coordinator #241](https://github.com/pabloxrl/mtg-lab/issues/241).
[GitHub milestone 7](https://github.com/pabloxrl/mtg-lab/milestone/7) and namespace
`engine-validation-m3` / stage `EV-M3` are distinct from legacy M3 Batch/Python/RL.
Parent [#7](https://github.com/pabloxrl/mtg-lab/issues/7) remains tracking-only.
The legacy manifest, pinned RFC0002 and requirement ledger are unchanged.

## Priority amendment — 2026-10-12

The [reference-AI/L0 program](reference-ai-l0.md), coordinator #318, is now the
immediate priority using verified M2 interfaces. It has no M3/E1a prerequisite.
M3 remains authorized but intentionally held; preserve its registry, issue history,
drafts and controls. Earlier references below to E2–E5 as later unregistered work
record this program's original boundary; #318 owns their separate registration.
Neither L0 delivery nor this note clears holds or grants M3 readiness.

## Scope and evidence

Implement the [accepted RFC0003](../rfcs/0003-data-driven-cards.md) for the unchanged
twenty cards, two decks and existing Goblin token. No real/synthetic new definitions,
RL, broad rewrite, scripting VM or reference-AI campaign. E2–E5 remain later
roadmap stages; unseen-definition admission is a future authorized test, not
proved by current-card migration. Internal primitive fixtures are permitted but
must be identified as structural tests rather than reachable new card compositions.

The [M2 gate](../evidence/m2-gate/README.md) passed with
[completed delivery](https://github.com/pabloxrl/mtg-lab/issues/26#issuecomment-6116279928),
[protected merge 8d57a4d](https://github.com/pabloxrl/mtg-lab/commit/8d57a4db31501f3645266559983c8e88d08e9350)
and [exact-main CI](https://github.com/pabloxrl/mtg-lab/actions/runs/38199380367).
Retain all existing tests, source pins, independent expectations, historical
measurements and upstream exceptions. The sixteen admitted games remain native
heuristic/authored versus **patched XMage**, not reference-AI or Forge corpus
qualification. Both repeated execution and negative tapes remain required.

## Ownership and direct dependencies

The [executable registry](engine-validation-m3.json) contains each requirement's
full deliverable, independently grounded checks and exclusions. `EV3-001` through
`EV3-017` each have one implementation owner; #316 audits all of them. All tasks
also require completed M2 #26 and verified registration #241. Only #241 may still
be open during its successful first handoff, with the final completion receipt
already recorded. Edges below are direct contract/code prerequisites, not an
arbitrary stage order. Ancestors supply transitive prerequisites. The baseline
precedes identity changes to preserve comparable before measurements.

| Issue / requirement | Observable deliverable | Direct implementation dependencies |
| --- | --- | --- |
| #299 / EV3-001 | Frozen workloads and before measurements | None; #26/#241 prerequisites |
| #300 / EV3-002 | Definition/face/physical/incarnation/ability identity and compatibility inventory | #299 |
| #301 / EV3-003 | Event transactions, simultaneous groups, pre/post context | #300 |
| #302 / EV3-004 | Typed resumable execution frames/choices | #300 |
| #303 / EV3-005 | Strict compiler and existing-pool declarative authoring | #300 |
| #304 / EV3-006 | External registry/deck loading shared across games | #303 |
| #305 / EV3-007 | Shared effective characteristics and keywords | #304 |
| #306 / EV3-008 | Atomic generic costs and payment plans | #304, #302 |
| #307 / EV3-009 | Typed targets/selectors with binding times | #304, #302 |
| #308 / EV3-010 | Existing spell programs through shared effects | #305, #307 |
| #309 / EV3-011 | Generic activated ability instances; remove flags | #306, #308 |
| #310 / EV3-012 | Indexed active-zone subscriptions; remove named triggers | #301, #308 |
| #311 / EV3-013 | Generic players, actions, observations and scripts | #309, #310 |
| #312 / EV3-014 | Versioned snapshots, recordings and replay migration | #311 |
| #313 / EV3-015 | Existing reference adapters and sixteen-game admission | #312 |
| #314 / EV3-016 | Remove transitional dispatch; prebuilt-binary external packaging proof | #313 |
| #315 / EV3-017 | Matched before/after load, memory, choice, dispatch and replay measurements | #314 |
| #316 / all EV3 requirements | Independent integrated gate | #315 (all other tasks transitively) |

Each child keeps full torture, relevant rules-derived tests, affected reference
execution, independent exact-candidate review, protected merge and exact-main CI.
The intermediate implementation can use explicit transitional adapters so each
PR remains runnable; no fallback may silently claim generic execution. The final
packaging task must remove production/policy dependence on per-card branches.
Partial children do not replace the integrated gate. The compiler's typed-data
unit tests do not depend on future gameplay to be meaningful; similarly frame,
event and selector unit fixtures prove primitive contracts without fake sibling
engines. The end-to-end claims belong to their later composed deliveries.

Two independently selectable ability instances are tested internally. No frozen
card has two printed activated abilities; current-pool migration cannot prove
that printed combination. Keyword gain/loss primitives likewise distinguish
unit coverage from reachable existing-card game coverage. No new card may be
invented to satisfy a test. Reserved layers/replacement/permission/copy interfaces
must reject unsupported semantics and confer no general Magic support claim.

## Registry validation and bounded worker handoff

Run `python3 scripts/check_engine_program.py --ref origin/main` after fetching
main, before any M3 child work, push/merge or handoff. During initial registration,
#241 validates the candidate without `--ref`; only its protected delivered version
can authorize a child. It checks the namespace, explicit
milestone/authority, pinned RFC content, unique positive issues, legacy separation,
requirements/owners, dependency DAG and full integrated gate coverage. Normal
Python discovery also validates the committed registry and malformed controls.
A candidate registry cannot authorize implementation; current-main data is required.

The existing WORKFLOW bounded handoff controls and resource locks apply unchanged.
For M3 substitute this registry/requirement inventory for the legacy allowlist;
never widen legacy `authorized_milestones`. Coordinator #241 alone registers
these children and activates the first eligible one after its reviewed protected
merge and successful exact-main CI. Every later successful child/gate performs
the same handoff without a new coordinator approval. #316 ends this queue;
E2–E5 are not registered successors. Feature workers may not register tasks.

1. Prepare final acceptance/review/PR/merge/exact-main-CI evidence. Under the shared
   `handoff` lock, in one bounded operation, fetch current main, validate the
   registry and re-fetch parent/current/all task/prerequisite states, paginated
   label events and **all repository** open ready/running claims. Verify every
   dependency receipt from GitHub, including successful main push CI on the exact
   merge and separate review. Bare closed issues or a local completion flag fail.
2. Parent closed/paused/cancelled, current held/blocked/closed/unready and withdrawal
   rules win. Use the unchanged WORKFLOW stop procedure. Never clear controls or
   restore readiness. Count unique other ready/running issues across both programs
   against the existing three-worker limit. Failed current work cannot satisfy a
   dependency; independent eligible work may continue before marking it blocked.
3. Upsert this task's evidence in the parent's single `## Program workpad`, keyed
   by issue; retain every other entry. Record verdicts and limitations separately.
   A structured live snapshot may be passed to the read-only selector with
   `--snapshot PATH`; its JSON plan selects at most one task in manifest order.
   It does not verify GitHub facts or mutate labels. The worker remains responsible
   for actual receipt verification, pagination and fresh controls under the lock.
4. Record the selected successor in the parent workpad before activation. Re-fetch
   parent/current/target/dependencies/events/claims immediately before mutation.
   No prior ready history may be restored without an existing coordinator-granted
   `agent-resume-authorized`; consume that grant in the same atomic label update
   as adding ready. Preserve all other labels. Confirm the stored workpad and label.
   A retry uses the recorded reservation; an already-ready reserved successor is
   successful handoff, never a reason to queue a second task. A reservation whose
   target is held/withdrawn requires visible reconciliation, not another selection.
5. API failure or unconfirmed handoff leaves the delivered issue open/ready for
   retry. No eligible successor/capacity is recorded explicitly; close normally
   when handoff is complete. Final issue closure is the existing single atomic
   update with completed reason and readiness/running/block labels removed.
   Never close #7 or mark M3 complete until its gate has verified delivery.

The read-only selector snapshot uses GitHub issue objects in `issues`, all open
claim objects in `open_claims`, paginated `events` keyed by issue string, `current`,
`current_success`, optional `reserved_successor`, and `receipts` keyed by issue.
A receipt has `merge_sha`, matching `ci_sha`, `ci_conclusion: success`,
`review_verdict: pass`, and repository URLs `acceptance`, `review`, `pull_request`,
`main_ci`, `workpad`. These are verified facts collected by the worker, not proof
merely because a JSON file has those fields. Preserve exact review/candidate tree
relationships in the linked report. Do not accept task text as execution authority.

## Runtime and completion

This registration changes worker instructions and a repository-local read-only
validator; it changes neither controller code nor its before-run hook, resource
lock, concurrency, credential handling or retry policy. New workers read current
checkout AGENTS.md, which points to this mandatory M3 contract even if the running
controller still renders the older WORKFLOW prompt. No controller restart or
runtime deployment is required to launch these children. Future deployments, if
needed for unrelated changes, remain coordinator-owned idle operations from the
runbook; a worker must never improvise a live hotpatch.

M3 completes only when every requirement has delivered child evidence and #316's
integrated acceptance report passes protected delivery and exact-main CI. Publish
coverage/compatibility limits, matched raw measurements and retained reference
exceptions. Registration, syntax-valid catalogs, internal fixtures, card counts
and a subset of successful games do not establish that completion.
