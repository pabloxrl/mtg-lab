# Autocoder: From RFC to Implementation

---

## Problem Statement

We write RFCs that carry the design: the problem, who it affects, what it has to
do, and how it should work. They are detailed enough for someone who knows the
codebase to build from, and never so complete that nothing is left to judgement.
Turning one into working code is manual, and it only starts when an engineer
with enough context to read the document correctly picks it up. Designs now
arrive faster than implementations.

Code generation tools produce a change from that document in minutes. They do
not establish that the change is right. Reviewing generated code closely enough
to trust it costs roughly what writing it would have, so the work moves rather
than disappears.

The repositories we target are open source. A change that is wrong, or that is
right but hard to review, spends maintainer attention we do not control.

---

## Assumptions

- Token budget is not a constraint. Compute may be spent freely on redundancy
  and verification.
- Multiple model families are available through interfaces that permit
  programmatic invocation.
- A Kubernetes cluster is available to run work on, with credentials held as
  cluster secrets.
- Target repositories have a working test suite, a linter, and CI. Repositories
  without these are out of scope — see *What We Are NOT Building*.
- A human reviews and merges. Nothing here merges autonomously.
- Target-repo maintainers are reviewers on the RFC and have approved it before
  any task is generated from it. Nothing arrives at a repository whose owners
  have not already agreed to the design.

---

## Personas

**RFC author.** Writes the design and owns its intent. Knows which parts of the
document are settled and which are still being argued. Is not necessarily the
person who implements it.

**Implementing engineer.** Translates the RFC into code today. Reads the
document, works out what it means for the target repository, and writes the
change. This is the time the system is meant to free, and the person who
inherits its failures.

**Target-repo maintainer.** Owns the repository the change lands in and reviews
what arrives. May be outside our team, and did not ask for the change. Pays the
review cost and carries the code afterwards.

---

## Use Cases

### RFC author

- As an RFC author, I want to review the implementation plan before code is
  written, so that a misread design costs a review rather than a rewrite.
- As an RFC author, I want the implementation to reflect the design as written,
  so that departures from it surface as decisions rather than substitutions.
- As an RFC author, I want to know which parts of the design were not
  implemented, so that gaps are visible instead of silent.
- As an RFC author, I want sections that are still under discussion to be
  excluded, so that unsettled design does not become code.

### Implementing engineer

- As an implementing engineer, I want the mechanical parts of the translation
  handled, so that my time goes to the parts that need judgement.
- As an implementing engineer, I want to know why an attempt failed, so that I
  can tell a hard problem from a broken tool.
- As an implementing engineer, I want to see what was attempted and rejected,
  not only what was produced, so that I can judge whether the result was the
  best available or the only one that survived.
- As an implementing engineer, I want to understand the change well enough to
  defend it in review, so that I am not shipping code I cannot explain.
- As an implementing engineer, I want to start work and walk away from it, so
  that my attention is not what paces it.
- As an implementing engineer, I want a URL that shows where every task is, so
  that checking in costs me a glance rather than reconstructing state.
- As an implementing engineer, I want anything waiting on me shown as waiting on
  me, so that I am not the silent bottleneck.
- As an implementing engineer, I want the work to run somewhere other than my
  laptop, so that starting it does not cost me the machine I work on.
- As an implementing engineer, I want several tasks in flight at once, so that
  throughput is not one at a time.

### Target-repo maintainer

- As a maintainer, I want each change to do one thing, so that I can review it
  in one sitting.
- As a maintainer, I want to know what was verified and what was not, so that I
  know where to spend attention.
- As a maintainer, I want evidence that the change does something, so that I am
  not relying on the claim that it does.
- As a maintainer, I want the change to follow the conventions already in the
  repository, so that review is about behaviour rather than style.
- As a maintainer, I want rejecting a change to cost one action and end there,
  so that declining is cheaper than accepting something I am unsure about.
- As a maintainer, I want to still understand this code in a year, so that
  accepting it does not transfer a maintenance cost I did not agree to.

---

## Requirements

| # | Requirement | Serves | Priority |
|---|---|---|---|
| R1 | No claim about a change's correctness rests on the assessment of the model that produced it | maintainer, engineer | P0 |
| R2 | The implementation intent is reviewable before code exists | RFC author | P0 |
| R3 | Each change addresses one objective | maintainer | P0 |
| R4 | Every change carries evidence of effect that does not depend on the model's account of it | maintainer | P0 |
| R5 | Parts of the design that were not implemented are reported | RFC author | P0 |
| R6 | The mechanisms that judge a change cannot be altered by the process that produces it | maintainer, engineer | P0 |
| R7 | Changes conform to the conventions already present in the target repository | maintainer | P1 |
| R8 | Departures from the RFC are surfaced as decisions rather than applied silently | RFC author | P1 |
| R9 | A failed attempt is diagnosable — the operator can distinguish a hard problem from a broken tool | engineer | P1 |
| R10 | Attempts that were made and rejected are retained and inspectable | engineer | P1 |
| R11 | Rejecting a change costs the maintainer one action and does not recur | maintainer | P1 |
| R12 | Outcomes after merge are tracked, not just outcomes at submission | RFC author, engineer | P1 |
| R13 | Sections of the RFC still under discussion are excluded from scope | RFC author | P2 |
| R14 | A human decides whether a change is merged | maintainer | P0 |
| R15 | Work executes off the engineer's machine | engineer | P0 |
| R16 | Run state is observable from a URL, with no local tooling and no attached session | engineer | P0 |
| R17 | Anything waiting on a human is shown as waiting, naming what is needed | engineer, RFC author | P0 |
| R18 | Tasks and attempts run concurrently, bounded by cluster capacity rather than by one machine | engineer | P1 |
| R19 | Credentials are held by the execution environment, never in images, logs or event streams | engineer | P0 |
| R20 | Acceptance criteria are executable, and are authored before implementation begins | RFC author, maintainer | P0 |
| R21 | Part of the acceptance suite is withheld from the process that implements the task | maintainer | P0 |
| R22 | An acceptance suite that passes before implementation is rejected | RFC author | P1 |
| R23 | Stages exchange files. No stage depends on another's conversation history | RFC author, engineer | P0 |
| R24 | Every requirement, test and task is traceable by id, and gaps at either end are errors | RFC author | P0 |
| R25 | Every requirement cites the RFC text it was derived from | RFC author | P1 |
| R26 | What the design excludes is compiled and enforced, not only stated | maintainer | P0 |
| R27 | Names the design specifies exactly are compiled as an interface surface | maintainer | P1 |
| R28 | A lost analysis dimension is reported rather than absorbed into the total | engineer | P1 |

---

## What good looks like

A maintainer receives a change that does one thing, states what was verified and
how, carries evidence that it has an effect, and touches nothing that would
weaken its own checks. They merge it or close it in under ten minutes. Ninety
days later it has not been reverted.

---

## How the best known solution looks

No published system does this end to end at our scale: 25–39% resolve on
multi-file, specification-driven tasks against roughly 80% on single-issue ones
(A.3). The shape below is assembled from parts that have been measured
separately. A.2 compares what was rejected.

| Property | Evidence |
|---|---|
| The document is not the unit of work; decompose to issue scale | Same model: 72.8% on single-issue, 25% at 2,391-word specs over 20.9 files |
| Check the decomposition with a human before generating | Largest failure category across 1,600+ annotated traces is specification and coordination, not model capability |
| One thin agent, not a hierarchy of them | 100-line scaffold reaches >74% SWE-bench Verified; removing 3 of MetaGPT's 5 agents raised pass@1 62% → 79% |
| Several attempts, independent, across model families | Cross-family ensembles solve 83% more than the best single model; diversity-aware selection realises 95% of it |
| The attempts must not see each other | Answer entropy declines monotonically across debate rounds; independent sampling preserves it |
| Select with something that executes | LLM judges: 7–8pt position bias, up to 30% verbosity bias, 50.4% test-retest consistency. Judging tests is their weakest category |
| Static analysis in the loop, every iteration | Security >40% → 13%, readability >80% → 11%, reliability >50% → 11%, within 10 iterations |
| Cap the repair loop at 3–5 rounds | Plateaus across model scales; assertion errors resist repair and consume the most iterations |
| Make the gates unmodifiable by the producer | 63% of successful resolutions retrieved rather than derived; sealing git history and egress removed 14–21pp of apparent success |
| Anchor test evidence outside the model | Model-written oracles encode actual, not intended, behaviour; coverage and mutation score correlate only weakly on buggy code |
| Write the tests before the implementation exists | The oracle problem is an observation problem — a test authored against a spec with no implementation to look at can only encode intent |
| Withhold part of the test set from the implementer | Passing the visible tests and passing the contract are different claims; the gap between them is where overfitting lives |
| One objective per change, structured rather than long | Agent PRs are multi-objective 39.9% vs 12.2% human; unstructured verbosity correlates with lower merge rates |
| Human on the merge decision; measure after it | 61.4% of AI PRs get no human review; developers measured 19% slower while estimating 20% faster |

