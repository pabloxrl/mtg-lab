# Executable torture and regression baseline

Tests are the repository's delivery foundation. From the host run:

```sh
./scripts/verify-docker.sh
```

The Docker toolchain runs `./scripts/torture.sh`: documentation, lossless program
and design validation, all discovered Python `unittest` tests, Rust formatting,
Clippy and all Rust workspace tests in debug **and release**. The host wrapper
also runs the isolated runtime smoke. Required PR/main CI uses the same wrapper in four parallel partitions.
Agents already inside the managed container run `./scripts/torture.sh` directly.
`verify.sh` is the constituent debug/general check, not the full delivery command.
Python discovery rejects empty execution, skips and expected failures as success.
Rust tests must not be ignored or selectively filtered for delivery; reviewers
inspect exemptions and required acceptance explicitly.

The suite includes verifier/schema/corpus/program/tooling regressions, the
synthetic checkpoint comparator, implemented rules tests, native-policy games,
and scripted full-game capture/publication/replay checks. `tests/test_torture.py`
adds fixed seeded nested-difference, mutation-detection, ordering, type and
presence adversaries; those specific tests exercise the verifier rather than
Magic rules. The [capability catalog](../testing/README.md) distinguishes designs
from implemented execution. Full-pool qualification, long fuzz campaigns,
trainers and the complete dual-reference matrix are separate delivery gates;
passing this suite does not claim those future capabilities.

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

## Parallel CI and caches

Operations #237 partitions the existing checks into `checks`, `debug`, `release`
and `runtime` jobs. Run a partition using `./scripts/verify-docker.sh debug` (or
one of the other names); omit the argument to run the complete local contract.
Every PR and main commit still runs all four partitions. The test profile uses optimization level 1 with debug information, debug
assertions and integer-overflow checks explicitly enabled. The separate release
profile is unchanged. No test, assertion, fixture, or reference requirement is
removed. Matrix fail-fast is disabled so one failure does not cancel the other
coverage. The required `verify` job succeeds only when the entire matrix succeeds;
a failed, cancelled, skipped or missing matrix result fails the aggregate gate.

CI caches only Cargo registry sources, Git dependency sources and compiled target
artifacts under the runner's temporary directory. Keys separate OS, architecture,
verification partition and pinned toolchain/dependency inputs. Commit-specific
entries restore from the compatible prefix; tests execute on every run, including
cache hits. No credentials, agent home, workpads or prior test results are cached.
Cold-cache runs execute the same checks. GitHub caches are an optimization, not
verification evidence.

For local reuse, set `MTG_VERIFY_CACHE` to an absolute dedicated cache directory.
The wrapper prepares its ownership for the container's UID 1001; do not point it
at another application's data. Without this variable, the original disposable
build behavior is retained. Source is still mounted read-only.

The 30-minute limit applies independently to each partition, including runtime
smoke checks. Parallelism removes serial waiting but does not reduce the total
number of checks. Debug game/replay tests may still dominate elapsed time; further
speed claims require measured runs. Full default verification remains mandatory
for delivery. The runtime image and smoke contract are unchanged.

The checked test profile changes compiler optimization, not test selection. It
retains `cfg(debug_assertions)` behavior and overflow panics, unlike release.
The default development profile is unchanged. Cargo profile settings are
specified explicitly in the workspace manifest; see the
[Cargo profile reference](https://doc.rust-lang.org/cargo/reference/profiles.html).
