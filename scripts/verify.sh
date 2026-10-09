#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
./scripts/verify-phase.sh checks
./scripts/verify-phase.sh debug