The organising principle has a name and a soundness proof. Lemur (ICLR 2024)
pairs a model proposing invariants with a reasoner discharging them, and proves
the hybrid sound because soundness comes from the checker, never the model.
Every row above is that shape at a weaker tier of checker than a theorem prover,
which is what an ordinary repository affords.

---

## Proposed Design

Three phases, matching three commands. **Compile** turns an RFC into a spec and
needs no repository. **Plan** turns that spec plus a repository into an approved
task DAG. **Implement** turns each approved task into a reviewed change. Each
has substages, and each substage is separately invokable, because each writes an
artifact that stands on its own.

The boundary is not cosmetic. Verifying an acceptance suite is red requires a
checkout, so it belongs to `plan`; while suite generation sat inside a
repository-free compile stage, R22 could not be enforced and silently was not.

An RFC is compiled into requirements, the repository is analysed against them,
and the two produce a plan. Once approved, each task is attempted several times
independently, the attempts are filtered by checks they cannot influence, and
one survivor is submitted for human review.

```
COMPILE
───────
RFC on main, minus text under open suggestion
        │
        ▼
   COMPILE   ──▶ requirements.json    ids, criterion, priority   (R5, R13, R20)
        │         exclusions.json      what not to build → gate 6  (R26)
        │         interfaces.json      names spelled exactly       (R27)
        │              │
        │              ▼
        │         TEST ──▶ tests/     per requirement, red, split (R20-R22)
        │
        ▼
PLAN
────
   ANALYZE   ──▶ analysis.json        5 lenses, independent, union (R2)
        │
        ▼
   VERIFY    ──▶ red suites only      vacuous tests dropped       (R22)
        │
        ▼
   PLAN      ──▶ plan.json            task DAG, traceability      (R2, R5)
        │
        ▼
   human approval of spec, analysis and plan                      (R2)
        │
IMPLEMENT
─────────
        ▼
   GENERATE   N independent attempts, mixed model
              families, no shared state                           (R8)
        │
        ▼
   GATE       ordered checks, cheap first;
              protected paths unwritable                    (R1, R4, R6, R7)
        │
        ▼
   SELECT     rank survivors by gate evidence                     (R1, R10)
        │
        ▼
   SUBMIT     one task, one pull request                          (R3, R11)
        │
        ▼
   human review ──▶ merge ──▶ measure at 30 and 90 days      (R12, R14)
```

Every arrow is a file. A stage reads the artifacts before it and writes one of
its own; none inherits a conversation. Any artifact can be inspected, diffed,
corrected by hand, or regenerated without re-running the stages above it.

Three properties carry the design. **Nothing the generating model asserts is
treated as evidence** — every gate is a program with an exit code, and the
model cannot edit any of them. **Attempts never see each other** — the value of
running several comes from their errors being uncorrelated, which conversation
between them would destroy. **Stages hand off documents, not context** — a
contract that can be read and corrected, rather than a history that can only be
re-run.

Where a task cannot be completed, the pipeline reports that and stops. Producing
nothing is an expected outcome, not a failure of the system.

Everything runs in a Kubernetes cluster, one pod per attempt, and reports to a
web page. Starting work does not occupy the engineer's machine and does not
require them to stay attached to it (§10).

### Form factor

A command that starts work and hands back a URL. Everything after that is a web
page.

```
$ autocoder compile environment-onboarding-ux
  10 requirements, 14 exclusions, 39 interfaces → spec/     a3f81c02

$ autocoder plan a3f81c02 -C ~/repos/NeMo-Gym
  decomposing... 9 tasks, 2 sections unimplemented
  plan posted for approval → https://autocoder/runs/47

$ 
```

The engineer approves the plan on that page and closes the tab. Attempts run in
the cluster. The page shows where each task is and, above everything else, what
is waiting on a person.

| Piece | What it is | Where it runs |
|---|---|---|
| `autocoder` | A CLI. Starts a run, prints a URL, exits | The engineer's shell, or CI |
| Controller | Owns run state, schedules attempts, collects events, serves the page | One long-lived pod |
| Attempt | A coding agent plus the gate chain, scoped to one task | An ephemeral pod, N per task |
| Gate chain | The checks in §6, packaged so they run anywhere | Inside attempts, and standalone |
| Page | Read-only view of runs, tasks and attempts | Served by the controller |

The gate chain is packaged to run on its own, outside a pipeline run. A human
can point it at a branch and get the same verdict an attempt got, which is what
makes the gates auditable rather than internal machinery.

It is not an IDE plugin, not a chat interface, and not a bot that watches
repositories for work to do. The input is an approved RFC and nothing else.

---

## Relationship to existing work

**Internally.** `frontier-eval-rfcs/scripts/build_from_rfc.py` already turns a
`build:`-prefixed merge request into a merge request in a target repository. A
single model plans, implements, writes its own tests, reviews its own plan and
code through role-played personas, and commits.

This is a different system, not an evolution of that one. The property the whole
design rests on — that no signal deciding anything comes from the model that
produced the change — is one that script lacks by construction, and it is not
reachable by adding checks to it. What that script does get right is isolation:
token stripping, kernel-level network blocking, and a scoped tool surface. That
part informed §10 and is worth carrying over.

**Externally.** Devin, Google Jules, Codex cloud and Cursor background agents all
take work asynchronously in the cloud and return a pull request, which is the
same execution model proposed here. Two things differ. Their input is an issue or
a prompt; ours is a design document that the target repository's maintainers have
already reviewed and approved. And they do not withhold any part of the test set
from the model doing the work. The novelty claimed here is in what goes in and
what verifies the output, not in running agents remotely.

---

## Design Details

### 1. Compile the RFC into a spec (R2, R5, R13, R20)

RFCs come from the **RFC portal** at `rfc.frontier-evals.nvidia.com`, which is
where they are written, reviewed and approved. An RFC is named, not located: a
name, a portal URL and a path all resolve to the same document.

Input is the RFC as it stands on `main`, **excluding any text covered by an open
suggestion merge request**. Open suggestions already mark exactly the passages
under discussion, so R13 needs no new field and no status marker in the
document. Suggestion branches live in the source repository and a rendered page
cannot carry them, so resolution prefers a local clone and falls back to
fetching the portal. A fetched copy is correct but **has no scope derivation**,
which is reported rather than hidden.

Compilation reads the RFC's **own requirements table** when it has one. These
documents carry a table with an id, a statement, a success criterion and a
priority; that table is the contract the author wrote and reviewers approved.
Re-deriving it from prose produces more requirements, worse attributed, and
numbered differently from the document everyone read. Section-by-section
inference is the fallback for documents without one.

The success criterion matters most: it is the acceptance criterion, stated by
whoever knew what the requirement was for, and the acceptance suite is
generated against it.

Compilation produces three artifacts.

**`spec/requirements.json`.** Each requirement carries:

| Field | Content |
|---|---|
| `id` | Stable. Everything downstream references it |
| `statement` | One testable assertion |
| `source` | The RFC heading and quoted line it came from |
| `kind` | `behaviour`, `interface`, `constraint`, or `non-functional` |
| `verifiable` | Whether it can be checked by running code, and if not, why |

