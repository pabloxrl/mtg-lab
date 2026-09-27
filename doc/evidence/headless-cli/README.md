# GH-79 headless command acceptance

Atomic R0002-B020/B039/B041 command wiring; #21 retains cross-feature acceptance.
No full catalog cases were assigned. All nine `headless_commands` subprocess tests
are part of normal Cargo discovery. Every process closes stdin, removes display
variables, captures machine stdout/stderr and has an external 15-second deadline.

Independent expected outcomes derive from RFC §9's unattended/error contract,
CR 103.4/103.5 opening counts/life, CR 103.8a/504.1/704.5b passive empty-draw loss,
the preexisting handwritten JSONL fixture, and the existing CR 117.3d neutral
priority fixture. Conformance test observations are explicitly synthetic
comparator inputs, not claimed engine/reference results. No engine output is
used to author an expected outcome.

[Initial red](red.txt): all seven original tests compiled and failed against the
unchanged simulate-only CLI, before command implementation. [Initial green
attempt](green-initial.txt): six passed; the seventh exposed an author mistake in
the new test, not an engine defect. It incorrectly asserted three decisions for
the preexisting one-action recorder fixture. [Independent read-only Codex review](expectation-review.txt)
explicitly approved changing this assertion to one based on the handwritten
fixture, its footer/seal, prior evidence document and an independent SHA-256
recomputation. The correction was made only after this review. Added stronger
coverage for a tampered seal count and the known empty-stream SHA-256 with zero
episodes/decisions; existing malformed/missing-seal assertions retained. An
initial empty fixture's noncanonical key order was fixed without changing its
expected behavior.

[Green named contract run](green.txt) and [all CLI tests](cli-tests.txt) retain
replay verification/seat redaction, private-error redaction, missing replay choices,
canonical trajectory counts/integrity/incompleteness, strict checkpoint/script
comparison and retained diff artifacts, missing Python, unavailable suites/references,
malformed configuration, file/output errors and explicit scalar benchmark accounting.
Additional resource-limit and real subprocess-timeout tests exercise oversize files,
smoke budget caps and a deliberately unresponsive dependency (never a fake passing
reference). Production comparison calls the existing strict comparator; production
benchmark calls the existing scalar runner. No game-rule code was changed.

README impact: new supported command row, executable container-inner smoke and
trajectory examples, updated replay/recorder CLI limitations. The milestone table
is unchanged. No M1 completion, reference agreement or M2 performance claim.
See [command contract](../../headless-commands.md) for exact scope and limits.

Full torture, quickstart, independent candidate review and protected merge/main CI
receipts are recorded in the issue workpad and PR; local test logs accompany this
report. Failed reviewer runtime initialization is never counted as approval.

Local full-suite verification: [torture](torture.txt) passed 132 Python tests,
199 Rust tests plus one doctest in **each** debug/release mode, documentation,
program/catalog, formatting and Clippy checks. Fresh `origin/main` remained
`d32df1a2fbf907c6b4fd1045b6d2fb2d82cf395f`; ff-only integration was already up to
date. [Required post-main run](torture-current-main.txt) repeats the whole suite.
Both ran directly in the managed Docker worker; no worker Docker invocation,
installation, host mount or policy change. [Benchmark README example](benchmark-smoke.json)
records two rules-terminal passive games and no truncations/errors; [trajectory
README example](trajectory-example.json) validates exactly one episode/decision.
The raw timing is merely a scalar command smoke, not a benchmark qualification.
