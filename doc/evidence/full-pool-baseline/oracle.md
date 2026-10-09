# Independent expectations and red receipts

Requirements are RFC 0002 B008/B014/B019/B020/B021, the #80/#217 partial
allocation, and original SYS-PERF-001/002/003 and SYS-METRIC-002 expectations.
This artifact does not certify those whole blocks or the M2 gate.

The arithmetic tests use authored literal data: five 30-second intervals, sixty
natural completions per interval, thirty wins for each seat and one unfinished
attempt. The expected rate is 60/30 = 2 completed games/second; 600/30 = 20
decisions/second. Five intervals total 300 completions. The declared rotation has
eight rows, so each interval's row-attempt counts follow its first ordinal and
attempt count, independently of engine outcomes. Completion counts must sum to
wins plus draws and never include unfinished/truncated/failed/conceded games.
The literal memory example adds 1,024 OS bytes for each resident state, so its
10,000-state first-cycle slope is 1,024 bytes/state. Linux proc `123 kB` is
125,952 bytes; serialized JSON size is not an OS resident-memory oracle.

The tests first ran against executable no-op contract surfaces. The retained
`artifact-*-red.log` files contain assertion failures for accepted omissions,
contamination, horizon changes, inconsistent pairs and absent memory points.
Positive synthetic arithmetic/classification inputs test the verifier only;
they never supply measured game or stress evidence. The Linux subprocess tests
exercise real exit 7 and a killed timeout; failed processes cannot become success.

`rust-red.log` records successful compilation followed by the unsupported
full-pool workload assertion and incorrect RSS-unit conversion assertion. Its
initial stress-classification test failed too early because the Driver still had
internal work. That setup failure is **not** evidence for the classifier. The
corrected test drains internal work before inspecting its observation; its
`stress-red.log` then records its intended 0-versus-4 token assertion, the
missing probe callback, and the omitted six-point sweep assertion. `strict-request-red.log.gz` additionally preserves the later compiled failure
for a capture request with an unknown field. Serde's internally tagged unit
variant accepted that field; an empty struct variant now rejects it. The test
keeps the exact exit-2 expectation while avoiding a huge debug dump on failure.
No engine output was adopted as an
expected rules outcome, and no existing assertion was removed or weakened.

The old `prerequisite-gap.md`, `audit.json` and `activation-mana-red.patch` are
historical diagnostics from base `522b7da`. The exact three tests are delivered
by [repair PR #260](https://github.com/pabloxrl/mtg-lab/pull/260), merge
`9a4e8812ecf9fc019cb2c76812058840da89a543`, with
[passing exact-main CI](https://github.com/pabloxrl/mtg-lab/actions/runs/37956881953).
They are preserved as history, not a current gap or a new execution receipt.

Correctness qualification comes from the executable suite, applicable pinned
reference receipts and independent review. Native policy agreement, repeated
snapshots, rates and this arithmetic verifier are not independent Magic rules
oracles. Measurements, full torture, review and delivery remain pending until
recorded in the final report.

The existing native client checks its deadline before reset. One final scheduled
attempt can therefore start no game. The validator reports `attempts - started`
separately (at most one such boundary attempt per complete interval), keeps its
time and ordinal, and still requires started = terminal + failed + unfinished +
truncated + conceded. `artifact-not-started-red.log` preserves the initially too
strict validator rejection; a new literal regression retains the unstarted case
and rejects fewer attempts than started games. No old test was removed or changed.

`pending-reset-red.log` records the reset-churn probe rejecting a valid snapshot
with internal work still pending. The existing core correctly rejects such a
reset. The diagnostic now drains bounded work through `Game::resume` before
resetting, reports those settling calls, and retains the original pending-state
test and its corruption check. This does not invent a policy choice or complete
a game; settling/reset are separate from the resident-position measurement.

Repair diagnosis: the failures involved diagnostic client assumptions (request
strictness, deadline-before-reset accounting, and reset timing), not new Magic
semantics. Keep all assertions, use the existing Driver/work boundaries, and
validate the six focused contracts before full torture and any measurements.
No scope expansion, test waiver or rules workaround is justified by these failures.
