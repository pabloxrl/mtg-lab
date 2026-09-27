# Executable torture and regression baseline

Tests are the repository's delivery foundation. From the host run:

```sh
./scripts/verify-docker.sh
```

The Docker toolchain runs `./scripts/torture.sh`: documentation, lossless program
and design validation, all discovered Python `unittest` tests, Rust formatting,
Clippy and all Rust workspace tests in debug **and release**. The host wrapper
also runs the isolated runtime smoke. Required PR/main CI uses this same wrapper.
Agents already inside the managed container run `./scripts/torture.sh` directly.
`verify.sh` is the constituent debug/general check, not the full delivery command.
Python discovery rejects empty execution, skips and expected failures as success.
Rust tests must not be ignored or selectively filtered for delivery; reviewers
inspect exemptions and required acceptance explicitly.

Today this executes verifier/schema/corpus/program/tooling regressions and the
synthetic Rust checkpoint comparator. `tests/test_torture.py` adds fixed seeded
nested-difference, mutation-detection, ordering, type and presence adversaries.
They exercise the verifier, **not AI matches or implemented Magic rules**.
The 320 catalog designs remain designs until their owners implement actual tests.
The required wrapper does not run full games, long fuzz campaigns, trainers or
the dual-reference matrix that are not yet implemented. No empty placeholder
for those capabilities reports success.

For each delivered behavior, add positive, rejected-input and boundary/interaction
checks to normal discovery (`tests/test_*.py` or Rust unit/integration tests), using
the assigned stable catalog/system IDs where applicable. First demonstrate an
assertion failing for the intended behavioral reason, not an import/build error.
Expected results come from pinned rules/contracts, hand-derived fixtures or an
independent oracle; never approve output just because the current engine emits it.
For every real defect, keep the minimized input or replay and reproducible seed
as a permanent regression alongside its fix. A stochastic search supplements
these fixed regressions; it does not replace them. Run the entire suite after
changes and integration, then obtain independent review and exact-main CI.

The body of protected behavior must grow as features and defects accumulate.
Raw test counts are not a quality quota. Deleting, skipping, relaxing or replacing
an assertion needs independent review of the requirement correction and explicit
equivalent or stronger coverage. A failed expected result is investigated, not
regenerated from the implementation. Supported mandatory cases cannot be waived.
Known unsupported future behavior stays visible in the design catalog, without
fake passing tests or CI-success skips.

As real games become available, retain admitted AI/heuristic-generated traces
with deterministic semantic decisions and chance tapes, independently checked
checkpoints and minimized failures. Reference alignment is governed by
[game-replay.md](game-replay.md); same seeds alone do not align Forge and XMage.
Every rule change runs full supported conformance and its impacted reference
checks. Long fuzz/mutation, trainer and reference campaigns use dedicated jobs;
release qualification still requires the exact-candidate full matrix. Add those
jobs when their capabilities land; this baseline does not claim their coverage.

[Atomic deliveries](../programs/atomic-delivery.md) assign implementation and
integration checks without requiring a foundational task to implement its own
future dependents. The driver gives desired outcomes; agents retain the tests,
review and merge changes, and report only essential decisions or missing access.
