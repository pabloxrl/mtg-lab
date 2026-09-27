"""Start the pinned controller using credentials provisioned into its private volume."""
import os
from pathlib import Path
import subprocess

home = Path.home()
if not Path("/.dockerenv").exists():
    raise SystemExit("Use Docker Compose; native delivery is disabled")
if not (home / ".codex/auth.json").is_file():
    raise SystemExit("Missing Codex authentication: run docker bootstrap first")
# gh stores its runtime credential in the volume, never in the image or Compose.
os.environ["GITHUB_TOKEN"] = subprocess.check_output(
    ["gh", "auth", "token", "--hostname", "github.com"], text=True).strip()
runtime = home / ".local/share/mtg-lab-symphony"
runtime.mkdir(parents=True, exist_ok=True)
os.chdir("/opt/mtg-lab")
os.execvp("python3", ["python3", "/opt/mtg-lab/scripts/symphony/dashboard_runtime.py",
                        "--logs-root", str(runtime / "logs"), "WORKFLOW.md"])
