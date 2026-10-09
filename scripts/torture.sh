#!/usr/bin/env bash
# Mandatory executable regression baseline; extend normal test discovery as features land.
set -euo pipefail
cd "$(dirname "$0")/.."
./scripts/verify.sh
./scripts/verify-phase.sh release
