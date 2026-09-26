#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
python3 scripts/symphony/prepare-docker-host.py
docker compose build
project="mtg-lab-smoke-$$"
cleanup() { docker compose -p "$project" down -v --remove-orphans >/dev/null; }
trap cleanup EXIT
docker compose -p "$project" run --rm -T --no-deps symphony bash -s <<'CHECK'
set -euo pipefail
test "$(id -u)" != 0
test ! -e /var/run/docker.sock
for tool in cargo rustc java javac mvn python3 git gh codex; do command -v "$tool"; done
java -version
javac -version
codex --version
codex sandbox -- /bin/echo nested-sandbox-ok
codex sandbox -- python3 -c 'import errno; from pathlib import Path
try:
    Path("/home/agent/forbidden-write").write_text("bad")
except OSError as e:
    assert e.errno in (errno.EROFS, errno.EACCES, errno.EPERM)
else:
    raise SystemExit("Read-only sandbox allowed a write")'
# Invalid test token cannot authorize issue dispatch; dashboard startup needs no secret.
GITHUB_TOKEN=smoke-no-credential symphony --i-understand-that-this-will-be-running-without-the-usual-guardrails --logs-root /tmp/smoke-logs /opt/mtg-lab/WORKFLOW.md >/tmp/controller.log 2>&1 &
controller_pid=$!
trap 'kill "$controller_pid" 2>/dev/null || true; wait "$controller_pid" 2>/dev/null || true' EXIT
python3 - <<'PYCODE'
import json, time, urllib.request
for attempt in range(30):
    try:
        state = json.load(urllib.request.urlopen('http://127.0.0.1:4318/api/v1/state', timeout=1))
        assert state['counts']['running'] == 0
        print('credential-free-controller-ok')
        break
    except OSError:
        time.sleep(1)
else:
    raise SystemExit('Controller did not become healthy')
PYCODE
CHECK
