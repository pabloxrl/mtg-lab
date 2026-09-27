#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
python3 scripts/check_docs.py
python3 scripts/check_program.py
python3 scripts/test_plan.py
python3 -m unittest discover -s tests -p 'test_*.py'
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
