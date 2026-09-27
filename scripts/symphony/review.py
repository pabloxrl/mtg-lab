"""Run an independent read-only Codex review; fail on findings or tool failure."""
import json
from pathlib import Path
import subprocess
import sys

root = Path(__file__).resolve().parents[2]
if subprocess.check_output(["git", "status", "--porcelain"], cwd=root, text=True).strip():
    raise SystemExit("Commit the complete candidate before review; working tree must be clean.")
base = sys.argv[1] if len(sys.argv) > 1 else "origin/main"
base_sha = subprocess.check_output(["git", "rev-parse", "--verify", base], cwd=root, text=True).strip()
head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip()
artifacts = root / ".agent-artifacts"
artifacts.mkdir(exist_ok=True)
output = artifacts / "review.json"
output.unlink(missing_ok=True)
prompt = f"""Independently review this repository's candidate against base {base_sha}.
Inspect git diff {base_sha}...{head}. The candidate is committed and clean.
Read AGENTS.md and relevant RFC requirements. Do not edit files, use network,
publish comments, or invoke another review. Find concrete correctness, verification,
security, and integration defects. Tests are executable evidence, not proof of all
requirements. Inspect test deletions/skips/weakened assertions or rebaselined
expectations for independently justified requirement corrections and equivalent
or stronger coverage. Check README accuracy and atomic task/integration ownership.
Do not invent findings or demand unrelated scope. Return structured
findings; changes_requested if any blocking finding, otherwise pass.
"""
with (artifacts / "review-events.jsonl").open("w") as events, (artifacts / "review-stderr.log").open("w") as errors:
    result = subprocess.run([
        "codex", "exec", "--ignore-user-config", "--ephemeral", "--sandbox", "read-only",
        "--json", "--output-schema", str(root / "scripts/symphony/review-schema.json"),
        "--output-last-message", str(output), prompt,
    ], cwd=root, stdout=events, stderr=errors, timeout=900)
if result.returncode != 0 or not output.is_file():
    raise SystemExit("Reviewer failed; inspect .agent-artifacts/review-stderr.log")
review = json.loads(output.read_text())
if subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip() != head or subprocess.check_output(["git", "status", "--porcelain"], cwd=root, text=True).strip():
    output.unlink()
    raise SystemExit("Candidate changed during review; commit and run review again.")
review["base_sha"] = base_sha
review["head_sha"] = head
output.write_text(json.dumps(review, indent=2) + "\n")
print(json.dumps(review, indent=2))
if review.get("verdict") != "pass" or any(f.get("severity") == "blocking" for f in review["findings"]):
    raise SystemExit(1)