**`spec/exclusions.json`.** What the document says not to build, what it has
not decided, and what it defers. This is the only spec content that can *gate*
rather than advise: gate 6 fails an attempt that implements one. Each exclusion
carries markers — identifiers or phrases whose appearance in added source means
it was built anyway. A marker must be specific enough to fail a build on, so
generic words are rejected: gating on `runtime` would fail almost any change,
and a gate that fires constantly gets turned off.

**`spec/interfaces.json`.** The names the document spells exactly — commands,
manifest fields, types, endpoints. Wrong names are the most common reason a
change is rejected on convention rather than behaviour, and unlike prose they
can be checked.

The spec is the contract. Every later stage reads files and writes files;
nothing inherits a conversation. That is the part of multi-agent design that
survived the evidence — structured artifacts and fixed task graphs held up
where free conversation did not (A.2).

**This stage has no oracle.** Nothing external can establish that the spec
captures the RFC faithfully, which makes it the weakest link in the chain and
the reason it is separated out rather than folded into planning. Four checks
make it auditable instead of trusted, and each reports to a person rather than
deciding:

1. **Citation.** Every requirement quotes the sentence that states it, and the
   quote is checked to appear in the document. A fabricated citation defeats
   the only mechanical check this stage has.
2. **Sections that produced nothing.** A section that compiles to nothing is
   either out of scope or a miss, and the difference is a judgement someone has
   to make.
3. **Coverage, the reverse direction.** Obligations in the document that no
   requirement covers. Check 2 catches a section skipped whole; nothing caught
   an obligation missed inside a section that yielded something else. This
   matters most when the RFC states its own requirements, because that table is
   adopted verbatim and no prose is mined at all — without it the recall of the
   whole spec rests on the author having tabulated everything.
4. **Independent review, by every available family, unioned**: requirements
   their quote does not support, statements carrying two obligations,
   ambiguities, and pairs that conflict. Asking the model that made a mistake
   to find it is the correlation this design avoids everywhere else, and this
   is the stage that most needs the second opinion.

   **Every lens on every family, unioned.** Four configurations were run
   against the same ten requirements and the counts were 3, 9, 5 and 7 — the
   spread between two identical configurations is as wide as the spread between
   different ones. Asking one call for all four defect kinds returns whichever
   it notices; unioning two such calls lost to a single lucky one; four distinct
   lenses with one call each matched the best singularity recall and then found
   a third as many ambiguities, because distinctness had *replaced* replication
   instead of joining it. N-fold inspection is both (A.6): distinct questions,
   each asked by more than one independent reviewer.

   **The counts do not measure spec quality and are not reported as if they
   did.** Across those four runs, thirteen distinct concerns exist in one
   ten-requirement table and no single run found more than nine. A clean review
   is not evidence of a clean spec.

   The `support` lens is skipped when every requirement quotes itself, which is
   the case for any RFC that states its own requirements: the table cell is both
   the statement and the citation, so asking whether the quote supports the
   statement asks whether a sentence entails itself. It returned zero on every
   run because it could not do otherwise.

   Findings are unioned, never voted on. Deliberation between reviewers
   measured *negative* against pooling their individual findings (A.6), and
   finding a defect is recall-bound: a missed one is invisible, an extra one
   costs a person a line to dismiss.

Checks 3 and 4 report; they never drop. A false positive that removed a
requirement would lose scope silently, which is the failure the whole design
exists to prevent. Concerns are recorded on the requirement, travel with the
spec, and are read at approval.

The spec is a file, and it is the handover rather than a report of one. `plan`
reads `spec/` back from disk before it runs, so the spec can be diffed against
the next RFC revision and corrected by hand without re-running anything
upstream, and the correction is what the next phase sees. Test bodies live only
under `spec/tests/`; `requirements.json` refers to them by path and never
repeats their content, because two copies of a test are two specs that can
disagree. Deleting a generated test deletes it from the suite.

### 2. Acceptance suites, per requirement (R20, R21, R22)

Tests are generated **per requirement, not per task**. Requirements are stable;
task decomposition is a scheduling decision that will change. Attaching tests
to tasks means re-planning discards them.

The rules are unchanged from the design they replace. Suites are written before
any implementation exists, which is what defeats the oracle problem — a test
authored when there is nothing to observe can only encode the criteria. This is
not a theoretical benefit: given an implementation to look at, LLM-generated
oracles capture the *actual* behaviour rather than the intended one, the same
failure as the random-testing tools they were meant to improve on (A.4).

Each must be **red** against the unmodified repository or it is vacuous and
rejected (R22). **Red is classified rather than counted**, because failing and
failing to run are not the same thing. Suites are written without a checkout by
construction, so they name imports that may not resolve, and pytest exits
non-zero either way — the best measured pass rate for repository-level
specification generation is 20.2%, with a third of failures caused by exactly
this kind of grounding error (A.4). Four outcomes are distinguished:

| Outcome | Meaning | Kept | Counts as verified |
|---|---|---|---|
| assertion | ran and failed | yes | **yes** |
| absent | cannot import what it tests, which is not built yet | yes | no |
| unresolved | cannot import something else — wrong path, or a library the target lacks | no | no |
| unparsable | not valid Python | no | no |

Only an assertion failure shows a test discriminates. A suite red solely
because the feature is missing is the ordinary test-first case and is kept, but
it has demonstrated nothing and is reported as such rather than counted.

Each is **split** into a visible set and a held-out set by independent passes
(R21), and the overlap between the two is **measured**: both halves come from
one model reading one statement, so their independence is an assumption until
something checks it. A held-out half that repeats most of the visible
assertions is flagged, since gate 10 would then be re-running what the attempt
already saw. All are immutable and join the protected paths in gate 3.

A third pass writes **properties** rather than examples: round trips,
invariants, metamorphic relations, error contracts. Across a corpus of 426
programs the average property-based test kills roughly **fifty times as many
mutants** as the average unit test (A.4). The same study reports why that is
not free — constant-equality is the most common property form and among the
least effective — so a generated file asserting only fixed values is rejected
rather than counted. Properties run with the visible half, since the attempt is
told they exist; special-casing the example inputs fails them.

A task inherits the suites of the requirements it satisfies. Re-planning is
then free.

### 3. Analyze the repository (R2)

**History is read first, deterministically.** Every strongest predictor of
change difficulty is a process metric drawn from `git log` — relative churn at
89% discrimination, change entropy, minor-contributor counts at ρ 0.86–0.93,
high-confidence co-change (A.5). None of it needs a parser, language support or
a model call. On a repository of 800-odd commits it costs under a second, and
it finds couplings static analysis structurally cannot: config to code, test to
code, across languages.

That bundle goes to **every** analyst rather than being a lens of its own.
Deterministic anchoring roughly halves run-to-run variance (A.5), which matters
more here than accuracy does: a union of noisy independent findings is mostly
noise.

Five agents then read the repository against the spec, independently, in mixed
model families, with no communication:

| Lens | Asks |
|---|---|
| localization | which files and symbols each requirement will have to change |
| prior art | what already exists that a requirement should call, and the conventions that code follows |
| architecture | layering, boundaries, extension points, and where the public surface lives |
| integration | what elsewhere assumes the current behaviour and would break silently |
| verification | how this area is tested, where the fixtures are, what has no coverage |

Five, not eight. Inspection converges on four to five reviewers; repeated
passes scale log-linearly, so doubling buys what the previous doubling bought
at twice the cost; and duplicate or ambiguous roles are the largest documented
multi-agent failure category (A.6). Coupling and resistance were removed
because the history bundle answers them deterministically and better. The
freed budget bought **integration** — the question none of the eight asked,
and the one category in the classical taxonomy of what programmers ask that
was missing.

*Prior art* asks what to call, not where duplication is. Clones are measurably
*less* fault-prone than non-cloned code (A.5); the supported question is
retrieval, not repetition.

A lens that fails is a dimension lost, and union aggregation means the run
survives it — which is exactly why the loss would go unnoticed. An analyst
retries on the other model family before giving up, and dimensions that are
lost anyway are reported rather than absorbed into a healthy-looking total.

