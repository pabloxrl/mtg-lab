# Verification execution changes — operations #237

This change accelerates execution without removing test cases or relaxing their
assertions. Delivery still requires independent review, protected merge and
successful exact-main CI; local evidence alone is not completion of #237.

## What is preserved

All documentation/program/design checks, Python discovery, formatting, Clippy,
checked Rust tests, release Rust tests and runtime/sandbox/controller/dashboard
smoke checks remain mandatory. The default Docker entry point executes the full
contract. CI runs four partitions concurrently and the existing required `verify`
name is an aggregate that fails unless the whole matrix succeeds. No path filters
or documentation-only skips are introduced; no timeout is increased.

The test profile changes compiler optimization from O0 to O1. Debug information,
debug assertions (`cfg(debug_assertions)`) and integer-overflow checks remain
explicitly enabled. Development and release profiles are unchanged. This preserves
test selection and safety checks, not the previous unoptimized compiler setting.

## Local validation, 2026-10-09

- Nine focused entry-point tests verify the complete command inventory, partition
  equivalence, failure propagation, invalid inputs, cache mount boundaries,
  mandatory profile checks and the aggregate's failure/cancel/skip behavior.
- Complete cold Docker verification passed for runtime candidate `9a6b6ad`:
  181 Python tests; 621 Rust test executions in each mode; formatting, Clippy,
  sandbox, credential-free controller and dashboard checks all passed.
- The test-name/outcome multiset matches the original main CI log in **both**
  Rust modes: 621 executions, including subprocess executions, with no missing,
  added or ignored Rust cases. Baseline: main `2e0d830`,
  [CI attempt 1](https://github.com/pabloxrl/mtg-lab/actions/runs/37867323250/attempts/1).
- The later `31c9403` delta changes only cache keys to include Cargo manifests;
  independent review passed. It does not alter test execution.
- The complete local serial run took approximately 629 seconds, measured from
  fresh log creation to final smoke-cleanup output. This includes cold Rust
  compilation; the Docker toolchain image was already available locally.

Selected test-binary timings from sequential local runs on the same Mac/Colima
Linux ARM64 setup (single samples, not controlled benchmark statistics):

| Existing test binary | O0 test execution | Checked O1 test execution |
| --- | ---: | ---: |
| `native_simulate` (13 tests) | 250.25 s | 10.26 s |
| `script_simulate` (8 tests) | 45.02 s | 0.38 s |
| `composed_cli` (5 tests) | 19.66 s | 3.05 s |

Cold test compilation increased from 29.88 seconds to about 121 seconds in these
runs; O1 exchanges compile time for faster execution. The O0 local full run was
stopped before completion once profiling justified validating O1. No full O0
local pass or like-for-like full-suite speedup is claimed from that run.

Cached general checks also passed (180 Python tests before the additional profile
guard, cached Clippy 0.33 seconds). One initial macOS bind-mounted-cache full build
reported a compiler output permission error; the successful full run used the
original disposable container target. Full cached-suite success on macOS is not
claimed. Linux CI cache behavior and parallel wall time require the live run.

## Played-game example

The unchanged [lethal script](../../../fixtures/simulate/script-lethal-v3.json)
was executed through the real CLI. Two ordered green decks keep seven. The
starting player plays Forests, casts Bear Cub, then resolves Giant Growth and
attacks unblocked for five damage on turn 5. Eight later unblocked attacks deal
two each. The defender loses through the rules at life -1, without concession.

The run reported 434 decisions and 434 consumed script records, life `[20,-1]`,
hands `[8,7]`, libraries `[23,23]`, one completed episode, and zero failed,
truncated or incomplete episodes. This is a deterministic regression scenario,
not a claim about learned-policy strength. The independently specified ledger
and executable coverage remain in the [script acceptance report](../script-cli/README.md).
