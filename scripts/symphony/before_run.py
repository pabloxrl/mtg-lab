"""Check execution prerequisites and retain diagnostic dispatch history."""
import json
import math
import os
from pathlib import Path
import re
import subprocess
import time

# Fresh clones do not inherit the control checkout's repository-local identity.
# Reapply on every dispatch, including workspaces created before this policy.
subprocess.run(["git", "config", "--local", "user.name", "pablo ribalta"], check=True)
subprocess.run(["git", "config", "--local", "user.email", "pabloxrl@gmail.com"], check=True)

if os.environ.get("MTG_CONTAINER") != "1" or not Path("/.dockerenv").exists():
    raise SystemExit("Agent delivery requires the managed Docker runtime")

# Read the authoritative runtime gate from current main, not a candidate branch.
subprocess.run(["git", "fetch", "origin", "main"], check=True)
program = json.loads(subprocess.check_output(
    ["git", "show", "origin/main:doc/programs/rfc-0002.json"], text=True))
execution = program.get("execution")
if execution:
    if execution.get("kind") != "docker":
        raise SystemExit("Unsupported execution contract")
    prerequisite = execution["prerequisite_issue"]
    match = re.fullmatch(r"GH-(\d+)", Path.cwd().name)
    current = int(match.group(1)) if match else None
    registered = {task["issue"] for task in program["tasks"]}
    if current in registered and current != prerequisite:
        gate = json.loads(subprocess.check_output(
            ["gh", "api", f"repos/pabloxrl/mtg-lab/issues/{prerequisite}"], text=True))
        if gate["state"] != "closed" or gate.get("state_reason") != "completed":
            raise SystemExit(f"Execution prerequisite #{prerequisite} is incomplete")

state_path = Path(".symphony-attempts.json")
# Legacy started/attempts fields are telemetry only, never dispatch authority.
# Reject malformed telemetry visibly without rewriting evidence or labels.
try:
    state = json.loads(state_path.read_text()) if state_path.exists() else {
        "started": time.time(), "attempts": 0}
    if (not isinstance(state, dict)
            or type(state.get("attempts")) is not int or state["attempts"] < 0
            or type(state.get("started")) not in (int, float)
            or not math.isfinite(state["started"]) or state["started"] < 0):
        raise ValueError("expected nonnegative attempts and finite started timestamp")
except (ValueError, OverflowError) as error:
    raise SystemExit(f"Invalid dispatch diagnostics in {state_path}: {error}") from error
state["attempts"] += 1
# Avoid leaving truncated diagnostics if the worker stops during the write.
temporary = state_path.with_suffix(".json.tmp")
temporary.write_text(json.dumps(state) + "\n")
temporary.replace(state_path)
print(f"Symphony dispatch attempt {state['attempts']} (diagnostic only)")