**Findings are aggregated by union, not by vote.** This is the opposite rule
from persona review, and the difference is the shape of the problem.
Localization is recall-bound: general agents reach 0.64–0.68 file-level hit
rate against 0.08–0.14 for lexical retrieval, and patchers tolerate irrelevant
context but degrade sharply when the evidence they need is absent (A.6).
Intersection or majority optimises precision on a problem where precision is
not the constraint. Findings are deduplicated by `(file, symbol)` and a
location named by one analyst alone is kept, not discarded.

Output is `spec/analysis.json`.

### 4. Plan (R2, R5)

One agent reads `requirements.json` and `analysis.json` — and nothing else, no
conversation history — and emits `plan.json`: a **task DAG**.

Each task states its objective, the requirement IDs it satisfies, the files it
is expected to touch, and its dependencies. The graph is explicit rather than
an ordered list, so tasks with no path between them run concurrently and
`blocked on dependency` means something precise.

Tasks are sized to where models are known to succeed. Resolve rates fall from
roughly 73% to 25% for the same model when a specification reaches a few
thousand words across twenty files (A.1), so the document is never the unit of
work.

**Traceability is checked here and costs nothing, because the IDs already
exist.** A requirement no task satisfies, or a task that satisfies no
requirement, is an error surfaced at approval. Combined with the coverage
report from §1, the two ends are closed: RFC sections that produced no
requirement, and requirements that produced no task.

The plan **requires human approval before any code is generated** (R2). What is
approved is the spec, the analysis, and the DAG — each a file, each reviewable
on its own, each replaceable without re-running the others. `implement` reads
`plan.json` back, so editing a task before approving it is a supported way to
correct the plan, and deleting one is a supported way to decline it. Approvals
and attempts belong to the run record rather than the document and survive the
re-read; only the description of the work comes from the file.

### 5. Generation (R8)

Per approved task, N ≥ 3 attempts run concurrently, spanning at least two model
families. Each gets its own git worktree. There is no shared scratchpad, no
message passing, and no consensus step.

Each attempt runs a single loop: locate, edit, run gates, repair. Three
parameters are fixed by the evidence in A.2 rather than left to taste:

- **Location** is by search over the repository, not by an embedding index.
- **Edits** are exact-string replacements with a fuzzy-match fallback. Never
  line-numbered diffs.
- **Repair** stops after five rounds. The measured plateau is three to five (A.2).

An attempt that cannot satisfy an acceptance criterion as written stops and
records why. It does not substitute a different criterion (R8).

### 6. Gates (R1, R4, R6, R7)

Ordered cheapest first. Any failure returns the attempt to repair; failure at
the repair cap ends it.

| # | Gate | Fails on |
|---|---|---|
| 1 | Formatter and linter, using the target repository's configuration | Any violation |
| 2 | Type checker, strict | Any new error |
| 3 | Protected-path check | Any diff against tests, CI, lint or build config |
| 4 | Duplication detector, on the diff | A new duplicated block |
| 5 | Banned constructs | Bare `except`, swallowed errors, `# type: ignore`, `@ts-ignore`, `// nolint`, empty catch |
| 6 | Excluded scope | Anything the spec says not to build |
| 7 | Static application security testing | Any new finding |
| 8 | The repository's existing test suite | Any regression |
| 9 | Visible acceptance tests | Any test that does not fail on the parent commit and pass on the head commit |
| 10 | Held-out acceptance tests | Any failure. Terminal — see below |

**Which tool each gate runs is discovered, not configured.** Gates 1, 2 and 6
try a fixed order of candidates and use the first present, so the target
repository's own linter and its own configuration decide what passes — most
rejections are convention mismatches rather than defects (R7). Tests run under
the repository's own virtualenv where one exists, since ours cannot see its
dependencies. A gate with no tool available reports that it skipped and names
what is missing; it never reports a pass. In the cluster, where the image
controls what is installed, a skip is a failure.

Gate 3 is enforced twice: those paths are read-only to the attempt at the
filesystem layer, and the diff is checked against the base commit regardless.
Weakening a check to pass it is the failure mode this exists to prevent (R6).

Gates 9 and 10 are the only test artifacts treated as evidence (R4). Both were
written before the attempt existed and neither can be edited by it, so no model
certifies its own work.

**Gate 10 does not feed repair.** It runs once, after the attempt reports done,
and a failure ends that attempt rather than returning it to the loop. This is
not severity for its own sake: repairing against held-out failures would leak
the held-out set one failure at a time, and after a few rounds it would be
visible in all but name. An attempt that passes gate 9 and fails gate 10 has
overfitted to the tests it could see, and that is exactly the outcome the split
exists to detect. It is recorded as such (R10).

Coverage and mutation score are not gates and are not reported.

Network egress is denied except to the package registry, and git history is
sealed for the duration of the attempt (§10).

A small **canary suite** of deliberately impossible tasks — where the acceptance
criteria contradict the tests — runs continuously alongside real work. Any pass
is proof the gates are leaking.

### 7. Selection (R1, R10)

Among attempts that clear every gate, ranking is by gate evidence only:

1. Fewest repair rounds consumed
2. Smallest diff
3. Fewest files touched
4. Net line count, lower first

No model ranks the candidates. Every attempt is retained with its gate output
and linked from the submission, including the ones that failed (R10).

If no attempt clears the gates, nothing is submitted and the task is reported as
unconverged with the gate output that stopped each attempt (R9).

### 8. Submission (R3, R11)

One task, one pull request. A task whose diff exceeds the size ceiling is
returned to decomposition rather than submitted.

Commits separate mechanical changes — renames, moves, formatting — from semantic
ones, so a reviewer can skim the first and concentrate on the second.

The description has fixed sections and no others: what changed in one sentence;
the RFC task it implements, linked; what was verified, with the gate 8 output;
what was **not** verified; and disclosure of machine authorship.

A closed pull request is never reopened and the task is never resubmitted (R11).

### 9. Measurement (R12)

Recorded per submission: merge rate, revisions requested by the maintainer, time
to first human response, and reverts or rewrites at 30 and 90 days.

Not recorded as success: pull requests opened, attempts passing gates, coverage.

### 10. Execution and observability (R15–R19)

**Where it runs.** A Kubernetes cluster, one pod per attempt. Pods are ephemeral
and isolated by construction, which subsumes the git worktrees in §5 — attempts
cannot see each other because they do not share a machine. Nothing runs on the
engineer's workstation (R15).

**Concurrency** is two-dimensional: several tasks in flight, and N attempts
within each. The bound is a cluster quota, not a laptop (R18).

**Credentials** are cluster secrets mounted at run time. They are not baked into
images and are redacted from the event stream, which is rendered in a browser
and therefore has to be assumed readable (R19).

**Egress** is a NetworkPolicy allowing the model API and the package registry.
Everything else is denied, including the target repository's upstream host. This
is what makes the §6 history-and-network sealing enforceable rather than
aspirational: on a workstation it is awkward to arrange, in a cluster it is a
policy object.

**Events.** Each attempt emits structured events on stage transitions, gate
results and repair rounds, appended to a per-run log. The page reads that log.
Pods are not queried directly, so an attempt that dies leaves its history intact.

**The page** (R16) shows runs, their tasks, and each task's attempts with current
stage and last gate result. Two things sort above everything else: **what is
blocked on a human, and what it needs** (R17), and what has finished. The rest
is detail available on click. It is a URL with no login-shell dependency and no
local tooling — the engineer's interaction is a glance.

Task states:

| State | Meaning |
|---|---|
| `awaiting plan approval` | Blocked on a human. The task plan needs sign-off before generation |
| `queued` | Approved, waiting on cluster capacity |
| `running` | One or more attempts live; the stage of each is shown |
| `unconverged` | No attempt cleared the gates. Gate output retained per attempt (R9) |
| `submitted` | Pull request open. Blocked on a human |
| `merged` / `declined` | Terminal. `declined` is never retried (R11) |
| `blocked on dependency` | An earlier task in the plan has not merged |

Three of the seven states are blocked on a person. Surfacing that is the point
of the page: the system is designed to be left alone, so its main job when
observed is to say whether it is waiting on you.

---

## Coding standards

Generated code is a permanent contribution to a codebase other people maintain.
Standards are ranked by how hard they are to evade:

