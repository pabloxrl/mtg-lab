#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
phase="${1:-all}"
case "$phase" in
  all|checks|debug|release) ;;
  runtime) exec ./scripts/symphony/runtime-smoke.sh ;;
  *) echo "Usage: $0 [all|checks|debug|release|runtime]" >&2; exit 2 ;;
esac
docker build --target toolchain -f docker/Dockerfile -t mtg-lab-toolchain:local .
docker_args=(--rm --cap-drop ALL --security-opt no-new-privileges)
if [[ -n "${MTG_VERIFY_CACHE:-}" ]]; then
  case "$MTG_VERIFY_CACHE" in
    /*) ;;
    *) echo "MTG_VERIFY_CACHE must be an absolute directory" >&2; exit 2 ;;
  esac
  mkdir -p "$MTG_VERIFY_CACHE"/{target,registry,git}
  # Cache artifacts only: never mount or cache authentication or the agent home.
  docker run --rm --user 0 --cap-drop ALL --cap-add CHOWN --security-opt no-new-privileges \
    --mount "type=bind,source=$MTG_VERIFY_CACHE,target=/cache" \
    mtg-lab-toolchain:local chown -R 1001:1001 /cache
  docker_args+=(
    --mount "type=bind,source=$MTG_VERIFY_CACHE/target,target=/tmp/target"
    --mount "type=bind,source=$MTG_VERIFY_CACHE/registry,target=/home/agent/.cargo/registry"
    --mount "type=bind,source=$MTG_VERIFY_CACHE/git,target=/home/agent/.cargo/git"
  )
fi
if [[ "$phase" == all ]]; then
  command=(./scripts/torture.sh)
else
  command=(./scripts/verify-phase.sh "$phase")
fi
docker run "${docker_args[@]}" \
  --mount "type=bind,source=$PWD,target=/workspace,readonly" --workdir /workspace \
  -e CARGO_TARGET_DIR=/tmp/target -e PYTHONDONTWRITEBYTECODE=1 \
  -e GIT_CONFIG_COUNT=1 -e GIT_CONFIG_KEY_0=safe.directory -e GIT_CONFIG_VALUE_0=/workspace \
  mtg-lab-toolchain:local "${command[@]}"
if [[ "$phase" == all ]]; then
  ./scripts/symphony/runtime-smoke.sh
fi
