#!/usr/bin/env bash
# CI partitions of the full verification contract; no test filters or exclusions.
set -euo pipefail
cd "$(dirname "$0")/.."
case "${1:-}" in
  checks)
    python3 scripts/check_docs.py
    python3 scripts/check_program.py
    python3 scripts/test_plan.py
    python3 scripts/run_tests.py
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets --locked -- -D warnings
    ;;
  debug) cargo test --workspace --locked ;;
  release) cargo test --workspace --release --locked ;;
  *) echo "Usage: $0 {checks|debug|release}" >&2; exit 2 ;;
esac