**Filesystem permissions > CI exit codes > lint rules > type checks > tests >
prose.**

Anything expressed only in Markdown is advisory. A controlled study over 288
trials found context files (`CLAUDE.md`, `AGENTS.md`) do not measurably move
correctness and never convert a near-miss into a pass. Encode the rule where it
executes.

Beyond the gates in §6:

- **Prefer deletion.** Refactoring collapsed from 21% to 3.8% of changed lines
  between 2022 and 2026. A diff that only adds and never touches an existing
  abstraction is the statistical signature of generated slop. This is why net
  line count is a ranking criterion in §7.
- **Search before generating.** An attempt establishes whether the thing already
  exists before writing it. Duplication is the clearest measurable signature of
  *generated* code — up 81% in a 623-million-change corpus, with copy/paste now
  five times more likely than refactoring. It is not a defect predictor in
  general: clones are measurably *less* fault-prone than non-cloned code (A.5).
  The gate catches slop, not bugs, and should not be argued for as the latter.
- **No TODO, FIXME or placeholder comments**, and no comments that narrate what
  the code does.
- **Architectural constraints are lint rules**, not prose — import boundaries,
  layering, forbidden APIs, file-size and complexity caps.
- **The target repository's configuration wins** over ours, always.

Security is non-negotiable and non-improving: 45% of AI-generated code carries
an OWASP Top 10 vulnerability, 2.74× the human rate, and the figure was flat
across the year to March 2026 with larger models performing no better. One
introduced CVE ends the project's credibility permanently.

---

## What We Are NOT Building

- **Reviewer or critic agents.** A model reviewing code is not a gate. Ungrounded
  self-review degrades output rather than improving it.
- **Agent-to-agent conversation of any kind.** Attempts are isolated by
  construction, and that isolation is the source of their value.
- **An embedding index over the repository.** Search measures better and costs
  less to maintain.
- **Autonomous merge.** A human merges. Always.
- **Volume submission.** Roughly 17 million agent-authored pull requests were
  opened on GitHub in March 2026, and the practitioner estimate is that one in
  ten is legitimate. GitHub shipped a maintainer kill switch in February 2026.
  The marginal unsolicited pull request currently has negative expected value to
  the ecosystem. Ours are not unsolicited — they implement a design their
  maintainers signed off on — but the system must still be willing to submit
  nothing.
- **Targeting repositories without CI, tests and a linter.** Those provide the
  external signal the whole design rests on. Without them there is no gate, and
  per R1 there is nothing.
- **Bot-only review.** 61.4% of AI pull requests receive no recorded human review
  and 71.6% of comments on them come from other agents. We do not add to that.
- **A general-purpose coding agent.** The input is an RFC we wrote.

---

## Alternatives Considered

| Alternative | Why rejected |
|---|---|
| Role-based agent team (planner, coder, reviewer, QA) | Loses to a single agent at matched compute across every controlled 2025–26 study; 4–220× token cost; 70–90% collapse into unanimity |
| Multi-agent debate to reach consensus | Debate conditions each round on the last, collapsing diversity; entropy declines monotonically and the system tunnels onto wrong answers as budget grows. Self-consistency beats it at equal calls |
| LLM-as-judge for selection | 7–8pt position bias, up to 30% verbosity bias, 50.4% test-retest consistency; judging tests is its weakest category |
| Whole RFC in one context, one pull request | Measured resolve rate for 2,391-word specs is 25%; our RFCs are roughly an order of magnitude longer |
| Model writes tests, then code until green | Model-written oracles encode actual, not intended, behaviour; produces green CI over wrong changes |
| One attempt per task, with a bigger model | Forgoes the 83% cross-family headroom, which is the largest measured return available on the generation side |
| Fine-tuning on our RFC corpus | No corpus at this size; harness and verification dominate model choice at this stage |
| Formal specification as the gate (Dafny, Lean, refinement types) | Right principle, wrong tier. SMT-backed verifiers reach 82–96% on Dafny benchmarks, but Lean sits at 27% and Liquid Haskell rarely produces passing proofs — the gap is proof synthesis, the least-improved capability in the stack. Our targets carry no formal specifications. Revisit if one ever does |
| One decomposition step from RFC straight to tasks | No inspectable intermediate, so a misread RFC can only be fixed by re-running everything; and tests attached to tasks are discarded whenever the plan changes |
| Majority vote across repository analysts | Localization is recall-bound, and voting optimises precision. A location found by one analyst is signal, not noise |
| Running on the engineer's workstation | Occupies the machine they work on, caps concurrency at one box, and makes egress restriction awkward to enforce (§10) |

---

## Risks and Mitigations

| Risk | Mitigation |
|---|---|
| Green CI certifies a wrong change | Acceptance suite authored before implementation exists and immutable to it; red-at-approval check; held-out split; canary suite |
| Reward hacking produces a passing but fake diff | Unmodifiable tests and configs enforced twice, sealed history, egress policy, canary suite |
| Attempt satisfies the visible tests without satisfying the contract | Gate 9 held-out set, run once and never fed back into repair |
| Maintainer trust is spent | Changes only ever implement an RFC the maintainers reviewed and approved; one objective per change; disclosure; willingness to submit nothing |
| Duplication and non-refactoring slop accumulates | Search before generating, duplication gate, net line count as a ranking criterion |
| Introduced security vulnerability | SAST on every diff, blocking, no exemptions |
| The spec does not faithfully capture the RFC | The one stage with no oracle. R25 source citations make it spot-checkable, and uncompiled sections are reported. Mitigated, not solved |
| Decomposition is wrong and everything downstream is wasted | R2: a human approves spec, analysis and plan before generation, and each is separately replaceable |
| Succeeds on easy tasks and silently skips hard ones | R5 and R9: unconverged tasks are reported with the gate output that stopped each attempt |
| Cluster credentials leak through the event stream | R19: secrets mounted at run time, redacted from events, which are assumed readable |
| We cannot tell whether the system works | R12 post-merge tracking. Self-assessment on this question is known to err by ~39 points |

---

## Open Questions

1. **Task sizing.** SWE-bench Pro reference patches average 107 LoC across 4.1
   files and resolve at ~60%. Is that the right target, or should a first change
   into an unfamiliar repository be smaller still?
2. **Nothing measures whether any of this helps.** A.4 is a large-N negative
   result on spec-driven development generally. This design differs in emitting
   checkers rather than prose, which is the side of the line the evidence
   favours — but that is an argument, not a measurement, and R12 remains
   unimplemented.
3. **Which model families, and does N = 3 saturate?** The cross-family evidence
   comes from repair benchmarks, not RFC implementation. Worth measuring on our
   own tasks before fixing N.
4. **How to split the acceptance suite.** A random partition is the simple
   option. Generating the two sets by different means — examples visible,
   properties held out — would make gate 9 test a different thing rather than
   more of the same, but not every criterion is expressible as a property.
5. **Where the task plan lives.** The RFC merge request keeps design and
   implementation coupled; an issue in the target repository is where the
   maintainer actually looks.
6. **Run state store.** The page needs durable state with concurrent writers.
   SQLite on NFS is the known-fragile option and should not be repeated.
7. **Which cluster, and what quota.** The design does not depend on the answer,
   but concurrency limits and cost do. An operator cannot get started without
   it: no cluster, namespace or access path is named anywhere.

Findings 1, 3 and 6 were raised by `autocoder review RFC.md`. The claim that
RFCs carry every decision needed to build from was raised there too, and the
Problem Statement has been corrected — Appendix A's 25–39% resolve rate on
specification-scale tasks was evidence against it.

---

## Action Items

1. Land the deterministic gate chain — format, lint, type check, existing test
   suite (gates 1, 2, 7). Smallest change with the largest measured effect.
2. Add protected-path enforcement (gate 3): read-only at the filesystem layer
   plus a diff check against the base commit.
3. ~~Build the spec compiler.~~ Done: reads the RFC's own requirements table
   where one exists, falls back to section inference, and emits requirements,
   exclusions and interfaces.
