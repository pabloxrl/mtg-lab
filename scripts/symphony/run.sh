#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
export PATH="$HOME/.local/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin"
runtime="$HOME/.local/share/mtg-lab-symphony"
export SYMPHONY_CONTROL_ROOT="$PWD"
mkdir -p "$runtime/logs"
# Retrieve the host credential without storing it in the repository or plist.
export GITHUB_TOKEN
GITHUB_TOKEN="$(gh auth token --hostname github.com)"
exec "$runtime/bin/symphony-v0.0.3-macos_arm64" \
  --i-understand-that-this-will-be-running-without-the-usual-guardrails \
  --logs-root "$runtime/logs" "$PWD/WORKFLOW.md"
