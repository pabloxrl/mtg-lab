# Captured CLI component acceptance

Issue #179 owns partial R0002-B020/B039/B041 only. #120 retains EVERY composed
CLI clause, #21 integration/catalog acceptance, and #22 the M1 gate. No prior
owner, catalog assertion or RFC block changes. This report is candidate evidence;
delivery additionally requires independent review, protected merge and exact-main
CI linked from the issue workpad/PR.

## Independent expectations and real execution

Normal-discovery `crates/mtg-cli/tests/captured_simulate.rs` invokes the delivered
binary with stdin closed/display unset. Real script and both-seat native games
publish through the actual owner, recorder, registry and publisher; existing
manifest/v2/replay libraries reload their artifacts. No fake future verifier is
used. The original #178 seed-178 ordered-green script remains unchanged: 101
choices plus concession, life [20,15], hands [5,7], libraries [31,31], winner P0.
CR103/305/601/608/508–510 and 104.3a justify the original land/Cub/Growth/combat
ledger. Its existing independently reviewed arithmetic/correction evidence is
retained in [script acceptance](../script-cli/README.md), not regenerated here.

`capture_tests.rs` compares every persisted canonical field with the actual owned
trajectory, each script record with the literal input, and once-only [1,-1]
returns. Full replay/capture-on/off normalized snapshots include RNG and every
semantic field; only documented process-local scope/store namespaces are removed.
Native equivalence is metamorphic evidence, not an independent rules oracle.
The original scripted ledger supplies independent checkpoints; all prior native,
script and collector regressions remain mandatory and unchanged.

## Negative boundaries

Explicit synthetic faults supplement those actual played artifacts:

- Zero/overflow/count limits, same/ancestor/alias/missing roots and denied owner
  declarations fail before output/storage mutation. Queue and byte exhaustion
  report failed publication without changing a genuine game winner.
- Publisher create/write/flush/sync/hard-link/unlink/directory-sync and withdrawal
  faults retain truthful failed/uncertain publication. The publisher uses
  no-replace hard links, not rename; collision tests preserve prior bytes.
- Missing/corrupt replay and corrupted JSONL fail existing strict readers. A replay
  ID alone cannot resolve or read a replay; exact owner grants are in memory only.
- Gameplay decision/record/work bounds and injected deadlines/signals preserve
  owner completed/truncated/failed/incomplete and requested/not-started accounting.
  Diagnostic manifests remain rejected by completed-only loaders.
- Publication stops before/during commit are distinct from game outcomes. Failed
  withdrawal is uncertain; after the final observed commit boundary, success is
  retained. No I/O-preemption or SIGKILL acknowledgment guarantee is claimed.

[Compiled behavioral red](behavioral-red.log): two public tests failed because
capture configuration was rejected (exit 2 instead of expected success/storage
failure). An earlier test `Read` type error was corrected before this retained
run and is not red evidence. [Flush behavioral red](flush-red.log): the real
publisher lacked an observable flush boundary; a compiled assertion detected it
before the hook was added. Runtime hook failures are explicitly synthetic; normal
publication artifacts are library-produced, not handwritten successes.

## Scope and validation

CLI wiring preserves canonical v2 records and existing manifest-last publication.
No rules, policy, schema, registry authority, CI/workflow/security policy, review
tooling, sampling/sharding, or task registration changes. Root README and
[simulation usage](../../simulate.md#canonical-capture-schemas-2-and-3) describe
actual optional capture, limits and local owner authorization. Previous pass,
native/script and verification commands retain their contracts.

Full torture, quickstart and independent candidate-review receipts are recorded
in the workpad/PR; completion is conditional on all delivery gates. Artifact
limits are per file plus explicit episode/record/queue counts, not total RSS/disk
quotas. Deadline checks are cooperative at operation/filesystem boundaries.
The run summary alone accounts never-started requests; manifests describe started
results only. Aggregate CLI/M1 acceptance remains pending.

## Independent review finding and provenance repair

[First candidate review](review-1.json) requested changes on `ec1c294`: canonical
headers copied the driver's internal one-tick deadline sentinel as a configured
one-millisecond limit. [Compiled provenance red](provenance-red.log) independently
asserts disabled deadlines are `None`; it failed with `Some(1)`. The corrected
run header and Driver budget use the actual `deadline_ms`, and expiry publishes
that configured threshold to the injected clock. No deadline is recorded when
disabled. The new regression checks manifest AND episode metadata for no deadline
and an explicitly specified 70,000 ms, plus an explicitly specified 20,000-choice
limit. These values are declared inputs, not implementation-generated oracles.

Two prior injected-clock tests in `native.rs` and `script_tests.rs` now explicitly
configure seven milliseconds only in their Deadline cases. Their previous null
configuration plus injected Deadline was impossible through production `main.rs`,
whose control emits Deadline only when `config.deadline_ms.is_some_and(...)`.
Every prior expected status/count/reason and assertion remains intact; signal
cases still have no configured deadline. This aligns the test trigger with the
existing public requirement rather than weakening a budget outcome. The next
prescribed independent candidate review must check this correction and the new
stronger provenance coverage explicitly. No rule behavior changes.