4. Build the repository analysts and union aggregation.
5. Build the planner and the traceability check, then the approval step.
6. Generate acceptance suites per requirement rather than per task: authoring
   from the statement, the red-at-approval check, and the visible / held-out
   split (gates 8, 9).
7. Stand up cluster execution: one pod per attempt, egress policy, secrets
   mounted at run time.
8. Build the event stream and the page, starting with the blocked-on-a-human
   view.
9. Run several attempts per task across model families, and add selection.
10. Add duplication, banned-construct and SAST gates (4, 5, 6).
11. Stand up the canary suite and wire it as a continuous regression.
12. Instrument post-merge outcome tracking (R12).
13. Validate end to end: run against one completed task from an existing RFC
    where a known-good human implementation exists, and compare.
14. ~~Precompute a git-history bundle and inject it into every lens.~~ Done.
15. ~~Emit invariants as a spec artifact.~~ Done. Mutation scoring of the
    generated suites remains open — it needs a mutation tool present in the
    target repository, which the gate chain cannot assume.
16. ~~Cut the lens set back, and add the missing question.~~ Done: five lenses,
    with `integration` replacing `coupling` and `resistance`.
17. Measure whether the history bundle changes what the lenses find. The claim
    is halved variance; running the same RFC twice with and without it is the
    experiment, and it has not been run.

---

## Appendix A: State of the Art

Evidence base as of August 2026. Numbers are from the cited work; where a figure
is vendor self-reported rather than independently reproduced, it is marked.

### A.1 Summary

**Scaffold complexity lost.** The 2024 argument that a fixed pipeline beats an
agent loop (Agentless) did not win as a system, but its thesis won as a
principle: minimal scaffold, maximal model. `mini-swe-agent` is 100 lines of
Python with bash as its only tool and reaches >74% on SWE-bench Verified,
matching frameworks an order of magnitude larger. A source-code taxonomy of 13
agents found they converge on the same four capabilities — read, search, edit,
execute — regardless of whether those are exposed as one tool or thirty-seven.
Engineering effort pays off around the loop (sandboxing, context compaction,
verification, retry budget) and does not pay off inside it (planner/reviewer/
critic hierarchies).

**Role-based multi-agent systems lose at matched compute.** This is the most
consistent finding of the last eighteen months. Under equal thinking-token
budgets, single agents match or beat multi-agent systems across five
architectures and three model families. Automated multi-agent frameworks
underperform plain self-consistency on SWE-bench Lite while costing up to 18×,
and reach unanimous consensus in 70–90% of cases — they behave as expensive
ensembles rather than as collaborators. Removal-based attribution on MetaGPT
raised pass@1 from 62% to 79% while cutting tokens 38%, by deleting three of its
five agents. The failure taxonomy built from 1,600+ annotated traces (κ=0.88)
finds the largest failure bucket is specification and coordination, not model
capability.

**What does replicate is independence, not conversation.** Parallel independent
sampling with an execution-grounded selector outperforms agents talking to each
other. Debate conditions each round on the previous one, which collapses answer
diversity — measured answer entropy declines monotonically across debate rounds
while independent sampling preserves it, so debate tunnels onto wrong answers as
budget grows. Cross-family ensembles show +83% oracle headroom over the best
single model on Defects4J, with a diversity-aware selector realising 95% of it.
The benefit is decorrelated errors.

**Verification must come from outside the model.** Ungrounded self-correction
degrades performance; execution feedback is the documented exception. Static
analysis in the loop is the highest-measured single intervention in the corpus:
security findings from >40% to 13%, readability violations from >80% to 11%,
reliability warnings from >50% to 11%, within ten iterations. LLM judges are
unreliable in the opposite direction — 7–8 point position bias, up to 30%
verbosity bias, and test-retest consistency as low as 50.4% on identical inputs,
with unit-test judging the most fragile category measured. Coverage and mutation
score correlate only weakly with real bug detection, and uniformly weakly when
the code under test is buggy.

**Our regime is the unsolved one.** Bug-fix-scale benchmarks are saturated; the
top models sit within three points of each other on SWE-bench Verified. Spec-
scale work is not. SWE-EVO, whose specifications average 2,391 words across 20.9
files, resolves at 25% for the best model against 72.8% for the same model on
single-issue tasks. RoadmapBench tops out at 39.1%. SlopCodeBench, where agents
extend their own prior work under an evolving specification, reports no agent
solving any problem end to end, with verbosity rising in 89.8% of trajectories
and quality decaying monotonically. Our RFCs are roughly an order of magnitude
longer than the specifications in the hardest of these benchmarks.

### A.2 Methodologies compared

| Approach | Family | Mechanism | Correctness signal | Measured result | Fit here |
|---|---|---|---|---|---|
| **Agentless** | Single, fixed pipeline | Localize → repair → validate; no LLM control flow | Regression + reproduction tests | 32.0% SWE-bench Lite at $0.70/instance | Pipeline shape is right; too rigid for multi-file design work |
| **mini-swe-agent** | Single, minimal | 100 LOC, bash only, linear history, stateless | Agent-run tests | >74% SWE-bench Verified; now the standard harness for SWE-bench Pro | Strong baseline; no verification of its own output |
| **SWE-agent / OpenHands** | Single, ReAct/CodeAct | Code execution as action space, sandboxed workspace | Sandbox execution | Dominant research substrate | Good substrate; verification still self-reported |
| **Aider** | Single, human-in-loop | PageRank repo map, five edit formats | Lint + test hooks | No published comparative benchmark | Repo map worth borrowing; assumes a human driver |
| **AutoCodeRover / Moatless** | Single, structured | AST-aware structured queries | Test-based | Precision 0.680, highest of localizers measured | Localization technique worth borrowing |
| **MetaGPT** | Multi-agent, role-based | PM/Architect/Engineer/QA over shared message pool; SOP artifacts | Peer agent review | HumanEval 85.9 pass@1 (self-reported); ablation removing 3 agents: 62% → 79% | Rejected. Its own ablation is the argument against it |
| **ChatDev** | Multi-agent, chat chain | CEO/CTO/Programmer/Reviewer dyads, ≤10 rounds | Peer agent review | Toy scale — 144 LoC average per "software" | Rejected. Not evaluated at repository scale |
| **CodeR** | Multi-agent, task graph | Pre-defined graphs, not free conversation | Verifier agent | 28.33% SWE-bench Lite | Task-graph structure aged well; the agent cast did not |
| **MAGIS** | Multi-agent, orchestrated | Manager, Custodian, Developer, QA | QA agent | 13.94% SWE-bench | Rejected |
| **AFlow / ADAS / DyLAN / MAS-Zero** | Multi-agent, auto-designed | LLM-generated orchestration graphs | Varies | Underperform CoT-SC at up to 18× cost; frequently rediscover self-consistency | Rejected. Automated MAS design does not find good architectures |
| **Multi-agent debate** | Sampling, sequential | Rounds conditioned on prior answers | Consensus | Loses to self-consistency at equal calls; answer entropy declines monotonically | Rejected. Actively destroys the diversity it needs |
| **Self-consistency / CoT-SC** | Sampling, parallel | N independent samples, majority vote | Majority | 57.09% SWE-bench Lite (GPT-5), beat every automated MAS tested | Strong. Majority vote is the weak part |
| **Cross-family ensemble** | Sampling, parallel | Different model families, diversity-aware selection | External selector | +83% oracle headroom on Defects4J; 95% realised | Strongest generation-side evidence available |
| **Parallel worktree agents** | Sampling, isolated | N agents in separate git worktrees, no communication | Per-agent gates | Converged on independently by Cursor 3, Grok Build, Antigravity in 2026 | The 2026 isolation primitive |
| **Test-time scaling (RTV/PDR)** | Sampling, staged | Tournament voting over compacted trajectory summaries | Tournament + execution | +6.7pp SWE-bench Verified, +12.2pp Terminal-Bench v2 | Applicable once a base loop exists |
| **Spec Kit / Kiro** | Spec-driven | spec → plan → tasks → code, human gates between | Human review at gates | No controlled evidence found | Workflow shape matches ours; efficacy unmeasured |
| **Constitutional SDD** | Spec-driven | RFC 2119 MUST/SHOULD constraints as executable gates | Spec conformance checks | Case study only, no baseline comparison | Catches architectural drift tests cannot; magnitude unknown |
| **Self-repair loop** | Verification | Iterate against test failures | Test exit codes | Universally positive but plateaus at 3–5 rounds; assertion errors most resistant | Necessary, with a hard cap |
| **Static analysis in loop** | Verification | Linter, type checker, SAST feedback every iteration | Tool exit codes | Security >40%→13%, readability >80%→11%, reliability >50%→11% in 10 iterations | Highest measured return of anything reviewed |
| **LLM-as-judge** | Verification | Model reviews code and selects | The model | Position bias 7–8pp, verbosity bias to 30%, test-retest 50.4% | Rejected as a gate. Unit-test judging is its worst category |
| **Mutation-guided testing (ACH)** | Verification | Generate a specific fault, then a test that kills it | Mutant survival | 73% engineer acceptance across 10,795 classes at Meta | The only test-generation shape with an external anchor |
| **Formal verification (Lemur, Dafny, Verus)** | Verification | Model proposes, solver discharges; soundness proof of the hybrid | SMT solver | Dafny 82–96%; Lean 27%; Liquid Haskell rarely passes | Right principle, wrong tier. Our targets carry no formal specs |

