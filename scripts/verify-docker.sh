#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
docker build --target toolchain -f docker/Dockerfile -t mtg-lab-toolchain:local .
docker run --rm --cap-drop ALL --security-opt no-new-privileges \
  --mount "type=bind,source=$PWD,target=/workspace,readonly" --workdir /workspace \
  -e CARGO_TARGET_DIR=/tmp/target -e PYTHONDONTWRITEBYTECODE=1 \
  -e GIT_CONFIG_COUNT=1 -e GIT_CONFIG_KEY_0=safe.directory -e GIT_CONFIG_VALUE_0=/workspace \
  mtg-lab-toolchain:local ./scripts/verify.sh
