"""Bound retries across Symphony sessions without changing the upstream scheduler."""
import json
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
state = json.loads(state_path.read_text()) if state_path.exists() else {"started": time.time(), "attempts": 0}
state["attempts"] += 1
state_path.write_text(json.dumps(state) + "\n")
if state["attempts"] > 5 or time.time() - state["started"] > 5400:
    match = re.fullmatch(r"GH-(\d+)", Path.cwd().name)
    if not match:
        raise SystemExit("Retry limit reached; cannot identify issue from workspace name")
    number = match.group(1)
    subprocess.run(["gh", "issue", "comment", number, "-R", "pabloxrl/mtg-lab", "--body",
                    "Symphony paused this issue after 5 dispatch attempts or a 90-minute work window. "
                    "Evidence and branch are retained. See the operations guide to reset and resume."], check=True)
    subprocess.run(["gh", "issue", "edit", number, "-R", "pabloxrl/mtg-lab", "--add-label",
                    "agent-blocked", "--remove-label", "agent-running", "--remove-label", "agent-ready"], check=True)
    raise SystemExit("Issue paused at retry/time limit")
print(f"Symphony dispatch attempt {state['attempts']}/5")