### A.3 What nobody has solved

- **Spec-scale implementation.** The best measured resolve rate on
  multi-file, spec-driven tasks is 25–39%. No published system takes a
  document of our size to a merged change reliably.
- **Quality decay under evolving specifications.** Agent code measured against
  48 maintained human repositories shows 2.2× verbosity and 2.2× structural
  erosion, and the decay is monotonic where human repositories stay flat.
  Quality-aware prompting cuts the initial level by ~34% but does not change
  the slope.
- **Test generation that certifies rather than describes.** LLM-generated
  oracles encode actual rather than intended behaviour, so a buggy
  implementation yields a test asserting the bug.
- **Distinguishing derived from retrieved solutions.** An audit of 731
  trajectories found 63% of successful resolutions retrieved the fix rather
  than deriving it; sealing git history and network egress cost 14–21 points
  of apparent success.
- **Measuring whether any of this helps.** The cleanest randomized study found
  experienced open-source developers 19% slower with AI assistance while
  estimating they had been 20% faster. Its authors have since withdrawn it as
  current and report they can no longer recruit a control group.

### A.4 Spec artifacts: what the evidence supports

Gathered after the compiler was built. Two findings argue against most of what
spec-driven tooling produces, and one argues for an artifact this design does
not yet emit.

| Claim | Source |
|---|---|
| **Spec-driven development shows no defect benefit and the sign is wrong.** 119 repositories, 100,247 PRs, SZZ tracing, within-author fixed effects. Five vendor claims tested, none supported: **+1.4pp defects** (p=0.056), **+5.0pp rework** (p<0.001). Spec *quality* had no effect on rework (p=0.997) | [Hill, SSRN 6515898](https://papers.ssrn.com/sol3/papers.cfm?abstract_id=6515898) |
| **Prose context files are a wash and cost 20% more.** 138 instances, 12 repos, four models: LLM-written −0.5% to −2% success, human-written +4%, both **+20% inference cost** | [arXiv 2602.11988](https://arxiv.org/abs/2602.11988) |
| **Property-based tests kill ~50× as many mutants as unit tests.** 426 Python programs, mutation analysis over 40 projects. Caveat from the same paper: trivial "constant equality" properties are the most common and least effective form | [OOPSLA 2025](https://dl.acm.org/doi/10.1145/3764068) |
| Mutation detection correlates with real-fault detection **independently of coverage**. 32,002 mutants, 144 real faults | [Just et al., FSE 2014](https://homes.cs.washington.edu/~rjust/publ/mutants_real_faults_fse_2014.pdf) |
| Requirement traceability: **24% faster** on maintenance and **50% more correct** solutions, 71 subjects | [Mäder & Egyed, EMSE 2015](https://link.springer.com/article/10.1007/s10664-014-9314-z) |
| Type annotations alone detect **15% of public bugs** — a lower bound, counting only bugs that survived to a public fix | [Gao, Bird & Barr, ICSE 2017](https://earlbarr.com/publications/typestudy.pdf) |
| Contracts in real projects **stay simple and rarely evolve** with the implementation | [Estler et al., FM 2014](https://arxiv.org/pdf/1211.4775) |
| **LLM oracles capture actual behaviour, not intended behaviour** — the same weakness as Randoop and EvoSuite, which LLMs were assumed to escape. 24 open-source Java repositories, oracle classification and generation, several prompts. This is the evidence for generating suites where there is no implementation to read | [arXiv 2410.21136](https://arxiv.org/abs/2410.21136) |
| **Repository-level executable specification generation is mostly unsolved: 20.2% pass rate** for the best model, against 47.0% at function level. 33–34% of repository-level failures are dependency and grounding errors. Models scoring 59–77% on writing the code score 4–21% on specifying it | [CodeSpecBench, arXiv 2604.12268](https://arxiv.org/html/2604.12268v1) |
| LLM-generated postconditions from natural language **discriminate 64 real historical bugs** in industrial Java — evidence that a spec artifact derived from prose can carry real discriminating power | [Endres et al., arXiv 2310.01831](https://arxiv.org/abs/2310.01831) |
| **Requirements smells degrade downstream LLM tasks, but not uniformly.** Significant effect on predicting whether a requirement was implemented; none on tracing to specific lines. Argues for reporting quality defects rather than gating on them | [arXiv 2501.04810](https://arxiv.org/abs/2501.04810) |
| **Non-goals have no empirical support.** Searched for specifically; the basis is project-management advice on scope creep | — |

The through-line: an artifact read by a model as prose measures at or below
zero; an artifact executed by a checker measures positive. Compiling exclusions
into a gate (§6) is the right side of that line, but this document should not
claim more for them than *enforceable* — the underlying idea is unevidenced.

**The invariant gap this opened is now closed:** properties are emitted as a
third pass with the trivial-equality category rejected (§2).

**What the specification-generation evidence changes.** The 20.2% figure is not
an argument against generating suites without a repository — the oracle-drift
result says a suite written *with* one encodes the implementation instead of the
intent, which is worse. It is an argument that a suite produced this way cannot
be assumed to run, and therefore that "red" has to be classified rather than
counted. That is what §2 does, and it is why an unimportable suite no longer
satisfies R22.

### A.5 What predicts change difficulty

Bearing on §3. Every strongest predictor comes from `git log`, needs no parser,
and is language-agnostic. None of the eight lenses read history, which is what
§3's deterministic bundle now supplies to all five of them.

| Claim | Source |
|---|---|
| **Relative churn discriminates fault-prone binaries at 89.0%**; absolute churn is a poor predictor | [Nagappan & Ball, ICSE 2005](https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/icse05churn.pdf) |
| **Change entropy beats prior faults, prior modifications and static complexity**, in that order; 13–42% lower prediction error | [Hassan, ICSE 2009](https://sailresearch.github.io/sail-website/data/pdfs/ICSE2009_PredictingFaultsUsingTheComplexityOfCodeChanges.pdf) |
| **Minor-contributor count correlates with failures at ρ 0.86–0.93**, out-correlating size, churn and complexity; adds **+20 to +46 points** of adjusted R² | [Bird et al., FSE 2011](https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/bird2011dtm.pdf) |
| Co-change coupling, **high-confidence mode: 66–70% precision at 3% coverage**; broad mode 29%. "One can either have precise suggestions or many suggestions, but not both." Catches config↔code and cross-language coupling static analysis cannot | [Zimmermann et al., TSE 2005](https://thomas-zimmermann.com/publications/files/zimmermann-tse-2005.pdf) |
| Architectural debt — co-change **not** explained by structural dependency — took **51–85% of maintenance effort** across 7 Apache projects | [Xiao et al., ICSE 2016](https://www.cs.drexel.edu/~yc349/papers/2016/icse2016-ArchDebt.pdf) |
| Deterministic static anchors: **variance roughly halved**, +3.4pp pass@1, ~10% tokens. "Static structure helps less by making agents smarter and more by making their navigation disciplined and reproducible" | [arXiv 2606.26979](https://arxiv.org/abs/2606.26979) |

**Refuted, and worth naming because they are what most tools lead with:**

| Claim | Source |
|---|---|
| Cyclomatic complexity has a "practically perfect linear relationship" with LOC and **"absolutely no explanatory power of its own"** (~1.2M files) | [Jay et al., JSEA 2009](http://www.scirp.org/jouRNAl/PaperDownload.aspx?paperID=779) |
| After adjusting for size and revision count, **none of 12 code smells** remained a significant driver of maintenance effort | [Sjøberg et al., TSE 2013](https://www.mn.uio.no/ifi/personer/vit/dagsj/sjoberg_etal_code-smells.pdf) |
| **Only 25 of 202 SonarQube Java rules** show even low fault-proneness | [Lenarduzzi et al., SANER 2020](https://researchportal.tuni.fi/en/publications/are-sonarqube-rules-inducing-bugs) |
| **Clones are less defect-prone** than non-cloned code; most bugs have nothing to do with duplication | [Rahman, Bird & Devanbu, MSR 2010](https://cabird.com/pdfs/rahman2010cws.pdf) |

That last row narrows a claim made earlier in this document. Gate 4 rests on
the GitClear corpus, which measures duplication rising in AI-assisted work —
a claim about generated code, not about defect-proneness in general. The gate
stands as a slop signal. It is not a defect predictor.

### A.6 How many independent passes

Bearing on §3, which carried eight lenses when this was gathered and now
carries five. Nobody has measured the marginal value of
the Nth diverse LLM analysis pass, so this is extrapolation from three adjacent
literatures — but they point one way.

| Claim | Source |
|---|---|
| **N-fold inspection**: N independent teams inspect one artifact, findings unioned by a moderator. The direct analogue of this design, from 1990 | [Martin & Tsai, CACM 1990](https://dl.acm.org/doi/abs/10.1145/75577.75587) |
| Inspection converges on **4–5 reviewers as optimal**, with clear diminishing returns beyond | [Porter, Votta & Basili](https://link.springer.com/article/10.1023/A:1009776104355) |
| **Negative meeting gain (~−1%)**: discussion *lost* defects relative to pooling individual findings. Direct evidence against voting or deliberation | ibid. |
| Repeated sampling scales **log-linearly** — 4→8 buys what 2→4 bought, at twice the cost — and converts to performance only where results are automatically verifiable, which analysis findings are not | [arXiv 2407.21787](https://arxiv.org/abs/2407.21787) |
| Multi-agent failures: **specification and system design 41.8%**, driven substantially by duplicate or ambiguous roles; **inter-agent misalignment 36.9%**, avoided by construction when agents do not talk | [arXiv 2503.13657](https://arxiv.org/abs/2503.13657) |
| 44 question types in four categories. The fourth — **integrating across models**, "does this break assumptions elsewhere" — was asked by none of the eight lenses, and is now the `integration` lens | [Sillito et al., FSE 2006](https://www.cs.ubc.ca/~murphy/papers/other/asking-answering-fse06.pdf) |

**This argued the expansion from four lenses to eight was poor value, and §3
was cut back to five in response.** The lever is lens distinctness, not count,
and role duplication is the largest documented failure category. What it
supports strongly is what was already here: independent passes, unioned, never
voted on.

### A.7 Sources

Architecture and scaffolding: [mini-swe-agent](https://github.com/swe-agent/mini-swe-agent) ·
[Inside the Scaffold, arXiv 2604.03515](https://arxiv.org/abs/2604.03515) ·
[Stop Comparing LLM Agents Without Disclosing the Harness, arXiv 2605.23950](https://arxiv.org/pdf/2605.23950) ·
[Holistic Agent Leaderboard, arXiv 2510.11977](https://arxiv.org/abs/2510.11977) ·
[Scale SWE-bench Pro](https://labs.scale.com/leaderboard/swe_bench_pro_public) ·
[Terminal-Bench 2.0](https://www.tbench.ai/leaderboard/terminal-bench/2.0)

Multi-agent: [Single-Agent Under Equal Thinking Budgets, arXiv 2604.02460](https://arxiv.org/pdf/2604.02460) ·
[The Illusion of Multi-Agent Advantage, arXiv 2606.13003](https://arxiv.org/html/2606.13003) ·
[Agent Frameworks on Code-centric SE Tasks, arXiv 2511.00872](https://arxiv.org/html/2511.00872v1) ·
[Agents that Matter, arXiv 2605.27621](https://arxiv.org/html/2605.27621) ·
[Why Do Multi-Agent LLM Systems Fail? arXiv 2503.13657](https://arxiv.org/abs/2503.13657) ·
[MetaGPT, arXiv 2308.00352](https://arxiv.org/html/2308.00352v6) ·
[ChatDev, arXiv 2307.07924](https://arxiv.org/html/2307.07924v5) ·
[CodeR, arXiv 2406.01304](https://arxiv.org/abs/2406.01304) ·
[MAGIS, arXiv 2403.17927](https://arxiv.org/pdf/2403.17927)

Sampling and selection: [Reasoning in Token Economies, arXiv 2406.06461](https://arxiv.org/pdf/2406.06461) ·
[The Cost of Consensus, arXiv 2605.00914](https://arxiv.org/pdf/2605.00914) ·
[Wisdom and Delusion of LLM Ensembles, arXiv 2510.21513](https://arxiv.org/html/2510.21513) ·
[Scaling Test-Time Compute for Agentic Coding, arXiv 2604.16529](https://arxiv.org/abs/2604.16529)

Verification: [LLMs Cannot Self-Correct Reasoning Yet, arXiv 2310.01798](https://arxiv.org/abs/2310.01798) ·
[Is Self-Repair a Silver Bullet? arXiv 2306.09896](https://arxiv.org/pdf/2306.09896) ·
[How Many Tries Does It Take? arXiv 2604.10508](https://arxiv.org/abs/2604.10508) ·
[Static Analysis as a Feedback Loop, arXiv 2508.14419](https://arxiv.org/abs/2508.14419) ·
[Bias in the Loop, arXiv 2604.16790](https://arxiv.org/html/2604.16790v1) ·
[Do LLMs generate test oracles that capture expected behaviour? arXiv 2410.21136](https://arxiv.org/abs/2410.21136) ·
[Coverage and Mutation Scores Replicability Study, arXiv 2607.22880](https://arxiv.org/abs/2607.22880) ·
[Mutation-Guided Test Generation at Meta, arXiv 2501.12862](https://arxiv.org/pdf/2501.12862) ·
[Lemur, arXiv 2310.04870 (ICLR 2024)](https://arxiv.org/html/2310.04870)

Localization and edits: [SWE-Explore, arXiv 2606.07297](https://arxiv.org/html/2606.07297) ·
[CORE-Bench, arXiv 2606.11864](https://arxiv.org/html/2606.11864v2) ·
[To Diff or Not to Diff? arXiv 2604.27296](https://arxiv.org/html/2604.27296)

Long-horizon: [SWE-EVO, arXiv 2512.18470](https://arxiv.org/html/2512.18470v5) ·
[RoadmapBench, arXiv 2605.15846](https://arxiv.org/html/2605.15846v1) ·
[SlopCodeBench, arXiv 2603.24755](https://arxiv.org/html/2603.24755v1) ·
[METR Time Horizons](https://metr.org/time-horizons/)

Reward hacking and open-source impact: [ImpossibleBench, arXiv 2510.20270](https://arxiv.org/abs/2510.20270) ·
[Cursor, Reward hacking is swamping model intelligence gains](https://cursor.com/blog/reward-hacking-coding-benchmarks) ·
[Agentic Coding PRs on GitHub, arXiv 2509.14745](https://arxiv.org/html/2509.14745v3) ·
[How Humans Review AI-Generated PRs, arXiv 2605.02273](https://arxiv.org/html/2605.02273v1) ·
[GitClear Maintainability Gap](https://www.gitclear.com/the_ai_code_quality_maintainability_gap) ·
[METR developer productivity RCT](https://metr.org/blog/2025-07-10-early-2025-ai-experienced-os-dev-study/)
