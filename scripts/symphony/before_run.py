"""Bound retries across Symphony sessions without changing the upstream scheduler."""
import json
from pathlib import Path
import re
import subprocess
import time

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
